#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
"""Decode-shaped drop-in gate for redline-dispatch's retained PM4 replay.

This is the per-token pattern an inference engine wants: build the retained IB
ONCE, then every token patch only the scalar/pointer that changed (here a token
value) and replay. No IB rebuild, one doorbell per token.

    hipcc --genco --offload-arch=gfx1201 bench/acc_kernel.hip -o acc.co
    ROCR_VISIBLE_DEVICES=0 python decode_kernargs.py 64 acc.co

The accumulator kernel adds a per-token scalar into a device counter. After T
tokens the counter must equal sum(1..T) iff every replay observed its patched
scalar. The comparison arm rebuilds the IB every token (the naive path) so the
timing delta isolates the value of in-place kernarg mutation.
"""
import struct
import sys
import time

import redline_dispatch as rl

T = int(sys.argv[1]) if len(sys.argv) > 1 else 64
co_path = sys.argv[2] if len(sys.argv) > 2 else "acc.co"
SYMBOL = "acc_k.kd"

gpu = rl.Gpu(0)
with open(co_path, "rb") as f:
    code = f.read()
mod = gpu.load_module(code, None)


def kernarg(acc_addr, val):
    # AMDGPU explicit-arg layout: acc:u64 @0, val:u32 @8.
    return acc_addr.to_bytes(8, "little") + struct.pack("<I", val)


expected = sum(range(1, T + 1))

# --- Retained arm: build once, patch the 4-byte val per token, replay. --------
acc = gpu.alloc(4)
ib = gpu.build(mod, [(SYMBOL, (1, 1, 1), (1, 1, 1), 0, kernarg(acc.address(), 0), True)])
t0 = time.perf_counter()
for token in range(1, T + 1):
    ib.set_kernargs(0, struct.pack("<I", token), 8)  # patch val @ offset 8
    ib.replay()
retained_us = (time.perf_counter() - t0) * 1e6 / T
retained_val = acc.read_u32(0)

# --- Naive arm: rebuild the IB every token (what you avoid). -------------------
acc2 = gpu.alloc(4)
t0 = time.perf_counter()
for token in range(1, T + 1):
    ib2 = gpu.build(mod, [(SYMBOL, (1, 1, 1), (1, 1, 1), 0, kernarg(acc2.address(), token), True)])
    ib2.replay()
rebuild_us = (time.perf_counter() - t0) * 1e6 / T
rebuild_val = acc2.read_u32(0)

ok = retained_val == expected and rebuild_val == expected
print(f"retained (build once, patch+replay): acc={retained_val}/{expected}  {retained_us:7.2f} us/token")
print(f"naive    (rebuild IB every token)  : acc={rebuild_val}/{expected}  {rebuild_us:7.2f} us/token")
print(f"in-place kernarg mutation speedup  : {rebuild_us / retained_us:.2f}x   [{'PASS' if ok else 'FAIL'}]")
sys.exit(0 if ok else 1)
