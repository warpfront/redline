#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
"""Summarise how a rocprofv3 kernel-dispatch trace distributed work across
hardware queues, per graph shape.

Why this exists
---------------
Wall-clock timing can show that two ROCm releases behave differently for the
same graph, but it cannot show *why*. rocprofv3's kernel-dispatch records carry
`dispatch_info.queue_id`, which is the runtime's own account of which hardware
queue each dispatch landed on, so the queue distribution is direct evidence
about scheduling rather than an inference from timing.

ROCm 10.0 additionally tags each dispatch with `graph_exec_id` and
`graph_node_id`. When those are present this script groups by `graph_exec_id`,
which is exact. When they are absent (7.14) it falls back to splitting the
time-ordered dispatch stream into fixed-size blocks, which is only valid if the
producing probe ran a known number of equal-sized phases -- so that fallback is
labelled in the output and must not be read as authoritative grouping.

What it deliberately does NOT do
--------------------------------
It does not report a per-dispatch cost derived from the record span. A span from
first-start to last-end across many graph launches includes the idle gaps
between launches, so dividing it by the dispatch count produces a number that
looks like a per-dispatch cost but is not one. Timing comes from the probe's own
matched-pair measurement; this script is only for scheduling evidence.

Usage
-----
  analyze_dispatch_queues.py <results.json> [--phases chain,independent,fanout]
                             [--per-phase N]
"""

import argparse
import collections
import json
import sys


def queue_of(rec):
    """Extract a comparable hardware-queue identity from a dispatch record."""
    q = rec.get("dispatch_info", {}).get("queue_id")
    if isinstance(q, dict):
        return q.get("handle", q.get("id"))
    return q


def stream_of(rec):
    s = rec.get("stream_id")
    if isinstance(s, dict):
        return s.get("handle", s.get("id"))
    return s


def load_dispatches(path):
    with open(path) as fh:
        doc = json.load(fh)
    tool = doc["rocprofiler-sdk-tool"][0]
    recs = tool["buffer_records"]["kernel_dispatch"]
    return sorted(recs, key=lambda r: r["start_timestamp"])


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("results")
    ap.add_argument("--phases", default="chain,independent,fanout")
    ap.add_argument("--per-phase", type=int, default=0,
                    help="dispatches per phase; required for the fallback path")
    args = ap.parse_args()

    recs = load_dispatches(args.results)
    phases = [p for p in args.phases.split(",") if p]

    have_graph_ids = any(r.get("graph_exec_id") for r in recs)
    print(f"total kernel_dispatch records: {len(recs)}")
    print(f"graph_exec_id present: {have_graph_ids}")
    print(f"distinct hardware queues seen overall: "
          f"{len({queue_of(r) for r in recs})}")
    print()

    groups = []
    if have_graph_ids:
        by_exec = collections.defaultdict(list)
        for r in recs:
            by_exec[r.get("graph_exec_id", 0)].append(r)
        # graph_exec_id 0 is non-graph work (e.g. memset blits); drop it.
        for key in sorted(k for k in by_exec if k):
            groups.append((f"graph_exec {key}", by_exec[key]))
        grouping = "exact (graph_exec_id)"
    else:
        if not args.per_phase:
            print("ERROR: this trace has no graph_exec_id, so --per-phase is "
                  "required to split phases, and the split is heuristic.",
                  file=sys.stderr)
            return 2
        n = args.per_phase
        # Graph dispatches are the trailing len(phases)*n records; anything
        # before that is setup work.
        body = recs[-len(phases) * n:]
        for i, name in enumerate(phases):
            blk = body[i * n:(i + 1) * n]
            if blk:
                groups.append((name, blk))
        grouping = f"HEURISTIC (fixed blocks of {n}, no graph ids in trace)"

    print(f"grouping: {grouping}")
    print()
    hdr = f"{'group':<16} {'disp':>5} {'queues':>7} {'streams':>8}  distribution"
    print(hdr)
    print("-" * len(hdr))
    for name, blk in groups:
        qs = collections.Counter(queue_of(r) for r in blk)
        ss = len({stream_of(r) for r in blk})
        dist = ", ".join(f"q{k}:{v}" for k, v in sorted(qs.items(),
                                                        key=lambda kv: -kv[1]))
        print(f"{name:<16} {len(blk):>5} {len(qs):>7} {ss:>8}  {dist}")

    print()
    print("Queue distribution is the evidence here. An even spread across "
          "several queues\nmeans the runtime scheduled that shape "
          "concurrently; everything on one queue\nmeans it serialised the "
          "shape regardless of the declared dependencies.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
