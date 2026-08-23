<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# The HIP dispatch floor on ROCm 7.14, and the reproducer AMD asked for

Context: on the AMD Discord thread following
[ROCm/ROCm#6409](https://github.com/ROCm/ROCm/issues/6409), a llama.cpp
maintainer reported that AQL dispatch costs several times what raw PM4
submission costs, and that dispatch overhead dominates decode. AMD said they
would check whether the team had managed to reproduce it. The thread has had no
technical answer since 2026-08-19 despite two follow-up pings.

The blocker for AMD is a reproducer that needs none of our code. This document
provides one, plus current-release numbers, plus the specific primitive that
would let HIP consumers fix this themselves.

**Everything below is ROCm 7.14.0, Ubuntu 26.04, measured 2026-08-23.**

## 1. The reproducer

[`bench/dispatch/aql_dispatch_floor.cpp`](../../bench/dispatch/aql_dispatch_floor.cpp)
— one file, no dependencies beyond HIP, no PM4, no Vulkan, **no profiler**.

```bash
hipcc --offload-arch=$(rocminfo | awk '/gfx/{print $2; exit}') -O3 \
    aql_dispatch_floor.cpp -o aql_dispatch_floor
./aql_dispatch_floor
```

The kernel is one workitem doing one `atomicAdd`, so the result is submission
and completion cost rather than compute. Three arms: `N` launches on one stream
with a single sync; `N` launches each with its own sync; and the same `N`
launches captured once into a `hipGraph` and replayed.

Two design choices are there because of specific complaints in that thread:

- **No profiler.** The thread notes `rocprofv3` does not report dispatch times
  usefully. Timing is a host `steady_clock` span and a `hipEvent` span over the
  *same* batch, reported side by side. If the two clocks disagree by more than
  25% the run prints `DIVERGE`, so a measurement artifact is visible rather than
  silently published. Every run below printed `agree`.
- **Correctness-gated.** The atomic counter must equal exactly
  `N × replays` or the row prints `COUNTER MISMATCH` and the timing is declared
  invalid. An arm cannot look fast by skipping the work.

## 2. What HIP charges per dispatch

Median of 200 timed replays after 20 warmups, host clock, µs per dispatch:

| GPU | arch | per-launch-sync | stream-loop | graph-replay | graph vs loop |
| --- | --- | ---: | ---: | ---: | ---: |
| Radeon AI PRO R9700 | gfx1201 | 18.477 | 2.530 | **2.149** | 1.18× |
| RX 7900 XTX | gfx1100 | 19.447 | 3.152 | **2.826** | 1.12× |
| Radeon 8060S (Strix Halo) | gfx1151 | 7.015 | 1.839 | **1.747** | 1.05× |

The gfx1201 sweep over `N`, showing where the floor settles:

| N | stream-loop | per-launch-sync | graph-replay |
| ---: | ---: | ---: | ---: |
| 1 | 22.880 | 22.659 | 28.733 |
| 8 | 4.987 | 19.124 | 6.743 |
| 64 | 2.824 | 18.648 | 2.528 |
| 256 | 2.600 | 18.529 | 2.200 |
| 512 | 2.530 | 18.477 | 2.149 |

Three things worth drawing out.

**`hipGraph` buys 5–18%, not an order of magnitude.** Capturing the work into a
graph and replaying it is HIP's own answer to dispatch overhead, and on these
parts it removes at most 18% of the per-dispatch cost. At small `N` it is
*slower* than the naive loop, because instantiation and launch setup are not
amortised yet.

**A per-dispatch sync costs ~18.5 µs on discrete parts** and ~7 µs on the APU.
That is the round trip, and it is the shape naive code takes.

**The floor has not moved since ROCm 7.2.** This repository's historical
`DISPATCH-FLOOR.md` recorded real `hipGraphLaunch` at 2.113–2.133 µs/dispatch on
ROCm 7.2 on the same GPU model. The measurement above is 2.149 µs on 7.14.

## 3. What one retained PM4 buffer costs instead

Same host, same GPU, same ROCm build, same no-op kernel, via
`cargo run --release --example dispatch_floor -p redline-dispatch`:

| path | N=256 | N=512 |
| --- | ---: | ---: |
| `SystemEveryDispatch` (models HIP's per-dispatch fence) | 2.3281 | 2.3326 |
| `AgentEveryInternalDispatch` | — | 2.0948 |
| `BoundarySerialized` (AQL, fences only where order requires) | 1.2994 | 1.2876 |
| `BoundaryIndependent` (AQL, independent dispatches only) | 1.0709 | 1.0574 |
| **PM4 retained IB, conservative / decode-safe** | **0.1759** | **0.1436** |
| PM4 retained IB, aggressive / minimal fence | 0.1003 | 0.0695 |

Head-to-head against the independently measured `hipGraph` number from §2:

| | N=256 | N=512 |
| --- | ---: | ---: |
| `hipGraph` replay (standalone HIP reproducer) | 2.200 | 2.149 |
| PM4 retained IB, conservative | 0.1759 | 0.1436 |
| **ratio** | **12.5×** | **15.0×** |

### The cross-check that makes this credible

These two harnesses share no code — one is standalone C++ calling HIP, the other
is Rust driving ROCr directly — and they were written to measure different
things. They agree where they overlap:

- standalone HIP, `hipGraph` replay at N=512: **2.149 µs/dispatch**
- redline's modelled agent-scope fence policy at N=512: **2.0948 µs/dispatch**

That is a 2.6% difference. `hipGraph` uses agent-scope fencing, so the
independently modelled policy lands on the measured behaviour of the real API.
The fence-policy ladder is therefore not a curiosity — it predicts what HIP
actually does, which is what makes the PM4 row meaningful rather than an
apples-to-oranges comparison.

Note the measurement bases differ and the table says so: the AQL rows are a
GPU-timestamped dispatch span, the PM4 rows are end-to-end host latency per
replay. The PM4 rows are the *less* flattering basis for PM4, since they include
host submission cost.

## 4. What this predicts for decode, as arithmetic

This section is arithmetic on the numbers above, not a measurement.

A dense 61-layer decode step issues roughly 7–9 kernels per layer, so order 450–550
dispatches per token. At the measured `hipGraph` floor of 2.149 µs that is
**~1.0–1.2 ms per token of pure submission cost**. A 70 tok/s decode is 14.3 ms
per token, so dispatch overhead is order **7–8% of the step**, and the PM4 floor
would reduce that to about 0.5%.

The independent end-to-end result reported in the thread was 62 → 68.5 tok/s
after routing llama.cpp's dispatches through a retained PM4 path, i.e. **+10.5%**.
That is the same order as this arithmetic predicts, and slightly larger, which is
consistent with the model used there being an MoE (many small expert kernels, so
more dispatches per token than a dense model).

Two independent routes to the same magnitude is the strongest statement in this
document: a floor microbenchmark and a real inference engine agree.

## 5. The primitive that would fix this

The thread's actual request was "a documented way of running low-level dispatches
with HIP". Restated as something implementable, and deliberately smaller than
"expose PM4":

**A documented HIP entry point that submits a pre-built batch of dispatches with
caller-declared fence scope, and a guarantee about what state persists between
them.**

Concretely, the properties that make the PM4 path fast are:

1. **One submission for many dispatches.** N dispatches enter the queue as one
   packet with one doorbell write, instead of N packets.
2. **Fences where the data flow needs them, not per dispatch.** The
   `BoundarySerialized` row above is still AQL — it keeps dependency order and
   only drops redundant fences, and that alone is 1.8× at the floor.
3. **Persistent kernel state across dispatches in a batch.** Consecutive
   dispatches of the same kernel need not re-emit identical SH-register writes.

Any one of the three is useful independently. (2) is the cheapest for AMD to
expose and needs no new submission path — a documented per-launch or per-graph
fence-scope hint would let a consumer opt into agent-scope or boundary-only
fencing where it knows the dependency structure, which is exactly what an
inference engine's graph gives it.

For (3), the specific thing we would need in writing is whether SH-register state
is guaranteed to persist across dispatches within a single submission on a given
architecture, or whether that is an implementation detail we must not rely on.
Today we cannot depend on it because it is undocumented, not because it does not
hold.

## 6. Scope and honesty

- The kernel is a no-op. This is a floor measurement; ratios shrink toward 1× as
  real per-kernel work grows. The 15× is the overhead removed, not a speedup any
  application should expect.
- The PM4 comparison is gfx1201 only. gfx1100 and gfx1151 have HIP-side floors
  measured above, and both are *higher* than gfx1201's, so the gap there is at
  least as large — but that is inference, not measurement. PM4 was deliberately
  not run on gfx1100 here: that combination is the subject of
  [ROCm/ROCm#6529](https://github.com/ROCm/ROCm/issues/6529) intermittent VM
  faults, and the host is shared.
- `n=1` graph replay being slower than a plain launch is a real result, not
  noise, and it is a reminder that graphs are an amortisation strategy.
- Everything in §2 is reproducible by anyone with a ROCm install and no
  third-party code. §3 requires this repository. The cross-check in §3 is the
  only place the two are compared, and it is the claim most worth attacking.

## Reproducing

```bash
# Section 2 -- stock ROCm only
hipcc --offload-arch=gfx1201 -O3 bench/dispatch/aql_dispatch_floor.cpp -o adf
./adf                  # sweep
./adf 512 200 20       # single N

# Section 3 -- this repository
hipcc --genco --offload-arch=gfx1201 bench/floor_kernel.hip -o /tmp/floor_kernel.co
ROCR_VISIBLE_DEVICES=0 REDLINE_FLOOR_HSACO=/tmp/floor_kernel.co \
  REDLINE_FLOOR_N=512 REDLINE_FLOOR_M=300 REDLINE_FLOOR_WARMUP=30 \
  cargo run --release --example dispatch_floor -p redline-dispatch
```
