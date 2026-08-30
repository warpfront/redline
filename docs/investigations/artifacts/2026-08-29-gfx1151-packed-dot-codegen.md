<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# The gfx1151 "13.3x outlier" is the whole packed-dot family, and it is pure codegen

**Status: internal. NOT posted upstream.**

Follow-up to `2026-08-29-hipengine-hipx.md`, which left one gfx1151 row at
13.3x vs Vulkan unexamined. Examined: it is not a row, it is the **entire
packed-dot family** — every variant (q8_signed, q4_unsigned, q6_zero,
scalar_dequant), both workgroup sizes, both modes, 10.2x-13.3x:

| variant (serial, wg64) | redline | hip | vulkan | rl/vk |
| --- | ---: | ---: | ---: | ---: |
| gfx1151 q8_signed | 3523.59 us | 3535.87 us | 289.11 us | **12.19x** |
| gfx1100 q8_signed | 330.63 us | 332.63 us | 289.20 us | 1.14x |

## The dispatcher is exonerated

redline tracks hip within 0.3% on the affected rows. The gap lives in the
kernel ISA that hipcc emits, not in anything redline does — same source, same
compiler, and gfx1100 is fine while gfx1151 is 12x off. A 40-CU part matching a
96-CU part under Vulkan (289 us on both) says ACO's kernel is bandwidth-bound;
ours is compute-crippled on gfx1151 specifically. This is
**llvm/llvm-project#219248** (AMDGPU packed-integer codegen vs RADV/ACO on
gfx1151, split from #6409 by AMD on 2026-08-27) quantified at 12x on shipped
silicon by lhl's own harness. [INFERENCE: the specific missing forms are the
packed-int dot ops (v_dot4 family / VOPD duals); confirmed only by the
magnitude and arch-specificity, not yet by reading the two ISAs.]

## What this anchors: payload recovery without writing a compiler

Redline dispatches code objects; the payload is orthogonal to the PM4
substrate. Two bounded paths to ACO-grade payloads:

1. **Radiowave recipe (prototype first).** Keep the HIP ABI; recipe-rewrite
   the hot loop with the packed forms hipcc fails to emit on gfx1151.
   Radiowave exists to do certified kernel rewriting; no Vulkan machinery.
2. **SPIR-V payload (general answer).** The suite already ships the .spv
   kernels. amdllpc compiles SPIR-V to GPU ELF today; RADV ISA is extractable
   via pipeline-executable-properties. The bounded work is an ABI adapter
   (user-SGPR/descriptor conventions instead of AQL kernargs) in front of the
   existing PM4 dispatch, which already programs SH registers directly.

Either way the pitch compounds: ACO-grade kernels plus PM4 submission beats
Vulkan on both axes at once, instead of conceding compute-bound rows.

## Next experiments, in order

1. Dump both ISAs for q8_signed on gfx1151 (llvm-objdump on our HSACO;
   RADV_DEBUG=shaders or executable-properties for ACO) and name the exact
   missing instructions — converts the [INFERENCE] above into evidence and
   makes a crisp llvm#219248 datum (posting gated on approval).
2. Radiowave recipe for one variant; target: gfx1151 q8_signed from 3523 us to
   ~289 us territory with certification passing.
3. Only then the ABI-adapter question, sized by what the recipe cannot reach.
