<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# HipEngine #6409 integration

Runs the pinned HipEngine micro suite with **HIP**, **Vulkan**, and **Redline**
(Radiowave-certified HSACO + retained PM4) without modifying the HipEngine
checkout.

Adapters:

- `toolchain/hipcc` — compile original HIP through Radiowave; hash-bound manifest
- `redline_timing_override.hpp` — synthetic C++ runners
- `redline_hip_timing.py` — production-shaped Python runners

Fail-closed: stale/missing manifest, absent inspection, unknown consumer, or
scalar mutable read cannot select the narrow VMEM boundary. HipEngine's invalid
independent HIP sampler remains a rejected artifact, not a timing row.

Optional preheat: `REDLINE_PREHEAT_REPLAYS=N` on `redline_hip_timing.py` replays
each measured IB outside the returned GPU sample set (overwrite-style rows only;
RMW must reset mutable state explicitly). Provenance records the count.

## Reproduce

```bash
export PATH=/opt/rocm/core/bin:/opt/rocm/core/lib/llvm/bin:$PATH
export ROCM_PATH=/opt/rocm/core HIP_PATH=/opt/rocm/core
export HIP_CLANG_PATH=/opt/rocm/core/lib/llvm/bin
export REDLINE_REAL_HIPCC=/opt/rocm/core/bin/hipcc

cargo build --release -p redline-capi

python3 examples/hipengine-6409/run_matrix.py \
  --hipengine-root /path/to/hipEngine \
  --backends hip,vulkan,redline \
  --modes serial_latency,independent_throughput \
  --gfx-arch gfx1201 \
  --reps 20 --warmup 3 --samples 7 \
  --out-dir examples/hipengine-6409/results/gfx1201/manual

python3 examples/hipengine-6409/summarize_results.py \
  examples/hipengine-6409/results/gfx1201/manual \
  --out examples/hipengine-6409/results/gfx1201/manual/summary.json
```

`run_matrix.py` resumes from completed artifacts. The ROCm 7.14 TheRock layout
expects `clang-offload-bundler` / `llvm-readobj` under the core LLVM dir (the
bundled `toolchain/` helpers resolve those paths).

Dispatch/grid controls (separate from the 224 core rows) use
`dispatch_matrix.py` when needed; retained ROCm 7.14 core reports below are the
112×2 matrix only.

## Retained result index (ROCm 7.14)

| Arch | Path | RL > Vulkan | Notes |
| --- | --- | ---: | --- |
| gfx1201 | [`results/gfx1201/2026-07-22-714-bench/`](results/gfx1201/2026-07-22-714-bench/REPORT.md) | **197/224 (87.9%)** | Primary HipEngine gfx1201; 151/224 RL firsts; 1737/1737 measured oracle passes in report |
| gfx1151 | [`results/gfx1151/2026-07-22-714-bench/`](results/gfx1151/2026-07-22-714-bench/REPORT.md) | **164/224 (73.2%)** | Short report from `summary.json` |
| gfx1100 | [`results/gfx1100/2026-07-22-714-bench/`](results/gfx1100/2026-07-22-714-bench/REPORT.md) | **127/224 (56.7%)** | Short report from `summary.json` |

Machine-readable: each tree's `summary.json` + `matrix.json`. Environment
records may include **absolute collector/runner paths** from the capture host;
leave them as-is.

## Interpretation guardrails

1. Ratios are **Redline GPU time / peer GPU time** (below 1.0 favors Redline).
2. Core matrix is **224** matched rows (112 serial + 112 independent). Do not
   conflate with Hipfire's 240-row four-backend matrix.
3. Remaining Vulkan losses concentrate in codegen-bound families (packed-dot,
   vopd, some two-stage shapes) — same HSACO for HIP/Redline.
4. **Do not cite pre-7.14 HipEngine trees** (`2026-07-12`, `2026-07-13-*`) as
   current product evidence.
5. Hipfire numbers live under [`../hipfire-6409`](../hipfire-6409/README.md);
   do not paste HipEngine tables into Hipfire reports as if they were the same
   run.
