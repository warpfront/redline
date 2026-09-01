<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# ROCm 10.0 graph scheduler: the collapse is a guard, and the classic path still fans out

Dated 2026-09-01. Internal. NOT posted upstream.

## Why this exists

`2026-08-27-stream-concurrency-and-queue-cap.md` measured that ROCm 10.0
converges independent and fanout graphs onto serial-chain cost (gfx1100
2.78 µs all three shapes) where 7.14 ran independent nodes 2.1–2.7× cheaper.
A source survey of `release/therock-10.0` (agent transcripts `ClrGraphPath`,
`RocrAndProfiler`) located the mechanism, and it is switchable at runtime, so
this artifact tests it.

## Source facts (`ROCm/rocm-systems@release/therock-10.0`)

- Every captured kernel packet in a graph carries **barrier=1, acquire=AGENT,
  release=AGENT** and **no completion signal** at capture
  (`projects/clr/rocclr/device/rocm/rocvirtual.cpp:2227-2262`, header baked
  once from `fenceScopeAgent_ = AMD_OPT_FLUSH`, default 1; capture path
  `rocvirtual.cpp:1556-1581`). The only flag that changes fence scope is
  `AMD_OPT_FLUSH=0`, which widens to SYSTEM. The barrier bit can only be
  cleared through `hipExtAnyOrderLaunch`, which graph nodes set only under
  `DEBUG_HIP_FORCE_ASYNC_QUEUE` (default false).
- 10.0 instantiates `GraphExecSegmented` unless `DEBUG_HIP_GRAPH_CLASSIC_PATH=1`
  (`hip_graph.cpp:1600-1622`). Its `BuildSyncPlan` PASS 0
  (`hip_graph_internal.cpp:417-428`) collapses every segment onto stream 0 when
  `DEBUG_HIP_GRAPH_SEGMENT_SCHEDULING==0` (default) and
  `ShouldCollapseToSingleStream()` (`1438-1516`) finds
  `parallel_slack < DEBUG_HIP_GRAPH_MIN_OVERLAP * (barrier_est + signal_est)`
  (default MIN_OVERLAP=2). For N independent leaves `signal_est = N`, so the
  gate `(N-1) < 2N` always collapses.
- All of `DEBUG_HIP_GRAPH_MIN_OVERLAP`, `DEBUG_HIP_GRAPH_SEGMENT_SCHEDULING`,
  `DEBUG_HIP_GRAPH_CLASSIC_PATH`, `DEBUG_HIP_FORCE_GRAPH_QUEUES` are
  `release()` flags (`flags.hpp:244-259`), i.e. env-settable on shipped builds.
- CLR contains no PM4 indirect-buffer path for kernels or graphs; the only PM4
  emit is `dispatchCounterAqlPacket` for AQLProfile counters
  (`rocvirtual.cpp:1969-1984`). ROCr's `AqlQueue::ExecutePM4`
  (`amd_aql_queue.cpp:1680-1709`, vendor packet `AMD_AQL_FORMAT_PM4_IB`) is
  internal-only, used for cache `ACQUIRE_MEM` and PC-sampling fallback. No DRM
  user-mode-queue backend exists in ROCr 10.0 (`DRM_IOCTL_AMDGPU_USERQ` is
  defined in the bundled `amdgpu_drm.h` with zero call sites).

## Experiment

`bench/dispatch/graph_dependency_fencing.cpp`, built with the 10.0 toolchain
and run under `/opt/rocm/core-10.0/lib` (provenance printed per run:
`hipRuntimeGetVersion=71526333`, `libamdhip64=/opt/rocm/core-10.0/lib/...`).
N=512 kernel nodes; gfx1201 60 replays, gfx1151 40 replays; median host µs per
dispatch; every arm passed its correctness gate. Only the env flag varies.

