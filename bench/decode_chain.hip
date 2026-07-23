// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
//
// Two-kernel dependent decode chain for the multi-kernel drop-in A/B. stage1
// writes an intermediate from a per-token scalar; stage2 reads that intermediate
// and accumulates. The stage1->stage2 edge requires an inter-dispatch RMW
// boundary (redline: rl_pm4_wait_rmw; hipGraph: the captured dependency). Per
// token only stage1's scalar changes, so acc must end at sum_t(2*t) = T*(T+1).
// Build:
//   hipcc --genco --offload-arch=gfx1201 decode_chain.hip -o decode_chain.co
// Symbols: "stage1.kd" (tmp@0:u64, val@8:u32), "stage2.kd" (acc@0:u64, tmp@8:u64).

#include <hip/hip_runtime.h>

extern "C" __global__ void stage1(unsigned int* tmp, unsigned int val) {
    *tmp = val * 2u;
}

extern "C" __global__ void stage2(unsigned int* acc, const unsigned int* tmp) {
    atomicAdd(acc, *tmp);
}
