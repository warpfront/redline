#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Run a pinned hipEngine Python runner with only its HIP timer replaced."""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

import redline_hip_timing


def main() -> int:
    if len(sys.argv) < 2:
        raise SystemExit("usage: run_python_runner.py RUNNER [ARGS ...]")
    runner = Path(sys.argv[1]).resolve()
    sys.argv = [str(runner), *sys.argv[2:]]
    spec = importlib.util.spec_from_file_location("redline_pinned_runner", runner)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {runner}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    if not hasattr(module, "hip_timing"):
        raise RuntimeError(f"{runner.name} does not expose hip_timing")
    module.hip_timing = redline_hip_timing
    result = module.main()
    return 0 if result is None else int(result)


if __name__ == "__main__":
    raise SystemExit(main())
