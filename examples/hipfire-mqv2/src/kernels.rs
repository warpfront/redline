// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

use crate::types::{Arch, ArgKind, ArgSlot, Family, KernelDesc, KernargLayout, PtrBinding, Shape, Variant};
use radiowave::SchedulerProfile;

/// All descriptors for the given arch.
pub fn descriptors(arch: Arch) -> Vec<KernelDesc> {
    let mut out = Vec::new();
    match arch {
        Arch::Gfx1100 | Arch::Gfx1151 => {
            // --- BT family (31 symbols) ---
            // 4x QKVZA BT4 (bits 2,3,5,6)
            for &bits in &[2u32, 3, 5, 6] {
                out.push(desc_bt("qkvza", bits, 4, Family::Qkvza, arch));
            }
            // 4x QKV BT4
            for &bits in &[2u32, 3, 5, 6] {
                out.push(desc_bt("qkv", bits, 4, Family::Qkv, arch));
            }
            // 4x gate_up BT12
            for &bits in &[2u32, 3, 5, 6] {
                out.push(desc_bt("gate_up", bits, 12, Family::GateUp, arch));
            }
            // 4x residual BT4
            for &bits in &[2u32, 3, 5, 6] {
                out.push(desc_bt("residual", bits, 4, Family::Residual, arch));
            }
            // gfx1100 policy extras for bits 3,5,6 only (15 symbols):
            // QKVZA BT12 x3
            for &bits in &[3u32, 5, 6] {
                out.push(desc_bt("qkvza", bits, 12, Family::Qkvza, arch));
            }
            // QKV BT12 x3
            for &bits in &[3u32, 5, 6] {
                out.push(desc_bt("qkv", bits, 12, Family::Qkv, arch));
            }
            // gate_up BT6 x3
            for &bits in &[3u32, 5, 6] {
                out.push(desc_bt("gate_up", bits, 6, Family::GateUp, arch));
            }
            // residual BT6 x3
            for &bits in &[3u32, 5, 6] {
                out.push(desc_bt("residual", bits, 6, Family::Residual, arch));
            }
            // residual BT8 x3
            for &bits in &[3u32, 5, 6] {
                out.push(desc_bt("residual", bits, 8, Family::Residual, arch));
            }

            // --- MW-LDS family (16 symbols): gate_up + residual x bits 3,4,5,6 x NW 4,8 ---
            for &bits in &[3u32, 4, 5, 6] {
                for &nw in &[4u32, 8] {
                    out.push(desc_mw("gate_up", bits, nw, Family::GateUp, arch));
                    out.push(desc_mw("residual", bits, nw, Family::Residual, arch));
                }
            }
        }
        Arch::Gfx1201 => {
            // gfx1201 BT8: gemm_qkv_mq{2,3,5,6}g256v2_wmma_gfx1201_bt8
            for &bits in &[2u32, 3, 5, 6] {
                out.push(KernelDesc {
                    symbol: format!("gemm_qkv_mq{}g256v2_wmma_gfx1201_bt8", bits),
                    family: Family::Qkv,
                    bits,
                    variant: Variant::Gfx1201Bt8,
                    archs: vec![Arch::Gfx1201],
                    source: "kernels/gemm_qkv_mqv2_wmma_gfx1201_bt.hip".to_owned(),
                });
            }
        }
    }
    out
}

fn desc_bt(kind: &str, bits: u32, bv: u32, family: Family, arch: Arch) -> KernelDesc {
    let (symbol, source) = match (family, kind) {
        (Family::Qkvza, _) => (
            format!("gemm_qkvza_mq{}g256v2_wmma_gfx11_bt{}", bits, bv),
            "kernels/gemm_mqv2_wmma_gfx11_bt.hip".to_owned(),
        ),
        (Family::Qkv, _) => (
            format!("gemm_qkv_mq{}g256v2_wmma_gfx11_bt{}", bits, bv),
            "kernels/gemm_mqv2_wmma_gfx11_bt.hip".to_owned(),
        ),
        (Family::GateUp, _) => (
            format!("gemm_gate_up_mq{}g256v2_wmma_gfx11_bt{}", bits, bv),
            "kernels/gemm_mqv2_wmma_gfx11_bt.hip".to_owned(),
        ),
        (Family::Residual, _) => (
            format!("gemm_mq{}g256v2_residual_wmma_gfx11_bt{}", bits, bv),
            "kernels/gemm_mqv2_wmma_gfx11_bt.hip".to_owned(),
        ),
    };
    KernelDesc {
        symbol,
        family,
        bits,
        variant: Variant::Bt { bv },
        archs: vec![Arch::Gfx1100, Arch::Gfx1151],
        source,
    }
}

