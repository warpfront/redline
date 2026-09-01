// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
//
// Microwave packed-dot regression: ACO body packaged in an LLVM HSACO.
//
//   hipcc --offload-arch=gfx1201 -O3 \
//     -I crates/radiowave/include \
//     -include crates/radiowave/include/radiowave/hip.h \
//     examples/hipfire-6409/kernels/microwave/selftest.hip -o /tmp/mw-selftest
//
// Wave64: add -mwavefrontsize64 -DHIPFIRE_BENCH_WAVE64=1
// q6:     add -DSELFTEST_MODE=2
// q4:     add -DSELFTEST_MODE=1
// scalar: add -DSELFTEST_MODE=3
#include <hip/hip_runtime.h>
#include <cstdint>
#include <cstdio>
#include <vector>

using u32 = uint32_t;

#include "../hipfire-inhouse-microwave/hipfire_6409.hip"

#ifndef SELFTEST_MODE
#define SELFTEST_MODE 0
#endif

u32 packed_word(u32 seed) {
    u32 word = seed * 1664525u + 1013904223u;
    word ^= word >> 16;
    word *= 0x7feb352du;
    word ^= word >> 15;
    word *= 0x846ca68bu;
    word ^= word >> 16;
    return word;
}

int unpack_i8(u32 word, int lane) {
    int x = int((word >> (lane * 8)) & 0xFFu);
    return x >= 128 ? x - 256 : x;
}
int unpack_u8(u32 word, int lane) { return int((word >> (lane * 8)) & 0xFFu); }

int host_dot_s8_s8(u32 a, u32 b) {
    int s = 0;
    for (int l = 0; l < 4; ++l) s += unpack_i8(a, l) * unpack_i8(b, l);
    return s;
}
int host_dot_u8_s8(u32 a, u32 b) {
    int s = 0;
    for (int l = 0; l < 4; ++l) s += unpack_u8(a, l) * unpack_i8(b, l);
    return s;
}
int host_scalar_q4(u32 a, u32 b) {
    int s = 0;
    for (int l = 0; l < 4; ++l) {
        const int q = int((a >> (l * 8)) & 15u) - 8;
        s += q * unpack_i8(b, l);
    }
    return s;
}

int host_word(u32 w, u32 x) {
#if SELFTEST_MODE == 0
    return host_dot_s8_s8(w, x);
#elif SELFTEST_MODE == 1
    return host_dot_u8_s8(w, x);
#elif SELFTEST_MODE == 2
    return host_dot_u8_s8(w, x) - 32 * host_dot_u8_s8(0x01010101u, x);
#else
    return host_scalar_q4(w, x);
#endif
}

int main() {
    const u32 n0 = 32768;
    const u32 n1 = 64;
    const u32 groups = 16;
    const u32 block = 64;
    size_t len = 1;
    while (len < (size_t)n0 * n1 * groups) len <<= 1;
    const u32 aux = (u32)(len - 1);
    printf("mode=%d n0=%u n1=%u aux=0x%x len=%zu wave=%s\n", SELFTEST_MODE, n0, n1, aux, len,
#ifdef HIPFIRE_BENCH_WAVE64
           "64"
#else
           "32"
#endif
    );
    std::vector<u32> ha(len), hb(len), ho(n0, 0), expected(n0, 0);
    for (size_t i = 0; i < len; ++i) {
        ha[i] = packed_word((u32)i * 17u + 3u);
        hb[i] = packed_word((u32)i * 29u + 7u);
    }
    for (u32 idx = 0; idx < n0; ++idx) {
        int sum = 0;
        for (u32 iter = 0; iter < n1; ++iter) {
            const u32 base = ((iter * n0 + idx) * groups) & aux;
            for (u32 g = 0; g < groups; ++g) {
                sum += host_word(ha[(base + g) & aux], hb[(base + g) & aux]);
            }
        }
        expected[idx] = (u32)sum;
    }
    u32 *da = nullptr, *db = nullptr, *dout = nullptr;
    if (hipMalloc(&da, len * 4) != hipSuccess || hipMalloc(&db, len * 4) != hipSuccess ||
        hipMalloc(&dout, n0 * 4) != hipSuccess) {
        printf("hipMalloc failed\n");
        return 1;
    }
    hipMemcpy(da, ha.data(), len * 4, hipMemcpyHostToDevice);
    hipMemcpy(db, hb.data(), len * 4, hipMemcpyHostToDevice);
    hipMemset(dout, 0, n0 * 4);
    const u32 grid = (n0 + block - 1u) / block;
    printf("launch grid=%u block=%u\n", grid, block);
#if SELFTEST_MODE == 0
    hipLaunchKernelGGL(dot_q8, dim3(grid), dim3(block), 0, 0, da, db, dout, n0, n1, 0u, aux);
#elif SELFTEST_MODE == 1
    hipLaunchKernelGGL(dot_q4, dim3(grid), dim3(block), 0, 0, da, db, dout, n0, n1, 0u, aux);
#elif SELFTEST_MODE == 2
    hipLaunchKernelGGL(dot_q6, dim3(grid), dim3(block), 0, 0, da, db, dout, n0, n1, 0u, aux);
#else
    hipLaunchKernelGGL(dot_scalar, dim3(grid), dim3(block), 0, 0, da, db, dout, n0, n1, 0u, aux);
#endif
    const hipError_t err = hipGetLastError();
    if (err != hipSuccess) {
        printf("launch error %s\n", hipGetErrorString(err));
        return 1;
    }
    hipDeviceSynchronize();
    hipMemcpy(ho.data(), dout, n0 * 4, hipMemcpyDeviceToHost);
    long bad = 0;
    for (u32 i = 0; i < n0; ++i) {
        if (ho[i] != expected[i]) {
            if (bad < 10)
                printf("mismatch %u: got %u expected %u\n", i, ho[i], expected[i]);
            ++bad;
        }
    }
    hipDeviceProp_t p;
    hipGetDeviceProperties(&p, 0);
    printf("mismatches=%ld %s\n", bad, bad ? "FAIL" : "PASS");
    printf("%s (%s)\n", p.name, p.gcnArchName);
    return bad ? 2 : 0;
}
