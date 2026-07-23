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

extern "C" __global__ __launch_bounds__(256, 16)
void dot_q8(BENCH_ARGS) { dot_body<0>(a, b, out, n0, n1, output_offset, aux); }
extern "C" __global__ __launch_bounds__(256, 16)
void dot_q4(BENCH_ARGS) { dot_body<1>(a, b, out, n0, n1, output_offset, aux); }
extern "C" __global__ __launch_bounds__(256, 16)
void dot_q6(BENCH_ARGS) { dot_body<2>(a, b, out, n0, n1, output_offset, aux); }
extern "C" __global__ __launch_bounds__(256, 16)
void dot_scalar(BENCH_ARGS) { dot_body<3>(a, b, out, n0, n1, output_offset, aux); }

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

// Register-blocked q4 dual-row dp4a: keep launch mapping pair→2 rows (harness
// grid_groups = ceil(n1/2)), but hold K_UNROLL independent accumulator banks per
// row so each thread keeps more integer dots in-flight and hides VMEM/dot latency.
extern "C" __global__ __launch_bounds__(256, 4)
void q4_selected_dual(BENCH_ARGS) {
    constexpr u32 ROW_TILE = 2u;
    constexpr u32 K_UNROLL = 4u;
    const u32 pair = workgroup_x();
    const u32 tid = static_cast<u32>(threadIdx.x);
    const u32 wg = static_cast<u32>(blockDim.x);
    const u32 groups = n0 >> 2;
    const u32 row0 = pair * ROW_TILE;
    const u32 blocks_per_row = n0 >> 5u;
    const u32 x_row0 = (row0 / 1024u) & 3u;
    const u32 x_row1 = ((row0 + 1u) / 1024u) & 3u;
    const bool live0 = row0 < n1;
    const bool live1 = (row0 + 1u) < n1;

    // acc[row][k_bank] — larger accumulator array for ILP; integer sum is associative.
    int acc[ROW_TILE][K_UNROLL] = {};

    u32 g = tid;
    const u32 span = K_UNROLL * wg;
    for (; g + span - wg < groups; g += span) {
        u32 x0s[K_UNROLL], x1s[K_UNROLL];
        u32 a0s[K_UNROLL], a1s[K_UNROLL];
#pragma unroll
        for (u32 u = 0; u < K_UNROLL; ++u) {
            const u32 gu = g + u * wg;
            x0s[u] = q8_word(b, out, output_offset, aux, x_row0, blocks_per_row, gu);
            x1s[u] = q8_word(b, out, output_offset, aux, x_row1, blocks_per_row, gu);
            if (live0) a0s[u] = a[row0 * groups + gu];
            if (live1) a1s[u] = a[(row0 + 1u) * groups + gu];
        }
#pragma unroll
        for (u32 u = 0; u < K_UNROLL; ++u) {
            if (live0) {
                const int sx = dot_u8_s8(0x01010101u, x0s[u], 0);
                acc[0][u] += dot_u8_s8(a0s[u], x0s[u], 0) - 8 * sx;
            }
            if (live1) {
                const int sx1 = dot_u8_s8(0x01010101u, x1s[u], 0);
                acc[1][u] += dot_u8_s8(a1s[u], x1s[u], 0) - 8 * sx1;
            }
        }
    }
    for (; g < groups; g += wg) {
        const u32 x0 = q8_word(b, out, output_offset, aux, x_row0, blocks_per_row, g);
        const u32 x1 = q8_word(b, out, output_offset, aux, x_row1, blocks_per_row, g);
        if (live0) {
            const int sx = dot_u8_s8(0x01010101u, x0, 0);
            acc[0][0] += dot_u8_s8(a[row0 * groups + g], x0, 0) - 8 * sx;
        }
        if (live1) {
            const int sx1 = dot_u8_s8(0x01010101u, x1, 0);
            acc[1][0] += dot_u8_s8(a[(row0 + 1u) * groups + g], x1, 0) - 8 * sx1;
        }
    }

    int sum0 = 0, sum1 = 0;
#pragma unroll
    for (u32 u = 0; u < K_UNROLL; ++u) {
        sum0 += acc[0][u];
        sum1 += acc[1][u];
    }
    sum0 = wave_sum_i32(sum0);
    sum1 = wave_sum_i32(sum1);

    const u32 lane = tid & (BENCH_WAVE_WIDTH - 1u);
    const u32 wave = tid / BENCH_WAVE_WIDTH;
    const u32 wave_count = (wg + BENCH_WAVE_WIDTH - 1u) / BENCH_WAVE_WIDTH;
    __shared__ int partial0[BENCH_WAVE_COUNT];
    __shared__ int partial1[BENCH_WAVE_COUNT];
    if (lane == 0u) {
        partial0[wave] = sum0;
        partial1[wave] = sum1;
    }
    __syncthreads();
    if (wave == 0u) {
        sum0 = lane < wave_count ? partial0[lane] : 0;
        sum1 = lane < wave_count ? partial1[lane] : 0;
        sum0 = wave_sum_i32(sum0);
        sum1 = wave_sum_i32(sum1);
    }
    if (tid == 0u) {
        if (live0) output_add_u32(out, output_offset + row0, static_cast<u32>(sum0));
        if (live1) output_add_u32(out, output_offset + row0 + 1u, static_cast<u32>(sum1));
    }
}

