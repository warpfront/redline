// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
//
// Hipfire-derived gfx12 microkernels for the ROCm #6409 attribution matrix.
// The wave reductions, x8 row reuse, and dp4a structure follow the production
// Hipfire kernel family pinned in ../../engines/hipfire at build time.  Every
// entry deliberately has one uniform explicit kernarg ABI so HIP, HipGraph,
// public-HSA-loaded Redline PM4, and Vulkan can execute identical workloads.

#include <hip/hip_runtime.h>
#include <hip/hip_fp16.h>
#include <stdint.h>

using u32 = uint32_t;

#ifdef HIPFIRE_BENCH_WAVE64
#define BENCH_WAVE_WIDTH 64u
#define BENCH_WAVE_COUNT 4u
#define BENCH_QUANT_BLOCK 64
#else
#define BENCH_WAVE_WIDTH 32u
#define BENCH_WAVE_COUNT 8u
#define BENCH_QUANT_BLOCK 32
#endif

__device__ __forceinline__ u32 workgroup_x() {
    return __builtin_amdgcn_workgroup_id_x();
}

__device__ __forceinline__ u32 global_x() {
    return workgroup_x() * static_cast<u32>(blockDim.x) + static_cast<u32>(threadIdx.x);
}

__device__ __forceinline__ float load_f32(const u32* p, u32 i) {
    return __uint_as_float(p[i]);
}

__device__ __forceinline__ void output_add_u32(u32* out, u32 index, u32 value) {
    const auto resource = radiowave::buffer_resource(out);
    const u32 previous = radiowave::buffer_load_u32(resource, index);
    radiowave::buffer_store_u32(resource, index, previous + value);
}

__device__ __forceinline__ void output_add_f32(u32* out, u32 index, float value) {
    const auto resource = radiowave::buffer_resource(out);
    const float previous = __uint_as_float(radiowave::buffer_load_u32(resource, index));
    radiowave::buffer_store_u32(resource, index, __float_as_uint(previous + value));
}

__device__ __forceinline__ float wave_sum_f32(float v) {
#pragma unroll
    for (int off = BENCH_WAVE_WIDTH / 2; off > 0; off >>= 1) v += __shfl_down(v, off);
    return v;
}

__device__ __forceinline__ int wave_sum_i32(int v) {
#pragma unroll
    for (int off = BENCH_WAVE_WIDTH / 2; off > 0; off >>= 1) v += __shfl_down(v, off);
    return v;
}

__device__ __forceinline__ int dot_s8_s8(u32 a, u32 b, int c) {
#ifdef HIPFIRE_BENCH_DOT8
    // gfx12 exposes the generalized signedness-controlled dot8 form rather
    // than the legacy dot1-insts-only sdot4 builtin.
    return __builtin_amdgcn_sudot4(
        true, static_cast<int>(a), true, static_cast<int>(b), c, false);
#else
    int sum = c;
#pragma unroll
    for (u32 lane = 0; lane < 4; ++lane) {
        const int x = static_cast<int>(static_cast<int8_t>(a >> (lane * 8)));
        const int y = static_cast<int>(static_cast<int8_t>(b >> (lane * 8)));
        sum += x * y;
    }
    return sum;
#endif
}

__device__ __forceinline__ int dot_u8_s8(u32 a, u32 b, int c) {
#ifdef HIPFIRE_BENCH_DOT8
    return __builtin_amdgcn_sudot4(
        false, static_cast<int>(a), true, static_cast<int>(b), c, false);
#else
    int sum = c;
#pragma unroll
    for (u32 lane = 0; lane < 4; ++lane) {
        const int x = static_cast<int>((a >> (lane * 8)) & 0xffu);
        const int y = static_cast<int>(static_cast<int8_t>(b >> (lane * 8)));
        sum += x * y;
    }
    return sum;
#endif
}

__device__ __forceinline__ int scalar_q4(u32 a, u32 b) {
    int sum = 0;
#pragma unroll
    for (u32 lane = 0; lane < 4; ++lane) {
        const int q = static_cast<int>((a >> (lane * 8)) & 15u) - 8;
        const int x = static_cast<int>(static_cast<int8_t>(b >> (lane * 8)));
        sum += q * x;
    }
    return sum;
}

#define BENCH_ARGS \
    const u32* __restrict__ a, const u32* __restrict__ b, \
    u32* __restrict__ out, u32 n0, u32 n1, u32 output_offset, u32 aux

extern "C" __global__ __launch_bounds__(256, 16)
void dispatch_tiny(BENCH_ARGS) {
    (void)a; (void)b; (void)n1; (void)aux;
    const u32 gid = workgroup_x();
    if (threadIdx.x == 0 && gid < n0) output_add_u32(out, output_offset + gid, 1u);
}

template <bool BUFFER_INPUTS>
__device__ __forceinline__ void geometry_wave_body(BENCH_ARGS) {
    __shared__ float wave_partials[BENCH_WAVE_COUNT];
    const u32 row = workgroup_x();
    const u32 tid = static_cast<u32>(threadIdx.x);
    if (row >= n1) return;
    const auto a_resource = radiowave::buffer_resource(a);
    const auto b_resource = radiowave::buffer_resource(b);
    float sum = 0.0f;
    for (u32 repeat = 0; repeat < aux; ++repeat) {
        for (u32 i = tid; i < n0; i += static_cast<u32>(blockDim.x)) {
            const u32 j = (i + repeat) % n0;
            float av;
            float bv;
            if constexpr (BUFFER_INPUTS) {
                av = __uint_as_float(radiowave::buffer_load_u32(a_resource, j));
                bv = __uint_as_float(
                    radiowave::buffer_load_u32(b_resource, row * n0 + j));
            } else {
                av = load_f32(a, j);
                bv = load_f32(b, row * n0 + j);
            }
            sum = fmaf(av, bv, sum);
        }
    }
    sum = wave_sum_f32(sum);
    const u32 lane = tid & (BENCH_WAVE_WIDTH - 1u);
    const u32 wave = tid / BENCH_WAVE_WIDTH;
    const u32 wave_count =
        (static_cast<u32>(blockDim.x) + BENCH_WAVE_WIDTH - 1u) / BENCH_WAVE_WIDTH;
    if (lane == 0) wave_partials[wave] = sum;
    __syncthreads();
    if (wave == 0) {
        float total = lane < wave_count ? wave_partials[lane] : 0.0f;
        total = wave_sum_f32(total);
        if (lane == 0) {
            output_add_f32(out, output_offset + row, total);
        }
    }
}

extern "C" __global__ __launch_bounds__(256, 16)
void geometry_fma(BENCH_ARGS) {
    geometry_wave_body<false>(a, b, out, n0, n1, output_offset, aux);
}

extern "C" __global__ __launch_bounds__(256, 16)
void reduction_wave(BENCH_ARGS) {
    geometry_wave_body<false>(a, b, out, n0, n1, output_offset, aux);
}

extern "C" __global__ __launch_bounds__(256, 16)
void geometry_fma_buffer(BENCH_ARGS) {
    geometry_wave_body<true>(a, b, out, n0, n1, output_offset, aux);
}

extern "C" __global__ __launch_bounds__(256, 16)
void reduction_wave_buffer(BENCH_ARGS) {
    geometry_wave_body<true>(a, b, out, n0, n1, output_offset, aux);
}

template <u32 ACCUM_COUNT, bool EXTRA_BARRIER>
__device__ __forceinline__ void reduction_lds_body(BENCH_ARGS) {
    __shared__ float scratch[256];
    const u32 row = workgroup_x();
    const u32 tid = static_cast<u32>(threadIdx.x);
    if (row >= n1) return;
    float accum[ACCUM_COUNT] = {};
    const u32 wg = static_cast<u32>(blockDim.x);
    for (u32 repeat = 0; repeat < aux; ++repeat) {
        for (u32 base = tid; base < n0; base += wg * ACCUM_COUNT) {
#pragma unroll
            for (u32 slot = 0; slot < ACCUM_COUNT; ++slot) {
                const u32 i = base + slot * wg;
                if (i < n0) {
                    const u32 j = (i + repeat) % n0;
                    accum[slot] = fmaf(
                        load_f32(a, j), load_f32(b, row * n0 + j), accum[slot]);
                }
            }
        }
    }
    float sum = 0.0f;
#pragma unroll
    for (u32 slot = 0; slot < ACCUM_COUNT; ++slot) sum += accum[slot];
    scratch[tid] = sum;
    __syncthreads();
    for (u32 off = wg >> 1; off; off >>= 1) {
        if (tid < off) scratch[tid] += scratch[tid + off];
        __syncthreads();
        if constexpr (EXTRA_BARRIER) __syncthreads();
    }
    if (tid == 0) output_add_f32(out, output_offset + row, scratch[0]);
}

extern "C" __global__ __launch_bounds__(256, 16)
void reduction_lds(BENCH_ARGS) {
    reduction_lds_body<1, false>(a, b, out, n0, n1, output_offset, aux);
}

extern "C" __global__ __launch_bounds__(256, 16)
void reduction_extra_barrier(BENCH_ARGS) {
    reduction_lds_body<1, true>(a, b, out, n0, n1, output_offset, aux);
}

