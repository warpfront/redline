/* SPDX-License-Identifier: Apache-2.0 */
/* SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> */
/*
 * Real-GPU C-ABI smoke = the engine integration pattern, end to end:
 *   HIP (the "engine") allocates a device counter, Redline records N dispatches
 *   of an atomic-increment kernel referencing THAT pointer, lowers to one
 *   retained PM4 IB, replays it, and HIP reads the counter back. It must equal N
 *   iff every dispatch executed against the engine-owned buffer.
 *
 * Build (hipcc links HIP; load Redline from <libdir>):
 *   hipcc -x hip gpu_smoke.c -I <include> -L <libdir> -lredline_dispatch \
 *     -Wl,-rpath,<libdir> -lpthread -ldl -lm -o gpu_smoke
 * Run (one device; ROCR_VISIBLE_DEVICES filters HIP too, so both target it):
 *   ROCR_VISIBLE_DEVICES=3 ./gpu_smoke 256 counter.co counter.radiowave.json
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
    int N = argc > 1 ? atoi(argv[1]) : 256;
    const char *co = argc > 2 ? argv[2] : "bench/floor_kernel_ctr.co";
    const char *radiowave_manifest = argc > 3 ? argv[3] : NULL;

    /* read the code object bytes */
    FILE *f = fopen(co, "rb");
    if (!f) { perror("open code object"); return 1; }
    fseek(f, 0, SEEK_END);
    long len = ftell(f);
    fseek(f, 0, SEEK_SET);
    unsigned char *code = (unsigned char *)malloc(len);
    if (fread(code, 1, len, f) != (size_t)len) { fprintf(stderr, "read co\n"); return 1; }
    fclose(f);

    /* the engine (HIP) owns the counter */
    unsigned int *d_counter = NULL;
    HCHECK(hipMalloc(&d_counter, sizeof(unsigned int)));
    HCHECK(hipMemset(d_counter, 0, sizeof(unsigned int)));

    /* redline: bind the GPU, load the module, build N dispatches against d_counter */
    RlGpu *gpu = rl_gpu_new(0);
    if (!gpu) { fprintf(stderr, "rl_gpu_new failed\n"); return 1; }
    RlModule *mod = NULL;
    int load_rc = RL_OK;
    if (radiowave_manifest) {
        FILE *mf = fopen(radiowave_manifest, "rb");
        if (!mf) { perror("open Radiowave manifest"); return 1; }
        fseek(mf, 0, SEEK_END);
        long manifest_len = ftell(mf);
        fseek(mf, 0, SEEK_SET);
        unsigned char *manifest = (unsigned char *)malloc(manifest_len);
        if (fread(manifest, 1, manifest_len, mf) != (size_t)manifest_len) {
            fprintf(stderr, "read Radiowave manifest\n");
            return 1;
        }
        fclose(mf);
        load_rc = rl_gpu_load_module_radiowave(
            gpu, code, (size_t)len, manifest, (size_t)manifest_len, &mod);
        free(manifest);
    } else {
        load_rc = rl_gpu_load_module(gpu, code, (size_t)len, &mod);
    }
    if (load_rc != RL_OK) {
        fprintf(stderr, "module load failed: %d\n", load_rc);
        return 1;
    }
    long ksz = rl_module_kernarg_size(mod, "ctr_k.kd");
    size_t klen = ksz > 0 ? (size_t)ksz : 8;

    /* kernarg segment: first 8 bytes = the engine's device pointer */
    unsigned char karg[512];
    memset(karg, 0, sizeof(karg));
    unsigned long long p = (unsigned long long)(uintptr_t)d_counter;
    memcpy(karg, &p, sizeof(p));

    RlPm4Builder *b = rl_pm4_builder_new(gpu);
    if (!b) { fprintf(stderr, "builder_new failed\n"); return 1; }
    for (int i = 0; i < N; i++) {
        if (rl_pm4_dispatch(b, mod, "ctr_k.kd", 1, 1, 1, 1, 1, 1, 0, karg, klen) != RL_OK) {
            fprintf(stderr, "dispatch %d failed\n", i);
            return 1;
        }
        if (i + 1 < N && rl_pm4_wait_rmw(b, mod, "ctr_k.kd") != RL_OK) {
            fprintf(stderr, "dependency %d failed\n", i);
            return 1;
        }
    }
    RlPm4Ib *ib = NULL;
    if (rl_pm4_finalize(gpu, b, &ib) != RL_OK) { fprintf(stderr, "finalize failed\n"); return 1; }
    if (rl_pm4_replay(ib) != RL_OK) { fprintf(stderr, "replay failed\n"); return 1; }

    /* the engine reads its counter back */
    unsigned int h = 0;
    HCHECK(hipMemcpy(&h, d_counter, sizeof(unsigned int), hipMemcpyDeviceToHost));
    int pass = (h == (unsigned)N);
    printf("real-GPU C-ABI gate: counter = %u / %d certified=%s [%s]\n",
           h, N, rl_module_radiowave_certified(mod) ? "yes" : "no",
           pass ? "PASS" : "FAIL");

    rl_pm4_ib_free(ib);
    rl_module_free(mod);
    rl_gpu_free(gpu);
    HCHECK(hipFree(d_counter));
    free(code);
    return pass ? 0 : 1;
}
