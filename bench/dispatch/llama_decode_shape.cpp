// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
//
// llama.cpp decode-shape probe: stream-capture graph replay for a single-token
// residual chain, with shape as a swept axis.
//
// Why this shape:
//   llama.cpp builds its decode graph by stream capture (hipStreamBeginCapture /
//   hipStreamEndCapture), NOT by hipGraphAddKernelNode.  That exercises the
//   interposer path that records dispatches from the stream-capture stream,
//   which is distinct from the explicit-node path.  See
//     ggml/src/ggml-cuda/common.cuh        -- USE_CUDA_GRAPH guarded by GGML_CUDA_GRAPHS
//     ggml/src/ggml-cuda/vendors/hip.h     -- 138 CUDA->HIP #defines, HIP is a translation header
//     ggml/src/ggml-cuda/ggml-cuda.cu      -- evaluate_and_capture_cuda_graph()
//     ggml/src/ggml-hip/CMakeLists.txt     -- GGML_HIP_GRAPHS -> USE_CUDA_GRAPH
//   The HIP backend is literally the CUDA backend behind ggml/src/ggml-cuda/vendors/hip.h:
//   WARP_SIZE is #define 32 unconditional at common.cuh:46, wave64 is special-cased only for
//   __GFX9__/__GFX8__.  The dispatch shape is therefore CUDA-idiomatic, not AMD-idiomatic:
//   one stream, one serial residual chain, GGML_CUDA_MAX_STREAMS 8 but decode using 1.  See
//   src/llama-graph.cpp: build_attn() then build_ffn() then residual adds, repeated per layer,
//   with no edges between layers except the residual.  Optional GGML_SCHED_GRAPH_OPTIMIZE
//   forks only the three Q/K/V projections onto at most 3 streams and rejoins; everything else
//   stays a chain.  Graph use is disabled for batch>1, for mul_mat_id, and on frequent-update
//   thrash, so graphs apply to single-token decode (tg), not prompt processing (pp).
//   See src/llama.cpp graph guards and tools/llama-bench/llama-bench.cpp for tgN/ppN reporting.
//
//   Because the default graph is a single chain, DAG segmentation reports
//   Unsplittable and the multi-queue win measured on synthetic parallel graphs
//   (2.24x-6.90x) is unavailable.  Only per-dispatch submission saving can
//   help.  A saving of 1-2 us per node is a shrinking fraction of a node that
//   does tens of microseconds of work, so the probe must sweep the ratio of
//   submission cost to kernel duration explicitly.  A result that shows no
//   benefit on this shape is as publishable as one that does.
//
//   The more valuable measurement is the GAP between what CUDA-shaped code gets
//   on AMD and what the same work could get if it expressed the parallelism AMD
//   hardware can actually use.  So shape is a first-class swept axis, not a
//   fixed default:
//
//     --shape=cuda-serial  (default): one stream, fully serial residual chain.
//                          This is llama.cpp today.  Single weakly-connected
//                          component, Unsplittable.
//
//     --shape=qkv-fork    : GGML_SCHED_GRAPH_OPTIMIZE variant, Q/K/V on <=3
//                          streams then rejoin, rest serial.  At most 3-way
//                          fork per layer, still essentially a chain.
//
//     --shape=amd-wide    : same total kernels and same dependency semantics
//                          *within* each layer, but layers' work expressed as
//                          genuinely independent paths up to --paths=N.  This is
//                          the hypothetical AMD-shaped engine.  It intentionally
//                          breaks the residual chain (layers become independent)
//                          and is therefore NOT a valid llama.cpp graph -- the
//                          probe says so and makes it obvious which shape
//                          actually contained independent components.  If the
//                          residual chain forbids a split, that is a legitimate
//                          finding, not a reason to approximate.
//
//   The headline result is thus a 2-D table: shape x --work, stock vs
//   interposer.  That answers three questions: does the interposer help
//   llama.cpp as it exists; does llama.cpp's own optional fork help; and how
//   much is left on the table by the CUDA-shaped graph.
//
// Default sizing -- arithmetic:
//   32 layers * 12 kernels/layer = 384 nodes, plus 6 prologue/epilogue nodes
//   (input norm, final norm, head) = 390 nodes per token graph.  This lands in
//   the 300-500 node range observed for 7B-32B class models at batch=1 (each
//   layer: Q/K/V proj, RoPE, flash-attn, out-proj, FFN up/gate/act/down,
//   2 norms, 2 residual adds; ~10-15 kernels/layer; 32 layers ~320-480 plus a
//   handful of non-layer nodes).  80-layer 70B range scales to ~966 with the
//   same per-layer count.  Override with --layers=N.  --shape=amd-wide keeps
//   the same total node count, distributing layers round-robin across --paths
//   streams, so each path is still a serial chain but paths are independent
//   (hypothetical).
//
// What this measures and what its labels mean:
//   Every printed timing names only the condition the code actually measured:
//   "synthetic tokens/s" is the rate at which this probe can replay its
//   captured graph, not a language model's throughput; "host us/token" and
//   "event us/token" are the two clocks over the same replays; per-dispatch
//   is token time divided by the fixed node count; expected compute is the
//   requested --work converted through wall_clock64's tick rate.  No label
//   implies a real model, a profiler, or a hardware-queue trace.  For
//   --shape=amd-wide the output explicitly states that the graph breaks the
//   residual dependency and how many independent components it contained.
//
// Build (on a GPU host with hipcc):
//   hipcc -O2 --offload-arch=gfx1201 bench/dispatch/llama_decode_shape.cpp -o /tmp/llama_decode_shape -ldl
//   hipcc -O2 --offload-arch=gfx1100 bench/dispatch/llama_decode_shape.cpp -o /tmp/llama_decode_shape -ldl
//   hipcc -O2 --offload-arch=gfx1151 bench/dispatch/llama_decode_shape.cpp -o /tmp/llama_decode_shape -ldl
//   (ROCm 7.14:  /opt/rocm/core-7.14/bin/hipcc -O2 --offload-arch=gfxNNNN ... -ldl)
//   (ROCm 10.0:  /opt/rocm/core-10.0/bin/hipcc -O2 --offload-arch=gfxNNNN ... -ldl  with LD_LIBRARY_PATH=/opt/rocm/core-10.0/lib)
//   The binary carries no RPATH; set LD_LIBRARY_PATH to the ROCm you built against when running.
//
// Run matrix -- stock versus interposer, shape x work (2-D table).  The parent
// runs these from git worktree checkouts; do NOT rsync/scp sources to hipx/hiptrx.
//
//   # 2-D sweep: shape {cuda-serial,qkv-fork,amd-wide} x work {0,500,5000,20000}
//   # stock (no interposer)
//   LD_LIBRARY_PATH=/opt/rocm/core-10.0/lib /tmp/llama_decode_shape --shape=cuda-serial --layers=32 --tokens=200 --work=0
//   LD_LIBRARY_PATH=/opt/rocm/core-10.0/lib /tmp/llama_decode_shape --shape=cuda-serial --layers=32 --tokens=200 --work=5000
//   LD_LIBRARY_PATH=/opt/rocm/core-10.0/lib /tmp/llama_decode_shape --shape=qkv-fork    --layers=32 --tokens=200 --work=0
//   LD_LIBRARY_PATH=/opt/rocm/core-10.0/lib /tmp/llama_decode_shape --shape=qkv-fork    --layers=32 --tokens=200 --work=5000
//   LD_LIBRARY_PATH=/opt/rocm/core-10.0/lib /tmp/llama_decode_shape --shape=amd-wide --paths=4 --layers=32 --tokens=200 --work=0
//   LD_LIBRARY_PATH=/opt/rocm/core-10.0/lib /tmp/llama_decode_shape --shape=amd-wide --paths=4 --layers=32 --tokens=200 --work=5000
//   LD_LIBRARY_PATH=/opt/rocm/core-10.0/lib /tmp/llama_decode_shape --shape=amd-wide --paths=8 --layers=32 --tokens=200 --work=5000
//   # with per-token hipGraphExecUpdate exercised (llama.cpp updates kernel params per token)
//   LD_LIBRARY_PATH=/opt/rocm/core-10.0/lib /tmp/llama_decode_shape --shape=cuda-serial --layers=32 --tokens=200 --work=5000 --update
//   LD_LIBRARY_PATH=/opt/rocm/core-10.0/lib /tmp/llama_decode_shape --shape=qkv-fork    --layers=32 --tokens=200 --work=5000 --update
//
//   # interposer, lanes disabled (control: graph as single lane)
//   LD_PRELOAD=/path/to/libredline_hipgraph.so REDLINE_HIPGRAPH_LANES=off \
//     LD_LIBRARY_PATH=/opt/rocm/core-10.0/lib /tmp/llama_decode_shape --shape=cuda-serial --layers=32 --tokens=200 --work=0
//   LD_PRELOAD=/path/to/libredline_hipgraph.so REDLINE_HIPGRAPH_LANES=off \
//     LD_LIBRARY_PATH=/opt/rocm/core-10.0/lib /tmp/llama_decode_shape --shape=amd-wide --paths=4 --layers=32 --tokens=200 --work=5000
//   # interposer, lanes auto (segmentation + multi-queue if any)
//   LD_PRELOAD=/path/to/libredline_hipgraph.so REDLINE_HIPGRAPH_LANES=auto \
//     LD_LIBRARY_PATH=/opt/rocm/core-10.0/lib /tmp/llama_decode_shape --shape=cuda-serial --layers=32 --tokens=200 --work=0
//   LD_PRELOAD=/path/to/libredline_hipgraph.so REDLINE_HIPGRAPH_LANES=auto \
//     LD_LIBRARY_PATH=/opt/rocm/core-10.0/lib /tmp/llama_decode_shape --shape=qkv-fork    --layers=32 --tokens=200 --work=5000
//   LD_PRELOAD=/path/to/libredline_hipgraph.so REDLINE_HIPGRAPH_LANES=auto \
//     LD_LIBRARY_PATH=/opt/rocm/core-10.0/lib /tmp/llama_decode_shape --shape=amd-wide --paths=4 --layers=32 --tokens=200 --work=0
//   LD_PRELOAD=/path/to/libredline_hipgraph.so REDLINE_HIPGRAPH_LANES=auto \
//     LD_LIBRARY_PATH=/opt/rocm/core-10.0/lib /tmp/llama_decode_shape --shape=amd-wide --paths=4 --layers=32 --tokens=200 --work=5000
//   # repeat the same shape x work lines on gfx1100 and gfx1151 binaries.
//
// How to read the result (both outcomes are reportable):
//   - Interposer helps llama.cpp-shaped work if, at a fixed --shape, --work
//     and fixed layer/node count, host us/token (and event us/token when the
//     clocks agree) is lower under LD_PRELOAD with REDLINE_HIPGRAPH_LANES=auto
//     than stock, with gate ok, and the improvement exceeds run-to-run noise
//     over >=3 fresh-process runs and survives graph capture (this probe is
//     already a captured graph, so that is inherent).  The per-dispatch
//     saving * nodes = token saving must be coherent.  The win should be
//     largest at small --work (submission-dominated) and shrink as --work
//     grows to tens of microseconds per kernel; the printed per-kernel
//     duration and submission-cost share make that shrinkage explicit.
//   - Interposer does not help this shape if host and event us/token agree
//     between stock and interposer within noise at every --work point, or if
//     the only delta appears at near-empty kernels and vanishes by the time
//     per-kernel duration reaches the 10-30 us range that real decode kernels
//     occupy (mmvq / flash-attn).  A tight stddev or an unusually high
//     synthetic tokens/s at near-zero work is a warning sign of a single-token
//     attractor or empty-graph timing, not a win; always eyeball the counter
//     gate and verify secondary streams did not collapse.  This probe is
//     explicitly designed to be able to show no benefit -- a successful run
//     that reports equivalence is a complete result.
//   - The shape axis adds: compare cuda-serial vs qkv-fork vs amd-wide at the
//     same --work to see how much the CUDA-shaped graph leaves on the table.
//     cuda-serial is Unsplittable (one component); qkv-fork has a tiny 3-way
//     fork per layer; amd-wide has up to --paths independent components but
//     breaks the residual chain (probe says so).  If the residual chain
//     forbids a split, the probe reports that instead of faking one.