extern "C" __global__ __launch_bounds__(256, 16)
void reduction_multi4(BENCH_ARGS) {
    reduction_lds_body<4, false>(a, b, out, n0, n1, output_offset, aux);
}

extern "C" __global__ __launch_bounds__(256, 16)
void reduction_multi8(BENCH_ARGS) {
    reduction_lds_body<8, false>(a, b, out, n0, n1, output_offset, aux);
}

extern "C" __global__ __launch_bounds__(256, 16)
void reduction_multi16(BENCH_ARGS) {
    reduction_lds_body<16, false>(a, b, out, n0, n1, output_offset, aux);
}

extern "C" __global__ __launch_bounds__(256, 16)
void memory_coalesced4(BENCH_ARGS) {
    const u32 idx = global_x();
    if (idx >= n0) return;
    float sum = 0.0f;
    for (u32 iter = 0; iter < n1; ++iter) {
        const u32 base = (iter * n0 * 4u + idx * 4u) & aux;
        const float x0 = load_f32(a, base);
        const float x1 = load_f32(a, (base + 1u) & aux);
        const float x2 = load_f32(a, (base + 2u) & aux);
        const float x3 = load_f32(a, (base + 3u) & aux);
        sum = fmaf(x0, 1.000001f, sum);
        sum = fmaf(x1, 1.000002f, sum);
        sum = fmaf(x2, 1.000003f, sum);
        sum = fmaf(x3, 1.000004f, sum);
    }
    reinterpret_cast<float*>(out)[output_offset + idx] += sum;
}

extern "C" __global__ __launch_bounds__(256, 16)
void memory_strided4(BENCH_ARGS) {
    const u32 idx = global_x();
    if (idx >= n0) return;
    float sum = 0.0f;
    for (u32 iter = 0; iter < n1; ++iter) {
        const u32 address = (iter * n0 * 4u + idx * 4u) & aux;
        sum = fmaf(load_f32(a, address), 1.000001f, sum);
    }
    reinterpret_cast<float*>(out)[output_offset + idx] += sum;
}

extern "C" __global__ __launch_bounds__(256, 16)
void memory_gather(BENCH_ARGS) {
    const u32 idx = global_x();
    if (idx >= n0) return;

    const auto a_resource = radiowave::buffer_resource(a);
    const auto b_resource = radiowave::buffer_resource(b);
    const auto out_resource = radiowave::buffer_resource(out);
    float sum = 0.0f;
    if (n1 == 16u) {
#pragma unroll 1
        for (u32 iter = 0; iter < 16u; iter += 4u) {
            const u32 address0 = radiowave::buffer_load_u32(b_resource, (iter + 0u) * n0 + idx) & aux;
            const u32 address1 = radiowave::buffer_load_u32(b_resource, (iter + 1u) * n0 + idx) & aux;
            const u32 address2 = radiowave::buffer_load_u32(b_resource, (iter + 2u) * n0 + idx) & aux;
            const u32 address3 = radiowave::buffer_load_u32(b_resource, (iter + 3u) * n0 + idx) & aux;
            const float value0 = __uint_as_float(radiowave::buffer_load_u32(a_resource, address0));
            const float value1 = __uint_as_float(radiowave::buffer_load_u32(a_resource, address1));
            const float value2 = __uint_as_float(radiowave::buffer_load_u32(a_resource, address2));
            const float value3 = __uint_as_float(radiowave::buffer_load_u32(a_resource, address3));
            sum = fmaf(value0, 1.000001f, sum);
            sum = fmaf(value1, 1.000001f, sum);
            sum = fmaf(value2, 1.000001f, sum);
            sum = fmaf(value3, 1.000001f, sum);
        }
    } else {
        for (u32 iter = 0; iter < n1; ++iter) {
            const u32 address = radiowave::buffer_load_u32(b_resource, iter * n0 + idx) & aux;
            sum = fmaf(__uint_as_float(radiowave::buffer_load_u32(a_resource, address)),
                       1.000001f, sum);
        }
    }
    const u32 out_idx = output_offset + idx;
    const float previous = __uint_as_float(radiowave::buffer_load_u32(out_resource, out_idx));
    radiowave::buffer_store_u32(out_resource, out_idx, __float_as_uint(previous + sum));
}


__device__ __forceinline__ u32 hash_u32(u32 v) {
    v ^= v >> 16; v *= 0x7feb352du; v ^= v >> 15;
    v *= 0x846ca68bu; v ^= v >> 16; return v;
}

__device__ __forceinline__ void hash_u32_pair(u32& x, u32& y) {
    x ^= x >> 16; y ^= y >> 16;
    x *= 0x7feb352du; y *= 0x7feb352du;
    x ^= x >> 15; y ^= y >> 15;
    x *= 0x846ca68bu; y *= 0x846ca68bu;
    x ^= x >> 16; y ^= y >> 16;
}

template <u32 BLOCK, bool VECTOR_LOAD, bool BUFFER_OUTPUT, bool SCALAR_PRELOAD = false>
__device__ __forceinline__ void memory_interleave4_body(BENCH_ARGS) {
    const u32 idx = workgroup_x() * BLOCK + static_cast<u32>(threadIdx.x);
    if (idx >= n0) return;
    const auto a_resource = radiowave::buffer_resource(a);
    float sum = 0.0f;
    u32 state = idx * 747796405u + 2891336453u;
    for (u32 iter = 0; iter < n1; ++iter) {
        const u32 base = (iter * n0 * 4u + idx * 4u) & aux;
        radiowave::u32x4 words = {};
        if constexpr (VECTOR_LOAD) {
            words = radiowave::buffer_load_u32x4(a_resource, base);
        } else if constexpr (SCALAR_PRELOAD) {
            // Keep four independently completable VMEM requests live before
            // the hash/FMA chain. ACO naturally forms this shape into a
            // four-load clause; the ordinary scalar path below intentionally
            // retains its load-at-use behavior as the control.
            u32 byte0 = (base + 0u) * sizeof(u32);
            u32 byte1 = (base + 1u) * sizeof(u32);
            u32 byte2 = (base + 2u) * sizeof(u32);
            u32 byte3 = (base + 3u) * sizeof(u32);
            radiowave::opaque_vgpr_u32x4(byte0, byte1, byte2, byte3);
            words[0] = radiowave::buffer_load_bytes_u32(a_resource, byte0);
            words[1] = radiowave::buffer_load_bytes_u32(a_resource, byte1);
            words[2] = radiowave::buffer_load_bytes_u32(a_resource, byte2);
            words[3] = radiowave::buffer_load_bytes_u32(a_resource, byte3);
        }
#pragma unroll
        for (u32 j = 0; j < 4; ++j) {
            state = hash_u32(state + iter + j);
            const float bias = static_cast<float>(state & 255u) * 0.0000001f;
            u32 word;
            if constexpr (VECTOR_LOAD || SCALAR_PRELOAD) {
                word = words[j];
            } else {
                word = radiowave::buffer_load_u32(a_resource, base + j);
            }
            sum = fmaf(__uint_as_float(word) + bias,
                       0.999999f + static_cast<float>(j) * 0.000001f, sum);
        }
    }
    const u32 out_idx = output_offset + idx;
    if constexpr (BUFFER_OUTPUT) {
        const auto out_resource = radiowave::buffer_resource(out);
        const float previous = __uint_as_float(
            radiowave::buffer_load_u32(out_resource, out_idx));
        radiowave::buffer_store_u32(
            out_resource, out_idx, __float_as_uint(previous + sum));
    } else {
        reinterpret_cast<float*>(out)[out_idx] += sum;
    }
}

extern "C" __global__ __launch_bounds__(256, 16)
void memory_interleave4(BENCH_ARGS) {
    if (blockDim.x == 64) {
        memory_interleave4_body<64u, true, false>(a, b, out, n0, n1, output_offset, aux);
    } else {
        memory_interleave4_body<256u, true, false>(a, b, out, n0, n1, output_offset, aux);
    }
}

extern "C" __global__ __launch_bounds__(256, 16)
void memory_interleave4_buffer(BENCH_ARGS) {
    if (blockDim.x == 64) {
        memory_interleave4_body<64u, false, true>(a, b, out, n0, n1, output_offset, aux);
    } else {
        memory_interleave4_body<256u, false, true>(a, b, out, n0, n1, output_offset, aux);
    }
}

extern "C" __global__ __launch_bounds__(64, 16)
void memory_interleave4_block64(BENCH_ARGS) {
    memory_interleave4_body<64u, true, false>(a, b, out, n0, n1, output_offset, aux);
}

extern "C" __global__ __launch_bounds__(64, 16)
void memory_interleave4_block64_b32(BENCH_ARGS) {
    memory_interleave4_body<64u, false, false, true>(
        a, b, out, n0, n1, output_offset, aux);
}

