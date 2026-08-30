<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# Upstream posting plan: REJECTED. Do not post the queued #10836 draft.

Two independent checks killed the same plan on the same day, for the same
underlying reason. Recorded so the draft is not picked up later as ready.

## 1. Adversarial review verdict: reject

Reviewed: a draft comment for `rocm-systems#10836` plus a five-step posting plan.
Five concrete errors, not stylistic objections.

1. **Version mislabel.** The draft's explicit-stream table was labelled
   "ROCm 10.0" and contained the **7.14** column verbatim
   (gfx1100 32.215/16.376/8.652 is 7.14; 10.0 is 32.213/16.361/8.712; gfx1151
   starts 32.426 not 32.492; gfx1201 at four streams is 11.325 not 11.457).
   The quoted 3.72x/3.52x/2.84x are therefore 7.14 ratios. Verified directly
   against `artifacts/2026-08-27-stream-concurrency-and-queue-cap.md`.
2. **"On four RDNA parts" unsupported.** The explicit-stream probe covers
   gfx1100, gfx1151, gfx1201. gfx1030 appears only in the hipGraph
   `ParallelChains` queue-cap sweep. A graph result cannot become a fourth
   eager-stream result.
3. **Mechanism conflation.** The Q=1..7 cliff is a 512-node, eight-chain,
   no-op-kernel **hipGraph** measurement. #10836 is four eager non-blocking
   streams with real geometry/reduction kernels. The eager-stream data support a
   default-cap limiter, not a cliff at Q=6; the draft transferred the graph cliff
   to the stream path.
4. **The falsifiable prediction was reckless.** Four streams cannot ordinarily
   consume a fifth or sixth queue — the sweep used eight chains precisely so
   Q=5/Q=6 could be exercised. Different host and different ROCm release from the
   reporter's 7.15. A failed prediction would have demonstrated the same
   measurement-to-conclusion error publicly.
5. **"Within 3% of optimal" false.** Versus Q=5: gfx1100 3.21%, gfx1151 4.85%,
   only gfx1030 within 3% (0.53%). "Within 5%" would have been true.

Also flagged: "host-specific threshold" / "cannot be derived from the spec sheet"
overreach, since architecture, host, firmware, GPU population and workload all
vary together and the artifact itself says host and architecture cannot be
separated.

## 2. Prior work in this repo had already superseded the plan

Ten commits sit between `d280d45` and `2ef12b8`. Three bear directly on it:

- **`2ef12b8`** measured llama.cpp's actual decode shape (390 nodes, 32 layers,
  one serial residual chain, gate 39000/39000). **Multi-queue contributes exactly
  nothing** — off and auto agree at every work level, because a decode chain is
  one weakly-connected component that `segment.rs` calls `Unsplittable`. The
  honest end-to-end interposer figure is **~4%** at 50 us/kernel, where a
  7B-class Q4 model sits. It states plainly that earlier 2.24x-6.90x figures came
  from "a probe built to contain independent components, with empty kernels --
  both conditions false for llama.cpp".
- **`83889d6`** found that PM4 **silently executed nothing** on gfx1030: both
  correctness gates read `counter = 0 / 512` while the call returned success. So
  the gfx1030 PM4 row in `artifacts/2026-08-27-pm4-four-arch-vs-rocm10.md`
  (0.2012 / 0.1108 us) is void, and RDNA1/RDNA2 are now refused.
- **`8ff8169`** revises the interposer's standing against Vulkan.

## 3. The common cause

Every measurement in the 2026-08-27 artifact set uses a no-op or spin kernel on a
shape chosen to expose parallelism. That is the doubly-favourable regime
`2ef12b8` repudiates, and it is why the internal review and the prior work
converged: the numbers are floor measurements being read as application claims.

## 4. What survives

- The **#9360 acknowledgment** to @doplxyz (crediting the 7.2-vs-develop
  `hipMemGetAllocationGranularity` divergence and rescoping the author's earlier
  claim to 7.14) is unaffected by any of the above and is still owed.
- The queue-cap **limiter** observation on eager streams is real; the **cliff**
  transfer is not.
- The `rocm_ident` provenance probe and the one-shape-per-process trace method
  remain correct and should stay in use.

## 5. What must happen before anything is posted

1. Read all ten prior commits, not just the three above.
2. Strike the gfx1030 PM4 row and add an empty-kernel caveat to the artifacts.
3. Re-derive any stream claim from the correct ROCm column, with the version
   stated per table.
4. Re-scope #10836 material to eager streams only, with no graph-cliff transfer
   and no prediction about the reporter's hardware.
