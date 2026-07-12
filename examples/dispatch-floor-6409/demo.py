#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
"""Reproduce hipEngine's ROCm/ROCm#6409 dispatch-floor and add a Redline arm.

The #6409 complaint: HIP graph replay is slower than a pre-recorded Vulkan
command buffer at the tiny-dispatch floor. This runs the *same* kernel
(`gmb_noop_kernel`, +1.0f per element, block=256), the *same* serial-latency
dependency chain, at the *same* counts (1, 50, 200, 941), via:

  * hip_graph  — hipStreamBeginCapture -> hipGraphInstantiate -> hipGraphLaunch
                 (compiled `hipgraph_baseline`, hipEvent + host timed);
  * redline    — the same dispatches lowered to ONE retained GFX12 PM4 indirect
                 buffer and replayed (`pip install redline-dispatch`).

Both arms verify correctness (every element == count). The headline speedup is
host µs/dispatch (matched to hipEngine's `host_wall` domain).

    pip install redline-dispatch
    ROCR_VISIBLE_DEVICES=3 python demo.py
"""
import os
import shutil
import statistics
import struct
import subprocess
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
ARCH = os.environ.get("REDLINE_GFX_ARCH") or os.environ.get("HIPENGINE_HIP_ARCH") or "gfx1201"
COUNTS = [int(x) for x in os.environ.get("REDLINE_6409_COUNTS", "1,50,200,941").split(",")]
N = int(os.environ.get("REDLINE_6409_N", "256"))
BLOCK = 256
REPS = int(os.environ.get("REDLINE_6409_REPS", "50"))
WARMUP = int(os.environ.get("REDLINE_6409_WARMUP", "10"))
CO = os.path.join(HERE, "gmb_noop.co")
BASE = os.path.join(HERE, "hipgraph_baseline")


def hipcc():
    return shutil.which("hipcc") or "/opt/rocm/bin/hipcc"


def build():
    hc = hipcc()
    if not os.path.exists(CO):
        subprocess.run([hc, "--genco", f"--offload-arch={ARCH}",
                        os.path.join(HERE, "gmb_noop.hip"), "-o", CO], check=True)
    if not os.path.exists(BASE):
        subprocess.run([hc, f"--offload-arch={ARCH}",
                        os.path.join(HERE, "hipgraph_baseline.hip"), "-o", BASE], check=True)


def run_hipgraph(count):
    r = subprocess.run([BASE, str(count), str(N), str(REPS), str(WARMUP)],
                       capture_output=True, text=True)
    if r.returncode not in (0, 1):
        raise RuntimeError(f"hipgraph_baseline failed: {r.stderr}")
    kv = dict(tok.split("=") for tok in r.stdout.split())
    return {"gpu_per_disp": float(kv["gpu_per_disp"]),
            "host_per_disp": float(kv["host_per_disp"]),
            "correct": kv["correct"] == "1"}


def run_redline(gpu, mod, count):
    grid_blocks = (N + BLOCK - 1) // BLOCK
    workitems = grid_blocks * BLOCK
    out = gpu.alloc(N * 4)  # zeroed
    kernarg = out.address().to_bytes(8, "little") + N.to_bytes(4, "little")
    dispatches = [("gmb_noop_kernel.kd", (workitems, 1, 1), (BLOCK, 1, 1), 0, kernarg, True)
                  for _ in range(count)]
    ib = gpu.build(mod, dispatches)
    ib.replay()  # correctness pass from the zeroed buffer
    val = struct.unpack("<f", struct.pack("<I", out.read_u32(0)))[0]
    correct = val == float(count)
    for _ in range(WARMUP):
        ib.replay()
    samples = []
    for _ in range(REPS):
        t0 = time.perf_counter()
        ib.replay()
        samples.append((time.perf_counter() - t0) * 1e6)
    med = statistics.median(samples)
    return {"host_per_disp": med / count, "correct": correct}


def main():
    build()
    import redline_dispatch as rl
    gpu = rl.Gpu(0)
    with open(CO, "rb") as f:
        mod = gpu.load_module(f.read())

    print(f"ROCm #6409 dispatch-floor  |  gmb_noop_kernel  n={N} block={BLOCK}  "
          f"reps={REPS} (median)  arch={ARCH}")
    print(f"{'count':>6} | {'hip_graph gpu µs/disp':>21} | {'hip_graph host µs/disp':>22} | "
          f"{'redline host µs/disp':>20} | {'speedup(host)':>13} | correct")
    print("-" * 108)
    all_ok = True
    for c in COUNTS:
        hg = run_hipgraph(c)
        rd = run_redline(gpu, mod, c)
        sp = hg["host_per_disp"] / rd["host_per_disp"] if rd["host_per_disp"] > 0 else float("inf")
        ok = hg["correct"] and rd["correct"]
        all_ok = all_ok and ok
        print(f"{c:>6} | {hg['gpu_per_disp']:>21.4f} | {hg['host_per_disp']:>22.4f} | "
              f"{rd['host_per_disp']:>20.4f} | {sp:>12.2f}x | {'PASS' if ok else 'FAIL'}")
    sys.exit(0 if all_ok else 1)


if __name__ == "__main__":
    main()
