#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
#
# ROCm/ROCm#6529 open work item 3 — contention × REDLINE_PM4_FULL_STATE A/B.
#
# Honest scope: this is a probability-shifting stress harness, not a
# deterministic reproducer. A clean run does not prove absence of the bug;
# a fault under contention with FULL_STATE=0 that vanishes with FULL_STATE=1
# (or with cwsr_enable=0) only shifts confidence toward the mid-IB
# preemption / SH-elision hypothesis.
#
# Target binary: examples/hipfire-6409 hipfire-6409-bench
#   Cargo.toml package name = hipfire-6409-bench
# Bench flags used below are cited to examples/hipfire-6409/src/main.rs.

set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/6529-contention-ab.sh [options] [-- extra bench args]

Probability-shifting A/B for ROCm/ROCm#6529 (gfx1100 address-0 SQC fault).
Sweeps REDLINE_PM4_FULL_STATE={0,1} × contention={off,on} for R repetitions
of hipfire-6409-bench on a pinned ROCR device.

  --device SELECTOR      Anchored pin (preferred). Grammar:
                           uuid:GPU-… | bdf:0000:66:00.0 | slot:N | name:gfx…
                           | index:N | @alias
                         Resolved via device_list --resolve --risk reset against
                         the host manifest (deny + fragile enforced; this
                         harness provokes resets). Never pin by bare index
                         alone: ROCr discovery order ≠ rocm-smi PCI order, so
                         an ordinal from the wrong tool can land on the APU
                         whose reset takes the host down.
                         Must be set explicitly if --device is omitted.
                         HIP_VISIBLE_DEVICES is a CLR filter and does not
                         affect HSA enumeration.

Options:
  --device SELECTOR      See above. Documented path for multi-GPU hosts.
  --force                Allow non-gfx1100 / non-gfx11 agents (local smoke only).
  --reps N               Outer repetitions per cell (default: 3).
  --filter TEXT          Passed to bench --filter (main.rs:1024). Default:
                         serial_latency (long retained-PM4 serial chains).
  --max-rows N           Passed to bench --max-rows (main.rs:1025-1030).
                         No start-row / row-index-range flag exists in the
                         bench; subset selection is filter + max-rows only.
  --warmups N            Passed to bench --warmups (main.rs:1022). Default: 1.
  --samples N            Passed to bench --samples (main.rs:1023). Default: 3.
  --backends LIST        Passed to bench --backends (main.rs:1094-1096).
                         Subsets must include both redline and vulkan
                         (main.rs:1283-1284). Default: redline,vulkan.
  --matrix NAME          Passed to bench --matrix (main.rs:1098-1102).
                         Default: hipengine.
  --bench PATH           Path to hipfire-6409-bench binary. Default: auto-
                         detect under examples/hipfire-6409/target*/release/.
  --arch ARCH            HIPFIRE_BENCH_ARCH (main.rs:1007-1008) and hipcc
                         offload arch. Default: detected agent name.
  --out-root DIR         Results root. Default:
                         examples/hipfire-6409/results/<arch>/
  --skip-contention-build  Do not rebuild bench/contention_load.hip.
  --dry-run              Print the matrix plan and exit after preflight.
  -h, --help             Show this help.

Module-parameter A/Bs (cwsr_enable, mcbp) are NOT toggled by this script —
see docs/INTEGRATION.md "ROCm/ROCm#6529 contention A/B". Never runs modprobe.

Examples:
  scripts/6529-contention-ab.sh --device uuid:GPU-43390a851e296ee5
  scripts/6529-contention-ab.sh --device @dev0 --reps 5 --max-rows 16
  scripts/6529-contention-ab.sh --device bdf:0000:66:00.0
  ROCR_VISIBLE_DEVICES=0 scripts/6529-contention-ab.sh --force   # gfx1201 smoke
EOF
}

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd "${SCRIPT_DIR}/.." && pwd)

