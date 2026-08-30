<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# Claims discipline: what the numbers are claims *about*

Every scoping error in this repo's history has been the same shape — a quantity
measured on one axis, restated as if it were on another. This file fixes the
language for the two figures that matter, and answers the standing objection.

## The two figures are different quantities. Both are true.

**Figure 1 — per-dispatch submission cost. 3.5x to 11.6x.**
PM4 conservative (retained IB), against *tuned* hipGraph, gate-verified:

| part | PM4 | hipGraph default | hipGraph tuned | vs default | vs tuned |
| --- | ---: | ---: | ---: | ---: | ---: |
| gfx1100 | 0.2265 | 2.791 | 0.842 (Q5) | 12.3x | **3.7x** |
| gfx1151 | 0.2381 | 1.741 | 0.825 (Q5) | 7.3x | **3.5x** |
| gfx1201 | 0.1471 | 2.168 | 1.705 (Q2) | 14.7x | **11.6x** |

gfx1030 is struck: `83889d6` proved PM4 executed nothing there (gate 0/512 with a
success return). RDNA1/RDNA2 are now refused.

**Figure 2 — end-to-end token latency. +8% to +10.5%.**
Measured independently by @Ilintar/@pwilkin (llama.cpp, Ornith 35B-A3B, 62 ->
68.5 tok/s) and @lhl (W7900, +8.13%).

Figure 1 is the cost of *submitting* a dispatch. Figure 2 is what that buys on a
token, given submission is currently 10-12% of one. Neither is a compute claim.
Compute is untouched — the same kernels, the same code objects, the same
hardware, byte-identical dispatch packets. **Nothing in this work makes a kernel
run faster.** It removes host-side cost between kernels.

Stating Figure 1 without naming "per dispatch, submission path" invites the
misreading, and stating it as "3-10x faster" full stop is the misreading.

## The standing objection, and why it is weak

> "Per-launch overhead amortizes away with real kernel sizes."

Four responses, in ascending order of force:

1. **It concedes the mechanism and argues only about the coefficient.** The
   objection agrees submission cost is real and serial; it claims the divisor is
   large. That makes it a question about workload regime, answerable by
   measurement — and measured, the divisor puts real models at 10-12% submission
   share, not 1%.

2. **The regime is moving toward us, not away.** Every current direction in
   inference increases dispatch count and decreases us/kernel: MoE (35B-A3B runs
   ~3B active as many small expert kernels), speculative decode, paged attention
   at long context, and heavier quantisation. This is already visible in the field
   data — the *larger* MoE model shows the *larger* win (+10.5%) than the dense
   measurement (+8.13%), which is backwards under dense reasoning and expected
   under MoE.

3. **Amdahl runs the wrong way for the runtime.** Device parallelism overlaps
   compute; it does not overlap host submission, which is serial. So exploiting
   more queues or streams *raises* submission share rather than lowering it. The
   queue-cap data show this directly: tuning hipGraph from default Q4 to its
   optimum cuts per-dispatch cost 2.1x-3.3x on the hipx parts, and every bit of
   that makes the remaining submission cost a larger fraction of the total.

4. **The strongest response: amortization has already been implemented, by AMD,
   and shipped.** hipGraph *is* the amortization argument in code — capture once,
   replay, skip re-validation — and CLR already batches AQL packets
   (`CaptureAQLPackets`, `dispatchAqlPacketBatchFlat`, both present in
   `release/therock-7.14`, source-verified). After all of that, the floor is still
   2.168 us/dispatch on gfx1201, and it did not move across **three** releases:
   7.2 (2.113-2.133), 7.14 (2.144), 10.0 (2.146), each measured as a matched pair
   with its own toolchain and verified runtime provenance. An overhead that
   survives the vendor's own amortization across three releases is structural,
   not amortizable.

## The bound the objection cannot cross

At 0 us/kernel the interposer is 2.26x on a real llama.cpp decode shape. That is
the ceiling: it is what submission cost is worth when compute is free. It matters
because it is a *floor on the floor* — no amount of amortization reduces
per-dispatch cost below what the submission path itself costs, so any workload
whose kernels approach that floor is dispatch-dominated by construction. The
amortization argument is an argument about where a given workload sits on this
curve. It is not an argument that the curve is flat.

## Required form for any published figure

- Name the axis: "per-dispatch submission cost," never bare "faster."
- Name the comparand: default hipGraph or tuned, with the queue count.
- Name the regime for any end-to-end figure: us/kernel, model, quantisation.
- State that compute is unchanged.
- Cite the correctness gate alongside the timing, since a fast wrong answer is
  what `83889d6` caught on gfx1030.
