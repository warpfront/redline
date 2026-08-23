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
//   ./graph_execupdate_oom --update-only [device] [nodes] [update_rounds]
//
// Exit codes: 0 = survived (runtime handled exhaustion cleanly),
//             3 = a HIP API reported an error (also a pass, but flagged),
//             2 = fatal setup failure,
//             SIGSEGV / 139 = reproduced the defect (NULL deref inside runtime).
//
#include <hip/hip_runtime.h>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <vector>

#define OK(x)                                                                      \
    do {                                                                           \
        hipError_t _e = (x);                                                       \
        if (_e != hipSuccess) {                                                    \
            printf("fatal: %s -> %s\n", #x, hipGetErrorString(_e));                \
            return 2;                                                              \
        }                                                                          \
    } while (0)

// Fail-fast helper: print round + call name and return 3 immediately so no
// later crash can be blamed on an earlier ignored async failure.
#define CHECK_OR_ABORT(expr, round, name)                                          \
    do {                                                                           \
        hipError_t _ce = (expr);                                                   \
        if (_ce != hipSuccess) {                                                   \
            printf("round %d: %s failed: %s (%d) -- aborting\n",                  \
                   (round), (name), hipGetErrorString(_ce), (int)_ce);            \
            fflush(stdout);                                                        \
            for (void* _p : hog) hipFree(_p);                                      \
            hipGraphExecDestroy(exec);                                             \
            hipGraphDestroy(graph);                                                \
            hipStreamDestroy(stream);                                              \
            hipFree(buf);                                                          \
            return 3;                                                              \
        }                                                                          \
    } while (0)

__global__ void bump(int* p, int add, int n) {
    int i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i < n) p[i] += add;
}