FORCE=0
REPS=3
FILTER="serial_latency"
MAX_ROWS=""
WARMUPS=1
SAMPLES=3
BACKENDS="redline,vulkan"
MATRIX="hipengine"
BENCH_PATH=""
ARCH_OVERRIDE=""
OUT_ROOT=""
SKIP_CONTENTION_BUILD=0
DRY_RUN=0
DEVICE_SELECTOR=""
DEVICE_IDENTITY=""
DEVICE_BDF=""
EXTRA_BENCH_ARGS=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --force) FORCE=1; shift ;;
    --device)
      DEVICE_SELECTOR=${2:?--device requires SELECTOR}
      shift 2
      ;;
    --reps)
      REPS=${2:?--reps requires N}
      shift 2
      ;;
    --filter)
      FILTER=${2:?--filter requires TEXT}
      shift 2
      ;;
    --max-rows)
      MAX_ROWS=${2:?--max-rows requires N}
      shift 2
      ;;
    --warmups)
      WARMUPS=${2:?--warmups requires N}
      shift 2
      ;;
    --samples)
      SAMPLES=${2:?--samples requires N}
      shift 2
      ;;
    --backends)
      BACKENDS=${2:?--backends requires LIST}
      shift 2
      ;;
    --matrix)
      MATRIX=${2:?--matrix requires NAME}
      shift 2
      ;;
    --bench)
      BENCH_PATH=${2:?--bench requires PATH}
      shift 2
      ;;
    --arch)
      ARCH_OVERRIDE=${2:?--arch requires ARCH}
      shift 2
      ;;
    --out-root)
      OUT_ROOT=${2:?--out-root requires DIR}
      shift 2
      ;;
    --skip-contention-build) SKIP_CONTENTION_BUILD=1; shift ;;
    --dry-run) DRY_RUN=1; shift ;;
    -h|--help) usage; exit 0 ;;
    --) shift; EXTRA_BENCH_ARGS+=("$@"); break ;;
    *)
      echo "error: unknown argument: $1 (use -- to pass through to the bench)" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if ! [[ "$REPS" =~ ^[1-9][0-9]*$ ]]; then
  echo "error: --reps must be a positive integer (got ${REPS})" >&2
  exit 2
fi

# --- device pin -------------------------------------------------------------
# Prefer --device (anchored). ROCR_VISIBLE_DEVICES remains a fallback for
# hosts without a manifest. An index alone is not a safe pin: ROCr discovery
# order ≠ rocm-smi PCI order.

resolve_device_list_bin() {
  # Prefer a prebuilt binary; otherwise build release into a stable target dir.
  local candidates=(
    "${REPO_ROOT}/target/release/examples/device_list"
    "${CARGO_TARGET_DIR:-}/release/examples/device_list"
  )
  local c
  for c in "${candidates[@]}"; do
    if [[ -n "$c" && -x "$c" ]]; then
      printf '%s\n' "$c"
      return
    fi
  done
  local td="${CARGO_TARGET_DIR:-${REPO_ROOT}/target}"
  echo "building device_list (release) into ${td} …" >&2
  if ! CARGO_TARGET_DIR="$td" cargo build -p redline-rocr --example device_list --release \
      --manifest-path "${REPO_ROOT}/Cargo.toml" >&2; then
    echo "error: failed to build device_list example" >&2
    exit 2
  fi
  local bin="${td}/release/examples/device_list"
  if [[ ! -x "$bin" ]]; then
    echo "error: device_list binary missing after build: ${bin}" >&2
    exit 2
  fi
  printf '%s\n' "$bin"
}

if [[ -n "$DEVICE_SELECTOR" ]]; then
  DEVICE_LIST_BIN=$(resolve_device_list_bin)
  # Capture stdout (resolve line) and stderr separately. Nonzero exit aborts
  # before any GPU work — deny/fragile (exit 3) must never reach a reset-provoking cell.
  resolve_err=$(mktemp)
  set +e
  resolve_out=$("$DEVICE_LIST_BIN" --resolve "$DEVICE_SELECTOR" --risk reset 2>"$resolve_err")
  resolve_rc=$?
  set -e
  if [[ "$resolve_rc" -ne 0 ]]; then
    cat "$resolve_err" >&2
    rm -f "$resolve_err"
    case "$resolve_rc" in
      3)
        echo "error: --device ${DEVICE_SELECTOR} is denied or fragile (reset-risk) on the host manifest; aborting before any GPU work." >&2
        exit 3
        ;;
      4)
        echo "error: --device ${DEVICE_SELECTOR} failed to resolve (not found / ambiguous / parse error)." >&2
        exit 4
        ;;
      *)
        echo "error: device_list --resolve failed (exit ${resolve_rc})." >&2
        exit "$resolve_rc"
        ;;
    esac
  fi
  rm -f "$resolve_err"
  # Line: <unfiltered_rocr_index>\t<bdf>\t<describe()>
  IFS=$'\t' read -r _rocr_idx DEVICE_BDF DEVICE_IDENTITY <<<"$resolve_out"
  if [[ -z "${_rocr_idx}" || -z "${DEVICE_IDENTITY}" ]]; then
    echo "error: device_list --resolve returned a malformed line: ${resolve_out}" >&2
    exit 1
  fi
  export ROCR_VISIBLE_DEVICES="${_rocr_idx}"
