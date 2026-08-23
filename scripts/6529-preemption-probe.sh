#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
#
# ROCm/ROCm#6529 preemption probe.
#
# Tests one specific mechanism: can mid-IB compute preemption lose the SH
# register state (COMPUTE_PGM_LO/HI, COMPUTE_USER_DATA) that a retained PM4
# indirect buffer wrote earlier and later dispatches inherit ("stateful
# elision")? If it can, a preempted elided IB dispatches from address 0, which
# is the ROCm#6529 fault tuple.
#
# Method: run long retained IBs while N independent GPU processes oversubscribe
# the device, so MES must time-slice the queue repeatedly *inside* a single
# indirect buffer. Count faults.
#
# This is deliberately dependency-light: it drives the redline C-ABI example
# harnesses, not the full hipfire benchmark matrix (which additionally needs
# Vulkan/glslc). scripts/6529-contention-ab.sh is the full-matrix counterpart.
#
# A null result does NOT prove absence: it bounds how often the tested
# mechanism fires, nothing more.

set -euo pipefail

DEVICE=""
TOKENS=12000
REPS=6
CONTENDERS=16
CONTENTION_SECONDS=400
OUT=""
FULL_STATE_ARMS="0 1"

usage() {
    cat <<'EOF'
Usage: scripts/6529-preemption-probe.sh --device SELECTOR [options]

  --device SELECTOR   Anchored device pin: uuid:… | bdf:… | slot:N | name:… |
                      index:N | @alias. Resolved through the host manifest;
                      deny- or fragile-listed devices abort before any GPU work
                      (fragile enforced because this harness provokes resets).
  --tokens N          Tokens per retained IB (2 dispatches each). Default 12000
                      = 24000 dispatches, sized to stay under the 20-bit
                      INDIRECT_BUFFER ceiling in full-state mode.
  --reps N            Measured repetitions per arm. Default 6.
  --contenders N      Independent GPU processes competing for the device.
                      Default 16. Use 0 for the no-contention control.
  --arms "0 1"        REDLINE_PM4_FULL_STATE values to sweep. Default "0 1".
  --out PATH          Write the JSON artifact here. Default: stdout only.
  -h, --help          This text.

Requires, already built for the target architecture:
  $TARGET_DIR/release/examples/device_list, libredline_dispatch.so,
  decode_chain_ab, and a decode_chain code object. See the header of
  crates/redline-capi/examples/decode_chain_ab.c for the build line.
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --device) DEVICE="${2:-}"; shift 2 ;;
        --tokens) TOKENS="${2:-}"; shift 2 ;;
        --reps) REPS="${2:-}"; shift 2 ;;
        --contenders) CONTENDERS="${2:-}"; shift 2 ;;
        --contention-seconds) CONTENTION_SECONDS="${2:-}"; shift 2 ;;
        --arms) FULL_STATE_ARMS="${2:-}"; shift 2 ;;
        --out) OUT="${2:-}"; shift 2 ;;
        -h|--help) usage; exit 0 ;;
        *) echo "error: unknown argument $1" >&2; usage >&2; exit 2 ;;
    esac
done

: "${TARGET_DIR:=/tmp/rl-probe}"
: "${CHAIN_AB:=/tmp/chain_ab}"
: "${CHAIN_CO:?set CHAIN_CO to the decode_chain code object for this arch}"
: "${CONTENTION_BIN:=/tmp/contention_load}"
DEVICE_LIST="${TARGET_DIR}/release/examples/device_list"

if [[ -z "$DEVICE" ]]; then
    echo "error: --device is required. An index copied from the wrong tool can" >&2
    echo "select the wrong GPU: ROCr discovery order is not rocm-smi PCI order." >&2
    exit 2
fi
for binary in "$DEVICE_LIST" "$CHAIN_AB" "$CHAIN_CO"; do
    [[ -e "$binary" ]] || { echo "error: missing $binary" >&2; exit 2; }
done

# Resolve with the manifest deny + fragile lists enforced (fragile under reset risk),
# and with ROCR_VISIBLE_DEVICES unset so the reported index is the unfiltered one we then pin children to.
resolve_err=$(mktemp)
set +e
resolved=$(env -u ROCR_VISIBLE_DEVICES "$DEVICE_LIST" --resolve "$DEVICE" --risk reset 2>"$resolve_err")
resolve_rc=$?
set -e
if [[ $resolve_rc -ne 0 ]]; then
  cat "$resolve_err" >&2
  rm -f "$resolve_err"
  case "$resolve_rc" in
    3) echo "error: --device $DEVICE is denied or fragile (reset-risk) on the host manifest; aborting before any GPU work." >&2; exit 3 ;;
    4) echo "error: --device $DEVICE failed to resolve (not found / ambiguous / parse error)." >&2; exit 4 ;;
    *) echo "error: device_list --resolve failed (exit $resolve_rc)." >&2; exit "$resolve_rc" ;;
  esac
