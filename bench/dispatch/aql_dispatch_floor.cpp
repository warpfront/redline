// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
//
// Per-dispatch cost floor for HIP on ROCm: how much does it cost to submit a
// kernel that does almost nothing?
//
// Why this exists:
//   ROCm/ROCm#6409 reports HIP losing to Vulkan at the tiny-dispatch floor, and
//   a llama.cpp maintainer measured AQL dispatch taking substantially longer
//   than raw PM4 submission, with dispatch overhead dominating decode. AMD asked
//   for a reproduction. This is the pure-HIP half: it needs no third-party
//   runtime, no PM4, no Vulkan, and no profiler, so it can be built and run
//   against a stock ROCm install in under a minute.
//
//   It deliberately measures only what HIP itself charges to get a kernel onto
//   the GPU and know it finished. The kernel is a single-workitem atomic
//   increment, which is the smallest thing that is still observable, so the
//   number is dominated by submission and completion rather than by compute.
//
// Why timing is done this way:
//   The referenced thread notes that rocprofv3 does not report dispatch times
//   usefully, so nothing here depends on a profiler. Two independent clocks are
//   reported side by side and should agree:
//     * host wall clock across a whole batch, divided by dispatch count
//     * hipEvent elapsed time across the same batch, divided by dispatch count
//   Reporting both makes a measurement artifact obvious: if they disagree, the
//   number is not trustworthy and the run says so.
//
//   Every arm is correctness-gated. The counter must land on exactly the
//   expected value, so an arm cannot look fast by failing to do the work.
//
// Arms:
//   stream-loop   N launches on one stream, one sync at the end. This is what
//                 an inference engine's decode step actually does.
//   per-launch-sync  N launches each followed by a sync. Isolates the
//                 round-trip, and is the pattern naive code uses.
//   graph-replay  the same N launches captured once into a hipGraph and
//                 replayed. This is HIP's own best answer to dispatch overhead.
//
// Build:
//   hipcc --offload-arch=$(rocminfo | awk '/gfx/{print $2; exit}') -O3 \
//       aql_dispatch_floor.cpp -o aql_dispatch_floor
// Run:
//   ./aql_dispatch_floor            # default sweep
//   ./aql_dispatch_floor 256 200 20 # N, replays, warmups

#include <hip/hip_runtime.h>
#include <algorithm>
#include <chrono>
#include <cstdio>
#include <cstdlib>
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

// Smallest kernel that is still observable: one workitem, one atomic add.
// Keeping it this small is the point -- it makes submission cost dominate.
__global__ void tick(unsigned long long* counter) {
    if (threadIdx.x == 0 && blockIdx.x == 0) atomicAdd(counter, 1ull);
}

static double median(std::vector<double>& v) {
    if (v.empty()) return 0.0;
    std::sort(v.begin(), v.end());
    const size_t m = v.size() / 2;
    return (v.size() & 1) ? v[m] : 0.5 * (v[m - 1] + v[m]);
}

struct Result {
    double host_us_per_dispatch = 0.0;
    double event_us_per_dispatch = 0.0;
    bool correct = false;
    unsigned long long counted = 0;
    unsigned long long expected = 0;
};

// Runs one arm `replays` times after `warmups` untimed passes, and returns the
// median per-dispatch cost from both clocks.
template <typename Body>
static Result time_arm(int n, int replays, int warmups, unsigned long long* d_counter,
                       Body&& body) {
    Result r;
    hipEvent_t ev_start, ev_stop;
    OK(hipEventCreate(&ev_start));
    OK(hipEventCreate(&ev_stop));

    for (int i = 0; i < warmups; ++i) body();
    OK(hipDeviceSynchronize());

    // Reset only after warmup so the gate counts exactly the timed dispatches.
    OK(hipMemset(d_counter, 0, sizeof(unsigned long long)));
    OK(hipDeviceSynchronize());

    std::vector<double> host_us, event_us;
    host_us.reserve(replays);
    event_us.reserve(replays);

    for (int i = 0; i < replays; ++i) {
        OK(hipEventRecord(ev_start, nullptr));
        const auto t0 = std::chrono::steady_clock::now();
        body();
        OK(hipDeviceSynchronize());
        const auto t1 = std::chrono::steady_clock::now();
        OK(hipEventRecord(ev_stop, nullptr));
        OK(hipEventSynchronize(ev_stop));

        float ms = 0.0f;
        OK(hipEventElapsedTime(&ms, ev_start, ev_stop));
        const double host_us_total =
            std::chrono::duration<double, std::micro>(t1 - t0).count();
        host_us.push_back(host_us_total / n);
        event_us.push_back((double)ms * 1000.0 / n);
    }

    OK(hipMemcpy(&r.counted, d_counter, sizeof(r.counted), hipMemcpyDeviceToHost));
    r.expected = (unsigned long long)n * (unsigned long long)replays;
    r.correct = (r.counted == r.expected);
    r.host_us_per_dispatch = median(host_us);
    r.event_us_per_dispatch = median(event_us);

    OK(hipEventDestroy(ev_start));
    OK(hipEventDestroy(ev_stop));
    return r;
}

