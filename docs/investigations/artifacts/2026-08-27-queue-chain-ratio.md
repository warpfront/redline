<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# Is there a queue:chain ratio? Full 2-D matrices

**Status: internal. NOT posted upstream.**

ROCm 10.0, `Shape::ParallelChains`, N=512 nodes, us per dispatch, runtime
provenance printed per run. Rows are chain count (distinct execution paths in the
graph), columns are `GPU_MAX_HW_QUEUES`.

## gfx1201 (4 SE, 64 CU) — hiptrx

| C\Q | 1 | 2 | 3 | 4 | 5 | 6 |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | — | 2.204 | 2.196 | 2.202 | 2.241 | 2.204 |
| 2 | 2.223 | **1.192** | 1.191 | 1.186 | 1.186 | 1.181 |
| 3 | 2.239 | 1.556 | 8.785 | 8.305 | 8.834 | 8.480 |
| 4 | 2.253 | 1.770 | 6.984 | 6.426 | 6.453 | 6.416 |
| 6 | 2.287 | 1.601 | 8.502 | 4.660 | 8.704 | 8.685 |
| 8 | 2.315 | 1.795 | 7.028 | 6.485 | 6.514 | 6.493 |
| 16 | 2.398 | 1.881 | 7.076 | 6.536 | 6.572 | 6.562 |

## gfx1100 (6 SE, 96 CU) — hipx

| C\Q | 1 | 2 | 4 | 5 | 6 | 8 |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 2 | 2.590 | 1.618 | 1.479 | 1.495 | 1.495 | 1.486 |
| 4 | 2.612 | 2.117 | **0.841** | 1.198 | 5.774 | 5.416 |
| 5 | 2.618 | 1.901 | 1.173 | 1.182 | 4.466 | 4.522 |
| 8 | 2.662 | 2.107 | 1.085 | 0.871 | 5.623 | 5.552 |
| 16 | 2.753 | 2.240 | 0.983 | 0.972 | 5.120 | 5.645 |

## gfx1151 (2 SE, 40 CU) — hipx

| C\Q | 1 | 2 | 4 | 5 | 6 | 8 |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 2 | 1.756 | 0.907 | 0.912 | 0.912 | 0.910 | 0.909 |
| 4 | 1.775 | 1.354 | 1.022 | 1.010 | 1.846 | 1.845 |
| 5 | 1.777 | 1.100 | 0.828 | **0.826** | 1.520 | 1.517 |
| 8 | 1.804 | 1.382 | 0.823 | 0.829 | 1.853 | 1.874 |
| 16 | 1.844 | 1.454 | 0.917 | 0.897 | 1.925 | 1.902 |

## gfx1030 (4 SE, 80 CU) — hipx

| C\Q | 1 | 2 | 4 | 5 | 6 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 2 | 4.532 | 2.239 | 2.249 | 2.244 | 2.250 |
| 4 | 4.620 | 3.416 | **1.285** | 1.283 | 6.575 |
| 5 | 4.624 | 2.761 | 1.959 | 1.959 | 5.245 |
| 8 | 4.747 | 3.515 | 1.324 | 1.323 | 6.669 |
| 16 | 4.989 | 3.692 | 1.422 | 1.396 | 6.869 |

## Optima, verified over three runs each

| GPU | best C:Q | ratio | us/disp (3 runs) |
| --- | --- | ---: | --- |
| gfx1201 | 2 : 2 | **1:1** | 1.192 / 1.191 / 1.186 |
| gfx1100 | 4 : 4 | **1:1** | 0.841 / 0.827 / 0.825 |
| gfx1151 | 5 : 5 | **1:1** | 0.828 / 0.830 / 0.819 |
| gfx1030 | 4 : 4 | **1:1** | 1.288 / 1.286 / 1.287 |

## The rule, and what it is not

**The ratio is 1:1 — one chain per queue — and the free parameter is not the
ratio but the queue width, which is device-specific.** Every part's best cell has
chains equal to the queue cap:

| GPU | queue width at optimum |
| --- | ---: |
| gfx1201 | 2 |
| gfx1030 | 4 |
| gfx1100 | 4 |
| gfx1151 | 5 |

Three corrections to natural guesses:

1. **1:1 alone is not sufficient** — it must be 1:1 *at the device's usable
   width*. On gfx1201, C=3:Q=3 gives 8.785 and C=4:Q=4 gives 6.426, both 1:1 and
   both terrible, because 3 and 4 are past that part's cliff. 1:1 is necessary,
   not sufficient.
2. **Width does not track shader engines.** gfx1151 has 2 SE and the widest
   optimum (5); gfx1100 has 6 SE and an optimum of 4; gfx1201 has 4 SE and an
   optimum of 2. Any rule derived from SE count is wrong.
3. **Overshooting chains is mild; overshooting queues is severe.** On gfx1100,
   C=16:Q=4 costs 0.983 versus the 0.841 optimum, a 17% penalty. Moving Q from 4
   to 6 at the same chain count costs 5.774 — a 6.9x penalty. Chains beyond the
   queue width degrade gracefully because usage caps at min(C, Q); queues beyond
   the cliff do not degrade at all, they fall off it.

Practical consequence: **set chains to the measured queue optimum, and find that
optimum by sweeping Q, because it cannot be derived from published device
properties.** The default `GPU_MAX_HW_QUEUES=4` happens to be optimal on gfx1030
and gfx1100, one short on gfx1151, and two past the cliff on gfx1201.

## Best tuned hipGraph versus PM4, using these optima

| GPU | best tuned hipGraph | PM4 conservative | PM4 advantage |
| --- | ---: | ---: | ---: |
| gfx1030 | 1.286 | 0.2012 | **6.4x** |
| gfx1100 | 0.831 | 0.2256 | **3.7x** |
| gfx1151 | 0.826 | 0.2382 | **3.5x** |
| gfx1201 | 1.190 | 0.1493 | **8.0x** |

gfx1201's PM4 advantage drops from 11.4x to **8.0x** once its graph side is tuned
to C=2:Q=2 rather than left at the default, which was past its cliff. PM4 still
leads on every part.

## What is NOT established

- Why the usable width differs per part, and why it does not follow SE count.
- Whether the width is a property of the part or of the host. gfx1201 was
  measured on a different machine from the other three, so those two variables
  are not separated for it.
- Whether 1:1 holds at other N, other chain lengths, or with real kernels. All
  cells here are N=512 with a no-op kernel.
- Multi-queue PM4 remains unmeasured, so the PM4 column is still single-queue and
  the advantage figures are a floor.
