// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
//
// Standalone hipGraph demo: chain of N atomic-inc kernels, instantiate once,
// launch M times. Two build modes:
//   GRAPH_MODE=explicit (default) — hipGraphAddKernelNode + hipGraphAddDependencies
//   GRAPH_MODE=capture            — hipStreamBeginCapture / launches / EndCapture
//
// Env:
//   GRAPH_N     kernel nodes in the chain (default 64)
//   GRAPH_M     timed launches (default 200)
//   GRAPH_MODE  explicit | capture
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

extern "C" __global__ void inc(unsigned int* c) { atomicAdd(c, 1u); }

// Optional second kernel for a mixed dependency chain (same effect: +1).
extern "C" __global__ void inc2(unsigned int* c) { atomicAdd(c, 1u); }

#define CHECK(x)                                                                   \
    do {                                                                           \
        hipError_t _e = (x);                                                       \
        if (_e != hipSuccess) {                                                    \
            fprintf(stderr, "hip error %s:%d: %s\n", __FILE__, __LINE__,           \
                    hipGetErrorString(_e));                                        \
            std::exit(1);                                                          \
        }                                                                          \
    } while (0)

static int env_int(const char* name, int def) {
    const char* v = std::getenv(name);
    if (!v || !*v) return def;
    return std::atoi(v);
}

static std::string env_str(const char* name, const char* def) {
    const char* v = std::getenv(name);
    if (!v || !*v) return def;
    return v;
}

static double median_us(std::vector<double>& samples) {
    if (samples.empty()) return 0.0;
    std::sort(samples.begin(), samples.end());
    const size_t n = samples.size();
    if (n % 2 == 1) return samples[n / 2];
    return 0.5 * (samples[n / 2 - 1] + samples[n / 2]);
}

static hipGraph_t build_explicit(unsigned int* d_counter, int N, bool chain) {
    hipGraph_t graph = nullptr;
    CHECK(hipGraphCreate(&graph, 0));

    void* kernel_args[] = {&d_counter};
    hipKernelNodeParams params{};
    std::memset(&params, 0, sizeof(params));
    params.blockDim = dim3(1, 1, 1);
    params.gridDim = dim3(1, 1, 1);
    params.sharedMemBytes = 0;
    params.kernelParams = kernel_args;
    params.extra = nullptr;

    std::vector<hipGraphNode_t> nodes(static_cast<size_t>(N), nullptr);
    for (int i = 0; i < N; ++i) {
        // Alternate kernels so the chain is not a single-function special case.
        params.func = (i % 2 == 0) ? reinterpret_cast<void*>(inc)
                                   : reinterpret_cast<void*>(inc2);
        CHECK(hipGraphAddKernelNode(&nodes[static_cast<size_t>(i)], graph,
                                    nullptr, 0, &params));
        if (chain && i > 0) {
            CHECK(hipGraphAddDependencies(
                graph, &nodes[static_cast<size_t>(i - 1)],
                &nodes[static_cast<size_t>(i)], 1));
        }
    }
    return graph;
}

static hipGraph_t build_capture(hipStream_t stream, unsigned int* d_counter,
                                int N) {
    CHECK(hipStreamBeginCapture(stream, hipStreamCaptureModeGlobal));
    for (int i = 0; i < N; ++i) {
        if (i % 2 == 0) {
            inc<<<dim3(1), dim3(1), 0, stream>>>(d_counter);
        } else {
            inc2<<<dim3(1), dim3(1), 0, stream>>>(d_counter);
        }
    }
    hipGraph_t graph = nullptr;
    CHECK(hipStreamEndCapture(stream, &graph));
    return graph;
}

int main() {
    const int N = env_int("GRAPH_N", 64);
    const int M = env_int("GRAPH_M", 200);
    const std::string mode = env_str("GRAPH_MODE", "explicit");
    const std::string topo = env_str("GRAPH_TOPO", "chain");

    if (N <= 0 || M <= 0) {
        fprintf(stderr, "GRAPH_N and GRAPH_M must be > 0\n");
        return 1;
    }
    if (mode != "explicit" && mode != "capture") {
        fprintf(stderr, "GRAPH_MODE must be 'explicit' or 'capture' (got %s)\n",
                mode.c_str());
        return 1;
    }

    CHECK(hipSetDevice(0));

    unsigned int* d_counter = nullptr;
    CHECK(hipMalloc(&d_counter, sizeof(unsigned int)));
    CHECK(hipMemset(d_counter, 0, sizeof(unsigned int)));

    hipStream_t stream = nullptr;
    CHECK(hipStreamCreate(&stream));

    hipGraph_t graph = nullptr;
    if (mode == "explicit") {
        graph = build_explicit(d_counter, N, topo != "independent");
    } else {
        graph = build_capture(stream, d_counter, N);
    }

    hipGraphExec_t exec = nullptr;
    CHECK(hipGraphInstantiate(&exec, graph, nullptr, nullptr, 0));

    // Timed loop: launch + sync, host wall clock.
    std::vector<double> samples;
    samples.reserve(static_cast<size_t>(M));
    for (int i = 0; i < M; ++i) {
        const auto t0 = std::chrono::steady_clock::now();
        CHECK(hipGraphLaunch(exec, stream));
        CHECK(hipStreamSynchronize(stream));
        const auto t1 = std::chrono::steady_clock::now();
        samples.push_back(
            std::chrono::duration<double, std::micro>(t1 - t0).count());
    }

    // One more launch after the timed loop for the correctness check.
    CHECK(hipGraphLaunch(exec, stream));
    CHECK(hipStreamSynchronize(stream));

    unsigned int host = 0;
    CHECK(hipMemcpy(&host, d_counter, sizeof(unsigned int),
                    hipMemcpyDeviceToHost));

    // M timed launches + 1 verify launch, each adds N.
    const unsigned long long expected =
        static_cast<unsigned long long>(N) *
        static_cast<unsigned long long>(M + 1);
    const bool correct = (static_cast<unsigned long long>(host) == expected);
    const double med = median_us(samples);

    std::printf("CORRECT=%s MEDIAN_US=%.3f\n", correct ? "true" : "false", med);
    if (!correct) {
        std::fprintf(stderr,
                     "counter mismatch: got %u expected %llu (N=%d M=%d mode=%s)\n",
                     host, expected, N, M, mode.c_str());
    }

    CHECK(hipGraphExecDestroy(exec));
    CHECK(hipGraphDestroy(graph));
    CHECK(hipStreamDestroy(stream));
    CHECK(hipFree(d_counter));

    return correct ? 0 : 2;
}
