<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# Why ROCm 10.0 changes concurrent-graph cost: queue distribution

**Status: internal. NOT posted upstream. Held pending review.**

The timing result (`2026-08-27-graph-shape-rocm714-vs-100.md`) shows 10.0
converging every graph shape onto chain cost. This document establishes the
mechanism using AMD's own instrument rather than inference from timing.

## Instrument

`rocprofv3` 1.3.5, shipped in ROCm 10.0, and `rocprofv3` from 7.14 for the 7.14
arms — each release profiled with its own tool against its own runtime, never
mixed. Kernel-dispatch records carry `dispatch_info.queue_id`, the runtime's own
account of which hardware queue each dispatch landed on.

Two properties of the collection method matter:

1. **One shape per process.** The probe takes `--only=chain|independent|fanout`,
   so each trace contains exactly one shape. Nothing has to be split back into
   phases afterwards. This matters because 7.14's dispatch records carry no
   `graph_exec_id`, so any within-trace phase attribution would have been a
   fixed-block heuristic; running one shape per process removes that assumption
   entirely.
2. **Runtime provenance is printed by the probe itself**, from `dladdr` on a HIP
   entry point plus `hipRuntimeGetVersion`, so every arm proves which runtime
   produced it (71460850 for 7.14, 71526333 for 10.0).

Analysis: `bench/dispatch/analyze_dispatch_queues.py`. N=64 nodes, 3 replays,
1 warmup, so 256 dispatches per shape (4 launches x 64 nodes) — a count the
profiler independently confirms.

## Result: identical scheduling behaviour on all four architectures

| arch | ROCm | shape | queues | distribution |
| --- | --- | --- | ---: | --- |
| gfx1100 | 7.14 | chain | 2 | q2:254, q1:2 |
| | | independent | **4** | q1:66, q4:64, q2:63, q3:63 |
| | | fanout | **4** | q4:68, q1:66, q3:63, q2:59 |
| | 10.0 | chain | 1 | q2:256 |
| | | independent | **1** | q2:256 |
| | | fanout | **1** | q2:256 |
| gfx1151 | 7.14 | chain | 2 | q2:254, q1:2 |
| | | independent | **4** | q1:65, q2:64, q4:64, q3:63 |
| | | fanout | **4** | q4:67, q1:65, q3:64, q2:60 |
| | 10.0 | all three | **1** | q2:256 |
| gfx1030 | 7.14 | chain | 2 | q2:254, q1:2 |
| | | independent | **4** | q1:65, q2:64, q4:64, q3:63 |
| | | fanout | **4** | q4:67, q1:66, q3:64, q2:59 |
| | 10.0 | all three | **1** | q2:256 |
| gfx1201 | 7.14 | chain | 2 | q2:254, q1:2 |
| | | independent | **4** | q1:65, q2:64, q3:64, q4:63 |
| | | fanout | **4** | q4:67, q1:65, q3:64, q2:60 |
| | 10.0 | all three | **1** | q2:256 |

**7.14 spreads concurrent graph shapes evenly across the four-queue hardware
pool. 10.0 places every dispatch on a single queue regardless of the declared
dependency structure.** The behaviour is the same on RDNA2, RDNA3, RDNA3.5 and
RDNA4; only its performance consequence differs by architecture.

## How that ties to the timing

From the matched-pair timing at N=512, independent vs chain on 7.14:

| arch | chain | independent | effect of spreading |
| --- | ---: | ---: | --- |
| gfx1030 | 4.452 | 1.875 | 2.37x cheaper |
| gfx1100 | 2.791 | 1.031 | 2.71x cheaper |
| gfx1151 | 1.741 | 0.805 | 2.16x cheaper |
| gfx1201 | 2.168 | 6.458 | **2.98x more expensive** |

So the same scheduling decision — spread across four queues — is a large win on
RDNA2/3/3.5 and a large loss on RDNA4. 10.0 stops making that decision, which
costs the older parts their advantage and removes RDNA4's penalty. On gfx1201 at
N=8 the effect is larger still: independent 18.723 -> 5.113.

This also accounts for the `GPU_MAX_HW_QUEUES` sensitivity measured on 7.14
(gfx1100, N=512): at the default 4 queues independent is 1.039, at 8 it is
6.789, at 16 it is 6.856. More queues to spread across is not monotonically
better, and past some width the cross-queue cost dominates. 10.0 is insensitive
to the knob at 4, 8 and 16, consistent with it not spreading at all.

## Cross-check between instruments

Under profiling on gfx1201/10.0 the probe's own wall clock reported chain 5.605,
independent 4.737, fanout 4.560 us/dispatch; the profiler's own dispatch
timestamps for the same run gave 5.764, 5.028, 4.874. Two independent clocks
within ~5% of each other. Profiling roughly doubles absolute cost versus the
unprofiled run (chain 2.802 unprofiled) and compresses ratios toward 1, so
profiled runs are used here for **scheduling evidence only**, never for the
headline timing.

## What is NOT established

- Which code path in 10.0 makes the single-queue choice. `GraphExecSegmented`
  and `GraphExecClassic` exist in 10.0's `libamdhip64` and in neither form in
  7.14's, and `hip_graph_internal.cpp` grows 2773 -> 3348 lines on
  `release/therock-10.0` with a `ScheduleNodesIntoBatches` pass bounded by
  `max_streams_`. The observed behaviour is consistent with that pass selecting a
  single stream, but this evidence is queue-distribution, not code-path
  attribution. No claim is made about which branch ran.
- Whether the single-queue choice is intentional. It may be a deliberate
  robustness trade; the 7.14 knob sensitivity is a reasonable motive for making
  one.
- Whether real workloads see the N>=64 effect. Every number here is a no-op
  kernel, so these are submission-cost ratios, not application speedups.