int main(int argc, char** argv) {
    bool update_only = false;
    size_t leave_mb = 0;  // if >0, release held blocks until this many MiB are free
    // Collect non-flag args as positionals.
    std::vector<const char*> pos;
    for (int i = 1; i < argc; ++i) {
        if (strcmp(argv[i], "--update-only") == 0) {
            update_only = true;
        } else if (strncmp(argv[i], "--leave=", 8) == 0) {
            leave_mb = (size_t)atoll(argv[i] + 8);
        } else {
            pos.push_back(argv[i]);
        }
    }
    const int dev = pos.size() > 0 ? atoi(pos[0]) : 0;
    const int nodes = pos.size() > 1 ? atoi(pos[1]) : 512;
    const int rounds = pos.size() > 2 ? atoi(pos[2]) : 8;
    OK(hipSetDevice(dev));

    hipDeviceProp_t prop;
    OK(hipGetDeviceProperties(&prop, dev));
    size_t free_b = 0, total_b = 0;
    OK(hipMemGetInfo(&free_b, &total_b));
    printf("=== %s (%s) ===\n", prop.name, prop.gcnArchName);
    printf("free %.2f GiB / total %.2f GiB | graph nodes %d | update rounds %d%s\n\n",
           free_b / 1073741824.0, total_b / 1073741824.0, nodes, rounds,
           update_only ? " | --update-only (no launches between updates)" : "");

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
    // Check launch errors before ending capture.
    {
        hipError_t le = hipGetLastError();
        if (le != hipSuccess) {
            printf("initial capture: kernel launch failed before hipStreamEndCapture: %s (%d) -- aborting\n",
                   hipGetErrorString(le), (int)le);
            return 3;
        }
    }
    hipGraph_t graph = nullptr;
    {
        hipError_t e = hipStreamEndCapture(stream, &graph);
        if (e != hipSuccess) {
            printf("initial capture: hipStreamEndCapture failed: %s (%d) -- aborting\n",
                   hipGetErrorString(e), (int)e);
            return 3;
        }
    }

    hipGraphExec_t exec = nullptr;
    OK(hipGraphInstantiate(&exec, graph, nullptr, nullptr, 0));
    OK(hipGraphLaunch(exec, stream));
    OK(hipStreamSynchronize(stream));
    printf("baseline: graph of %d nodes captured, instantiated, launched OK\n", nodes);

    // Exhaust device memory. Descend allocation size so we get as close to the
    // ceiling as the allocator allows.
    std::vector<void*> hog;
    std::vector<size_t> hogsz;
    size_t got = 0;
    for (size_t chunk = 1ull << 30; chunk >= (1ull << 20); chunk >>= 1) {
        for (;;) {
            void* p = nullptr;
            if (hipMalloc(&p, chunk) != hipSuccess) break;
            hog.push_back(p);
            hogsz.push_back(chunk);
            got += chunk;
        }
    }

    // Optionally hand memory back so the kernel launches inside capture can still
    // succeed. Total exhaustion makes the in-capture launch itself return
    // hipErrorOutOfMemory -- that is a legitimate failure and NOT the defect under
    // test. Leaving a margin keeps the kernarg pool under pressure while letting
    // every API call in the update loop succeed on its own terms.
    if (leave_mb > 0) {
        const size_t want_free = leave_mb * 1048576ull;
        for (;;) {
            size_t f = 0, t = 0;
            hipMemGetInfo(&f, &t);
            if (f >= want_free || hog.empty()) break;
            (void)hipFree(hog.back());
            got -= hogsz.back();
            hog.pop_back();
            hogsz.pop_back();
        }
    }

    // CRITICAL: the exhaustion loop above deliberately calls hipMalloc until it
    // FAILS, which leaves hipErrorOutOfMemory as this thread's sticky last-error.
    // Without clearing it here, the first hipGetLastError() in the update loop
    // reports that stale error and looks exactly like an in-capture launch
    // failure -- an artifact of the probe, not a runtime fault.
    (void)hipGetLastError();

    hipMemGetInfo(&free_b, &total_b);
    printf("exhausted: held %.2f GiB in %zu blocks, %.1f MiB reported free%s\n\n",
           got / 1073741824.0, hog.size(), free_b / 1048576.0,
           leave_mb ? " (after --leave release)" : "");

    // Now force the exec to re-capture kernargs with no memory available.
    // Every fallible call below aborts immediately on failure so a later
    // crash cannot be attributed to an earlier ignored error; --update-only
    // additionally ensures no launch poisons the stream before the update.
    for (int r = 1; r <= rounds; ++r) {
        hipGraph_t g2 = nullptr;
        {
            hipError_t e = hipStreamBeginCapture(stream, hipStreamCaptureModeGlobal);
            if (e != hipSuccess) {
                printf("round %d: hipStreamBeginCapture failed: %s (%d) -- aborting\n",
                       r, hipGetErrorString(e), (int)e);
                fflush(stdout);
                for (void* p : hog) hipFree(p);
                hipGraphExecDestroy(exec);
                hipGraphDestroy(graph);
                hipStreamDestroy(stream);
                hipFree(buf);
                return 3;
            }
        }
        for (int i = 0; i < nodes; ++i)
            bump<<<(N + 255) / 256, 256, 0, stream>>>(buf, r + 2, N);
        {
            hipError_t le = hipGetLastError();
            if (le != hipSuccess) {
                printf("round %d: kernel launch hipGetLastError failed: %s (%d) -- aborting\n",
                       r, hipGetErrorString(le), (int)le);
                fflush(stdout);
                hipStreamEndCapture(stream, &g2);
                if (g2) hipGraphDestroy(g2);
                for (void* p : hog) hipFree(p);
                hipGraphExecDestroy(exec);
                hipGraphDestroy(graph);
                hipStreamDestroy(stream);
                hipFree(buf);
                return 3;
            }
        }
        {
            hipError_t e = hipStreamEndCapture(stream, &g2);
            if (e != hipSuccess) {
                printf("round %d: hipStreamEndCapture failed: %s (%d) -- aborting\n",
                       r, hipGetErrorString(e), (int)e);
                fflush(stdout);
                if (g2) hipGraphDestroy(g2);
                for (void* p : hog) hipFree(p);
                hipGraphExecDestroy(exec);
                hipGraphDestroy(graph);
                hipStreamDestroy(stream);
                hipFree(buf);
                return 3;
            }
        }

        printf("round %d: hipGraphExecUpdate under exhaustion ... ", r);
        fflush(stdout);
        hipGraphExecUpdateResult ures{};
        hipGraphNode_t bad = nullptr;
        hipError_t ue = hipGraphExecUpdate(exec, g2, &bad, &ures);
        // Always print the update result enum and bad-node pointer for the
        // published excerpt, whether the update succeeded or failed.
        printf("%s (result=%d, bad_node=%p)\n",
               ue == hipSuccess ? "ok" : hipGetErrorString(ue), (int)ures, (void*)bad);
        fflush(stdout);

        if (ue != hipSuccess) {
            printf("round %d: hipGraphExecUpdate failed: %s (result=%d, bad_node=%p) -- aborting\n",
                   r, hipGetErrorString(ue), (int)ures, (void*)bad);
            fflush(stdout);
            hipGraphDestroy(g2);
            for (void* p : hog) hipFree(p);
            hipGraphExecDestroy(exec);
            hipGraphDestroy(graph);
            hipStreamDestroy(stream);
            hipFree(buf);
            return 3;
        }

        // On success, optionally launch+sync (skipped under --update-only).
        if (!update_only) {
            hipError_t le = hipGraphLaunch(exec, stream);
            if (le != hipSuccess) {
                printf("round %d: post-update hipGraphLaunch failed: %s (%d) -- aborting\n",
                       r, hipGetErrorString(le), (int)le);
                fflush(stdout);
                hipGraphDestroy(g2);
                for (void* p : hog) hipFree(p);
                hipGraphExecDestroy(exec);
                hipGraphDestroy(graph);
                hipStreamDestroy(stream);
                hipFree(buf);
                return 3;
            }
            hipError_t se = hipStreamSynchronize(stream);
            if (se != hipSuccess) {
                printf("round %d: post-update hipStreamSynchronize failed: %s (%d) -- aborting\n",
                       r, hipGetErrorString(se), (int)se);
                fflush(stdout);
                hipGraphDestroy(g2);
                for (void* p : hog) hipFree(p);
                hipGraphExecDestroy(exec);
                hipGraphDestroy(graph);
                hipStreamDestroy(stream);
                hipFree(buf);
                return 3;
            }
            printf("         launch %s / sync %s (result=%d, bad_node=%p)\n",
                   hipGetErrorString(le), hipGetErrorString(se), (int)ures, (void*)bad);
        }

        {
            hipError_t de = hipGraphDestroy(g2);
            if (de != hipSuccess) {
                printf("round %d: hipGraphDestroy(g2) failed: %s (%d) -- aborting\n",
                       r, hipGetErrorString(de), (int)de);
                fflush(stdout);
                for (void* p : hog) hipFree(p);
                hipGraphExecDestroy(exec);
                hipGraphDestroy(graph);
                hipStreamDestroy(stream);
                hipFree(buf);
                return 3;
            }
        }
    }

    printf("\n=== survived: the runtime did not dereference NULL under exhaustion ===\n");
    printf("    (an affected build dies with SIGSEGV inside libamdhip64 above)\n");

    for (void* p : hog) hipFree(p);
    hipGraphExecDestroy(exec);
    hipGraphDestroy(graph);
    hipStreamDestroy(stream);
    hipFree(buf);
    return 0;
}