#include <hip/hip_runtime.h>
#include <dlfcn.h>

#include <algorithm>
#include <chrono>
#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <string>
#include <vector>

#define OK(x)                                                                    \
    do {                                                                         \
        hipError_t _e = (x);                                                     \
        if (_e != hipSuccess) {                                                  \
            printf("FATAL %s -> %s (%d) at line %d\n", #x, hipGetErrorString(_e), \
                   (int)_e, __LINE__);                                           \
            exit(1);                                                             \
        }                                                                        \
    } while (0)

// ---------------------------------------------------------------------------
// Runtime provenance (rocm_ident.cpp pattern): hipRuntimeGetVersion plus
// dladdr on the HIP symbol so the result cannot be ambiguous about which
// libamdhip64 was loaded, regardless of LD_LIBRARY_PATH / RPATH.
// ---------------------------------------------------------------------------
static void print_runtime_provenance() {
    int rt = -1;
    (void)hipRuntimeGetVersion(&rt);
    Dl_info info{};
    const char* who = "(unresolved)";
    if (dladdr((void*)&hipRuntimeGetVersion, &info) && info.dli_fname)
        who = info.dli_fname;
    printf("runtime: hipRuntimeGetVersion=%d  libamdhip64=%s\n", rt, who);
}

static const char* hipGraphExecUpdateResultString(hipGraphExecUpdateResult r) {
    switch (r) {
        case hipGraphExecUpdateSuccess: return "hipGraphExecUpdateSuccess";
        case hipGraphExecUpdateError: return "hipGraphExecUpdateError";
        case hipGraphExecUpdateErrorTopologyChanged: return "hipGraphExecUpdateErrorTopologyChanged";
        case hipGraphExecUpdateErrorNodeTypeChanged: return "hipGraphExecUpdateErrorNodeTypeChanged";
        case hipGraphExecUpdateErrorFunctionChanged: return "hipGraphExecUpdateErrorFunctionChanged";
        case hipGraphExecUpdateErrorParametersChanged: return "hipGraphExecUpdateErrorParametersChanged";
        case hipGraphExecUpdateErrorNotSupported: return "hipGraphExecUpdateErrorNotSupported";
        case hipGraphExecUpdateErrorUnsupportedFunctionChange: return "hipGraphExecUpdateErrorUnsupportedFunctionChange";
        default: return "(unknown hipGraphExecUpdateResult)";
    }
}