template <int MODE>
__device__ __forceinline__ void dot_body(BENCH_ARGS) {
    const u32 idx = global_x();
    if (idx >= n0) return;
    const auto a_resource = radiowave::buffer_resource(a);
    const auto b_resource = radiowave::buffer_resource(b);
    const auto out_resource = radiowave::buffer_resource(out);
    int sum = 0;
    constexpr u32 groups = 16;
    for (u32 iter = 0; iter < n1; ++iter) {
        const u32 base = ((iter * n0 + idx) * groups) & aux;
#ifndef __gfx1010__
#pragma unroll
        for (u32 chunk = 0; chunk < groups; chunk += 4u) {
            // Each dot tile is eight-word aligned in a power-of-two ring.
            // Two B128 loads expose the same eight words with four times fewer
            // memory instructions than scalar B32 lowering.
            const auto weights = radiowave::buffer_load_u32x4(a_resource, base + chunk);
            const auto values = radiowave::buffer_load_u32x4(b_resource, base + chunk);
#pragma unroll
            for (u32 lane = 0; lane < 4u; ++lane) {
                const u32 w = weights[lane];
                const u32 x = values[lane];
                if constexpr (MODE == 0) sum = dot_s8_s8(w, x, sum);
                if constexpr (MODE == 1) sum = dot_u8_s8(w, x, sum);
                if constexpr (MODE == 2) {
                    sum += dot_u8_s8(w, x, 0) - 32 * dot_u8_s8(0x01010101u, x, 0);
                }
                if constexpr (MODE == 3) sum += scalar_q4(w, x);
            }
        }
#else
        // On gfx1010, keeping two B128 vectors live while expanding each
        // four-byte dot creates enough scalarized temporaries for LLVM to
        // spill. Stream B32 pairs instead: this preserves the exact operation
        // while keeping the retained-PM4 path inside its zero-scratch ABI.
        for (u32 chunk = 0; chunk < groups; ++chunk) {
            const u32 w = radiowave::buffer_load_u32(a_resource, base + chunk);
            const u32 x = radiowave::buffer_load_u32(b_resource, base + chunk);
            if constexpr (MODE == 0) sum = dot_s8_s8(w, x, sum);
            if constexpr (MODE == 1) sum = dot_u8_s8(w, x, sum);
            if constexpr (MODE == 2) {
                sum += dot_u8_s8(w, x, 0) - 32 * dot_u8_s8(0x01010101u, x, 0);
            }
            if constexpr (MODE == 3) sum += scalar_q4(w, x);
        }
#endif
    }
    const u32 out_idx = output_offset + idx;
    radiowave::buffer_store_u32(
        out_resource, out_idx,
        radiowave::buffer_load_u32(out_resource, out_idx) + static_cast<u32>(sum));
}

#include "microwave_gen.h"

extern "C" __global__ __launch_bounds__(256, 16)
void dot_q8(BENCH_ARGS) {
#if defined(__gfx1201__)
#ifdef HIPFIRE_BENCH_WAVE64
    asm volatile(MICROWAVE_DOT_Q8_GFX1201_W64_BODY
        :
        : "{s[2:3]}"(__builtin_amdgcn_kernarg_segment_ptr()), "{s4}"((u32)blockDim.x), "{v0}"(__builtin_amdgcn_workitem_id_x())
        : "v1", "v2", "v3", "v4", "v5", "v6", "v7", "v8", "v9", "v10", "v11", "v12", "v13", "v14", "v15", "v16", "v17", "v18", "v19", "v20", "v21", "v22", "v23", "v24", "v25", "v26", "v27", "v28", "v29", "v30", "v31", "v32", "v33", "v34", "v35", "v36", "v37", "v38", "v39", "v40", "v41", "v42", "v43", "s0", "s1", "s5", "s6", "s7", "vcc", "scc", "exec", "m0", "memory");
#else
    asm volatile(MICROWAVE_DOT_Q8_GFX1201_W32_BODY
        :
        : "{s[2:3]}"(__builtin_amdgcn_kernarg_segment_ptr()), "{s4}"((u32)blockDim.x), "{v0}"(__builtin_amdgcn_workitem_id_x())
        : "v1", "v2", "v3", "v4", "v5", "v6", "v7", "v8", "v9", "v10", "v11", "v12", "v13", "v14", "v15", "v16", "v17", "v18", "v19", "v20", "v21", "v22", "v23", "v24", "v25", "v26", "v27", "v28", "v29", "v30", "v31", "v32", "v33", "v34", "v35", "v36", "v37", "v38", "v39", "v40", "v41", "v42", "v43", "s0", "s1", "s5", "s6", "s7", "vcc", "scc", "exec", "m0", "memory");
#endif
#elif defined(__gfx1151__)
#ifdef HIPFIRE_BENCH_WAVE64
    asm volatile(MICROWAVE_DOT_Q8_GFX1151_W64_BODY
        :
        : "{s[2:3]}"(__builtin_amdgcn_kernarg_segment_ptr()), "{s4}"((u32)blockDim.x), "{s5}"(__builtin_amdgcn_workgroup_id_x()), "{v0}"(__builtin_amdgcn_workitem_id_x())
        : "v1", "v2", "v3", "v4", "v5", "v6", "v7", "v8", "v9", "v10", "v11", "v12", "v13", "v14", "v15", "v16", "v17", "v18", "v19", "v20", "v21", "v22", "v23", "v24", "v25", "v26", "v27", "v28", "v29", "v30", "v31", "v32", "v33", "v34", "v35", "v36", "v37", "v38", "v39", "s0", "s1", "s6", "s7", "s8", "vcc", "scc", "exec", "m0", "memory");
#else
    asm volatile(MICROWAVE_DOT_Q8_GFX1151_W32_BODY
        :
        : "{s[2:3]}"(__builtin_amdgcn_kernarg_segment_ptr()), "{s4}"((u32)blockDim.x), "{s5}"(__builtin_amdgcn_workgroup_id_x()), "{v0}"(__builtin_amdgcn_workitem_id_x())
        : "v1", "v2", "v3", "v4", "v5", "v6", "v7", "v8", "v9", "v10", "v11", "v12", "v13", "v14", "v15", "v16", "v17", "v18", "v19", "v20", "v21", "v22", "v23", "v24", "v25", "v26", "v27", "v28", "v29", "v30", "v31", "v32", "v33", "v34", "v35", "v36", "v37", "v38", "v39", "s0", "s1", "s6", "s7", "s8", "vcc", "scc", "exec", "m0", "memory");
#endif
#elif defined(__gfx1100__)
#ifdef HIPFIRE_BENCH_WAVE64
    asm volatile(MICROWAVE_DOT_Q8_GFX1100_W64_BODY
        :
        : "{s[2:3]}"(__builtin_amdgcn_kernarg_segment_ptr()), "{s4}"((u32)blockDim.x), "{s5}"(__builtin_amdgcn_workgroup_id_x()), "{v0}"(__builtin_amdgcn_workitem_id_x())
        : "v1", "v2", "v3", "v4", "v5", "v6", "v7", "v8", "v9", "v10", "v11", "v12", "v13", "v14", "v15", "v16", "v17", "v18", "v19", "v20", "v21", "v22", "v23", "v24", "v25", "v26", "v27", "v28", "v29", "v30", "v31", "v32", "v33", "v34", "v35", "v36", "v37", "v38", "v39", "s0", "s1", "s6", "s7", "s8", "vcc", "scc", "exec", "m0", "memory");
#else
    asm volatile(MICROWAVE_DOT_Q8_GFX1100_W32_BODY
        :
        : "{s[2:3]}"(__builtin_amdgcn_kernarg_segment_ptr()), "{s4}"((u32)blockDim.x), "{s5}"(__builtin_amdgcn_workgroup_id_x()), "{v0}"(__builtin_amdgcn_workitem_id_x())
        : "v1", "v2", "v3", "v4", "v5", "v6", "v7", "v8", "v9", "v10", "v11", "v12", "v13", "v14", "v15", "v16", "v17", "v18", "v19", "v20", "v21", "v22", "v23", "v24", "v25", "v26", "v27", "v28", "v29", "v30", "v31", "v32", "v33", "v34", "v35", "v36", "v37", "v38", "v39", "s0", "s1", "s6", "s7", "s8", "vcc", "scc", "exec", "m0", "memory");
#endif
#else
#warning "microwave body missing for dot_q8 arch, falling back to stock"
    dot_body<0>(a, b, out, n0, n1, output_offset, aux);
#endif
}

