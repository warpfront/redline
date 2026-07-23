#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# Clean A/B: pristine hipEngine dispatch microbench (stock hipGraph) vs the
# redline-hipgraph drop-in via LD_PRELOAD. No configuration, no hand-lowering —
# the ONLY difference between the two runs is LD_PRELOAD of libredline_hipgraph.so,
# which transparently replays the captured hipGraph as a retained PM4 IB.
#
# Usage: bash run_ab.sh [counts] [reps]
set -euo pipefail

WT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"          # worktree root
ENGINE="$WT/.engines/hipEngine"                                    # pristine hipEngine
SO="$WT/target/release/libredline_hipgraph.so"                    # default (C-only) build
COUNTS="${1:-1,50,200,941}"
REPS="${2:-40}"

export PATH="/opt/rocm/core/bin:$PATH" ROCM_PATH=/opt/rocm/core HIP_PATH=/opt/rocm/core
export HIP_CLANG_PATH=/opt/rocm/core/lib/llvm/bin
export ROCR_VISIBLE_DEVICES=0 HIP_VISIBLE_DEVICES=0 HIPENGINE_HIP_ARCH=gfx1201
export PYTHONPATH="$ENGINE"

if [ ! -f "$SO" ]; then
  echo "building redline-hipgraph (default C-only build) ..."
  ( cd "$WT" && cargo build --release -p redline-hipgraph >/dev/null 2>&1 )
fi

MB="$ENGINE/scripts/graph_node_microbench.py"
echo "control (stock hipGraph)  -> /tmp/he714-control.json"
python3 "$MB" --counts "$COUNTS" --timing-mode serial_latency --reps "$REPS" --warmup 10 --json /tmp/he714-control.json >/dev/null
echo "drop-in (LD_PRELOAD PM4)  -> /tmp/he714-redline.json"
LD_PRELOAD="$SO" python3 "$MB" --counts "$COUNTS" --timing-mode serial_latency --reps "$REPS" --warmup 10 --json /tmp/he714-redline.json >/dev/null

python3 - <<'PY'
import json, statistics
c={x['node_count']:x for x in json.load(open('/tmp/he714-control.json'))['rows']}
r={x['node_count']:x for x in json.load(open('/tmp/he714-redline.json'))['rows']}
m=statistics.median
print(f"\n{'count':>6} | {'stock_host_us':>13} {'redline_host_us':>15} {'speedup':>8} | correct")
print("-"*62)
for n in sorted(c):
    ch=m(c[n]['burst_host_samples_us']); rh=m(r[n]['burst_host_samples_us'])
    print(f"{n:6} | {ch:13.2f} {rh:15.2f} {ch/rh:7.2f}x | {r[n]['burst_correctness_pass']}")
PY
