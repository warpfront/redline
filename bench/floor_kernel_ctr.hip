// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Kaden Schutt
//
// Correctness-gate kernel: each dispatch atomically increments a counter, so a
// PM4 IB of N dispatches, replayed once, must leave the counter == N iff every
// dispatch actually executed AND the replay's completion waited for wave
// retirement. Build:
//   hipcc --genco --offload-arch=gfx1201 floor_kernel_ctr.hip -o floor_kernel_ctr.co
// Dispatched symbol: "ctr_k.kd".

#include <hip/hip_runtime.h>

extern "C" __global__ void ctr_k(unsigned int* p) {
    atomicAdd(p, 1u);
}
