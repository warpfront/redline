<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# Integrating Redline into an inference engine

Redline replaces an engine's HipGraph / CUDA-graph *replay* at its dispatch
seam. The engine keeps its kernels; Redline changes how the fixed per-token
launch sequence is submitted — one retained submission with derived minimal
fences instead of a graph relaunch that re-fences every dispatch.

This document pins the seam in two reference engines and the exact A/B protocol
to prove the speedup. Both engines are cloned under `.engines/` (gitignored) for
local/hiptrx testing.

## What Redline needs from the engine (the binding contract)

Redline drives the *order, lanes, and fences*; the engine supplies the *kernel
identities and buffers*:

1. **Kernel identity** — for each graph node, the loaded code-object symbol
   (`kernel_object`) and its kernarg ABI. Redline binds these through
   `redline_dispatch::ArtifactCatalog` / `KernelArtifactIdentity`.
2. **Buffer access** — each node's reads/writes (`Access::read/write` over
   device regions). The engine already knows this; it is the one datum HipGraph
   infers implicitly.
3. **A real backend** — `redline_dispatch::hip::HipMultiStreamBackend` (or the
   public-AQL replay in `redline_dispatch::aql`) executes the retained
   submission. The architecture-neutral AQL path and vendor PM4-IB carrier are
   live-validated on gfx1010, gfx1030, gfx1100, gfx1151, and gfx1201. Direct
   compute PM4 uses separate legacy GFX10/GFX11 and GFX12 encoders. The legacy
   path admits zero-scratch HSA kernels with kernarg plus optional
   private-segment-buffer user SGPRs, encodes static and dynamic LDS, and fails
   closed on unsupported implicit inputs or scratch.

The C-ABI (`redline-capi`) and Python (`redline-py`) currently expose
record → instantiate → plan-inspect → **mock** launch. The remaining
integration hook is a real-GPU launch entry (`rl_graphexec_launch_hip`) that
takes the engine's bound kernel objects; its Rust side already exists
(`HipMultiStreamBackend::prepare_plan` + `replay`).

## Seam 1 — lucebox (llama.cpp ggml-cuda backend)

lucebox vendors llama.cpp; the graph replay is the standard ggml CUDA-graph path
(hipified to `hipGraph*` on AMD):

- `.engines/lucebox/server/deps/llama.cpp/ggml/src/ggml-cuda/ggml-cuda.cu`
  - `cudaStreamBeginCapture` — line ~4414 (capture the decode graph)
  - `cudaGraphInstantiate` — lines ~3306, ~4289
  - `cudaGraphExecUpdate` — lines ~3289/3293 (re-point params without re-instantiate)
  - `cudaGraphLaunch` — line ~4295 (the replay to replace)
  - graph state struct — `common.cuh` ~1174-1183 (`cudaGraph_t`, `cudaGraphExec_t`, nodes)

**Swap:** in `evaluate_and_capture_cuda_graph` (the function around 4270-4420),
build the node list into a `redline_dispatch` graph once (kernel identity +
per-node buffer access from the ggml tensors), and replace `cudaGraphLaunch`
with a Redline replay. Reuse ggml's existing "graph is reusable" check as the
Redline re-instantiate trigger.

**Build:** `GGML_HIP=ON` build of lucebox, link `libredline_dispatch.a`
(C-ABI). **A/B:** decode tok/s, `GGML_CUDA_USE_GRAPHS=1` (hipGraph) vs Redline,
same model/prompt, GPU timestamps, ABBA interleave ≥10 warmups, auto clocks.

## Seam 2 — hipEngine (dispatch-floor micro + decode)

hipEngine is Python; its graph handling is strategy-selected:

- `.engines/hipEngine/benchmarks/micro/timing_contract.py` —
  `_PRE_RECORDED_STRATEGIES = {"hip_graph", "vulkan_command_buffer"}`
- `.engines/hipEngine/benchmarks/micro/runners/hip_dispatch_floor.py` — the
  `hip_graph` (serial-latency) vs `multi_stream` runner. **This is the harness
  behind ROCm/ROCm#6409.**
- decode capture: `scripts/qwen35_gguf_bench.py` (`graph_capture_seconds`,
  `capture_start`) and `scripts/persistent_barrier_microbench.hip`.

**Swap:** add a `"redline"` strategy next to `"hip_graph"` /
`"vulkan_command_buffer"`. Build the narrow-kernel chain through the
`redline_dispatch` Python wheel (`rl.Graph`), launch through Redline's HIP
backend bound to the same kernel the `hip_graph` path uses.

**A/B:** run the harness's own `serial_latency` contract with `hip_graph` vs
`redline`. This reproduces #6409 *with a Redline column* — the direct,
adversarial proof: Redline under the HIP-graph floor lhl reported, on his own
harness.

## Measurement protocol (both engines)

- GPU timestamps (not host wall) for the replay window.
- Cold-cache differential where relevant; auto clocks, no pinning.
- ABBA interleave, ≥10 warmups (RDNA boost is ±10% noisy — require a consistent
  multi-round win).
- Correctness gate first: identical output vs the HipGraph path before any timing
  is reported.
- Report the ratio honestly; a neutral/negative result is documented, not hidden.

## Status

- **Done & verified:** C-ABI + PyO3 bindings (Rust ≡ C ≡ Python plan
  fingerprint); engines cloned; seams located; repo synced to hiptrx; retained
  PM4 submission runs on the R9700; public-ROCr AQL plus retained direct PM4
  correctness pass on hipx's gfx1010, gfx1030, gfx1100, and gfx1151 devices.
- **Pending (the integration):** bind Redline's real HIP/AQL backend to each
  engine's kernel objects at the seam above, then run the A/B. The proven
  per-engine ratio comes from that step; it is not asserted here.
