// SPDX-License-Identifier: Apache-2.0
#include <hip/hip_runtime.h>
extern "C" __global__ void debug_threads(int* maxtid, int* threads, int* lane_max, int gx) {
    int wg = blockIdx.y * gx + blockIdx.x;
    int tid = threadIdx.x + threadIdx.y * 32 + threadIdx.z * 1024;
    atomicMax(&maxtid[wg], tid);
    atomicAdd(&threads[wg], 1);
    atomicMax(&lane_max[wg], (int)__lane_id());
}
