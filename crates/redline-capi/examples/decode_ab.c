/* SPDX-License-Identifier: Apache-2.0 */
/* SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> */
/*
 * Decode A/B on the SAME kernel: hipGraph (capture once, update the scalar node
 * param per token, hipGraphLaunch) vs the redline retained PM4 IB (build once,
 * set_kernargs per token, replay). Both are the "retained graph, per-token arg
 * update" pattern an autoregressive decoder wants; this isolates the submission
 * transport on an identical acc_k(acc, val) workload. Host us/token, correctness
 * gated (acc == sum(1..T)).
 *
 * Build:
 *   hipcc -x hip decode_ab.c -I <include> -L <libdir> -lredline_dispatch \
 *     -Wl,-rpath,<libdir> -lpthread -ldl -lm -o decode_ab
 * Run:
 *   ROCR_VISIBLE_DEVICES=0 ./decode_ab 512 acc.co
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
    const char *co = argc > 2 ? argv[2] : "acc.co";
    unsigned long long expected = (unsigned long long)T * (T + 1) / 2;

    FILE *f = fopen(co, "rb");
    if (!f) { perror("open code object"); return 1; }
    fseek(f, 0, SEEK_END);
    long len = ftell(f);
    fseek(f, 0, SEEK_SET);
    unsigned char *code = (unsigned char *)malloc(len);
    if (fread(code, 1, len, f) != (size_t)len) { fprintf(stderr, "read co\n"); return 1; }
    fclose(f);

    /* ---- hipGraph arm ---------------------------------------------------- */
    hipModule_t hmod;
    hipFunction_t facc;
    HCHECK(hipModuleLoadData(&hmod, code));
    HCHECK(hipModuleGetFunction(&facc, hmod, "acc_k"));

    unsigned int *g_acc = NULL;
    HCHECK(hipMalloc(&g_acc, 4));
    HCHECK(hipMemset(g_acc, 0, 4));

    hipStream_t stream;
    HCHECK(hipStreamCreate(&stream));
    unsigned int val = 0;
    void *kargs[] = {&g_acc, &val};

    /* capture one acc_k launch into a graph */
    HCHECK(hipStreamBeginCapture(stream, hipStreamCaptureModeThreadLocal));
    HCHECK(hipModuleLaunchKernel(facc, 1, 1, 1, 1, 1, 1, 0, stream, kargs, NULL));
    hipGraph_t graph;
    HCHECK(hipStreamEndCapture(stream, &graph));

    /* fetch the single kernel node so we can update its scalar param per token */
    size_t nnodes = 0;
    HCHECK(hipGraphGetNodes(graph, NULL, &nnodes));
    hipGraphNode_t *nodes = (hipGraphNode_t *)calloc(nnodes, sizeof(hipGraphNode_t));
    HCHECK(hipGraphGetNodes(graph, nodes, &nnodes));
    hipGraphNode_t knode = NULL;
    for (size_t i = 0; i < nnodes; i++) {
        hipGraphNodeType t;
        if (hipGraphNodeGetType(nodes[i], &t) == hipSuccess && t == hipGraphNodeTypeKernel) {
            knode = nodes[i];
            break;
        }
    }
    if (!knode) { fprintf(stderr, "no kernel node captured\n"); return 1; }

    hipGraphExec_t gexec;
    HCHECK(hipGraphInstantiate(&gexec, graph, NULL, NULL, 0));

    double t0 = now_us();
    for (unsigned int t = 1; t <= (unsigned)T; t++) {
        val = t;
        hipKernelNodeParams p;
        memset(&p, 0, sizeof(p));
        p.func = (void *)facc;
        p.gridDim = dim3(1, 1, 1);
        p.blockDim = dim3(1, 1, 1);
        p.sharedMemBytes = 0;
        p.kernelParams = kargs;
        p.extra = NULL;
        HCHECK(hipGraphExecKernelNodeSetParams(gexec, knode, &p));
        HCHECK(hipGraphLaunch(gexec, stream));
        HCHECK(hipStreamSynchronize(stream));
    }
    double hipgraph_us = (now_us() - t0) / T;
    unsigned int hg = 0;
    HCHECK(hipMemcpy(&hg, g_acc, 4, hipMemcpyDeviceToHost));

    /* ---- redline retained-IB arm ---------------------------------------- */
    unsigned int *r_acc = NULL;
    HCHECK(hipMalloc(&r_acc, 4));
    HCHECK(hipMemset(r_acc, 0, 4));

    RlGpu *gpu = rl_gpu_new(0);
    if (!gpu) { fprintf(stderr, "rl_gpu_new failed\n"); return 1; }
    RlModule *mod = NULL;
    if (rl_gpu_load_module(gpu, code, (size_t)len, &mod) != RL_OK) {
        fprintf(stderr, "rl load_module failed\n"); return 1;
    }
    unsigned char karg[512];
    memset(karg, 0, sizeof(karg));
    unsigned long long p = (unsigned long long)(uintptr_t)r_acc;
    memcpy(karg, &p, sizeof(p));
    RlPm4Builder *b = rl_pm4_builder_new(gpu);
    if (rl_pm4_dispatch(b, mod, "acc_k.kd", 1, 1, 1, 1, 1, 1, 0, karg, sizeof(karg)) != RL_OK) {
        fprintf(stderr, "rl dispatch failed\n"); return 1;
    }
    RlPm4Ib *ib = NULL;
    if (rl_pm4_finalize(gpu, b, &ib) != RL_OK) { fprintf(stderr, "rl finalize failed\n"); return 1; }

    t0 = now_us();
    for (unsigned int t = 1; t <= (unsigned)T; t++) {
        rl_pm4_ib_set_kernargs(ib, 0, 8, (const uint8_t *)&t, sizeof(t));
        rl_pm4_replay(ib);
    }
    double redline_us = (now_us() - t0) / T;
    unsigned int rl = 0;
    HCHECK(hipMemcpy(&rl, r_acc, 4, hipMemcpyDeviceToHost));

    int ok = (hg == (unsigned)expected) && (rl == (unsigned)expected);
    printf("decode A/B over %d tokens (acc must == %llu):\n", T, expected);
    printf("  hipGraph (setParams+launch): acc=%u  %8.2f us/token\n", hg, hipgraph_us);
    printf("  redline  (set_kernargs+replay): acc=%u  %8.2f us/token\n", rl, redline_us);
    printf("  redline/hipGraph host speedup: %.2fx   [%s]\n",
           hipgraph_us / redline_us, ok ? "PASS" : "FAIL");

    rl_pm4_ib_free(ib);
    rl_module_free(mod);
    rl_gpu_free(gpu);
    return ok ? 0 : 1;
}
