<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# Microwave hipEngine packed-dot: ACO ISA under HIP (2026-09-01)

**Status: internal.** Discriminator for lhl's gfx1151 q8 gap (HIP 3581 us =
75 GB/s vs Vulkan 1122 us = 239 GB/s). hipEngine's per-element shader is
compiled by RADV/ACO, wrapped into an LLVM HSACO, and run under HIP on the
same hipMalloc buffers beside LLVM's own kernel.

Sources: `bench/codegen/microwave_dotpath/`. hipEngine pinned at
f2c3ad6d74c86e3641ce09ff9fd759eaa6cd75e0 (verbatim under `upstream/`).

## Provenance

| | |
| --- | --- |
| host | hipx |
| HIP | `hipRuntimeGetVersion=71526333` (7.15.26333-0000000) |
| libamdhip64 | `/opt/rocm/core-10.0/lib/libamdhip64.so.7` (dladdr) |
| hipcc | `/opt/rocm/core-10.0/bin/hipcc` — AMD clang 23.0.0git `8f497e0992fb7513f7f78a6f6b6f1056c375e961` |
| Mesa / RADV | Mesa 26.0.3-1ubuntu1 |
| glslc | shaderc 2026.1-1 `/usr/bin/glslc` |
| gfx1151 | HIP_VISIBLE_DEVICES=1, `AMD Radeon 8060S Graphics` / `gfx1151`; Vulkan `Radeon 8060S Graphics (RADV STRIX_HALO)` idx=3; `timestampPeriod=10.019` |
| gfx1100 | HIP_VISIBLE_DEVICES=0, `AMD Radeon RX 7900 XTX` / `gfx1100`; Vulkan idx=0; `timestampPeriod=10` |
| row | n=32768, body_iters=64, groups=16, block=64, data_elems=33554432, data_mask=0x1ffffff, 268435456 bytes/dispatch; 20 chained launches, warmup 3, samples 7, 3 outer repeats, median of medians |

Gate: CPU oracle on idx 0, n-1, and 4096 evenly spaced idx against the final
chain (`sequence_id=19`). All arms **PASS**, mismatches=0.

## Arm × arch (median us / GB/s / gate / VGPRs)

GB/s = 268435456 / median_us / 1000. VGPRs = `hipFuncGetAttributes.numRegs`.
Spread is min–max of the three outer medians. No cell exceeds the device DRAM
peak on this streaming workload (gfx1151 ~256 GB/s, gfx1100 ~960 GB/s).

### gfx1151 (Radeon 8060S)

| arm | wave | median us | min–max us | GB/s | gate | VGPRs |
| --- | ---: | ---: | ---: | ---: | --- | ---: |
| llvm32 | 32 | 3534.2491 | 3533.8585–3534.4074 | **75.95** | PASS | 42 |
| aco32 | 32 | 1237.6472 | 1236.9399–1237.8016 | **216.89** | PASS | 96 |
| llvm128 | 32 | 1250.6076 | 1250.4753–1251.2026 | 214.64 | PASS | 37 |
| llvm64 | 64 | 3559.6107 | 3559.2098–3560.0376 | **75.41** | PASS | 42 |
| aco64 | 64 | 1141.6736 | 1141.6696–1141.7556 | **235.12** | PASS | 48 |
| llvm128 | 64 | 1183.3316 | 1183.3135–1183.3616 | 226.85 | PASS | 37 |
| vulkan orig q8 wg64 | 64 (RADV default) | 1118.2487 | — | **240.05** | PASS | — |

### gfx1100 (RX 7900 XTX)

| arm | wave | median us | min–max us | GB/s | gate | VGPRs |
| --- | ---: | ---: | ---: | ---: | --- | ---: |
| llvm32 | 32 | 332.2025 | 331.8945–332.2045 | 808.05 | PASS | 42 |
| aco32 | 32 | 304.8826 | 304.5707–305.0926 | 880.46 | PASS | 96 |
| llvm128 | 32 | 303.5407 | 303.4487–303.5786 | 884.35 | PASS | 37 |
| llvm64 | 64 | 323.8126 | 323.0105–324.4566 | 828.98 | PASS | 42 |
| aco64 | 64 | 307.4567 | 307.3546–308.0888 | 873.08 | PASS | 48 |
| llvm128 | 64 | 304.8027 | 304.7426–304.8106 | 880.69 | PASS | 37 |

## ISA comparison (gfx1151)

LLVM numbers from hipcc `--save-temps` of the wave32 probe
(`gen/gfx1151/llvm32_dot_path_kernel.s`; `.text` size 0x5dc = 1500 bytes via
`llvm-nm -S`). ACO numbers from RADV pipeline-executable Assembly
(`gen/gfx1151/dot_path_mw_w{32,64}.s`) plus `code_size` from the extract JSON.
`max outstanding` is consecutive non-kernarg loads before the first `s_wait*`
in that burst. RADV's reported `vgprs=12` is Pre-Sched (the extractor last-match
bug); HIP `numRegs` and the max VGPR index in the `.s` are the live counts.

