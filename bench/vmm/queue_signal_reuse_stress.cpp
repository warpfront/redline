// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
//
// Recreate-per-cycle queue / signal / allocation reuse stress for
// ROCm/ROCm#6529 (gfx1100 address-zero SQC-data VM fault).
//
// The leading hypothesis on that issue is queue / completion-signal /
// GPU-visible-allocation retirement and reuse, not the packed-dot math and not
// routine preemption (mid-IB preemption was tested separately and came back
// clean through ~8400 quantum-switch opportunities).
//
// This program deliberately contains **no Redline and no retained PM4**. It is
// pure HIP. That is the point: the single highest-value unknown on #6529 is
// whether the fault can occur with no retained-PM4 replay in the process at all.
// If it faults here, ownership sits with ROCr/CLR/KFD rather than with a
// retained-PM4 submitter. If it stays clean, that narrows the search back toward
// the retained-IB path.
//
// Each cycle: create a stream (a hardware queue underneath), allocate device
// memory, create events, dispatch a real kernel that READS device memory (the
// observed fault client is SQC (data), so a shader data fetch has to be in the
// loop), wait, then destroy the stream, destroy the events and free the memory
// so the next cycle reuses those addresses, doorbells and signal slots. Every
// status is checked and reported loudly.
//
// SAFETY: refuses to run on anything except gfx1100 unless --force is passed.
// A device reset triggered here takes that GPU's VRAM with it, so it must never
// be aimed at a shared APU by accident.
//
// Build:
//   hipcc --offload-arch=gfx1100 -O2 queue_signal_reuse_stress.cpp -o qsrs
// Run (pin the card first; check the kernel log afterwards):
//   ROCR_VISIBLE_DEVICES=<idx> ./qsrs [cycles] [streams_per_cycle] [--force]
//   journalctl -k --since "10 min ago" | grep -E 'VM_L2_PROTECTION_FAULT|SQC|REMOVE_QUEUE|GPU reset'
#include <hip/hip_runtime.h>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <string>
#include <vector>

static unsigned long long g_fail = 0;

#define CK(expr, what)                                                             \
    do {                                                                           \
        hipError_t _e = (expr);                                                    \
        if (_e != hipSuccess) {                                                    \
            printf("  [FAIL] cycle %llu %s: %s\n", cycle, what,                    \
                   hipGetErrorString(_e));                                         \
            ++g_fail;                                                              \
        }                                                                          \
    } while (0)

// Reads device memory (SQC data path) and writes a dependent result.
__global__ void chase(const int* in, int* out, int n, int add) {
    int i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i < n) {
        int v = in[i];
        out[i] = v + add;
    }
}

int main(int argc, char** argv) {
    unsigned long long cycles = 20000;
    int per_cycle = 4;
    bool force = false;
    std::vector<std::string> pos;
    for (int i = 1; i < argc; ++i) {
        if (std::strcmp(argv[i], "--force") == 0) force = true;
        else pos.push_back(argv[i]);
    }
    if (pos.size() > 0) cycles = strtoull(pos[0].c_str(), nullptr, 10);
    if (pos.size() > 1) per_cycle = atoi(pos[1].c_str());

    int ndev = 0;
    if (hipGetDeviceCount(&ndev) != hipSuccess || ndev == 0) {
        printf("no visible devices\n");
        return 1;
    }
    hipDeviceProp_t prop;
    if (hipGetDeviceProperties(&prop, 0) != hipSuccess) return 1;

    printf("device 0 of %d visible: %s (%s)\n", ndev, prop.name, prop.gcnArchName);
    if (std::string(prop.gcnArchName).find("gfx1100") == std::string::npos && !force) {
        printf("refusing: this stress is for gfx1100 (ROCm/ROCm#6529) and can provoke a\n"
               "device reset. Pin the right card with ROCR_VISIBLE_DEVICES, or pass\n"
               "--force if you really mean this one.\n");
        return 1;
    }
    if (ndev != 1) {
        printf("refusing: %d devices visible. Pin exactly one with ROCR_VISIBLE_DEVICES so a\n"
               "reset cannot land on a device you did not intend.\n", ndev);
        return 1;
    }

    const int N = 4096;
    const size_t bytes = N * sizeof(int);
    printf("cycles %llu, %d streams/cycle, %d ints per buffer\n\n", cycles, per_cycle, N);

    unsigned long long cycle = 0;
    unsigned long long dispatches = 0;
    for (cycle = 1; cycle <= cycles; ++cycle) {
        // Fresh queues, signals and allocations every cycle, then released, so the
        // next cycle reuses the same rings, doorbells, signal slots and addresses.
        std::vector<hipStream_t> streams(per_cycle, nullptr);
        std::vector<hipEvent_t> evs(per_cycle, nullptr);
        std::vector<int*> in(per_cycle, nullptr), out(per_cycle, nullptr);

        for (int s = 0; s < per_cycle; ++s) {
            CK(hipStreamCreateWithFlags(&streams[s], hipStreamNonBlocking), "stream create");
            CK(hipEventCreateWithFlags(&evs[s], hipEventDisableTiming), "event create");
            CK(hipMalloc(&in[s], bytes), "malloc in");
            CK(hipMalloc(&out[s], bytes), "malloc out");
            if (in[s]) CK(hipMemsetAsync(in[s], s + 1, bytes, streams[s]), "memset");
        }
        for (int s = 0; s < per_cycle; ++s) {
            if (!streams[s] || !in[s] || !out[s]) continue;
            chase<<<(N + 255) / 256, 256, 0, streams[s]>>>(in[s], out[s], N, (int)cycle);
            hipError_t le = hipGetLastError();
            if (le != hipSuccess) {
                printf("  [FAIL] cycle %llu launch: %s\n", cycle, hipGetErrorString(le));
                ++g_fail;
            }
            ++dispatches;
            CK(hipEventRecord(evs[s], streams[s]), "event record");
        }
        for (int s = 0; s < per_cycle; ++s) {
            if (evs[s]) CK(hipEventSynchronize(evs[s]), "event sync");
        }
        // Retire everything in an order that maximises reuse pressure: destroy the
        // queue first, then the signal it referenced, then the memory the packets
        // pointed at.
        for (int s = 0; s < per_cycle; ++s) {
            if (streams[s]) CK(hipStreamDestroy(streams[s]), "stream destroy");
            if (evs[s]) CK(hipEventDestroy(evs[s]), "event destroy");
            if (in[s]) CK(hipFree(in[s]), "free in");
            if (out[s]) CK(hipFree(out[s]), "free out");
        }

        if (cycle % 1000 == 0)
            printf("cycle %6llu: %llu dispatches, %llu failures so far\n", cycle, dispatches,
                   g_fail);
        if (g_fail > 32) {
            printf("\nstopping early: %llu failures — check the kernel log now\n", g_fail);
            return 4;
        }
    }

    printf("\n=== completed %llu cycles, %llu dispatches, %llu failures ===\n", cycles - 1,
           dispatches, g_fail);
    printf("Now check: journalctl -k --since \"15 min ago\" | grep -E "
           "'VM_L2_PROTECTION_FAULT|SQC|REMOVE_QUEUE|GPU reset'\n");
    return g_fail ? 3 : 0;
}
