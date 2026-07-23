/* SPDX-License-Identifier: Apache-2.0 */
/* SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> */
/*
 * Multi-kernel decode A/B: a 2-kernel dependent chain (stage1 -> stage2) per
 * token, hipGraph vs redline retained PM4 IB. Exercises per-token kernarg
 * mutation AND the inter-dispatch RMW boundary together on a realistic
 * multi-kernel-per-token decode shape. Same kernels, host us/token, correctness
 * gated (acc == sum_t(2*t) == T*(T+1)).
 *
 * Build:
 *   hipcc -x hip decode_chain_ab.c -I <include> -L <libdir> -lredline_dispatch \
 *     -Wl,-rpath,<libdir> -lpthread -ldl -lm -o decode_chain_ab
 * Run:
 *   ROCR_VISIBLE_DEVICES=0 ./decode_chain_ab 512 decode_chain.co
 */
#include "redline_dispatch.h"
#include <hip/hip_runtime.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#define HCHECK(x)                                                                  \
    do {                                                                           \
        hipError_t _e = (x);                                                       \
        if (_e != hipSuccess) {                                                    \
            fprintf(stderr, "hip error @%d: %s\n", __LINE__, hipGetErrorString(_e)); \
            return 1;                                                              \
        }                                                                          \
    } while (0)

static double now_us(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec * 1e6 + ts.tv_nsec / 1e3;
}

