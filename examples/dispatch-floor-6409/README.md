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
domain:

| count | hip_graph (gpu) | hip_graph (host) | redline (host) | speedup | correct |
| ---: | ---: | ---: | ---: | ---: | :---: |
| 1 | 16.36 | 32.57 | 17.28 | **1.88×** | PASS |
| 50 | 2.57 | 2.95 | 2.07 | **1.43×** | PASS |
| 200 | 2.46 | 2.55 | 1.95 | **1.31×** | PASS |
| 941 | 2.24 | 2.26 | 1.86 | **1.22×** | PASS |

Redline replays the same dependency chain faster than a real `hipGraphLaunch`,
correctness-gated, on the exact benchmark #6409 measures. The retained PM4 IB
fences each boundary with the minimal gfx12 dependency fence (`wait_compute_idle`
+ inter-node acquire — L2/MALL stays coherent) instead of HIP's heavier
per-dispatch fence.

## Honest scope

- **This is a conservative measurement.** The Redline arm runs through the
  Python binding (per-replay call overhead the C++ `hipGraphLaunch` baseline does
  not pay), and both arms pay the real kernel's compute — so the ratio understates
  the dispatch-overhead advantage. The *pure* dispatch floor (a no-op kernel,
  measured from native code) is ~10× — see [`../../docs/DISPATCH-FLOOR.md`](../../docs/DISPATCH-FLOOR.md).
- **`gmb_noop_kernel` is a real-engine-kernel test.** It reads
  `blockIdx`/`blockDim`/`threadIdx` (272-byte kernarg segment with hidden args);
  Redline dispatches it correctly through the retained PM4 IB, so this also
  validates the binding for real kernels, not just no-arg ones.
- The `vulkan_command_buffer` arm (#6409's other strategy) is the next comparison.

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
