<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# The interposer measured: drop-in PM4 is worth 6.8x, segmentation compounds only at the device's width

**Status: internal. NOT posted upstream.**

First end-to-end measurement of `redline-hipgraph` as an actual interposer: an
unmodified consumer calling `hipGraphAddKernelNode` / `hipGraphInstantiate` /
`hipGraphLaunch`, with `libredline_hipgraph.so` in `LD_PRELOAD` and no
application change whatsoever.

Driver is `bench/dispatch/graph_dependency_fencing.cpp --only=parallel-chains`,
which builds `chains` independent serial chains over 512 kernel nodes using
explicit graph nodes. Independent chains are distinct weakly-connected
components, so this is the shape segmentation is designed to split.

gfx1201, ROCm 10.0, 200 replays median, 20 warmups, three repeats each, probe's
own correctness gate `ok` in every cell. us per dispatch.

| chains | stock hipGraph | interposer `off` | interposer `auto` | explicit 2 | explicit 4 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 2 | 1.122 / 1.125 / 1.125 | 0.948 / 0.968 / 0.948 | **0.497 / 0.504 / 0.503** | 0.504 / 0.502 / 0.502 | 0.500 / 0.498 / 0.507 |
| 4 | 6.431 / 6.404 / 6.424 | 0.956 / 0.942 / 0.946 | 0.970 / 0.965 / 0.981 | 0.974 / 0.971 / 0.970 | 0.825 / 1.049 / 0.825 |
| 8 | 6.475 / 6.501 / 6.484 | 0.946 / 0.948 / 0.948 | 0.964 / 0.965 / 0.962 | 0.961 / 0.963 / 0.967 | 1.099 / 0.821 / 0.820 |

`off` is the default and lowers to a single-queue PM4 IB. `auto` resolves the
lane budget from `lanes::measured_lanes`, which is 2 on gfx1201.

## The interposer engages, verified rather than assumed

`REDLINE_HG_DEBUG=1` confirms the path taken rather than leaving it inferred:

```
redline-hg: __hipRegisterFatBinary magic=0x48495046 captured=true bundles=1
redline-hg: bundle magic=clang version=2 entries=2 selected=gfx1201
redline-hg: hipGraphInstantiate build_pm4_replay=ok force_native=false native_exec=true
redline-hg: hipGraphLaunch branch=pm4 replay
```

`branch=pm4 replay` on every launch. This mattered: the first reading of these
timings looked like stock hipGraph cost, and without the debug output the
reasonable conclusion would have been that interposition was silently failing.

## Three results

**1. Drop-in PM4 lowering is worth 1.19x to 6.84x, with no application change.**

| chains | stock | interposer `off` | speedup |
| ---: | ---: | ---: | ---: |
| 2 | 1.125 | 0.948 | 1.19x |
| 4 | 6.424 | 0.946 | **6.79x** |
| 8 | 6.484 | 0.948 | **6.84x** |

**2. The interposer removes gfx1201's queue-width cliff, which is arguably worth
more than the headline.** Stock degrades 5.71x going from 2 chains (1.125) to 4
(6.424) because 4 independent paths exceed that part's usable queue width. The
interposer is flat at ~0.947 across 2, 4 and 8 chains. A consumer that cannot
know the device's cliff still cannot fall off it.

**3. Segmentation compounds, but only where graph parallelism matches the device
width.** At chains=2 on a part whose width is 2, it is a clean 1.88x on top of
single-queue PM4 (0.948 -> 0.503), for **2.24x versus stock**. At chains=4 and 8
it is slightly *worse* than single-queue (0.965-0.970 vs 0.947), because a budget
of 2 packs multiple chains per lane and buys nothing while still paying per-lane
fencing.

Forcing 4 lanes at chains=4 or 8 gives the best single figures (0.820-0.825) but
is **bimodal across repeats** — 0.825/1.049/0.825 and 1.099/0.821/0.820 — which is
consistent with 4 lanes sitting past this part's measured width of 2. It is not a
usable configuration on the strength of this data.

## What this says about the compounding hypothesis

The hypothesis was that an interposer sees the DAG and can therefore take the
per-dispatch PM4 win and the queue-width win simultaneously, compounding them.
That is **confirmed, with a condition**: compounding requires the number of
independent execution paths to match the device's usable queue width. It is 1.88x
when they match and slightly negative when they do not.

So the useful lane budget is `min(independent_paths, device_width)` and the win
is only available when that minimum is the device width — one more reason a
single per-device constant is the wrong policy shape, consistent with the
work-shape reversal recorded in `lanes.rs`.

## What is NOT established

- Only gfx1201 measured. gfx1100 and gfx1151 have wider usable widths (4), so
  they may compound at chain counts where gfx1201 cannot. Not run.
- Only the no-op `tick` kernel. These are submission-cost ratios; a compute-bound
  kernel will dilute all of them.
- Only `parallel-chains`. Real graphs mix shapes, and a decode chain is a single
  component that segmentation correctly refuses to split.
