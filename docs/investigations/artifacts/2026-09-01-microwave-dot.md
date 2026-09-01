<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# Microwave packed-dot: ACO bodies inside LLVM HSACOs (2026-09-01)

**Status: internal.** First packed-dot target of Microwave: RADV/ACO ISA for
`dot_q8` / `dot_q4` / `dot_q6` / `dot_scalar` extracted from a pointer-ABI
GLSL clone, wrapped as HIP inline asm, packaged by hipcc, and dispatched
through the existing hipengine harness (HIP + Redline + Vulkan) on identical
rows with the CPU-oracle gate.

Vulkan always runs stock `kernels/hipfire_6409.comp`. HIP/Redline run either
stock `kernels/hipfire_6409.hip` or `HIPFIRE_KERNEL_SET=microwave`.

## Provenance

| arch | GPU | Mesa | HIP / libamdhip64 | notes |
| --- | --- | --- | --- | --- |
| gfx1201 | RX 9070 XT | 25.2.8-0ubuntu0.24.04.2 | ROCm 10.0 `/opt/rocm/core-10.0/lib/libamdhip64.so.7.15.26333-0000000` mixed=false | display GPU; warmups 3 / samples 7 |
| gfx1151 | Radeon 8060S | 26.0.3-1ubuntu1 | same libamdhip64.so.7, mixed=false | hipx; `ROCR_VISIBLE_DEVICES=1` (Redline uses HSA ordinal 0) |
| gfx1100 | RX 7900 XTX | 26.0.3-1ubuntu1 | same | hipx; `ROCR_VISIBLE_DEVICES=0` |

Pinned glslc: Ubuntu 26.04 `glslc` 2026.1-1 + `libshaderc1` dpkg-extracted to
`/tmp/shaderc-2025/root` locally; hipx used `/usr/bin/glslc` 2026.1. Both
compile `dotPacked4x8EXT`.

HSACO SHA-256 (wave32 `default` scheduler) differed microwave vs stock on every
arch, so the microwave objects are the ones the HIP/Redline arms ran:

| arch | microwave wave32 | stock wave32 |
| --- | --- | --- |
| gfx1201 | `a3e2fcf9…501840` | `2b155456…b91326` |
| gfx1151 | `56547510…95236d` | `33232865…3e192c` |
| gfx1100 | `e1ae80c7…cbf9ec` | `2f414512…7b81d1` |

Selftest (`kernels/microwave/selftest.hip`): PASS on gfx1201, gfx1151, gfx1100
for dot_q8 w32, dot_q8 w64 (`-mwavefrontsize64 -DHIPFIRE_BENCH_WAVE64=1`), and
dot_q6 w32. LLVM `--save-temps` on gfx1201 shows the 4-instruction prologue
`s_load_b32` hidden `blockDim.x` → `s4`, `s_mov_b64 s[2:3], s[0:1]`, then the
ACO body verbatim including `v_dot4_i32_iu8` and `s_endpgm`.

Harness: all **16/16** packed-dot rows gate-passed on both kernel sets × three
arches × {hip, redline, vulkan}. Zero mismatches.

## Median us (CPU-oracle pass on every cell)

Rows: `q8_signed` / `q4_unsigned` / `q6_zero` / `scalar_dequant` × wg 64/256 ×
serial/independent; n=32768, n1=64, groups=16, wave32 (radiowave policy does
not promote packed-dot to wave64). Vulkan medians are from the microwave run
(stock-comp, same SPIR-V); stock-run vulkan agreed to ~1%.

`mw/st` is microwave/stock median for that backend.

### gfx1201 (RX 9070 XT)

