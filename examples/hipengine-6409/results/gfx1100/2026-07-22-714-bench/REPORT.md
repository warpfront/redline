<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# ROCm 7.14 three-backend bench — pristine hipEngine, gfx1100

Full `run_matrix.py` retained-#6409 matrix through the pristine hipEngine
harness with three backends — direct HIP, Vulkan (RADV/ACO), and Redline
(retained GFX11 PM4, Radiowave-certified HSACO) — on ROCm 7.14
(TheRock `/opt/rocm/core`), AMD Radeon RX 7900 XTX / gfx1100.

This short report is derived strictly from the retained `summary.json` and
`environment.json` in this directory (no live re-run).

- Sampling / matrix: core 112 serial + 112 independent = **224** matched rows
  (dispatch/grid controls are not included).
- Metric: GPU time; ratios are Redline / peer (below 1.0 favors Redline).
- Host capture: `hipx` / `Linux 7.0.0-28-generic`
- Collector path (as recorded): `/tmp/rl714/.engines/hipEngine/benchmarks/micro/collect_env.py`
- HipEngine checkout at capture: `/tmp/rl714/.engines/hipEngine` @ `a187cb0` dirty=`False`

## Result

| Scope | Rows | RL > Vulkan | RL > HIP | RL 1st place |
|---|---:|---:|---:|---:|
| Serial RMW latency | 112 | 70 (62.5%) | 67 (59.8%) | 39 (34.8%) |
| Independent throughput | 112 | 57 (50.9%) | 81 (72.3%) | 52 (46.4%) |
| **All comparable rows** | **224** | **127 (56.7%)** | **148 (66.1%)** | **91 (40.6%)** |

## Per-family (RL > Vulkan / RL > HIP / RL 1st / rows)

| Family | Serial | Independent |
|---|---|---|
| geometry | 5/4/2 · 8 | 3/8/3 · 8 |
| reduction | 13/7/0 · 24 | 13/24/13 · 24 |
| two-stage-reduction | 14/12/10 · 16 | 0/1/0 · 16 |
| sampler | 12/8/8 · 12 | 12/12/12 · 12 |
| dense-q8 | 16/19/16 · 20 | 17/19/16 · 20 |
| memory-waitcnt | 0/2/0 · 8 | 2/8/2 · 8 |
| q4-selected-dual | 4/3/3 · 5 | 4/2/1 · 5 |
| q6-x8-selected-down | 2/0/0 · 3 | 2/2/1 · 3 |
| packed-dot | 0/8/0 · 8 | 0/0/0 · 8 |
| vopd | 4/4/0 · 8 | 4/5/4 · 8 |

Overall RL > Vulkan **127/224 (56.7%)**;
RL > HIP **148/224 (66.1%)**;
RL first place **91/224 (40.6%)**.
Pairwise medians (RL/peer):
- vs Vulkan median ratio `0.8300` (min `0.2040`, max `3.7470`)
- vs HIP median ratio `0.9186` (min `0.1378`, max `3.7836`)

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
  --gfx-arch gfx1100 --reps 20 --warmup 3 --samples 7 --out-dir <out>
python3 examples/hipengine-6409/summarize_results.py <out> --out <out>/summary.json
```
