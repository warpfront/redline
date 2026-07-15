# HipEngine #6409 with Radiowave + Redline

This run applies the portable Hipfire findings to the pinned HipEngine micro
suite at `f2c3ad6d74c86e3641ce09ff9fd759eaa6cd75e0`: Radiowave owns HIP
compilation and exact-manifest certification; Redline records a stateful
retained PM4 IB; and every serialized edge selects its cache acquire from the
verified *next consumer*.

## Result

On the gfx1201 R9700, Redline beats Vulkan in **192/212 matched kernel rows
(90.57%)**. The previous HipEngine run was 186/212 (87.74%): six rows flipped
from Vulkan to Redline, and none flipped back.

| Metric | Result |
|---|---:|
| Redline faster than Vulkan | 192/212 (90.57%) |
| Redline faster than HIP | 158/212 (74.53%) |
| Redline first / second / third | 149 / 52 / 11 |
| Redline first-place rate | 70.28% |
| Median Redline / Vulkan GPU time | 0.4589 |
| Median Redline / HIP GPU time | 0.8343 |
| Serial Redline wins over Vulkan | 103/112 (91.96%) |
| Independent Redline wins over Vulkan | 89/100 (89.00%) |

This is the same approximately 90% Vulkan win rate as the Hipfire-native
result, now reproduced through HipEngine's own pinned runners and kernels.
“Beats Vulkan” is intentionally separate from “first place”: HIP wins 44 of
Redline's 63 non-first rows, while Vulkan wins 19; one additional row has HIP
first and Vulkan second.

All **224/224 Redline** kernel rows passed their CPU oracles. Vulkan retained
288/288 passing rows. HIP retained 212/212 passing rows; its independent
sampler rejected its own timed dependency contract twice, exactly as in the
historical run, so those 12 shapes have no valid three-way comparison.

## Remaining Vulkan wins

The 20 losses are concentrated rather than submission-wide:

- packed dot: 16/16 losses, 2.87% to 21.28% behind Vulkan;
- independent memory/waitcnt: three losses—coalesced WG64 by 0.85%, strided
  WG256 by 85.06%, and strided WG64 by 90.15%;
- serial selected-dual Q4 WG128: 6.76% behind Vulkan (HIP is first).

Geometry, reduction, VOPD, sampler, two-stage reduction, Q6-x8, and dense-Q8
all beat Vulkan in every matched row. The remaining cluster is therefore a
HipEngine-kernel lowering target—especially packed-dot and strided memory—not
evidence of a retained-PM4 submission floor.

## What was transferred

- Every Redline-side HipEngine build is produced by Radiowave, with an adjacent
  schema-3 manifest bound to the exact loadable bundle SHA-256.
- Original HipEngine flags are preserved. Fast math is preserved only when the
  original build requested it; enabling it by default failed the production Q4
  oracle and was rejected.
- The portable wave policy uses wave64 for VOPD and interleaved memory. The
  Hipfire production-Q wave64 rule was tested but not copied mechanically:
  HipEngine's Q4 source assumes wave32 subgroup structure and failed its oracle,
  so the certified integration correctly retains wave32 there.
- C and PyO3 consume the common `CodeObjectCertification` API. Raw modules,
  stale manifests, missing symbols, and scalar/unknown consumers fail closed to
  the generic same-agent RMW boundary.
- Synthetic C++ runners and production-shaped Python runners both use the C ABI
  to capture HipEngine's launch closure once and replay the resulting stateful
  PM4 IB. The HIP graph is introspection-only and never timed.

Radiowave's buffer-resource helpers remain explicit source operations; this run
does not pretend that force-including the header rewrites arbitrary HipEngine
pointer expressions. The result isolates the improvements that transfer safely
without forking or silently replacing HipEngine's kernels.

## Dispatch control

The separate GMB dispatch/grid control is now also Radiowave-produced and
hash-verified. Its manifest certifies `vmem_only`, so serial RMW chains use the
`hip-llvm-vmem` boundary. All 16 Redline dispatch rows pass full-grid
correctness, including grids 128, 1024, and 8192.

Redline beats HIP in 16/16 dispatch rows and Vulkan in 6/16. At the issue's
941-launch, one-block serial chain, Redline is **0.7577 us/dispatch**, versus
Vulkan **1.1226 us** and HIP graph **3.2946 us**. Vulkan retains the advantage
at N=1 (1.6000 us versus Redline 3.3600 us) and on most explicitly independent
dispatch rows. If the distinct dispatch control is mechanically pooled with
the kernel matrix, the combined count is 198/228 Vulkan wins (86.84%); the
machine-readable summary keeps the two populations separate.

## Method

- GPU: AMD Radeon Graphics, gfx1201
- pinned HipEngine commit: `f2c3ad6d74c86e3641ce09ff9fd759eaa6cd75e0`
- kernel matrix: 10 repetitions, 3 warmups, 5 samples
- modes: `serial_latency`, `independent_throughput` with four independent streams
- comparison clock: each backend's retained GPU timestamp/event span
- raw matrix: [`matrix.json`](matrix.json)
- matched analysis and every loss: [`summary.json`](summary.json)
- dispatch control: [`dispatch-matrix.json`](dispatch-matrix.json)
- captured environment: [`environment.json`](environment.json)

Reproduce from `examples/hipengine-6409`:

```bash
python3 run_matrix.py \
  --hipengine-root /tmp/hipEngine-f2c \
  --out-dir results/gfx1201/2026-07-13-radiowave-redline \
  --gfx-arch gfx1201 --gpu-name 'AMD Radeon Graphics' \
  --reps 10 --warmup 3 --samples 5 \
  --backends hip,vulkan,redline \
  --modes serial_latency,independent_throughput
```