// ---------------------------------------------------------------------------
// Work kernel: spins on wall_clock64 for `iters` ticks.  wall_clock64 runs at
// hipDeviceAttributeWallClockRate, so iters maps to microseconds via
// ticks_per_us = wallClockKHz / 1000.  The atomic counter plus the acc
// dependency prevents the loop from being optimised away, and makes the gate
// exact: one increment per kernel launch, thread 0 / block 0 only.
// ---------------------------------------------------------------------------
__global__ void spin_kernel(unsigned long long* counter, unsigned long long iters) {
    const unsigned long long t0 = wall_clock64();
    unsigned long long acc = 0;
    while (wall_clock64() - t0 < iters) {
        acc += wall_clock64() & 0xf;
    }
    if (threadIdx.x == 0 && blockIdx.x == 0) {
        atomicAdd(counter, 1ull + (acc & 0ull));
    }
}

static double median(std::vector<double>& v) {
    if (v.empty()) return 0.0;
    std::sort(v.begin(), v.end());
    const size_t m = v.size() / 2;
    return (v.size() & 1) ? v[m] : 0.5 * (v[m - 1] + v[m]);
}

static double stdev(const std::vector<double>& v, double mean) {
    if (v.size() < 2) return 0.0;
    double acc = 0.0;
    for (double x : v) acc += (x - mean) * (x - mean);
    return std::sqrt(acc / (double)v.size());
}