extern "C" __global__ __launch_bounds__(256, 16)
void dot_q4(BENCH_ARGS) {
#if defined(__gfx1201__)
#ifdef HIPFIRE_BENCH_WAVE64
    asm volatile(MICROWAVE_DOT_Q4_GFX1201_W64_BODY
        :
        : "{s[2:3]}"(__builtin_amdgcn_kernarg_segment_ptr()), "{s4}"((u32)blockDim.x), "{v0}"(__builtin_amdgcn_workitem_id_x())
        : "v1", "v2", "v3", "v4", "v5", "v6", "v7", "v8", "v9", "v10", "v11", "v12", "v13", "v14", "v15", "v16", "v17", "v18", "v19", "v20", "v21", "v22", "v23", "v24", "v25", "v26", "v27", "v28", "v29", "v30", "v31", "v32", "v33", "v34", "v35", "v36", "v37", "v38", "v39", "v40", "v41", "v42", "v43", "s0", "s1", "s5", "s6", "s7", "vcc", "scc", "exec", "m0", "memory");
#else
    asm volatile(MICROWAVE_DOT_Q4_GFX1201_W32_BODY
        :
        : "{s[2:3]}"(__builtin_amdgcn_kernarg_segment_ptr()), "{s4}"((u32)blockDim.x), "{v0}"(__builtin_amdgcn_workitem_id_x())
        : "v1", "v2", "v3", "v4", "v5", "v6", "v7", "v8", "v9", "v10", "v11", "v12", "v13", "v14", "v15", "v16", "v17", "v18", "v19", "v20", "v21", "v22", "v23", "v24", "v25", "v26", "v27", "v28", "v29", "v30", "v31", "v32", "v33", "v34", "v35", "v36", "v37", "v38", "v39", "v40", "v41", "v42", "v43", "s0", "s1", "s5", "s6", "s7", "vcc", "scc", "exec", "m0", "memory");
#endif
#elif defined(__gfx1151__)
#ifdef HIPFIRE_BENCH_WAVE64
    asm volatile(MICROWAVE_DOT_Q4_GFX1151_W64_BODY
        :
        : "{s[2:3]}"(__builtin_amdgcn_kernarg_segment_ptr()), "{s4}"((u32)blockDim.x), "{s5}"(__builtin_amdgcn_workgroup_id_x()), "{v0}"(__builtin_amdgcn_workitem_id_x())
        : "v1", "v2", "v3", "v4", "v5", "v6", "v7", "v8", "v9", "v10", "v11", "v12", "v13", "v14", "v15", "v16", "v17", "v18", "v19", "v20", "v21", "v22", "v23", "v24", "v25", "v26", "v27", "v28", "v29", "v30", "v31", "v32", "v33", "v34", "v35", "v36", "v37", "v38", "v39", "s0", "s1", "s6", "s7", "s8", "vcc", "scc", "exec", "m0", "memory");
#else
    asm volatile(MICROWAVE_DOT_Q4_GFX1151_W32_BODY
        :
        : "{s[2:3]}"(__builtin_amdgcn_kernarg_segment_ptr()), "{s4}"((u32)blockDim.x), "{s5}"(__builtin_amdgcn_workgroup_id_x()), "{v0}"(__builtin_amdgcn_workitem_id_x())
        : "v1", "v2", "v3", "v4", "v5", "v6", "v7", "v8", "v9", "v10", "v11", "v12", "v13", "v14", "v15", "v16", "v17", "v18", "v19", "v20", "v21", "v22", "v23", "v24", "v25", "v26", "v27", "v28", "v29", "v30", "v31", "v32", "v33", "v34", "v35", "v36", "v37", "v38", "v39", "s0", "s1", "s6", "s7", "s8", "vcc", "scc", "exec", "m0", "memory");
#endif
#elif defined(__gfx1100__)
#ifdef HIPFIRE_BENCH_WAVE64
    asm volatile(MICROWAVE_DOT_Q4_GFX1100_W64_BODY
        :
        : "{s[2:3]}"(__builtin_amdgcn_kernarg_segment_ptr()), "{s4}"((u32)blockDim.x), "{s5}"(__builtin_amdgcn_workgroup_id_x()), "{v0}"(__builtin_amdgcn_workitem_id_x())
        : "v1", "v2", "v3", "v4", "v5", "v6", "v7", "v8", "v9", "v10", "v11", "v12", "v13", "v14", "v15", "v16", "v17", "v18", "v19", "v20", "v21", "v22", "v23", "v24", "v25", "v26", "v27", "v28", "v29", "v30", "v31", "v32", "v33", "v34", "v35", "v36", "v37", "v38", "v39", "s0", "s1", "s6", "s7", "s8", "vcc", "scc", "exec", "m0", "memory");
#else
    asm volatile(MICROWAVE_DOT_Q4_GFX1100_W32_BODY
        :
        : "{s[2:3]}"(__builtin_amdgcn_kernarg_segment_ptr()), "{s4}"((u32)blockDim.x), "{s5}"(__builtin_amdgcn_workgroup_id_x()), "{v0}"(__builtin_amdgcn_workitem_id_x())
        : "v1", "v2", "v3", "v4", "v5", "v6", "v7", "v8", "v9", "v10", "v11", "v12", "v13", "v14", "v15", "v16", "v17", "v18", "v19", "v20", "v21", "v22", "v23", "v24", "v25", "v26", "v27", "v28", "v29", "v30", "v31", "v32", "v33", "v34", "v35", "v36", "v37", "v38", "v39", "s0", "s1", "s6", "s7", "s8", "vcc", "scc", "exec", "m0", "memory");
#endif
#else
#warning "microwave body missing for dot_q4 arch, falling back to stock"
    dot_body<1>(a, b, out, n0, n1, output_offset, aux);
#endif
}

extern "C" __global__ __launch_bounds__(256, 16)
void dot_q6(BENCH_ARGS) {
#if defined(__gfx1201__)
#ifdef HIPFIRE_BENCH_WAVE64
    asm volatile(MICROWAVE_DOT_Q6_GFX1201_W64_BODY
        :
        : "{s[2:3]}"(__builtin_amdgcn_kernarg_segment_ptr()), "{s4}"((u32)blockDim.x), "{v0}"(__builtin_amdgcn_workitem_id_x())
        : "v1", "v2", "v3", "v4", "v5", "v6", "v7", "v8", "v9", "v10", "v11", "v12", "v13", "v14", "v15", "v16", "v17", "v18", "v19", "v20", "v21", "v22", "v23", "v24", "v25", "v26", "v27", "v28", "v29", "v30", "v31", "v32", "v33", "v34", "v35", "v36", "v37", "v38", "v39", "v40", "v41", "v42", "v43", "s0", "s1", "s5", "s6", "s7", "s8", "vcc", "scc", "exec", "m0", "memory");
#else
    asm volatile(MICROWAVE_DOT_Q6_GFX1201_W32_BODY
        :
        : "{s[2:3]}"(__builtin_amdgcn_kernarg_segment_ptr()), "{s4}"((u32)blockDim.x), "{v0}"(__builtin_amdgcn_workitem_id_x())
        : "v1", "v2", "v3", "v4", "v5", "v6", "v7", "v8", "v9", "v10", "v11", "v12", "v13", "v14", "v15", "v16", "v17", "v18", "v19", "v20", "v21", "v22", "v23", "v24", "v25", "v26", "v27", "v28", "v29", "v30", "v31", "v32", "v33", "v34", "v35", "v36", "v37", "v38", "v39", "v40", "v41", "v42", "v43", "s0", "s1", "s5", "s6", "s7", "s8", "vcc", "scc", "exec", "m0", "memory");
#endif
#elif defined(__gfx1151__)
#ifdef HIPFIRE_BENCH_WAVE64
    asm volatile(MICROWAVE_DOT_Q6_GFX1151_W64_BODY
        :
        : "{s[2:3]}"(__builtin_amdgcn_kernarg_segment_ptr()), "{s4}"((u32)blockDim.x), "{s5}"(__builtin_amdgcn_workgroup_id_x()), "{v0}"(__builtin_amdgcn_workitem_id_x())
        : "v1", "v2", "v3", "v4", "v5", "v6", "v7", "v8", "v9", "v10", "v11", "v12", "v13", "v14", "v15", "v16", "v17", "v18", "v19", "v20", "v21", "v22", "v23", "v24", "v25", "v26", "v27", "v28", "v29", "v30", "v31", "v32", "v33", "v34", "v35", "v36", "v37", "v38", "v39", "s0", "s1", "s6", "s7", "s8", "s9", "vcc", "scc", "exec", "m0", "memory");
#else
    asm volatile(MICROWAVE_DOT_Q6_GFX1151_W32_BODY
        :
        : "{s[2:3]}"(__builtin_amdgcn_kernarg_segment_ptr()), "{s4}"((u32)blockDim.x), "{s5}"(__builtin_amdgcn_workgroup_id_x()), "{v0}"(__builtin_amdgcn_workitem_id_x())
        : "v1", "v2", "v3", "v4", "v5", "v6", "v7", "v8", "v9", "v10", "v11", "v12", "v13", "v14", "v15", "v16", "v17", "v18", "v19", "v20", "v21", "v22", "v23", "v24", "v25", "v26", "v27", "v28", "v29", "v30", "v31", "v32", "v33", "v34", "v35", "v36", "v37", "v38", "v39", "s0", "s1", "s6", "s7", "s8", "s9", "vcc", "scc", "exec", "m0", "memory");
#endif
#elif defined(__gfx1100__)
#ifdef HIPFIRE_BENCH_WAVE64
    asm volatile(MICROWAVE_DOT_Q6_GFX1100_W64_BODY
        :
        : "{s[2:3]}"(__builtin_amdgcn_kernarg_segment_ptr()), "{s4}"((u32)blockDim.x), "{s5}"(__builtin_amdgcn_workgroup_id_x()), "{v0}"(__builtin_amdgcn_workitem_id_x())
        : "v1", "v2", "v3", "v4", "v5", "v6", "v7", "v8", "v9", "v10", "v11", "v12", "v13", "v14", "v15", "v16", "v17", "v18", "v19", "v20", "v21", "v22", "v23", "v24", "v25", "v26", "v27", "v28", "v29", "v30", "v31", "v32", "v33", "v34", "v35", "v36", "v37", "v38", "v39", "s0", "s1", "s6", "s7", "s8", "s9", "vcc", "scc", "exec", "m0", "memory");
#else
    asm volatile(MICROWAVE_DOT_Q6_GFX1100_W32_BODY
        :
        : "{s[2:3]}"(__builtin_amdgcn_kernarg_segment_ptr()), "{s4}"((u32)blockDim.x), "{s5}"(__builtin_amdgcn_workgroup_id_x()), "{v0}"(__builtin_amdgcn_workitem_id_x())
        : "v1", "v2", "v3", "v4", "v5", "v6", "v7", "v8", "v9", "v10", "v11", "v12", "v13", "v14", "v15", "v16", "v17", "v18", "v19", "v20", "v21", "v22", "v23", "v24", "v25", "v26", "v27", "v28", "v29", "v30", "v31", "v32", "v33", "v34", "v35", "v36", "v37", "v38", "v39", "s0", "s1", "s6", "s7", "s8", "s9", "vcc", "scc", "exec", "m0", "memory");
#endif
#else
#warning "microwave body missing for dot_q6 arch, falling back to stock"
    dot_body<2>(a, b, out, n0, n1, output_offset, aux);
#endif
}