extern "C" __global__ __launch_bounds__(256, 16)
void q6_x8(BENCH_ARGS) {
    const u32 tile = workgroup_x();
    const u32 tid = static_cast<u32>(threadIdx.x);
    const u32 wg = static_cast<u32>(blockDim.x);
    const u32 groups = n0 >> 2;
    const u32 row0 = tile * 8u;
    // Early-out empty tiles (past n1).
    if (row0 >= n1) return;

    // Hoist loop-invariants: x-row selection, block stride, buffer descriptors.
    const u32 x_row = row0 / 2048u;
    const u32 blocks_per_row = n0 >> 5u;
    const u32 x_row_base = x_row * blocks_per_row;
    const auto a_resource = radiowave::buffer_resource(a);
    // Scratch (quantize_plus_dot) lives just below output_offset in `out`;
    // prequantized path reads packed q8 from `b`.
    const auto x_resource = aux != 0u ? radiowave::buffer_resource(out)
                                      : radiowave::buffer_resource(b);
    const u32 x_base =
        aux != 0u ? (output_offset - aux) + x_row_base * 9u
                  : x_row_base * 9u;

    // Fully-unrolled 8-way row tile (named regs for ILP / dual-issue).
    int acc0 = 0, acc1 = 0, acc2 = 0, acc3 = 0;
    int acc4 = 0, acc5 = 0, acc6 = 0, acc7 = 0;
    const bool r1 = row0 + 1u < n1;
    const bool r2 = row0 + 2u < n1;
    const bool r3 = row0 + 3u < n1;
    const bool r4 = row0 + 4u < n1;
    const bool r5 = row0 + 5u < n1;
    const bool r6 = row0 + 6u < n1;
    const bool r7 = row0 + 7u < n1;
    // Base word offsets for the 8 consecutive weight rows (stride = groups).
    const u32 a0 = row0 * groups;
    const u32 a1 = a0 + groups;
    const u32 a2 = a1 + groups;
    const u32 a3 = a2 + groups;
    const u32 a4 = a3 + groups;
    const u32 a5 = a4 + groups;
    const u32 a6 = a5 + groups;
    const u32 a7 = a6 + groups;

    for (u32 g = tid; g < groups; g += wg) {
        // q8_word via buffer: block = g>>3, packed = g&7, +1 past scale dword.
        const u32 x = radiowave::buffer_load_u32(
            x_resource, x_base + (g >> 3u) * 9u + 1u + (g & 7u));
        // Bias for q6 zero-point: 32 * sum(x_bytes) as signed i8.
        const int sx32 = 32 * dot_u8_s8(0x01010101u, x, 0);

        // Schedule: issue all 8 weight loads, then accumulate (hide latency).
        const u32 w0 = radiowave::buffer_load_u32(a_resource, a0 + g);
        const u32 w1 = r1 ? radiowave::buffer_load_u32(a_resource, a1 + g) : 0u;
        const u32 w2 = r2 ? radiowave::buffer_load_u32(a_resource, a2 + g) : 0u;
        const u32 w3 = r3 ? radiowave::buffer_load_u32(a_resource, a3 + g) : 0u;
        const u32 w4 = r4 ? radiowave::buffer_load_u32(a_resource, a4 + g) : 0u;
        const u32 w5 = r5 ? radiowave::buffer_load_u32(a_resource, a5 + g) : 0u;
        const u32 w6 = r6 ? radiowave::buffer_load_u32(a_resource, a6 + g) : 0u;
        const u32 w7 = r7 ? radiowave::buffer_load_u32(a_resource, a7 + g) : 0u;

        // Bit-exact: acc += udot4(w, x) - 32*sdot_sum(x). Fold bias into c.
        acc0 = dot_u8_s8(w0, x, acc0) - sx32;
        if (r1) acc1 = dot_u8_s8(w1, x, acc1) - sx32;
        if (r2) acc2 = dot_u8_s8(w2, x, acc2) - sx32;
        if (r3) acc3 = dot_u8_s8(w3, x, acc3) - sx32;
        if (r4) acc4 = dot_u8_s8(w4, x, acc4) - sx32;
        if (r5) acc5 = dot_u8_s8(w5, x, acc5) - sx32;
        if (r6) acc6 = dot_u8_s8(w6, x, acc6) - sx32;
        if (r7) acc7 = dot_u8_s8(w7, x, acc7) - sx32;
    }

    // Intra-wave reduce all 8 rows (independent, good ILP).
    acc0 = wave_sum_i32(acc0);
    acc1 = wave_sum_i32(acc1);
    acc2 = wave_sum_i32(acc2);
    acc3 = wave_sum_i32(acc3);
    acc4 = wave_sum_i32(acc4);
    acc5 = wave_sum_i32(acc5);
    acc6 = wave_sum_i32(acc6);
    acc7 = wave_sum_i32(acc7);

    const u32 lane = tid & (BENCH_WAVE_WIDTH - 1u);
    const u32 wave = tid / BENCH_WAVE_WIDTH;
    const u32 wave_count = (wg + BENCH_WAVE_WIDTH - 1u) / BENCH_WAVE_WIDTH;
    __shared__ int partial[8][BENCH_WAVE_COUNT];
    if (lane == 0u) {
        partial[0][wave] = acc0;
        partial[1][wave] = acc1;
        partial[2][wave] = acc2;
        partial[3][wave] = acc3;
        partial[4][wave] = acc4;
        partial[5][wave] = acc5;
        partial[6][wave] = acc6;
        partial[7][wave] = acc7;
    }
    __syncthreads();
    if (wave == 0u) {
        acc0 = lane < wave_count ? partial[0][lane] : 0;
        acc1 = lane < wave_count ? partial[1][lane] : 0;
        acc2 = lane < wave_count ? partial[2][lane] : 0;
        acc3 = lane < wave_count ? partial[3][lane] : 0;
        acc4 = lane < wave_count ? partial[4][lane] : 0;
        acc5 = lane < wave_count ? partial[5][lane] : 0;
        acc6 = lane < wave_count ? partial[6][lane] : 0;
        acc7 = lane < wave_count ? partial[7][lane] : 0;
        acc0 = wave_sum_i32(acc0);
        acc1 = wave_sum_i32(acc1);
        acc2 = wave_sum_i32(acc2);
        acc3 = wave_sum_i32(acc3);
        acc4 = wave_sum_i32(acc4);
        acc5 = wave_sum_i32(acc5);
        acc6 = wave_sum_i32(acc6);
        acc7 = wave_sum_i32(acc7);
    }
    if (tid == 0u) {
        output_add_u32(out, output_offset + row0, static_cast<u32>(acc0));
        if (r1) output_add_u32(out, output_offset + row0 + 1u, static_cast<u32>(acc1));
        if (r2) output_add_u32(out, output_offset + row0 + 2u, static_cast<u32>(acc2));
        if (r3) output_add_u32(out, output_offset + row0 + 3u, static_cast<u32>(acc3));
        if (r4) output_add_u32(out, output_offset + row0 + 4u, static_cast<u32>(acc4));
        if (r5) output_add_u32(out, output_offset + row0 + 5u, static_cast<u32>(acc5));
        if (r6) output_add_u32(out, output_offset + row0 + 6u, static_cast<u32>(acc6));
        if (r7) output_add_u32(out, output_offset + row0 + 7u, static_cast<u32>(acc7));
    }
}

