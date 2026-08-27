// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
//
// Does hipGraph fence according to the dependency structure it was given?
//
// Why this matters:
//   A hipGraph is an explicit DAG. The runtime therefore knows, before it
//   submits anything, which dispatches are ordered and which are independent.
//   If per-dispatch cost is the same whether the graph is a strict serial chain
//   or N mutually independent nodes, then that structural information is not
//   being used to relax fencing -- and the cost of the strongest case is being
//   paid by every case.
//
//   This is the difference between "HIP needs a new low-level submission API"
//   and "HIP already has the information required to fence less". The second is
//   a far smaller ask, so it is worth establishing which one is true.
//
// Arms, all with identical kernels and identical node counts:
//   chain        node i depends on node i-1. Every dispatch is ordered.
//   independent  every node depends only on the graph root. Maximum freedom.
//   fanout-join  independent middle, joined at the end. What a real layer does.
//
// Nodes are added explicitly with hipGraphAddKernelNode rather than by stream
// capture, so the dependency edges are exactly what this file states and not an
// artifact of capture order.
//
// The kernel is a one-workitem atomicAdd, so per-dispatch cost is dominated by
// submission and fencing rather than compute. Correctness is gated: the counter
// must equal exactly N * replays in every arm, which also confirms the
// independent arm really did run every node.
//
// Timing is host steady_clock and hipEvent over the same batch, as in
// aql_dispatch_floor.cpp, with no profiler involved.
//
// Build: hipcc --offload-arch=gfx1201 -O3 graph_dependency_fencing.cpp -o gdf
// Run:   ./gdf [N] [replays] [warmups]

#include <hip/hip_runtime.h>
#include <dlfcn.h>
#include <algorithm>
#include <chrono>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <vector>

// Every number this probe prints is only meaningful if we know which runtime
// produced it. A side-by-side ROCm install makes it easy to believe an arm ran
// against one version when it actually loaded another: hipcc emits no RPATH and
// /opt/rocm/core is a symlink to one specific version, so an unset or wrong
// LD_LIBRARY_PATH silently redirects the whole measurement. That failure is
// invisible in the results except that the two arms agree suspiciously well.
//
// So the probe reports its own provenance, from inside the process, before it
// reports any timing: the HIP runtime version it actually loaded, and the
// on-disk object that supplied a HIP symbol as resolved by dladdr. Neither can
// be faked by the environment that launched it.
static void print_runtime_provenance() {
    int rt = -1;
    hipRuntimeGetVersion(&rt);
    Dl_info info{};
    const char* who = "(unresolved)";
    if (dladdr((void*)&hipRuntimeGetVersion, &info) && info.dli_fname)
        who = info.dli_fname;
    printf("runtime: hipRuntimeGetVersion=%d  libamdhip64=%s\n", rt, who);
}

#define OK(x)                                                                    \
    do {                                                                         \
        hipError_t _e = (x);                                                     \
        if (_e != hipSuccess) {                                                  \
            printf("FATAL %s -> %s (%d) at line %d\n", #x, hipGetErrorString(_e), \
                   (int)_e, __LINE__);                                           \
            exit(1);                                                             \
        }                                                                        \
    } while (0)

__global__ void tick(unsigned long long* counter) {
    if (threadIdx.x == 0 && blockIdx.x == 0) atomicAdd(counter, 1ull);
}

static double median(std::vector<double>& v) {
    if (v.empty()) return 0.0;
    std::sort(v.begin(), v.end());
    const size_t m = v.size() / 2;
    return (v.size() & 1) ? v[m] : 0.5 * (v[m - 1] + v[m]);
}

// ParallelChains exists because the other three shapes may not exercise graph
// segmentation at all. ROCm 10.0's graph executor derives segments from
// *execution paths* and schedules them across a bounded number of streams, so a
// graph of N dependency-free singletons plausibly collapses to a single segment
// (everything at one dependency level) and a strict chain is trivially one path.
// Neither would produce multi-stream output even if the machinery works.
//
// ParallelChains builds `chains` independent serial chains of equal length, so
// the graph contains exactly `chains` distinct execution paths. That is the
// shape a real decode step resembles, and it is the one that should segment.
enum class Shape { Chain, Independent, FanoutJoin, ParallelChains };

// Number of independent chains for Shape::ParallelChains; set from --chains=.
static int g_chains = 4;

