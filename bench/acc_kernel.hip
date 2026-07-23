// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
//
// Decode-shaped correctness kernel. Each dispatch adds a per-token scalar `val`
// (carried in the kernarg segment) into an accumulator. Build ONE retained PM4
// IB, then per token patch only the 4-byte `val` in place via
// rl_pm4_ib_set_kernargs / Pm4Ib.set_kernargs and replay: the accumulator must
// end at sum(val_t) iff each replay observed its patched scalar with no rebuild.
// Build:
//   hipcc --genco --offload-arch=gfx1201 acc_kernel.hip -o acc_kernel.co
// Dispatched symbol: "acc_k.kd". Kernarg layout: [acc:u64 @0][val:u32 @8].

#include <hip/hip_runtime.h>

extern "C" __global__ void acc_k(unsigned int* acc, unsigned int val) {
    atomicAdd(acc, val);
}
