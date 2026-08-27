<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# Explicit stream concurrency, the 4-queue cap, and what ROCm 10.0 did not change

**Status: internal. NOT posted upstream. Held pending review.**

Relevant to two issues an AMD engineer (@Jonathan03ant) filed on 2026-08-27 as
explicit splits of ROCm/ROCm#6409:

- `rocm-systems#10834` — graph replay dispatch overhead vs Vulkan (10.12x on
  gfx1100, 10.75x on gfx1151 single-dispatch; 4.41x / 3.04x per node at 941
  nodes)
- `rocm-systems#10836` — independent stream concurrency vs Vulkan queues
  (Vulkan 20.8x-21.0x faster on gfx1151 small kernels), whose stated hypothesis
  is "HIP streams not actually running concurrently, or significant scheduling
  overhead"

This measures that hypothesis directly. Probe:
`bench/dispatch/explicit_multistream.cpp` — N launches round-robin across M
explicitly created non-blocking streams, no graphs anywhere, spin kernel so
overlap is observable, correctness-gated on the launch count, and printing its
own runtime provenance. Matched pairs: each release built and run with its own
toolchain.

## 1. HIP streams do run concurrently

us per launch, N=256, spin=3000 ticks, median of 20 replays:

| streams | gfx1100 7.14 | gfx1100 10.0 | gfx1151 7.14 | gfx1151 10.0 | gfx1201 7.14 | gfx1201 10.0 |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 32.215 | 32.213 | 32.426 | 32.492 | 32.150 | 32.186 |
| 2 | 16.376 | 16.361 | 16.251 | 16.612 | 16.192 | 16.218 |
| 4 | 8.652 | 8.712 | 9.219 | 9.251 | 11.325 | 11.457 |
| 8 | 8.851 | 8.873 | 9.372 | 9.362 | 11.680 | 11.886 |
| 16 | 9.481 | 9.582 | 10.006 | 9.351 | — | — |

Scaling from 1 to 4 streams is 3.72x on gfx1100, 3.52x on gfx1151, 2.84x on
gfx1201. **Streams demonstrably execute concurrently.** The hypothesis in #10836
that they may not be is not supported on these three architectures.

Two further points relevant to that issue:

- **gfx1151 is not anomalous here.** It scales 3.52x, between gfx1100's 3.72x
  and gfx1201's 2.84x. Whatever produces a 21x Vulkan advantage on gfx1151 in
  that report, it is not a failure of HIP streams to run concurrently.
- **ROCm 10.0 changes nothing for streams.** Every column pair agrees within
  run-to-run noise. This matters because 10.0 *does* change graph scheduling
  substantially (see the companion artifact), so the two issues have different
  causes and should not be expected to move together.

## 2. The plateau is the default queue cap, and raising it helps — up to a cliff

Scaling stops at 4 streams on every architecture, which is exactly
`GPU_MAX_HW_QUEUES`'s default. Sweeping it, ROCm 10.0:

| GPU | GPU_MAX_HW_QUEUES | streams | us/launch |
| --- | ---: | ---: | ---: |
| gfx1100 | 4 (default) | 8 | 9.014 |
| | 8 | 8 | **7.496** |
| | 16 | 8 | **7.224** |
| | 16 | 16 | **27.011** |
| gfx1151 | 4 (default) | 8 | 9.440 |
| | 8 | 8 | **6.360** |
| | 16 | 8 | 7.069 |
| | 16 | 16 | **40.393** |

- The default cap is a genuine limiter: Q=8 with 8 streams is **1.20x** better on
  gfx1100 and **1.48x** better on gfx1151 than the default.
- There is a cliff. Q=16 with 16 streams is **27.011** on gfx1100 and **40.393**
  on gfx1151 — the latter is *worse than a single stream* (32.492), i.e. that
  configuration is actively harmful, not merely unhelpful.

This is a plausible contributor to the Vulkan comparison in #10836: Vulkan
exposes independent hardware queues directly, whereas HIP caps concurrency at 4
by default and degrades sharply if the cap is raised too far. It suggests the
gap may be less "streams do not run concurrently" and more "concurrency width is
capped at 4 by default, with no safe way to widen it".

## 3. Interaction with the graph finding

The companion artifact establishes, from rocprofv3 dispatch records, that 7.14
spreads concurrent *graph* shapes across all four queues while 10.0 places every
graph dispatch on one queue. Combined with the result here:

- The concurrency **capability** is intact in 10.0 — explicit streams still fan
  out and still scale 2.8x-3.7x.
- What 10.0 stopped doing is *hipGraph* asking for it.
- So an engine that expressed concurrency through graph node independence loses
  it on 10.0, while an engine that manages streams explicitly does not. That is
  an application-side workaround for #10834's graph path, and it is worth
  stating because it does not require any runtime change.

## What is NOT established

- No claim about Vulkan. Nothing here measures Vulkan; the comparisons in #10834
  and #10836 are theirs, and this artifact only addresses the HIP side.
- The spin kernel is synthetic. These are submission-and-overlap costs for a
  fixed artificial workload, not application speedups.
- The Q=16/16-stream collapse is reproduced but not explained. It could be
  oversubscription of hardware queues, scheduler thrash, or a cost in queue
  creation; this probe does not distinguish them.
- Nothing here explains the gfx1151 21x figure. It narrows the cause by
  excluding one hypothesis, which is not the same as identifying it.
