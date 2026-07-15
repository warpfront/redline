<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# Exact HipEngine VOPD artifact through the Hipfire Rust harness

## Outcome

The Hipfire Rust harness loads the exact hash-certified HipEngine
`hip_vopd_sweep.redline.co` artifacts and passes the original CPU oracle for
all 48 measured combinations: 16 operation/mode/workgroup rows under three
replay policies.

Against a same-session, saturation-warmed run of HipEngine's C-ABI adapter,
the bytecode-identical Hipfire `hipengine_compatible` path is within **+0.35%**
geometric mean across all 16 rows. It is +0.78% in serial mode and -0.08% in
independent mode. This is replay parity, not a distinct performance class.

The safe Hipfire policy, which resets each sample and completes a separate
gfx12 system ownership acquire before the timed retained tape, is within
**+0.77%** geometric mean overall: +1.94% serial and -0.38% independent.
The ownership acquire is therefore not the cause of the old VOPD gap.

Compared with the matched HipEngine Vulkan rows, every exact-artifact Redline
policy wins **16/16**. The safe policy's geometric-mean Redline/Vulkan ratio is
0.5705 (about 43% lower GPU time). The earlier Hipfire-native reproduction won
only 5/16 VOPD rows, so exact HipEngine bytecode restores all 11 missing VOPD
wins. The Vulkan values are the retained matched-suite artifacts rather than a
same-session rerun, but the individual Redline margins are 35-57%, well beyond
the observed clock noise.

## What changed and what did not

- No kernel was recompiled. Each `.redline.co` hash is verified against its
  Radiowave manifest before loading.
- The launch grid, block size, kernarg ABI, hidden arguments, repetition graph,
  and VMEM-certified dependency boundary match the HipEngine adapter.
- `hipengine_compatible` allocates one kernarg per recorded dispatch, matching
  the C ABI.
- `hipfire_reuse_hot` reuses the identical serial kernarg address and shortens
  a ten-dispatch tape from 199 to 172 dwords.
- `hipfire_safe` adds no commands to the timed tape; ownership transfer is a
  separate completed IB.

Kernarg reuse is not universally beneficial for these kernels. It is neutral
in independent mode, where output pointers necessarily differ, but regresses
the WG64 serial `independent_fma` and `dependent_fma` rows by roughly 3-6% in
this run. It should remain a selected optimization rather than a blanket
policy.

## Clock-control finding

The unlocked gfx1201 APU changes clocks enough to invalidate short-warmup
cross-run comparisons. The same exact `independent_fma` WG64 serial artifact
moved from about 148 us to 86 us and then 71 us as the GPU ramped, without any
code or PM4 change. The final result therefore uses 1,000 retained-tape
preheat replays followed by 1,000 per-policy warmups and 21 measured samples.

Machine-readable Hipfire results:
[`2026-07-13-hipengine-exact-vopd-saturated.json`](2026-07-13-hipengine-exact-vopd-saturated.json).
