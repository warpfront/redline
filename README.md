<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# Redline

**Lightning-fast kernel dispatch for ROCm.** Record a kernel DAG once, derive
only the required fences from declared memory access, and replay it as one
retained submission over the public ROCr/HSA ABI — avoiding HIP's per-launch
submission-and-fence floor.

Redline targets the well-documented HIP dispatch-floor gap
([ROCm/ROCm#6409](https://github.com/ROCm/ROCm/issues/6409)): at tiny-dispatch
workloads, Vulkan/RADV command-buffer replay is faster than `hipGraph` because
HIP fences and submits more work per launch than the data flow requires.

## What you get

Three integration surfaces, same retained-PM4 engine:

| Surface | Use when |
| --- | --- |
| **`redline-hipgraph` preload interposer** | Drop-in for existing `hipGraph*` apps via `LD_PRELOAD` (and optional Python control module). Module-loaded / static-fatbin kernels can replay through Redline; unsupported graphs fall back to real HIP. |
| **Explicit C ABI** (`redline-capi`) | Engine integration with `rl_*` record → load → build → launch. |
| **Rust + Python APIs** (`redline-dispatch`, `redline-py`) | First-class graph authoring, Radiowave-bound module load, and PM4 wait/RMW selection. |

Radiowave is the compiler **policy** layer (not a compiler fork): it drives
installed LLVM/hipcc, inspects the AMDGPU code object, certifies VMEM-only
mutable reads for the narrow RMW boundary, and emits a hashed build manifest.
Scalar or ambiguous consumers fail closed to the broader same-agent boundary.

## Architecture support

| Path | Coverage |
| --- | --- |
| Public ROCr/AQL replay | Architecture-neutral; exercised on gfx1010, gfx1030, gfx1100, gfx1151, gfx1201 |
| Retained direct PM4 | Family-specific GFX10/GFX11 and GFX12 encoders; device-family mismatch fails closed |
| Multi-queue independent IBs | Auto policy: Q1 (unswept gfx10), Q4 (gfx11/gfx1151), Q2 (gfx12); serial RMW stays single-queue |

Legacy direct PM4 supports zero-scratch HSA kernels whose implicit user data is
the optional private-segment buffer plus kernarg pointer. Unsupported scratch,
queue, dispatch, or flat-scratch contracts fail closed.

**Requires ROCm Core SDK ≥ 7.14** (TheRock layout, typically `/opt/rocm/core`).

## Current results (ROCm 7.14)

Primary headline — Hipfire-native 240-row four-backend matrix on **gfx1201**
(RX 9070 XT), correctness-gated:

| Run | Role | Redline firsts | Notes |
| --- | --- | ---: | --- |
| [`2026-07-22-rocm7.14-retest`](examples/hipfire-6409/results/gfx1201/2026-07-22-rocm7.14-retest/REPORT.md) | **Primary** | **192/240 (80.0%)** | RL > Vulkan 192/240; RL > HipGraph/HIP 222/240 each; 0 rejected rows |
| [`2026-07-22-rocm714-leverage-certification`](examples/hipfire-6409/results/gfx1201/2026-07-22-rocm714-leverage-certification/REPORT.md) | Secondary A/B | 194/240 (80.83%) | **Dirty-tree / non-regression leverage evidence — not a clean certification** |

Cross-RDNA portability (ROCm 7.2-era retained native PM4, still the published
multi-arch control):

- Aggregate [`2026-07-14-rdna-rocr-native`](examples/hipfire-6409/results/2026-07-14-rdna-rocr-native/REPORT.md): **537/960** firsts (55.94%), RL > Vulkan **606/960** (63.13%), 960/960 correct — gfx1010 / gfx1030 / gfx1100 / gfx1151.

Multi-queue controls (independent throughput + full certifications):

| Arch | Retained runs |
| --- | --- |
| gfx1100 | [Q1 independent](examples/hipfire-6409/results/gfx1100/2026-07-14-redline-current-q1-independent/REPORT.md), [Q4 full](examples/hipfire-6409/results/gfx1100/2026-07-14-redline-multiqueue-q4/REPORT.md) (189/240) |
| gfx1151 | [Q1 independent](examples/hipfire-6409/results/gfx1151/2026-07-14-redline-current-q1-independent/REPORT.md), [Q4 full](examples/hipfire-6409/results/gfx1151/2026-07-14-redline-multiqueue-q4/REPORT.md) (206/240) |
| gfx1201 | [Q1 independent](examples/hipfire-6409/results/gfx1201/2026-07-14-redline-current-q1-independent/REPORT.md), [Q2 independent](examples/hipfire-6409/results/gfx1201/2026-07-14-redline-multiqueue-q2-independent/REPORT.md), [Q2 full](examples/hipfire-6409/results/gfx1201/2026-07-14-redline-multiqueue-q2/REPORT.md) (187/240), [Q4 negative control](examples/hipfire-6409/results/gfx1201/2026-07-14-redline-multiqueue-q4/REPORT.md) |

HipEngine pristine harness (core 224 rows, three backends) on ROCm 7.14:

| Arch | RL > Vulkan | Report |
| --- | ---: | --- |
| gfx1201 | **197/224 (87.9%)** | [REPORT](examples/hipengine-6409/results/gfx1201/2026-07-22-714-bench/REPORT.md) |
| gfx1151 | **164/224 (73.2%)** | [REPORT](examples/hipengine-6409/results/gfx1151/2026-07-22-714-bench/REPORT.md) |
| gfx1100 | **127/224 (56.7%)** | [REPORT](examples/hipengine-6409/results/gfx1100/2026-07-22-714-bench/REPORT.md) |

Raw JSON records keep their original dirty-tree flags, empty/`hipcc` capture
fields, and absolute runner paths where the harness wrote them. Product docs
label that provenance; they do not rewrite the artifacts.

Harness details and reproduce commands:
[`examples/hipfire-6409`](examples/hipfire-6409/README.md),
[`examples/hipengine-6409`](examples/hipengine-6409/README.md).
Dispatch-floor methodology (historical ROCm 7.2 microbench):
[`docs/DISPATCH-FLOOR.md`](docs/DISPATCH-FLOOR.md).

## Quick start

```bash
# Toolchain (TheRock / ROCm 7.14+)
export PATH=/opt/rocm/core/bin:/opt/rocm/core/lib/llvm/bin:$PATH
export ROCM_PATH=/opt/rocm/core HIP_PATH=/opt/rocm/core

cargo build --release -p redline-dispatch -p redline-capi -p redline-hipgraph

# Explicit hipGraph-shaped Rust API
cargo run -p redline-dispatch --example hipgraph_migration

# Optional: preload interposer into a HIP graph process
LD_PRELOAD="$PWD/target/release/libredline_hipgraph.so" your_hip_app
```

C header: [`crates/redline-capi/include/redline_dispatch.h`](crates/redline-capi/include/redline_dispatch.h).
Python package notes: [`crates/redline-py`](crates/redline-py/README.md).
Preload crate: [`crates/redline-hipgraph`](crates/redline-hipgraph/README.md).

## Limitations (honest)

- **Not a full HIP runtime replacement.** The preload path covers supported
  `hipGraph*` / module-loaded kernels; everything else falls through to real HIP.
- **Retained PM4 is family-specific.** Wrong encoder for the device is rejected.
- **Residual Vulkan wins are mostly codegen.** Packed-dot / VOPD-class rows and
  some production shapes remain RADV/ACO advantages on the shared-HSACO control;
  transport is not the whole story.
- **Published ROCm 7.14 Hipfire runs carry dirty-tree provenance** in the raw
  records (`repository_dirty` / `hipfire_clone_dirty`). The primary retest also
  recorded an empty `hipcc` version string. Treat numbers as correctness-gated
  measurements with that capture context, not as a laundered clean CI stamp.
- **Leverage A/B is secondary.** The 194/240 partitioned run is non-regression
  evidence under dirty tree; roctx/amd-smi observation was not uniformly
  available across arms.
- **Historical dispatch-floor µs tables** in `docs/DISPATCH-FLOOR.md` are ROCm
  7.2 methodology on R9700; current product validation is the retained ROCm
  7.14 set above.

## Crate map

| Crate | Role |
| --- | --- |
| **`radiowave`** | HIP compiler policy: lowering helpers, wave/target selection, hipcc invocation, CO inspection, manifests |
| **`redline-dispatch`** | Record/replay core, hazard checks, minimal fences, backends, Rust `hipgraph` adapter |
| **`redline-rocr`** | Public ROCr/HSA ABI, AQL, queue publication ([provenance](crates/redline-rocr/PROVENANCE.md)) |
| **`redline-capi`** | C ABI for engines |
| **`redline-py`** | Python bindings |
| **`redline-hipgraph`** | `hipGraph*` ABI + `LD_PRELOAD` interposer |
| **`redline-observe`** | Optional ROCm 7.14 observability hooks (roctx / amd-smi when present) |

## License

Licensed under the **Apache License, Version 2.0** — see [`LICENSE`](LICENSE).
Redistributions must retain [`NOTICE`](NOTICE) per section 4(d).

"Redline" is a trademark of Kaden Schutt. As stated in section 6 of the License,
the Apache-2.0 grant does not include trademark rights: you may use, modify, and
redistribute this code, but not the "Redline" name to identify your own fork or
product.

`/opt/rocm/core` is the expected TheRock layout on the machines that produced
the retained results; adjust `PATH` / `ROCM_PATH` if your install differs.
