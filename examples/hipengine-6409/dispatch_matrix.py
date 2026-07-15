#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Run #6409's dispatch/grid family, including retained-PM4 Redline rows."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
COUNTS = (1, 50, 200, 941)
GRIDS = (1, 128, 1024, 8192)
ROW = re.compile(r"count=\s*(\d+)\s+([0-9.]+) µs/dispatch\s+\[(PASS|FAIL)\]")


def command(args: list[str], *, cwd: Path, env: dict[str, str], log: Path) -> None:
    log.parent.mkdir(parents=True, exist_ok=True)
    with log.open("w") as output:
        completed = subprocess.run(args, cwd=cwd, env=env, stdout=output, stderr=subprocess.STDOUT, text=True)
    if completed.returncode:
        raise RuntimeError(f"command failed; see {log}")


def radiowave_dispatch_kernel(
    out: Path, *, arch: str, env: dict[str, str]
) -> tuple[Path, Path, dict]:
    build = out / "build/redline/dispatch"
    build.mkdir(parents=True, exist_ok=True)
    code = build / "gmb_noop.co"
    manifest = build / "gmb_noop.radiowave.json"
    source = ROOT / "examples/dispatch-floor-6409/gmb_noop.hip"
    radiowave = ROOT / "target/release/radiowave"
    if not code.exists() or not manifest.exists():
        command(
            [
                str(radiowave),
                "compile",
                "--source",
                str(source),
                "--output",
                str(code),
                "--manifest",
                str(manifest),
                "--arch",
                arch,
                "--wave32",
                "--scheduler-profile",
                "default",
                "--no-fast-math",
            ],
            cwd=ROOT,
            env=env,
            log=out / "logs/radiowave-dispatch.log",
        )
    certification = json.loads(manifest.read_text())
    digest = hashlib.sha256(code.read_bytes()).hexdigest()
    if certification.get("compiler") != "radiowave" or certification.get("schema_version", 0) < 3:
        raise RuntimeError(f"invalid Radiowave certification: {manifest}")
    if certification.get("output_sha256") != digest:
        raise RuntimeError(f"Radiowave code-object hash mismatch: {manifest}")
    kernels = certification.get("inspection", {}).get("kernels", [])
    kernel = next((item for item in kernels if item.get("name") == "gmb_noop_kernel"), None)
    if kernel is None:
        raise RuntimeError(f"Radiowave manifest has no gmb_noop_kernel: {manifest}")
    return code, manifest, kernel


