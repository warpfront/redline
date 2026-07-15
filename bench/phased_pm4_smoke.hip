// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

#include <hip/hip_runtime.h>
#include <stdint.h>

extern "C" __global__ void phased_write(uint32_t* output, uint32_t value, uint32_t delay_ticks) {
    if (threadIdx.x != 0 || blockIdx.x != 0) return;
    const uint64_t started = clock64();
    while (clock64() - started < delay_ticks) {
        __builtin_amdgcn_s_sleep(1);
    }
    *output = value;
}

extern "C" __global__ void phased_consume(const uint32_t* input, uint32_t* output) {
    if (threadIdx.x == 0 && blockIdx.x == 0) *output = *input + 1;
}
