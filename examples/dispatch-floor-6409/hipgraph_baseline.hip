// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
//
// Faithful reproduction of hipEngine's #6409 hip_graph serial_latency floor:
// capture `count` gmb_noop_kernel launches into a HIP graph
// (hipStreamBeginCapture, relaxed), instantiate, and replay with hipGraphLaunch,
// timed with hipEvents (median of REPS after WARMUP). Same kernel, block=256,
// shared-buffer dependency chain, correctness = every element == count.
//
//   hipcc --offload-arch=gfx1201 hipgraph_baseline.hip -o hipgraph_baseline
//   ./hipgraph_baseline <count> <n> <reps> <warmup>   # prints one result line
#include <hip/hip_runtime.h>
#include <algorithm>
#include <chrono>
#include <cstdio>
#include <cstdlib>
#include <vector>

extern "C" __global__ void gmb_noop_kernel(float *__restrict__ out, int n) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < n) {
        out[idx] = out[idx] + 1.0f;
    }
}

#define CK(x)                                                                      \
    do {                                                                           \
        hipError_t _e = (x);                                                       \
        if (_e != hipSuccess) {                                                    \
            fprintf(stderr, "hip: %s\n", hipGetErrorString(_e));                   \
            return 2;                                                              \
        }                                                                          \
    } while (0)

int main(int argc, char **argv) {
    int count = argc > 1 ? atoi(argv[1]) : 941;
    int n = argc > 2 ? atoi(argv[2]) : 256;
    int reps = argc > 3 ? atoi(argv[3]) : 50;
    int warmup = argc > 4 ? atoi(argv[4]) : 10;
    const int block = 256;
    const int grid = (n + block - 1) / block;

    float *out = nullptr;
    CK(hipMalloc(&out, (size_t)n * sizeof(float)));
    hipStream_t stream;
    CK(hipStreamCreateWithFlags(&stream, hipStreamNonBlocking));

    /* capture the burst graph: `count` launches, shared buffer (dependency chain) */
    CK(hipMemsetAsync(out, 0, (size_t)n * sizeof(float), stream));
    CK(hipStreamSynchronize(stream));
    CK(hipStreamBeginCapture(stream, hipStreamCaptureModeRelaxed));
    for (int i = 0; i < count; i++) {
        gmb_noop_kernel<<<grid, block, 0, stream>>>(out, n);
    }
    hipGraph_t graph;
    CK(hipStreamEndCapture(stream, &graph));
    hipGraphExec_t exec;
    CK(hipGraphInstantiate(&exec, graph, nullptr, nullptr, 0));

    for (int i = 0; i < warmup; i++) CK(hipGraphLaunch(exec, stream));
    CK(hipStreamSynchronize(stream));

    /* timed: per-replay GPU (hipEvent) + host (steady_clock), median of reps */
    hipEvent_t es, ee;
    CK(hipEventCreate(&es));
    CK(hipEventCreate(&ee));
    std::vector<double> gpu_us, host_us;
    gpu_us.reserve(reps);
    host_us.reserve(reps);
    for (int r = 0; r < reps; r++) {
        auto t0 = std::chrono::high_resolution_clock::now();
        CK(hipEventRecord(es, stream));
        CK(hipGraphLaunch(exec, stream));
        CK(hipEventRecord(ee, stream));
        CK(hipEventSynchronize(ee));
        auto t1 = std::chrono::high_resolution_clock::now();
        float ms = 0.0f;
        CK(hipEventElapsedTime(&ms, es, ee));
        gpu_us.push_back((double)ms * 1000.0);
        host_us.push_back(std::chrono::duration<double, std::micro>(t1 - t0).count());
    }

    /* correctness: one clean replay from zero -> every element == count.
     * Zero and launch on the SAME stream so they serialize (a default-stream
     * memset races into a non-blocking stream's graph). */
    CK(hipMemsetAsync(out, 0, (size_t)n * sizeof(float), stream));
    CK(hipGraphLaunch(exec, stream));
    CK(hipStreamSynchronize(stream));
    std::vector<float> host(n);
    CK(hipMemcpy(host.data(), out, (size_t)n * sizeof(float), hipMemcpyDeviceToHost));
    int correct = 1;
    for (int i = 0; i < n; i++)
        if (host[i] != (float)count) { correct = 0; break; }

    std::sort(gpu_us.begin(), gpu_us.end());
    std::sort(host_us.begin(), host_us.end());
    double g = gpu_us[reps / 2], h = host_us[reps / 2];
    printf("count=%d n=%d grid=%d gpu_us=%.3f host_us=%.3f gpu_per_disp=%.4f host_per_disp=%.4f correct=%d\n",
           count, n, grid, g, h, g / count, h / count, correct);
    return correct ? 0 : 1;
}
