#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
"""Extract the small kernel-argument manifest needed by the Redline capture shim."""

from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
from pathlib import Path


FIELD = re.compile(r"^\s+\.([A-Za-z0-9_]+):\s*(.*?)\s*$")
ARG_START = re.compile(r"^\s+- \.([A-Za-z0-9_]+):\s*(.*?)\s*$")


def metadata_text(path: Path, readobj: str) -> str:
    result = subprocess.run(
        [readobj, "--notes", "--elf-output-style=JSON", str(path)],
        check=True,
        capture_output=True,
        text=True,
    )
    document = json.loads(result.stdout)
    notes = document[0]["NoteSections"][0]["NoteSection"]["Notes"]
    for note in notes:
        if "AMDGPU Metadata" in note:
            return note["AMDGPU Metadata"]
    raise RuntimeError(f"{path} contains no AMDGPU metadata note")


def parse_kernels(text: str) -> list[dict]:
    kernels: list[dict] = []
    current: dict | None = None
    current_arg: dict | None = None
    for line in text.splitlines():
        if line.strip() == "- .args:":
            if current:
                kernels.append(current)
            current = {"args": []}
            current_arg = None
            continue
        if current is None:
            continue
        match = ARG_START.match(line)
        if match:
            key, value = match.groups()
            current_arg = {}
            current["args"].append(current_arg)
            if key == "offset":
                current_arg[key] = int(value)
            elif key in {"size", "value_kind"}:
                current_arg[key] = int(value) if key == "size" else value
            continue
        match = FIELD.match(line)
        if not match:
            continue
        key, value = match.groups()
        if current_arg is not None and key in {"offset", "size", "value_kind"}:
            current_arg[key] = int(value) if key in {"offset", "size"} else value
        elif key in {"name", "symbol"}:
            current[key] = value
        elif key == "kernarg_segment_size":
            current["kernarg_size"] = int(value)
    if current:
        kernels.append(current)
    required = {"name", "symbol", "kernarg_size", "args"}
    for kernel in kernels:
        missing = required - kernel.keys()
        if missing:
            raise RuntimeError(f"incomplete kernel metadata: missing {sorted(missing)}")
        for arg in kernel["args"]:
            if not {"offset", "size", "value_kind"} <= arg.keys():
                raise RuntimeError(f"incomplete argument metadata for {kernel['symbol']}")
    return kernels


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("hsaco", type=Path)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument(
        "--readobj",
        default=shutil.which("llvm-readobj")
        or next(
            (
                candidate
                for candidate in (
                    "/opt/rocm/llvm/bin/llvm-readobj",
                    "/opt/rocm/core/lib/llvm/bin/llvm-readobj",
                )
                if Path(candidate).exists()
            ),
            "/opt/rocm/llvm/bin/llvm-readobj",
        ),
    )
    args = parser.parse_args()
    kernels = parse_kernels(metadata_text(args.hsaco, args.readobj))
    args.out.write_text(json.dumps({"kernels": kernels}, indent=2) + "\n")


if __name__ == "__main__":
    main()
