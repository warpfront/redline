#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
"""Join suite result JSONs from different arm configurations into one table.

The point: new arms (segmented hipGraph via --hip-queues auto, multi-queue PM4
via --redline-queues auto, single-queue PM4 via --redline-queues 1) are added as
COLUMNS against the reference run's Vulkan and redline rows, without remeasuring
backends that did not change. Each cell is only quoted if that backend's
correctness gate passed in that run; failed or missing cells are counted, never
silently dropped into a ratio.

Usage:
  join_arms.py ref=legacy.json auto=auto.json rq1=redline-q1.json [--mode MODE]

The first file is the reference: its vulkan column is the denominator. Ratios
are per-row backend/vulkan; the summary reports geometric means per mode plus
win counts, and refuses to aggregate cells whose gate failed.
"""
import json
import math
import sys
from collections import defaultdict


def load(path):
    with open(path) as f:
        art = json.load(f)
    env = art.get("environment", {})
    prov = env.get("rocm_provenance", env.get("provenance", {}))
    return art, prov


def cell(row, backend):
    b = row.get("backends", {}).get(backend)
    if not b:
        return None, "absent"
    corr = b.get("correctness") or {}
    dist = b.get("distribution") or {}
    if b.get("error"):
        return None, "error"
    if not corr.get("pass"):
        return None, "gate-fail"
    med = dist.get("median_us")
    return (med, "ok") if med is not None else (None, "no-dist")


def main():
    tagged = [a.split("=", 1) for a in sys.argv[1:] if "=" in a]
    mode_filter = None
    for i, a in enumerate(sys.argv[1:]):
        if a == "--mode":
            mode_filter = sys.argv[1:][i + 1]
    if not tagged:
        sys.exit(__doc__)

    arts = {}
    for tag, path in tagged:
        art, prov = load(path)
        arts[tag] = art
        print(f"# {tag}: {path} rows={len(art['rows'])} provenance={json.dumps(prov)[:160]}")

    ref_tag = tagged[0][0]
    ref_rows = {r["key"]: r for r in arts[ref_tag]["rows"]}

    # Column spec: (label, tag, backend)
    columns = [("vulkan", ref_tag, "vulkan"), ("hip", ref_tag, "hip"),
               ("hipgraph", ref_tag, "hipgraph"), ("redline", ref_tag, "redline")]
    for tag, _ in tagged[1:]:
        art = arts[tag]
        present = set()
        for r in art["rows"]:
            present.update(r.get("backends", {}).keys())
        for b in sorted(present):
            columns.append((f"{b}@{tag}", tag, b))

    rows_by_tag = {tag: {r["key"]: r for r in arts[tag]["rows"]} for tag in arts}

    stats = defaultdict(lambda: defaultdict(list))  # mode -> label -> [ratio]
    gate_fails = defaultdict(int)
    for key, ref_row in sorted(ref_rows.items()):
        mode = ref_row["mode"]
        if mode_filter and mode != mode_filter:
            continue
        vk, vk_state = cell(ref_row, "vulkan")
        if vk_state != "ok":
            gate_fails[f"vulkan/{vk_state}"] += 1
            continue
        for label, tag, backend in columns:
            row = rows_by_tag[tag].get(key)
            if row is None:
                gate_fails[f"{label}/missing-row"] += 1
                continue
            v, state = cell(row, backend)
            if state != "ok":
                gate_fails[f"{label}/{state}"] += 1
                continue
            stats[mode][label].append(v / vk)

    for mode in sorted(stats):
        print(f"\n## mode={mode}  (ratio vs {ref_tag} vulkan; <1 = faster than Vulkan)")
        print(f"{'column':24} {'n':>4} {'geomean':>9} {'median':>9} {'best':>9} {'worst':>9} {'wins':>5}")
        for label, _, _ in columns:
            r = stats[mode].get(label)
            if not r:
                continue
            r.sort()
            gm = math.exp(sum(math.log(x) for x in r) / len(r))
            med = r[len(r) // 2]
            wins = sum(1 for x in r if x < 1.0)
            print(f"{label:24} {len(r):4d} {gm:9.4f} {med:9.4f} {r[0]:9.4f} {r[-1]:9.4f} {wins:5d}")

    if gate_fails:
        print("\n## excluded cells (never silently ratioed)")
        for k, v in sorted(gate_fails.items()):
            print(f"  {k}: {v}")


if __name__ == "__main__":
    main()
