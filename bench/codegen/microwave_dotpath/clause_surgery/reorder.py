#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
"""Extract A_control.s from hipcc --save-temps and produce B/C/D.

B hoists every address VALU ahead of the 32 loads, assigns a fresh VGPR
pair per load (v32..v95), then emits the loads back-to-back. C is B
without the s_clause 0x1f. D keeps LLVM's interleaved order and wraps
each already-adjacent load run in s_clause.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

KERNEL = "_Z15dot_path_kernelPKjS0_Pijjjj"

# Dest VGPRs of the 32 loads in original issue order (group 0 weights,
# group 0 activations, ... group 15 activations). Must match A_control.s.
LOAD_DESTS = [
    25, 26, 27, 28, 15, 29, 16, 30, 17, 31, 18, 32, 19, 33, 20, 34,
    21, 35, 22, 36, 23, 37, 24, 38, 39, 40, 1, 41, 13, 9, 5, 6,
]


def tab(s: str) -> str:
    return "\t" + s


def extract_a(full: str) -> str:
    lines = full.splitlines()
    # Header through first kernel, stopping at the next .protected / next
    # function after this kernel's csdata.
    start = 0
    while start < len(lines) and not lines[start].strip().startswith(".amdgcn_target"):
        start += 1
    if start >= len(lines):
        raise SystemExit("no .amdgcn_target in save-temps")

    # End of this kernel: the .text that precedes the next .protected, or
    # the line before `.protected _Z18dot_path_kernel_u4`.
    end = None
    seen_kernel = False
    for i, ln in enumerate(lines):
        if i < start:
            continue
        if KERNEL in ln and (".globl" in ln or ".protected" in ln or ln.strip().startswith(KERNEL)):
            seen_kernel = True
        if seen_kernel and i > start + 20 and ".protected" in ln and KERNEL not in ln:
            # walk back over blank / .text
            end = i
            while end > 0 and lines[end - 1].strip() in ("", ".text"):
                end -= 1
            break
    if end is None:
        raise SystemExit("could not find end of dot_path_kernel")

    body = lines[start:end]

    # Metadata: keep only this kernel's YAML entry.
    meta_start = None
    meta_end = None
    for i, ln in enumerate(lines):
        if ln.strip() == ".amdgpu_metadata":
            meta_start = i
        if ln.strip() == ".end_amdgpu_metadata":
            meta_end = i
            break
    if meta_start is None or meta_end is None:
        raise SystemExit("no amdgpu_metadata block")

    yaml_lines = lines[meta_start + 1 : meta_end]
    # Drop leading/trailing `---` / `...` for parsing, re-add later.
    text = "\n".join(yaml_lines)
    # Split kernel list on "  - .args:" at beginning of entries.
    m = re.search(r"amdhsa\.kernels:\n", text)
    if not m:
        raise SystemExit("no amdhsa.kernels")
    rest = text[m.end() :]
    entries = re.split(r"\n(?=  - \.args:)", rest)
    wanted = None
    tail = None
    for ent in entries:
        if f".name:           {KERNEL}" in ent or f".name: {KERNEL}" in ent:
            # strip anything after amdhsa.target if it got glued
            if "\namdhsa.target:" in ent:
                ent, after = ent.split("\namdhsa.target:", 1)
                tail = "amdhsa.target:" + after
            wanted = ent.rstrip()
            break
    if wanted is None:
        raise SystemExit("kernel not in metadata")
    if tail is None:
        # target/version live after the list
        idx = text.find("amdhsa.target:")
        if idx < 0:
            raise SystemExit("no amdhsa.target")
        tail = text[idx:]

    ident = None
    for ln in lines:
        if ln.strip().startswith(".ident"):
            ident = ln
            break

    out = []
    out.extend(body)
    if body and body[-1].strip():
        out.append("")
    if ident:
        out.append(ident)
    out.append('\t.section	".note.GNU-stack","",@progbits')
    out.append("\t.amdgpu_metadata")
    out.append("---")
    out.append("amdhsa.kernels:")
    out.append(wanted)
    out.append(tail.rstrip())
    if not tail.rstrip().endswith("..."):
        out.append("...")
    out.append("")
    out.append("\t.end_amdgpu_metadata")
    out.append("")
    return "\n".join(out)


def find_loop(asm: str) -> tuple[int, int, list[str]]:
    """Return (start_idx, end_idx, lines) of the inner loop body.

    start is the `.LBB0_3:` line, end is exclusive at `s_cbranch_scc1 .LBB0_3`.
    """
    lines = asm.splitlines()
    start = None
    end = None
    for i, ln in enumerate(lines):
        if ln.startswith(".LBB0_3:"):
            start = i
        if start is not None and "s_cbranch_scc1 .LBB0_3" in ln:
            end = i + 1
            break
    if start is None or end is None:
        raise SystemExit("could not find .LBB0_3 loop")
    return start, end, lines


def split_loop_tail(loop_lines: list[str]) -> tuple[list[str], list[str]]:
    """Head = up through last global_load_b32; tail = v3 bump / wait / dots / branch."""
    last_load = None
    for i, ln in enumerate(loop_lines):
        if "global_load_b32" in ln:
            last_load = i
    if last_load is None:
        raise SystemExit("no global_load_b32 in loop")
    return loop_lines[: last_load + 1], loop_lines[last_load + 1 :]


def make_b_loop(tail: list[str], with_clause: bool) -> list[str]:
    """Hoisted address VALU, then 32 back-to-back loads, then original tail."""
    out: list[str] = []
    out.append(".LBB0_3:                                ; %.lr.ph.i")
    out.append("                                        ; =>This Inner Loop Header: Depth=1")
    out.append(tab("s_delay_alu instid0(VALU_DEP_3)"))
    # 16 groups: offset (v3 + (g-15)) & mask, <<2, + weights base, + acts base.
    # Address pair for load i lives in v[32+2*i : 33+2*i]. Temps: v1 (idx),
    # v[5:6] (byte offset). Live across the hoist: v0 (tid), v2 (=0), v3
    # (cursor), v4 (acc).
    for g in range(16):
        imm = g - 15
        w_lo = 32 + 4 * g
        a_lo = w_lo + 2
        out.append(tab(f"v_add_nc_u32_e32 v1, {imm}, v3"))
        out.append(tab("s_delay_alu instid0(VALU_DEP_1)"))
        out.append(tab("v_and_b32_e32 v1, s6, v1"))
        out.append(tab("s_delay_alu instid0(VALU_DEP_1)"))
        out.append(tab("v_lshlrev_b64 v[5:6], 2, v[1:2]"))
        if g == 0:
            # Same placement as LLVM: wait for the pointer s_loads before
            # the first v_add_co that consumes s[8:11].
            out.append(tab("s_waitcnt lgkmcnt(0)"))
        out.append(tab("s_delay_alu instid0(VALU_DEP_1)"))
        out.append(tab(f"v_add_co_u32 v{w_lo}, vcc_lo, s8, v5"))
        out.append(tab("s_delay_alu instid0(VALU_DEP_1)"))
        out.append(tab(f"v_add_co_ci_u32_e64 v{w_lo + 1}, null, s9, v6, vcc_lo"))
        out.append(tab(f"v_add_co_u32 v{a_lo}, vcc_lo, s10, v5"))
        out.append(tab("s_delay_alu instid0(VALU_DEP_1)"))
        out.append(tab(f"v_add_co_ci_u32_e64 v{a_lo + 1}, null, s11, v6, vcc_lo"))
    if with_clause:
        out.append(tab("s_clause 0x1f"))
    for i, dst in enumerate(LOAD_DESTS):
        lo = 32 + 2 * i
        out.append(tab(f"global_load_b32 v{dst}, v[{lo}:{lo + 1}], off"))
    out.extend(tail)
    return out


def bump_vgpr(asm: str, n: int) -> str:
    asm = re.sub(
        r"(\.amdhsa_next_free_vgpr )42",
        rf"\g<1>{n}",
        asm,
        count=1,
    )
    asm = re.sub(
        rf"(\.L{re.escape(KERNEL)}\.num_vgpr, )42",
        rf"\g<1>{n}",
        asm,
        count=1,
    )
    asm = re.sub(
        r"(NumVgprs: )42",
        rf"\g<1>{n}",
        asm,
        count=1,
    )
    asm = re.sub(
        r"(NumVGPRsForWavesPerEU: )42",
        rf"\g<1>{n}",
        asm,
        count=1,
    )
    asm = re.sub(
        r"(VGPRBlocks: )5",
        r"\g<1>11",
        asm,
        count=1,
    )
    asm = re.sub(
        r"(\.vgpr_count:     )42",
        rf"\g<1>{n}",
        asm,
        count=1,
    )
    return asm


def make_d_loop(head: list[str], tail: list[str]) -> list[str]:
    """Keep interleaved order; wrap each run of consecutive loads in s_clause."""
    out: list[str] = []
    i = 0
    while i < len(head):
        ln = head[i]
        if "global_load_b32" in ln:
            j = i
            while j < len(head) and "global_load_b32" in head[j]:
                j += 1
            n = j - i
            if n >= 2:
                out.append(tab(f"s_clause {hex(n - 1)}"))
            out.extend(head[i:j])
            i = j
            continue
        out.append(ln)
        i += 1
    out.extend(tail)
    return out


def replace_loop(lines: list[str], start: int, end: int, new_loop: list[str]) -> str:
    return "\n".join(lines[:start] + new_loop + lines[end:]) + "\n"


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--full", type=Path, help="hipcc --save-temps device .s")
    ap.add_argument("--a-control", type=Path, help="existing A_control.s (skip extract)")
    ap.add_argument("--out-dir", type=Path, required=True)
    args = ap.parse_args()
    out_dir: Path = args.out_dir
    out_dir.mkdir(parents=True, exist_ok=True)

    if args.a_control:
        a_text = args.a_control.read_text()
    else:
        if not args.full:
            print("need --full or --a-control", file=sys.stderr)
            return 2
        a_text = extract_a(args.full.read_text())
    a_path = out_dir / "A_control.s"
    a_path.write_text(a_text if a_text.endswith("\n") else a_text + "\n")

    start, end, lines = find_loop(a_text)
    loop = lines[start:end]
    head, tail = split_loop_tail(loop)

    b_loop = make_b_loop(tail, with_clause=True)
    c_loop = make_b_loop(tail, with_clause=False)
    d_loop = make_d_loop(head, tail)

    b_text = bump_vgpr(replace_loop(lines, start, end, b_loop), 96)
    c_text = bump_vgpr(replace_loop(lines, start, end, c_loop), 96)
    d_text = replace_loop(lines, start, end, d_loop)

    (out_dir / "B_group_clause.s").write_text(b_text)
    (out_dir / "C_group_noclause.s").write_text(c_text)
    (out_dir / "D_clause_only.s").write_text(d_text)

    def nloads(p: Path) -> int:
        return sum(1 for ln in p.read_text().splitlines() if "global_load_b32" in ln)

    print(f"A_control.s loads={nloads(a_path)}")
    print(f"B_group_clause.s loads={nloads(out_dir / 'B_group_clause.s')} vgpr=96 clause=0x1f")
    print(f"C_group_noclause.s loads={nloads(out_dir / 'C_group_noclause.s')} vgpr=96 no clause")
    print(f"D_clause_only.s loads={nloads(out_dir / 'D_clause_only.s')} interleaved clauses")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
