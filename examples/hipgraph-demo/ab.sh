#!/usr/bin/env bash
# A/B: stock HIP vs LD_PRELOAD libredline_hipgraph.so for explicit + capture.
set -euo pipefail
cd "$(dirname "$0")"

SO_PATH="${1:-../../target/release/libredline_hipgraph.so}"
export ROCR_VISIBLE_DEVICES="${ROCR_VISIBLE_DEVICES:-0}"
export HIP_VISIBLE_DEVICES="${HIP_VISIBLE_DEVICES:-0}"
export PATH="/opt/rocm/core/bin:${PATH:-}"
export ROCM_PATH="${ROCM_PATH:-/opt/rocm/core}"
export HIP_PATH="${HIP_PATH:-/opt/rocm/core}"
export HIP_CLANG_PATH="${HIP_CLANG_PATH:-/opt/rocm/core/lib/llvm/bin}"
export LD_LIBRARY_PATH="/opt/rocm/core/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"

if [[ ! -x ./graph_demo ]]; then
  echo "graph_demo missing; running build.sh first..." >&2
  bash ./build.sh
fi

if [[ ! -f "${SO_PATH}" ]]; then
  echo "warning: preload .so not found at ${SO_PATH}" >&2
  echo "         redline leg will still attempt LD_PRELOAD (may fall through)." >&2
fi

run_one() {
  local mode="$1"
  local preload="$2"
  local out
  if [[ -n "${preload}" ]]; then
    out="$(GRAPH_MODE="${mode}" LD_PRELOAD="${preload}" ./graph_demo 2>/dev/null || true)"
  else
    out="$(GRAPH_MODE="${mode}" ./graph_demo 2>/dev/null || true)"
  fi
  # Expect: CORRECT=<bool> MEDIAN_US=<f>
  local correct median
  correct="$(echo "${out}" | sed -n 's/.*CORRECT=\([^ ]*\).*/\1/p' | tail -n1)"
  median="$(echo "${out}" | sed -n 's/.*MEDIAN_US=\([^ ]*\).*/\1/p' | tail -n1)"
  if [[ -z "${correct}" ]]; then correct="?"; fi
  if [[ -z "${median}" ]]; then median="nan"; fi
  printf '%s\n' "${correct} ${median}"
}

printf '%-10s %-12s %-12s %-14s %-14s %s\n' \
  "MODE" "STOCK_OK" "REDLINE_OK" "STOCK_US" "REDLINE_US" "SPEEDUP"
printf '%-10s %-12s %-12s %-14s %-14s %s\n' \
  "----" "--------" "----------" "--------" "----------" "-------"

for mode in explicit capture; do
  stock="$(run_one "${mode}" "")"
  red="$(run_one "${mode}" "${SO_PATH}")"
  s_ok="$(echo "${stock}" | awk '{print $1}')"
  s_us="$(echo "${stock}" | awk '{print $2}')"
  r_ok="$(echo "${red}" | awk '{print $1}')"
  r_us="$(echo "${red}" | awk '{print $2}')"

  speedup="n/a"
  if [[ "${s_us}" != "nan" && "${r_us}" != "nan" ]]; then
    speedup="$(awk -v s="${s_us}" -v r="${r_us}" 'BEGIN{
      if (r+0 <= 0) { print "n/a"; exit }
      printf "%.3fx", s/r
    }')"
  fi

  printf '%-10s %-12s %-12s %-14s %-14s %s\n' \
    "${mode}" "${s_ok}" "${r_ok}" "${s_us}" "${r_us}" "${speedup}"
done

echo
echo "preload: ${SO_PATH}"
echo "GRAPH_N=${GRAPH_N:-64} GRAPH_M=${GRAPH_M:-200}"