| row | stock hip | stock redline | mw hip | mw redline | vulkan | mw/st hip | mw/st redline |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| ser q8 wg64 | 424.09 | 421.64 | 424.83 | 424.00 | 421.77 | 1.002 | 1.006 |
| ser q8 wg256 | 425.34 | 423.75 | 444.94 | 429.94 | 422.21 | 1.046 | 1.015 |
| ser q4 wg64 | 425.12 | 423.47 | 426.76 | 426.62 | 421.62 | 1.004 | 1.007 |
| ser q4 wg256 | 425.00 | 423.93 | 445.02 | 430.04 | 422.50 | 1.047 | 1.014 |
| ser q6 wg64 | 425.05 | 422.87 | 427.00 | 427.46 | 421.64 | 1.005 | 1.011 |
| ser q6 wg256 | 424.93 | 423.83 | 443.84 | 430.15 | 422.02 | 1.044 | 1.015 |
| ser scalar wg64 | 425.60 | 425.45 | 430.92 | 436.38 | 425.55 | 1.012 | 1.026 |
| ser scalar wg256 | 425.58 | 428.98 | 453.69 | 451.74 | 425.86 | 1.066 | 1.053 |
| ind q8 wg64 | 407.38 | 215.80 | 341.50 | 234.10 | 422.63 | 0.838 | 1.085 |
| ind q8 wg256 | 420.89 | 220.06 | 407.60 | 245.29 | 422.23 | 0.968 | 1.115 |
| ind q4 wg64 | 417.00 | 216.00 | 346.12 | 256.70 | 423.17 | 0.830 | 1.188 |
| ind q4 wg256 | 419.34 | 220.34 | 408.52 | 236.77 | 422.56 | 0.974 | 1.075 |
| ind q6 wg64 | 414.88 | 216.62 | 357.32 | 231.53 | 423.04 | 0.861 | 1.069 |
| ind q6 wg256 | 414.65 | 214.24 | 410.96 | 249.89 | 421.86 | 0.991 | 1.166 |
| ind scalar wg64 | 396.45 | 217.18 | 378.12 | 284.21 | 422.71 | 0.954 | 1.309 |
| ind scalar wg256 | 414.32 | 225.34 | 409.27 | 289.14 | 417.49 | 0.988 | 1.283 |

### gfx1151 (Radeon 8060S)

| row | stock hip | stock redline | mw hip | mw redline | vulkan | mw/st hip | mw/st redline |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| ser q8 wg64 | 1209.79 | 1196.33 | 1207.48 | 1198.64 | 1121.64 | 0.998 | 1.002 |
| ser q8 wg256 | 1206.05 | 1193.28 | 1209.16 | 1197.91 | 1121.68 | 1.003 | 1.004 |
| ser q4 wg64 | 1201.32 | 1187.18 | 1208.33 | 1202.67 | 1121.62 | 1.006 | 1.013 |
| ser q4 wg256 | 1196.14 | 1185.33 | 1207.59 | 1197.77 | 1121.46 | 1.010 | 1.010 |
| ser q6 wg64 | 1213.61 | 1200.53 | 1208.62 | 1197.07 | 1119.50 | 0.996 | 0.997 |
| ser q6 wg256 | 1207.83 | 1197.03 | 1208.03 | 1200.16 | 1122.14 | 1.000 | 1.003 |
| ser scalar wg64 | 1246.58 | 1242.67 | 1209.56 | 1197.96 | 1143.71 | 0.970 | 0.964 |
| ser scalar wg256 | 1232.55 | 1219.04 | 1214.12 | 1201.92 | 1143.91 | 0.985 | 0.986 |
| ind q8 wg64 | 950.89 | 976.78 | 812.49 | 901.44 | 654.73 | 0.854 | 0.923 |
| ind q8 wg256 | 1048.48 | 988.44 | 757.15 | 811.82 | 714.51 | 0.722 | 0.821 |
| ind q4 wg64 | 728.85 | 967.19 | 800.01 | 893.52 | 781.52 | 1.098 | 0.924 |
| ind q4 wg256 | 855.30 | 940.89 | 770.24 | 819.06 | 767.47 | 0.901 | 0.871 |
| ind q6 wg64 | 827.68 | 904.94 | 806.11 | 834.71 | 731.87 | 0.974 | 0.922 |
| ind q6 wg256 | 873.48 | 1048.89 | 761.68 | 772.58 | 770.20 | 0.872 | 0.737 |
| ind scalar wg64 | 850.43 | 1013.82 | 814.74 | 810.39 | 751.96 | 0.958 | 0.799 |
| ind scalar wg256 | 932.97 | 1059.41 | 798.15 | 955.31 | 721.57 | 0.856 | 0.902 |

### gfx1100 (RX 7900 XTX)

