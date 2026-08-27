// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
//
// Does explicit multi-stream concurrency still map to multiple hardware queues?
//
// Why this matters, and why it is a different question from the graph probe:
//   Profiling shows ROCm 10.0 places every dispatch of a hipGraph on a single
//   hardware queue, whatever dependency structure the graph declares, whereas
//   7.14 spread concurrent shapes evenly across the four-queue pool. That leaves
//   the important practical question open: did 10.0 remove the runtime's ability
//   to run work concurrently, or only stop hipGraph from asking for it?
//
//   The distinction decides what a consumer can do about it. If explicit
//   streams still fan out across queues, an engine can recover the concurrency
//   by managing streams itself instead of relying on graph node independence. If
//   explicit streams also collapse onto one queue, the capability is gone and
//   there is no application-side workaround.
//
//   This probe therefore does not use graphs at all. It launches N kernels
//   round-robin across M explicitly created streams and synchronises at the end,
//   which is the plainest possible request for concurrency through the public
//   API.
//
// What to look at:
//   * the per-dispatch cost as M grows -- if concurrency works, cost should fall
//   * the hardware-queue distribution under rocprofv3, which is the direct
//     evidence; run with --kernel-trace and read dispatch_info.queue_id
//
// The kernel is deliberately NOT a no-op here: it spins for a controllable
// number of iterations. A no-op kernel finishes before concurrency can be
// observed, so it would make every stream count look identical regardless of
// whether the work actually overlapped. `--spin` sets the per-kernel work.
//
// Correctness is gated: the counter must equal exactly N * replays, so a stream
// configuration cannot look fast by dropping launches.
//
// Build: hipcc --offload-arch=gfxNNNN -O3 explicit_multistream.cpp -o ems -ldl
// Run:   ./ems [N] [streams] [replays] [--spin=K]

#include <hip/hip_runtime.h>
#include <dlfcn.h>
#include <algorithm>
#include <chrono>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <vector>

#define OK(x)                                                                    \
    do {                                                                         \
        hipError_t _e = (x);                                                     \
        if (_e != hipSuccess) {                                                  \
            printf("FATAL %s -> %s (%d) at line %d\n", #x, hipGetErrorString(_e), \
                   (int)_e, __LINE__);                                           \
            exit(1);                                                             \
        }                                                                        \
    } while (0)

// Same provenance reporting as the other probes in this directory: a
// side-by-side ROCm install makes it easy to measure the wrong runtime, and the
// only defence is for the number to carry proof of its own origin.
static void print_runtime_provenance() {
    int rt = -1;
    hipRuntimeGetVersion(&rt);
    Dl_info info{};
    const char* who = "(unresolved)";
    if (dladdr((void*)&hipRuntimeGetVersion, &info) && info.dli_fname)
        who = info.dli_fname;
    printf("runtime: hipRuntimeGetVersion=%d  libamdhip64=%s\n", rt, who);
}

// Spins for `iters` clock ticks so that concurrent execution is observable.
__global__ void spin(unsigned long long* counter, unsigned long long iters) {
    const unsigned long long t0 = wall_clock64();
    unsigned long long acc = 0;
    while (wall_clock64() - t0 < iters) acc += wall_clock64() & 0xf;
    if (threadIdx.x == 0 && blockIdx.x == 0) atomicAdd(counter, 1ull + (acc & 0ull));
}

static double median(std::vector<double>& v) {
    if (v.empty()) return 0.0;
    std::sort(v.begin(), v.end());
    const size_t m = v.size() / 2;
    return (v.size() & 1) ? v[m] : 0.5 * (v[m - 1] + v[m]);
}

int main(int argc, char** argv) {
    unsigned long long spin_ticks = 2000;
    std::vector<const char*> pos;
    for (int i = 1; i < argc; ++i) {
        if (strncmp(argv[i], "--spin=", 7) == 0)
            spin_ticks = strtoull(argv[i] + 7, nullptr, 10);
        else
            pos.push_back(argv[i]);
    }
    const int n = pos.size() > 0 ? atoi(pos[0]) : 256;
    const int nstreams = pos.size() > 1 ? atoi(pos[1]) : 4;
    const int replays = pos.size() > 2 ? atoi(pos[2]) : 30;

    OK(hipSetDevice(0));
    hipDeviceProp_t p;
    OK(hipGetDeviceProperties(&p, 0));
    printf("=== %s (%s) ===\n", p.name, p.gcnArchName);
    print_runtime_provenance();
    printf("N=%d launches round-robin over %d stream(s), %d replays, spin=%llu ticks\n",
           n, nstreams, replays, spin_ticks);
    printf("No graphs involved: plain hipLaunchKernelGGL on explicit streams.\n\n");

    unsigned long long* d = nullptr;
    OK(hipMalloc(&d, sizeof(unsigned long long)));

    std::vector<hipStream_t> streams(nstreams);
    for (int i = 0; i < nstreams; ++i)
        OK(hipStreamCreateWithFlags(&streams[i], hipStreamNonBlocking));

    // Warm up so first-touch costs are not counted.
    for (int i = 0; i < n; ++i)
        hipLaunchKernelGGL(spin, dim3(1), dim3(64), 0, streams[i % nstreams], d,
                           spin_ticks);
    OK(hipDeviceSynchronize());
    OK(hipMemset(d, 0, sizeof(unsigned long long)));
    OK(hipDeviceSynchronize());

    std::vector<double> per;
    per.reserve(replays);
    for (int r = 0; r < replays; ++r) {
        const auto t0 = std::chrono::steady_clock::now();
        for (int i = 0; i < n; ++i)
            hipLaunchKernelGGL(spin, dim3(1), dim3(64), 0, streams[i % nstreams], d,
                               spin_ticks);
        OK(hipDeviceSynchronize());
        const auto t1 = std::chrono::steady_clock::now();
        per.push_back(std::chrono::duration<double, std::micro>(t1 - t0).count() / n);
    }

    unsigned long long got = 0;
    OK(hipMemcpy(&got, d, sizeof(got), hipMemcpyDeviceToHost));
    const unsigned long long want =
        (unsigned long long)n * (unsigned long long)replays;

    for (auto& s : streams) OK(hipStreamDestroy(s));
    OK(hipFree(d));

    printf("  streams=%-4d  %8.3f us/launch   gate %s\n", nstreams, median(per),
           got == want ? "ok" : "COUNTER MISMATCH");
    if (got != want)
        printf("  counted %llu, expected %llu -- timing above is not valid\n", got,
               want);
    printf("\nIf concurrency is working, us/launch falls as streams rise. Confirm\n"
           "with rocprofv3 --kernel-trace and read the queue_id distribution: that\n"
           "is the runtime's own account of whether the work actually fanned out.\n");
    return got == want ? 0 : 2;
}
