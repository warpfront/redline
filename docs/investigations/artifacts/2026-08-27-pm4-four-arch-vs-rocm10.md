<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# PM4 on all four RDNA architectures, against ROCm 7.14 and 10.0

**Status: internal. NOT posted upstream.**

## The instrumentation gap this closes

The `dispatch_floor` harness reported PM4 numbers on gfx1201 only. On every other
part the whole PM4 row vanished behind
`ArchitectureMismatch { required: "gfx12", actual: "gfx1100" }`, which reads as
"PM4 is unavailable on this architecture" when in fact the encoders existed and
the harness simply never selected them. Two hard-coded sites were responsible:

- `Gfx12Pm4CommandBuffer::new_stateful()` and `SingleQueuePm4Ib::create` in both
  `measure_pm4_ib_host` and `verify_pm4_execution`
- a top-level `if !device.name().starts_with("gfx12")` guard in `main`

`redline-rocr` already exported `Gfx10Pm4CommandBuffer` and
`Gfx11Pm4CommandBuffer` (an alias of the gfx10 register map), and
`SingleQueuePm4Ib` already had `create_gfx10` / `create_gfx11` beside `create`.
The fix adds a `FloorFamily` resolver and a `FloorPm4` enum over the two buffer
encodings, so the family is decided once from the agent name and both the buffer
type and the constructor follow from it. Each constructor still re-checks the
device family, so a mismatch is caught rather than submitted.

The gfx12 arm matches `gfx120`, not `gfx12`, so gfx125x is refused rather than
sent RDNA4-derived PM4.

## PM4, four architectures, both runtimes

us per dispatch, N=512, M=50 timed replays, 10 warmup. ROCr selected per arm and
verified with `LD_DEBUG=libs`.

| arch | PM4 family | conservative 7.14 | conservative 10.0 | aggressive 7.14 | aggressive 10.0 |
| --- | --- | ---: | ---: | ---: | ---: |
| gfx1030 | Gfx10 | 0.2010 | 0.2012 | 0.1107 | 0.1108 |
| gfx1100 | Gfx11 | 0.2265 | 0.2256 | 0.1159 | 0.1150 |
| gfx1151 | Gfx11 | 0.2381 | 0.2382 | 0.1165 | 0.1164 |
| gfx1201 | Gfx12 | 0.1471 | 0.1493 | 0.0713 | 0.0713 |

**PM4 is unaffected by ROCm 10.0 on every architecture**, which is expected:
redline creates its own queue via `hsa_queue_create` and never enters CLR's graph
or stream scheduling.

gfx1201 is the fastest PM4 target by a clear margin (0.0713 aggressive vs
0.1107-0.1165 elsewhere), consistent with it being the only part using the gfx12
encoder rather than the legacy gfx10 register map.

## What that does to the comparison

Against HIP `hipGraph` replay measured by `bench/dispatch/aql_dispatch_floor.cpp`
and `graph_dependency_fencing.cpp` on the same hosts.

Under **ROCm 10.0**, where every graph shape converges on chain cost:

| arch | hipGraph 10.0 | PM4 conservative | ratio | PM4 aggressive | ratio |
| --- | ---: | ---: | ---: | ---: | ---: |
| gfx1030 | 4.450 | 0.2012 | **22.1x** | 0.1108 | 40.2x |
| gfx1100 | 2.791 | 0.2256 | **12.4x** | 0.1150 | 24.3x |
| gfx1151 | 1.742 | 0.2382 | **7.3x** | 0.1164 | 15.0x |
| gfx1201 | 2.147 | 0.1493 | **14.4x** | 0.0713 | 30.1x |

Under **ROCm 7.14**, compared against HIP's *best* available shape — which on
RDNA2/3/3.5 was the independent-node graph, not the chain:

| arch | HIP best 7.14 | shape | PM4 conservative | ratio |
| --- | ---: | --- | ---: | ---: |
| gfx1030 | 1.875 | independent | 0.2010 | 9.3x |
| gfx1100 | 1.031 | independent | 0.2265 | 4.6x |
| gfx1151 | 0.805 | independent | 0.2381 | 3.4x |
| gfx1201 | 2.168 | chain | 0.1471 | 14.7x |

**ROCm 10.0 widened the PM4 advantage by 2.2x-2.7x on RDNA2, RDNA3 and RDNA3.5,
and left RDNA4 unchanged:**

| arch | PM4 advantage 7.14 | PM4 advantage 10.0 | change |
| --- | ---: | ---: | ---: |
| gfx1030 | 9.3x | 22.1x | **2.4x wider** |
| gfx1100 | 4.6x | 12.4x | **2.7x wider** |
| gfx1151 | 3.4x | 7.3x | **2.2x wider** |
| gfx1201 | 14.7x | 14.4x | unchanged |

The mechanism is the one established in the companion artifact: 7.14 spread
independent graph nodes across four hardware queues and 10.0 places them all on
one, so HIP lost its best concurrent number on the three older architectures
while PM4 did not move.

gfx1151 remains the narrowest margin at 7.3x, because its hipGraph floor is the
lowest of the four (1.742).

## What is NOT established

- Measurement bases differ and the difference does not favour PM4: hipGraph rows
  are host wall-clock per dispatch over a batch, PM4 rows are end-to-end host
  latency per replay including submission.
- The kernel is a no-op throughout. These are submission-overhead ratios, and
  they shrink toward 1x as real per-kernel work grows.
- Multi-queue PM4 replay is still untested. 10.0 caps hipGraph at one queue while
  explicit streams still fan out to four, so a multi-queue PM4 arm is the obvious
  next measurement and would change these ratios.
- The correctness gate output was not captured in these runs; the counter check
  exists in `verify_pm4_execution` and should be recorded alongside any published
  figure.
