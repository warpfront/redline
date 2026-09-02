// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
//
// hipfire mqv2 WMMA GEMM microbench on HIP, hipGraph and Redline retained PM4.
// Module ownership (see README.md):
//   kernel/oracle half : build.rs, kernels.rs, oracle.rs, fixture.rs
//   runtime/driver half: rocm_provenance.rs, hip_backend.rs, redline_backend.rs,
//                        spec.rs, report.rs, driver.rs
// Both halves build only against types.rs.

mod driver;
mod fixture;
mod hip_backend;
mod kernels;
mod oracle;
mod redline_backend;
mod report;
mod rocm_provenance;
mod spec;
mod types;

fn main() -> anyhow::Result<()> {
    driver::run()
}