def redline(
    exe: Path,
    hsaco: Path,
    *,
    boundary: str,
    mode: str,
    counts: tuple[int, ...],
    grid: int | None,
    reps: int,
    warmup: int,
    log: Path,
) -> list[dict]:
    env = dict(os.environ)
    env.update(
        {
            "GMB_HSACO": str(hsaco),
            "GMB_COUNTS": ",".join(map(str, counts)),
            "GMB_REPS": str(reps),
            "GMB_WARMUP": str(warmup),
            "GMB_PROFILE_ONLY": "1",
            "GMB_TIMING_MODE": mode,
            "GMB_ONLY": boundary,
        }
    )
    if grid is not None:
        env["GMB_GRID_BLOCKS"] = str(grid)
    completed = subprocess.run([str(exe)], cwd=ROOT, env=env, capture_output=True, text=True, check=True)
    log.parent.mkdir(parents=True, exist_ok=True)
    log.write_text(completed.stdout + completed.stderr)
    parsed = ROW.findall(completed.stdout)
    if len(parsed) != len(counts):
        raise RuntimeError(f"could not parse Redline rows from {log}")
    return [
        {"count": int(count), "median_us": float(us), "correctness_pass": status == "PASS"}
        for count, us, status in parsed
    ]


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--hipengine-root", type=Path, default=Path("/tmp/hipEngine-f2c"))
    parser.add_argument("--out-dir", type=Path, required=True)
    parser.add_argument("--environment", type=Path, required=True)
    parser.add_argument("--gfx-arch", default="gfx1201")
    parser.add_argument("--gpu-name", default="AMD Radeon Graphics")
    parser.add_argument("--reps", type=int, default=20)
    parser.add_argument("--warmup", type=int, default=5)
    args = parser.parse_args()
    hipengine = args.hipengine_root.resolve()
    out = args.out_dir.resolve()
    environment = args.environment.resolve()
    out.mkdir(parents=True, exist_ok=True)
    exe = ROOT / "target/release/examples/gmb_floor"
    env = dict(os.environ)
    env["HIPENGINE_HIP_ARCH"] = args.gfx_arch
    env["PATH"] = f"{env['PATH']}:/opt/rocm/llvm/bin:/opt/rocm/bin"
    shaderc = Path(os.environ.get("REDLINE_SHADERC_ROOT", "/tmp/shaderc-2025/root"))
    if (shaderc / "usr/bin/glslc").exists():
        env["PATH"] = f"{shaderc / 'usr/bin'}:{env['PATH']}"
        env["LD_LIBRARY_PATH"] = f"{shaderc / 'usr/lib/x86_64-linux-gnu'}:{env.get('LD_LIBRARY_PATH', '')}"
    hsaco, radiowave_manifest, kernel_certification = radiowave_dispatch_kernel(
        out, arch=args.gfx_arch, env=env
    )
    read_cache = kernel_certification.get("mutable_read_cache", "scalar_or_unknown")
    boundary = "hip-llvm-vmem" if read_cache == "vmem_only" else "same-l1"
    artifacts = []
    redline_rows = []
    for mode in ("serial_latency", "independent_throughput"):
        mode_dir = out / mode
        mode_dir.mkdir(parents=True, exist_ok=True)
        hip = mode_dir / "hip-dispatch.json"
        vulkan = mode_dir / "vulkan-dispatch.json"
        if not hip.exists():
            command(
                [sys.executable, str(hipengine / "benchmarks/micro/runners/hip_dispatch_floor.py"), "--counts", "1,50,200,941", "--kernels", "tiny", "--grid-sweep", "1,128,1024,8192", "--reps", str(args.reps), "--warmup", str(args.warmup), "--timing-mode", mode, "--independent-streams", "4", "--environment-json", str(environment), "--environment-ref", str(environment), "--gfx-arch", args.gfx_arch, "--hardware-gpu", args.gpu_name, "--out", str(hip), "--pretty"],
                cwd=hipengine, env=env, log=out / "logs" / mode / "hip-dispatch.log")
        if not vulkan.exists():
            command(
                [sys.executable, str(hipengine / "benchmarks/micro/runners/vulkan_dispatch_floor.py"), "--counts", "1,50,200,941", "--grid-sweep", "1,128,1024,8192", "--reps", str(args.reps), "--warmup", str(args.warmup), "--timing-mode", mode, "--environment-json", str(environment), "--environment-ref", str(environment), "--gfx-arch", args.gfx_arch, "--hardware-gpu", args.gpu_name, "--out", str(vulkan), "--pretty"],
                cwd=hipengine, env=env, log=out / "logs" / mode / "vulkan-dispatch.log")
        count_rows = redline(exe, hsaco, boundary=boundary, mode=mode, counts=COUNTS, grid=None, reps=args.reps, warmup=args.warmup, log=out / "logs" / mode / "redline-dispatch-counts.log")
        for row in count_rows:
            redline_rows.append({"mode": mode, "sweep": "count", "grid_blocks": 1, **row})
        for grid in GRIDS:
            [row] = redline(exe, hsaco, boundary=boundary, mode=mode, counts=(941,), grid=grid, reps=args.reps, warmup=args.warmup, log=out / "logs" / mode / f"redline-dispatch-grid-{grid}.log")
            redline_rows.append({"mode": mode, "sweep": "grid", "grid_blocks": grid, **row})
        artifacts.extend([{"backend": "hip", "mode": mode, "path": str(hip)}, {"backend": "vulkan", "mode": mode, "path": str(vulkan)}])
    payload = {
        "kind": "redline_hipengine_6409_dispatch_matrix",
        "schema_version": 1,
        "source_commit": "f2c3ad6d74c86e3641ce09ff9fd759eaa6cd75e0",
        "sampling": {"reps": args.reps, "warmup": args.warmup},
        "redline": {
            "codegen": {
                "compiler": "radiowave",
                "manifest": str(radiowave_manifest),
                "output_sha256": hashlib.sha256(hsaco.read_bytes()).hexdigest(),
                "scheduler_profile": "default",
                "wavefront_size": kernel_certification.get("wavefront_size"),
                "mutable_read_cache": read_cache,
            },
            "submission": "retained_stateful_pm4_ib",
            "dependency_boundary": boundary,
            "gpu_clock": "redline_pm4_timestamp",
            "rows": redline_rows,
        },
        "artifacts": artifacts,
    }
    (out / "dispatch-matrix.json").write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")


if __name__ == "__main__":
    main()