fn desc_mw(_kind: &str, bits: u32, nw: u32, family: Family, arch: Arch) -> KernelDesc {
    let (symbol, source) = match family {
        Family::GateUp => (
            format!("gemm_gate_up_mq{}g256v2_wmma_gfx11_mw{}_lds", bits, nw),
            "kernels/gemm_mqv2_wmma_gfx11_mw_lds.hip".to_owned(),
        ),
        Family::Residual => (
            format!("gemm_mq{}g256v2_residual_wmma_gfx11_mw{}_lds", bits, nw),
            "kernels/gemm_mqv2_wmma_gfx11_mw_lds.hip".to_owned(),
        ),
        _ => unreachable!(),
    };
    KernelDesc {
        symbol,
        family,
        bits,
        variant: Variant::MwLds { nw },
        archs: vec![Arch::Gfx1100, Arch::Gfx1151],
        source,
    }
}

/// Grid per the kernel's blockIdx usage.
///
/// BT (gfx11):  gemm_mqv2_wmma_gfx11_bt.hip lines 72-73
///   const int row_start = blockIdx.x * 16;
///   const int batch_start = blockIdx.y * (16 * BV);
///   Grid: [ceil(total_m/16), ceil(N/(16*BV)), 1]
///
/// MW (gfx11):  gemm_mqv2_wmma_gfx11_mw_lds.hip lines 93-94 (gate) / 185-186 (resid)
///   const int row_start = blockIdx.x * 16;
///   const int n_block_start = blockIdx.y * (16 * NW);
///   Grid: [ceil(total_m/16), ceil(N/(16*NW)), 1]
///
/// gfx1201 BT8: gemm_qkv_mqv2_wmma_gfx1201_bt.hip lines 73-74
///   const int rs = blockIdx.x * 16;
///   const int bs = blockIdx.y * (16 * BV); // BV=8 => 128
///   Grid: [ceil(total_m/16), ceil(N/128), 1]
pub fn grid(desc: &KernelDesc, shape: &Shape) -> [u32; 3] {
    let total_m = shape.total_m();
    let n = shape.n_tokens;
    let batch_tile = desc.variant.batch_tile_tokens();
    let gx = total_m.div_ceil(16);
    let gy = n.div_ceil(batch_tile);
    [gx, gy, 1]
}