extern "C" __global__ __launch_bounds__(256, 16)
void dot_scalar(BENCH_ARGS) {
#if defined(__gfx1201__)
#ifdef HIPFIRE_BENCH_WAVE64
    asm volatile(MICROWAVE_DOT_SCALAR_GFX1201_W64_BODY
        :
        : "{s[2:3]}"(__builtin_amdgcn_kernarg_segment_ptr()), "{s4}"((u32)blockDim.x), "{v0}"(__builtin_amdgcn_workitem_id_x())
        : "v1", "v2", "v3", "v4", "v5", "v6", "v7", "v8", "v9", "v10", "v11", "v12", "v13", "v14", "v15", "v16", "v17", "v18", "v19", "v20", "v21", "v22", "v23", "v24", "v25", "v26", "v27", "v28", "v29", "v30", "v31", "v32", "v33", "v34", "v35", "v36", "v37", "v38", "v39", "v40", "v41", "v42", "v43", "v44", "v45", "v46", "v47", "s0", "s1", "s5", "s6", "s7", "s8", "s9", "vcc", "scc", "exec", "m0", "memory");
#else
    asm volatile(MICROWAVE_DOT_SCALAR_GFX1201_W32_BODY
        :
        : "{s[2:3]}"(__builtin_amdgcn_kernarg_segment_ptr()), "{s4}"((u32)blockDim.x), "{v0}"(__builtin_amdgcn_workitem_id_x())
        : "v1", "v2", "v3", "v4", "v5", "v6", "v7", "v8", "v9", "v10", "v11", "v12", "v13", "v14", "v15", "v16", "v17", "v18", "v19", "v20", "v21", "v22", "v23", "v24", "v25", "v26", "v27", "v28", "v29", "v30", "v31", "v32", "v33", "v34", "v35", "v36", "v37", "v38", "v39", "v40", "v41", "v42", "v43", "v44", "v45", "v46", "v47", "v48", "v49", "v50", "v51", "v52", "v53", "v54", "v55", "v56", "v57", "v58", "v59", "v60", "v61", "v62", "v63", "v64", "v65", "v66", "v67", "v68", "v69", "v70", "v71", "v72", "v73", "v74", "v75", "v76", "v77", "v78", "v79", "v80", "v81", "v82", "v83", "v84", "v85", "v86", "v87", "v88", "v89", "v90", "v91", "v92", "v93", "v94", "v95", "s0", "s1", "s5", "s6", "s7", "s8", "s9", "vcc", "scc", "exec", "m0", "memory");
#endif
#elif defined(__gfx1151__)
#ifdef HIPFIRE_BENCH_WAVE64
    asm volatile(MICROWAVE_DOT_SCALAR_GFX1151_W64_BODY
        :
        : "{s[2:3]}"(__builtin_amdgcn_kernarg_segment_ptr()), "{s4}"((u32)blockDim.x), "{s5}"(__builtin_amdgcn_workgroup_id_x()), "{v0}"(__builtin_amdgcn_workitem_id_x())
        : "v1", "v2", "v3", "v4", "v5", "v6", "v7", "v8", "v9", "v10", "v11", "v12", "v13", "v14", "v15", "v16", "v17", "v18", "v19", "v20", "v21", "v22", "v23", "v24", "v25", "v26", "v27", "v28", "v29", "v30", "v31", "v32", "v33", "v34", "v35", "v36", "v37", "v38", "v39", "v40", "v41", "v42", "v43", "v44", "v45", "v46", "v47", "s0", "s1", "s6", "s7", "s8", "s9", "s10", "vcc", "scc", "exec", "m0", "memory");
#else
    asm volatile(MICROWAVE_DOT_SCALAR_GFX1151_W32_BODY
        :
        : "{s[2:3]}"(__builtin_amdgcn_kernarg_segment_ptr()), "{s4}"((u32)blockDim.x), "{s5}"(__builtin_amdgcn_workgroup_id_x()), "{v0}"(__builtin_amdgcn_workitem_id_x())
        : "v1", "v2", "v3", "v4", "v5", "v6", "v7", "v8", "v9", "v10", "v11", "v12", "v13", "v14", "v15", "v16", "v17", "v18", "v19", "v20", "v21", "v22", "v23", "v24", "v25", "v26", "v27", "v28", "v29", "v30", "v31", "v32", "v33", "v34", "v35", "v36", "v37", "v38", "v39", "v40", "v41", "v42", "v43", "v44", "v45", "v46", "v47", "v48", "v49", "v50", "v51", "v52", "v53", "v54", "v55", "v56", "v57", "v58", "v59", "v60", "v61", "v62", "v63", "v64", "v65", "v66", "v67", "v68", "v69", "v70", "v71", "v72", "v73", "v74", "v75", "v76", "v77", "v78", "v79", "v80", "v81", "v82", "v83", "v84", "v85", "v86", "v87", "v88", "v89", "v90", "v91", "v92", "v93", "v94", "v95", "s0", "s1", "s6", "s7", "s8", "s9", "s10", "vcc", "scc", "exec", "m0", "memory");
#endif
#elif defined(__gfx1100__)
#ifdef HIPFIRE_BENCH_WAVE64
    asm volatile(MICROWAVE_DOT_SCALAR_GFX1100_W64_BODY
        :
        : "{s[2:3]}"(__builtin_amdgcn_kernarg_segment_ptr()), "{s4}"((u32)blockDim.x), "{s5}"(__builtin_amdgcn_workgroup_id_x()), "{v0}"(__builtin_amdgcn_workitem_id_x())
        : "v1", "v2", "v3", "v4", "v5", "v6", "v7", "v8", "v9", "v10", "v11", "v12", "v13", "v14", "v15", "v16", "v17", "v18", "v19", "v20", "v21", "v22", "v23", "v24", "v25", "v26", "v27", "v28", "v29", "v30", "v31", "v32", "v33", "v34", "v35", "v36", "v37", "v38", "v39", "v40", "v41", "v42", "v43", "v44", "v45", "v46", "v47", "s0", "s1", "s6", "s7", "s8", "s9", "s10", "vcc", "scc", "exec", "m0", "memory");
#else
    asm volatile(MICROWAVE_DOT_SCALAR_GFX1100_W32_BODY
        :
        : "{s[2:3]}"(__builtin_amdgcn_kernarg_segment_ptr()), "{s4}"((u32)blockDim.x), "{s5}"(__builtin_amdgcn_workgroup_id_x()), "{v0}"(__builtin_amdgcn_workitem_id_x())
        : "v1", "v2", "v3", "v4", "v5", "v6", "v7", "v8", "v9", "v10", "v11", "v12", "v13", "v14", "v15", "v16", "v17", "v18", "v19", "v20", "v21", "v22", "v23", "v24", "v25", "v26", "v27", "v28", "v29", "v30", "v31", "v32", "v33", "v34", "v35", "v36", "v37", "v38", "v39", "v40", "v41", "v42", "v43", "v44", "v45", "v46", "v47", "v48", "v49", "v50", "v51", "v52", "v53", "v54", "v55", "v56", "v57", "v58", "v59", "v60", "v61", "v62", "v63", "v64", "v65", "v66", "v67", "v68", "v69", "v70", "v71", "v72", "v73", "v74", "v75", "v76", "v77", "v78", "v79", "v80", "v81", "v82", "v83", "v84", "v85", "v86", "v87", "v88", "v89", "v90", "v91", "v92", "v93", "v94", "v95", "s0", "s1", "s6", "s7", "s8", "s9", "s10", "vcc", "scc", "exec", "m0", "memory");
#endif
#else
#warning "microwave body missing for dot_scalar arch, falling back to stock"
    dot_body<3>(a, b, out, n0, n1, output_offset, aux);
#endif
}

