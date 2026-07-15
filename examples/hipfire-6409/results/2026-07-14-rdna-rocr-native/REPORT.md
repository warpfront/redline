# Native ROCr/Redline result across RDNA

The Hipfire-native ROCm issue 6409 matrix now runs through retained direct PM4
on gfx1010, gfx1030, gfx1100, and gfx1151. All **960/960 rows** and all
**3,840 backend executions** passed their CPU correctness oracle.

Across the four devices, Redline takes **537/960 four-way first places
(55.94%)**, finishes first or second in **862/960 (89.79%)**, beats Vulkan
pairwise in **606/960 (63.13%)**, beats HipGraph in **837/960 (87.19%)**, and
beats direct HIP in **874/960 (91.04%)**.

## Per-device result

| Architecture | Device | RL 1st | RL top two | RL > Vulkan | RL > HipGraph | RL > HIP | Correct rows |
|---|---|---:|---:|---:|---:|---:|---:|
| gfx1100 | RX 7900 XTX | 158/240 (65.83%) | 224/240 (93.33%) | 171/240 (71.25%) | 225/240 (93.75%) | 216/240 (90.00%) | 240/240 |
| gfx1151 | Radeon 8060S | 156/240 (65.00%) | 229/240 (95.42%) | 156/240 (65.00%) | 239/240 (99.58%) | 229/240 (95.42%) | 240/240 |
| gfx1030 | RX 6950 XT | 94/240 (39.17%) | 199/240 (82.92%) | 125/240 (52.08%) | 175/240 (72.92%) | 214/240 (89.17%) | 240/240 |
| gfx1010 | RX 5700 XT | 129/240 (53.75%) | 210/240 (87.50%) | 154/240 (64.17%) | 198/240 (82.50%) | 215/240 (89.58%) | 240/240 |
| **All** | **Four GPUs** | **537/960 (55.94%)** | **862/960 (89.79%)** | **606/960 (63.13%)** | **837/960 (87.19%)** | **874/960 (91.04%)** | **960/960** |

## Aggregate placement

| Backend | 1st | 2nd | 3rd | 4th | N |
|---|---:|---:|---:|---:|---:|
| Redline | 537 | 325 | 56 | 42 | 960 |
| Vulkan | 330 | 400 | 20 | 210 | 960 |
| HipGraph | 77 | 168 | 322 | 393 | 960 |
| HIP | 16 | 67 | 562 | 315 | 960 |

Redline takes 259/480 serial-RMW firsts and 278/480 independent-throughput
firsts. The complete row-level loss attribution remains in each architecture's
generated report: [gfx1100](../gfx1100/2026-07-14-rdna-rocr-native/REPORT.md),
[gfx1151](../gfx1151/2026-07-14-rdna-rocr-native/REPORT.md),
[gfx1030](../gfx1030/2026-07-14-rdna-rocr-native/REPORT.md), and
[gfx1010](../gfx1010/2026-07-14-rdna-rocr-native/REPORT.md).

## What this establishes

This is direct evidence that the retained Redline command stream is not a
gfx1201-only accident. The legacy GFX10/GFX11 encoder loads ordinary
Radiowave/hipcc HSA code objects, derives the loader-resolved program resources,
encodes kernarg user SGPRs and static/dynamic LDS, and submits one retained PM4
IB through the public ROCr vendor packet. Vulkan is used only as an independent
comparison backend and is pinned to the same physical GPU by PCI identity.

The result separates submission from codegen cleanly. Redline exceeds Vulkan
pairwise on every tested architecture, but the lower gfx1030 first-place rate
also shows that untuned per-architecture kernel geometry still matters. No
non-gfx1201 Radiowave recipe was promoted for this run: those devices use the
architecture-safe baseline plan. The remaining losses are therefore tuning
work, not evidence that ROCr-native retained PM4 is unavailable.

gfx1010 exposed one useful fail-closed check during bring-up: LLVM initially
spilled `dot_q6` into a 92-byte private segment, which the zero-scratch direct
backend rejected. Its target-specific low-register B32 load loop reduces the
kernel to 17 VGPRs with zero private segment while preserving the same packed
dot operation. The final 240-row run contains no rejected or incorrect row.

## Protocol

- Hipfire `--matrix hipengine`: 120 serial plus 120 independent rows per GPU.
- Three warmups and seven measured GPU samples per backend and row.
- HIP, HipGraph, and Redline load the same per-row HSACO; Vulkan runs matched
  GLSL through RADV.
- Redline uses the Radiowave-certified VMEM boundary where certified and the
  broader same-agent boundary otherwise.
- ROCm 7.2.2 / hipcc 7.2.53211, Mesa/RADV 25.2.8, shaderc 2026.1.
- Rebuilding the current source reproduces every measured wave32 HSACO SHA-256.

The [machine-readable aggregate](aggregate.json) records the exact hashes,
PCI identities, placements, and pairwise totals.