// ---------------------------------------------------------------------------
// Captured-graph builders (stream capture, not explicit nodes).
//
// This is the single most important fidelity requirement: llama.cpp calls
// hipStreamBeginCapture / hipStreamEndCapture and then hipGraphInstantiate,
// with the HIP names arriving via CUDA aliases in ggml/src/ggml-cuda/vendors/hip.h.
// Using hipGraphAddKernelNode would exercise a different interposer path and
// would not be a valid proxy for llama.cpp's dispatch shape.
//
// Shapes (first-class axis):
//   cuda-serial -- every kernel on one stream, strict serial order (default).
//                  This is llama.cpp today; one weakly-connected component.
//   qkv-fork    -- each layer forks exactly the three projections (Q/K/V) onto
//                  up to 3 streams and rejoins before the rest of the layer,
//                  mirroring GGML_SCHED_GRAPH_OPTIMIZE.  Everything else stays
//                  a chain.  This is the only parallelism llama.cpp exposes.
//   amd-wide    -- same total kernels and same dependency semantics *within*
//                  each layer, but layers distributed round-robin across up to
//                  --paths streams so the graph has up to --paths independent
//                  components.  This breaks the residual chain and is explicitly
//                  hypothetical (probe reports that).  It measures the GAP
//                  between CUDA-shaped and AMD-shaped dispatch.
// ---------------------------------------------------------------------------
enum class Shape { CudaSerial, QkvFork, AmdWide };

struct CaptureStreams {
    hipStream_t primary = nullptr;    // capture stream (hipStreamBeginCapture)
    hipStream_t qkv_b = nullptr;      // secondary streams for Q/K/V
    hipStream_t qkv_c = nullptr;
    hipEvent_t ev_b = nullptr;
    hipEvent_t ev_c = nullptr;
    std::vector<hipStream_t> wide;    // for amd-wide
};

static void capture_streams_create(CaptureStreams& cs, Shape shape, int paths) {
    OK(hipStreamCreateWithFlags(&cs.primary, hipStreamNonBlocking));
    if (shape == Shape::QkvFork) {
        OK(hipStreamCreateWithFlags(&cs.qkv_b, hipStreamNonBlocking));
        OK(hipStreamCreateWithFlags(&cs.qkv_c, hipStreamNonBlocking));
        OK(hipEventCreate(&cs.ev_b));
        OK(hipEventCreate(&cs.ev_c));
    } else if (shape == Shape::AmdWide) {
        if (paths < 1) paths = 1;
        if (paths > 8) paths = 8; // GGML_CUDA_MAX_STREAMS 8
        int extra = paths - 1; // primary counts as one path
        if (extra < 0) extra = 0;
        cs.wide.resize(extra);
        for (int i = 0; i < extra; ++i) {
            OK(hipStreamCreateWithFlags(&cs.wide[i], hipStreamNonBlocking));
        }
    }
}

static void capture_streams_destroy(CaptureStreams& cs) {
    for (auto s : cs.wide) if (s) OK(hipStreamDestroy(s));
    cs.wide.clear();
    if (cs.ev_b) OK(hipEventDestroy(cs.ev_b));
    if (cs.ev_c) OK(hipEventDestroy(cs.ev_c));
    if (cs.qkv_c) OK(hipStreamDestroy(cs.qkv_c));
    if (cs.qkv_b) OK(hipStreamDestroy(cs.qkv_b));
    if (cs.primary) OK(hipStreamDestroy(cs.primary));
    cs = CaptureStreams{};
}

// Per-layer kernel count is the structural assumption.  Changing it changes the
// default node count; keep it in one place.
static const int kKernelsPerLayer = 12;
static const int kPrologueKernels = 6; // input embedding / norm + final norm + head

// Record the kernels for one token graph into the current capture.  The caller
// must have already called hipStreamBeginCapture(cs.primary, Global) and must
// call hipStreamEndCapture afterwards.  All launches are placed inside the
// capture so the graph the runtime instantiates has exactly the intended edges:
static void record_decode_graph(CaptureStreams& cs, Shape shape, int paths,
                                unsigned long long* d_counter,
                                unsigned long long work_iters, int layers) {
    if (shape == Shape::CudaSerial) {
        for (int i = 0; i < kPrologueKernels; ++i) {
            hipLaunchKernelGGL(spin_kernel, dim3(1), dim3(64), 0, cs.primary, d_counter, work_iters);
        }
        for (int L = 0; L < layers; ++L) {
            for (int k = 0; k < kKernelsPerLayer; ++k) {
                hipLaunchKernelGGL(spin_kernel, dim3(1), dim3(64), 0, cs.primary, d_counter, work_iters);
            }
        }
        return;
    }
    if (shape == Shape::QkvFork) {
        for (int i = 0; i < kPrologueKernels; ++i) {
            hipLaunchKernelGGL(spin_kernel, dim3(1), dim3(64), 0, cs.primary, d_counter, work_iters);
        }
        for (int L = 0; L < layers; ++L) {
            hipLaunchKernelGGL(spin_kernel, dim3(1), dim3(64), 0, cs.primary, d_counter, work_iters); // Q
            hipLaunchKernelGGL(spin_kernel, dim3(1), dim3(64), 0, cs.qkv_b, d_counter, work_iters);    // K
            hipLaunchKernelGGL(spin_kernel, dim3(1), dim3(64), 0, cs.qkv_c, d_counter, work_iters);    // V
            OK(hipEventRecord(cs.ev_b, cs.qkv_b));
            OK(hipEventRecord(cs.ev_c, cs.qkv_c));
            OK(hipStreamWaitEvent(cs.primary, cs.ev_b, 0));
            OK(hipStreamWaitEvent(cs.primary, cs.ev_c, 0));
            for (int k = 3; k < kKernelsPerLayer; ++k) {
                hipLaunchKernelGGL(spin_kernel, dim3(1), dim3(64), 0, cs.primary, d_counter, work_iters);
            }
        }
        return;
    }
    // Shape::AmdWide
    // Same total nodes (layers*12+6), but layers distributed round-robin across
    // wide streams.  Within each layer kernels are serial on that layer's
    // stream, but layers assigned to different streams become independent
    // paths.  This intentionally breaks the residual chain (layer N depends on
    // layer N-1 in a real model) and is therefore hypothetical; the probe
    // reports that explicitly.  No cross-stream events are recorded, so the
    // graph has up to paths disconnected components.  The capturing stream
    // (primary) is one of the paths, so even on runtimes where Global capture
    // only records the capturing stream, at least that path's nodes are
    // captured (gate still fails if other paths are dropped, which is honest).
    {
        std::vector<hipStream_t> pool;
        pool.reserve(1 + cs.wide.size());
        pool.push_back(cs.primary);
        for (auto s : cs.wide) pool.push_back(s);
        int P = (int)pool.size();
        if (P < 1) P = 1;
        for (int i = 0; i < kPrologueKernels; ++i) {
            hipStream_t s = pool[i % P];
            hipLaunchKernelGGL(spin_kernel, dim3(1), dim3(64), 0, s, d_counter, work_iters);
        }
        for (int L = 0; L < layers; ++L) {
            hipStream_t s = pool[L % P];
            for (int k = 0; k < kKernelsPerLayer; ++k) {
                hipLaunchKernelGGL(spin_kernel, dim3(1), dim3(64), 0, s, d_counter, work_iters);
            }
        }
    }
}

