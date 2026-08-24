// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
//
// Can a hardware queue be released while it still has work in flight, using
// nothing but public HIP calls?
//
// Why this probe exists:
//   ROCm/ROCm#6529 reports intermittent address-zero SQC-data VM faults on
//   gfx1100, observed during retained-PM4 integration testing, escalating
//   through "MES failed to respond to msg=REMOVE_QUEUE" into a MODE1 device
//   reset. Our own gfx1100 logged 78 such faults on 2026-07-23 with the same
//   fault status word (0x00801431), the same client (SQC (data) (0xa)), the same
//   address (0x0), and the same escalation chain -- one burst per process
//   launch, across 8 launches, not deep inside a long run.
//
//   A REMOVE_QUEUE that fails is a queue being torn down while it is not
//   actually idle. rocm-systems#8113 ("fix(clr): use tracker-owned signal for
//   queue idle") describes precisely that defect: CLR cached a raw
//   hsa_signal_t for queue-idle detection, that handle could be recycled or
//   destroyed, and the review of that PR states the consequence plainly --
//   IsQueueIdle() can report idle while a submitted packet is still in flight,
//   "enabling ReleaseHwQueue() to release a non-idle queue". The fix also resets
//   queue-progress state when switching or reassigning hardware queues, which
//   says where the exposure is worst.
//
//   #8113 merged to develop on 2026-07-06 and reached release/therock-7.14 only
//   via cherry-pick rocm-systems#10005 on 2026-08-13. Our fleet's libamdhip64
//   was installed 2026-07-09, so the faults above were logged on a runtime
//   without the fix.
//
//   If that is the mechanism, then PM4 is not required to reach it, and this
//   probe tries to reach it from public HIP alone. That distinction matters: it
//   decides whether #6529 is a property of retained PM4 or a CLR queue-lifetime
//   defect that PM4 workloads merely expose sooner by keeping a queue busy far
//   longer than an AQL packet stream does.
//
// How it provokes the path:
//   1. More concurrent streams than GPU_MAX_HW_QUEUES (default 4), so CLR must
//      reassign pooled hardware queues between streams rather than give each its
//      own.
//   2. A kernel that stays resident for a controllable duration, so work is
//      genuinely in flight rather than already retired.
//   3. hipStreamDestroy called WITHOUT synchronising first. This is legal HIP --
//      the stream is destroyed once its work completes -- and it is the call
//      that routes into releaseQueue while the queue is still busy.
//
// What a result means:
//   Faults are logged by the kernel, not by this process, so this probe cannot
//   report them. It prints what it did and exits; the caller must check
//   `journalctl -k` for VM_L2_PROTECTION_FAULT / SQC / REMOVE_QUEUE lines.
//   A clean run does NOT mean the defect is absent -- the upstream report says
//   it reproduced only on specific systems and could not be captured in a unit
//   test. Absence of a fault here is weak evidence; presence is strong.
//
// THIS PROBE CAN WEDGE OR RESET THE GPU. It is designed to stress a queue
// lifetime path that upstream documents as a use-after-free. Do not run it on a
// machine doing anything you care about, or on a display-attached GPU.
//
// Build: hipcc --offload-arch=gfx1100 -O2 queue_release_busy.cpp -o queue_release_busy
// Run:   ./queue_release_busy [streams] [cycles] [spin_us]

#include <hip/hip_runtime.h>
#include <cstdio>
#include <cstdlib>
#include <vector>

#define OK(x)                                                                    \
    do {                                                                         \
        hipError_t _e = (x);                                                     \
        if (_e != hipSuccess) {                                                  \
            printf("FATAL %s -> %s (%d) at line %d\n", #x, hipGetErrorString(_e), \
                   (int)_e, __LINE__);                                           \
            return 3;                                                            \
        }                                                                        \
    } while (0)

