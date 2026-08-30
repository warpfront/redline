<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# gfx1151 packed-dot 12x: NOT codegen. Identical ISA, 76 vs 928 GB/s.

**Status: internal. NOT posted upstream.**

## Correction notice

The first version of this document attributed the gap to kernel codegen and
claimed it "quantifies llvm/llvm-project#219248 at 12x on shipped silicon."
**That attribution is falsified** by its own next-listed experiment, executed
30 minutes later: the gfx1151 and gfx1100 objects are instruction-identical
(240 lines, v_dot4_i32_i8 x16, 32 global_load, 19 s_waitcnt, 27 s_delay_alu,
zero spills, zero scratch on both). Identical ISA cannot explain 330 vs
3523 us. The [INFERENCE] tag on the original claim did its job: the inference
was scoped, tested, and killed. This project's 12x datum says nothing about
llvm#219248 either way.

## What the numbers actually say

q8_signed, wg64, serial, 256 MiB nominal bytes/dispatch (4 MiB working set
re-read 64 times by body_iters):

| part | backend | median | achieved bandwidth |
| --- | --- | ---: | ---: |
| gfx1151 | hip | 3535.87 us | **75.9 GB/s** |
| gfx1151 | vulkan | 289.11 us | **928.5 GB/s** |
| gfx1100 | hip | 332.63 us | 807.0 GB/s |
| gfx1100 | vulkan | 289.20 us | 928.2 GB/s |

928 GB/s is far above LPDDR5X DRAM rates: the 4 MiB working set is
cache-resident under Vulkan (and under HIP on the gfx1100 dGPU). On gfx1151 the
HIP path streams at 76 GB/s — DRAM-or-worse, consistent with an **uncached
mapping**, not a slow kernel.

## Leading hypothesis (untested)

Allocation coherence/placement policy on the APU: hipMalloc on gfx1151
producing a fine-grained-coherent (effectively uncached-to-GPU) mapping for
this buffer, where RADV allocates DEVICE_LOCAL and gets full cache residency.
Other gfx1151 families are unaffected (redline's overall median vs Vulkan is
0.601 there), so it is buffer- or size-conditional, not global.

Discriminating experiments, cheap to run:
1. Same kernel, buffer via `hipExtMallocWithFlags` coarse-grain /
   `hipDeviceMallocDefault` vs `hipMallocManaged` vs `hipHostMalloc`, sweep
   4-512 MiB: if a flag recovers ~900 GB/s, it is allocation policy.
2. `hipMemAdvise` coarse-grain on the existing allocation.
3. Check what the suite's runner actually allocates with (hipMalloc?) before
   blaming the runtime.

## Why this matters more than the codegen story

gfx1151 (Strix Halo) is the laptop inference target. If bandwidth-bound quant
kernels can silently run 12x under hardware capability depending on allocation
flags, that is worth an upstream issue with a minimal reproducer — a far
stronger and more actionable artifact than a codegen complaint, and it is
mechanically checkable. All posting gated on approval as usual.

The SPIR-V/ACO payload idea from the first version loses this specific anchor.
It remains architecturally sound (payload is orthogonal to the PM4 substrate)
for wherever real codegen gaps exist, but this dataset no longer demonstrates
one.

## Error ledger note

This is the session's sixth wrong attribution caught before posting, and the
fastest kill yet (30 minutes, own experiment). The pattern remains "attribute
first, measure second"; the countermeasure that worked was writing the
discriminating experiment into the artifact at claim time.

## Addendum: allocation-policy hypothesis also eliminated (same night)

`bench/dispatch/apu_membw.cpp`, matched pairs on gfx1151:

- Plain hipMalloc reads reach **868-877 GB/s** at 16 MiB (wave32 AND wave64,
  7.14 AND 10.0) — the runtime, allocation path, and cache hierarchy are fine,
  and there is no 7.14 -> 10.0 regression.
- hipHostMalloc reads run at **57-62 GB/s** — the hipEngine signature — but the
  harness verifiably calls plain hipMalloc.
- XNACK is disabled in the recorded environment. The harness objects contain
  zero cache-bypass modifiers (32x bare global_load_b32).

Eliminated tonight: codegen (identical ISA), allocation policy (hipMalloc is
fast), wave64 (877 GB/s), XNACK (off), cache-bypass bits (none), ROCm version
(identical both ways).

**Remaining leading hypothesis: dispatch geometry.** The family runs
groups=16 x wg64 = 1,024 threads on the whole chip (~0.4 workgroups/CU on
gfx1151). At that occupancy the loop is memory-latency-bound, and APU LPDDR
latency is where it hurts; gfx1100's dGPU memory system tolerates it (807
GB/s equivalent). The Vulkan shader is different code and may extract more
parallelism per thread. If this holds, the 12x is substantially a
benchmark-shape artifact at pathological occupancy — which matters upstream,
because hipEngine feeds #6409's Vulkan-vs-HIP narrative and gfx1151 rows would
overstate the HIP deficit on realistic workloads.

Decisive probe, not yet run: replicate the exact geometry (16 x 64, b32
dependent loads, 4 MiB set) in a dependency-free pair (HIP + trivial Vulkan
twin), sweep workgroup count 16 -> 2048 on both parts. If the HIP/Vulkan gap
collapses as occupancy rises, shape is the story; if it persists at full
occupancy, something real remains.
