<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# Dispatch-floor measurement

The gap that [ROCm/ROCm#6409](https://github.com/ROCm/ROCm/issues/6409)
documents — HIP graph replay losing to Vulkan command-buffer replay at the
tiny-dispatch floor — is largely a *fence and submission* problem: HIP applies
heavy per-dispatch fencing/submission costs, while a retained command buffer
fences only where the data flow requires it and can submit many dispatches as
one stream.

Redline isolates that lever: same kernels, retained indirect buffer, policy
knobs for fence scope and (where applicable) a single-stream PM4 IB path.

## Current product validation

**Do not treat the historical tables below as the current product scorecard.**

Current published validation is the retained **ROCm 7.14** result set:

- Hipfire gfx1201 primary:
  [`examples/hipfire-6409/results/gfx1201/2026-07-22-rocm7.14-retest/`](../examples/hipfire-6409/results/gfx1201/2026-07-22-rocm7.14-retest/REPORT.md)
  — **192/240** Redline firsts (80.0%), correctness-gated.
- Hipfire leverage A/B (secondary, dirty-tree non-regression — not primary):
  [`…/2026-07-22-rocm714-leverage-certification/`](../examples/hipfire-6409/results/gfx1201/2026-07-22-rocm714-leverage-certification/REPORT.md)
  — 194/240 (80.83%).
- HipEngine gfx1201/1151/1100 ROCm 7.14 benches under
  `examples/hipengine-6409/results/*/2026-07-22-714-bench/`.
- Cross-RDNA and multiqueue controls under
  `examples/hipfire-6409/results/2026-07-14-rdna-rocr-native/` and the retained
  `2026-07-14-redline-*` queue runs.

See the [root README](../README.md#current-results-rocm-714) for the full index.

## Historical methodology (ROCm 7.2, R9700 / gfx1201, 2026-07-12)

The measurements in this section are **historical methodology**: they document
how the fence-policy spectrum and PM4 IB floor were first isolated on ROCm 7.2.
They remain useful for interpreting the lever; they are not the ROCm 7.14
product certification.

### Method

The same retained batch of `N` tiny (single-workitem, no-op) kernel dispatches
is replayed on one queue under two fence policies, and the GPU-timed span
(first-dispatch-start → last-dispatch-end, from device timestamps) is taken as
the median of 300 replays after 30 warmups:

- **`SystemEveryDispatch`** — a system-scope fence on every dispatch. Models
  HIP's default per-dispatch fence (the #6409 floor).
- **`BoundarySerialized`** — Redline's minimal fences: only where dispatch order
  requires visibility.

Everything else is held constant — same kernels, same retained indirect buffer,
same single queue. Only the fence policy differs.

- Harness: `cargo run --example dispatch_floor -p redline-dispatch`
- Kernel: `bench/floor_kernel.hip` → `floor_kernel.co` (gfx1201)

### Fence-policy spectrum (historical)

Full fence-policy spectrum, µs per dispatch (median of 300 replays after 30
warmups), device 3 of 4 on AMD Radeon AI PRO R9700 (gfx1201):

| policy | N=256 | N=512 | vs floor |
| --- | ---: | ---: | ---: |
| `SystemEveryDispatch` — HIP per-dispatch floor | 2.389 | 2.372 | 1.00× |
| `SystemAcquireAgentRelease` | 2.145 | 2.115 | 1.12× |
| `AgentEveryInternalDispatch` | 2.143 | 2.109 | 1.12× |
| **`BoundarySerialized` — Redline safe (decode)** | **1.325** | **1.300** | **1.83×** |
| `BoundaryIndependent` — Redline aggressive (independent-only) | 1.094 | 1.068 | 2.22× |

**The durable decode number is the conservative one: ~1.8×.**
`BoundarySerialized` preserves dependency order — which a decode token graph
requires — while dropping to boundary-only fences.

The aggressive `BoundaryIndependent` (~2.2×) *additionally* drops inter-dispatch
serialization. That is correct only when dispatches touch disjoint writable
state, not a serial decode chain.

### Literal hipGraphLaunch head-to-head (historical)

| | N=256 | N=512 |
| --- | ---: | ---: |
| real `hipGraphLaunch` | 2.133 µs/disp | 2.113 µs/disp |
| Redline `BoundarySerialized` (safe / decode) | 1.299 µs/disp | 1.307 µs/disp |
| **head-to-head (conservative)** | **1.64×** | **1.62×** |
| Redline `BoundaryIndependent` (aggressive) | 1.072 µs/disp | 1.073 µs/disp |
| head-to-head (aggressive, independent-only) | 1.99× | 1.97× |

On that ROCm 7.2 host, real hipGraph landed near ~2.12 µs/dispatch (agent-scope
fencing), slightly below the modeled system-every-dispatch worst case.

Measurement basis: Redline reports the dispatch execution span (GPU timestamps,
first-start → last-end); the hipGraph baseline reports sustained per-launch GPU
time (hipEvent over 100 launches ÷ 100, median of 9 batches).

### PM4 champion path (historical)

`SingleQueuePm4Ib` lowers N dispatches into one retained GFX12 PM4 indirect
buffer, submitted with one doorbell. Measured end-to-end **host latency** on the
R9700, correctness-gated with a per-dispatch atomic-increment kernel:

| | N=256 | N=512 |
| --- | ---: | ---: |
| real `hipGraphLaunch` (host) | 2.189 µs/disp | 2.137 µs/disp |
| PM4 IB conservative (serialized, decode-safe) | 0.211 µs/disp | 0.176 µs/disp |
| **head-to-head (conservative)** | **10.4×** | **12.1×** |
| PM4 IB aggressive (minimal fence) | 0.148 µs/disp | 0.124 µs/disp |
| head-to-head (aggressive) | 14.8× | 17.3× |

Correctness gate: counter at exactly N (`256/256`, `512/512`) in both builds.
The ~10× is submission/processing overhead removed relative to N AQL packets;
kernel compute on both paths is the same, so ratios shrink toward 1× as real
work grows.

### Honest scope (methodology)

- Internal A/B isolates the fence lever on Redline's retained IB; the literal
  hipGraph rows above are a separate external baseline from the same era.
- The kernel is a no-op — pure floor. Real kernels with memory traffic can give
  the system-fence arm more to flush.
- Auto clocks, GPU timestamps, median of 300 replays after 30 warmups, isolated
  device ordinal on that host.

## Reproduce the microbench today

```bash
export PATH=/opt/rocm/core/bin:/opt/rocm/core/lib/llvm/bin:$PATH
export ROCM_PATH=/opt/rocm/core HIP_PATH=/opt/rocm/core

hipcc --genco --offload-arch=gfx1201 bench/floor_kernel.hip -o bench/floor_kernel.co
ROCR_VISIBLE_DEVICES=0 \
  REDLINE_FLOOR_HSACO="$PWD/bench/floor_kernel.co" \
  REDLINE_FLOOR_N=256 REDLINE_FLOOR_M=300 REDLINE_FLOOR_WARMUP=30 \
  cargo run --example dispatch_floor -p redline-dispatch
```

For the full matrix product numbers, use the Hipfire / HipEngine harnesses and
compare against the retained ROCm 7.14 reports linked above — not the historical
µs tables in this file.
