<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# Clause surgery on LLVM `dot_path_kernel` (gfx1151 wave32)

Pin whether the gfx1151 3.2x packed-dot gap is **load grouping** (addresses
hoisted, 32 `global_load_b32` back-to-back), the **`s_clause` directive**,
or neither. Method: hand-edit LLVM's own assembly and reassemble so nothing
but instruction order / clause / extra address VGPRs changes.

## Arms

All from the same hipcc `--save-temps` `.s` of `microwave_dotpath_probe.hip`
for gfx1151 wave32, kernel `_Z15dot_path_kernelPKjS0_Pijjjj`.

| arm | transform |
| --- | --- |
| `A_control` | untouched extract, reassembled |
| `B_group_clause` | hoist all address VALU, `s_clause 0x1f`, 32 loads back-to-back |
| `C_group_noclause` | identical to B minus the `s_clause 0x1f` line |
| `D_clause_only` | LLVM interleaved order; `s_clause` over each already-adjacent load run |

## Regeneration

On hipx, ROCm 10.0:

```
./build.sh regen     # hipcc --save-temps, reorder.py, assemble gfx1151+gfx1100
HIP_VISIBLE_DEVICES=1 LD_LIBRARY_PATH=/opt/rocm/core-10.0/lib \
  ./run_hsaco --hsaco A_control.gfx1151.hsaco --arm A_control --json /tmp/out.jsonl
```

`reorder.py` is the reproducible transform. Do not hand-edit the `.s` files.

## Transform (B / C), step by step

LLVM reuses address VGPR pairs across loads (`v[5:6]`, `v[7:8]`, `v[9:10]`,
`v[11:12]`). A pure instruction move is illegal: later loads would read
addresses already overwritten. Hoisting therefore **renames**.

1. Keep the loop header `s_delay_alu instid0(VALU_DEP_3)` (v3 forwarding
   from the preheader / previous iteration).
2. For group `g = 0..15`, compute
   `idx = (v3 + (g-15)) & s6`, `off = idx << 2` in temps `v1` / `v[5:6]`.
3. Place `s_waitcnt lgkmcnt(0)` immediately before the first `v_add_co_u32`
   that consumes the pointer SGPRs `s[8:11]` (same as LLVM: after the first
   group's `v_lshlrev_b64`, before the first pointer add).
4. Write the 64-bit addresses into a dedicated pair per load:
   load `i` (0..31) uses `v[32+2*i : 33+2*i]`. That is `v32..v95`.
   Live across the hoist: `v0` (tid), `v2` (=0, lshl high), `v3` (cursor),
   `v4` (acc). Occupancy stays 16: 96 VGPRs is the gfx11 wave32 cap.
5. Dest VGPRs of the 32 loads are **unchanged** (same 32 names LLVM used,
   in the same issue order). Dest `v32` belongs to load 11; by then load 0's
   address in `v[32:33]` has already been issued, so the overlap is safe.
6. Emit `s_clause 0x1f` (B only) then the 32 `global_load_b32` back-to-back.
7. Keep the original tail: `v3 += s2`, `s5--`, `s_cmp`, then the identical
   `s_waitcnt vmcnt(30)` / `v_dot4` / `vmcnt(28)` ... `vmcnt(0)` sequence
   and `s_cbranch_scc1 .LBB0_3`.
8. Bump `.amdhsa_next_free_vgpr`, `.num_vgpr`, and metadata `.vgpr_count`
   from 42 to 96.

C is B with step 6's `s_clause 0x1f` omitted.

D walks the original loop and, for each run of `n>=2` consecutive
`global_load_b32`, inserts `s_clause (n-1)` immediately before the run.
Single loads get no clause. Dest/address VGPRs and VALU interleaving stay
exactly LLVM's.

## Host

`run_hsaco.hip` `hipModuleLoad`s the HSACO and launches with a 296-byte
kernarg (40-byte explicit args + hidden `block_count_*` / `group_size_x=64`
at offset 0x34). The kernel's `s_load_b32 s3, s[0:1], 0x34` needs that
hidden field; HIP does not append it when `extra[]` is used.

Oracle, fill, and timing match the probe: idx 0, n-1, and 4096 strided idx
against `sequence_id=19`; 20 chained launches, 3 warmups, 7 samples, 3
repeats; GB/s = 268435456 / median_us / 1000.

gfx1100 objects are the same `.s` with `.amdgcn_target` / metadata retargeted
at assemble time (`-mcpu=gfx1100`). ISA used here is gfx11-common.
