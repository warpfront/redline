// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
//
// Buffer-resource form of the exact #6409 read-modify-write kernel. This keeps
// the HSA code-object container but makes the memory ABI explicit: Redline
// supplies one 128-bit SRD instead of requiring LLVM to rebuild a 64-bit flat
// address in every lane.
//
// Build:
//   hipcc --genco --offload-arch=gfx1201 gmb_buffer.hip -o gmb_buffer.co
// Symbol:
//   gmb_buffer_kernel.kd

#include <hip/hip_runtime.h>

using int32x4_t = int __attribute__((ext_vector_type(4)));

__device__ int llvm_amdgcn_raw_buffer_load_i32(
    int32x4_t rsrc,
    int voffset,
    int soffset,
    int aux) __asm("llvm.amdgcn.raw.buffer.load.i32");

__device__ void llvm_amdgcn_raw_buffer_store_i32(
    int value,
    int32x4_t rsrc,
    int voffset,
    int soffset,
    int aux) __asm("llvm.amdgcn.raw.buffer.store.i32");

extern "C" __global__ __launch_bounds__(256) void
gmb_buffer_kernel(int32x4_t out, int n) {
    const int idx = static_cast<int>(blockIdx.x) * 256 + static_cast<int>(threadIdx.x);
    if (idx < n) {
        const int byte_offset = idx * static_cast<int>(sizeof(float));
        const int bits = llvm_amdgcn_raw_buffer_load_i32(out, byte_offset, 0, 0);
        const float value = __builtin_bit_cast(float, bits) + 1.0f;
        llvm_amdgcn_raw_buffer_store_i32(
            __builtin_bit_cast(int, value), out, byte_offset, 0, 0);
    }
}
