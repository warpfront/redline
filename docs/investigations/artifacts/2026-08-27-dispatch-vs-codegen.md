<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# Correction: redline recovers 91-99% of HIP's gap to Vulkan; the residual is codegen

**Status: internal. NOT posted upstream.**

Earlier in this session I characterised the suite result as "redline loses to Vulkan
1.6x-2.2x on quantised matmul, the inference-critical path" and ranked explaining
it as the top strategic unknown. That framing was wrong, and the error was mine:
I compared redline to Vulkan without also comparing HIP to Vulkan, so I attributed
a kernel-quality gap to redline's dispatch path.

## The two arms do not run the same kernels

- `kernels/hipfire_6409.hip` is compiled by `hipcc` (`examples/hipfire-6409/build.rs:135-142`)
  and is what **both** the HIP arm and the redline arm execute.
- `kernels/hipfire_6409.comp` is GLSL, compiled by `glslc` to SPIR-V and consumed
  by the Vulkan arm, which on AMD means RADV and ACO.

So redline and HIP execute byte-identical kernels and differ only in submission.
Any residual redline-vs-Vulkan difference on a compute-bound family therefore
cannot be dispatch cost — it is the HIP kernel versus the GLSL shader.

## What the numbers actually say

gfx1201, ROCm 10.0, `--hip-queues legacy`, medians over the family, us per
operation. "recovered" is the share of HIP's gap to Vulkan that redline closes.

| family | mode | vulkan | redline | hip | hip/vk | rl/vk | recovered |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| q4-selected-dual | serial | 56.85 | 90.94 | 2475.92 | **43.6x** | 1.60x | **98.6%** |
| q4-selected-dual | indep | 54.69 | 87.15 | 2062.86 | 37.7x | 1.59x | 98.4% |
| q6-x8-selected-down | serial | 10.39 | 25.66 | 299.92 | 28.9x | 2.47x | 94.7% |
| q6-x8-selected-down | indep | 7.10 | 37.48 | 347.95 | 49.0x | 5.28x | 91.1% |
| dense-q8 | serial | 12.24 | 12.22 | 48.74 | 4.0x | **1.00x** | **100.1%** |
| dense-q8 | indep | 7.58 | 7.88 | 49.22 | 6.5x | 1.04x | 99.3% |
| memory-waitcnt | serial | 27.31 | 29.66 | 43.37 | 1.6x | 1.09x | 85.4% |
| memory-waitcnt | indep | 20.83 | 22.39 | 48.11 | 2.3x | 1.07x | 94.3% |
| vopd | serial | 150.12 | 169.51 | 165.06 | 1.1x | 1.13x | n/a |
| vopd | indep | 141.87 | 155.22 | 160.32 | 1.1x | 1.09x | n/a |
| packed-dot | serial | 422.68 | 423.55 | 424.52 | 1.0x | 1.00x | n/a |
| packed-dot | indep | 392.94 | 218.68 | 410.92 | 1.0x | **0.56x** | n/a |

On quantised matmul HIP is **28x-49x slower than Vulkan**, and redline closes
91%-99% of that. On `dense-q8` it closes the gap completely (1.00x-1.04x). The
`recovered` column is meaningless where HIP was already at parity (`vopd`,
`packed-dot`, gap ~1.0x), so it is marked n/a rather than reported as a large
percentage of nothing.

## What this means for the architecture

The levers are orthogonal and the data now says which one is binding where:

- **Dispatch is redline's lever, and it is close to saturated.** Where HIP had a
  submission gap, redline removes 91%-100% of it. There is little left to win by
  making submission cheaper on these families.
- **The residual 1.6x-5.3x is codegen or kernel implementation**, since redline
  and HIP share kernels. That is radiowave's lever: comparing against Vulkan
  bytecode and emitting what `hipcc` does not produce by default.
- **Where HIP is already at Vulkan parity, redline gives nothing and costs about
  10%** (`vopd` 1.09x-1.13x). A dispatch optimiser cannot help a workload with no
  dispatch gap, and the retained-IB path is not free.

The one genuine outlier is `packed-dot` independent, where redline is **1.8x
faster than Vulkan** (218.68 vs 392.94) while HIP sits at 410.92. Unexplained; it
is the only family where redline beats Vulkan on compute-bound work, and it is
worth understanding rather than celebrating.

## Why the earlier framing was wrong, mechanically

A two-way comparison against the strongest rival is not enough when the arms do
not share an implementation. The correct baseline for "is redline doing its job"
is HIP, because that isolates the variable redline changes. Vulkan answers a
different question -- how far the platform is from a well-tuned AMD-native
consumer -- and conflating the two produced a claim that redline was failing at
precisely the thing it does best.
