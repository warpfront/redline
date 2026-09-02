// SPDX-License-Identifier: Apache-2.0
#include <hip/hip_runtime.h>
extern "C" __global__ void residual_like(float* y, int n, int m, int gx) {
    int x = blockIdx.x;
    int y0 = blockIdx.y;
    // Token = y * 64 + x? Not exact; just increment one element per workgroup in a pattern.
    int idx = (y0 * gx + x);
    y[idx] += 1.0f;
}
