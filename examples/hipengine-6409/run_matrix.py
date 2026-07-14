#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Run the retained #6409 matrix for HIP, Vulkan, and Redline."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[2]
TOOLCHAIN = ROOT / "examples/hipengine-6409/toolchain"
TOOLCHAIN_HIPGRAPH = ROOT / "examples/hipengine-6409/toolchain-hipgraph"
PY_LAUNCHER = ROOT / "examples/hipengine-6409/run_python_runner.py"
NORMALIZER = ROOT / "examples/hipengine-6409/normalize_redline.py"

FAMILIES = [
    ("geometry", "geometry_sweep.py", ["--k-list", "512,2048", "--rows-list", "1,4", "--workgroups", "64,256", "--body-repeats", "32"]),
    ("reduction", "reduction_sweep.py", ["--k-list", "512,2048", "--rows-list", "1", "--workgroups", "64,256", "--body-repeats", "32"]),
    ("memory-waitcnt", "memory_waitcnt.py", ["--variants", "coalesced:4,strided:4,gather:1,interleave:4", "--n", "32768", "--body-iters", "64", "--workgroups", "64,256"]),
    ("packed-dot", "dot_path.py", ["--variants", "q8_signed:16,q4_unsigned:16,q6_zero:16,scalar_dequant:16", "--n", "32768", "--body-iters", "64", "--workgroups", "64,256"]),
    ("vopd", "vopd_sweep.py", ["--variants", "independent_fma:4,dependent_fma:4,mixed_int_float:4,dequant_like:4", "--n", "65536", "--body-iters", "512", "--workgroups", "64,256"]),
    ("sampler", "sampler_argmax.py", ["--rows-list", "1,4,8", "--workgroups", "64,256", "--top-k-list", "1,8", "--vocab", "32768"]),
    ("two-stage-reduction", "two_stage_reduction.py", ["--k-list", "8192,32768", "--rows-list", "1,4", "--workgroups", "128,256", "--split-counts", "2,4", "--body-repeats", "16"]),
    ("q4-selected-dual", "q4_selected_dual_real_slice.py", ["--x-rows", "4", "--rows", "32", "--experts", "256", "--in-features", "2048", "--out-features", "512", "--workgroups", "64,128"]),
    ("q6-x8-selected-down", "q6_x8_real_slice.py", ["--rows", "8", "--experts", "256", "--in-features", "512", "--out-features", "2048", "--local-size", "64"]),
    ("dense-q8", "q8_0_dense_real_slice.py", ["--shapes", "768x2048,2048x2048", "--rows-list", "1,4", "--local-sizes", "32,64,128", "--row-tiles", "1,4"]),
]
PRODUCTION = {"q4-selected-dual", "q6-x8-selected-down", "dense-q8"}


def run(command: list[str], *, cwd: Path, env: dict[str, str], log: Path) -> None:
    log.parent.mkdir(parents=True, exist_ok=True)
    print("+", " ".join(command), flush=True)
    with log.open("w") as output:
        completed = subprocess.run(command, cwd=cwd, env=env, text=True, stdout=output, stderr=subprocess.STDOUT)
    if completed.returncode:
        tail = log.read_text(errors="replace").splitlines()[-40:]
        raise RuntimeError(f"command failed ({completed.returncode}); {log}\n" + "\n".join(tail))


def rows(value: Any) -> list[dict[str, Any]]:
    if isinstance(value, dict):
        measurements = value.get("measurements")
        if isinstance(measurements, dict) and isinstance(measurements.get("rows"), list):
            return [row for row in measurements["rows"] if isinstance(row, dict)]
        if isinstance(value.get("rows"), list):
            candidates = [row for row in value["rows"] if isinstance(row, dict)]
            if any(isinstance(row.get("timing"), dict) for row in candidates):
                return candidates
    found: list[dict[str, Any]] = []
    if isinstance(value, dict):
        timing = value.get("timing")
        if isinstance(timing, dict) and isinstance(timing.get("burst"), dict):
            found.append(value)
        else:
            for child in value.values():
                found.extend(rows(child))
    elif isinstance(value, list):
        for child in value:
            found.extend(rows(child))
    return found


