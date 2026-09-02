<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# hipfire-mqv2-bench

Microbench for hipfire's mqv2 WMMA GEMM kernels on HIP, hipGraph and Redline
retained PM4, with the same discipline as `../hipfire-6409`: every row is
gated by a CPU reference before its timing counts, provenance is printed from
inside the process, and results are JSON that `../hipfire-6409/join_arms.py`
can consume.

It exists so the production kernels can be A/B'd here without touching the
hipfire tree: scheduler profiles, wave64 ports, Microwave (ACO) bodies,
dword-window decode, nontemporal payload loads, occupancy hints.

## Imported kernels

Copied verbatim from hipfire commit `0f1628241` (2026-09-01) into `kernels/`:

| file | sha256 (16) | families |
| --- | --- | --- |
| `gemm_mqv2_wmma_gfx11_bt.hip` | `5aa3f6489d8a410e` | qkvza/qkv/gate_up/residual BT, gfx1100+gfx1151, bits 2/3/5/6 |
| `gemm_mqv2_wmma_gfx11_mw_lds.hip` | `87d51439f8a66da4` | gate_up/residual MW-LDS, gfx1100+gfx1151, bits 3/4/5/6, NW 4/8 |
| `gemm_qkv_mqv2_wmma_gfx1201_bt.hip` | `b0fea89599813195` | qkv BT8, gfx1201 only, bits 2/3/5/6 |

The wire format is hipfire's dual-half FP16 affine G256 packing
(`group_bytes = 8 + 32*bits`); the CPU reference mirrors
`crates/hipfire-runtime/examples/mqv2_family_parity.rs` (`pack_blob`,
`decode_and_gemm_batched`, rel-RMS <= 0.05).

## Layout and ownership

`src/types.rs` is the contract between the two halves:

- kernel/oracle half: `build.rs` (Radiowave compile per arch, wave32),
  `kernels.rs` (symbol table, grid rule, kernarg layout), `oracle.rs`
  (pack + f64 reference + verify), `fixture.rs` (deterministic inputs).
- runtime/driver half: `hip_backend.rs` (hip + hipgraph), `redline_backend.rs`
  (retained PM4), `spec.rs` (row matrix), `driver.rs` (CLI, loop),
  `report.rs` (JSON), `rocm_provenance.rs`.

## Build and run

```
HIPCC=/opt/rocm/core-10.0/bin/hipcc HIPFIRE_BENCH_ARCH=gfx1151 cargo build --release
LD_LIBRARY_PATH=/opt/rocm/core-10.0/lib ./target/release/hipfire-mqv2-bench \
  --backends hip,hipgraph,redline --warmups 2 --samples 5 --out results/gfx1151/<date>.json
```