- The `explicit 4` bimodality is unexplained. It could be queue contention, lane
  imbalance, or a scheduling artifact; it was not investigated.
- `redline-hipgraph` does not link on either GPU host with stock binutils:
  `build.rs:45` adds a named version script while rustc emits an anonymous one,
  and `ld.bfd` on Ubuntu 26.04 refuses the combination. It builds only via mold
  (`REDLINE_USE_MOLD=1`, mold 2.40.4 installed for this measurement). This is
  pre-existing — it reproduces at `455ff07` and `588bf22` — and is a real
  portability defect for anyone shipping the interposer.

## Cross-architecture: the picture is not one story, and the default is wrong

Same driver, same method, three repeats, gate `ok` in every cell. Medians, us per
dispatch. `off` is the shipped default (single-queue PM4); `auto` resolves the
lane budget from `lanes::measured_lanes` (gfx1201 => 2, gfx1100 => 4).

| arch | chains | stock | `off` | `auto` | best vs stock |
| --- | ---: | ---: | ---: | ---: | ---: |
| gfx1201 | 2 | 1.125 | 0.948 | **0.503** | 2.24x |
| gfx1201 | 4 | 6.424 | **0.946** | 0.970 | 6.79x |
| gfx1201 | 8 | 6.484 | **0.948** | 0.964 | 6.84x |
| gfx1100 | 2 | 1.434 | 1.202 | **0.626** | 2.29x |
| gfx1100 | 4 | 0.805 | 1.215 | **0.354** | 2.27x |
| gfx1100 | 8 | 0.828 | 1.220 | **0.349** | 2.37x |
| gfx1151 | 2 | 0.904 | 0.907 | 0.903 | none |
| gfx1151 | 4 | 1.005 | 1.005 | 1.006 | none |
| gfx1151 | 8 | 0.811 | 0.818 | 0.811 | none |

### The shipped default is harmful on gfx1100

At chains=4 and 8, single-queue PM4 (`off`, 1.215-1.220) is **1.47-1.51x SLOWER
than stock hipGraph** (0.805-0.828). Stock spreads independent paths across
queues; single-queue PM4 refuses to, and on a part with a usable width of 4 that
loss exceeds everything PM4 saves on submission.

This is not a regression introduced by the segmentation work — the interposer was
always single-queue, so this is what it has always done on gfx1100. Naming `off`
as "preserves existing behaviour" is accurate and simultaneously an
understatement: the existing behaviour was the wrong choice on that part.

Segmentation reverses it completely: `auto` is 2.27-2.37x faster than stock and
**3.44x faster than the current default**. That is the case for making `auto` the
default, and it should be made once gfx1151 works and more shapes are covered.

The two parts want opposite things from the default, but not symmetrically:
`auto` costs gfx1201 1.9% at chains>=4 (0.965 vs 0.947) and gains gfx1100 244%.

### gfx1201 and gfx1100 win by different mechanisms

gfx1201's advantage is PM4 lowering itself: stock collapses to 6.4 us past its
2-wide queue limit, and PM4 is immune at ~0.947 regardless of chain count.
Segmentation adds nothing beyond chains=2 because the budget is 2.

gfx1100's advantage is almost entirely segmentation: stock is already competent
(0.805-0.828) because 4 paths fit its 4-wide width, PM4 alone loses, and only
PM4 *plus* four lanes wins. Same interposer, same flag, opposite reasons.

So "the interposer is worth Nx" is not a well-formed claim. It is worth 6.8x on
gfx1201 by avoiding a cliff, 2.3x on gfx1100 by exploiting width, and 0x on
gfx1151 because of a bug.

### gfx1151: the interposer never engages

All three configurations are identical to stock within noise, because it silently
falls back:

```
redline-hg: bundle magic=clang version=2 entries=2 selected=none
redline-hg: hipGraphInstantiate build_pm4_replay=skipped force_native=true
redline-hg: hipGraphLaunch branch=native_launch(force_native)
```

The fat binary is captured and the bundle parsed, but no entry matches the
gfx1151 agent, so it degrades honestly to native HIP. Correct behaviour, zero
benefit. gfx1201 and gfx1100 report `selected=gfx1201` / `selected=gfx1100` and
`branch=pm4 replay` on the same code path.

Hypothesis, not yet proven: this is the same defect as the `hipfire-6409` suite's
redline backend failing only on gfx1151 with
`hsa_executable_load_agent_code_object ... HSA_STATUS_ERROR_INCOMPATIBLE_ARGUMENTS`
after an arch-matched rebuild. One target-string mismatch, two symptoms.

### The fail-closed guard verified on hardware

With RDNA1/RDNA2 refused at family selection, gfx1030 through the interposer now
reports `build_pm4_replay=skipped force_native=true` and
`branch=native_launch(force_native)`, and the driver's own gate passes (`chains=2
2.248 us, gate ok`). Before the guard the same path lowered to PM4 that executed
nothing. gfx1100 and gfx1151 are unaffected by the guard, and gfx1100 still takes
`branch=pm4 replay`.
