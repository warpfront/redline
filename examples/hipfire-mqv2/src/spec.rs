// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

use crate::kernels;
use crate::types::{Arch, RowSpec, Shape, TimingMode};
use radiowave::SchedulerProfile;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShapeSet {
    Smoke,
    Prefill,
    All,
}

impl ShapeSet {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "smoke" => Some(Self::Smoke),
            "prefill" => Some(Self::Prefill),
            "all" => Some(Self::All),
            _ => None,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Smoke => "smoke",
            Self::Prefill => "prefill",
            Self::All => "all",
        }
    }
}

/// Smoke shapes: n16 k512 with 64 rows per projection (fast parity-like).
fn smoke_shapes() -> Vec<Shape> {
    // One shape per family: projections = 4,3,2,1 each with 64 rows.
    vec![
        Shape { n_tokens: 16, k: 512, proj_m: vec![64, 64, 64, 64] }, // qkvza
        Shape { n_tokens: 16, k: 512, proj_m: vec![64, 64, 64] },     // qkv
        Shape { n_tokens: 16, k: 512, proj_m: vec![64, 64] },         // gate_up
        Shape { n_tokens: 16, k: 512, proj_m: vec![64] },            // residual
    ]
}

fn prefill_shapes() -> Vec<Shape> {
    vec![
        // n128 k2048
        Shape { n_tokens: 128, k: 2048, proj_m: vec![2048, 512, 512] }, // qkv
        Shape { n_tokens: 128, k: 2048, proj_m: vec![2048, 512, 512, 256] }, // qkvza
        Shape { n_tokens: 128, k: 2048, proj_m: vec![6144, 6144] },     // gate_up
        Shape { n_tokens: 128, k: 2048, proj_m: vec![2048] },           // residual
        // n512 k4096
        Shape { n_tokens: 512, k: 4096, proj_m: vec![4096, 1024, 1024] }, // qkv
        Shape { n_tokens: 512, k: 4096, proj_m: vec![4096, 1024, 1024, 512] }, // qkvza
        Shape { n_tokens: 512, k: 4096, proj_m: vec![12288, 12288] },   // gate_up
        Shape { n_tokens: 512, k: 4096, proj_m: vec![4096] },           // residual
    ]
}

pub fn shapes_for_set(set: ShapeSet) -> Vec<Shape> {
    match set {
        ShapeSet::Smoke => smoke_shapes(),
        ShapeSet::Prefill => prefill_shapes(),
        ShapeSet::All => {
            let mut v = smoke_shapes();
            v.extend(prefill_shapes());
            v
        }
    }
}

/// Build the row matrix: descriptors(arch) x shapes x modes x scheduler profiles.
/// Default profile only unless `all_profiles` is true.
/// Skips (kernel, shape) pairs where proj_m.len() != family.projections().
pub fn matrix(
    arch: Arch,
    shape_set: ShapeSet,
    modes: &[TimingMode],
    scheduler_profiles: &[SchedulerProfile],
    iterations: usize,
) -> Vec<RowSpec> {
    let descriptors = kernels::descriptors(arch);
    let shapes = shapes_for_set(shape_set);
    let mut rows = Vec::new();
    for desc in &descriptors {
        for shape in &shapes {
            if shape.proj_m.len() != desc.family.projections() {
                continue;
            }
            // Ensure k is multiple of GROUP_SIZE (all shapes are, but guard)
            if shape.k % crate::types::GROUP_SIZE != 0 {
                continue;
            }
            for &mode in modes {
                for &profile in scheduler_profiles {
                    rows.push(RowSpec {
                        kernel: desc.clone(),
                        shape: shape.clone(),
                        mode,
                        iterations,
                        scheduler_profile: profile,
                        wave_size: 32,
                    });
                }
            }
        }
    }
    rows
}