| row | stock hip | stock redline | mw hip | mw redline | vulkan | mw/st hip | mw/st redline |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| ser q8 wg64 | 303.68 | 290.30 | 298.51 | 452.46 | 286.76 | 0.983 | 1.559 |
| ser q8 wg256 | 301.43 | 289.66 | 298.92 | 452.60 | 285.47 | 0.992 | 1.563 |
| ser q4 wg64 | 301.57 | 289.80 | 296.24 | 455.25 | 286.28 | 0.982 | 1.571 |
| ser q4 wg256 | 301.59 | 290.61 | 298.17 | 452.53 | 286.26 | 0.989 | 1.557 |
| ser q6 wg64 | 301.00 | 290.14 | 300.81 | 455.25 | 287.15 | 0.999 | 1.569 |
| ser q6 wg256 | 301.57 | 289.90 | 301.70 | 452.71 | 285.98 | 1.000 | 1.562 |
| ser scalar wg64 | 301.96 | 289.00 | 305.98 | 292.49 | 295.51 | 1.013 | 1.012 |
| ser scalar wg256 | 302.22 | 290.42 | 305.20 | 452.48 | 292.87 | 1.010 | 1.558 |
| ind q8 wg64 | 248.00 | 193.91 | 184.55 | 177.68 | 236.29 | 0.744 | 0.916 |
| ind q8 wg256 | 257.27 | 241.14 | 161.87 | 192.10 | 275.13 | 0.629 | 0.797 |
| ind q4 wg64 | 252.00 | 235.87 | 198.83 | 198.68 | 263.39 | 0.789 | 0.842 |
| ind q4 wg256 | 260.27 | 237.72 | 164.13 | 200.02 | 275.00 | 0.631 | 0.841 |
| ind q6 wg64 | 264.65 | 195.98 | 214.55 | 196.33 | 233.22 | 0.811 | 1.002 |
| ind q6 wg256 | 262.26 | 218.58 | 169.59 | 209.56 | 281.22 | 0.647 | 0.959 |
| ind scalar wg64 | 265.41 | 209.50 | 188.51 | 252.57 | 273.27 | 0.710 | 1.206 |
| ind scalar wg256 | 265.11 | 226.96 | 178.53 | 226.90 | 308.77 | 0.673 | 1.000 |

## Codegen: ACO extract vs LLVM stock

ACO bodies: zero scratch, zero spills, zero LDS. q8/q4/q6 contain
`v_dot4_i32_iu8` (16× q8/q4, 32× q6); scalar has none. gfx12 ABI is s[2:3]
kernarg, s4 wg_size, ttmp9 tgid, v0 tid. gfx11 ABI is the same with tgid in
**s5** (`s_mul_i32 s5, s5, s4`). gfx12 waits are `s_wait_loadcnt`; gfx11 waits
are `s_waitcnt vmcnt`. Loads are `global_load_b128` (loop) plus `b96`/`b32`
kernarg.

RADV's reported `sgprs` is an allocation constant (128 on gfx12, 108 on gfx11).
`wrap.py` clobbers from the max v/s index in the `.s` (s0–s105 only).

| kernel | wave | ACO vgpr (RADV) | ACO code bytes | ACO inst | ACO waits | LLVM vgpr | LLVM sgpr | LLVM static inst | LLVM waits | LLVM loads |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| gfx1201 q8 | 32 | 37 | 780 | 134 | 20 | 35 | 16 | 86 | 18 | 9× buffer (MUBUF), 0 global |
| gfx1201 q4 | 32 | 37 | 780 | 134 | 20 | 35 | 16 | 86 | 18 | same |
| gfx1201 q6 | 32 | 38 | 1128 | 196 | 21 | 35 | 16 | 113 | 14 | same |
| gfx1201 scalar | 32 | 41 | 3688 | 624 | 87 | 40 | 16 | 357 | (higher) | same |
| gfx1151/gfx1100 q8 | 32 | 36 | 696 | 127 | 10 | [INFERENCE] similar MUBUF stock | | | | |
| gfx1151/gfx1100 q6 | 32 | 37 | 1036 | 187 | 10 | | | | | |

LLVM numbers are Radiowave inspection of the gfx1201 stock wave32 HSACO
(`HIPFIRE_BENCH_DOT8=1`). ACO uses flat/global pointer arithmetic; stock HIP
uses Radiowave `buffer_load_u32x4` (MUBUF). Both emit packed `v_dot4*` for
q8/q4/q6. ACO's extra static instruction count is mostly address math +
`s_delay_alu` around the 8× B128 clause, not extra dots.

## Does ACO beat LLVM on the same dispatch path?

**No, not as a packed-dot codegen story.**

- Serial rows: this harness launches n=32768 work-items (512 workgroups of
  64, or 128 of 256) and runs ~1200 us on gfx1151 for every backend, so the
  family is bandwidth/latency-bound at full occupancy, not occupancy-starved.
  Microwave/stock HIP ratios cluster at **1.00±0.05** on gfx1201 and gfx1151
  serial, and **~0.99** on gfx1100 serial HIP.
- Independent throughput is where Microwave HIP can win: gfx1100 mw/st hip
  **0.63–0.81** on the packed q8/q4/q6 wg256 rows; gfx1201 **0.83–0.99**;
  gfx1151 mixed **0.72–1.10**. That is consistent with a different memory
  instruction mix (GLOBAL B128 vs MUBUF) under multi-queue traffic, not with
  a better dot instruction — both already use `v_dot4*`.
