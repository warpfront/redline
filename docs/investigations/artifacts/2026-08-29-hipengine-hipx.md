<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# hipEngine on hipx: lhl's harness agrees with ours, from a different point on the curve

**Status: internal. NOT posted upstream.**

First hipEngine (shisa-ai/hipEngine, pinned `6da4702` — the same commit as the
July gfx1201 baseline) runs on gfx1100 and gfx1151. ROCm 10.0, reps 20 /
warmup 3 / samples 7 (July parameters, kept for comparability), backends
hip/vulkan/redline both modes plus hipgraph serial-only (the shim's documented
limit; its independent-mode failure killed the first launch — run split
accordingly). Fail-closed Radiowave certification on every redline row. Results
committed under `examples/hipengine-6409/results/{gfx1100,gfx1151}/`.

## Overall (224 matched rows per arch; ratio < 1 = redline faster)

| arch (runtime) | redline 1st | vs hip median | vs hip wins | vs vulkan median | vs vulkan wins |
| --- | ---: | ---: | ---: | ---: | ---: |
| gfx1201 (7.14, July pin) | 67.4% | 0.855 | 71.4% | **0.480** | 87.9% |
| gfx1100 (10.0, tonight) | 48.2% | 0.946 | 64.7% | **0.821** | 66.5% |
| gfx1151 (10.0, tonight) | 65.2% | 0.889 | 88.4% | **0.601** | 70.1% |

Extremes worth knowing: redline's best row is ~0.11-0.22x on every arch (9x
faster than hip, 4.5x faster than Vulkan); its worst is 2.76x vs hip (gfx1100)
and **13.3x vs Vulkan on one gfx1151 row** — unexamined, and the honest next
question on this dataset.

## Two independent harnesses, one story

The Rust suite (240 rows, same silicon, same night) put redline at geomean
0.67-0.78x of Vulkan with 85-98/120 wins. hipEngine — lhl's own Python harness,
unmodified, compute-weighted row mix with real model slices — compresses that
to medians 0.60-0.82x with 66-70% wins. Same direction, smaller magnitude on
heavier rows: exactly the compression claims-discipline predicts when
us/kernel rises. Two harnesses, different authors, different row mixes,
agreeing on sign everywhere and disagreeing on magnitude only where kernel
weight differs.

The per-arch contrast is informative rather than noisy: gfx1100 is the
tightest (median vs hip 0.946) because its hipEngine mix leans hardest on
compute-bound slices where nothing can differ; gfx1151 — the APU AMD ships in
laptops, the part #10836's reporter measured — is redline's strongest 10.0
result (88.4% wins vs hip, 0.601 median vs Vulkan).

## Gaps and caveats

- hipgraph rows are serial-only on the Python side (shim limitation); the
  independent hipgraph comparison lives only in the Rust suite. The summarizer's
  overall hipgraph block is null for the same reason.
- gfx1201 has no ROCm 10.0 hipEngine run yet (hiptrx untouched tonight); its
  row is the July 7.14 pin. Rerunning it under 10.0 is the missing cell in the
  matrix.
- The 13.3x gfx1151 vs-Vulkan outlier row is unexamined.
- gfx1030 remains excluded end to end (PM4 refused by design; suite kernels
  also fail to compile there).

## Addendum: gfx1201 rerun under ROCm 10.0 (hiptrx, R9700)

The missing cell is filled. hipEngine, same pinned commit and parameters:

| arch (runtime) | redline 1st | vs hip median | vs hip wins | vs vulkan median | vs vulkan wins |
| --- | ---: | ---: | ---: | ---: | ---: |
| gfx1201 (10.0, hiptrx R9700) | 56.7% | 0.896 | 67.4% | **0.627** | 80.8% |

Against the July 7.14 pin (0.480 median vs Vulkan, 87.9% wins) the margin is
narrower — but host SKU (RX 9070 XT class vs R9700), Mesa, and ROCm all changed
together, so no per-factor attribution is available from these two points.

Rust suite on the same host/runtime (240 rows, join vs Vulkan reference):
redline geomean **0.59** independent (87/120 wins; multi-queue worth +19% over
single-lane) and **0.90** serial (82/120 wins). Stock hip/hipGraph: 1.8-3.9x
slower than Vulkan by geomean, medians 1.17-3.11, tails to 93x.

Third confirmation that empty-kernel queue optima do not transfer: the cliff
sweep said Q2 is optimal on gfx1201, but on real kernels `--hip-queues auto`
(Q2) is WORSE than legacy Q4 (hip geomean 3.29 -> 3.76 independent, worst row
92x -> 168x). Together with gfx1151's Q5 regression this now holds on every
part tested, in both directions. The tuned-width table in the suite should be
treated as an empty-kernel artifact, not guidance.