static int total_nodes(int layers) {
    return layers * kKernelsPerLayer + kPrologueKernels;
}

// Capture one graph and instantiate it; return exec and optionally the graph.
static hipGraphExec_t capture_and_instantiate(CaptureStreams& cs, Shape shape, int paths,
                                              unsigned long long* d_counter,
                                              unsigned long long work_iters, int layers,
                                              hipGraph_t* out_graph = nullptr) {
    OK(hipStreamBeginCapture(cs.primary, hipStreamCaptureModeGlobal));
    record_decode_graph(cs, shape, paths, d_counter, work_iters, layers);
    hipGraph_t g = nullptr;
    OK(hipStreamEndCapture(cs.primary, &g));
    if (out_graph) {
        *out_graph = g;
    }
    hipGraphExec_t exec = nullptr;
    OK(hipGraphInstantiate(&exec, g, nullptr, nullptr, 0));
    if (!out_graph) {
        OK(hipGraphDestroy(g));
    }
    return exec;
}

static const char* shape_name(Shape s) {
    switch (s) {
        case Shape::CudaSerial: return "cuda-serial";
        case Shape::QkvFork: return "qkv-fork";
        case Shape::AmdWide: return "amd-wide";
    }
    return "(unknown)";
}

static Shape parse_shape(const char* v) {
    if (strcmp(v, "cuda-serial") == 0) return Shape::CudaSerial;
    if (strcmp(v, "qkv-fork") == 0) return Shape::QkvFork;
    if (strcmp(v, "amd-wide") == 0) return Shape::AmdWide;
    printf("unknown --shape=%s (expected cuda-serial|qkv-fork|amd-wide)\n", v);
    exit(1);
}

static void print_usage(const char* prog) {
    printf("Usage: %s [--shape=NAME] [--paths=N] [--layers=N] [--tokens=N] [--work=ITERS] [--warmups=N] [--update] [--help]\n", prog);
    printf("  --shape=NAME  cuda-serial (default, 1 stream chain), qkv-fork (Q/K/V on 3 streams),\n");
    printf("                amd-wide (layers round-robin across --paths streams, hypothetical)\n");
    printf("  --paths=N     for amd-wide: number of independent paths (1..8, default 4; GGML_CUDA_MAX_STREAMS=8)\n");
    printf("  --layers=N    number of transformer layers (default 32; nodes = layers*%d + %d)\n", kKernelsPerLayer, kPrologueKernels);
    printf("  --tokens=N    graph replays (default 200): instantiate once then launch N times\n");
    printf("  --work=ITERS  wall_clock64 ticks each kernel spins (default 0 = near-empty)\n");
    printf("                sweep e.g. 0,500,5000,20000 to span submission- to compute-dominated\n");
    printf("  --warmups=N   untimed warmup launches before the measured window (default 20)\n");
    printf("  --qkv-fork    deprecated alias for --shape=qkv-fork\n");
    printf("  --update      between replays, capture a new graph and try hipGraphExecUpdate; on\n");
    printf("                failure report the result verbatim and fall back to relaunching exec\n");
    printf("\n");
    printf("Each kernel does atomicAdd(counter,1) on thread 0/block 0, so the gate checks\n");
    printf("counter == tokens * nodes_per_token exactly.\n");
}

