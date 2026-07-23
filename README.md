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

Four integration paths, one retained-PM4 engine:

| Path | Use when |
| --- | --- |
| **Explicit C/C++ ABI** (`redline-capi`) | Your engine owns its HIP allocations, code objects, and fixed launch sequence. |
| **Python** (`redline-py`) | You want graph authoring or retained module load/build/replay from Python. |
| **`hipGraph*` preload** (`redline-hipgraph`) | An existing application already uses compatible HIP graph capture; unsupported operations fall through to HIP. |
| **Rust** (`redline-dispatch`) | You want the native graph, public-AQL, HIP multistream, or direct retained-PM4 APIs. |

Start with the choose-your-path **[Using Redline](docs/INTEGRATION.md)** guide.

Radiowave is the compiler **policy** layer (not a compiler fork): it drives
installed LLVM/hipcc, inspects the AMDGPU code object, certifies VMEM-only
mutable reads for the narrow RMW boundary, and emits a hashed build manifest.
Scalar or ambiguous consumers fail closed to the broader same-agent boundary.

## Architecture support

| Path | Coverage |
| --- | --- |
| Public ROCr/AQL replay | Architecture-neutral; exercised on gfx1010, gfx1030, gfx1100, gfx1151, gfx1201 |
| Retained direct PM4 | Family-specific GFX10/GFX11 and GFX12 encoders; device-family mismatch fails closed |
| Multi-queue independent IBs | Auto policy: Q2 (gfx1100 and gfx12), Q4 (other measured gfx11), Q1 (unswept gfx10); serial RMW stays single-queue |

Legacy direct PM4 supports zero-scratch HSA kernels whose implicit user data is
the optional private-segment buffer plus kernarg pointer. Unsupported scratch,
queue, dispatch, or flat-scratch contracts fail closed.

**Requires ROCm Core SDK ≥ 7.14** (TheRock layout, typically `/opt/rocm/core`).

## Current results (ROCm 7.14)

Current headline — clean, same-commit three-card comparison at
`bb612d14a95c92c4bf20f492be28f7ae11cb51f7`: the full 240-row HipEngine-shaped
matrix, Redline/HIP/Vulkan, 3 warmups, 7 samples, and CPU-oracle correctness
gating. **All 720 rows matched; zero were rejected.**

| GPU | Arch | Auto queues | Redline first | RL > HIP | Median speedup vs HIP | RL > Vulkan | Median speedup vs Vulkan | Report |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| AMD Radeon RX 9070 XT | gfx1201 | 2 | **187/240 (77.92%)** | 222/240 | **3.52×** | 187/240 | **1.32×** | [`REPORT`](examples/hipfire-6409/results/gfx1201/2026-07-23-rocm714-three-way/REPORT.md) |
| AMD Radeon RX 7900 XTX | gfx1100 | 2 | **182/240 (75.83%)** | 227/240 | **2.34×** | 192/240 | **1.43×** | [`REPORT`](examples/hipfire-6409/results/gfx1100/2026-07-23-rocm714-three-way/REPORT.md) |
| Radeon 8060S Graphics | gfx1151 | 4 | **185/240 (77.08%)** | 233/240 | **2.68×** | 185/240 | **1.30×** | [`REPORT`](examples/hipfire-6409/results/gfx1151/2026-07-23-rocm714-three-way/REPORT.md) |

Median speedup is backend time divided by Redline time. On gfx1100, `auto` is
capped at Q2: clean explicit-Q3 and Q4 controls both reproduced retained-PM4
timeouts in adjacent memory-waitcnt rows. Those queue counts remain diagnostic
overrides, not supported defaults.

Earlier controls remain available for comparison:

- Four-backend gfx1201 retest:
  [`2026-07-22-rocm7.14-retest`](examples/hipfire-6409/results/gfx1201/2026-07-22-rocm7.14-retest/REPORT.md) —
  192/240 Redline firsts; Redline beat Vulkan and HipGraph/HIP in 192 and
  222 rows respectively.
- Cross-RDNA ROCm 7.2-era native PM4:
  [`2026-07-14-rdna-rocr-native`](examples/hipfire-6409/results/2026-07-14-rdna-rocr-native/REPORT.md) —
  537/960 firsts, Redline > Vulkan 606/960, and 960/960 correct across gfx1010,
  gfx1030, gfx1100, and gfx1151.
- HipEngine pristine 224-row harness:
  [gfx1201](examples/hipengine-6409/results/gfx1201/2026-07-22-714-bench/REPORT.md)
  197/224, [gfx1151](examples/hipengine-6409/results/gfx1151/2026-07-22-714-bench/REPORT.md)
  164/224, and [gfx1100](examples/hipengine-6409/results/gfx1100/2026-07-22-714-bench/REPORT.md)
  127/224 Redline wins over Vulkan.

Raw JSON records keep their original dirty-tree flags, empty/`hipcc` capture
fields, and absolute runner paths where the harness wrote them. Product docs
label that provenance; they do not rewrite the artifacts.

Harness details and reproduce commands:
[`examples/hipfire-6409`](examples/hipfire-6409/README.md),
[`examples/hipengine-6409`](examples/hipengine-6409/README.md).
Dispatch-floor methodology (historical ROCm 7.2 microbench):
[`docs/DISPATCH-FLOOR.md`](docs/DISPATCH-FLOOR.md).

## Start here

The canonical **[Using Redline](docs/INTEGRATION.md)** guide covers prerequisites,
the shared kernel/kernarg contract, verified C, Python, preload, and Rust
walkthroughs, failure behavior, and current package availability.

Common source-build setup:

```bash
export PATH=/opt/rocm/core/bin:/opt/rocm/core/lib/llvm/bin:$PATH
export ROCM_PATH=/opt/rocm/core HIP_PATH=/opt/rocm/core

cargo build --release -p redline-dispatch -p redline-capi -p redline-hipgraph
```

Redline is currently distributed from source. The guide distinguishes commands
that work now from the intended PyPI wheel and versioned C SDK release.

C header: [`crates/redline-capi/include/redline_dispatch.h`](crates/redline-capi/include/redline_dispatch.h).
Python reference: [`crates/redline-py`](crates/redline-py/README.md).
Preload reference: [`crates/redline-hipgraph`](crates/redline-hipgraph/README.md).

## Limitations (honest)

- **Not a full HIP runtime replacement.** The preload path covers supported
  `hipGraph*` / module-loaded kernels; everything else falls through to real HIP.
- **Retained PM4 is family-specific.** Wrong encoder for the device is rejected.
- **Residual Vulkan wins are mostly codegen.** Packed-dot / VOPD-class rows and
  some production shapes remain RADV/ACO advantages on the shared-HSACO control;
  transport is not the whole story.
- **The current three-card comparison is clean and same-commit.** Earlier
  2026-07-22 controls retain their original dirty-tree flags, empty `hipcc`
  capture fields, and absolute runner paths. Those artifacts remain useful
  controls but are not the current certification.
- **Leverage A/B is secondary.** The earlier 194/240 partitioned run is
  dirty-tree non-regression evidence; roctx/amd-smi observation was not
  uniformly available across arms.
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
