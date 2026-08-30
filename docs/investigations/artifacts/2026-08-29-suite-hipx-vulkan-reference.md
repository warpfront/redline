<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# Full 240-row suite on hipx vs RADV Vulkan: redline beats Vulkan; empty-kernel tuning does not transfer

**Status: internal. NOT posted upstream.**

First full-suite run on gfx1100 and gfx1151 (previous suite artifacts were
gfx1201-only). ROCm 10.0 matched-pair builds, per-run provenance recorded in
each JSON. 240 hipEngine-comparable rows (real kernels, real data, per-row
correctness validation), 4 backends, 6 runs, **5,760 correctness gates, zero
failures**. Vulkan is RADV. Joined by `examples/hipfire-6409/join_arms.py`;
raw JSONs committed under `examples/hipfire-6409/results/{gfx1100,gfx1151}/`.

Arms: `legacy` (hip/hipGraph fixed 4 lanes, redline auto), `auto`
(`--hip-queues auto`: gfx1100=4, gfx1151=5 — the empty-kernel cliff optima),
`rq1` (`--redline-queues 1`, single-lane PM4). Resolved widths verified from
the row records, not assumed: gfx1100 auto=legacy=Q4 (that arm is a no-op there
by construction), gfx1151 auto=Q5 vs legacy Q4; redline lane policy picks 2 on
gfx1100, 4 on gfx1151.

## Geomean ratio vs the reference run's Vulkan column (<1 = faster than Vulkan)

| arch / mode | hip | hipgraph | redline (multi-lane) | redline (1 lane) | wins vs Vulkan (redline) |
| --- | ---: | ---: | ---: | ---: | ---: |
| gfx1100 serial_latency | 1.35 | 1.30 | **0.72-0.74** | 0.71 | 96-98 / 120 |
| gfx1100 independent | 2.43 | 2.97 | **0.67-0.69** | 0.79 | 90-91 / 120 |
| gfx1151 serial_latency | 1.51 | 1.50 | **0.78** | 0.78 | 95-96 / 120 |
| gfx1151 independent | 2.59 | 3.23 | **0.72** | 0.77 | 84-85 / 120 |

Medians tell the same story with less tail: stock hip/hipGraph sit near Vulkan
parity at the serial median (0.95-1.07) but carry catastrophic tail rows
(19x-89x slower than Vulkan); redline's worst row is 3.4x (serial) / 9.4x
(independent, gfx1100).

## Findings

1. **The spec claim is understated.** On this suite redline does not merely
   promote HIP to Vulkan-matching levels — it beats RADV outright on 84-98 of
   120 rows per mode/arch, geomean 0.67-0.78x, while stock HIP/hipGraph are
   1.3-3.2x slower than Vulkan (geomean). The gap between "HIP loses to Vulkan"
   and "redline beats Vulkan" is the entire product thesis, measured on real
   kernels with per-row output validation, on two architectures.

2. **Empty-kernel lane optima do NOT transfer to real kernels.** The
   queue-width-cliff sweep (no-op kernels) put gfx1151's optimum at Q5.
   On the real suite, Q5 is 9-12% WORSE than legacy Q4 (hip geomean 2.59 ->
   2.82, hipGraph 3.23 -> 3.63, independent mode). This resolves the
   2026-08-27 multiqueue artifact's open caveat "whether the lane optima hold
   for real kernels": they do not. `--hip-queues auto`'s per-device table is
   built from the wrong regime and should not be treated as tuned guidance
   for real workloads.

3. **Multi-queue PM4 is worth +15% / +6% on real kernels, not 2.19x / 3.51x.**
   redline multi-lane vs `--redline-queues 1`, independent mode: gfx1100
   0.675 vs 0.790 geomean (+15%, 2 lanes), gfx1151 0.724 vs 0.770 (+6%,
   4 lanes). Serial rows are identical by design (single lane either way).
   Same lesson as the llama-shape probe: submission wins compress as kernels
   get real. The empty-kernel multi-queue figures are floor measurements.

4. **Noise floor, measured not assumed.** The Vulkan column repeats across
   runs: gfx1100 cross-run geomean drift 3-5%, gfx1151 0.1-4%. Every claim
   above exceeds it except none; the gfx1100 auto-vs-legacy hip/hipGraph
   deltas (<0.3%) are pure noise as expected from identical resolved widths.

5. **gfx1030 is blocked at compile time.** `reduction_multi16` exhausts
   registers under wave64 `gcn-max-ilp` for gfx1030, and the CLI guardrail
   requires redline+vulkan in any `--backends` subset while PM4 correctly
   refuses RDNA2. RDNA2 suite coverage needs a kernel-suite fix (skip or
   retune that scheduler profile), not a runner change.

## Caveats

- One run per cell (the suite internally does 3 warmups / 7 samples per row);
  the 2026-08-27 gfx1201 artifact showed single-run 2x outliers on individual
  rows at scale, so per-row figures should not be quoted to two decimals —
  the geomeans over 120 rows are the stable objects.
- Vulkan reference is RADV/Mesa as shipped on Ubuntu 26.04; a Mesa upgrade
  moves the denominator.
- gfx1201 (hiptrx) not rerun tonight; its full-suite numbers live in the
  2026-08-27 queue-policy artifact.