template <int MODE>
__device__ __forceinline__ void vopd_iteration(
    u32 iter, u32& bias_state, u32& u0, u32& u1,
    float& v0, float& v1, float& v2, float& v3) {
    if constexpr (MODE == 0 || MODE == 1) {
        const float bias0 = static_cast<float>((bias_state >> 24) & 31u) * 0.000001f + 0.000003f;
        const float bias1 = static_cast<float>(((bias_state + 2246822519u) >> 24) & 31u) * 0.000001f + 0.000003f;
        const float bias2 = static_cast<float>(((bias_state + 198677742u) >> 24) & 31u) * 0.000001f + 0.000003f;
        const float bias3 = static_cast<float>(((bias_state + 2445500261u) >> 24) & 31u) * 0.000001f + 0.000003f;
        bias_state += 1013904223u;
        if constexpr (MODE == 0) {
            v0 = fmaf(v0, 1.000001f, bias0);
            v1 = fmaf(v1, 0.999999f, bias1);
            v2 = fmaf(v2, 1.000002f, bias2);
            v3 = fmaf(v3, 0.999998f, bias3);
        } else {
            v0 = fmaf(v0, 1.000001f, bias0);
            v0 = fmaf(v0, 0.999999f, bias1);
            v0 = fmaf(v0, 1.000002f, bias2);
            v0 = fmaf(v0, 0.999998f, bias3);
        }
    } else if constexpr (MODE == 2) {
        u0 = hash_u32(u0 + iter); u1 = hash_u32(u1 + iter + 1u);
        v0 = fmaf(v0, 1.000001f, static_cast<float>(u0 & 31u) * 0.000001f);
        v1 = fmaf(v1, 0.999999f, static_cast<float>(u1 & 31u) * 0.000001f);
        v2 = fmaf(v2, 1.000002f, static_cast<float>((u0 >> 7) & 31u) * 0.000001f);
        v3 = fmaf(v3, 0.999998f, static_cast<float>((u1 >> 11) & 31u) * 0.000001f);
    } else {
        u0 = hash_u32(u0 + iter); u1 = hash_u32(u1 + iter + 1u);
        const int q0 = static_cast<int>((u0 >> (iter & 15u)) & 255u) - 128;
        const int q1 = static_cast<int>((u1 >> ((iter + 3u) & 15u)) & 255u) - 128;
        v0 = fmaf(static_cast<float>(q0) * 0.0078125f, 0.03125f, v0);
        v1 = fmaf(static_cast<float>(q1) * 0.0078125f, 0.03125f, v1);
    }
}

template <int MODE>
__device__ __forceinline__ void vopd_body(BENCH_ARGS) {
    (void)a; (void)b; (void)n1;
    const u32 idx = global_x();
    if (idx >= n0) return;
    const auto out_resource = radiowave::buffer_resource(out);
    float v0 = 0.25f + static_cast<float>((idx * 747796405u) & 255u) * 0.0009765625f;
    float v1 = 0.25f + static_cast<float>(((idx * 747796405u) ^ 2891336453u) & 255u) * 0.0009765625f;
    float v2 = v0 + 0.03125f, v3 = v1 + 0.046875f;
    u32 u0 = idx * 747796405u + 2891336453u;
    u32 u1 = idx ^ 0xa5a5a5a5u;
    u32 bias_state = idx * 1664525u;
    if constexpr (MODE == 0) {
#pragma unroll 8
        for (u32 iter = 0; iter < aux; ++iter) {
            vopd_iteration<MODE>(iter, bias_state, u0, u1, v0, v1, v2, v3);
        }
    } else if constexpr (MODE == 2) {
#pragma unroll 8
        for (u32 iter = 0; iter < aux; ++iter) {
            vopd_iteration<MODE>(iter, bias_state, u0, u1, v0, v1, v2, v3);
        }
    } else {
#pragma unroll 2
        for (u32 iter = 0; iter < aux; ++iter) {
            vopd_iteration<MODE>(iter, bias_state, u0, u1, v0, v1, v2, v3);
        }
    }
    const u32 out_idx = output_offset + idx;
    const float previous = __uint_as_float(
        radiowave::buffer_load_u32(out_resource, out_idx));
    radiowave::buffer_store_u32(
        out_resource, out_idx,
        __float_as_uint(previous + v0 + v1 + v2 + v3));
}

__device__ __forceinline__ void vopd_store(
    u32* out, u32 output_offset, u32 idx,
    float v0, float v1, float v2, float v3) {
    const auto out_resource = radiowave::buffer_resource(out);
    const u32 out_idx = output_offset + idx;
    const float previous = __uint_as_float(
        radiowave::buffer_load_u32(out_resource, out_idx));
    radiowave::buffer_store_u32(
        out_resource, out_idx,
        __float_as_uint(previous + v0 + v1 + v2 + v3));
}

__device__ __forceinline__ void vopd_mixed_pair_body(BENCH_ARGS) {
    (void)a; (void)b; (void)n1;
    const u32 idx = global_x();
    if (idx >= n0) return;
    float v0 = 0.25f + static_cast<float>((idx * 747796405u) & 255u) * 0.0009765625f;
    float v1 = 0.25f + static_cast<float>(((idx * 747796405u) ^ 2891336453u) & 255u) * 0.0009765625f;
    float v2 = v0 + 0.03125f, v3 = v1 + 0.046875f;
    u32 u0 = idx * 747796405u + 2891336453u;
    u32 u1 = idx ^ 0xa5a5a5a5u;
#pragma unroll 8
    for (u32 iter = 0; iter < aux; ++iter) {
        u0 += iter;
        u1 += iter + 1u;
        hash_u32_pair(u0, u1);
        v0 = fmaf(v0, 1.000001f, static_cast<float>(u0 & 31u) * 0.000001f);
        v1 = fmaf(v1, 0.999999f, static_cast<float>(u1 & 31u) * 0.000001f);
        v2 = fmaf(v2, 1.000002f, static_cast<float>((u0 >> 7) & 31u) * 0.000001f);
        v3 = fmaf(v3, 0.999998f, static_cast<float>((u1 >> 11) & 31u) * 0.000001f);
    }
    vopd_store(out, output_offset, idx, v0, v1, v2, v3);
}

extern "C" __global__ __launch_bounds__(256, 16)
void vopd_mixed_pair(BENCH_ARGS) {
    vopd_mixed_pair_body(a, b, out, n0, n1, output_offset, aux);
}


template <u32 STEP>
__device__ __forceinline__ void dequant_accumulate_step(
    u32 base, u32& u0, u32& u1, int& q0_sum, int& q1_sum) {
    u0 = hash_u32(u0 + base + STEP);
    u1 = hash_u32(u1 + base + STEP + 1u);
    constexpr u32 shift0 = STEP & 15u;
    constexpr u32 shift1 = (STEP + 3u) & 15u;
    q0_sum += static_cast<int>((u0 >> shift0) & 255u) - 128;
    q1_sum += static_cast<int>((u1 >> shift1) & 255u) - 128;
}

extern "C" __global__ __launch_bounds__(256, 16)
void vopd_dequant_chunk16(BENCH_ARGS) {
    (void)a; (void)b; (void)n1;
    const u32 idx = global_x();
    if (idx >= n0) return;

    float v0 = 0.25f + static_cast<float>((idx * 747796405u) & 255u) * 0.0009765625f;
    float v1 = 0.25f + static_cast<float>(((idx * 747796405u) ^ 2891336453u) & 255u) * 0.0009765625f;
    float v2 = v0 + 0.03125f, v3 = v1 + 0.046875f;
    u32 u0 = idx * 747796405u + 2891336453u;
    u32 u1 = idx ^ 0xa5a5a5a5u;
    u32 base = 0;
    for (; base + 15u < aux; base += 16u) {
        int q0_sum = 0;
        int q1_sum = 0;
        dequant_accumulate_step<0>(base, u0, u1, q0_sum, q1_sum);
        dequant_accumulate_step<1>(base, u0, u1, q0_sum, q1_sum);
        dequant_accumulate_step<2>(base, u0, u1, q0_sum, q1_sum);
        dequant_accumulate_step<3>(base, u0, u1, q0_sum, q1_sum);
        dequant_accumulate_step<4>(base, u0, u1, q0_sum, q1_sum);
        dequant_accumulate_step<5>(base, u0, u1, q0_sum, q1_sum);
        dequant_accumulate_step<6>(base, u0, u1, q0_sum, q1_sum);
        dequant_accumulate_step<7>(base, u0, u1, q0_sum, q1_sum);
        dequant_accumulate_step<8>(base, u0, u1, q0_sum, q1_sum);
        dequant_accumulate_step<9>(base, u0, u1, q0_sum, q1_sum);
        dequant_accumulate_step<10>(base, u0, u1, q0_sum, q1_sum);
        dequant_accumulate_step<11>(base, u0, u1, q0_sum, q1_sum);
        dequant_accumulate_step<12>(base, u0, u1, q0_sum, q1_sum);
        dequant_accumulate_step<13>(base, u0, u1, q0_sum, q1_sum);
        dequant_accumulate_step<14>(base, u0, u1, q0_sum, q1_sum);
        dequant_accumulate_step<15>(base, u0, u1, q0_sum, q1_sum);
        v0 = fmaf(static_cast<float>(q0_sum), 0.000244140625f, v0);
        v1 = fmaf(static_cast<float>(q1_sum), 0.000244140625f, v1);
    }
    for (; base < aux; ++base) {
        u32 ignored_bias = 0;
        vopd_iteration<3>(base, ignored_bias, u0, u1, v0, v1, v2, v3);
    }
    vopd_store(out, output_offset, idx, v0, v1, v2, v3);
}