| GPU | flags | chain | independent | fanout-join |
| --- | --- | ---: | ---: | ---: |
| gfx1201 (RX 9070 XT, local) | default (segmented + collapse) | 2.365 | 2.362 | 2.361 |
| | `DEBUG_HIP_GRAPH_MIN_OVERLAP=0` | 2.364 | **17.600** | **18.175** |
| | `DEBUG_HIP_GRAPH_SEGMENT_SCHEDULING=1` | 2.355 | 17.598 | 18.142 |
| | `DEBUG_HIP_GRAPH_CLASSIC_PATH=1` | 2.750 | 7.897 | 8.672 |
| gfx1151 (Radeon 8060S, hipx) | default (segmented + collapse) | 1.744 | 1.744 | 1.744 |
| | `DEBUG_HIP_GRAPH_MIN_OVERLAP=0` | 1.743 | 2.708 | 2.641 |
| | `DEBUG_HIP_GRAPH_CLASSIC_PATH=1` | 1.842 | **1.156** | 1.663 |

Reference, 7.14 matched pairs from the 08-27 artifact: gfx1151 chain 1.741 /
independent 0.817 / fanout 0.921.

hipx note: gfx1100 was carrying a live `hipfire serve` VMM capacity test at run
time and was deliberately not touched; gfx1151 (HIP device 1) was idle.

## Reading

1. **The collapse is a guard, not the regression.** Turning it off on the
   segmented path makes independent graphs *worse* than a chain on both
   architectures: 1.55× worse on gfx1151, **7.4× worse on gfx1201**. AMD's
   heuristic is doing exactly what its comment says: avoiding a multi-stream
   sync plan whose cost exceeds the overlap it buys.
2. **The regression is in the segmented multi-stream sync plan itself.** The
   old `GraphExecClassic` path, still present behind
   `DEBUG_HIP_GRAPH_CLASSIC_PATH=1`, fans independent nodes out at 1.156 µs
   on gfx1151 — 1.6× cheaper than its own chain and within 1.4× of 7.14's
   0.817. The segmented path's un-collapsed multi-stream mode costs 2.708 on
   the same graph. Same hardware, same runtime, same packets per node; only
   the cross-stream sync plan differs. `[INFERENCE]` from the source: the
   segmented plan attaches a completion signal per leaf and accumulates them
   (`BuildSyncPlan` PASS1/3, `EnqueueSegmentedGraph`), whereas classic joins
   through stream markers; the ~18 µs per-leaf cost on gfx1201 is the same
   magnitude as the measured per-launch-sync cost (18.5 µs), which is what a
   per-leaf interrupting signal would look like.
3. **gfx1201 penalises multi-queue graph fanout on every path.** Classic
   independent is 3.3× worse than chain there (7.897 vs 2.365), consistent
   with the 08-24 single-measurement figure (6.432) that was withdrawn on
   08-27 as a possible queue-count artifact. That withdrawal was over-cautious:
   the gfx1201 penalty is real and architecture-specific. It also matches
   the multi-queue PM4 result (gfx1201 lane optimum 2; 8 lanes 127× worse).
   gfx12 multi-queue behaviour is a separate question from the CLR scheduler.
4. **None of this moves the chain floor.** Chain cost is flat across every
   flag on both parts (2.36/1.74), as expected from the header facts above:
   the serial floor is barrier-bit ordering plus agent-scope fences per
   packet, and no shipped flag thins either.

## What it enables

- A comment on `rocm-systems#10834`/`#10836` (both AMD-filed, zero comments)
  that is new relative to their filing: 10.0 changed the graph exec, here is
  the flag matrix with in-process runtime provenance, classic still fans out,
  segmented multi-stream regressed, gfx12 penalises fanout regardless.
  Gated on user approval like everything else.
- A candidate CLR PR with a clear target: make `GraphExecSegmented`'s
  multi-stream sync plan no worse than classic's for leaf-heavy graphs, then
  recalibrate `ShouldCollapseToSingleStream` so launch-bound independent
  graphs can use it. Requires reading `BuildSyncPlan` PASS1/3 and
  `EnqueueSegmentedGraph` leaf handling; the regression test is this probe.
- Correction owed to the 08-27 #6409 comment: the gfx1201 shape figure was
  withdrawn as possibly artefactual; it was not.

## Not established

- Why the segmented multi-stream plan is costlier than classic (packet-level
  attribution not done; see [INFERENCE] above).
- Whether gfx1201's multi-queue penalty is MES scheduling, queue mapping, or
  firmware; only its magnitude is measured.
- gfx1100 and gfx1030 rows for this matrix (hipx gfx1100 busy; gfx1030 not run).