- gfx1100 microwave **Redline serial** is the one clear anomaly: **1.56×**
  vs stock redline on q8/q4/q6 (452 vs 290 us) while HIP stays at parity.
  **A Redline-side artifact on this arch, not a property of the ACO body.**
  Three back-to-back reruns (same build, same box, redline pci 0000:66:00.0,
  HIP ordinal 0; raw JSON on hipx `/tmp/mw1100/run{1,2,3}.json`) give a
  bimodal, quantised picture: 21 of 24 serial row×run cells sit at
  452.37–452.54 us with p95−min under 0.3 us; the exceptions are q8 wg64 run 1
  at 294.8 (runs 2–3 back at 452.4), q4 wg256 run 3 at 442.3, and scalar wg64
  at 296.8–297.2 in all three runs. HIP medians for the same rows are
  300.3–309.4 us in every run. The plateau is independent of kernel, wg and
  run, and 452−300 ≈ 150 us over a 10-dispatch serial chain is a fixed
  ~15 us per dependency edge — the size of a full L2/MALL writeback+invalidate
  on this dGPU. `redline_dependency_cache_policies` reports the same
  `certified_vector_l1_0x00300` for stock and microwave, so if that is the
  mechanism the certification string and the emitted fence disagree for a
  code object whose loads/stores are FLAT-global rather than MUBUF.
  [INFERENCE] until the retained IB is dumped. Discriminator: replay the stock
  and microwave HSACOs through Redline with per-dispatch timestamps and read
  the inter-dispatch gap; then diff the emitted ACQUIRE_MEM/RELEASE_MEM
  between the two IBs. gfx1151 and gfx1201 do not show it; independent-mode
  redline on gfx1100 does not show it. Not a Microwave blocker, a Redline bug
  to file internally.
- **What this says about the gfx1151 "12×".** Two corrections first, both
  from reading the hipEngine sources at the pinned commit
  (`hip_dot_path.hip:171-174,285-337`, `vulkan_dot_path.cpp:177-179,818`)
  rather than from memory. (1) The hipEngine packed-dot workload is the SAME
  shape as this suite's rows: n=32768 work-items in 512 workgroups of 64,
  body_iters=64, 16 groups per iteration, `data_mask = 2^25−1`, i.e. 128 MiB
  per buffer and 256 MiB streamed per dispatch. The "16 groups × wg64 =
  1,024 threads / 4 MiB working set" geometry stated in
  `2026-08-29-gfx1151-packed-dot-codegen.md` and in the previous revision of
  this paragraph was a misreading of `groups=16` (packed groups per
  iteration, not workgroups). (2) The 08-29 hipEngine gfx1151 Vulkan datum,
  289 us = 928 GB/s for 256 MiB, is physically impossible on Strix Halo
  (256-bit LPDDR5X, ~256 GB/s peak) and must be a runner/timing artifact on
  that box; lhl's own Vulkan figure for the identical row is 1122 us
  (#6409 table), and this suite's Vulkan is 1121 us — both at DRAM peak. So
  the real hipEngine-shaped gap is lhl's **3.2×** (HIP 3581 us = 75 GB/s vs
  Vulkan 1122 us = 239 GB/s), not 12×.
  With the geometry identical, the one thing that differs between the two
  HIP kernels is load width: hipEngine's kernel indexes `(base+group) & mask`
  per element and LLVM emits 32 bare `global_load_b32` per iteration; this
  suite's `dot_body` uses Radiowave `buffer_load_b128` (8 per iteration) and
  reaches DRAM peak on gfx1151 (1210 us). Vulkan/ACO reaches DRAM peak with
  the per-element form. That is a codegen question after all — for narrow
  loads on the APU — and exactly the shader llvm#219248 is about. Microwave
  turns it into a one-process experiment: hipEngine's shader through ACO,
  wrapped, run under HIP beside LLVM's kernel on the same buffers. If the ACO
  b32 body runs at ~1120 us under HIP, the 3.2× is LLVM's scheduling of
  narrow loads (clauses, `s_waitcnt` placement, loads in flight) and the ISA
  diff is the patch spec; if it runs at ~3500 us, it is environment.
  Secondary: on gfx1151 the same ACO b128 body runs 1207 us under HIP,
  1198 under Redline, ~1121 under Vulkan — a 7% residual with codegen and
  launch removed, which points at memory placement (hipMalloc pools and page
  attributes vs RADV's heap); the VMM-handle → dma-buf → Vulkan import swap
  is the discriminator for that one.

What Microwave *does* prove: an ACO-emitted CS with a 3-dword inlined push
constant ABI can be constraint-pinned into an LLVM-built HSACO (`s[2:3]`, `s4`,
`s5` or ttmp9, `v0`), launched by HIP and by Redline PM4, and pass the same
CPU oracle as stock LLVM and stock Vulkan. The harness already had that
dispatch path; the missing piece was a real `GL_EXT_integer_dot_product`
toolchain and clobbers parsed from the `.s` rather than RADV's sgprs=128.