elif [[ -z "${ROCR_VISIBLE_DEVICES+x}" || -z "${ROCR_VISIBLE_DEVICES}" ]]; then
  cat >&2 <<'EOF'
error: device pin required: pass --device <selector> or set ROCR_VISIBLE_DEVICES.

Why: HIP_VISIBLE_DEVICES is a CLR (HIP) filter and does not affect HSA
enumeration. Redline and hipfire-6409-bench select agents via ROCr/HSA, which
honors ROCR_VISIBLE_DEVICES only. On a multi-GPU host an unpinned run may
bind the wrong GPU — including an APU whose device reset takes the host down.

Prefer an anchored selector (ROCr order ≠ rocm-smi PCI order):
  scripts/6529-contention-ab.sh --device uuid:GPU-43390a851e296ee5
  scripts/6529-contention-ab.sh --device @dev0
Fallback when no manifest is available:
  ROCR_VISIBLE_DEVICES=0 scripts/6529-contention-ab.sh
EOF
  exit 2
else
  DEVICE_IDENTITY="ROCR_VISIBLE_DEVICES=${ROCR_VISIBLE_DEVICES} (unresolved ordinal fallback)"
  DEVICE_BDF=""
fi

# --- agent detection --------------------------------------------------------
detect_agent() {
  local info name
  if ! command -v rocminfo >/dev/null 2>&1; then
    echo "error: rocminfo not found on PATH; cannot detect GPU agent" >&2
    exit 2
  fi
  # First GPU agent Name: line after ROCR filtering. CPU agents are skipped.
  info=$(rocminfo 2>/dev/null || true)
  if [[ -z "$info" ]]; then
    echo "error: rocminfo produced no output" >&2
    exit 2
  fi
  name=$(
    awk '
      /^  Name:/ {
        n=$0
        sub(/^  Name:[[:space:]]*/, "", n)
        gsub(/[[:space:]]+$/, "", n)
        if (n ~ /^gfx/) { print n; exit }
      }
    ' <<<"$info"
  )
  if [[ -z "$name" ]]; then
    # Fallback: Marketing Name / Device Name style if Name is a product string.
    name=$(
      awk '
        /Device Type:[[:space:]]*GPU/ { gpu=1; next }
        gpu && /^  Name:/ {
          n=$0
          sub(/^  Name:[[:space:]]*/, "", n)
          gsub(/[[:space:]]+$/, "", n)
          print n
          exit
        }
      ' <<<"$info"
    )
  fi
  if [[ -z "$name" ]]; then
    echo "error: could not parse a GPU agent from rocminfo" >&2
    exit 2
  fi
  printf '%s\n' "$name"
}

AGENT=$(detect_agent)
ARCH=${ARCH_OVERRIDE:-$AGENT}

is_gfx11_class() {
  local a=$1
  # gfx1100 (Navi31), gfx1101, gfx1102, gfx1103, gfx1150/1151 (Strix), etc.
  [[ "$a" == gfx11* ]]
}

echo "=== ROCm/ROCm#6529 contention A/B preflight ==="
echo "repo_root:              ${REPO_ROOT}"
echo "device_selector:        ${DEVICE_SELECTOR:-<none — ROCR_VISIBLE_DEVICES fallback>}"
echo "device_identity:        ${DEVICE_IDENTITY}"
if [[ -n "$DEVICE_BDF" ]]; then
  echo "device_bdf:             ${DEVICE_BDF}"
fi
echo "ROCR_VISIBLE_DEVICES:   ${ROCR_VISIBLE_DEVICES}"
echo "HIP_VISIBLE_DEVICES:    ${HIP_VISIBLE_DEVICES:-<unset>}"
echo "detected_agent:         ${AGENT}"
echo "bench_arch:             ${ARCH}"

