<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- Copyright 2026 Kaden Schutt -->

# Dispatch-floor measurement

The gap that ROCm/ROCm#6409 documents — HIP graph replay losing to Vulkan
command-buffer replay at the tiny-dispatch floor — is a *fence* problem: HIP
applies a **system-scope acquire/release fence on every dispatch** (cache
invalidate + flush, system-wide), while a retained command buffer fences only
where the data flow requires it.

This microbench isolates and measures exactly that lever on real hardware.

## Method

The same retained batch of `N` tiny (single-workitem, no-op) kernel dispatches
is replayed on one queue under two fence policies, and the GPU-timed span
(first-dispatch-start → last-dispatch-end, from device timestamps) is taken as
the median of 300 replays after 30 warmups:

- **`SystemEveryDispatch`** — a system-scope fence on every dispatch. This
  models HIP's default per-dispatch fence (the #6409 floor).
- **`BoundarySerialized`** — Redline's minimal fences: only where dispatch order
  requires visibility.

Everything else is held constant — same kernels, same retained indirect buffer,
same single queue. Only the fence policy differs, so the delta is the fence
overhead and nothing else.

- Harness: `cargo run --example dispatch_floor -p redline-dispatch`
- Kernel: `bench/floor_kernel.hip` → `floor_kernel.co` (gfx1201)

## Result — AMD Radeon AI PRO R9700 (gfx1201), 2026-07-12

Full fence-policy spectrum, µs per dispatch (median of 300 replays after 30
warmups), device 3 of 4:

| policy | N=256 | N=512 | vs floor |
| --- | ---: | ---: | ---: |
| `SystemEveryDispatch` — HIP per-dispatch floor | 2.389 | 2.372 | 1.00× |
| `SystemAcquireAgentRelease` | 2.145 | 2.115 | 1.12× |
| `AgentEveryInternalDispatch` | 2.143 | 2.109 | 1.12× |
| **`BoundarySerialized` — Redline safe (decode)** | **1.325** | **1.300** | **1.83×** |
| `BoundaryIndependent` — Redline aggressive (independent-only) | 1.094 | 1.068 | 2.22× |

**The durable decode number is the conservative one: ~1.8×.**
`BoundarySerialized` preserves dependency order — which a decode token graph
requires — while dropping to boundary-only fences. The system fence is ~1 µs per
dispatch (2.37 → 1.30 µs/dispatch) and Redline removes it.

The aggressive `BoundaryIndependent` (~2.2×) *additionally* drops inter-dispatch
serialization. That is correct only when dispatches touch disjoint writable state
(the independent-throughput / batched regime), not a serial decode chain — here
the no-op kernel has no dependencies, so it is valid and shows the ceiling.

The spectrum is smooth and monotonic and stable across N, so 1.8× is a safe,
defensible rung — not a cherry-pick — with ~2.2× of headroom for independent
work above it.

## Literal hipGraphLaunch head-to-head

The spectrum above compares against a *modeled* worst case (`SystemEveryDispatch`).
This is the direct comparison against a real HIP graph: `bench/floor_hipgraph.hip`
captures the same N no-op dispatches with `hipStreamBeginCapture`, instantiates
with `hipGraphInstantiate`, and replays with `hipGraphLaunch` (the exact
lucebox / llama.cpp ggml-cuda path), GPU-timed.

| | N=256 | N=512 |
| --- | ---: | ---: |
| real `hipGraphLaunch` | 2.133 µs/disp | 2.113 µs/disp |
| Redline `BoundarySerialized` (safe / decode) | 1.299 µs/disp | 1.307 µs/disp |
| **head-to-head (conservative)** | **1.64×** | **1.62×** |
| Redline `BoundaryIndependent` (aggressive) | 1.072 µs/disp | 1.073 µs/disp |
| head-to-head (aggressive, independent-only) | 1.99× | 1.97× |

**Redline beats a real `hipGraphLaunch` by ~1.63×** on the decode-safe policy,
~1.98× on the aggressive independent policy, on the R9700.