// Register-blocked dense q8: harness grid is ceil(n1/TILE), so TILE stays the
// launch row mapping. K_UNROLL widens the per-thread accumulator array to
// raise ILP across independent group strides; integer banks sum bit-exactly.
template <u32 TILE>
__device__ __forceinline__ void dense_q8_body(BENCH_ARGS) {
    constexpr u32 K_UNROLL = 4u;
    const u32 tile = workgroup_x();
    const u32 tid = static_cast<u32>(threadIdx.x);
    const u32 wg = static_cast<u32>(blockDim.x);
    const u32 groups = n0 >> 2;
    const u32 row0 = tile * TILE;
    const u32 blocks_per_row = n0 >> 5u;
    const u32 x_row = row0 / 2048u;

    int acc[TILE][K_UNROLL] = {};

    u32 g = tid;
    const u32 span = K_UNROLL * wg;
    for (; g + span - wg < groups; g += span) {
        u32 xs[K_UNROLL];
        u32 ws[TILE][K_UNROLL];
#pragma unroll
        for (u32 u = 0; u < K_UNROLL; ++u) {
            const u32 gu = g + u * wg;
            xs[u] = q8_word(b, out, output_offset, aux, x_row, blocks_per_row, gu);
#pragma unroll
            for (u32 r = 0; r < TILE; ++r) {
                if (row0 + r < n1) ws[r][u] = a[(row0 + r) * groups + gu];
            }
        }
#pragma unroll
        for (u32 u = 0; u < K_UNROLL; ++u) {
#pragma unroll
            for (u32 r = 0; r < TILE; ++r) {
                if (row0 + r < n1)
                    acc[r][u] = dot_s8_s8(ws[r][u], xs[u], acc[r][u]);
            }
        }
    }
    for (; g < groups; g += wg) {
        const u32 x = q8_word(b, out, output_offset, aux, x_row, blocks_per_row, g);
#pragma unroll
        for (u32 r = 0; r < TILE; ++r) {
            if (row0 + r < n1)
                acc[r][0] = dot_s8_s8(a[(row0 + r) * groups + g], x, acc[r][0]);
        }
    }

    int sum[TILE] = {};
#pragma unroll
    for (u32 r = 0; r < TILE; ++r) {
#pragma unroll
        for (u32 u = 0; u < K_UNROLL; ++u) sum[r] += acc[r][u];
        sum[r] = wave_sum_i32(sum[r]);
    }

    // Multi-wave LDS tree (no-op extra when block == wave); matches q4/q6.
    const u32 lane = tid & (BENCH_WAVE_WIDTH - 1u);
    const u32 wave = tid / BENCH_WAVE_WIDTH;
    const u32 wave_count = (wg + BENCH_WAVE_WIDTH - 1u) / BENCH_WAVE_WIDTH;
    __shared__ int partial[TILE > 0 ? TILE : 1][BENCH_WAVE_COUNT];
    if (lane == 0u) {
#pragma unroll
        for (u32 r = 0; r < TILE; ++r) partial[r][wave] = sum[r];
    }
    __syncthreads();
    if (wave == 0u) {
#pragma unroll
        for (u32 r = 0; r < TILE; ++r) {
            sum[r] = lane < wave_count ? partial[r][lane] : 0;
            sum[r] = wave_sum_i32(sum[r]);
        }
    }
    if (tid == 0u) {
#pragma unroll
        for (u32 r = 0; r < TILE; ++r)
            if (row0 + r < n1) {
                output_add_u32(out, output_offset + row0 + r, static_cast<u32>(sum[r]));
            }
    }
}

extern "C" __global__ __launch_bounds__(256, 4)
void dense_q8(BENCH_ARGS) {
    dense_q8_body<4>(a, b, out, n0, n1, output_offset, aux);
}

extern "C" __global__ __launch_bounds__(256, 4)
void dense_q8_single(BENCH_ARGS) {
    dense_q8_body<1>(a, b, out, n0, n1, output_offset, aux);
}


#undef BENCH_ARGS
#undef BENCH_QUANT_BLOCK
#undef BENCH_WAVE_COUNT
#undef BENCH_WAVE_WIDTH
