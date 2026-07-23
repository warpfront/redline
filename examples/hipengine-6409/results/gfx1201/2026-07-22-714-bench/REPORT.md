<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# ROCm 7.14 three-backend bench — pristine hipEngine, gfx1201

Full `run_matrix.py` retained-#6409 matrix run **through the pristine hipEngine
harness** (`.engines/hipEngine`) with three backends — direct HIP, Vulkan
(RADV/ACO), and Redline (retained GFX12 PM4, Radiowave-certified HSACO) — on
ROCm 7.14 (TheRock `/opt/rocm/core`), AMD Radeon RX 9070 XT / gfx1201.

- Sampling: 20 reps, 3 warmups, 7 measured GPU samples per backend/row.
- Metric: GPU time (hipEvent / device timestamp), the suite's primary domain.
- Correctness: **1737/1737 measured rows pass their CPU oracle, 0 failures.**
- Core rows only (112/mode); the 8 dispatch/grid rows per mode run separately
  via `dispatch_matrix.py` and are not included here.

## Result

| Scope | Rows | RL > Vulkan | RL > HIP | RL 1st place |
|---|---:|---:|---:|---:|
| Serial RMW latency | 112 | 97 (86.6%) | 87 (77.7%) | 81 (72.3%) |
| Independent throughput | 112 | 100 (89.3%) | 73 (65.2%) | 70 (62.5%) |
| **All comparable rows** | **224** | **197 (87.9%)** | **160 (71.4%)** | **151 (67.4%)** |

## Per-family (RL > Vulkan / RL > HIP / RL 1st / rows)

| Family | Serial | Independent |
|---|---|---|
| geometry | 8/8/8 · 8 | 8/8/8 · 8 |
| reduction | 24/24/24 · 24 | 24/24/24 · 24 |
| two-stage-reduction | 16/16/16 · 16 | 16/2/2 · 16 |
| sampler | 12/12/12 · 12 | 12/12/12 · 12 |
| dense-q8 | 16/20/16 · 20 | 20/20/20 · 20 |
| memory-waitcnt | 4/4/4 · 8 | 4/3/0 · 8 |
| q4-selected-dual | 3/1/0 · 5 | 5/1/1 · 5 |
| q6-x8-selected-down | 2/2/1 · 3 | 3/3/3 · 3 |
| packed-dot | 0/0/0 · 8 | 0/0/0 · 8 |
| vopd | 0/0/0 · 8 | 0/0/0 · 8 |

Redline wins the transport decisively (RL > Vulkan 87.9%). The remaining Vulkan
losses are the known codegen-bound families — **packed-dot** and **vopd** (0
wins in both modes) — plus part of independent **two-stage-reduction**. Those
are RADV/ACO code-generation targets, not transport losses; every row is the
same selected HSACO for HIP/Redline.

## Reproduce
```bash
export PATH=/opt/rocm/core/bin:/opt/rocm/core/lib/llvm/bin:$PATH
export ROCM_PATH=/opt/rocm/core HIP_PATH=/opt/rocm/core HIP_CLANG_PATH=/opt/rocm/core/lib/llvm/bin
export ROCR_VISIBLE_DEVICES=0 HIP_VISIBLE_DEVICES=0 REDLINE_REAL_HIPCC=/opt/rocm/core/bin/hipcc
python3 examples/hipengine-6409/run_matrix.py \
  --hipengine-root "$(readlink -f .engines/hipEngine)" \
  --backends hip,vulkan,redline --modes serial_latency,independent_throughput \
  --gfx-arch gfx1201 --reps 20 --warmup 3 --samples 7 --out-dir <out>
python3 examples/hipengine-6409/summarize_results.py <out> --out <out>/summary.json
```
The ROCm 7.14 TheRock layout needed two harness path fixes (both committed):
`toolchain/hipcc` resolves `clang-offload-bundler` and `hsaco_manifest.py`
resolves `llvm-readobj` from the core LLVM dir instead of the classic
`/opt/rocm/llvm/bin`.
