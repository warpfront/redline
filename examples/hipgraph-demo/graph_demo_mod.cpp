// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
//
// Module-loaded hipGraph demo: the kernel comes from hipModuleLoadData (a raw
// .hsaco), so the redline-hipgraph interposer can associate it and take the PM4
// replay path (statically-compiled __hipRegisterFunction kernels fall back to
// native HIP). Explicit kernel-node graph, instantiate once, launch M times.
//
// Env:
//   GRAPH_HSACO  path to code object (default /tmp/ctr.hsaco)
//   GRAPH_SYM    kernel symbol (default ctr_k)
//   GRAPH_N      kernel nodes (default 64)
//   GRAPH_M      timed launches (default 200)
//   GRAPH_TOPO   chain | independent (default chain)
//
// Prints: CORRECT=<bool> MEDIAN_US=<f>

#include <hip/hip_runtime.h>

#include <algorithm>
#include <chrono>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <string>
#include <vector>

#define CHECK(x)                                                                   \
    do {                                                                           \
        hipError_t _e = (x);                                                       \
        if (_e != hipSuccess) {                                                    \
            fprintf(stderr, "hip error %s:%d: %s\n", __FILE__, __LINE__,           \
                    hipGetErrorString(_e));                                        \
            std::exit(1);                                                          \
        }                                                                          \
    } while (0)

static int env_int(const char* n, int d) {
    const char* v = std::getenv(n);
    return (!v || !*v) ? d : std::atoi(v);
}
static std::string env_str(const char* n, const char* d) {
    const char* v = std::getenv(n);
    return (!v || !*v) ? std::string(d) : std::string(v);
}
static double median_us(std::vector<double>& s) {
    if (s.empty()) return 0.0;
    std::sort(s.begin(), s.end());
    size_t n = s.size();
    return (n % 2) ? s[n / 2] : 0.5 * (s[n / 2 - 1] + s[n / 2]);
}

static std::vector<char> read_file(const std::string& path) {
    FILE* f = std::fopen(path.c_str(), "rb");
    if (!f) { fprintf(stderr, "cannot open %s\n", path.c_str()); std::exit(1); }
    std::fseek(f, 0, SEEK_END);
    long sz = std::ftell(f);
    std::fseek(f, 0, SEEK_SET);
    std::vector<char> buf(static_cast<size_t>(sz));
    if (std::fread(buf.data(), 1, buf.size(), f) != buf.size()) { std::exit(1); }
    std::fclose(f);
    return buf;
}

int main() {
    const std::string hsaco = env_str("GRAPH_HSACO", "/tmp/ctr.hsaco");
    const std::string sym = env_str("GRAPH_SYM", "ctr_k");
    const int N = env_int("GRAPH_N", 64);
    const int M = env_int("GRAPH_M", 200);
    const std::string topo = env_str("GRAPH_TOPO", "chain");
    const bool chain = (topo != "independent");

    CHECK(hipSetDevice(0));

    std::vector<char> image = read_file(hsaco);
    hipModule_t module = nullptr;
    CHECK(hipModuleLoadData(&module, image.data()));
    hipFunction_t func = nullptr;
    CHECK(hipModuleGetFunction(&func, module, sym.c_str()));

    unsigned int* d_counter = nullptr;
    CHECK(hipMalloc(&d_counter, sizeof(unsigned int)));
    CHECK(hipMemset(d_counter, 0, sizeof(unsigned int)));

    hipStream_t stream = nullptr;
    CHECK(hipStreamCreate(&stream));

    void* kernel_args[] = {&d_counter};
    hipKernelNodeParams params{};
    std::memset(&params, 0, sizeof(params));
    params.func = reinterpret_cast<void*>(func);
    params.gridDim = dim3(1, 1, 1);
    params.blockDim = dim3(1, 1, 1);
    params.sharedMemBytes = 0;
    params.kernelParams = kernel_args;
    params.extra = nullptr;

    hipGraph_t graph = nullptr;
    CHECK(hipGraphCreate(&graph, 0));
    std::vector<hipGraphNode_t> nodes(static_cast<size_t>(N), nullptr);
    for (int i = 0; i < N; ++i) {
        CHECK(hipGraphAddKernelNode(&nodes[static_cast<size_t>(i)], graph, nullptr,
                                    0, &params));
        if (chain && i > 0) {
            CHECK(hipGraphAddDependencies(graph, &nodes[static_cast<size_t>(i - 1)],
                                          &nodes[static_cast<size_t>(i)], 1));
        }
    }

    hipGraphExec_t exec = nullptr;
    CHECK(hipGraphInstantiate(&exec, graph, nullptr, nullptr, 0));

    std::vector<double> samples;
    samples.reserve(static_cast<size_t>(M));
    for (int i = 0; i < M; ++i) {
        auto t0 = std::chrono::steady_clock::now();
        CHECK(hipGraphLaunch(exec, stream));
        CHECK(hipStreamSynchronize(stream));
        auto t1 = std::chrono::steady_clock::now();
        samples.push_back(std::chrono::duration<double, std::micro>(t1 - t0).count());
    }
    CHECK(hipGraphLaunch(exec, stream));
    CHECK(hipStreamSynchronize(stream));

    unsigned int host = 0;
    CHECK(hipMemcpy(&host, d_counter, sizeof(unsigned int), hipMemcpyDeviceToHost));
    unsigned long long expected =
        static_cast<unsigned long long>(N) * static_cast<unsigned long long>(M + 1);
    bool correct = (static_cast<unsigned long long>(host) == expected);

    std::printf("CORRECT=%s MEDIAN_US=%.3f\n", correct ? "true" : "false",
                median_us(samples));
    if (!correct) {
        fprintf(stderr, "mismatch: got %u expected %llu (N=%d M=%d topo=%s)\n", host,
                expected, N, M, topo.c_str());
    }
    return correct ? 0 : 2;
}
