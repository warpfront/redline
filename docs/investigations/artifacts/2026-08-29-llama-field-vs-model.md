<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# The field data lands on the model. The "~4%" headline was the wrong point on it.

`2026-08-27-llama-shape-result.md` closes with "the honest headline is the 4% one,
because that is the regime real users are in." That last clause is unsupported, and
two independent real-world integrations contradict it. Correcting.

## The two field measurements

Neither is ours; both are end-to-end on real workloads, not synthetic shapes.

| source | workload | hardware | result |
| --- | --- | --- | --- |
| @Ilintar / @pwilkin, llama.cpp fork | Ornith 35B-A3B (MoE, ~3B active) | RDNA3 | 62 -> **68.5 tok/s**, +10.5% |
| @lhl | hipEngine decode, 239-row median | W7900 | **+8.13%** |

## They are on the curve, not off it

The probe measured that the interposer removes 75-83% of submission cost. Taking
r = 0.78, a speedup S implies a stock submission share p via `S = 1/(1 - r*p)`:

| field result | speedup | implied submission share |
| --- | ---: | ---: |
| +10.5% | 1.1048 | **12.2%** |
| +8.13% | 1.0813 | **9.6%** |

Against the probe's own measured anchors — 29.0% share at 5 us/kernel, 4.8% at
50 us/kernel — both field results sit **between** the anchors, implying roughly
15-22 us/kernel. That is an ordinary regime for real models, and it is not where
the artifact placed "real users."

So the synthetic probe **predicted the field results** from an independent
direction. Prediction plus independent confirmation is stronger evidence than
either alone; the artifact treated its own model as if it superseded the field
data, when in fact the two agree.

## Why 50 us/kernel was the wrong anchor to call typical

The artifact reasoned "~390-500 nodes at 20-50 us each gives 10-20 ms/token,
i.e. 50-100 tok/s" and then chose the top of the 20-50 us range. Two problems:

1. It chose the end of its own stated range least favourable to the result, with
   no measurement selecting it. The range's other end, 20 us, is exactly where
   the field data lands.
2. **MoE breaks the per-kernel work assumption.** Ornith 35B-A3B activates ~3B
   of 35B parameters, so decode is many small expert kernels rather than fewer
   large dense ones. Lower us/kernel means a higher submission share, which is
   why the MoE integration shows the *larger* win (+10.5%) despite the *larger*
   model. Dense-model reasoning does not transfer, and MoE is where the field is
   moving.

## Corrected claim

**Measured, end-to-end, by two independent parties: +8% to +10.5%.** The model
explains both, bounds the range as 4% (dense, ~50 us/kernel) to ~31% (small or
MoE-heavy, ~5 us/kernel), and the multi-queue finding still stands: a decode
chain is `Unsplittable`, so the entire win is PM4 lowering.

What remains true from the earlier correction: 6.9x and 2.26x are not available
to llama.cpp, and any figure must name its us/kernel regime. What was wrong: a
single unmeasured choice of anchor became "the honest headline," discarding two
real integrations that had already answered the question.

## Open item

Neither field result recorded us/kernel directly, so the implied shares above are
inferences from a model, not measurements. The probe already sweeps `--work`; the
missing measurement is a real llama.cpp run under `rocprofv3` to get the actual
per-kernel duration distribution for a named model, which would convert these
inferences into a direct check. Worth doing before any figure is published.
