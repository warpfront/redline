// SPDX-License-Identifier: Apache-2.0
#include <hip/hip_runtime.h>
extern "C" __global__ void residual_atomic(float* y, int n, int m, int gx) {
    int idx = (blockIdx.y * gx + blockIdx.x);
    if (idx >= n * gx) return;
    atomicAdd(&y[idx], 1.0f);
}
