#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
"""Summarise a hipfire-mqv2-bench result JSON.

Prints, per (family, shape, mode): every kernel variant with its hip /
hipgraph / redline median microseconds, GFLOP/s on the hip column, the gate
status, and the redline/hip ratio. Scratch-refused redline cells print as
"scratch". A second table ranks variants per (family, shape, mode) by hip
median so the best variant per shape is visible at a glance.

    python3 summarize.py results/gfx1151/2026-09-01-prefill.json [--family gate_up]
"""
import argparse
import json
import sys
from collections import defaultdict


def median_us(backend):
    if not backend or backend.get("error"):
        return None
    dist = backend.get("distribution") or {}
    return dist.get("median_us")


def gate(backend):
    if not backend:
        return "-"
    if backend.get("error"):
        return "scratch" if "scratch" in backend["error"] else "error"
    corr = backend.get("correctness") or {}
    return "pass" if corr.get("pass") else "FAIL"


def gflops(row, us):
    if us is None or us <= 0:
        return None
    shape = row["shape"]
    total_m = sum(shape["proj_m"])
    return 2.0 * shape["n_tokens"] * shape["k"] * total_m / us / 1e3


def variant_label(row):
    kernel = row["kernel"]
    variant = row.get("variant")
    if variant is None and isinstance(kernel, dict):
        variant = kernel.get("variant")
    if isinstance(variant, dict):
        ((kind, arg),) = variant.items()
        if isinstance(arg, dict):
            arg = next(iter(arg.values()))
        variant = f"{kind}{arg}"
    if variant is None:
        variant = kernel["symbol"] if isinstance(kernel, dict) else str(kernel)
    return f"mq{row['bits']}/{variant}"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("path")
    ap.add_argument("--family", default=None)
    ap.add_argument("--mode", default=None, help="serial_latency|independent_throughput")
    args = ap.parse_args()
    data = json.load(open(args.path))
    rows = data["rows"]
    if args.family:
        rows = [r for r in rows if r["family"] == args.family]
    if args.mode:
        rows = [r for r in rows if r["mode"] == args.mode]

    groups = defaultdict(list)
    for r in rows:
        shape = r["shape"]
        key = (r["family"], shape["n_tokens"], shape["k"], tuple(shape["proj_m"]), r["mode"])
        groups[key].append(r)

    print(f"{len(rows)} rows from {args.path}")
    print()
    print(f"{'family':9s} {'shape':22s} {'mode':4s} {'variant':14s} {'hip us':>10s} {'hg us':>10s} {'rl us':>10s} {'hip GF/s':>9s} {'rl/hip':>7s}  gates(hip/hg/rl) ident")
    ranking = []
    for key in sorted(groups):
        family, n, k, proj, mode = key
        shape_label = f"n{n} k{k} m{'+'.join(map(str, proj))}"
        best = None
        for r in sorted(groups[key], key=lambda r: variant_label(r)):
            b = r["backends"]
            hip, hg, rl = (median_us(b.get(x)) for x in ("hip", "hipgraph", "redline"))
            gf = gflops(r, hip)
            ratio = (rl / hip) if (hip and rl) else None
            fmt = lambda v: f"{v:10.1f}" if v is not None else f"{'-':>10s}"
            rl_cell = fmt(rl) if rl is not None else f"{gate(b.get('redline')):>10s}"
            gf_cell = f"{gf:9.0f}" if gf is not None else f"{'-':>9s}"
            ratio_cell = f"{ratio:7.2f}" if ratio is not None else f"{'-':>7s}"
            gates = f"{gate(b.get('hip'))}/{gate(b.get('hipgraph'))}/{gate(b.get('redline'))}"
            print(
                f"{family:9s} {shape_label:22s} {mode[:3]:4s} {variant_label(r):14s} "
                f"{fmt(hip)} {fmt(hg)} {rl_cell} {gf_cell} {ratio_cell}  "
                f"{gates} {r.get('bit_identical_across_backends')}"
            )
            if hip is not None and (best is None or hip < best[0]):
                best = (hip, variant_label(r))
        if best:
            ranking.append((family, shape_label, mode[:3], best[1], best[0]))
        print()

    print("best variant per (family, shape, mode) by hip median:")
    for family, shape_label, mode, label, us in ranking:
        print(f"  {family:9s} {shape_label:22s} {mode:4s} -> {label:14s} {us:10.1f} us")


if __name__ == "__main__":
    sys.exit(main())