fi
rm -f "$resolve_err"
ROCR_INDEX=$(cut -f1 <<<"$resolved")
DEVICE_BDF=$(cut -f2 <<<"$resolved")
DEVICE_DESC=$(cut -f3 <<<"$resolved")
export ROCR_VISIBLE_DEVICES="$ROCR_INDEX"

# Kernel-log access decides whether this probe can see a fault at all.
# `dmesg` is unreadable under kernel.dmesg_restrict=1, which is the default on
# many distributions, and a naive `dmesg | grep -c` then returns 0 from an
# empty pipe — indistinguishable from "no faults". That silent zero is exactly
# how a null result gets manufactured, so the source is probed by exit status
# and a missing source reports null, never 0.
FAULT_PATTERN='VM_L2_PROTECTION_FAULT|page fault|SQC|REMOVE_QUEUE|GPU reset'
if dmesg -T >/dev/null 2>&1; then
    LOG_SOURCE=dmesg
elif journalctl -k -n 1 >/dev/null 2>&1; then
    LOG_SOURCE=journalctl
else
    LOG_SOURCE=none
fi

fault_lines() {
    case "$LOG_SOURCE" in
        dmesg) dmesg -T 2>/dev/null | grep -icE "$FAULT_PATTERN" || true ;;
        journalctl) journalctl -k --no-pager 2>/dev/null | grep -icE "$FAULT_PATTERN" || true ;;
        *) echo "" ;;
    esac
}

host=$(hostname)
started=$(date -u +%Y-%m-%dT%H:%M:%SZ)
base_commit=$(git -C "$(dirname "$0")/.." rev-parse HEAD 2>/dev/null || echo unknown)
cwsr=$(cat /sys/module/amdgpu/parameters/cwsr_enable 2>/dev/null || echo unknown)
mcbp=$(cat /sys/module/amdgpu/parameters/mcbp 2>/dev/null || echo unknown)

echo "=== ROCm/ROCm#6529 preemption probe ==="
echo "device:       ${DEVICE_DESC}"
echo "pinned:       ROCR_VISIBLE_DEVICES=${ROCR_INDEX} (${DEVICE_BDF})"
echo "cwsr_enable:  ${cwsr}    mcbp: ${mcbp}"
echo "shape:        ${TOKENS} tokens = $((TOKENS * 2)) dispatches per retained IB"
echo "contenders:   ${CONTENDERS}    reps/arm: ${REPS}    arms: ${FULL_STATE_ARMS}"
echo "kernel log:   ${LOG_SOURCE}"
if [[ "$LOG_SOURCE" == none ]]; then
    echo
    echo "WARNING: no readable kernel log (dmesg is restricted and journalctl -k" >&2
    echo "is unavailable). This probe cannot observe a VM fault. Functional" >&2
    echo "failures are still detected, but fault counts will be reported as null," >&2
    echo "NOT as zero. Re-run with kernel log access for a usable negative result." >&2
fi
echo

# Neighbour safety. On a shared multi-GPU host another workload may be resident
# on a device this probe must not disturb. Snapshot every other GPU's VRAM
# before and after so the artifact can show the probe stayed on its pin.
neighbour_vram() {
    rocm-smi --showmeminfo vram 2>/dev/null |
        grep -oE 'GPU\[[0-9]+\][^:]*: VRAM Total Used Memory \(B\): [0-9]+' |
        grep -oE '[0-9]+$' | tr '\n' ' ' || true
}
NEIGHBOUR_VRAM_BEFORE=$(neighbour_vram)

arm_json=""
overall_faults=0