int main(int argc, char **argv) {
    int T = argc > 1 ? atoi(argv[1]) : 512;
    const char *co = argc > 2 ? argv[2] : "decode_chain.co";
    unsigned long long expected = (unsigned long long)T * (T + 1); /* sum 2*t */

    FILE *f = fopen(co, "rb");
    if (!f) { perror("open code object"); return 1; }
    fseek(f, 0, SEEK_END);
    long len = ftell(f);
    fseek(f, 0, SEEK_SET);
    unsigned char *code = (unsigned char *)malloc(len);
    if (fread(code, 1, len, f) != (size_t)len) { fprintf(stderr, "read co\n"); return 1; }
    fclose(f);

    /* ---- hipGraph arm: 2-node dependent graph -------------------------- */
    hipModule_t hmod;
    hipFunction_t f1, f2;
    HCHECK(hipModuleLoadData(&hmod, code));
    HCHECK(hipModuleGetFunction(&f1, hmod, "stage1"));
    HCHECK(hipModuleGetFunction(&f2, hmod, "stage2"));

    unsigned int *g_acc = NULL, *g_tmp = NULL;
    HCHECK(hipMalloc(&g_acc, 4));
    HCHECK(hipMalloc(&g_tmp, 4));
    HCHECK(hipMemset(g_acc, 0, 4));

    hipStream_t stream;
    HCHECK(hipStreamCreate(&stream));
    unsigned int val = 0;
    void *k1[] = {&g_tmp, &val};
    void *k2[] = {&g_acc, &g_tmp};

    HCHECK(hipStreamBeginCapture(stream, hipStreamCaptureModeThreadLocal));
    HCHECK(hipModuleLaunchKernel(f1, 1, 1, 1, 1, 1, 1, 0, stream, k1, NULL));
    HCHECK(hipModuleLaunchKernel(f2, 1, 1, 1, 1, 1, 1, 0, stream, k2, NULL));
    hipGraph_t graph;
    HCHECK(hipStreamEndCapture(stream, &graph));

    /* find stage1's node so we can update its scalar per token */
    size_t nnodes = 0;
    HCHECK(hipGraphGetNodes(graph, NULL, &nnodes));
    hipGraphNode_t *nodes = (hipGraphNode_t *)calloc(nnodes, sizeof(hipGraphNode_t));
    HCHECK(hipGraphGetNodes(graph, nodes, &nnodes));
    hipGraphNode_t n1 = NULL;
    for (size_t i = 0; i < nnodes; i++) {
        hipGraphNodeType t;
        if (hipGraphNodeGetType(nodes[i], &t) != hipSuccess || t != hipGraphNodeTypeKernel) continue;
        hipKernelNodeParams gp;
        memset(&gp, 0, sizeof(gp));
        if (hipGraphKernelNodeGetParams(nodes[i], &gp) == hipSuccess && gp.func == (void *)f1) {
            n1 = nodes[i];
            break;
        }
    }
    if (!n1) { fprintf(stderr, "stage1 node not found\n"); return 1; }

    hipGraphExec_t gexec;
    HCHECK(hipGraphInstantiate(&gexec, graph, NULL, NULL, 0));

    double t0 = now_us();
    for (unsigned int t = 1; t <= (unsigned)T; t++) {
        val = t;
        hipKernelNodeParams p;
        memset(&p, 0, sizeof(p));
        p.func = (void *)f1;
        p.gridDim = dim3(1, 1, 1);
        p.blockDim = dim3(1, 1, 1);
        p.kernelParams = k1;
        HCHECK(hipGraphExecKernelNodeSetParams(gexec, n1, &p));
        HCHECK(hipGraphLaunch(gexec, stream));
        HCHECK(hipStreamSynchronize(stream));
    }
    double hipgraph_us = (now_us() - t0) / T;
    unsigned int hg = 0;
    HCHECK(hipMemcpy(&hg, g_acc, 4, hipMemcpyDeviceToHost));

    /* ---- redline arm: stage1 -> RMW boundary -> stage2 ----------------- */
    unsigned int *r_acc = NULL, *r_tmp = NULL;
    HCHECK(hipMalloc(&r_acc, 4));
    HCHECK(hipMalloc(&r_tmp, 4));
    HCHECK(hipMemset(r_acc, 0, 4));

    RlGpu *gpu = rl_gpu_new(0);
    if (!gpu) { fprintf(stderr, "rl_gpu_new failed\n"); return 1; }
    RlModule *mod = NULL;
    if (rl_gpu_load_module(gpu, code, (size_t)len, &mod) != RL_OK) {
        fprintf(stderr, "rl load_module failed\n"); return 1;
    }

    unsigned char karg1[256], karg2[256];
    memset(karg1, 0, sizeof(karg1));
    memset(karg2, 0, sizeof(karg2));
    unsigned long long ptmp = (unsigned long long)(uintptr_t)r_tmp;
    unsigned long long pacc = (unsigned long long)(uintptr_t)r_acc;
    memcpy(karg1, &ptmp, 8);              /* stage1: tmp@0, val@8 (starts 0) */
    memcpy(karg2, &pacc, 8);             /* stage2: acc@0, tmp@8 */
    memcpy(karg2 + 8, &ptmp, 8);

    RlPm4Builder *b = rl_pm4_builder_new(gpu);
    if (rl_pm4_dispatch(b, mod, "stage1.kd", 1, 1, 1, 1, 1, 1, 0, karg1, sizeof(karg1)) != RL_OK) {
        fprintf(stderr, "rl dispatch stage1 failed\n"); return 1;
    }
    /* inter-dispatch RMW boundary selected for the stage2 consumer */
    if (rl_pm4_wait_rmw(b, mod, "stage2.kd") != RL_OK) {
        fprintf(stderr, "rl wait_rmw failed\n"); return 1;
    }
    if (rl_pm4_dispatch(b, mod, "stage2.kd", 1, 1, 1, 1, 1, 1, 0, karg2, sizeof(karg2)) != RL_OK) {
        fprintf(stderr, "rl dispatch stage2 failed\n"); return 1;
    }
    RlPm4Ib *ib = NULL;
    if (rl_pm4_finalize(gpu, b, &ib) != RL_OK) { fprintf(stderr, "rl finalize failed\n"); return 1; }

    t0 = now_us();
    for (unsigned int t = 1; t <= (unsigned)T; t++) {
        rl_pm4_ib_set_kernargs(ib, 0, 8, (const uint8_t *)&t, sizeof(t)); /* stage1 val */
        rl_pm4_replay(ib);
    }
    double redline_us = (now_us() - t0) / T;
    unsigned int rl = 0;
    HCHECK(hipMemcpy(&rl, r_acc, 4, hipMemcpyDeviceToHost));

    int ok = (hg == (unsigned)expected) && (rl == (unsigned)expected);
    printf("multi-kernel decode A/B over %d tokens (acc must == %llu):\n", T, expected);
    printf("  hipGraph (2-node, setParams+launch): acc=%u  %8.2f us/token\n", hg, hipgraph_us);
    printf("  redline  (stage1|rmw|stage2, replay): acc=%u  %8.2f us/token\n", rl, redline_us);
    printf("  redline/hipGraph host speedup: %.2fx   [%s]\n",
           hipgraph_us / redline_us, ok ? "PASS" : "FAIL");

    rl_pm4_ib_free(ib);
    rl_module_free(mod);
    rl_gpu_free(gpu);
    return ok ? 0 : 1;
}