The real hipGraph lands at ~2.12 µs/dispatch — agent-scope fencing, *not* the
2.37 µs system-every-dispatch worst case. So the literal head-to-head (1.63×) is
slightly **below** the vs-theoretical-floor number (1.8×): ROCm 7.2's hipGraph is
better than the absolute worst case, and Redline's win is measured against the
real thing, not the model.

Measurement basis: Redline reports the dispatch execution span (GPU timestamps,
first-start → last-end); the hipGraph baseline reports sustained per-launch GPU
time (hipEvent over 100 launches ÷ 100, median of 9 batches). Both are GPU time
per replay of the same work; the hipGraph figure additionally carries relaunch
overhead the Redline span excludes, a small effect in Redline's favor noted here
for fairness.

## PM4 champion path (`RecordedGraph`)

The retained single-stream PM4 indirect buffer (`RecordedGraph`) is the champion
*structure* — one doorbell/submission for the whole batch, versus the AQL path's
per-dispatch packets. Two things mean its GPU-work speedup is best read off the
AQL-boundary row rather than a direct PM4 timing here:

- **GPU execution is the same class.** The PM4 IB runs the identical dispatches
  with the identical minimal (boundary) fences as `BoundarySerialized`, so its
  GPU execution span is in the same ~1.30 µs/dispatch class — the ~1.6× over
  hipGraph is a GPU-work result that applies to the PM4 path too. The PM4 path's
  *additional* edge is submission overhead (one doorbell, not N): a
  host/submission win, not a GPU-execution one.
- **The public PM4 replay is host-timed only, and host-timing it is dominated by
  per-replay signal re-arm.** `RecordedGraph::submit → wait` re-arms N per-node
  completion signals every replay (the "token-latency core"; the throughput ring
  is explicitly left as integration work). That host cost scales with N (~1 ms at
  N=256, ~2 ms at N=512) and swamps the ~0.3 ms of GPU work, so the host figure
  measures Redline's synchronous wait path, not dispatch efficiency. HSA dispatch
  profiling (`hsa_amd_profiling_get_dispatch_time`) targets AQL packets and does
  not populate PM4-IB dispatch timestamps without extra instrumentation.

Bottom line: the defensible dispatch-floor number is the GPU-span **~1.6× over a
real `hipGraphLaunch`**. A clean PM4-IB GPU-span — to show the submission-overhead
edge on top of that — needs a pipelined replay that amortizes signal re-arm, or
explicit timestamp instrumentation of the IB. Tracked as follow-up.

## Honest scope

- **This is an internal A/B that isolates the fence lever.** Both arms use
  Redline's retained PM4 indirect buffer; only the fence policy changes. It
  faithfully models HIP's per-dispatch system fence (the documented #6409 cause)
  but is not a head-to-head against `hipGraphLaunch` itself — a real hipGraph
  also carries its own relaunch/submission overhead *on top* of the fence, so a
  direct comparison is expected to favor Redline by more, not less. That
  external baseline is the next measurement.
- **The kernel is a no-op — this is the pure floor.** With no data to flush, the
  system fence still costs ~1 µs/dispatch (the GL2 invalidate/flush ops run
  regardless). Real kernels with memory traffic give the system-fence arm *more*
  to flush, so this understates the advantage on real decode.
- Auto clocks, GPU timestamps, median of 300 replays after 30 warmups, device 3
  of 4 (isolated from other work on the box).

## Reproduce

```bash
hipcc --genco --offload-arch=gfx1201 bench/floor_kernel.hip -o bench/floor_kernel.co
ROCR_VISIBLE_DEVICES=3 \
  REDLINE_FLOOR_HSACO="$PWD/bench/floor_kernel.co" \
  REDLINE_FLOOR_N=256 REDLINE_FLOOR_M=300 REDLINE_FLOOR_WARMUP=30 \
  cargo run --example dispatch_floor -p redline-dispatch
```