| kernel | `global_load_b32` | `b64` | `b128` | `s_waitcnt` | max outstanding | `s_clause` | VGPRs | code bytes |
| --- | ---: | ---: | ---: | ---: | ---: | --- | ---: | ---: |
| LLVM llvm32 (`dot_path_kernel`) | **32** | 0 | 0 | 19 (16 vmcnt + 3 lgkmcnt) | 32 | 2× `0x1` (not on the 32 loads) | 42 | 1500 |
| LLVM llvm128 (`uint4`) | 0 | 0 | **8** | 7 | 8 | 2× `0x1` | 37 | 644 |
| ACO aco32 extract | 35 (32 loop + 3 kernarg) | 0 | 1 kernarg (+ b96) | 21 (all vmcnt) | **32** | `0x1` kernarg + **`0x1f` (32 loads)** | 96 (parsed max 95) | 1544 |
| ACO aco64 extract | 35 | 0 | 1 kernarg (+ b96) | 22 | **32** | 4 (includes **`0x1f`**) | 48 (parsed max 47) | 1564 |

Both LLVM and ACO emit 16× `v_dot4_i32_iu8` per iteration body. LLVM issues
the same 32 `global_load_b32` before the first `s_waitcnt vmcnt(30)`, then
drip-feeds `vmcnt(28)…(0)` around each dot. ACO wraps those 32 loads in
`s_clause 0x1f`.

### Original `dot_path.comp` vs pointer-ABI port

Compiled original with glslc (`-DHIPENGINE_DOT_MODE=0 -DHIPENGINE_DOT_GROUPS=16
-DHIPENGINE_BLOCK_SIZE=64`) and dumped via `dump_orig.cpp` (20-byte push
const, 3 storage buffers, subgroup 32 and 64). gfx1151 w64: 32× `buffer_load`
(32-bit), 0× `global_load_b*`, 16× `v_dot4`, 18× `s_waitcnt`, 3× `s_clause`,
max outstanding 32, code_size 816. **Same load count and width as the
pointer-ABI ACO body (32× 32-bit loads per iteration); different opcode
(MUBUF `buffer_load` vs GLOBAL `global_load_b32`) because the original uses
a descriptor set.** Could compare; not identical ISA.

## Optional Vulkan cross-check

08-29 runner still on disk:
`/home/kaden/redline-suite-20260829/examples/hipengine-6409/results/gfx1151/2026-08-29-rocm100-bench/build/vulkan/packed-dot/vulkan/q8_signed_g16/wg64/vulkan_dot_path`
with the recorded `--spirv` and harness args, except `--device-index 3` (STRIX_HALO;
idx 0 is now the 7900 XTX). q8 wg64 serial: **1118.2487 us = 240.05 GB/s**,
correctness pass, `gpu_timestamps_supported=true`, `timestampPeriod=10.019`.
Matches lhl's 1122 us = 239 GB/s. The 08-29 289 us / 928 GB/s figure remains a
measurement bug.

## Reading

**Yes: ACO's code for hipEngine's shader runs at DRAM peak under HIP dispatch
on gfx1151, while LLVM's does not.**

Discriminating numbers, same process, same hipMalloc buffers, 20-chain
hipEvent timing: HIP LLVM llvm32 **3534.2491 us = 75.95 GB/s** (llvm64
3559.6107 us = 75.41 GB/s) vs HIP ACO aco64 **1141.6736 us = 235.12 GB/s**
(aco32 1237.6472 us = 216.89 GB/s) vs Vulkan original **1118.2487 us =
240.05 GB/s**. LLVM-with-`uint4` loads (llvm128) is 1250.6076 us = 214.64 GB/s
on wave32 / 1183.3316 us = 226.85 GB/s on wave64 — the width effect on the
LLVM side.

This **does** establish that the 3.2× hipEngine HIP-vs-Vulkan gap on this
row is LLVM codegen of the per-element `global_load_b32` form, not HIP
dispatch or hipMalloc environment: the ACO body, constraint-pinned into an
LLVM HSACO and launched by HIP, lands on the same DRAM-peak plateau as
Vulkan. It **does not** establish a ready-to-land LLVM patch, and it does
not isolate `s_clause 0x1f` as the sole cause — LLVM also has 32 loads in
flight before the first wait. [INFERENCE] the clause packing plus the
drip-feed `s_waitcnt vmcnt` around each dot is the scheduling difference
to inspect for llvm-project#219248. gfx1100 does not show the gap (LLVM
already 808–829 GB/s; ACO 873–880).