/// Kernarg layout per ABI: ptrs (8-byte) then ints (4-byte) natural alignment.
pub fn kernarg_layout(desc: &KernelDesc) -> KernargLayout {
    let p = desc.family.projections();
    let mut slots: Vec<ArgSlot> = Vec::new();
    let mut offset: u32 = 0;
    let align = 8u32;

    let align_up = |off: u32, a: u32| -> u32 { (off + a - 1) / a * a };

    // Weight pointers per projection.
    for i in 0..p {
        offset = align_up(offset, 8);
        let name = match (desc.family, i) {
            (Family::Qkvza, 0) => "A_qkv".to_owned(),
            (Family::Qkvza, 1) => "A_z".to_owned(),
            (Family::Qkvza, 2) => "A_beta".to_owned(),
            (Family::Qkvza, 3) => "A_alpha".to_owned(),
            (Family::Qkv, 0) => "A_q".to_owned(),
            (Family::Qkv, 1) => "A_k".to_owned(),
            (Family::Qkv, 2) => "A_v".to_owned(),
            (Family::GateUp, 0) => "A_gate".to_owned(),
            (Family::GateUp, 1) => "A_up".to_owned(),
            (Family::Residual, 0) => "A".to_owned(),
            _ => format!("A{i}"),
        };
        slots.push(ArgSlot {
            name,
            kind: ArgKind::Ptr,
            offset,
            size: 8,
            binding: Some(PtrBinding::Weights(i)),
        });
        offset += 8;
    }
    // X
    offset = align_up(offset, 8);
    slots.push(ArgSlot {
        name: "X".to_owned(),
        kind: ArgKind::Ptr,
        offset,
        size: 8,
        binding: Some(PtrBinding::X),
    });
    offset += 8;

    // Y pointers per projection
    for i in 0..p {
        offset = align_up(offset, 8);
        let name = match (desc.family, i) {
            (Family::Qkvza, 0) => "Y_qkv".to_owned(),
            (Family::Qkvza, 1) => "Y_z".to_owned(),
            (Family::Qkvza, 2) => "Y_beta".to_owned(),
            (Family::Qkvza, 3) => "Y_alpha".to_owned(),
            (Family::Qkv, 0) => "Y_q".to_owned(),
            (Family::Qkv, 1) => "Y_k".to_owned(),
            (Family::Qkv, 2) => "Y_v".to_owned(),
            (Family::GateUp, 0) => "Y_gate".to_owned(),
            (Family::GateUp, 1) => "Y_up".to_owned(),
            (Family::Residual, 0) => "Y".to_owned(),
            _ => format!("Y{i}"),
        };
        slots.push(ArgSlot {
            name,
            kind: ArgKind::Ptr,
            offset,
            size: 8,
            binding: Some(PtrBinding::Y(i)),
        });
        offset += 8;
    }

    // ints per projection (m)
    for i in 0..p {
        offset = align_up(offset, 4);
        let name = match (desc.family, i) {
            (Family::Qkvza, 0) => "qkv_m".to_owned(),
            (Family::Qkvza, 1) => "z_m".to_owned(),
            (Family::Qkvza, 2) => "beta_m".to_owned(),
            (Family::Qkvza, 3) => "alpha_m".to_owned(),
            (Family::Qkv, 0) => "q_m".to_owned(),
            (Family::Qkv, 1) => "k_m".to_owned(),
            (Family::Qkv, 2) => "v_m".to_owned(),
            (Family::GateUp, 0) => "gate_m".to_owned(),
            (Family::GateUp, 1) => "up_m".to_owned(),
            (Family::Residual, 0) => "M".to_owned(),
            _ => format!("m{i}"),
        };
        slots.push(ArgSlot {
            name,
            kind: ArgKind::I32,
            offset,
            size: 4,
            binding: None,
        });
        offset += 4;
    }
    // K
    offset = align_up(offset, 4);
    slots.push(ArgSlot {
        name: "K".to_owned(),
        kind: ArgKind::I32,
        offset,
        size: 4,
        binding: None,
    });
    offset += 4;
    // N
    offset = align_up(offset, 4);
    slots.push(ArgSlot {
        name: "N".to_owned(),
        kind: ArgKind::I32,
        offset,
        size: 4,
        binding: None,
    });
    offset += 4;

    let explicit_size = align_up(offset, align);
    KernargLayout {
        slots,
        explicit_size,
        align,
    }
}

