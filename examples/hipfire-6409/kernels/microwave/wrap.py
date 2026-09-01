#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
"""Turn extracted ACO bodies into HIP inline-asm wrappers.

Reads kernels/microwave/gen/<arch>/<kernel>_w<wave>.{s,json} and writes
kernels/hipfire-inhouse-microwave/{microwave_gen.h,hipfire_6409.hip}.
Clobber lists come from the max v/s index in the assembly, not RADV's
sgprs=128 allocation constant. s106+ are not general SGPRs.
"""
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
GEN_ROOT = ROOT / "kernels/microwave/gen"
OUT_HEADER = ROOT / "kernels/hipfire-inhouse-microwave/microwave_gen.h"
OUT_HIP = ROOT / "kernels/hipfire-inhouse-microwave/hipfire_6409.hip"
STOCK_HIP = ROOT / "kernels/hipfire_6409.hip"

KERNELS = ["dot_q8", "dot_q4", "dot_q6", "dot_scalar"]
ARCH_SPECS = [
    # (arch, preprocessor symbol, tgid lives in s5)
    ("gfx1201", "__gfx1201__", False),
    ("gfx1151", "__gfx1151__", True),
    ("gfx1100", "__gfx1100__", True),
]
SGPR_GENERAL_MAX = 105  # vcc=106/107, ttmp=108..123, m0=124, exec=126/127

STOCK_DOTS = (
    'extern "C" __global__ __launch_bounds__(256, 16)\n'
    "void dot_q8(BENCH_ARGS) { dot_body<0>(a, b, out, n0, n1, output_offset, aux); }\n"
    'extern "C" __global__ __launch_bounds__(256, 16)\n'
    "void dot_q4(BENCH_ARGS) { dot_body<1>(a, b, out, n0, n1, output_offset, aux); }\n"
    'extern "C" __global__ __launch_bounds__(256, 16)\n'
    "void dot_q6(BENCH_ARGS) { dot_body<2>(a, b, out, n0, n1, output_offset, aux); }\n"
    'extern "C" __global__ __launch_bounds__(256, 16)\n'
    "void dot_scalar(BENCH_ARGS) { dot_body<3>(a, b, out, n0, n1, output_offset, aux); }"
)

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
            kernel = data["kernel"]
            wave = int(data["wave"])
            if kernel not in KERNELS:
                continue
            raw = s_path.read_text()
            if kernel in ("dot_q8", "dot_q4", "dot_q6") and "v_dot4" not in raw:
                die(f"refusing {s_path}: packed-dot body has no v_dot4*")
            max_v, max_s = max_regs(raw)
            body = process_body(raw, kernel, wave, arch)
            if ")MW" in body:
                die(f"body contains )MW marker for {kernel} {arch} w{wave}")
            entries.append(
                {
                    "arch": arch,
                    "kernel": kernel,
                    "wave": wave,
                    "data": data,
                    "body": body,
                    "max_v": max_v,
                    "max_s": max_s,
                }
            )
    entries.sort(key=lambda e: (e["arch"], e["kernel"], e["wave"]))
    return entries


def write_header(entries) -> None:
    lines = [
        "// SPDX-License-Identifier: Apache-2.0",
        "// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>",
        "// Auto-generated by kernels/microwave/wrap.py — do not edit",
        "#pragma once",
        "",
    ]
    for e in entries:
        d = e["data"]
        macro = f"MICROWAVE_{e['kernel'].upper()}_{e['arch'].upper()}_W{e['wave']}_BODY"
        lines.append(
            f"// {e['kernel']} {e['arch']} w{e['wave']} vgprs={d['vgprs']} "
            f"sgprs={d['sgprs']} parsed_max_v={e['max_v']} parsed_max_s={e['max_s']} "
            f"code_size={d['code_size']} lds={d['lds_bytes']} scratch={d['scratch_bytes']}"
        )
        lines.append(f"#define {macro} R\"MW({e['body']})MW\"")
        lines.append("")
    OUT_HEADER.parent.mkdir(parents=True, exist_ok=True)
    OUT_HEADER.write_text("\n".join(lines) + "\n")
    print(f"wrote {OUT_HEADER} with {len(entries)} entries")


def emit_asm(kernel: str, arch: str, wave: int, needs_s5: bool, max_v: int, max_s: int) -> str:
    macro = f"MICROWAVE_{kernel.upper()}_{arch.upper()}_W{wave}_BODY"
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


def fallback(kernel: str, arch: str, wave, mode: int, detail: str) -> list[str]:
    label = f"{kernel} {arch}" if wave is None else f"{kernel} {arch} wave{wave}"
    return [
        f'#warning "microwave body missing for {label}{detail}, falling back to stock"',
        f"    dot_body<{mode}>(a, b, out, n0, n1, output_offset, aux);",
    ]


def gen_wrapper(kernel: str, mode: int, available: dict) -> str:
    lines = [
        'extern "C" __global__ __launch_bounds__(256, 16)',
        f"void {kernel}(BENCH_ARGS) {{",
    ]
    for i, (arch, symbol, needs_s5) in enumerate(ARCH_SPECS):
        directive = "#if" if i == 0 else "#elif"
        lines.append(f"#if defined({symbol})" if directive == "#if" else f"#elif defined({symbol})")
        lines.append("#ifdef HIPFIRE_BENCH_WAVE64")
        e64 = available.get((arch, kernel, 64))
        if e64:
            lines.append(emit_asm(kernel, arch, 64, needs_s5, e64["max_v"], e64["max_s"]))
        else:
            lines.extend(fallback(kernel, arch, 64, mode, ""))
        lines.append("#else")
        e32 = available.get((arch, kernel, 32))
        if e32:
            lines.append(emit_asm(kernel, arch, 32, needs_s5, e32["max_v"], e32["max_s"]))
        else:
            lines.extend(fallback(kernel, arch, 32, mode, ""))
        lines.append("#endif")
    lines.append("#else")
    lines.extend(fallback(kernel, "arch", None, mode, ""))
    lines.append("#endif")
    lines.append("}")
    return "\n".join(lines)


def write_hip(entries) -> None:
    stock = STOCK_HIP.read_text()
    if STOCK_DOTS not in stock:
        die("stock hipfire_6409.hip is missing the expected packed-dot entry points")
    available = {(e["arch"], e["kernel"], e["wave"]): e for e in entries}
    wrappers = ['#include "microwave_gen.h"', ""]
    for kernel, mode in [("dot_q8", 0), ("dot_q4", 1), ("dot_q6", 2), ("dot_scalar", 3)]:
        wrappers.append(gen_wrapper(kernel, mode, available))
        wrappers.append("")
    new_hip = stock.replace(STOCK_DOTS, "\n".join(wrappers).rstrip())
    OUT_HIP.parent.mkdir(parents=True, exist_ok=True)
    OUT_HIP.write_text(new_hip)
    print(f"wrote {OUT_HIP}")


def main() -> None:
    entries = load_gen()
    write_header(entries)
    write_hip(entries)


if __name__ == "__main__":
    main()