// Builds a graph of `n` kernel nodes with the requested dependency structure.
static hipGraph_t build(int n, Shape shape, unsigned long long* d_counter) {
    hipGraph_t g = nullptr;
    OK(hipGraphCreate(&g, 0));

    void* args[1] = {(void*)&d_counter};
    hipKernelNodeParams np{};
    np.func = (void*)tick;
    np.gridDim = dim3(1);
    np.blockDim = dim3(1);
    np.sharedMemBytes = 0;
    np.kernelParams = args;
    np.extra = nullptr;

    std::vector<hipGraphNode_t> nodes;
    nodes.reserve(n);

    for (int i = 0; i < n; ++i) {
        hipGraphNode_t node = nullptr;
        switch (shape) {
            case Shape::Chain: {
                // Strict serial order: depend on the previous node.
                hipGraphNode_t* dep = (i == 0) ? nullptr : &nodes[i - 1];
                size_t ndep = (i == 0) ? 0 : 1;
                OK(hipGraphAddKernelNode(&node, g, dep, ndep, &np));
                break;
            }
            case Shape::Independent: {
                // No edges at all: every node is a root, all are concurrent.
                OK(hipGraphAddKernelNode(&node, g, nullptr, 0, &np));
                break;
            }
            case Shape::FanoutJoin: {
                // All but the last are independent; the last joins them.
                if (i + 1 < n) {
                    OK(hipGraphAddKernelNode(&node, g, nullptr, 0, &np));
                } else {
                    OK(hipGraphAddKernelNode(&node, g, nodes.data(),
                                             (size_t)nodes.size(), &np));
                }
                break;
            }
            case Shape::ParallelChains: {
                // `chains` independent chains, laid out round-robin so chain c
                // owns nodes c, c+chains, c+2*chains, ... Each node depends on
                // the previous node of its own chain and on nothing else, so the
                // graph has exactly `chains` distinct execution paths.
                const int c = i % g_chains;
                const int prev = i - g_chains;
                if (prev < 0) {
                    OK(hipGraphAddKernelNode(&node, g, nullptr, 0, &np));
                } else {
                    hipGraphNode_t dep = nodes[prev];
                    OK(hipGraphAddKernelNode(&node, g, &dep, 1, &np));
                }
                (void)c;
                break;
            }
        }
        nodes.push_back(node);
    }
    return g;
}

struct Result {
    double host_us = 0.0;
    double event_us = 0.0;
    bool correct = false;
    unsigned long long counted = 0, expected = 0;
};

static Result run_shape(int n, int replays, int warmups, Shape shape,
                        unsigned long long* d_counter) {
    hipGraph_t g = build(n, shape, d_counter);
    hipGraphExec_t exec = nullptr;
    OK(hipGraphInstantiate(&exec, g, nullptr, nullptr, 0));

    hipStream_t s;
    OK(hipStreamCreate(&s));
    hipEvent_t e0, e1;
    OK(hipEventCreate(&e0));
    OK(hipEventCreate(&e1));

    for (int i = 0; i < warmups; ++i) {
        OK(hipGraphLaunch(exec, s));
        OK(hipStreamSynchronize(s));
    }

    OK(hipMemset(d_counter, 0, sizeof(unsigned long long)));
    OK(hipDeviceSynchronize());

    std::vector<double> hv, ev;
    hv.reserve(replays);
    ev.reserve(replays);
    for (int i = 0; i < replays; ++i) {
        OK(hipEventRecord(e0, s));
        const auto t0 = std::chrono::steady_clock::now();
        OK(hipGraphLaunch(exec, s));
        OK(hipStreamSynchronize(s));
        const auto t1 = std::chrono::steady_clock::now();
        OK(hipEventRecord(e1, s));
        OK(hipEventSynchronize(e1));
        float ms = 0.0f;
        OK(hipEventElapsedTime(&ms, e0, e1));
        hv.push_back(std::chrono::duration<double, std::micro>(t1 - t0).count() / n);
        ev.push_back((double)ms * 1000.0 / n);
    }

    Result r;
    OK(hipMemcpy(&r.counted, d_counter, sizeof(r.counted), hipMemcpyDeviceToHost));
    r.expected = (unsigned long long)n * (unsigned long long)replays;
    r.correct = (r.counted == r.expected);
    r.host_us = median(hv);
    r.event_us = median(ev);

    OK(hipEventDestroy(e0));
    OK(hipEventDestroy(e1));
    OK(hipStreamDestroy(s));
    OK(hipGraphExecDestroy(exec));
    OK(hipGraphDestroy(g));
    return r;
}