CWSR_PATH=/sys/module/amdgpu/parameters/cwsr_enable
MCBP_PATH=/sys/module/amdgpu/parameters/mcbp
if [[ -r "$CWSR_PATH" ]]; then
  CWSR_VAL=$(cat "$CWSR_PATH")
else
  CWSR_VAL="<unreadable ${CWSR_PATH}>"
fi
if [[ -r "$MCBP_PATH" ]]; then
  MCBP_VAL=$(cat "$MCBP_PATH")
else
  MCBP_VAL="<unreadable ${MCBP_PATH}>"
fi
echo "amdgpu.cwsr_enable:     ${CWSR_VAL}"
echo "amdgpu.mcbp:            ${MCBP_VAL}"
echo "note: this script never changes module parameters (no modprobe/reboot)."

if ! is_gfx11_class "$AGENT"; then
  if [[ "$FORCE" -eq 1 ]]; then
    echo "warning: agent '${AGENT}' is not gfx11-class; continuing due to --force" >&2
  else
    cat >&2 <<EOF
error: detected agent '${AGENT}' is not gfx1100/gfx11-class.

ROCm/ROCm#6529 is a gfx1100 (RDNA3) investigation. Refusing to run the
contention matrix on this device so a local gfx1201 box cannot be mistaken
for a gfx1100 result.

Re-run with --force only for harness smoke (argument parsing / preflight),
or pin a gfx11 GPU with --device uuid:… / --device @alias.
EOF
    exit 3
  fi
fi

# --- locate bench -----------------------------------------------------------
resolve_bench() {
  if [[ -n "$BENCH_PATH" ]]; then
    if [[ ! -x "$BENCH_PATH" ]]; then
      echo "error: --bench path is not executable: ${BENCH_PATH}" >&2
      exit 2
    fi
    printf '%s\n' "$BENCH_PATH"
    return
  fi
  local candidates=(
    "${REPO_ROOT}/examples/hipfire-6409/target/release/hipfire-6409-bench"
    "${REPO_ROOT}/examples/hipfire-6409/target/${ARCH}/release/hipfire-6409-bench"
    "${REPO_ROOT}/target/release/hipfire-6409-bench"
  )
  local c
  for c in "${candidates[@]}"; do
    if [[ -x "$c" ]]; then
      printf '%s\n' "$c"
      return
    fi
  done
  cat >&2 <<EOF
error: hipfire-6409-bench not found.

Build it first (from examples/hipfire-6409):
  HIPFIRE_BENCH_ARCH=${ARCH} cargo build --release --bin hipfire-6409-bench

Or pass --bench /path/to/hipfire-6409-bench
EOF
  exit 2
}

BENCH=$(resolve_bench)
echo "bench_binary:           ${BENCH}"

# --- results dir (match examples/hipfire-6409/results/<arch>/...) -----------
TS=$(date -u +%Y%m%dT%H%M%SZ)
if [[ -z "$OUT_ROOT" ]]; then
  OUT_ROOT="${REPO_ROOT}/examples/hipfire-6409/results/${ARCH}"
fi
RUN_DIR="${OUT_ROOT}/6529-contention-ab-${TS}"
echo "run_dir:                ${RUN_DIR}"

# --- dmesg helpers ----------------------------------------------------------
DMESG_OK=1
DMESG_NOTE=""
if ! dmesg -T >/dev/null 2>&1; then
  DMESG_OK=0
  DMESG_NOTE="dmesg not readable without elevated privileges; kernel fault capture disabled for this run"
  echo "warning: ${DMESG_NOTE}" >&2
fi

dmesg_cursor_mark() {
  # Return a unique marker string and inject it if possible; else use timestamp.
  if [[ "$DMESG_OK" -eq 0 ]]; then
    date -u +%Y-%m-%dT%H:%M:%S
    return
  fi
  # Prefer boot-time monotonic seconds from /proc/uptime as a cursor.
  awk '{print $1}' /proc/uptime
}

