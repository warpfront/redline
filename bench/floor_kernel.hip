// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Kaden Schutt
//
// Minimal kernel for the dispatch-floor microbench. The bench measures
// per-dispatch + fence overhead, not compute, so the body is intentionally
// trivial. Build to a code object with:
//   hipcc --genco --offload-arch=gfx1201 floor_kernel.hip -o floor_kernel.co
// The dispatched symbol is then "floor_k.kd".

extern "C" __global__ void floor_k() {
    // no-op: one workitem, one workgroup; the cost under test is the dispatch
    // and its cache fence, which the packet header carries regardless of body.
}