/// Code object bytes embedded via include_bytes! of build.rs outputs.
/// When an (arch, profile) HSACO was not produced (placeholder empty file)
/// this returns an empty slice and the driver must skip.
pub fn code_object(arch: Arch, profile: SchedulerProfile) -> &'static [u8] {
    match (arch, profile) {
        (Arch::Gfx1100, SchedulerProfile::Default) => include_bytes!(concat!(env!("OUT_DIR"), "/mqv2_gfx1100_default.hsaco")),
        (Arch::Gfx1100, SchedulerProfile::MaxIlp) => include_bytes!(concat!(env!("OUT_DIR"), "/mqv2_gfx1100_max_ilp.hsaco")),
        (Arch::Gfx1100, SchedulerProfile::IterativeIlp) => include_bytes!(concat!(env!("OUT_DIR"), "/mqv2_gfx1100_iterative_ilp.hsaco")),
        (Arch::Gfx1100, SchedulerProfile::MemoryClause) => include_bytes!(concat!(env!("OUT_DIR"), "/mqv2_gfx1100_memory_clause.hsaco")),
        (Arch::Gfx1100, SchedulerProfile::PipelineIlp) => include_bytes!(concat!(env!("OUT_DIR"), "/mqv2_gfx1100_pipeline_ilp.hsaco")),
        (Arch::Gfx1151, SchedulerProfile::Default) => include_bytes!(concat!(env!("OUT_DIR"), "/mqv2_gfx1151_default.hsaco")),
        (Arch::Gfx1151, SchedulerProfile::MaxIlp) => include_bytes!(concat!(env!("OUT_DIR"), "/mqv2_gfx1151_max_ilp.hsaco")),
        (Arch::Gfx1151, SchedulerProfile::IterativeIlp) => include_bytes!(concat!(env!("OUT_DIR"), "/mqv2_gfx1151_iterative_ilp.hsaco")),
        (Arch::Gfx1151, SchedulerProfile::MemoryClause) => include_bytes!(concat!(env!("OUT_DIR"), "/mqv2_gfx1151_memory_clause.hsaco")),
        (Arch::Gfx1151, SchedulerProfile::PipelineIlp) => include_bytes!(concat!(env!("OUT_DIR"), "/mqv2_gfx1151_pipeline_ilp.hsaco")),
        (Arch::Gfx1201, SchedulerProfile::Default) => include_bytes!(concat!(env!("OUT_DIR"), "/mqv2_gfx1201_default.hsaco")),
        (Arch::Gfx1201, SchedulerProfile::MaxIlp) => include_bytes!(concat!(env!("OUT_DIR"), "/mqv2_gfx1201_max_ilp.hsaco")),
        (Arch::Gfx1201, SchedulerProfile::IterativeIlp) => include_bytes!(concat!(env!("OUT_DIR"), "/mqv2_gfx1201_iterative_ilp.hsaco")),
        (Arch::Gfx1201, SchedulerProfile::MemoryClause) => include_bytes!(concat!(env!("OUT_DIR"), "/mqv2_gfx1201_memory_clause.hsaco")),
        (Arch::Gfx1201, SchedulerProfile::PipelineIlp) => include_bytes!(concat!(env!("OUT_DIR"), "/mqv2_gfx1201_pipeline_ilp.hsaco")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Shape;

    #[test]
    fn grid_smoke_bt4() {
        let desc = KernelDesc {
            symbol: "gemm_qkv_mq6g256v2_wmma_gfx11_bt4".into(),
            family: Family::Qkv,
            bits: 6,
            variant: Variant::Bt { bv: 4 },
            archs: vec![Arch::Gfx1151],
            source: "".into(),
        };
        let shape = Shape { n_tokens: 16, k: 512, proj_m: vec![64, 32, 32] };
        // total_m=128 => ceil(128/16)=8; batch_tile=64 => ceil(16/64)=1
        assert_eq!(grid(&desc, &shape), [8, 1, 1]);
        let shape2 = Shape { n_tokens: 128, k: 2048, proj_m: vec![64, 32, 32] };
        assert_eq!(grid(&desc, &shape2), [8, 2, 1]);
    }

    #[test]
    fn grid_mw() {
        let desc = KernelDesc {
            symbol: "gemm_gate_up_mq5g256v2_wmma_gfx11_mw8_lds".into(),
            family: Family::GateUp,
            bits: 5,
            variant: Variant::MwLds { nw: 8 },
            archs: vec![],
            source: "".into(),
        };
        let shape = Shape { n_tokens: 128, k: 2048, proj_m: vec![64, 64] };
        // batch_tile=128 => ceil(128/128)=1
        assert_eq!(grid(&desc, &shape), [8, 1, 1]);
        let shape2 = Shape { n_tokens: 130, k: 2048, proj_m: vec![64, 64] };
        assert_eq!(grid(&desc, &shape2), [8, 2, 1]);
    }

    #[test]
    fn grid_gfx1201_bt8() {
        let desc = KernelDesc {
            symbol: "gemm_qkv_mq2g256v2_wmma_gfx1201_bt8".into(),
            family: Family::Qkv,
            bits: 2,
            variant: Variant::Gfx1201Bt8,
            archs: vec![],
            source: "".into(),
        };
        let shape = Shape { n_tokens: 128, k: 2048, proj_m: vec![64, 32, 32] };
        assert_eq!(grid(&desc, &shape), [8, 1, 1]);
        let shape2 = Shape { n_tokens: 129, k: 2048, proj_m: vec![64, 32, 32] };
        assert_eq!(grid(&desc, &shape2), [8, 2, 1]);
    }

    #[test]
    fn kernarg_qkv_offsets() {
        let desc = KernelDesc {
            symbol: "gemm_qkv_mq5g256v2_wmma_gfx11_bt4".into(),
            family: Family::Qkv,
            bits: 5,
            variant: Variant::Bt { bv: 4 },
            archs: vec![],
            source: "".into(),
        };
        let layout = kernarg_layout(&desc);
        // 7 ptrs at 0,8,16,24,32,40,48
        let ptr_offsets: Vec<u32> = layout.slots.iter().filter(|s| s.kind == ArgKind::Ptr).map(|s| s.offset).collect();
        assert_eq!(ptr_offsets, vec![0, 8, 16, 24, 32, 40, 48]);
        // ints at 56,60,64,68,72 => explicit 76 rounded to 80
        let int_offsets: Vec<u32> = layout.slots.iter().filter(|s| s.kind == ArgKind::I32).map(|s| s.offset).collect();
        assert_eq!(int_offsets, vec![56, 60, 64, 68, 72]);
        assert_eq!(layout.explicit_size, 80);
        assert_eq!(layout.align, 8);
        // bindings
        assert_eq!(layout.slots[0].binding, Some(PtrBinding::Weights(0)));
        assert_eq!(layout.slots[3].binding, Some(PtrBinding::X));
        assert_eq!(layout.slots[6].binding, Some(PtrBinding::Y(2)));
    }

    #[test]
    fn kernarg_residual_offsets() {
        let desc = KernelDesc {
            symbol: "gemm_mq6g256v2_residual_wmma_gfx11_bt4".into(),
            family: Family::Residual,
            bits: 6,
            variant: Variant::Bt { bv: 4 },
            archs: vec![],
            source: "".into(),
        };
        let layout = kernarg_layout(&desc);
        let ptr_offsets: Vec<u32> = layout.slots.iter().filter(|s| s.kind == ArgKind::Ptr).map(|s| s.offset).collect();
        assert_eq!(ptr_offsets, vec![0, 8, 16]);
        let int_offsets: Vec<u32> = layout.slots.iter().filter(|s| s.kind == ArgKind::I32).map(|s| s.offset).collect();
        assert_eq!(int_offsets, vec![24, 28, 32]);
        assert_eq!(layout.explicit_size, 40);
    }

    #[test]
    fn kernarg_qkvza_offsets() {
        let desc = KernelDesc {
            symbol: "gemm_qkvza_mq3g256v2_wmma_gfx11_bt4".into(),
            family: Family::Qkvza,
            bits: 3,
            variant: Variant::Bt { bv: 4 },
            archs: vec![],
            source: "".into(),
        };
        let layout = kernarg_layout(&desc);
        assert_eq!(layout.slots.iter().filter(|s| s.kind==ArgKind::Ptr).count(), 9);
        assert_eq!(layout.slots.iter().filter(|s| s.kind==ArgKind::I32).count(), 6);
        assert_eq!(layout.explicit_size, 96);
    }

    #[test]
    fn descriptor_counts() {
        assert_eq!(descriptors(Arch::Gfx1100).len(), 47);
        assert_eq!(descriptors(Arch::Gfx1151).len(), 47);
        assert_eq!(descriptors(Arch::Gfx1201).len(), 4);
    }
}