dmesg_since() {
  local cursor=$1
  if [[ "$DMESG_OK" -eq 0 ]]; then
    echo "[${DMESG_NOTE}]"
    return
  fi
  # Emit lines whose kernel timestamp (seconds) is >= cursor when dmesg -T
  # is unavailable for parsing; fall back to full filtered tail via dmesg --since
  # is not portable. Use dmesg -t (raw seconds) when supported.
  if dmesg -t >/dev/null 2>&1; then
    dmesg -t 2>/dev/null | awk -v c="$cursor" '
      {
        ts=$1
        sub(/\]/, "", ts)
        # lines look like: [ 12345.678901] msg  OR 12345.678901 msg
        if (ts ~ /^[0-9]+\.[0-9]+$/) {
          if ((ts + 0) + 0 >= (c + 0)) print
        } else if ($1 ~ /^\[/) {
          raw=$1
          gsub(/[\[\]]/, "", raw)
          if ((raw + 0) >= (c + 0)) print
        }
      }
    ' | grep -E 'amdgpu|VM_L2_PROTECTION_FAULT|SQC|MES|page fault|PERMISSION_FAULT' || true
  else
    # Last-resort: dump matching recent lines (no reliable cursor).
    dmesg -T 2>/dev/null | grep -E 'amdgpu|VM_L2_PROTECTION_FAULT|SQC|MES|page fault|PERMISSION_FAULT' | tail -n 200 || true
  fi
}

# --- contention binary ------------------------------------------------------
CONTENTION_SRC="${REPO_ROOT}/bench/contention_load.hip"
CONTENTION_BIN="${RUN_DIR}/contention_load"
HIPCC=${HIPCC:-}
if [[ -z "$HIPCC" ]]; then
  if [[ -x /opt/rocm/core/bin/hipcc ]]; then
    HIPCC=/opt/rocm/core/bin/hipcc
  elif command -v hipcc >/dev/null 2>&1; then
    HIPCC=$(command -v hipcc)
  else
    HIPCC=""
  fi
fi

build_contention() {
  if [[ "$SKIP_CONTENTION_BUILD" -eq 1 && -x "$CONTENTION_BIN" ]]; then
    return
  fi
  if [[ ! -f "$CONTENTION_SRC" ]]; then
    echo "error: missing contention source: ${CONTENTION_SRC}" >&2
    exit 2
  fi
  if [[ -z "$HIPCC" ]]; then
    echo "error: hipcc not found; cannot build contention load" >&2
    exit 2
  fi
  echo "building contention load: ${CONTENTION_BIN}"
  "$HIPCC" --offload-arch="${ARCH}" "$CONTENTION_SRC" -o "$CONTENTION_BIN"
}

if [[ "$DRY_RUN" -eq 1 ]]; then
  echo "dry-run plan:"
  for fs in 0 1; do
    for ct in off on; do
      for ((r = 1; r <= REPS; r++)); do
        echo "  cell full_state=${fs} contention=${ct} rep=${r}"
      done
    done
  done
  echo "dry-run complete; nothing executed."
  exit 0
fi

mkdir -p "${RUN_DIR}/cells"
build_contention

# --- meta -------------------------------------------------------------------
{
  echo "tracker: ROCm/ROCm#6529"
  echo "honest_scope: probability-shifting stress harness, not a deterministic reproducer"
  echo "timestamp_utc: ${TS}"
  echo "repo_root: ${REPO_ROOT}"
  echo "device_selector: ${DEVICE_SELECTOR:-}"
  echo "device_identity: ${DEVICE_IDENTITY}"
  echo "device_bdf: ${DEVICE_BDF}"
  echo "ROCR_VISIBLE_DEVICES: ${ROCR_VISIBLE_DEVICES}"
  echo "HIP_VISIBLE_DEVICES: ${HIP_VISIBLE_DEVICES:-}"
  echo "detected_agent: ${AGENT}"
  echo "bench_arch: ${ARCH}"
  echo "amdgpu.cwsr_enable: ${CWSR_VAL}"
  echo "amdgpu.mcbp: ${MCBP_VAL}"
  echo "bench_binary: ${BENCH}"
  echo "reps: ${REPS}"
  echo "filter: ${FILTER}"
  echo "max_rows: ${MAX_ROWS:-<unset>}"
  echo "warmups: ${WARMUPS}"
  echo "samples: ${SAMPLES}"
  echo "backends: ${BACKENDS}"
  echo "matrix: ${MATRIX}"
  echo "extra_bench_args: ${EXTRA_BENCH_ARGS[*]-}"
  echo "dmesg_capture: $([[ "$DMESG_OK" -eq 1 ]] && echo enabled || echo disabled)"
  echo "dmesg_note: ${DMESG_NOTE}"
  echo "force: ${FORCE}"
} | tee "${RUN_DIR}/meta.txt"
printf '%s\n' "${DEVICE_IDENTITY}" >"${RUN_DIR}/device_identity.txt"

