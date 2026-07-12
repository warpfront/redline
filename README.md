<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- Copyright 2026 Kaden Schutt -->

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
(see e.g. [ROCm/ROCm#6409](https://github.com/ROCm/ROCm/issues/6409)). The cause
is not the kernels; it is that HIP re-submits and applies a system-scope
acquire/release fence *per dispatch*, while a retained command buffer submits
once with only the fences the data flow requires.

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

The independent-set GPU lower bounds remain above the historical RADV **1.14×**
result on the same launch set. A standalone reproduction harness is being lifted
out alongside these crates; until then the ratios above are reported as
certified in the originating integration, not yet as a one-command repro here.

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
