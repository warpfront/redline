<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# Graph dependency shape: ROCm 7.14 vs 10.0

Probe: `bench/dispatch/graph_dependency_fencing.cpp`. Nodes added with
`hipGraphAddKernelNode` so edges are exactly as stated; same kernel and node
count in every arm, only the DAG differs. Matched pairs throughout — each
release is built AND run with its own toolchain, verified by
`bench/dispatch/rocm_ident.cpp` (16/16 mapped ROCm objects from the intended
tree, `hipRuntimeGetVersion` 71460850 vs 71526333).

Host `hipx`, Ubuntu 26.04, kernel 7.0.0-30. µs per dispatch, median.

## N sweep, gfx1100, 60 replays

| N | chain 7.14 | chain 10.0 | indep 7.14 | indep 10.0 | fanout 7.14 | fanout 10.0 |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 8 | 9.627 | 9.579 | 10.263 | 8.427 | 11.019 | 6.081 |
| 64 | 3.245 | 3.229 | 1.893 | 3.131 | 2.199 | 3.149 |
| 256 | 2.854 | 2.853 | 1.117 | 2.848 | 1.638 | 2.853 |
| 512 | 2.800 | 2.791 | 1.034 | 2.784 | 1.549 | 2.782 |

## Three architectures, N=512, 60 replays

| GPU | ROCm | chain | indep | fanout |
| --- | --- | ---: | ---: | ---: |
| gfx1100 (RX 7900 XTX) | 7.14 | 2.800 | 1.034 | 1.549 |
| | 10.0 | 2.791 | 2.784 | 2.782 |
| gfx1151 (Radeon 8060S) | 7.14 | 1.741 | 0.817 | 0.921 |
| | 10.0 | 1.742 | 1.744 | 1.744 |
| gfx1030 (RX 6950 XT) | 7.14 | 4.458 | 1.870 | 3.147 |
| | 10.0 | 4.457 | 4.451 | 4.451 |

## Sensitivity to GPU_MAX_HW_QUEUES, gfx1100, N=512

| ROCm | queues | chain | indep | fanout |
| --- | ---: | ---: | ---: | ---: |
| 7.14 | 4 (default) | 2.800 | 1.039 | 1.564 |
| 7.14 | 8 | 2.799 | 6.789 | 1.530 |
| 7.14 | 16 | 2.791 | 6.856 | 8.847 |
| 10.0 | 4 | 2.791 | 2.786 | 2.783 |
| 10.0 | 8 | 2.792 | 2.786 | 2.784 |
| 10.0 | 16 | 2.791 | 2.786 | 2.783 |

The 7.14 queue sensitivity reproduces (3 runs each, gfx1100, N=512):
Q=4 indep 1.016 / 1.026 / 1.038; Q=8 indep 6.864 / 6.877 / 6.678. Chain is
2.790-2.803 throughout, so the effect is specific to concurrent shapes.

## Reading

1. **Serial chains are unchanged.** Every chain figure matches between releases
   to three digits, consistent with the separately measured per-dispatch floor
   being unchanged across 7.2 -> 7.14 -> 10.0.
2. **7.14 has a large but fragile concurrent-graph fast path.** At the default
   queue count, declaring nodes independent is 2.1x-2.7x cheaper than a chain on
   all three architectures. Raising `GPU_MAX_HW_QUEUES` destroys it (1.04 ->
   6.8 at Q=8, and fanout 1.56 -> 8.85 at Q=16).
3. **10.0 removes both the best and the worst case.** Independent and fanout
   converge on the chain cost, within 0.3%, and become insensitive to
   `GPU_MAX_HW_QUEUES`. That is a peak-for-predictability trade, not a simple
   regression: worse than 7.14's best case by ~2.7x, better than 7.14's
   misconfigured case by ~2.4x.
4. **10.0 is better for small graphs.** At N=8 fanout is 11.019 -> 6.081 and
   independent 10.263 -> 8.427.

10.0 ships a new graph-exec implementation absent from 7.14: `GraphExecSegmented`
and `GraphExecClassic` appear in 10.0's `libamdhip64` and in neither form in
7.14's, and `hip_graph_internal.cpp` grows 2773 -> 3348 lines on
`release/therock-10.0` with a segmentation pass (`ScheduleNodesIntoBatches`,
segments derived from execution paths, dependency levels, a `max_streams_`
bound). The behaviour above is consistent with that path being taken and
serialising, but this probe does not inspect emitted packets and does not
establish which code path ran.

## Open, and NOT claimed here

An earlier single measurement on gfx1201 (ROCm 7.14, N=512) recorded chain
2.163, independent 6.432, fanout 7.055 — independent *more* expensive, the
opposite of the three architectures above. That figure is unverified: it was
taken before the queue-sensitivity effect was known, and it resembles 7.14's
Q>=8 behaviour rather than its Q=4 behaviour. gfx1201 must be re-measured with
matched pairs and an explicit queue count before any gfx1201 claim is made. The
host carrying those GPUs was down when this was written.
