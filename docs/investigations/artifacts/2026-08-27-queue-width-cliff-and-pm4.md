<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# How many queues are reasonable, and does PM4 still help once you tune them

**Status: internal. NOT posted upstream.**

All figures ROCm 10.0, matched pairs, runtime provenance printed per run,
`Shape::ParallelChains` (N independent serial chains, so the graph has exactly
that many distinct execution paths), N=512 nodes, us per dispatch.

## Hardware capacity, for reference

| arch | max queues | CUs | shader engines |
| --- | ---: | ---: | ---: |
| gfx1030 | 128 | 80 | 4 |
| gfx1100 | 128 | 96 | 6 |
| gfx1151 | 128 | 40 | 2 |
| gfx1201 | 128 | 64 | 4 |

Every part advertises 128 hardware queues. `GPU_MAX_HW_QUEUES` defaults to 4 on
all of them.

## Queues actually used tracks min(chains, GPU_MAX_HW_QUEUES)

gfx1201, from rocprofv3 dispatch records, 2048 dispatches per trace:

| chains | queues | distribution |
| ---: | ---: | --- |
| 1 | 1 | q2:2048 |
| 2 | 2 | q3:1024, q2:1024 |
| 3 | 3 | q3:684, q2:684, q4:680 |
| 4 | 4 | q1:512, q2:512, q4:512, q3:512 |
| 6 | 4 | q1:684, q2:684, q4:340, q3:340 |
| 8 | 4 | 512 each |

Even splits where the counts divide, and correctly capped at the configured
limit. The scheduling itself is working as documented.

## There is a sharp cliff, and its position is per-host, not per-architecture

chains=8, sweeping `GPU_MAX_HW_QUEUES`:

| GPU | Q=1 | Q=2 | Q=3 | Q=4 | Q=5 | Q=6 | Q=7 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| gfx1100 (6 SE) | 2.663 | 2.092 | 1.448 | 0.869 | **0.842** | 5.532 | 5.702 |
| gfx1151 (2 SE) | 1.802 | 1.394 | 1.055 | 0.865 | **0.825** | 1.862 | 1.857 |
| gfx1030 (4 SE) | 4.744 | 3.520 | 2.384 | 1.331 | **1.324** | 6.645 | 6.657 |
| gfx1201 (4 SE) | 2.253 | **1.705** | 6.996 | 6.529 | 6.470 | 6.707 | — |

gfx1201 rows are the median of three runs each; the Q=2 and Q=3 values are tight
(1.705/1.709/1.714 and 6.996/7.016/7.031), so the cliff is real and not noise.

Two things stand out:

1. **Below the cliff, more queues helps monotonically** — up to 3.2x on gfx1100
   (2.663 -> 0.842), 3.6x on gfx1030, 2.2x on gfx1151.
2. **The cliff position does not track shader engines.** All three hipx parts
   cliff between Q=5 and Q=6 despite having 2, 4 and 6 shader engines. gfx1201 on
   the other host cliffs between Q=2 and Q=3. So the threshold is a property of
   the host or the part's queue management, not of hardware width.

Consequence for the default: `GPU_MAX_HW_QUEUES=4` is close to optimal on the
three hipx parts (within 3% of the Q=5 best) and **badly wrong on gfx1201**,
where the default sits past the cliff and costs 3.8x versus Q=2
(6.529 vs 1.705).

## Does PM4 still help once the queues are tuned?

Yes, but by much less than against the default. PM4 conservative (serialized,
decode-safe) versus the *best tuned* hipGraph configuration on each part:

| arch | best tuned hipGraph | at | PM4 conservative | PM4 advantage |
| --- | ---: | --- | ---: | ---: |
| gfx1030 | 1.324 | Q=5, chains=8 | 0.2012 | **6.6x** |
| gfx1100 | 0.842 | Q=5, chains=8 | 0.2256 | **3.7x** |
| gfx1151 | 0.825 | Q=5, chains=8 | 0.2382 | **3.5x** |
| gfx1201 | 1.705 | Q=2, chains=8 | 0.1493 | **11.4x** |

For contrast, against **default-configuration** hipGraph (chain shape, Q=4) the
same PM4 numbers were 22.1x / 12.4x / 7.3x / 14.4x.

So tuning queue width and using a segmentable graph shape recovers a large part
of the gap: on gfx1100 it goes from 12.4x down to 3.7x, on gfx1151 from 7.3x to
3.5x. **PM4 remains ahead on every part, by 3.5x-11.4x**, but the honest headline
is much smaller than the default-configuration comparison implies, and any
published figure should state which configuration it is against.

gfx1201 is the exception in both directions: it has the worst default behaviour
(cliff at Q=3) and the largest remaining PM4 advantage (11.4x).

## What is NOT established

- Why the cliff exists, or why it sits at a different width on the two hosts.
  Candidates not distinguished here: MES queue-management limits, per-host
  firmware configuration, host CPU thread availability for doorbell handling, or
  the number of GPUs present on the host.
- Why gfx1201 cliffs so much earlier. Only one RDNA4 host was available, so
  host-versus-architecture cannot be separated for that part.
- Whether the cliff moves with N, kernel duration, or chain length. All of these
  used N=512 and a no-op kernel.
- Multi-queue **PM4** is still not measured. Every PM4 figure here is
  single-queue; if PM4 can also fan out, the 3.5x-11.4x column is a floor.