def summarize(artifacts: list[dict[str, Any]], out: Path, elapsed: float, metadata: dict[str, Any]) -> None:
    records: list[dict[str, Any]] = []
    for artifact in artifacts:
        data = json.loads(Path(artifact["path"]).read_text())
        for row in rows(data):
            gpu = row["timing"]["burst"]["gpu_elapsed"]
            per = gpu.get("per_iteration_us", {})
            records.append(
                {
                    "family": artifact["family"],
                    "mode": artifact["mode"],
                    "backend": artifact["backend"],
                    "median_us": per.get("median"),
                    "correctness_pass": row.get("correctness_pass"),
                    "shape": {k: v for k, v in row.items() if k not in {"timing", "correctness", "dependency_contract", "submission", "numeric_correctness"} and isinstance(v, (str, int, float, bool))},
                }
            )
    payload = {
        "kind": "redline_hipengine_6409_three_backend_matrix",
        "schema_version": 1,
        "metadata": metadata,
        "elapsed_seconds": elapsed,
        "artifacts": artifacts,
        "rows": records,
        "dispatch_status": "run separately by dispatch_matrix.py; upstream HIP graph-inside-timer is not recapturable",
    }
    out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")


def write_upstream_sampler_rejection(path: Path, log: Path, retry_log: Path, args: argparse.Namespace) -> None:
    """Retain the pinned runner's known invalid independent HIP sampler as data."""
    payload = {
        "backend": "hip",
        "bench": "sampler_argmax",
        "classification": "rejected_correctness_failure",
        "correctness": {"status": "fail", "timed_sequence_pass": False},
        "diagnostic": {
            "attempts": 2,
            "logs": [str(log), str(retry_log)],
            "reason": (
                "Pinned hipEngine runner rejected both attempts because its timed "
                "dependency contract was not validated. No HIP performance row is retained."
            ),
        },
        "kind": "hipengine_micro_rejected_result",
        "measurements": {"rows": []},
        "parameters": {
            "repetitions": args.reps,
            "samples": args.samples,
            "timing_mode": "independent_throughput",
            "warmup_logical_iterations": args.warmup,
        },
        "schema_version": 2,
        "source": {
            "commit": "f2c3ad6d74c86e3641ce09ff9fd759eaa6cd75e0",
            "repo": str(args.hipengine_root.resolve()),
        },
    }
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--hipengine-root", type=Path, default=Path("/tmp/hipEngine-f2c"))
    parser.add_argument("--out-dir", type=Path, required=True)
    parser.add_argument("--gfx-arch", default="gfx1201")
    parser.add_argument("--gpu-name", default="AMD Radeon Graphics")
    parser.add_argument("--reps", type=int, default=10)
    parser.add_argument("--warmup", type=int, default=3)
    parser.add_argument("--samples", type=int, default=5)
    parser.add_argument("--backends", default="hip,vulkan,redline")
    parser.add_argument("--modes", default="serial_latency,independent_throughput")
    parser.add_argument("--families", default=",".join(item[0] for item in FAMILIES))
    args = parser.parse_args()
    hipengine = args.hipengine_root.resolve()
    runners = hipengine / "benchmarks/micro/runners"
    out = args.out_dir.resolve()
    out.mkdir(parents=True, exist_ok=True)
    env_path = out / "environment.json"
    base_env = dict(os.environ)
    # Keep distro glslc first (it exposes GL_EXT_integer_dot_product here), but
    # make ROCm-only llvm-readobj/objdump available to the ISA collectors.
    base_env["PATH"] = f"{base_env['PATH']}:/opt/rocm/llvm/bin:/opt/rocm/bin"
    shaderc_root = Path(os.environ.get("REDLINE_SHADERC_ROOT", "/tmp/shaderc-2025/root"))
    if (shaderc_root / "usr/bin/glslc").exists():
        base_env["PATH"] = f"{shaderc_root / 'usr/bin'}:{base_env['PATH']}"
        shaderc_lib = shaderc_root / "usr/lib/x86_64-linux-gnu"
        base_env["LD_LIBRARY_PATH"] = f"{shaderc_lib}:{base_env.get('LD_LIBRARY_PATH', '')}"
    base_env["HIPENGINE_HIP_ARCH"] = args.gfx_arch
    if not env_path.exists():
        run([sys.executable, str(hipengine / "benchmarks/micro/collect_env.py"), "--out", str(env_path), "--pretty"], cwd=hipengine, env=base_env, log=out / "logs/collect-env.log")
    backends = [x for x in args.backends.split(",") if x]
    modes = [x for x in args.modes.split(",") if x]
    selected_families = {x for x in args.families.split(",") if x}
    artifacts: list[dict[str, Any]] = []
    started = time.monotonic()
    for mode in modes:
        for family, filename, family_args in FAMILIES:
            if family not in selected_families:
                continue
            runner = runners / filename
            for backend in backends:
                raw_backend = "hip" if backend in ("redline", "hipgraph") else backend
                raw_out = out / mode / f"{backend}-{family}.raw.json"
                final_out = out / mode / f"{backend}-{family}.json"
                artifact_record = {"family": family, "mode": mode, "backend": backend, "path": str(final_out)}
                if final_out.exists():
                    artifacts.append(artifact_record)
                    continue
                command = [sys.executable, str(runner)]
                env = dict(base_env)
                if backend == "redline":
                    env["PATH"] = f"{TOOLCHAIN}:{env['PATH']}"
                    env["REDLINE_HIPCC_VERSION_SUFFIX"] = "6409-radiowave-certified-pm4-v5"
                    env["RADIOWAVE_SCHEDULER_PROFILE"] = "default"
                    if family in PRODUCTION:
                        command = [sys.executable, str(PY_LAUNCHER), str(runner)]
                if backend == "hipgraph":
                    # Same unmodified HIP harness, but the force-included shim
                    # swaps HipSequenceTimer -> HipGraphSequenceTimer (capture +
                    # hipGraphLaunch). No manifest/capi needed; native output.
                    env["PATH"] = f"{TOOLCHAIN_HIPGRAPH}:{env['PATH']}"
                    env["HIPGRAPH_HIPCC_VERSION_SUFFIX"] = "6409-hipgraph-v1"
                if backend == "redline" and raw_out.exists():
                    run([sys.executable, str(NORMALIZER), str(raw_out), "--timing-mode", mode, "--out", str(final_out)], cwd=ROOT, env=env, log=out / "logs" / mode / f"normalize-{family}.log")
                    artifacts.append(artifact_record)
                    continue
                command += [
                    "--backend", raw_backend,
                    "--timing-mode", mode,
                    "--independent-streams", "4",
                    "--reps", str(args.reps),
                    "--warmup", str(args.warmup),
                    "--samples", str(args.samples),
                    "--environment-json", str(env_path),
                    "--environment-ref", str(env_path),
                    "--gfx-arch", args.gfx_arch,
                    "--hardware-gpu", args.gpu_name,
                    "--build-dir", str(out / "build" / backend / family),
                    *family_args,
                    "--out", str(raw_out if backend == "redline" else final_out),
                    "--pretty",
                ]
                log = out / "logs" / mode / f"{backend}-{family}.log"
                try:
                    run(command, cwd=hipengine, env=env, log=log)
                except RuntimeError:
                    known_sampler_rejection = (
                        backend == "hip"
                        and family == "sampler"
                        and mode == "independent_throughput"
                    )
                    if not known_sampler_rejection:
                        raise
                    retry_log = out / "logs" / mode / "hip-sampler-retry.log"
                    try:
                        run(command, cwd=hipengine, env=env, log=retry_log)
                    except RuntimeError:
                        write_upstream_sampler_rejection(final_out, log, retry_log, args)
                        artifacts.append(artifact_record)
                        summarize(
                            artifacts,
                            out / "matrix.partial.json",
                            time.monotonic() - started,
                            vars(args)
                            | {"hipengine_root": str(hipengine), "out_dir": str(out)},
                        )
                        continue
                if backend == "redline":
                    run([sys.executable, str(NORMALIZER), str(raw_out), "--timing-mode", mode, "--out", str(final_out)], cwd=ROOT, env=env, log=out / "logs" / mode / f"normalize-{family}.log")
                artifacts.append(artifact_record)
                summarize(artifacts, out / "matrix.partial.json", time.monotonic() - started, vars(args) | {"hipengine_root": str(hipengine), "out_dir": str(out)})
    summarize(artifacts, out / "matrix.json", time.monotonic() - started, vars(args) | {"hipengine_root": str(hipengine), "out_dir": str(out)})
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
