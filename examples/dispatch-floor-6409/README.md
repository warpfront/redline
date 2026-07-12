<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# Beating the HIP graph dispatch floor (ROCm/ROCm#6409)

[ROCm#6409](https://github.com/ROCm/ROCm/issues/6409) reports that HIP graph
replay loses to a pre-recorded Vulkan command buffer at the tiny-dispatch floor.
This runs the **exact hipEngine microbenchmark** — the same kernel
(`gmb_noop_kernel`, `out[idx] += 1.0f`, block=256), the same `serial_latency`
dependency chain on a shared buffer, at the same counts (1, 50, 200, 941) — and
adds a Redline arm that lowers those dispatches to **one retained GFX12 PM4
indirect buffer**. Both arms verify correctness (every element == count).

## Result — AMD Radeon AI PRO R9700 (gfx1201), device 3

Host µs/dispatch (median of 50 replays), matched to hipEngine's `host_wall`
domain. Three HIP submission paths are shown: `hip_serial` (plain host-enqueued
launches), `hip_graph` (captured graph replay), and `redline` (one retained PM4
IB):

| count | hip_serial | hip_graph | redline | vs serial | vs graph | correct |
| ---: | ---: | ---: | ---: | ---: | ---: | :---: |
| 1 | 26.02 | 33.34 | 17.56 | **1.48×** | **1.90×** | PASS |
| 50 | 3.19 | 2.99 | 2.08 | **1.53×** | **1.44×** | PASS |
| 200 | 2.81 | 2.55 | 1.96 | **1.43×** | **1.30×** | PASS |
| 941 | 2.74 | 2.26 | 1.90 | **1.44×** | **1.19×** | PASS |

Redline replays the same dependency chain faster than both plain HIP launches and
a real `hipGraphLaunch`, correctness-gated, on the exact benchmark #6409 measures.

**The submission overhead is deterministic and bakeable.** `hip_serial − hip_graph`
≈ 0.48 µs/dispatch at count=941 is the per-dispatch host-submission cost the graph
amortizes. Redline pays submission *once* (the whole sequence is a single retained
PM4 IB) and fences each boundary with the minimal gfx12 dependency fence
(`wait_compute_idle` + inter-node acquire — L2/MALL stays coherent) instead of
HIP's heavier per-dispatch fence, so it beats even the graph.

## Why 1.2× here and not the ~10× floor (it is NOT the binding)

`crates/redline-dispatch/examples/gmb_floor.rs` runs the *same* experiment in
**pure Rust** (no Python, no C — `SingleQueuePm4Ib` driven directly). At count=941
on the R9700:

Three fence modes at the dependency boundary (count=941):

| mode | boundary fence | pure Rust µs/disp | correct |
| --- | --- | ---: | :---: |
| conservative | `wait_compute_idle` + inter-node acquire | ~1.9 | PASS |
| tuned | inter-node acquire only (no wait) | ~0.32 | FAIL (races) |
| aggressive | none | ~0.11 | FAIL (races) |

Three conclusions:

- **The binding is not the bottleneck.** Pure-Rust conservative ≈ the PyO3 arm
  (1.86) — ~4% wrapper overhead. Python did not swamp the win.
- **`wait_compute_idle` is irreducible for a non-atomic RMW chain.** Decomposed,
  the acquire costs ~0.15 µs/disp and the wait ~1.67. Dropping the wait (the
  *tuned* row) invalidates caches but does not stop wave overlap, so the shared
  buffer races (FAIL). So conservative is the **minimal correct** fence — there is
  no correctness-preserving speed left on the table for a strict chain.
- **The ~10–20× floor is fence-free** (the aggressive row, ~0.11 µs/disp ≈ 20× over
  hip_graph) and applies to *independent* dispatches (disjoint outputs / no-op —
  see [`../../docs/DISPATCH-FLOOR.md`](../../docs/DISPATCH-FLOOR.md)). A true
  dependency chain is serialization-bound; there Redline still beats HIP because
  its minimal-*scope* fence is cheaper than HIP's system-scope fence (1.2×). The
  big win comes from workload structure (overlapping the independent parts of a
  decode graph), not from tuning the fence of a strict chain.

Real decode is a mix — cheaper fences on the serial parts, overlap on the
independent parts, and fewer submissions.

`gmb_noop_kernel` also reads `blockIdx`/`blockDim`/`threadIdx` (272-byte kernarg
with hidden args) and Redline dispatches it correctly, validating the binding for
real engine kernels. The `vulkan_command_buffer` arm (#6409's other strategy) is
the next comparison.

## Reproduce

```bash
pip install redline-dispatch          # the PyO3 wheel (Rust core, dlopens ROCr)
# gmb_noop.hip and hipgraph_baseline are auto-compiled with hipcc on first run.
ROCR_VISIBLE_DEVICES=3 python demo.py
```

Environment knobs: `REDLINE_GFX_ARCH` (default `gfx1201`), `REDLINE_6409_COUNTS`,
`REDLINE_6409_N`, `REDLINE_6409_REPS`. Select the GPU with `ROCR_VISIBLE_DEVICES`
(it filters HIP too, so both arms target the same device).

## Files

- `gmb_noop.hip` — the exact #6409 narrow kernel (for Redline's code object).
- `hipgraph_baseline.hip` — faithful `hipStreamBeginCapture → hipGraphInstantiate
  → hipGraphLaunch` baseline, hipEvent + host timed, correctness == count.
- `demo.py` — runs both arms and prints the comparison. The `run_redline`
  function is the drop-in pattern for a Python engine: `Gpu(0)` →
  `load_module(code)` → `build(module, dispatches)` → `replay()`.