extern "C" __global__ __launch_bounds__(256, 16)
void vopd_independent(BENCH_ARGS) { vopd_body<0>(a,b,out,n0,n1,output_offset,aux); }
extern "C" __global__ __launch_bounds__(256, 16)
void vopd_dependent(BENCH_ARGS) { vopd_body<1>(a,b,out,n0,n1,output_offset,aux); }
extern "C" __global__ __launch_bounds__(256, 16)
void vopd_mixed(BENCH_ARGS) { vopd_body<2>(a,b,out,n0,n1,output_offset,aux); }
extern "C" __global__ __launch_bounds__(256, 16)
void vopd_dequant(BENCH_ARGS) { vopd_body<3>(a,b,out,n0,n1,output_offset,aux); }

extern "C" __global__ __launch_bounds__(256, 16)
void sampler_argmax(BENCH_ARGS) {
    (void)b; (void)aux;
    __shared__ float wave_values[BENCH_WAVE_COUNT];
    __shared__ u32 wave_indices[BENCH_WAVE_COUNT];
    const u32 row = workgroup_x();
    const u32 tid = static_cast<u32>(threadIdx.x);
    if (row >= n1) return;
    const auto a_resource = radiowave::buffer_resource(a);
    const auto out_resource = radiowave::buffer_resource(out);
    float best = -3.402823466e38f;
    u32 best_i = 0xffffffffu;
    for (u32 col = tid; col < n0; col += static_cast<u32>(blockDim.x)) {
        const float v = __uint_as_float(
            radiowave::buffer_load_u32(a_resource, row * n0 + col));
        if (v > best || (v == best && col < best_i)) { best = v; best_i = col; }
    }
    const u32 lane = tid & (BENCH_WAVE_WIDTH - 1u);
    const u32 wave = tid / BENCH_WAVE_WIDTH;
    const u32 wave_count =
        (static_cast<u32>(blockDim.x) + BENCH_WAVE_WIDTH - 1u) / BENCH_WAVE_WIDTH;
#pragma unroll
    for (int off = BENCH_WAVE_WIDTH / 2; off > 0; off >>= 1) {
        const float ov = __shfl_down(best, off);
        const u32 oi = __shfl_down(best_i, off);
        if (lane < static_cast<u32>(off) && (ov > best || (ov == best && oi < best_i))) {
            best = ov; best_i = oi;
        }
    }
    if (lane == 0) { wave_values[wave] = best; wave_indices[wave] = best_i; }
    __syncthreads();
    if (wave == 0) {
        best = lane < wave_count ? wave_values[lane] : -3.402823466e38f;
        best_i = lane < wave_count ? wave_indices[lane] : 0xffffffffu;
#pragma unroll
        for (int off = BENCH_WAVE_WIDTH / 2; off > 0; off >>= 1) {
            const float ov = __shfl_down(best, off);
            const u32 oi = __shfl_down(best_i, off);
            if (lane < static_cast<u32>(off) && (ov > best || (ov == best && oi < best_i))) {
                best = ov; best_i = oi;
            }
        }
        if (lane == 0) {
            const u32 out_idx = output_offset + row;
            radiowave::buffer_store_u32(
                out_resource, out_idx,
                radiowave::buffer_load_u32(out_resource, out_idx) + best_i + 1u);
        }
    }
}

__device__ __forceinline__ bool better_pair(
    float lhs_value, u32 lhs_index, float rhs_value, u32 rhs_index) {
    return lhs_value > rhs_value || (lhs_value == rhs_value && lhs_index < rhs_index);
}

extern "C" __global__ __launch_bounds__(256, 16)
void sampler_topk(BENCH_ARGS) {
    (void)b;
    __shared__ float values[256];
    __shared__ u32 indices[256];
    __shared__ u32 selected[8];
    const u32 row = workgroup_x();
    const u32 tid = static_cast<u32>(threadIdx.x);
    const u32 wg = static_cast<u32>(blockDim.x);
    const u32 top_k = aux;
    if (row >= n1 || top_k == 0u || top_k > 8u) return;
    if (tid < top_k) selected[tid] = 0xffffffffu;
    __syncthreads();
    const auto a_resource = radiowave::buffer_resource(a);
    const auto out_resource = radiowave::buffer_resource(out);
    for (u32 pass = 0; pass < top_k; ++pass) {
        float best = -3.402823466e38f;
        u32 best_i = 0xffffffffu;
        for (u32 col = tid; col < n0; col += wg) {
            bool already = false;
#pragma unroll
            for (u32 prior = 0; prior < 8u; ++prior) {
                if (prior < pass && selected[prior] == col) already = true;
            }
            const float candidate = __uint_as_float(
                radiowave::buffer_load_u32(a_resource, row * n0 + col));
            if (!already && better_pair(candidate, col, best, best_i)) {
                best = candidate;
                best_i = col;
            }
        }
        values[tid] = best;
        indices[tid] = best_i;
        __syncthreads();
        for (u32 off = wg >> 1; off; off >>= 1) {
            if (tid < off && better_pair(
                    values[tid + off], indices[tid + off], values[tid], indices[tid])) {
                values[tid] = values[tid + off];
                indices[tid] = indices[tid + off];
            }
            __syncthreads();
        }
        if (tid == 0) selected[pass] = indices[0];
        __syncthreads();
    }
    if (tid < top_k) {
        const u32 out_idx = output_offset + row * top_k + tid;
        radiowave::buffer_store_u32(
            out_resource, out_idx,
            radiowave::buffer_load_u32(out_resource, out_idx) + selected[tid] + 1u);
    }
}

extern "C" __global__ __launch_bounds__(256, 16)
void two_stage_partial(BENCH_ARGS) {
    __shared__ float wave_partials[BENCH_WAVE_COUNT];
    const u32 gid = workgroup_x();
    const u32 splits = aux;
    const u32 row = gid / splits;
    const u32 split = gid - row * splits;
    const u32 tid = static_cast<u32>(threadIdx.x);
    if (row >= n1) return;
    const auto a_resource = radiowave::buffer_resource(a);
    const auto b_resource = radiowave::buffer_resource(b);
    const auto out_resource = radiowave::buffer_resource(out);
    const u32 begin = (n0 * split) / splits;
    const u32 end = (n0 * (split + 1u)) / splits;
    float sum = 0.0f;
    for (u32 i = begin + tid; i < end; i += static_cast<u32>(blockDim.x)) {
        const float av = __uint_as_float(
            radiowave::buffer_load_u32(a_resource, i));
        const float bv = __uint_as_float(
            radiowave::buffer_load_u32(b_resource, row * n0 + i));
        sum = fmaf(av, bv, sum);
    }
    sum = wave_sum_f32(sum);
    const u32 lane = tid & (BENCH_WAVE_WIDTH - 1u), wave = tid / BENCH_WAVE_WIDTH;
    const u32 wave_count =
        (static_cast<u32>(blockDim.x) + BENCH_WAVE_WIDTH - 1u) / BENCH_WAVE_WIDTH;
    if (lane == 0) wave_partials[wave] = sum;
    __syncthreads();
    if (wave == 0) {
        float total = lane < wave_count ? wave_partials[lane] : 0.0f;
        total = wave_sum_f32(total);
        if (lane == 0) {
            const u32 out_idx = output_offset + row * splits + split;
            const float previous = __uint_as_float(
                radiowave::buffer_load_u32(out_resource, out_idx));
            radiowave::buffer_store_u32(
                out_resource, out_idx, __float_as_uint(previous + total));
        }
    }
}

extern "C" __global__ __launch_bounds__(256, 16)
void two_stage_final(BENCH_ARGS) {
    (void)a; (void)b; (void)n0;
    __shared__ float wave_partials[BENCH_WAVE_COUNT];
    const u32 row = workgroup_x();
    const u32 tid = static_cast<u32>(threadIdx.x);
    if (row >= n1) return;
    const auto out_resource = radiowave::buffer_resource(out);
    float sum = 0.0f;
    for (u32 i = tid; i < aux; i += static_cast<u32>(blockDim.x))
        sum += __uint_as_float(radiowave::buffer_load_u32(
            out_resource, output_offset + row * aux + i));
    sum = wave_sum_f32(sum);
    const u32 lane = tid & (BENCH_WAVE_WIDTH - 1u), wave = tid / BENCH_WAVE_WIDTH;
    const u32 wave_count =
        (static_cast<u32>(blockDim.x) + BENCH_WAVE_WIDTH - 1u) / BENCH_WAVE_WIDTH;
    if (lane == 0) wave_partials[wave] = sum;
    __syncthreads();
    if (wave == 0) {
        float total = lane < wave_count ? wave_partials[lane] : 0.0f;
        total = wave_sum_f32(total);
        if (lane == 0) {
            const u32 out_idx = output_offset + n1 * aux + row;
            const float previous = __uint_as_float(
                radiowave::buffer_load_u32(out_resource, out_idx));
            radiowave::buffer_store_u32(
                out_resource, out_idx, __float_as_uint(previous + total));
        }
    }
}

