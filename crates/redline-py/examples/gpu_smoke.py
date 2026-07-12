# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
"""Real-GPU PyO3 gate for redline-dispatch.

Loads an atomic-increment code object, allocates a GPU-accessible counter,
records N dispatches referencing it into one retained PM4 IB, replays, and reads
the counter back -- it must equal N iff every dispatch executed to completion.

    ROCR_VISIBLE_DEVICES=3 python gpu_smoke.py 256 bench/floor_kernel_ctr.co
"""
import sys

import redline_dispatch as rl

N = int(sys.argv[1]) if len(sys.argv) > 1 else 256
co_path = sys.argv[2] if len(sys.argv) > 2 else "bench/floor_kernel_ctr.co"

gpu = rl.Gpu(0)  # ROCR_VISIBLE_DEVICES picks the physical device
with open(co_path, "rb") as f:
    mod = gpu.load_module(f.read())

counter = gpu.alloc(4)  # zeroed, GPU-accessible
kernarg = counter.address().to_bytes(8, "little")  # ctr_k(unsigned int* p): p = counter
dispatches = [("ctr_k.kd", (1, 1, 1), (1, 1, 1), 0, kernarg, True) for _ in range(N)]

ib = gpu.build(mod, dispatches)
ib.replay()

val = counter.read_u32(0)
ok = val == N
print(f"PyO3 real-GPU gate: counter = {val} / {N}  [{'PASS' if ok else 'FAIL'}]")
sys.exit(0 if ok else 1)
