#!/usr/bin/env bash
# Compile graph_demo for gfx1201 with the ROCm core toolchain.
set -euo pipefail
cd "$(dirname "$0")"

export PATH="/opt/rocm/core/bin:${PATH:-}"
export ROCM_PATH="${ROCM_PATH:-/opt/rocm/core}"
export HIP_PATH="${HIP_PATH:-/opt/rocm/core}"
export HIP_CLANG_PATH="${HIP_CLANG_PATH:-/opt/rocm/core/lib/llvm/bin}"
export LD_LIBRARY_PATH="/opt/rocm/core/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"

ARCH="${HIP_OFFLOAD_ARCH:-gfx1201}"
hipcc --offload-arch="${ARCH}" -O2 graph_demo.cpp -o graph_demo
echo "built $(pwd)/graph_demo (arch=${ARCH})"