__device__ __forceinline__ float load_bf16(const u32* packed, u32 element) {
    const u32 word = packed[element >> 1u];
    return __uint_as_float(((word >> ((element & 1u) * 16u)) & 0xffffu) << 16u);
}

__device__ __forceinline__ void q8_1_quantize_body(BENCH_ARGS) {
    (void)a;
    const u32 gid = workgroup_x();
    const u32 blocks_per_row = (n0 + 31u) >> 5u;
    const u32 row = gid / blocks_per_row;
    const u32 block = gid - row * blocks_per_row;
    const u32 lane = static_cast<u32>(threadIdx.x);
    if (row >= n1 || lane >= 32u) return;
    const u32 element = row * n0 + block * 32u + lane;
    const float value = block * 32u + lane < n0 ? load_bf16(b, element) : 0.0f;
    float amax = fabsf(value);
    float sum = value;
#pragma unroll
    for (int off = 16; off > 0; off >>= 1) {
        amax = fmaxf(amax, __shfl_down(amax, off, 32));
        sum += __shfl_down(sum, off, 32);
    }
    const float d = amax == 0.0f ? 0.0f : amax * (1.0f / 127.0f);
    int q = d == 0.0f ? 0 : __float2int_rn(value / d);
    q = q < -128 ? -128 : (q > 127 ? 127 : q);
    const u32 output_base = output_offset + gid * 9u;
    if (lane < 9u) out[output_base + lane] = 0u;
    __syncthreads();
    if (lane == 0u) {
        const u32 d_bits = static_cast<u32>(__half_as_ushort(__float2half(d)));
        const u32 sum_bits = static_cast<u32>(__half_as_ushort(__float2half(sum)));
        out[output_base] = d_bits | (sum_bits << 16u);
    }
    atomicOr(out + output_base + 1u + (lane >> 2u),
             (static_cast<u32>(q) & 255u) << ((lane & 3u) * 8u));
}

extern "C" __global__ __launch_bounds__(32, 16)
void q8_1_quantize_q4(BENCH_ARGS) {
    q8_1_quantize_body(a, b, out, n0, n1, output_offset, aux);
}

extern "C" __global__ __launch_bounds__(32, 16)
void q8_1_quantize_q6(BENCH_ARGS) {
    q8_1_quantize_body(a, b, out, n0, n1, output_offset, aux);
}

extern "C" __global__ __launch_bounds__(32, 16)
void q8_1_quantize_dense(BENCH_ARGS) {
    q8_1_quantize_body(a, b, out, n0, n1, output_offset, aux);
}

__device__ __forceinline__ u32 q8_word(
    const u32* b, const u32* out, u32 output_offset, u32 scratch_words,
    u32 x_row, u32 blocks_per_row, u32 group) {
    const u32 block = group >> 3u;
    const u32 packed = group & 7u;
    if (scratch_words != 0u) {
        const u32 scratch_base = output_offset - scratch_words;
        return out[scratch_base + (x_row * blocks_per_row + block) * 9u + 1u + packed];
    }
    return b[(x_row * blocks_per_row + block) * 9u + 1u + packed];
}

extern "C" __global__ __launch_bounds__(256, 16)
void q4_selected_dual(BENCH_ARGS) {
    const u32 pair = workgroup_x();
    const u32 tid = static_cast<u32>(threadIdx.x);
    const u32 groups = n0 >> 2;
    const u32 row0 = pair * 2u, row1 = row0 + 1u;
    int acc0 = 0, acc1 = 0;
    for (u32 g = tid; g < groups; g += static_cast<u32>(blockDim.x)) {
        const u32 x_row0 = (row0 / 1024u) & 3u;
        const u32 x_row1 = (row1 / 1024u) & 3u;
        const u32 x0 = q8_word(b, out, output_offset, aux, x_row0, n0 >> 5u, g);
        const u32 x1 = q8_word(b, out, output_offset, aux, x_row1, n0 >> 5u, g);
        const u32 x = x0;
        const int sx = dot_u8_s8(0x01010101u, x, 0);
        if (row0 < n1) acc0 += dot_u8_s8(a[row0 * groups + g], x, 0) - 8 * sx;
        if (row1 < n1) {
            const int sx1 = dot_u8_s8(0x01010101u, x1, 0);
            acc1 += dot_u8_s8(a[row1 * groups + g], x1, 0) - 8 * sx1;
        }
    }
    acc0 = wave_sum_i32(acc0); acc1 = wave_sum_i32(acc1);
    const u32 lane = tid & (BENCH_WAVE_WIDTH - 1u);
    const u32 wave = tid / BENCH_WAVE_WIDTH;
    const u32 wave_count =
        (static_cast<u32>(blockDim.x) + BENCH_WAVE_WIDTH - 1u) / BENCH_WAVE_WIDTH;
    __shared__ int partial0[BENCH_WAVE_COUNT];
    __shared__ int partial1[BENCH_WAVE_COUNT];
    if (lane == 0u) { partial0[wave] = acc0; partial1[wave] = acc1; }
    __syncthreads();
    if (wave == 0u) {
        acc0 = lane < wave_count ? partial0[lane] : 0;
        acc1 = lane < wave_count ? partial1[lane] : 0;
        acc0 = wave_sum_i32(acc0); acc1 = wave_sum_i32(acc1);
    }
    if (tid == 0u) {
        if (row0 < n1) output_add_u32(out, output_offset + row0, static_cast<u32>(acc0));
        if (row1 < n1) output_add_u32(out, output_offset + row1, static_cast<u32>(acc1));
    }
}

extern "C" __global__ __launch_bounds__(256, 16)
void q6_x8(BENCH_ARGS) {
    const u32 tile = workgroup_x();
    const u32 tid = static_cast<u32>(threadIdx.x);
    const u32 groups = n0 >> 2;
    const u32 row0 = tile * 8u;
    int acc[8] = {};
    for (u32 g = tid; g < groups; g += static_cast<u32>(blockDim.x)) {
        const u32 x_row = row0 / 2048u;
        const u32 x = q8_word(b, out, output_offset, aux, x_row, n0 >> 5u, g);
        const int sx32 = 32 * dot_u8_s8(0x01010101u, x, 0);
#pragma unroll
        for (u32 r = 0; r < 8; ++r)
            if (row0 + r < n1) acc[r] += dot_u8_s8(a[(row0 + r) * groups + g], x, 0) - sx32;
    }
#pragma unroll
    for (u32 r = 0; r < 8; ++r) acc[r] = wave_sum_i32(acc[r]);
    const u32 lane = tid & (BENCH_WAVE_WIDTH - 1u);
    const u32 wave = tid / BENCH_WAVE_WIDTH;
    const u32 wave_count =
        (static_cast<u32>(blockDim.x) + BENCH_WAVE_WIDTH - 1u) / BENCH_WAVE_WIDTH;
    __shared__ int partial[8][BENCH_WAVE_COUNT];
    if (lane == 0u) for (u32 r = 0; r < 8; ++r) partial[r][wave] = acc[r];
    __syncthreads();
    if (wave == 0u) for (u32 r = 0; r < 8; ++r) {
        acc[r] = lane < wave_count ? partial[r][lane] : 0;
        acc[r] = wave_sum_i32(acc[r]);
    }
    if (tid == 0u) {
#pragma unroll
        for (u32 r = 0; r < 8; ++r)
            if (row0 + r < n1) {
                output_add_u32(out, output_offset + row0 + r, static_cast<u32>(acc[r]));
            }
    }
}

template <u32 TILE>
__device__ __forceinline__ void dense_q8_body(BENCH_ARGS) {
    const u32 tile = workgroup_x();
    const u32 tid = static_cast<u32>(threadIdx.x);
    const u32 groups = n0 >> 2;
    const u32 row0 = tile * TILE;
    int acc[TILE] = {};
    for (u32 g = tid; g < groups; g += static_cast<u32>(blockDim.x)) {
        const u32 x_row = row0 / 2048u;
        const u32 x = q8_word(b, out, output_offset, aux, x_row, n0 >> 5u, g);
#pragma unroll
        for (u32 r = 0; r < TILE; ++r)
            if (row0 + r < n1) acc[r] = dot_s8_s8(a[(row0 + r) * groups + g], x, acc[r]);
    }
#pragma unroll
    for (u32 r = 0; r < TILE; ++r) acc[r] = wave_sum_i32(acc[r]);
    if (tid == 0u) {
#pragma unroll
        for (u32 r = 0; r < TILE; ++r)
            if (row0 + r < n1) {
                output_add_u32(out, output_offset + row0 + r, static_cast<u32>(acc[r]));
            }
    }
}

extern "C" __global__ __launch_bounds__(256, 16)
void dense_q8(BENCH_ARGS) {
    dense_q8_body<4>(a, b, out, n0, n1, output_offset, aux);
}

extern "C" __global__ __launch_bounds__(256, 16)
void dense_q8_single(BENCH_ARGS) {
    dense_q8_body<1>(a, b, out, n0, n1, output_offset, aux);
}

#undef BENCH_ARGS
#undef BENCH_QUANT_BLOCK
#undef BENCH_WAVE_COUNT
#undef BENCH_WAVE_WIDTH
