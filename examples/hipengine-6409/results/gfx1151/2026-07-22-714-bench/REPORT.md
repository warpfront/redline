<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# ROCm 7.14 three-backend bench — pristine hipEngine, gfx1151

Full `run_matrix.py` retained-#6409 matrix through the pristine hipEngine
harness with three backends — direct HIP, Vulkan (RADV/ACO), and Redline
(retained GFX11 PM4, Radiowave-certified HSACO) — on ROCm 7.14
(TheRock `/opt/rocm/core`), AMD Radeon 8060S Graphics / gfx1151.

This short report is derived strictly from the retained `summary.json` and
`environment.json` in this directory (no live re-run).

- Sampling / matrix: core 112 serial + 112 independent = **224** matched rows
  (dispatch/grid controls are not included).
- Metric: GPU time; ratios are Redline / peer (below 1.0 favors Redline).
- Host capture: `hipx` / `Linux 7.0.0-28-generic`
- Collector path (as recorded): `/tmp/rl714/.engines/hipEngine/benchmarks/micro/collect_env.py`
- HipEngine checkout at capture: `/tmp/rl714/.engines/hipEngine` @ `9d2ca23` dirty=`False`

## Result

| Scope | Rows | RL > Vulkan | RL > HIP | RL 1st place |
|---|---:|---:|---:|---:|
| Serial RMW latency | 112 | 92 (82.1%) | 108 (96.4%) | 92 (82.1%) |
| Independent throughput | 112 | 72 (64.3%) | 74 (66.1%) | 58 (51.8%) |
| **All comparable rows** | **224** | **164 (73.2%)** | **182 (81.2%)** | **150 (67.0%)** |

## Per-family (RL > Vulkan / RL > HIP / RL 1st / rows)

| Family | Serial | Independent |
|---|---|---|
| geometry | 8/8/8 · 8 | 6/8/6 · 8 |
| reduction | 24/24/24 · 24 | 21/24/21 · 24 |
| two-stage-reduction | 16/16/16 · 16 | 0/0/0 · 16 |
| sampler | 11/12/11 · 12 | 12/12/12 · 12 |
| dense-q8 | 17/20/17 · 20 | 18/11/9 · 20 |
| memory-waitcnt | 3/8/3 · 8 | 1/0/0 · 8 |
| q4-selected-dual | 3/4/3 · 5 | 3/1/0 · 5 |
| q6-x8-selected-down | 2/3/2 · 3 | 3/2/2 · 3 |
| packed-dot | 0/5/0 · 8 | 0/8/0 · 8 |
| vopd | 8/8/8 · 8 | 8/8/8 · 8 |

Overall RL > Vulkan **164/224 (73.2%)**;
RL > HIP **182/224 (81.2%)**;
RL first place **150/224 (67.0%)**.
Pairwise medians (RL/peer):
- vs Vulkan median ratio `0.8068` (min `0.2331`, max `3.6600`)
- vs HIP median ratio `0.9105` (min `0.0700`, max `3.6005`)

Stack recorded in summary:
- **codegen**: Radiowave manifest-bound upstream LLVM HIP code object
- **dependency**: consumer-aware RMW: VMEM-only minimal boundary, fail-closed generic fallback
- **submission**: Redline retained stateful PM4 IB

## Provenance

Raw `environment.json` keeps absolute collector/runner paths and host env from
the capture machine. This report labels them and does not rewrite the JSON.

## Reproduce
```bash
export PATH=/opt/rocm/core/bin:/opt/rocm/core/lib/llvm/bin:$PATH
export ROCM_PATH=/opt/rocm/core HIP_PATH=/opt/rocm/core HIP_CLANG_PATH=/opt/rocm/core/lib/llvm/bin
export ROCR_VISIBLE_DEVICES=0 HIP_VISIBLE_DEVICES=0 REDLINE_REAL_HIPCC=/opt/rocm/core/bin/hipcc
python3 examples/hipengine-6409/run_matrix.py \
  --hipengine-root /path/to/hipEngine \
  --backends hip,vulkan,redline --modes serial_latency,independent_throughput \
  --gfx-arch gfx1151 --reps 20 --warmup 3 --samples 7 --out-dir <out>
python3 examples/hipengine-6409/summarize_results.py <out> --out <out>/summary.json
```
