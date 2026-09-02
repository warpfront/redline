// SPDX-License-Identifier: Apache-2.0
#include <hip/hip_runtime.h>
extern "C" __global__ void debug_grid(int* out, int* hits, int* counter, int gx) {
    if (threadIdx.x != 0 || threadIdx.y != 0 || threadIdx.z != 0) return;
    int x = blockIdx.x;
    int y = blockIdx.y;
    int idx = atomicAdd(counter, 1);
    if (idx < 4096) out[idx] = (y << 16) | x;
    atomicAdd(&hits[y * gx + x], 1);
}
