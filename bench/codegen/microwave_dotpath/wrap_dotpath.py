#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
"""Turn extracted ACO bodies for dot_path_mw into HIP inline-asm wrappers.

Reads gen/<arch>/dot_path_mw_w<wave>.{s,json} and writes microwave_dotpath_gen.h.
Clobber lists come from the max v/s index in the assembly, not RADV's sgprs
allocation constant. s106+ are not general SGPRs.

Processing (strip hex comments, expand repeats, remap BB labels) is copied
from examples/hipfire-6409/kernels/microwave/wrap.py.
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
GEN_ROOT = HERE / "gen"
OUT_HEADER = HERE / "microwave_dotpath_gen.h"
KERNEL = "dot_path_mw"
SGPR_GENERAL_MAX = 105  # vcc=106/107, ttmp=108..123, m0=124, exec=126/127

ARCH_SPECS = [
    # (arch, tgid lives in s5)
    ("gfx1151", True),
    ("gfx1100", True),
    ("gfx1201", False),
]

V_RANGE = re.compile(r"\bv\[(\d+):(\d+)\]")
V_SINGLE = re.compile(r"\bv(\d+)\b")
S_RANGE = re.compile(r"\bs\[(\d+):(\d+)\]")
S_SINGLE = re.compile(r"\bs(\d+)\b")
BB_RE = re.compile(r"BB(\d+)")
REPEAT_RE = re.compile(r"^\t\(then repeated (\d+) times\)\s*$")
BB_LABEL_RE = re.compile(r"^\s*BB\d+:")


def die(msg: str) -> None:
    print(msg, file=sys.stderr)
    sys.exit(1)


def strip_hex_comment(line: str) -> str:
    if " ;" in line:
        line = line.split(" ;", 1)[0]
    return line.rstrip()


def expand_repeats(lines: list[str]) -> list[str]:
    out: list[str] = []
    for line in lines:
        m = REPEAT_RE.match(line)
        if m:
            if not out:
                die("repeat marker with no previous instruction")
            out.extend([out[-1]] * int(m.group(1)))
        else:
            out.append(line)
    return out


def max_regs(text: str) -> tuple[int, int]:
    max_v = 0
    max_s = -1
    for raw in text.splitlines():
        line = strip_hex_comment(raw)
        for m in V_RANGE.finditer(line):
            max_v = max(max_v, int(m.group(1)), int(m.group(2)))
        for m in V_SINGLE.finditer(line):
            max_v = max(max_v, int(m.group(1)))
        for m in S_RANGE.finditer(line):
            max_s = max(max_s, int(m.group(1)), int(m.group(2)))
        for m in S_SINGLE.finditer(line):
            max_s = max(max_s, int(m.group(1)))
    if max_s > SGPR_GENERAL_MAX:
        max_s = SGPR_GENERAL_MAX
    return max_v, max_s


def process_body(text: str, kernel: str, wave: int, arch: str) -> str:
    lines = text.splitlines()
    start = 0
    for i, line in enumerate(lines):
        if BB_LABEL_RE.match(line):
            start = i
            break
    end_idx = None
    for i, line in enumerate(lines):
        if "s_endpgm" in line:
            end_idx = i
            break
    if end_idx is None:
        die(f"no s_endpgm in {kernel} {arch} w{wave}")
    relevant = expand_repeats(lines[start : end_idx + 1])
    prefix = f".Lmw_{kernel}_{arch}_w{wave}_BB"
    processed = []
    for line in relevant:
        line = strip_hex_comment(line)
        if not line.strip():
            continue
        line = BB_RE.sub(lambda m: prefix + m.group(1), line)
        processed.append(line)
    return "\n".join(processed)


def isa_stats(text: str) -> dict:
    body = process_body(text, KERNEL, 0, "tmp") if "s_endpgm" in text else text
    lines = [strip_hex_comment(l) for l in body.splitlines()]
    counts = {
        "global_load_b32": 0,
        "global_load_b64": 0,
        "global_load_b96": 0,
        "global_load_b128": 0,
        "s_waitcnt": 0,
        "s_wait_loadcnt": 0,
        "s_wait_": 0,
        "s_clause": 0,
        "v_dot4": 0,
    }
    for line in lines:
        if "global_load_b32" in line:
            counts["global_load_b32"] += 1
        if "global_load_b64" in line:
            counts["global_load_b64"] += 1
        if "global_load_b96" in line:
            counts["global_load_b96"] += 1
        if "global_load_b128" in line:
            counts["global_load_b128"] += 1
        if "s_waitcnt" in line:
            counts["s_waitcnt"] += 1
        if "s_wait_loadcnt" in line or "s_wait_dscnt" in line or "s_wait_kmcnt" in line:
            counts["s_wait_"] += 1
        if "s_wait_loadcnt" in line:
            counts["s_wait_loadcnt"] += 1
        if "s_clause" in line:
            counts["s_clause"] += 1
        if "v_dot4" in line:
            counts["v_dot4"] += 1
    # Max loads outstanding before the first wait in the unrolled group loop:
    # walk from the first global_load_b32 that is not a kernarg (offset:N)
    # until the first s_wait*, counting consecutive global_load*.
    max_outstanding = 0
    outstanding = 0
    in_burst = False
    for line in lines:
        is_load = "global_load_" in line
        is_wait = "s_waitcnt" in line or "s_wait_" in line
        is_kernarg = is_load and "s[2:3]" in line
        if is_load and not is_kernarg:
            outstanding += 1
            in_burst = True
            max_outstanding = max(max_outstanding, outstanding)
        elif is_wait and in_burst:
            outstanding = 0
            in_burst = False
        elif is_wait:
            outstanding = 0
    counts["max_loads_outstanding"] = max_outstanding
    return counts


def load_gen():
    entries = []
    if not GEN_ROOT.exists():
        return entries
    for arch_dir in sorted(p for p in GEN_ROOT.iterdir() if p.is_dir()):
        arch = arch_dir.name
        for json_path in sorted(arch_dir.glob("*.json")):
            s_path = json_path.with_suffix(".s")
            if not s_path.exists():
                die(f"missing .s for {json_path}")
            data = json.loads(json_path.read_text())
            for key in ("spilled_vgprs", "spilled_sgprs", "scratch_bytes", "lds_bytes"):
                if data.get(key, 0) != 0:
                    die(f"refusing {json_path}: {key}={data.get(key)}")
            kernel = data.get("kernel", json_path.stem.rsplit("_w", 1)[0])
            if kernel != KERNEL and "dot_path" not in kernel:
                continue
            wave = int(data["wave"])
            raw = s_path.read_text()
            if "v_dot4" not in raw:
                die(f"refusing {s_path}: packed-dot body has no v_dot4*")
            max_v, max_s = max_regs(raw)
            body = process_body(raw, KERNEL, wave, arch)
            if ")MW" in body:
                die(f"body contains )MW marker for {kernel} {arch} w{wave}")
            stats = isa_stats(raw)
            entries.append(
                {
                    "arch": arch,
                    "kernel": KERNEL,
                    "wave": wave,
                    "data": data,
                    "body": body,
                    "max_v": max_v,
                    "max_s": max_s,
                    "stats": stats,
                }
            )
    entries.sort(key=lambda e: (e["arch"], e["wave"]))
    return entries


def emit_invoke(arch: str, wave: int, needs_s5: bool, max_v: int, max_s: int) -> str:
    macro = f"MICROWAVE_DOTPATH_{arch.upper()}_W{wave}_BODY"
    exclude = {2, 3, 4}
    inputs = [
        '"{s[2:3]}"(__builtin_amdgcn_kernarg_segment_ptr())',
        '"{s4}"((u32)blockDim.x)',
    ]
    if needs_s5:
        exclude.add(5)
        inputs.append('"{s5}"(__builtin_amdgcn_workgroup_id_x())')
    inputs.append('"{v0}"(__builtin_amdgcn_workitem_id_x())')
    clobbers = [f'"v{i}"' for i in range(1, max_v + 1)]
    clobbers += [
        f'"s{i}"' for i in range(0, min(max_s + 1, SGPR_GENERAL_MAX + 1)) if i not in exclude
    ]
    clobbers += ['"vcc"', '"scc"', '"exec"', '"m0"', '"memory"']
    return (
        f"    asm volatile({macro}\n"
        f"        :\n"
        f"        : {', '.join(inputs)}\n"
        f"        : {', '.join(clobbers)});"
    )


def write_header(entries) -> None:
    lines = [
        "// SPDX-License-Identifier: Apache-2.0",
        "// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>",
        "// Auto-generated by wrap_dotpath.py — do not edit",
        "#pragma once",
        "",
    ]
    by_key = {(e["arch"], e["wave"]): e for e in entries}
    for e in entries:
        d = e["data"]
        st = e["stats"]
        macro = f"MICROWAVE_DOTPATH_{e['arch'].upper()}_W{e['wave']}_BODY"
        lines.append(
            f"// {e['kernel']} {e['arch']} w{e['wave']} vgprs={d['vgprs']} "
            f"sgprs={d['sgprs']} parsed_max_v={e['max_v']} parsed_max_s={e['max_s']} "
            f"code_size={d['code_size']} lds={d['lds_bytes']} scratch={d['scratch_bytes']} "
            f"b32={st['global_load_b32']} b64={st['global_load_b64']} "
            f"b128={st['global_load_b128']} waitcnt={st['s_waitcnt']} "
            f"s_wait_={st['s_wait_']} s_clause={st['s_clause']} v_dot4={st['v_dot4']} "
            f"max_outstanding={st['max_loads_outstanding']}"
        )
        lines.append(f"#define {macro} R\"MW({e['body']})MW\"")
        lines.append("")

    lines.append("// Invoke macros pin the RADV CS ABI into HIP inline asm.")
    for arch, needs_s5 in ARCH_SPECS:
        for wave in (32, 64):
            e = by_key.get((arch, wave))
            name = f"MICROWAVE_DOTPATH_INVOKE_{arch.upper()}_W{wave}"
            if not e:
                lines.append(f"// missing {arch} w{wave}")
                continue
            invoke = emit_invoke(arch, wave, needs_s5, e["max_v"], e["max_s"])
            lines.append(f"#define {name} do {{ \\")
            for line in invoke.splitlines():
                lines.append(f"{line} \\")
            lines.append("    } while (0)")
            lines.append("")

    OUT_HEADER.write_text("\n".join(lines) + "\n")
    print(f"wrote {OUT_HEADER} with {len(entries)} entries")
    for e in entries:
        st = e["stats"]
        print(
            f"  {e['arch']} w{e['wave']}: vgpr_parsed={e['max_v']} "
            f"b32={st['global_load_b32']} b128={st['global_load_b128']} "
            f"v_dot4={st['v_dot4']} max_out={st['max_loads_outstanding']}"
        )


def main() -> None:
    entries = load_gen()
    if not entries:
        die(f"no gen entries under {GEN_ROOT}")
    write_header(entries)


if __name__ == "__main__":
    main()
