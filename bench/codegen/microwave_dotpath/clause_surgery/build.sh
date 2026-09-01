#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
ROCM="${ROCM:-/opt/rocm/core-10.0}"
CLANG="${CLANG:-$ROCM/lib/llvm/bin/clang}"
LLD="${LLD:-$ROCM/lib/llvm/bin/ld.lld}"
HIPCC="${HIPCC:-$ROCM/bin/hipcc}"
OBJDUMP="${OBJDUMP:-$ROCM/lib/llvm/bin/llvm-objdump}"
export PATH="$ROCM/bin:$ROCM/lib/llvm/bin:$PATH"
export LD_LIBRARY_PATH="$ROCM/lib:${LD_LIBRARY_PATH:-}"

cd "$ROOT"

if [[ "${1:-}" == "regen" ]]; then
  tmp=$(mktemp -d)
  trap 'rm -rf "$tmp"' EXIT
  (cd "$tmp" && "$HIPCC" --offload-arch=gfx1151 -O3 -std=c++17 --save-temps \
    -c "$ROOT/../microwave_dotpath_probe.hip" -o probe.o)
  full=$(ls "$tmp"/microwave_dotpath_probe-hip-amdgcn-amd-amdhsa-gfx1151.s)
  python3 "$ROOT/reorder.py" --full "$full" --out-dir "$ROOT"
else
  if [[ ! -f A_control.s ]]; then
    echo "A_control.s missing; run: $0 regen" >&2
    exit 1
  fi
fi

assemble() {
  local src=$1 arch=$2 out=$3
  local tmp
  tmp=$(mktemp)
  sed "s/amdgcn-amd-amdhsa--gfx1151/amdgcn-amd-amdhsa--${arch}/g" "$src" >"$tmp"
  "$CLANG" -x assembler -target amdgcn-amd-amdhsa -mcpu="$arch" \
    -mcode-object-version=6 -c "$tmp" -o "${out}.o"
  "$LLD" -shared "${out}.o" -o "$out"
  rm -f "$tmp" "${out}.o"
}

for arm in A_control B_group_clause C_group_noclause D_clause_only; do
  assemble "${arm}.s" gfx1151 "${arm}.gfx1151.hsaco"
  assemble "${arm}.s" gfx1100 "${arm}.gfx1100.hsaco"
done

echo "--- llvm-objdump A_control gfx1151 inner loop (loads / clauses) ---"
"$OBJDUMP" -d A_control.gfx1151.hsaco | awk '
  /global_load_b32/ { loads++ }
  /s_clause/ { clauses++ }
  END { printf "loads=%d s_clause=%d\n", loads+0, clauses+0 }
'
"$HIPCC" -O3 -std=c++17 run_hsaco.hip -o run_hsaco -ldl
echo "built run_hsaco and hsaco objects in $ROOT"
