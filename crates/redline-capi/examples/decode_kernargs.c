/* SPDX-License-Identifier: Apache-2.0 */
/* SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> */
/*
 * Real-GPU C-ABI decode pattern: build the retained PM4 IB ONCE, then per token
 * patch only the scalar that changed (rl_pm4_ib_set_kernargs) and replay. No IB
 * rebuild, one doorbell per token -- the per-token update path an inference
 * engine wants. The accumulator kernel adds a per-token value into an
 * engine-owned (HIP-allocated) counter; after T tokens it must equal sum(1..T)
 * iff every replay observed its patched scalar.
 *
 * Build (hipcc links HIP; load Redline from <libdir>):
 *   hipcc -x hip decode_kernargs.c -I <include> -L <libdir> -lredline_dispatch \
 *     -Wl,-rpath,<libdir> -lpthread -ldl -lm -o decode_kernargs
 * Run:
 *   ROCR_VISIBLE_DEVICES=0 ./decode_kernargs 64 acc.co
 */
#include "redline_dispatch.h"
#include <hip/hip_runtime.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define HCHECK(x)                                                                  \
    do {                                                                           \
        hipError_t _e = (x);                                                       \
        if (_e != hipSuccess) {                                                    \
            fprintf(stderr, "hip error: %s\n", hipGetErrorString(_e));             \
            return 1;                                                              \
        }                                                                          \
    } while (0)

int main(int argc, char **argv) {
    int T = argc > 1 ? atoi(argv[1]) : 64;
    const char *co = argc > 2 ? argv[2] : "acc.co";

    FILE *f = fopen(co, "rb");
    if (!f) { perror("open code object"); return 1; }
    fseek(f, 0, SEEK_END);
    long len = ftell(f);
    fseek(f, 0, SEEK_SET);
    unsigned char *code = (unsigned char *)malloc(len);
    if (fread(code, 1, len, f) != (size_t)len) { fprintf(stderr, "read co\n"); return 1; }
    fclose(f);

    /* the engine (HIP) owns the accumulator */
    unsigned int *d_acc = NULL;
    HCHECK(hipMalloc(&d_acc, sizeof(unsigned int)));
    HCHECK(hipMemset(d_acc, 0, sizeof(unsigned int)));

    RlGpu *gpu = rl_gpu_new(0);
    if (!gpu) { fprintf(stderr, "rl_gpu_new failed\n"); return 1; }
    RlModule *mod = NULL;
    if (rl_gpu_load_module(gpu, code, (size_t)len, &mod) != RL_OK) {
        fprintf(stderr, "load_module failed\n");
        return 1;
    }

    /* kernarg layout for acc_k(unsigned* acc, unsigned val): acc@0 (8B), val@8 (4B) */
    unsigned char karg[512];
    memset(karg, 0, sizeof(karg));
    unsigned long long p = (unsigned long long)(uintptr_t)d_acc;
    memcpy(karg, &p, sizeof(p)); /* val starts at 0 */

    /* build the retained IB once: a single acc_k dispatch */
    RlPm4Builder *b = rl_pm4_builder_new(gpu);
    if (!b) { fprintf(stderr, "builder_new failed\n"); return 1; }
    if (rl_pm4_dispatch(b, mod, "acc_k.kd", 1, 1, 1, 1, 1, 1, 0, karg, sizeof(karg)) != RL_OK) {
        fprintf(stderr, "dispatch failed\n");
        return 1;
    }
    RlPm4Ib *ib = NULL;
    if (rl_pm4_finalize(gpu, b, &ib) != RL_OK) { fprintf(stderr, "finalize failed\n"); return 1; }

    /* decode loop: patch only the 4-byte val at offset 8, then replay */
    unsigned long long expected = 0;
    for (unsigned int t = 1; t <= (unsigned)T; t++) {
        if (rl_pm4_ib_set_kernargs(ib, 0, 8, (const uint8_t *)&t, sizeof(t)) != RL_OK) {
            fprintf(stderr, "set_kernargs failed at t=%u\n", t);
            return 1;
        }
        if (rl_pm4_replay(ib) != RL_OK) { fprintf(stderr, "replay failed at t=%u\n", t); return 1; }
        expected += t;
    }

    unsigned int h = 0;
    HCHECK(hipMemcpy(&h, d_acc, sizeof(unsigned int), hipMemcpyDeviceToHost));
    int pass = (h == (unsigned)expected);
    printf("real-GPU C-ABI decode gate: acc = %u / %llu over %d tokens [%s]\n",
           h, expected, T, pass ? "PASS" : "FAIL");

    rl_pm4_ib_free(ib);
    rl_module_free(mod);
    rl_gpu_free(gpu);
    HCHECK(hipFree(d_acc));
    free(code);
    return pass ? 0 : 1;
}
