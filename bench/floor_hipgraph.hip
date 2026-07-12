// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
//
// Literal hipGraph baseline for the dispatch-floor head-to-head.
//
// Builds a real HIP graph the way engines do: capture N chained tiny-kernel
// launches on a stream (hipStreamBeginCapture), instantiate it
// (hipGraphInstantiate), then replay with hipGraphLaunch and measure the GPU
// time per launch. This is the exact path lucebox / llama.cpp's ggml-cuda uses.
//
//   hipcc --offload-arch=gfx1201 floor_hipgraph.hip -o floor_hipgraph
//   ROCR_VISIBLE_DEVICES=3 ./floor_hipgraph <N> <M> <R> <WARMUP>
//
// N = kernel nodes in the chain, M = launches per timed batch,
// R = timed batches (median reported), WARMUP = warmup launches.

#include <hip/hip_runtime.h>
#include <algorithm>
#include <chrono>
#include <cstdio>
#include <cstdlib>
#include <vector>

extern "C" __global__ void floor_k() {
    // no-op: measures dispatch + fence overhead, matching the redline bench.
}

#define CHECK(x)                                                                   \
    do {                                                                           \
        hipError_t _e = (x);                                                       \
        if (_e != hipSuccess) {                                                    \
            fprintf(stderr, "hip error %s:%d: %s\n", __FILE__, __LINE__,           \
                    hipGetErrorString(_e));                                        \
            return 1;                                                              \
        }                                                                          \
    } while (0)

int main(int argc, char** argv) {
    int N = argc > 1 ? atoi(argv[1]) : 256;
    int M = argc > 2 ? atoi(argv[2]) : 100;
    int R = argc > 3 ? atoi(argv[3]) : 9;
    int WARMUP = argc > 4 ? atoi(argv[4]) : 30;

    hipStream_t stream;
    CHECK(hipStreamCreate(&stream));

    // Capture N tiny kernel launches into a graph (stream order => linear chain).
    CHECK(hipStreamBeginCapture(stream, hipStreamCaptureModeGlobal));
    for (int i = 0; i < N; i++) {
        floor_k<<<dim3(1), dim3(1), 0, stream>>>();
    }
    hipGraph_t graph;
    CHECK(hipStreamEndCapture(stream, &graph));

    hipGraphExec_t exec;
    CHECK(hipGraphInstantiate(&exec, graph, nullptr, nullptr, 0));

    hipEvent_t start, stop;
    CHECK(hipEventCreate(&start));
    CHECK(hipEventCreate(&stop));

    for (int i = 0; i < WARMUP; i++) {
        CHECK(hipGraphLaunch(exec, stream));
    }
    CHECK(hipStreamSynchronize(stream));

    std::vector<double> per_launch_us;
    per_launch_us.reserve(R);
    for (int r = 0; r < R; r++) {
        CHECK(hipEventRecord(start, stream));
        for (int i = 0; i < M; i++) {
            CHECK(hipGraphLaunch(exec, stream));
        }
        CHECK(hipEventRecord(stop, stream));
        CHECK(hipEventSynchronize(stop));
        float ms = 0.0f;
        CHECK(hipEventElapsedTime(&ms, start, stop));
        per_launch_us.push_back(static_cast<double>(ms) * 1000.0 / M);
    }
    std::sort(per_launch_us.begin(), per_launch_us.end());
    double med = per_launch_us[R / 2];

    printf("hipGraphLaunch: N=%d  median GPU span   = %.3f us  (%.4f us/dispatch)\n",
           N, med, med / N);

    // Host per-replay latency (launch + synchronize), matched to redline's PM4
    // submit -> wait, so the PM4-vs-hipGraph comparison is on one basis.
    std::vector<double> host_us;
    host_us.reserve(M);
    for (int i = 0; i < M; i++) {
        auto t0 = std::chrono::high_resolution_clock::now();
        CHECK(hipGraphLaunch(exec, stream));
        CHECK(hipStreamSynchronize(stream));
        auto t1 = std::chrono::high_resolution_clock::now();
        host_us.push_back(std::chrono::duration<double, std::micro>(t1 - t0).count());
    }
    std::sort(host_us.begin(), host_us.end());
    double host_med = host_us[M / 2];
    printf("hipGraphLaunch: N=%d  median host lat.  = %.3f us  (%.4f us/dispatch)\n",
           N, host_med, host_med / N);
    return 0;
}
