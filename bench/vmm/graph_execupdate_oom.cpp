// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
//
// Reproducer for ROCm/rocm-systems#10021: HIP runtime SIGSEGV during
// hipGraphExecUpdate kernarg capture when device memory is exhausted.
//
// Upstream analysis (fix PR #10022) locates it in CLR: the graph-capture branch
// of submitKernelInternal calls getGraphKernArg() and then hands the result to
// amd::nontemporalMemcpy without a NULL check, while
// GraphKernelArgManager::AllocKernArg can legitimately return nullptr when the
// kernarg pool cannot grow. The report is filed against gfx1100; the code path
// carries no architecture predicate, so this program tests whether it is in fact
// architecture-independent.
//
// Shape: capture a graph of many kernel nodes, instantiate it, exhaust device
// memory, then drive hipGraphExecUpdate so the exec has to re-capture kernargs
// with no memory available. A clean runtime returns an error. An affected
// runtime dereferences NULL inside the HIP runtime and dies.
//
// Build:
//   hipcc --offload-arch=gfx1201 -O2 graph_execupdate_oom.cpp -o graph_execupdate_oom
// Run (one GPU; it deliberately fills that GPU's VRAM, then frees it):
//   ./graph_execupdate_oom [device] [nodes] [update_rounds]
//
// Exit codes: 0 = survived (runtime handled exhaustion), 3 = graph API reported
// an error (also a pass, reported), 139/SIGSEGV = reproduced the defect.
#include <hip/hip_runtime.h>
#include <cstdio>
#include <cstdlib>
#include <vector>

#define OK(x)                                                                      \
    do {                                                                           \
        hipError_t _e = (x);                                                       \
        if (_e != hipSuccess) {                                                    \
            printf("fatal: %s -> %s\n", #x, hipGetErrorString(_e));                \
            return 2;                                                              \
        }                                                                          \
    } while (0)

__global__ void bump(int* p, int add, int n) {
    int i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i < n) p[i] += add;
}

int main(int argc, char** argv) {
    const int dev = argc > 1 ? atoi(argv[1]) : 0;
    const int nodes = argc > 2 ? atoi(argv[2]) : 512;
    const int rounds = argc > 3 ? atoi(argv[3]) : 8;
    OK(hipSetDevice(dev));

    hipDeviceProp_t prop;
    OK(hipGetDeviceProperties(&prop, dev));
    size_t free_b = 0, total_b = 0;
    OK(hipMemGetInfo(&free_b, &total_b));
    printf("=== %s (%s) ===\n", prop.name, prop.gcnArchName);
    printf("free %.2f GiB / total %.2f GiB | graph nodes %d | update rounds %d\n\n",
           free_b / 1073741824.0, total_b / 1073741824.0, nodes, rounds);

    const int N = 1024;
    int* buf = nullptr;
    OK(hipMalloc(&buf, N * sizeof(int)));
    OK(hipMemset(buf, 0, N * sizeof(int)));

    hipStream_t stream;
    OK(hipStreamCreate(&stream));

    // Capture a graph with many kernel nodes so the exec owns a sizeable kernarg pool.
    OK(hipStreamBeginCapture(stream, hipStreamCaptureModeGlobal));
    for (int i = 0; i < nodes; ++i)
        bump<<<(N + 255) / 256, 256, 0, stream>>>(buf, 1, N);
    hipGraph_t graph = nullptr;
    OK(hipStreamEndCapture(stream, &graph));

    hipGraphExec_t exec = nullptr;
    OK(hipGraphInstantiate(&exec, graph, nullptr, nullptr, 0));
    OK(hipGraphLaunch(exec, stream));
    OK(hipStreamSynchronize(stream));
    printf("baseline: graph of %d nodes captured, instantiated, launched OK\n", nodes);

    // Exhaust device memory. Descend allocation size so we get as close to the
    // ceiling as the allocator allows.
    std::vector<void*> hog;
    size_t got = 0;
    for (size_t chunk = 1ull << 30; chunk >= (1ull << 20); chunk >>= 1) {
        for (;;) {
            void* p = nullptr;
            if (hipMalloc(&p, chunk) != hipSuccess) break;
            hog.push_back(p);
            got += chunk;
        }
    }
    hipMemGetInfo(&free_b, &total_b);
    printf("exhausted: held %.2f GiB in %zu blocks, %.1f MiB reported free\n\n",
           got / 1073741824.0, hog.size(), free_b / 1048576.0);

    // Now force the exec to re-capture kernargs with no memory available.
    int rc = 0;
    for (int r = 1; r <= rounds; ++r) {
        hipGraph_t g2 = nullptr;
        OK(hipStreamBeginCapture(stream, hipStreamCaptureModeGlobal));
        for (int i = 0; i < nodes; ++i)
            bump<<<(N + 255) / 256, 256, 0, stream>>>(buf, r + 2, N);
        OK(hipStreamEndCapture(stream, &g2));

        printf("round %d: hipGraphExecUpdate under exhaustion ... ", r);
        fflush(stdout);
        hipGraphExecUpdateResult ures{};
        hipGraphNode_t bad = nullptr;
        hipError_t ue = hipGraphExecUpdate(exec, g2, &bad, &ures);
        printf("%s (result=%d)\n", ue == hipSuccess ? "ok" : hipGetErrorString(ue), (int)ures);

        if (ue == hipSuccess) {
            hipError_t le = hipGraphLaunch(exec, stream);
            hipError_t se = hipStreamSynchronize(stream);
            printf("         launch %s / sync %s\n", hipGetErrorString(le), hipGetErrorString(se));
            if (le != hipSuccess || se != hipSuccess) rc = 3;
        } else {
            rc = 3;
        }
        hipGraphDestroy(g2);
    }

    printf("\n=== survived: the runtime did not dereference NULL under exhaustion ===\n");
    printf("    (an affected build dies with SIGSEGV inside libamdhip64 above)\n");

    for (void* p : hog) hipFree(p);
    hipGraphExecDestroy(exec);
    hipGraphDestroy(graph);
    hipStreamDestroy(stream);
    hipFree(buf);
    return rc;
}
