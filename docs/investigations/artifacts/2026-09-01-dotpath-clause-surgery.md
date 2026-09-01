<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# Clause surgery: s_clause vs load grouping on gfx1151 packed-dot (2026-09-01)

**Status: internal.** Follow-up to `2026-09-01-microwave-dotpath.md`. Isolates
whether LLVM's 3.2x gfx1151 gap vs ACO is issue grouping (loads back-to-back),
the `s_clause` directive, or neither. Method: extract LLVM's own
`dot_path_kernel` from hipcc `--save-temps`, hand-reorder the inner loop,
reassemble, `hipModuleLoad`.

Sources: `bench/codegen/microwave_dotpath/clause_surgery/`. Kernel symbol
`_Z15dot_path_kernelPKjS0_Pijjjj`, wave32. Transform is `reorder.py`
(not a one-off edit).

## Provenance

| | |
| --- | --- |
| host | hipx |
| HIP | `hipRuntimeGetVersion=71526333` (7.15.26333-0000000) |
| libamdhip64 | `/opt/rocm/core-10.0/lib/libamdhip64.so.7` (dladdr) |
| assembler | `/opt/rocm/core-10.0/lib/llvm/bin/clang` `-x assembler -target amdgcn-amd-amdhsa -mcpu=<arch> -mcode-object-version=6`; `ld.lld -shared` |
| gfx1151 | HIP_VISIBLE_DEVICES=1, `AMD Radeon 8060S Graphics` / `gfx1151` |
| gfx1100 | HIP_VISIBLE_DEVICES=0, `AMD Radeon RX 7900 XTX` / `gfx1100` (same `.s`, retargeted `.amdgcn_target` at assemble) |
| row | n=32768, body_iters=64, groups=16, block=64, data_elems=33554432, data_mask=0x1ffffff, 268435456 bytes/dispatch; 20 chained launches, warmup 3, samples 7, 3 outer repeats, median of medians |
| source commit | `b7d32696e012487b8270993ae5491d405e63acb4` |

Gate: CPU oracle on idx 0, n-1, and 4096 evenly spaced idx against the final
chain (`sequence_id=19`). All eight cells **PASS**, mismatches=0.

GB/s = 268435456 / median_us / 1000. VGPRs = `HIP_FUNC_ATTRIBUTE_NUM_REGS`.

## Arms

| arm | loop body |
| --- | --- |
| A_control | untouched LLVM extract, reassembled |
| B_group_clause | all address VALU hoisted; 32 `global_load_b32` back-to-back under `s_clause 0x1f`; dest VGPRs unchanged; address pairs renamed to v32..v95; `.amdhsa_next_free_vgpr` 96 |
| C_group_noclause | identical to B minus the `s_clause 0x1f` line |
| D_clause_only | LLVM interleaved order; `s_clause` over each already-adjacent load run (nine `0x1` = 2 loads, one `0x3` = 4 loads) |

llvm-objdump of the gfx1151 objects: A has 32 loads / 2× `s_clause 0x1` (kernarg only), interleaved VALU between loads; B has `s_clause 0x1f` immediately followed by 32 `global_load_b32`; C has max consecutive loads = 32 and no loop clause; D has 11 `s_clause` (2 kernarg + 9 loop).

## Arm × arch

### gfx1151 (Radeon 8060S)

| arm | median us | min–max us | GB/s | gate | VGPRs |
| --- | ---: | ---: | ---: | --- | ---: |
| A_control | 3533.8367 | 3533.4717–3535.1528 | **75.96** | PASS | 42 |
| B_group_clause | 1275.9478 | 1273.0776–1295.9926 | **210.38** | PASS | 96 |
| C_group_noclause | 3243.7962 | 3243.6550–3245.3526 | **82.75** | PASS | 96 |
| D_clause_only | 3212.7655 | 3212.3947–3213.3183 | **83.55** | PASS | 42 |

A_control vs the hipcc `llvm32` probe (3534.2491 us): 0.012% — reassembly is
the same kernel. B vs the probe's `aco32` (1237.6472 us / 216.89 GB/s): B is
3.1% slower, on the same DRAM-peak plateau.

C vs B is a one-instruction delta (the `s_clause 0x1f` line) at the same 96
VGPRs: 3243.7962 us vs 1275.9478 us = **2.54×**.

### gfx1100 (RX 7900 XTX)

| arm | median us | min–max us | GB/s | gate | VGPRs |
| --- | ---: | ---: | ---: | --- | ---: |
| A_control | 331.0827 | 331.0507–331.1707 | 810.78 | PASS | 42 |
| B_group_clause | 309.3707 | 309.2766–309.4048 | 867.68 | PASS | 96 |
| C_group_noclause | 307.2726 | 307.1887–307.2810 | 873.61 | PASS | 96 |
| D_clause_only | 329.4226 | 329.4028–329.4347 | 814.87 | PASS | 42 |

No 3× cliff. B/C are ~7% faster than A, matching the original probe (llvm32
332.2025 us / aco32 304.8826 us).

## B vs A instruction reorder

Inner loop `.LBB0_3` only. Prologue, epilogue, dest VGPRs of the 32 loads,
and the `s_waitcnt vmcnt(30)` / `v_dot4` / `vmcnt(28)` … `vmcnt(0)` tail are
unchanged.

- A issues the first `global_load_b32` after 25 non-load ops (index math +
  first address pair), then interleaves 2–4 VALU (`v_lshlrev_b64`,
  `v_add_co_u32`, `v_add_co_ci_u32`, `v_and_b32`) between every 1–3 loads.
  Max consecutive loads = 4 (near the tail). Two `s_clause 0x1` live only on
  the kernarg `s_load`s.
- B computes all 16 group offsets and 32 64-bit addresses first (fresh pairs
  `v[32:33]…v[94:95]`), keeps `s_waitcnt lgkmcnt(0)` immediately before the
  first pointer `v_add_co_u32` that consumes `s[8:11]`, then `s_clause 0x1f`
  and 32 consecutive `global_load_b32` into the original dests, then the
  original tail. Extra VGPRs: 42 → 96 (occupancy stays 16).
- C is B with that `s_clause 0x1f` deleted. D leaves A's interleaving and
  only inserts `s_clause` on runs that were already adjacent.

## Reading

**It is the `s_clause` over a back-to-back 32-load burst, not grouping
alone and not `s_clause` on LLVM's existing 2–4 load runs.**

On gfx1151, B (hoist + `s_clause 0x1f`) is 1275.9478 us = 210.38 GB/s;
C (same hoist, no clause, same 96 VGPRs) is 3243.7962 us = 82.75 GB/s;
D (interleaved order, `s_clause` on adjacent runs) is 3212.7655 us =
83.55 GB/s; A is 3533.8367 us = 75.96 GB/s.

This **does** establish that, once the 32 `global_load_b32` are consecutive,
the single `s_clause 0x1f` is the difference between ~83 GB/s and ~210 GB/s
on this row, and that extra VGPRs/occupancy are not the cause (C has them).
It **does not** establish a landable LLVM scheduler patch, does not test
`s_clause 0x1f` in front of interleaved VALU (hardware would not treat that
as a 32-op clause), and does not explain why gfx1100 already clauses
effectively without it. [INFERENCE] gfx1151's load/VMEM issue path needs
the clause packet to burst the 32-wide request; gfx1100 does not.