SUMMARY_TSV="${RUN_DIR}/summary.tsv"
printf 'cell\trep\tfull_state\tcontention\texit\tfaults\tstatus\tlogfile\n' >"$SUMMARY_TSV"

# Cell axes
FULL_STATES=(0 1)
CONTENTIONS=(off on)

fault_count_in() {
  local f=$1
  if [[ ! -s "$f" ]]; then
    echo 0
    return
  fi
  # Count distinctive fault signatures, not every amdgpu noise line.
  grep -cE 'VM_L2_PROTECTION_FAULT|SQC \(data\)|page fault at address 0x0|PERMISSION_FAULTS|MES +REMOVE_QUEUE|mode1.?reset|GPU reset' "$f" 2>/dev/null || echo 0
}

run_cell() {
  local full_state=$1
  local contention=$2
  local rep=$3
  local cell="fs${full_state}_ct${contention}_r${rep}"
  local cell_dir="${RUN_DIR}/cells/${cell}"
  mkdir -p "$cell_dir"

  local out_json="${cell_dir}/results.json"
  local stdout_log="${cell_dir}/stdout.log"
  local stderr_log="${cell_dir}/stderr.log"
  local dmesg_log="${cell_dir}/dmesg_delta.log"
  local status_file="${cell_dir}/status.txt"

  local bench_cmd=(
    env
    "ROCR_VISIBLE_DEVICES=${ROCR_VISIBLE_DEVICES}"
    "HIPFIRE_BENCH_ARCH=${ARCH}"
    "REDLINE_PM4_FULL_STATE=${full_state}"
    "$BENCH"
    --matrix "$MATRIX"
    --backends "$BACKENDS"
    --filter "$FILTER"
    --warmups "$WARMUPS"
    --samples "$SAMPLES"
    --out "$out_json"
  )
  # --max-rows only when requested (main.rs:1025-1030)
  if [[ -n "$MAX_ROWS" ]]; then
    bench_cmd+=(--max-rows "$MAX_ROWS")
  fi
  if [[ ${#EXTRA_BENCH_ARGS[@]} -gt 0 ]]; then
    bench_cmd+=("${EXTRA_BENCH_ARGS[@]}")
  fi

  {
    echo "cell=${cell}"
    echo "REDLINE_PM4_FULL_STATE=${full_state}"
    echo "contention=${contention}"
    echo "rep=${rep}"
    echo "cmd=${bench_cmd[*]}"
  } >"${cell_dir}/cmdline.txt"

  local contention_pid=""
  cleanup_contention() {
    if [[ -n "$contention_pid" ]] && kill -0 "$contention_pid" 2>/dev/null; then
      kill "$contention_pid" 2>/dev/null || true
      wait "$contention_pid" 2>/dev/null || true
    fi
  }
  trap cleanup_contention RETURN

  if [[ "$contention" == "on" ]]; then
    # Long enough to outlive the bench cell; killed on RETURN.
    env ROCR_VISIBLE_DEVICES="${ROCR_VISIBLE_DEVICES}" \
      "$CONTENTION_BIN" 3600 \
      >"${cell_dir}/contention.stdout" \
      2>"${cell_dir}/contention.stderr" &
    contention_pid=$!
    # Brief settle so the loader's first launches hit the device.
    sleep 0.5
    if ! kill -0 "$contention_pid" 2>/dev/null; then
      echo "error: contention load exited immediately; see ${cell_dir}/contention.stderr" >&2
      echo "fail" >"$status_file"
      printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
        "$cell" "$rep" "$full_state" "$contention" "spawn_fail" "0" "fail" "$cell_dir" \
        >>"$SUMMARY_TSV"
      return 0
    fi
  fi

  local cursor
  cursor=$(dmesg_cursor_mark)

  set +e
  "${bench_cmd[@]}" >"$stdout_log" 2>"$stderr_log"
  local rc=$?
  set -e

  cleanup_contention
  trap - RETURN

  dmesg_since "$cursor" >"$dmesg_log"
  # Also scrape stderr for fault-ish strings the runtime may print.
  cat "$stderr_log" >>"${cell_dir}/fault_scan_input.txt" 2>/dev/null || true
  cat "$dmesg_log" >>"${cell_dir}/fault_scan_input.txt" 2>/dev/null || true
  local faults
  faults=$(fault_count_in "${cell_dir}/fault_scan_input.txt")
  # Normalize possible "0\n0" from grep -c || echo
  faults=$(echo "$faults" | awk '{s+=$1} END{print s+0}')

  local status="pass"
  if [[ "$rc" -ne 0 || "$faults" -gt 0 ]]; then
    status="fail"
  fi

  {
    echo "exit=${rc}"
    echo "faults=${faults}"
    echo "status=${status}"
    echo "full_state=${full_state}"
    echo "contention=${contention}"
  } | tee "$status_file"

  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$cell" "$rep" "$full_state" "$contention" "$rc" "$faults" "$status" "$cell_dir" \
    >>"$SUMMARY_TSV"

  echo "--- cell ${cell}: exit=${rc} faults=${faults} status=${status}"
}

echo "=== running matrix: full_state×contention×reps=${REPS} ==="
for fs in "${FULL_STATES[@]}"; do
  for ct in "${CONTENTIONS[@]}"; do
    for ((r = 1; r <= REPS; r++)); do
      run_cell "$fs" "$ct" "$r"
    done
  done
done

# --- aggregate summary table ------------------------------------------------
SUMMARY_MD="${RUN_DIR}/SUMMARY.md"
{
  echo "# ROCm/ROCm#6529 contention A/B summary"
  echo
  echo "Generated: ${TS} UTC"
  echo
  echo "**Honest scope:** this harness shifts fault probability under GPU"
  echo "contention; it is **not** a deterministic reproducer. A clean matrix"
  echo "**cannot prove absence** of the bug. Interpret only as supporting or"
  echo "weakening the mid-IB CWSR/MCBP × SH-elision hypothesis."
  echo
  echo "## Device"
  echo
  echo "- device_selector: \`${DEVICE_SELECTOR:-<ROCR_VISIBLE_DEVICES fallback>}\`"
  echo "- device_identity: \`${DEVICE_IDENTITY}\`"
  if [[ -n "$DEVICE_BDF" ]]; then
    echo "- device_bdf: \`${DEVICE_BDF}\`"
  fi
  echo "- ROCR_VISIBLE_DEVICES: \`${ROCR_VISIBLE_DEVICES}\`"
  echo
  echo "| cell | rep | FULL_STATE | contention | exit | faults | status |"
  echo "| --- | ---: | ---: | --- | ---: | ---: | --- |"
  tail -n +2 "$SUMMARY_TSV" | awk -F'\t' '{
    printf "| %s | %s | %s | %s | %s | %s | %s |\n", $1,$2,$3,$4,$5,$6,$7
  }'
  echo
  echo "## Totals by axis"
  echo
  echo '```'
  awk -F'\t' 'NR>1 {
    key=sprintf("full_state=%s contention=%s", $3, $4)
    n[key]++; faults[key]+=$6; fail[key]+=($7=="fail")
  }
  END {
    for (k in n) {
      printf "%-40s  runs=%d  fails=%d  fault_hits=%d\n", k, n[k], fail[k], faults[k]
    }
  }' "$SUMMARY_TSV"
  echo '```'
  echo
  echo "## Host controls (unchanged by this script)"
  echo
  echo "- detected_agent: \`${AGENT}\`"
  echo "- amdgpu.cwsr_enable: \`${CWSR_VAL}\`"
  echo "- amdgpu.mcbp: \`${MCBP_VAL}\`"
  echo "- ROCR_VISIBLE_DEVICES: \`${ROCR_VISIBLE_DEVICES}\`"
  echo
  echo "Module-parameter A/Bs (manual reboot/reload) are documented in"
  echo "docs/INTEGRATION.md § ROCm/ROCm#6529 contention A/B."
} | tee "$SUMMARY_MD"

echo
echo "=== final summary table ==="
column -t -s $'\t' "$SUMMARY_TSV" 2>/dev/null || cat "$SUMMARY_TSV"
echo
echo "results: ${RUN_DIR}"
echo "summary: ${SUMMARY_MD}"