int main(int argc, char** argv) {
    int layers = 32;
    int tokens = 200;
    int warmups = 20;
    unsigned long long work_iters = 0;
    Shape shape = Shape::CudaSerial;
    int paths = 4;
    bool do_update = false;
    bool help = false;

    for (int i = 1; i < argc; ++i) {
        if (strncmp(argv[i], "--shape=", 8) == 0) shape = parse_shape(argv[i] + 8);
        else if (strncmp(argv[i], "--paths=", 8) == 0) paths = atoi(argv[i] + 8);
        else if (strncmp(argv[i], "--layers=", 9) == 0) layers = atoi(argv[i] + 9);
        else if (strncmp(argv[i], "--tokens=", 9) == 0) tokens = atoi(argv[i] + 9);
        else if (strncmp(argv[i], "--work=", 7) == 0) work_iters = strtoull(argv[i] + 7, nullptr, 10);
        else if (strncmp(argv[i], "--warmups=", 10) == 0) warmups = atoi(argv[i] + 10);
        else if (strcmp(argv[i], "--qkv-fork") == 0) shape = Shape::QkvFork;
        else if (strcmp(argv[i], "--update") == 0) do_update = true;
        else if (strcmp(argv[i], "--help") == 0 || strcmp(argv[i], "-h") == 0) help = true;
        else {
            printf("unknown argument: %s\n", argv[i]);
            print_usage(argv[0]);
            return 1;
        }
    }
    if (help) {
        print_usage(argv[0]);
        return 0;
    }
    if (layers < 1) layers = 1;
    if (tokens < 1) tokens = 1;
    if (warmups < 0) warmups = 0;
    if (paths < 1) paths = 1;
    if (paths > 8) paths = 8;

    const int nodes_per_token = total_nodes(layers);
    const long long expected_total_kernels = (long long)tokens * (long long)nodes_per_token;

    OK(hipSetDevice(0));
    hipDeviceProp_t prop{};
    OK(hipGetDeviceProperties(&prop, 0));
    int wall_khz = 0;
    if (hipDeviceGetAttribute(&wall_khz, hipDeviceAttributeWallClockRate, 0) != hipSuccess || wall_khz <= 0) {
        wall_khz = 25000;
    }
    const double ticks_per_us = (double)wall_khz / 1000.0;
    const double expected_compute_us_per_kernel = work_iters / ticks_per_us;
    const double expected_compute_us_per_token = expected_compute_us_per_kernel * (double)nodes_per_token;

    printf("=== %s (%s) ===\n", prop.name, prop.gcnArchName);
    print_runtime_provenance();
    printf("llama.cpp decode-shape probe (stream capture, synthetic dispatch shape)\n");
    printf("layers=%d  nodes/token=%d  (= %d*%d + %d)  tokens=%d  warmups=%d  work=%llu ticks  shape=%s%s\n",
           layers, nodes_per_token, layers, kKernelsPerLayer, kPrologueKernels, tokens, warmups, work_iters,
           shape_name(shape), shape == Shape::AmdWide ? " (hypothetical)" : "");
    if (shape == Shape::AmdWide) {
        printf("shape amd-wide: --paths=%d (independent paths, same total kernels; breaks residual chain)\n", paths);
    } else if (shape == Shape::QkvFork) {
        printf("shape qkv-fork: Q/K/V per layer on 3 streams then rejoin (GGML_SCHED_GRAPH_OPTIMIZE)\n");
    } else {
        printf("shape cuda-serial: one stream, serial residual chain (llama.cpp today; Unsplittable)\n");
    }
    printf("wall_clock64: %d kHz  ->  %.3f ticks/us  ->  expected compute %.3f us/kernel  %.3f us/token at this --work\n",
           wall_khz, ticks_per_us, expected_compute_us_per_kernel, expected_compute_us_per_token);
    printf("graph: hipStreamBeginCapture / hipStreamEndCapture -> hipGraphInstantiate, replay %d times\n", tokens);
    if (do_update) {
        printf("update: hipGraphExecUpdate will be attempted between replays (fallback to same exec on failure)\n");
    }
    printf("\n");

    unsigned long long* d_counter = nullptr;
    OK(hipMalloc(&d_counter, sizeof(unsigned long long)));
    OK(hipMemset(d_counter, 0, sizeof(unsigned long long)));
    OK(hipDeviceSynchronize());

    CaptureStreams cs{};
    capture_streams_create(cs, shape, paths);

    hipGraph_t instantiated_graph = nullptr;
    hipGraphExec_t exec = capture_and_instantiate(cs, shape, paths, d_counter, work_iters, layers, &instantiated_graph);

    {
        size_t captured = 0;
        hipError_t ge = hipGraphGetNodes(instantiated_graph, nullptr, &captured);
        size_t edge_cnt = 0;
        if (ge == hipSuccess) {
            // Try to get edge count as well (from/to arrays optional)
            size_t ecap = 0;
            hipGraphGetEdges(instantiated_graph, nullptr, nullptr, &ecap);
            edge_cnt = ecap;
        }
        if (ge == hipSuccess) {
            printf("captured graph: %zu nodes, %zu edges (expected %d kernel nodes)\n", captured, edge_cnt, nodes_per_token);
            if ((int)captured != nodes_per_token) {
                printf("note: captured node count != expected. On this ROCm, stream capture may not\n");
                printf("have recorded kernels on secondary streams (only the capturing stream).\n");
                if (shape != Shape::CudaSerial) {
                    printf("If --shape=%s is set, this explains a gate FAIL: the graph replayed fewer\n",
                           shape_name(shape));
                    printf("kernels than claimed and its timing is therefore not a valid proxy for\n");
                    printf("that shape. The gate below will suppress it.\n");
                }
            }
            // Make independent-component status obvious.
            if (shape == Shape::CudaSerial) {
                printf("graph connectivity: single weakly-connected component (serial chain, Unsplittable)\n");
            } else if (shape == Shape::QkvFork) {
                printf("graph connectivity: one component with a 3-way fork per layer (still one WCC, tiny parallelism)\n");
                if (captured != (size_t)nodes_per_token) {
                    printf("  but capture dropped secondary-stream nodes, so fork had no effect on this ROCm.\n");
                }
            } else { // AmdWide
                printf("graph connectivity: up to %d independent components (hypothetical; breaks residual chain)\n", paths);
                printf("  amd-wide: same %d kernels but layers round-robin across %d streams. In a real\n", nodes_per_token, paths);
                printf("  model each layer depends on the previous residual add, so this graph is NOT\n");
                printf("  a valid llama.cpp decode graph -- it measures the GAP between CUDA-shaped\n");
                printf("  and hypothetical AMD-shaped dispatch. If this shape shows a multi-queue win\n");
                printf("  where cuda-serial does not, that delta is the amount left on the table.\n");
                if (captured != (size_t)nodes_per_token) {
                    printf("  on this ROCm, multi-stream capture is incomplete (got %zu vs %d), so the\n", captured, nodes_per_token);
                    printf("  wide shape's parallelism was not faithfully captured; gate will FAIL.\n");
                }
            }
        } else {
            printf("captured graph: hipGraphGetNodes -> %s (%d)\n", hipGetErrorString(ge), (int)ge);
        }
    }

    hipGraph_t update_graph = nullptr;
    if (do_update) {
        OK(hipStreamBeginCapture(cs.primary, hipStreamCaptureModeGlobal));
        record_decode_graph(cs, shape, paths, d_counter, work_iters, layers);
        OK(hipStreamEndCapture(cs.primary, &update_graph));
    }
    for (int i = 0; i < warmups; ++i) {
        OK(hipGraphLaunch(exec, cs.primary));
        OK(hipStreamSynchronize(cs.primary));
    }
    OK(hipMemset(d_counter, 0, sizeof(unsigned long long)));
    OK(hipDeviceSynchronize());

    hipEvent_t ev0 = nullptr, ev1 = nullptr;
    OK(hipEventCreate(&ev0));
    OK(hipEventCreate(&ev1));

    std::vector<double> host_per_token, event_per_token;
    host_per_token.reserve(tokens);
    event_per_token.reserve(tokens);
    int update_successes = 0;
    int update_failures = 0;
    hipGraphExecUpdateResult last_update_result = hipGraphExecUpdateSuccess;
    bool any_update_failure_reported = false;

    for (int t = 0; t < tokens; ++t) {
        if (do_update && update_graph) {
            hipGraphExecUpdateResult r = hipGraphExecUpdateSuccess;
            hipGraphNode_t err_node = nullptr;
            hipError_t ue = hipGraphExecUpdate(exec, update_graph, &err_node, &r);
            if (ue != hipSuccess) {
                printf("[update token %d] hipGraphExecUpdate -> %s (%d) result=%s (%d) err_node=%p -- falling back to relaunch of same exec\n",
                       t, hipGetErrorString(ue), (int)ue,
                       hipGraphExecUpdateResultString(r), (int)r, (void*)err_node);
                any_update_failure_reported = true;
                ++update_failures;
            } else if (r != hipGraphExecUpdateSuccess) {
                printf("[update token %d] hipGraphExecUpdate result %s (%d) err_node=%p -- falling back to relaunch of same exec\n",
                       t, hipGraphExecUpdateResultString(r), (int)r, (void*)err_node);
                any_update_failure_reported = true;
                ++update_failures;
            } else {
                ++update_successes;
            }
            last_update_result = r;
            (void)last_update_result;
        }

        OK(hipEventRecord(ev0, cs.primary));
        const auto t0 = std::chrono::steady_clock::now();
        OK(hipGraphLaunch(exec, cs.primary));
        OK(hipStreamSynchronize(cs.primary));
        const auto t1 = std::chrono::steady_clock::now();
        OK(hipEventRecord(ev1, cs.primary));
        OK(hipEventSynchronize(ev1));
        float ms = 0.0f;
        OK(hipEventElapsedTime(&ms, ev0, ev1));
        double host_us = std::chrono::duration<double, std::micro>(t1 - t0).count();
        double event_us = (double)ms * 1000.0;
        host_per_token.push_back(host_us);
        event_per_token.push_back(event_us);
    }

    unsigned long long counted = 0;
    OK(hipMemcpy(&counted, d_counter, sizeof(counted), hipMemcpyDeviceToHost));
    const unsigned long long expected = (unsigned long long)expected_total_kernels;
    const bool gate_ok = (counted == expected);

    double host_med = median(host_per_token);
    double event_med = median(event_per_token);
    double host_mean = 0.0, event_mean = 0.0;
    for (double x : host_per_token) host_mean += x;
    for (double x : event_per_token) event_mean += x;
    if (!host_per_token.empty()) host_mean /= (double)host_per_token.size();
    if (!event_per_token.empty()) event_mean /= (double)event_per_token.size();
    double host_sd = stdev(host_per_token, host_mean);
    double event_sd = stdev(event_per_token, event_mean);

    const double lo = std::min(host_med, event_med);
    const double hi = std::max(host_med, event_med);
    const bool clocks_agree = lo > 0.0 && (hi / lo) < 1.25;
    const char* clock_flag = clocks_agree ? "agree" : "DIVERGE";

    const double host_us_per_dispatch_med = nodes_per_token > 0 ? host_med / (double)nodes_per_token : 0.0;
    const double event_us_per_dispatch_med = nodes_per_token > 0 ? event_med / (double)nodes_per_token : 0.0;
    const double measured_kernel_us = event_us_per_dispatch_med;

    double submission_share = 0.0;
    if (event_med > 0.0) {
        double uncovered = event_med - expected_compute_us_per_token;
        if (uncovered < 0.0) uncovered = 0.0;
        submission_share = uncovered / event_med;
    }
    if (submission_share < 0.0) submission_share = 0.0;
    if (submission_share > 1.0) submission_share = 1.0;

    const double host_tok_s = host_med > 0.0 ? 1e6 / host_med : 0.0;
    const double event_tok_s = event_med > 0.0 ? 1e6 / event_med : 0.0;

    printf("--- gate ---\n");
    printf("gate %s  counted %llu / expected %llu  (tokens %d * nodes %d)  shape=%s\n",
           gate_ok ? "ok" : "FAIL", counted, expected, tokens, nodes_per_token, shape_name(shape));
    if (shape == Shape::AmdWide) {
        printf("shape amd-wide note: this graph breaks the residual dependency (hypothetical).\n");
        printf("  counted vs expected still checks exact kernel execution, but the graph\n");
        printf("  itself is not a valid llama.cpp decode graph; use only for GAP measurement.\n");
    }
    if (!gate_ok) {
        printf("FAIL: expected exactly %llu kernel executions; observed %llu.\n", expected, counted);
        printf("Timing below measured less work than claimed and MUST NOT be used to compare runtimes.\n");
    }
    if (do_update) {
        printf("hipGraphExecUpdate: %d success, %d failure", update_successes, update_failures);
        if (any_update_failure_reported) printf(" (failures reported verbatim above; fell back to same exec)");
        printf("\n");
    }
    printf("\n");

    printf("--- timing (median over %d replays of one captured graph; gate %s; shape=%s) ---\n",
           tokens, gate_ok ? "ok" : "FAIL", shape_name(shape));
    if (!gate_ok) {
        printf("timing suppressed: gate FAIL -- the replays did not execute the claimed node count.\n");
    } else {
        printf("host median  : %9.3f us/token  (mean %.3f  stdev %.3f)  synthetic %.1f tok/s\n",
               host_med, host_mean, host_sd, host_tok_s);
        printf("event median : %9.3f us/token  (mean %.3f  stdev %.3f)  synthetic %.1f tok/s\n",
               event_med, event_mean, event_sd, event_tok_s);
        printf("clocks: host vs event %s  (host/event %.3f; flag DIVERGE if ratio >=1.25)\n",
               clock_flag, host_med > 0.0 ? event_med / host_med : 0.0);
        printf("per-dispatch : %9.3f host us/dispatch  %9.3f event us/dispatch  (dispatch = kernel node)\n",
               host_us_per_dispatch_med, event_us_per_dispatch_med);
        printf("per-kernel measured (event): %.3f us/kernel  (event us/token / %d)\n",
               measured_kernel_us, nodes_per_token);
        printf("expected compute at this --work: %.3f us/kernel  %.3f us/token  (%llu ticks @ %.3f ticks/us)\n",
               expected_compute_us_per_kernel, expected_compute_us_per_token, work_iters, ticks_per_us);
        printf("submission-cost share of measured token time: %.1f%%  (1 - expected_compute/event; 100%% at --work=0)\n",
               submission_share * 100.0);
        printf("synthetic tokens/s is a dispatch-shape proxy, not a real model's throughput.\n");
    }
    printf("\n");

    if (gate_ok) {
        printf("Reading this run:\n");
        printf("  - Compare host us/token (and event us/token when they agree) between\n");
        printf("    stock and LD_PRELOAD=libredline_hipgraph.so at the SAME --shape/--work\n");
        printf("    and same --layers/--paths.  A lower number under the interposer is the\n");
        printf("    per-token saving; divide by %d to get saving per dispatch.\n", nodes_per_token);
        printf("  - At --work=0 token time is submission/fencing; at --work=20000 compute\n");
        printf("    (tens of us) dominates and submission share falls toward 0%%.  The\n");
        printf("    interposer can only help the submission fraction; the sweep makes the\n");
        printf("    ceiling explicit.\n");
        if (shape == Shape::CudaSerial) {
            printf("  - This shape is one weakly-connected chain (cuda-serial, llama.cpp today),\n");
            printf("    so a multi-queue win requires distinct segments.  If stock vs\n");
            printf("    interposer agree within noise at every --work, the interposer does not\n");
            printf("    help this shape.  Both outcomes are reportable.\n");
        } else if (shape == Shape::QkvFork) {
            printf("  - This shape has a 3-way fork per layer (qkv-fork, GGML_SCHED_GRAPH_OPTIMIZE),\n");
            printf("    still essentially a chain.  Compare to cuda-serial at same --work to see\n");
            printf("    whether llama.cpp's own optional fork helps on this ROCm.\n");
        } else {
            printf("  - This shape is amd-wide with up to %d independent components.\n", paths);
            printf("    It is hypothetical: it breaks the residual chain (layers are independent\n");
            printf("    where a real model is serial).  Compare its us/token to cuda-serial at the\n");
            printf("    same --work to estimate the GAP between CUDA-shaped and AMD-shaped dispatch.\n");
            printf("    If amd-wide shows a multi-queue win where cuda-serial does not, that delta\n");
            printf("    is what a re-expressed engine could in principle recover.\n");
        }
        printf("  - Build the 2-D table: shape {cuda-serial,qkv-fork,amd-wide} x work {0,500,5000,20000}\n");
        printf("    for stock vs interposer (REDLINE_HIPGRAPH_LANES=off vs auto).  That answers:\n");
        printf("    (a) does the interposer help llama.cpp as it exists, (b) does llama.cpp's\n");
        printf("    own fork help, and (c) how much is left on the table.\n");
    } else {
        printf("No reading: gate FAIL.  Fix shape/capture before comparing runtimes.\n");
        if (shape == Shape::AmdWide || shape == Shape::QkvFork) {
            printf("  On this ROCm, global stream capture did not record secondary-stream kernels\n");
            printf("  (captured nodes < expected).  Try a different ROCm or treat this shape as\n");
            printf("  unsupported on this build; the gate correctly suppressed its timing.\n");
        }
    }

    OK(hipEventDestroy(ev0));
    OK(hipEventDestroy(ev1));
    if (update_graph) OK(hipGraphDestroy(update_graph));
    OK(hipGraphExecDestroy(exec));
    OK(hipGraphDestroy(instantiated_graph));
    capture_streams_destroy(cs);
    OK(hipFree(d_counter));

    return gate_ok ? 0 : 2;
}
