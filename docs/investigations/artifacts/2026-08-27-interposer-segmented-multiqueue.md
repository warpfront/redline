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
