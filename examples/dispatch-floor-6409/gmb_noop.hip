// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
//
// The exact hipEngine #6409 narrow kernel (gmb_noop_kernel): each dispatch adds
// 1.0f to every in-range element, so replaying it `count` times drives every
// element to exactly `count` (the correctness basis). Uses blockIdx/blockDim/
// threadIdx — firmware-provided, so no hidden kernargs are required; this is a
// good real-engine-kernel test of Redline's PM4 dispatch.
//
// Build for Redline:  hipcc --genco --offload-arch=gfx1201 gmb_noop.hip -o gmb_noop.co
// Dispatched symbol:  gmb_noop_kernel.kd

#include <hip/hip_runtime.h>

extern "C" __global__ void gmb_noop_kernel(float *__restrict__ out, int n) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < n) {
        out[idx] = out[idx] + 1.0f;
    }
}