int main(int argc, char** argv) {
    // Positional: N, replays, warmups. Optional --only=<shape> restricts the run
    // to a single graph shape.
    //
    // Why --only exists: a profiler trace from a run containing all three shapes
    // has to be split back into phases afterwards, and on runtimes whose
    // dispatch records carry no graph identity that split can only be done by
    // assuming fixed-size blocks in time order. Running one shape per process
    // makes the whole trace unambiguously that shape, so the scheduling evidence
    // needs no assumption at all.
    const char* only = nullptr;
    std::vector<const char*> pos;
    for (int i = 1; i < argc; ++i) {
        if (strncmp(argv[i], "--only=", 7) == 0) only = argv[i] + 7;
        else if (strncmp(argv[i], "--chains=", 9) == 0) g_chains = atoi(argv[i] + 9);
        else pos.push_back(argv[i]);
    }
    const int n = pos.size() > 0 ? atoi(pos[0]) : 512;
    const int replays = pos.size() > 1 ? atoi(pos[1]) : 200;
    const int warmups = pos.size() > 2 ? atoi(pos[2]) : 20;
    if (g_chains < 1) g_chains = 1;
    const bool do_chain = !only || strcmp(only, "chain") == 0;
    const bool do_indep = !only || strcmp(only, "independent") == 0;
    const bool do_fanout = !only || strcmp(only, "fanout") == 0;
    // parallel-chains is opt-in only: it is a different question from the other
    // three (does segmentation happen at all) and mixing it into the default run
    // would change the shape count that profiler traces are split by.
    const bool do_pchains = only && strcmp(only, "parallel-chains") == 0;
    if (only && !do_chain && !do_indep && !do_fanout && !do_pchains) {
        printf("unknown --only=%s (expected chain|independent|fanout|parallel-chains)\n",
               only);
        return 1;
    }

    OK(hipSetDevice(0));
    hipDeviceProp_t p;
    OK(hipGetDeviceProperties(&p, 0));
    printf("=== %s (%s) ===\n", p.name, p.gcnArchName);
    print_runtime_provenance();
    printf("N=%d kernel nodes, %d replays (median), %d warmups\n", n, replays, warmups);
    printf("Same kernel and node count in every arm; only the DAG edges differ.\n\n");
    printf("  %-14s %9s %9s   %-6s %s\n", "graph shape", "host us", "event us",
           "gate", "");

    unsigned long long* d = nullptr;
    OK(hipMalloc(&d, sizeof(unsigned long long)));

    Result chain{}, indep{}, fj{}, pc{};
    if (do_chain) {
        chain = run_shape(n, replays, warmups, Shape::Chain, d);
        printf("  %-14s %9.3f %9.3f   %s\n", "chain", chain.host_us, chain.event_us,
               chain.correct ? "ok" : "MISMATCH");
    }
    if (do_indep) {
        indep = run_shape(n, replays, warmups, Shape::Independent, d);
        printf("  %-14s %9.3f %9.3f   %s\n", "independent", indep.host_us,
               indep.event_us, indep.correct ? "ok" : "MISMATCH");
    }
    if (do_fanout) {
        fj = run_shape(n, replays, warmups, Shape::FanoutJoin, d);
        printf("  %-14s %9.3f %9.3f   %s\n", "fanout-join", fj.host_us, fj.event_us,
               fj.correct ? "ok" : "MISMATCH");
    }
    if (do_pchains) {
        pc = run_shape(n, replays, warmups, Shape::ParallelChains, d);
        char label[32];
        snprintf(label, sizeof(label), "chains=%d", g_chains);
        printf("  %-14s %9.3f %9.3f   %s\n", label, pc.host_us, pc.event_us,
               pc.correct ? "ok" : "MISMATCH");
    }

    OK(hipFree(d));

    // With --only there is nothing to compare, so skip the interpretation
    // entirely rather than print a ratio against an unpopulated arm.
    if (only) {
        const Result& r = do_chain ? chain
                                   : (do_indep ? indep : (do_fanout ? fj : pc));
        printf("\nsingle-shape run (--only=%s); gate %s\n", only,
               r.correct ? "passed" : "FAILED");
        return r.correct ? 0 : 2;
    }

    printf("\n--- reading ---\n");
    if (!chain.correct || !indep.correct || !fj.correct) {
        printf("A correctness gate failed; the timings above are not valid.\n");
        return 2;
    }
    const double ratio = indep.host_us > 0.0 ? chain.host_us / indep.host_us : 0.0;
    printf("chain / independent = %.3fx\n", ratio);
    if (ratio > 1.10) {
        printf("The independent graph is measurably cheaper per dispatch, so the\n"
               "runtime does exploit declared independence on this build.\n");
    } else if (ratio >= 0.90) {
        printf("Declaring every node independent neither helps nor hurts, so on\n"
               "this build the declared dependency structure does not measurably\n"
               "change per-dispatch cost either way.\n");
    } else {
        printf("Declaring nodes independent is %.2fx MORE expensive per dispatch\n"
               "than a strict serial chain. The cheapest shape here is the fully\n"
               "ordered one, so on this build the penalty is attached to the\n"
               "concurrent shape itself rather than to ordering. A plausible\n"
               "reading is extra submission or cross-queue join machinery for\n"
               "concurrent nodes, but this probe does not inspect packets and\n"
               "does not establish the mechanism.\n",
               ratio > 0.0 ? 1.0 / ratio : 0.0);
    }
    printf("\nThis measures cost as a function of declared dependency structure\n"
           "only. It does not inspect emitted packets and does not identify\n"
           "which fence or barrier bit is responsible.\n");
    return 0;
}
