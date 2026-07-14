<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# Redline

**A leaner, safer HipGraph.** Record a dispatch DAG once, derive the *minimal*
correct fences from declared memory access, and replay it as a single retained
submission on the public ROCr/HSA ABI — instead of paying HIP's per-dispatch
submission-and-fence floor on every launch.

Redline is a drop-in-shaped replacement for `hipGraph` on ROCm/RDNA: the same
capture → instantiate → launch model, plus an explicit tuning surface and
hazard-checked safety.

## Why

On RDNA, Vulkan/RADV command-buffer replay is measurably faster than HIP graph
replay at the tiny-dispatch floor — the well-documented HIP dispatch-floor gap
(see e.g. [ROCm/ROCm#6409](https://github.com/ROCm/ROCm/issues/6409)). It is not
one problem: HIP pays a per-dispatch submission/dependency floor, while some
kernels also expose ACO-versus-LLVM lowering differences. A retained command
buffer removes the first cost by submitting once with only the fences the data
flow requires.

Redline closes that gap **on the pure ROCm/HIP stack**:

- **One retained submission, not N fenced dispatches.** A captured graph lowers
  to a single retained submission published through the public ROCr/HSA queue
  ABI (`redline-rocr`).
- **Minimal fences, derived — not blanket.** Each node declares what it reads
  and writes; Redline derives the smallest acquire/release scope the DAG
  actually needs (`FenceScope::None` where there is no hazard), rather than a
  system-scope fence on every launch.
- **Hazard-checked, fail-closed.** Memory ordering is validated at instantiate
  time; an unsafe graph is rejected, not silently mis-fenced.

## Results

Measured against the equivalent HIP path on the originating integration, GPU
timestamps:

| Workload                        | Local RDNA | R9700 (gfx1201) |
|---------------------------------|-----------:|----------------:|
| Real single-token dispatch DAG  |    1.076×  |  1.059–1.060×   |
| Expanded independent launch set |    1.378×  |  1.265–1.292×   |

The standalone Hipfire-native ROCm issue 6409 harness now covers 45 shapes in
serial, independent, and aggressive single-kernel modes. The current
Radiowave-certified HIP policy and minimal same-agent RMW boundary take
**121/133 four-way first places (90.98%)**, beat Vulkan strictly in **121/133
(90.98%)**, and take **39/45 safe serial** plus **40/43 aggressive** rows. The
941-launch correct RMW chain runs at **0.7549 us/dispatch**, versus Vulkan at
**1.0400 us**. All 532 row-runs in the counterbalanced four-pass gfx1201 A/B
passed all four backend CPU oracles. See the
[certified-boundary report](examples/hipfire-6409/results/gfx1201/2026-07-13-radiowave-vmem-final/comparison/REPORT.md)
and [aggregate artifact](examples/hipfire-6409/results/gfx1201/2026-07-13-radiowave-vmem-final/comparison/aggregate.json).

The result also transfers back to the pinned HipEngine #6409 suite without
replacing its kernels: Radiowave + Redline beats Vulkan in **192/212 matched
kernel rows (90.57%)**, up from 186/212 before the full compiler/submission
contract was integrated. All 224 Redline rows pass their CPU oracles. See the
[HipEngine integration report](examples/hipengine-6409/results/gfx1201/2026-07-13-radiowave-redline/REPORT.md)
and [matched summary](examples/hipengine-6409/results/gfx1201/2026-07-13-radiowave-redline/summary.json).

For the historical wave control, a counterbalanced
all32/targeted64/blanket64 comparison passed all 133 four-way CPU oracles in
every replicate. Targeted wave64 took 94/133 aggregate first places and its
unfenced single-kernel mode took 36/43. See the
[`hipfire-6409` wave-policy report](examples/hipfire-6409/results/gfx1201/2026-07-12-wave-policy-comparison/REPORT.md)
and [aggregate artifact](examples/hipfire-6409/results/gfx1201/2026-07-12-wave-policy-comparison/aggregate.json).

The first low-margin tuning pass with ordinary hipcc buffer
operations raises that policy to **98/133** aggregate firsts: packed Q8/Q4
independent throughput and the aggressive gather row cross Vulkan, with all
133 correctness oracles still passing. See the
[loser-tuning report](examples/hipfire-6409/results/gfx1201/2026-07-12-loser-tuning/REPORT.md)
and [tuned aggregate](examples/hipfire-6409/results/gfx1201/2026-07-12-loser-tuning/full-buffer/aggregate.json).

The accepted lowering rules live in **Radiowave**, Redline's compiler
policy crate. Callers compile HIP through its Rust API or CLI; Radiowave
force-includes reviewed gfx11/gfx12 buffer-resource helpers, selects tested
wave/workgroup/source variants, invokes the installed LLVM/hipcc backend,
inspects the AMDGPU code object for instruction-class and register/spill
regressions, certifies whether mutable reads are VMEM-only for Redline's
minimal cache boundary, and emits a hashed JSON build manifest. Scalar or
ambiguous consumers fail closed to the broader same-agent shader-cache
invalidation. It is a policy layer
around upstream ROCm, not a compiler fork or Vulkan path. See
[`crates/radiowave`](crates/radiowave/README.md).

The same contract is exposed outside Rust. The C ABI function
`rl_gpu_load_module_radiowave` and Python `Gpu.load_module(code, manifest)`
bind the exact code-object bytes to the manifest before loading them. Their
module queries expose the certified scheduler profile, wavefront width, and
per-kernel mutable-read cache class. `rl_pm4_wait_rmw` in C and `Gpu.build()`
in Python then select the VMEM-only boundary from the *next consumer*; raw
modules, stale manifests, missing symbols, and scalar/unknown consumers fail
closed to the generic same-agent boundary. See the
[`redline-capi` header](crates/redline-capi/include/redline_dispatch.h) and
[`redline-py` example](crates/redline-py/README.md).

## Migrating from HipGraph (or a Vulkan command buffer)

The [`hipgraph`](crates/redline-dispatch/src/hipgraph.rs) adapter presents the
familiar shape:

| HipGraph / Vulkan                         | Redline                              |
|-------------------------------------------|--------------------------------------|
| `hipGraphCreate`                          | `Graph::new` / `Graph::with_tuning`  |
| buffer a node reads/writes                | `Graph::buffer` + `Graph::region`    |
| `hipGraphAddKernelNode`                   | `Graph::kernel` / `kernel_after`     |
| node dependency array                     | `deps` arg to `kernel_after`         |
| `hipGraphInstantiate` → `hipGraphExec_t`  | `Graph::instantiate` → `GraphExec`   |
| `hipGraphLaunch(exec, stream)`            | `GraphExec::launch`                  |

```rust
use redline_dispatch::hipgraph::{Graph, Tuning};
use redline_dispatch::{Access, Dim3, KernelLaunch, ReplayToken};

let mut graph = Graph::with_tuning(Tuning::latency());
let acts = graph.buffer("activations", 4096)?;
let input = graph.region(acts, 0, 2048)?;
let output = graph.region(acts, 2048, 2048)?;

let project = graph.kernel(
    KernelLaunch::new("project", Dim3::x(32)?, Dim3::x(64)?)?,
    [Access::read(input), Access::write(output)],
)?;
graph.kernel_after(
    KernelLaunch::new("consume", Dim3::x(32)?, Dim3::x(64)?)?,
    [Access::read(output)],
    [project],
)?;

let exec = graph.instantiate()?;      // == hipGraphInstantiate
exec.launch(&mut backend, ReplayToken(0))?; // == hipGraphLaunch
```

Run it: `cargo run -p redline-dispatch --example hipgraph_migration`.

**The one thing HipGraph infers that Redline asks you to state** is each node's
buffer reads/writes (`Access::read` / `Access::write`). That single declaration
is what buys both the safety and the lower fence floor — and if you author
HipGraph nodes you already know your kernel's I/O.

### Tuning

`Tuning` controls how a graph replays. The default matches a single serial
stream; raise it to overlap independent branches.

- `Tuning::latency()` — one lane, each launch fully completes (default; safest).
- `Tuning::overlap(lanes)` — independent lanes run concurrently.
- `Tuning::throughput(lanes, max_in_flight)` — whole-graph overlap with a
  bounded number of launches in flight.

## Crates

- **`radiowave`** — the HIP compiler policy boundary: reviewed lowering
  helpers, wave/target configuration, LLVM/hipcc invocation, code-object
  inspection, and reproducible build manifests.
- **`redline-dispatch`** — backend-neutral record/replay core: the dispatch DAG,
  hazard checking, minimal-fence derivation, the `hipgraph` adapter, and HIP /
  public-AQL replay backends.
- **`redline-rocr`** — the auditable public ROCr/HSA ABI, resource ownership,
  AQL packet encoding, and release-ordered queue publication. It is an
  independent implementation against the public headers; no ROCm implementation
  source is vendored (see [`crates/redline-rocr/PROVENANCE.md`](crates/redline-rocr/PROVENANCE.md)).

## License and attribution

Licensed under the **Apache License, Version 2.0** — see [`LICENSE`](LICENSE).
Redistributions must retain the [`NOTICE`](NOTICE) file per section 4(d).

"Redline" is a trademark of Kaden Schutt. As stated in section 6 of the License,
the Apache-2.0 grant does not include trademark rights: you may use, modify, and
redistribute this code, but not the "Redline" name to identify your own fork or
product.
