<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# Exact HipEngine source isolates the Hipfire parity gap

## Outcome

The five families that accounted for most of the Hipfire/HipEngine discrepancy
now have an exact-source control. VOPD and memory/waitcnt load the already-built,
hash-certified HipEngine code objects directly through the Hipfire Rust harness.
Dense Q8, selected-dual Q4, and selected-down Q6 run their unchanged multi-kernel
HipEngine fixtures through the same Rust Redline C ABI after VOPD and memory
established direct-Rust/C-ABI replay parity.

All correctness gates pass: 48/48 VOPD policy observations, 48/48 memory policy
observations, and 56/56 production rows.

Against the original matched HipEngine Vulkan implementations, exact-source
Redline wins **85/88 (96.59%)**:

| Family | Exact-source Redline wins | N | Geomean RL/Vulkan |
|---|---:|---:|---:|
| VOPD | 16 | 16 | 0.5681 |
| Memory/waitcnt | 13 | 16 | 0.8591 |
| Dense Q8 | 40 | 40 | 0.2153 |
| Selected-dual Q4 | 10 | 10 | 0.4575 |
| Selected-down Q6 | 6 | 6 | 0.3287 |
| **Total** | **85** | **88** | **0.3703** |

This is the valid original-source control. It shows that the old bespoke
Hipfire HIP reconstruction was not measuring a Redline replay limitation.

## Do not turn this into a 225/240 claim

The earlier Hipfire matrix used different custom HIP and Vulkan
implementations. Its five affected families won 45/88 against Hipfire's own
Vulkan rows. Reusing those retained Hipfire Vulkan times as a diagnostic
reference, exact HipEngine HIP source wins 65/88:

| Family | Custom Hipfire wins | Exact HIP wins vs Hipfire Vulkan | N |
|---|---:|---:|---:|
| VOPD | 5 | 8 | 16 |
| Memory/waitcnt | 4 | 9 | 16 |
| Dense Q8 | 32 | 40 | 40 |
| Selected-dual Q4 | 2 | 2 | 10 |
| Selected-down Q6 | 2 | 6 | 6 |
| **Total** | **45** | **65** | **88** |

Those measurements are cross-session, and Hipfire's Vulkan shaders are not
the original HipEngine Vulkan implementations. A mechanical row replacement
would estimate 205/240 rather than 225/240, but even **205/240 (85.42%) is not
a measured full-matrix result**. A same-session run with explicit GPU-only
preheat on both retained command paths is required before publishing a new
240-row headline.

The disagreement itself is useful. Hipfire's custom selected-dual Q4 Vulkan
shader is much faster than the original HipEngine Vulkan implementation, so
exact HipEngine HIP source remains 2/10 against that stronger implementation
even though it wins 10/10 in the matched HipEngine control. That is now a real
Radiowave/kernel target, not replay-controller ambiguity.

## Memory/waitcnt direct-Rust result

The new `hipengine_exact_memory` runner reconstructs HipEngine's exact six-
argument ABI and deterministic memory fixture for all four variants, two
workgroups, and both timing modes. It does not recompile the kernel.

The HipEngine-compatible Rust path is at parity with a saturation-warmed C-ABI
run of the same code objects: Rust/C-ABI geometric mean is **0.9902** overall,
0.9983 serial, and 0.9822 independent. The safe Rust policy is **1.0203**
overall, 1.0126 serial, and 1.0279 independent. Replay transport is not the
source of the old memory gap.

Against the matched HipEngine Vulkan rows, compatible and safe replay each win
**13/16**. Compatible replay has a 0.8591 Redline/Vulkan geometric mean; safe
replay is 0.8851.

The remaining losses are localized to independent throughput:

- coalesced WG64 is effectively tied: 11.844 us Redline versus 11.828 us
  Vulkan (+0.14%);
- strided WG64 is 11.832 us versus 6.576 us;
- strided WG256 is 11.844 us versus 6.480 us.

The two material losses survive bytecode-identical Rust/C-ABI replay, so they
are kernel/LLVM-versus-ACO codegen targets, not dependency-boundary or harness
failures. Every certified memory consumer is `vmem_only` and selects Redline's
narrow LLVM-VMEM dependency boundary.

Kernarg reuse again fails as a blanket policy. It shortens the ten-dispatch
serial tape from 199 to 172 dwords, but its overall C-ABI ratio is 0.9925
versus 0.9902 for distinct kernargs. It remains an opt-in measured selection.

## Production-source result

The unchanged dense-Q8, selected-Q4, and selected-Q6 production slices pass
all 56 correctness gates after 1,000 out-of-sample retained-IB preheat replays.
Compared with the retained matched HipEngine Vulkan artifacts:

| Family | Redline wins | N | Geomean RL/Vulkan |
|---|---:|---:|---:|
| Dense Q8 | 40 | 40 | 0.2153 |
| Selected-dual Q4 | 10 | 10 | 0.4575 |
| Selected-down Q6 | 6 | 6 | 0.3287 |
| **Total** | **56** | **56** | **0.2577** |

The earlier short-warmup exact HipEngine run already won 55/56. Preconditioning
flips its only loss, the serial Q4 WG128 prequantized dot, from 226.792 us to
114.492 us against retained Vulkan's 212.432 us.

The Vulkan values here remain the retained correctness-gated suite artifacts.
A symmetric `--warmup 1000` rerun was attempted and rejected: the pinned
production harness uses `warmup` to multiply full CPU oracle fixtures, so it
would spend hours constructing references rather than performing a clean GPU
preheat. A future Vulkan-side out-of-sample command-buffer preheat hook is the
right control. This report does not label the retained comparison same-session.

## Verdict

Radiowave is doing its job in the matched original-source experiment. Exact
HipEngine HIP artifacts through Hipfire/Redline beat their original Vulkan
counterparts in 85/88 cases, while direct Rust and C-ABI replay are at parity.
The old custom-source gap was therefore primarily an implementation/codegen
comparison, not inherent HIP dispatch overhead.

It is not yet valid to say the exact-source Hipfire matrix wins 225/240. The
remaining practical target is precise: tune the exact HIP selected-dual Q4 and
independent strided-memory kernels against Hipfire's stronger Vulkan shaders,
then run both retained paths with symmetric GPU-only preheat.

Machine-readable summary:
[`summary.json`](summary.json).

Primary result artifacts:

- [exact VOPD result](../2026-07-13-hipengine-exact-vopd-saturated.json)
- [exact memory result](../2026-07-13-hipengine-exact-memory-saturated.json)
- [preheated production matrix](../../../../hipengine-6409/results/gfx1201/2026-07-13-hipengine-production-preheated/matrix.json)
