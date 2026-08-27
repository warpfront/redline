<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# Multi-queue PM4: the win is real on three parts, and gfx1030 was never real

**Status: internal. NOT posted upstream.**

## Correction notice

The first version of this document (commit `72ff005`) reported multi-queue PM4
figures for four architectures including gfx1030, and claimed a 13.5x advantage
there. **The gfx1030 numbers were invalid and are withdrawn.** They were produced
before the multi-queue path had any correctness gate. Once gated, gfx1030 reports
`counter = 0 / 512` — it executes no dispatches at all — so every gfx1030 PM4
number this project has published internally was measuring the submission of
commands that never ran.

The gate was added specifically because the first version admitted it had none.
It found this within one run. That is the entire argument for gating a
measurement before quoting it.

## What the gate does

`verify_pm4_multiqueue_execution` in `crates/redline-dispatch/examples/dispatch_floor.rs`
rebuilds the same per-lane split the timing path uses, points every lane's
dispatches at ONE shared atomic counter, replays once, and requires the counter
to equal N exactly. A shared counter is the point: it proves the total is N with
no loss and no duplication across lanes. It also asserts host-side that the
per-lane split sums to N, so a split bug is caught even when the GPU path is
fine. A FAIL skips that lane count's timing rather than printing a number.

## gfx1030: PM4 does not execute on RDNA2

Three independent observations, all on ROCm 10.0:

1. **Gated PM4 on gfx1030 executes nothing.** `counter = 0 / 512` at every lane
   count including one lane, and the pre-existing single-queue gate fails
   identically. It is not a multi-queue bug.
2. **The hardware and the kernel are fine.** `bench/dispatch/aql_dispatch_floor.cpp`,
   which uses plain HIP and its own counter gate, passes every arm on the same
   device in the same session (`gate: ok` on stream-loop, per-launch-sync and
   graph-replay at N=1/8/64). So the GPU, `atomicAdd`, and host readback all work.
3. **It is not a missing flush.** The first hypothesis was that the gate read a
   stale counter: `pm4_gfx10_smoke.rs:65-68` emits `wait_compute_idle()` then
   `acquire_system()` before building its IB, with the comment that the AMD
   vendor packet has no architected AQL release scope, and the gate omitted that.
   RDNA3/RDNA4 would hide such an omission where RDNA2 would not. The flush was
   added per lane and **gfx1030 still reports 0 / 512**, which kills the
   hypothesis. gfx1100 and gfx1151 continue to pass with the flush in place, so
   the fence is sound and simply was not the cause.

The same `Gfx10Pm4CommandBuffer` encoder, byte-identical for identical inputs,
works on gfx1100. So the fault is RDNA2-specific in redline's Legacy register
map, not in the PM4 approach. The leading remaining candidate is a register
offset or resource-descriptor mismatch on RDNA2 (`COMPUTE_PGM_RSRC3`,
`COMPUTE_TMPRING_SIZE`); that is unconfirmed and is the next experiment, not a
finding.

There is also no evidence the GFX10 PM4 path was ever validated on gfx1030
hardware: the repo's PM4 validation artifacts are gfx1201-only, and gfx1030
appears only in AQL contexts. **gfx1030 PM4 support was never real**, and the
earlier numbers did not reveal that because nothing checked execution.

The flush fix is kept regardless. A gate that can report zero for work that ran
is worse than no gate, and the smoke test already demonstrated the correct
sequence.

## Gated lane sweeps, three repeats each

ROCm 10.0, N=512, dependency-ordered within each lane, us per dispatch. Every
figure below was taken with the gate reading `512 / 512`.

| lanes | gfx1100 | gfx1151 | gfx1201 |
| ----: | ------: | ------: | ------: |
| 1 | 0.2214 | 0.2377 | 0.1490 |
| 2 | 0.1381 | 0.1481 | **0.0920** |
| 3 | — | — | 0.1523 |
| 4 | **0.1012** | **0.0678** | 0.1324 |
| 5 | 0.1724 | 0.1019 | — |
| 6 | 0.1412 | 0.0917 | — |
| 8 | 0.1409 | 0.0755 | — |

Repeat spread was tight where it mattered: gfx1151 at 4 lanes measured 0.0678 /
0.0679 / 0.0675, gfx1201 at 2 lanes 0.0924 / 0.0914 / 0.0922, gfx1100 at 4 lanes
0.1012 across runs. At lower repetition counts the same points moved by 10-20%,
so single runs are not adequate here — an earlier single run made gfx1151 look
like it preferred 8 lanes (0.0756) purely because that run's 4-lane sample came
in high at 0.0785.

Multi-queue beats single-queue PM4 by **2.19x on gfx1100, 3.51x on gfx1151 and
1.62x on gfx1201**.

Overshoot remains punishing but is not uniform: gfx1100 loses 39% going from 4 to
6 lanes, gfx1151 only loses 35% at 6 and recovers at 8, while gfx1201 at 8 lanes
costs 11.68 us/dispatch — 127x its own 2-lane optimum. Notably that collapse
**passes the gate at 512 / 512**, so it is a genuine slowdown and not lost work.

## Against a fully tuned hipGraph

Each part at its own best lane count versus hipGraph at its own best chains:queues
ratio, both on ROCm 10.0:

| arch | tuned hipGraph | 1-lane PM4 | multi-queue PM4 | advantage |
| --- | ---: | ---: | ---: | ---: |
| gfx1100 | 0.831 | 0.2214 | **0.1012** | **8.2x** |
| gfx1151 | 0.826 | 0.2377 | **0.0678** | **12.2x** |
| gfx1201 | 1.190 | 0.1490 | **0.0920** | **12.9x** |
| gfx1030 | 1.286 | — | — | withdrawn, PM4 does not execute |

So on the three parts where PM4 provably executes, multi-queue roughly doubles
the advantage over a hipGraph that has been given every advantage: segmentable
shape, tuned queue width, and the same 1:1 chain-per-queue structure.

## The load-bearing caveat: which graphs can use this

`crates/redline-dispatch/src/aql/segment.rs` decomposes a captured DAG into lanes
by weakly-connected component, so lanes have no edge between them and need no
cross-lane synchronisation. It returns `Unsplittable` when there is exactly one
component, rather than inventing a split by cutting real edges.

**A transformer decode graph is one long serial chain, therefore one component,
therefore `Unsplittable`.** The multi-queue win is not available for that shape at
all. It applies to graphs that genuinely contain multiple disconnected subgraphs.

That materially narrows where the 8.2x-12.9x figure applies, and it is the honest
framing: the number is real, and the set of workloads that can collect it is
smaller than "inference".

## What is NOT established

- The RDNA2 root cause. Three hypotheses are eliminated; the register-map
  candidate is untested.
- Whether the lane optima hold for real kernels. Every figure here uses an empty
  kernel, so these are submission costs, not occupancy. A kernel that saturates
  CUs may prefer a different width, and nothing here claims otherwise.
- Why gfx1201's optimum is 2 while gfx1100 and gfx1151 want 4. gfx1201 is also
  the only part on a different host, so part and host are not separated.
- Whether the interposer can map segments onto lanes end to end. `segment.rs` is
  the analysis plus its proof; it is deliberately not wired into
  `redline-hipgraph` yet.
