<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# Multi-queue PM4: redline takes the queue-width win too

**Status: internal. NOT posted upstream.**

Every earlier PM4 figure was single-queue, while hipGraph on ROCm 10.0 gains
2.2x-3.6x by spreading a parallel-path graph across hardware queues. Comparing a
single-queue PM4 path against a tuned multi-queue hipGraph understates PM4 by
whatever the queue width is worth, so this measures PM4 with the same structural
advantage: `lanes` retained IBs on independent queues, one chain per lane, which
is the 1:1 shape the graph side peaks at.

Added `measure_pm4_multiqueue_host` using `MultiQueuePm4Ib`, which takes a slice
of command buffers (one per lane) and has gfx10 / gfx11 / gfx12 constructors. The
lane count is swept via `REDLINE_FLOOR_LANES` rather than fixed, because the
useful width is device-specific and cannot be derived from published device
properties. The reported queue count is what the runtime actually granted, not
what was requested.

ROCm 10.0, N=512, conservative (dependency-ordered within each lane), us per
dispatch.

## Lane sweep

| lanes | gfx1030 | gfx1100 | gfx1151 | gfx1201 |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 0.2009 | 0.2239 | 0.2379 | 0.1472 |
| 2 | 0.1325 | 0.1331 | 0.1482 | **0.0908** |
| 3 | — | — | — | 0.1370 |
| 4 | **0.0952** | **0.1005** | **0.0786** | 0.1212 |
| 5 | 0.1537 | 0.1568 | 0.1024 | — |
| 6 | 0.1462 | 0.1622 | 0.0919 | — |
| 8 | — | — | — | 11.7072 |

Multi-queue PM4 improves on single-queue PM4 by **2.1x on gfx1030, 2.2x on
gfx1100, 3.0x on gfx1151 and 1.6x on gfx1201**.

The optimal width broadly matches the width the graph side wanted — 4 on gfx1030,
2 on gfx1201 — which is consistent with both paths hitting the same hardware or
queue-management constraint rather than a software one. gfx1151 is the exception:
its graph optimum was 5, its PM4 optimum is 4.

The gfx1201 8-lane figure (11.7072) is the same class of collapse the graph side
shows past its cliff, and is 129x worse than that part's 2-lane optimum. Width
must be measured per part in either path; overshooting is not a small penalty.

## The comparison that matters

PM4 at its best lane count versus hipGraph at its best C:Q, both on ROCm 10.0:

| arch | tuned hipGraph | single-queue PM4 | advantage | **multi-queue PM4** | **advantage** |
| --- | ---: | ---: | ---: | ---: | ---: |
| gfx1030 | 1.286 | 0.2009 | 6.4x | **0.0952** | **13.5x** |
| gfx1100 | 0.831 | 0.2237 | 3.7x | **0.1005** | **8.3x** |
| gfx1151 | 0.826 | 0.2379 | 3.5x | **0.0786** | **10.5x** |
| gfx1201 | 1.190 | 0.1472 | 8.0x | **0.0908** | **13.1x** |

**Multi-queue roughly doubles to triples the PM4 advantage over a fully tuned
hipGraph**, and it restores the gap that queue tuning had closed:

| arch | vs default hipGraph | vs tuned, single-queue PM4 | vs tuned, multi-queue PM4 |
| --- | ---: | ---: | ---: |
| gfx1030 | 22.1x | 6.4x | **13.5x** |
| gfx1100 | 12.4x | 3.7x | **8.3x** |
| gfx1151 | 7.3x | 3.5x | **10.5x** |
| gfx1201 | 14.4x | 8.0x | **13.1x** |

The honest headline is the third column: **8.3x-13.5x against a hipGraph that has
been given every advantage** — segmentable graph shape, tuned queue width, and
the same 1:1 chain-per-queue structure PM4 is using. gfx1151 ends up *better* than
its default-configuration figure (10.5x vs 7.3x) because its PM4 side gains more
from width (3.0x) than its graph side does.

## Why this matters for the interposer

`redline-hipgraph` exports the hipGraph ABI, so it sees the DAG before CLR does.
That means the lane structure measured here is derivable from information the
interposer already has: the number of independent execution paths in the graph.
A consumer that expresses concurrency as graph node independence would get
hipGraph's tuned number; the same consumer, unmodified, routed through the
interposer could get the multi-queue PM4 number instead.

What is required for that, and is not yet built: mapping graph segments onto PM4
lanes, and choosing the lane count per device rather than a constant. The second
is not optional — a fixed lane count of 8 would cost gfx1201 129x versus its
2-lane optimum.

## What is NOT established

- No correctness gate was captured for the multi-queue arm. `verify_pm4_execution`
  covers the single-queue path only, so these figures are timing without an
  execution check and must not be published until that is added.
- The no-op kernel means these are submission-cost ratios throughout.
- Whether segment-to-lane mapping in the interposer preserves the dependency
  semantics a real graph requires. Lanes here are independent by construction;
  a real graph's segments may not be.
- Why gfx1201's optimum is 2 while the other three want 4, and whether that is
  the part or the host it sits in.