static void print_row(const char* label, int n, const Result& r) {
    // Flag the two clocks disagreeing by more than 25%: that means the number is
    // an artifact of the measurement, not a property of the runtime.
    const double lo = std::min(r.host_us_per_dispatch, r.event_us_per_dispatch);
    const double hi = std::max(r.host_us_per_dispatch, r.event_us_per_dispatch);
    const bool clocks_agree = lo > 0.0 && (hi / lo) < 1.25;

    printf("  %-16s N=%-5d %8.3f %8.3f   %-7s %s\n", label, n,
           r.host_us_per_dispatch, r.event_us_per_dispatch,
           clocks_agree ? "agree" : "DIVERGE",
           r.correct ? "ok" : "COUNTER MISMATCH");
    if (!r.correct)
        printf("  %-16s   counted %llu, expected %llu -- timing above is not valid\n",
               "", r.counted, r.expected);
}

int main(int argc, char** argv) {
    const int fixed_n = argc > 1 ? atoi(argv[1]) : 0;
    const int replays = argc > 2 ? atoi(argv[2]) : 200;
    const int warmups = argc > 3 ? atoi(argv[3]) : 20;

    OK(hipSetDevice(0));
    hipDeviceProp_t p;
    OK(hipGetDeviceProperties(&p, 0));

    unsigned long long* d_counter = nullptr;
    OK(hipMalloc(&d_counter, sizeof(unsigned long long)));

    printf("=== %s (%s) ===\n", p.name, p.gcnArchName);
    printf("ROCm HIP per-dispatch floor. Kernel is one workitem doing one\n"
           "atomicAdd, so these numbers are submission and completion cost,\n"
           "not compute. No profiler involved.\n\n");
    printf("replays %d (median reported), warmups %d\n\n", replays, warmups);
    printf("  %-16s %-7s %8s %8s   %-7s %s\n", "arm", "", "host us", "event us",
           "clocks", "gate");

    std::vector<int> ns;
    if (fixed_n > 0) ns.push_back(fixed_n);
    else ns = {1, 8, 64, 256, 512};

    for (int n : ns) {
        // --- arm 1: N launches on one stream, single sync at the end ---
        Result stream_loop = time_arm(n, replays, warmups, d_counter, [&]() {
            for (int i = 0; i < n; ++i)
                hipLaunchKernelGGL(tick, dim3(1), dim3(1), 0, nullptr, d_counter);
        });
        print_row("stream-loop", n, stream_loop);

        // --- arm 2: N launches, each followed by its own sync ---
        Result per_sync = time_arm(n, replays, warmups, d_counter, [&]() {
            for (int i = 0; i < n; ++i) {
                hipLaunchKernelGGL(tick, dim3(1), dim3(1), 0, nullptr, d_counter);
                (void)hipStreamSynchronize(nullptr);
            }
        });
        print_row("per-launch-sync", n, per_sync);

        // --- arm 3: the same N launches captured once and replayed ---
        hipStream_t cap_stream;
        OK(hipStreamCreate(&cap_stream));
        OK(hipStreamBeginCapture(cap_stream, hipStreamCaptureModeGlobal));
        for (int i = 0; i < n; ++i)
            hipLaunchKernelGGL(tick, dim3(1), dim3(1), 0, cap_stream, d_counter);
        hipGraph_t graph = nullptr;
        OK(hipStreamEndCapture(cap_stream, &graph));
        hipGraphExec_t exec = nullptr;
        OK(hipGraphInstantiate(&exec, graph, nullptr, nullptr, 0));

        Result graph_replay = time_arm(n, replays, warmups, d_counter, [&]() {
            OK(hipGraphLaunch(exec, cap_stream));
            OK(hipStreamSynchronize(cap_stream));
        });
        print_row("graph-replay", n, graph_replay);

        OK(hipGraphExecDestroy(exec));
        OK(hipGraphDestroy(graph));
        OK(hipStreamDestroy(cap_stream));

        if (stream_loop.correct && graph_replay.correct &&
            graph_replay.host_us_per_dispatch > 0.0) {
            printf("  %-16s graph-replay vs stream-loop: %.2fx\n\n", "",
                   stream_loop.host_us_per_dispatch / graph_replay.host_us_per_dispatch);
        } else {
            printf("\n");
        }
    }

    OK(hipFree(d_counter));
    printf("Read the host and event columns together. They measure the same\n"
           "batch from two independent clocks; if they diverge the run says so.\n");
    return 0;
}