for mode in $FULL_STATE_ARMS; do
    pids=()
    if [[ "$CONTENDERS" -gt 0 ]]; then
        for _ in $(seq 1 "$CONTENDERS"); do
            # Pin every contender explicitly rather than trusting inheritance.
            # These are the processes most likely to be copy-pasted elsewhere,
            # and an unpinned GPU hog can land on a neighbouring device that
            # somebody else's workload is holding.
            env ROCR_VISIBLE_DEVICES="$ROCR_INDEX" HIP_VISIBLE_DEVICES=0 \
                "$CONTENTION_BIN" "$CONTENTION_SECONDS" >/dev/null 2>&1 &
            pids+=($!)
        done
        sleep 5
    fi

    base=$(fault_lines)
    pass=0; fail=0; samples=""
    for rep in $(seq 1 "$REPS"); do
        if out=$(timeout 600 env REDLINE_PM4_FULL_STATE="$mode" "$CHAIN_AB" "$TOKENS" "$CHAIN_CO" 2>&1) \
            && grep -q PASS <<<"$out"; then
            pass=$((pass + 1))
            us=$(grep -oE 'replay\): acc=[0-9]+ +[0-9.]+' <<<"$out" | grep -oE '[0-9.]+$' || echo "null")
            samples="${samples:+$samples, }${us}"
            printf '  arm full_state=%s rep %s: PASS %s us/token\n' "$mode" "$rep" "$us"
        else
            fail=$((fail + 1))
            printf '  arm full_state=%s rep %s: FAIL :: %s\n' "$mode" "$rep" "$(tail -3 <<<"$out" | tr '\n' ' ')"
        fi
    done
    for p in "${pids[@]:-}"; do kill "$p" 2>/dev/null || true; done
    wait 2>/dev/null || true

    if [[ "$LOG_SOURCE" == none ]]; then
        delta=null
        printf 'ARM full_state=%s: pass=%s fail=%s new_kernel_fault_lines=UNOBSERVABLE\n\n' \
            "$mode" "$pass" "$fail"
    else
        delta=$(( $(fault_lines) - base ))
        overall_faults=$((overall_faults + delta))
        printf 'ARM full_state=%s: pass=%s fail=%s new_kernel_fault_lines=%s\n\n' \
            "$mode" "$pass" "$fail" "$delta"
    fi

    arm_json="${arm_json:+$arm_json,}
    {
      \"full_state\": ${mode},
      \"pass\": ${pass},
      \"fail\": ${fail},
      \"new_kernel_fault_lines\": ${delta},
      \"us_per_token\": [${samples}]
    }"
done

artifact=$(cat <<JSON
{
  "schema_version": 1,
  "kind": "redline_6529_preemption_probe",
  "date": "${started}",
  "host": "${host}",
  "base_commit": "${base_commit}",
  "hypothesis": "mid-IB compute preemption (CWSR/MES) loses SH state that stateful elision depends on, making a later DISPATCH_DIRECT fetch its kernel descriptor from address 0",
  "device": {
    "selector": "${DEVICE}",
    "pci": "${DEVICE_BDF}",
    "rocr_index_unfiltered": ${ROCR_INDEX},
    "describe": "${DEVICE_DESC}"
  },
  "controls": {
    "cwsr_enable": "${cwsr}",
    "mcbp": "${mcbp}",
    "kernel_log_source": "${LOG_SOURCE}",
    "faults_observable": $([[ "$LOG_SOURCE" == none ]] && echo false || echo true)
  },
  "shape": {
    "tokens_per_ib": ${TOKENS},
    "dispatches_per_ib": $((TOKENS * 2)),
    "reps_per_arm": ${REPS},
    "contender_processes": ${CONTENDERS}
  },
  "neighbour_vram_used_bytes": {
    "before": "${NEIGHBOUR_VRAM_BEFORE}",
    "after": "$(neighbour_vram)",
    "note": "per-GPU VRAM in rocm-smi index order; devices other than the pinned one must be unchanged apart from their own workloads"
  },
  "arms": [${arm_json}
  ],
  "total_new_kernel_fault_lines": $([[ "$LOG_SOURCE" == none ]] && echo null || echo "${overall_faults}"),
  "interpretation": "A zero fault count bounds how often the tested mechanism fires under this load; it does not prove the mechanism cannot fire on rarer preemption paths (eviction via TTM/USERPTR/SVM, suspend/resume, debugger attach), none of which this probe exercises. A null fault count means no kernel log was readable and the run says nothing about faults at all."
}
JSON
)

if [[ -n "$OUT" ]]; then
    mkdir -p "$(dirname "$OUT")"
    printf '%s\n' "$artifact" >"$OUT"
    echo "artifact: $OUT"
else
    printf '%s\n' "$artifact"
fi

if [[ "$overall_faults" -gt 0 ]]; then
    echo "REPRODUCED: ${overall_faults} new kernel fault lines." >&2
    exit 1
fi
echo "No faults observed. This bounds the mechanism; it does not clear it."
