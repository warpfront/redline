<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# llama.cpp-shaped decode: the interposer is worth ~4%, and multi-queue is worth zero

**Status: internal. NOT posted upstream.**

`bench/dispatch/llama_decode_shape.cpp` reproduces llama.cpp's per-token decode
dispatch shape: a graph built by **stream capture** (not explicit node APIs, which
is what llama.cpp does), 390 kernel nodes by default (32 layers x 12 + 6), one
serial residual chain on one stream, instantiated once and replayed per token.
`--work` sets per-kernel duration so the ratio of submission cost to kernel time
can be swept rather than assumed.

gfx1201, ROCm 10.0, 100 tokens, `--shape=cuda-serial`, gate `ok` (39000 / 39000
kernels counted) in every cell.

| `--work` | us/kernel | stock us/token | `off` | `auto` | best speedup | submission share, stock -> interposer |
| ---: | ---: | ---: | ---: | ---: | ---: | --- |
| 0 | 0.0 | 856.87 | 384.52 | **378.80** | **2.26x** | 100% -> 100% |
| 500 | 5.0 | 2744.34 | **2097.84** | 2098.48 | **1.31x** | 29.0% -> 7.1% |
| 5000 | 50.0 | 20477.11 | 19645.96 | **19645.77** | **1.04x** | 4.8% -> 0.8% |

## Multi-queue contributes nothing to this shape, as predicted

`off` and `auto` agree at every point: 384.52 vs 378.80, 2097.84 vs 2098.48,
19645.96 vs 19645.77. A decode chain is a single weakly-connected component, so
`segment.rs` reports `Unsplittable` and there is nothing to spread across queues.

The 2.24x-6.90x multi-queue results measured earlier came from `parallel-chains`,
a probe built to contain independent components, with empty kernels. Both of those
conditions are false for llama.cpp decode. **None of that speedup is available to
it.** The entire llama-shaped win is PM4 lowering — the per-dispatch submission
saving — and nothing else.

## The interposer removes most of the submission cost; submission is just small

It cuts the submission share from 29.0% to 7.1% at 5 us/kernel and from 4.8% to
0.8% at 50 us/kernel, i.e. it removes roughly 75%-83% of submission cost, which is
consistent with the dispatch-floor measurements. The end-to-end effect is bounded
by how much of a token submission was to begin with:

- **~50 us/kernel: ~4%.** A 7B-class Q4 model on this part sits near here, since
  ~390-500 nodes at 20-50 us each gives 10-20 ms/token, i.e. 50-100 tok/s.
- **~5 us/kernel: ~31%.** Small or heavily quantised models running at several
  hundred tok/s land in this regime.
- **0 us/kernel: 2.26x.** Pathological, reported only to show where the ceiling
  comes from. No real model is here.

So the defensible claim for llama.cpp-shaped decode is **single-digit percent at
typical model sizes, rising toward ~30% for small fast models**. Not 6.9x, and not
2.26x. The honest headline is the 4% one, because that is the regime real users
are in.

## Why this probe was worth building

Every prior interposer number in this project came from a synthetic shape chosen
to expose parallelism, measured with an empty kernel. That is doubly favourable,
and it produced numbers 1-2 orders of magnitude larger than what the dominant
deployed consumer shape would see. A sweep over per-kernel duration makes the
ceiling explicit and un-spinnable: the interposer can only ever help the
submission fraction, and that fraction is printed alongside every result.

## What is NOT established

- gfx1100 and gfx1151 not yet run for this shape. gfx1151 showed the largest
  parallel-chains win, but since multi-queue is inert here, its llama-shaped
  result should track the PM4 saving only.
- `--shape=qkv-fork` and `--shape=amd-wide` not yet measured on ROCm 10.0. On
  7.14 the probe's own gate caught multi-stream capture dropping secondary-stream
  kernels (qkv-fork 26/30 nodes, amd-wide 14/54) and correctly suppressed those
  timings. Whether 10.0 captures multi-stream graphs correctly is untested, and
  until it does, the "how much is left on the table by the CUDA-shaped graph"
  question is open.
- No real model, no real weights, no `mmvq` kernel. The kernels here are
  duration-controlled stand-ins, so this measures dispatch shape only. The
  suite's separate finding that HIP is 28x-49x slower than Vulkan on quantised
  matmul, of which redline recovers 91%-99%, is the codegen half of the picture
  and is not captured here.
- `--update` (exercising `hipGraphExecUpdate` per token, as llama.cpp does) not
  yet run under the interposer.