// Stays resident for approximately `us` microseconds so that a stream destroyed
// immediately after launch is destroyed while this is still executing. Uses
// wall_clock64 where available; the exact duration does not matter, only that it
// is long enough to still be running when hipStreamDestroy is called.
__global__ void resident(unsigned long long* sink, unsigned long long us,
                         unsigned long long ticks_per_us) {
    const unsigned long long target = us * ticks_per_us;
    const unsigned long long t0 = wall_clock64();
    unsigned long long acc = 0;
    while (wall_clock64() - t0 < target) {
        // Keep the wave doing real work so it cannot be optimised away and so
        // it holds its scalar state live across the whole span.
        acc += wall_clock64() & 0xff;
    }
    if (threadIdx.x == 0 && blockIdx.x == 0) atomicAdd(sink, acc | 1ull);
}

int main(int argc, char** argv) {
    const int streams = argc > 1 ? atoi(argv[1]) : 32;
    const int cycles = argc > 2 ? atoi(argv[2]) : 200;
    const unsigned long long spin_us = argc > 3 ? strtoull(argv[3], nullptr, 10) : 200;

    OK(hipSetDevice(0));
    hipDeviceProp_t p;
    OK(hipGetDeviceProperties(&p, 0));

    // wall_clock64 ticks at a fixed rate; derive it so spin_us is meaningful.
    int clk_khz = 0;
    if (hipDeviceGetAttribute(&clk_khz, hipDeviceAttributeWallClockRate, 0) != hipSuccess ||
        clk_khz <= 0) {
        clk_khz = 100000;  // 100 MHz fallback; duration precision is not critical
    }
    const unsigned long long ticks_per_us = (unsigned long long)clk_khz / 1000ull;

    printf("=== %s (%s) ===\n", p.name, p.gcnArchName);
    printf("streams %d (GPU_MAX_HW_QUEUES defaults to 4, so this forces queue\n"
           "reassignment), cycles %d, resident kernel ~%llu us, wall clock %d kHz\n",
           streams, cycles, spin_us, clk_khz);
    printf("Destroying each stream WITHOUT synchronising, while its kernel is\n"
           "still resident. Faults, if any, appear in journalctl -k, not here.\n\n");

    unsigned long long* sink = nullptr;
    OK(hipMalloc(&sink, sizeof(unsigned long long)));
    OK(hipMemset(sink, 0, sizeof(unsigned long long)));

    int destroyed = 0;
    for (int c = 0; c < cycles; ++c) {
        std::vector<hipStream_t> s(streams, nullptr);

        // Create more streams than the hardware-queue pool holds, and put
        // long-running work on every one of them.
        for (int i = 0; i < streams; ++i) {
            OK(hipStreamCreateWithFlags(&s[i], hipStreamNonBlocking));
            hipLaunchKernelGGL(resident, dim3(1), dim3(64), 0, s[i], sink, spin_us,
                               ticks_per_us);
            hipError_t le = hipGetLastError();
            if (le != hipSuccess) {
                printf("cycle %d stream %d: launch failed: %s (%d)\n", c, i,
                       hipGetErrorString(le), (int)le);
                return 3;
            }
        }

        // Tear every stream down with its kernel still resident. No sync here on
        // purpose: this is the call that reaches releaseQueue with the queue
        // still busy.
        for (int i = 0; i < streams; ++i) {
            hipError_t de = hipStreamDestroy(s[i]);
            if (de != hipSuccess) {
                printf("cycle %d stream %d: hipStreamDestroy -> %s (%d)\n", c, i,
                       hipGetErrorString(de), (int)de);
                return 3;
            }
            ++destroyed;
        }

        if (((c + 1) % 50) == 0) {
            printf("  cycle %d/%d, %d streams destroyed while busy\n", c + 1, cycles,
                   destroyed);
            fflush(stdout);
        }
    }

    // Only now drain, so any deferred teardown has to resolve against real work.
    OK(hipDeviceSynchronize());

    unsigned long long got = 0;
    OK(hipMemcpy(&got, sink, sizeof(got), hipMemcpyDeviceToHost));
    OK(hipFree(sink));

    printf("\ncompleted: %d streams created and destroyed while their kernel was\n"
           "resident, sink=%llu (nonzero means kernels really ran)\n", destroyed, got);
    printf("Now check the kernel log for this window:\n"
           "  journalctl -k --since \"5 min ago\" | grep -E "
           "'VM_L2_PROTECTION_FAULT|SQC|REMOVE_QUEUE|reset'\n");
    return 0;
}
