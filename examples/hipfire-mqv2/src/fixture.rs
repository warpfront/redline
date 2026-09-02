// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

use crate::oracle;
use crate::types::{Arch, Fixture, KernelDesc, Shape, GROUP_SIZE};
use half::f16;

fn prng(i: usize, salt: u32) -> f32 {
    let x = (i as u32)
        .wrapping_mul(0x9E37_79B9)
        .wrapping_add(salt.wrapping_mul(0x85EB_CA6B));
    let x = x ^ (x >> 15);
    let x = x.wrapping_mul(0x2545_F491);
    let x = x ^ (x >> 13);
    (x >> 8) as f32 / (1u32 << 24) as f32
}

/// Build deterministic inputs + f64 reference for ONE launch.
///
/// - Weights: hipfire-style discriminating halves per G256 group. For each row
///   and group, half0 in [-1,1] (small scale/zero), half1 in [96,160] (large).
///   Codes are the quantized q in [0, 2^bits) uniform-ish due to uniform w.
///   Salted per projection, row and group via seed.
/// - X: FP16 in [-1,1] uniform via prng (row-major [n_tokens x k]).
/// - y_init: zero except Residual = deterministic canary pattern (small non-zero).
/// - expected_once: f64 GEMM of decoded W * X (+ y_init if Residual), rounded f32.
pub fn build(desc: &KernelDesc, shape: &Shape, seed: u64) -> Fixture {
    let bits = desc.bits;
    assert!(shape.proj_m.len() == desc.family.projections());
    assert_eq!(shape.k % GROUP_SIZE, 0);
    let gpr = shape.k / GROUP_SIZE;
    let n = shape.n_tokens as usize;
    let k = shape.k as usize;

    // Weights per projection as packed blobs.
    let mut weights: Vec<Vec<u8>> = Vec::with_capacity(shape.proj_m.len());
    let mut expected_once: Vec<Vec<f32>> = Vec::with_capacity(shape.proj_m.len());
    let mut y_init: Vec<Vec<f32>> = Vec::with_capacity(shape.proj_m.len());

    for (proj_idx, &m_u) in shape.proj_m.iter().enumerate() {
        let m = m_u as usize;
        // Generate w_f32 with discriminating halves.
        let mut w_f32 = vec![0.0f32; m * k];
        for r in 0..m {
            for g in 0..gpr as usize {
                let base = r * k + g * GROUP_SIZE as usize;
                // salt per projection/row/group derived from seed
                let salt = (seed as u32)
                    .wrapping_add((proj_idx as u32).wrapping_mul(0x9E37_79B9))
                    .wrapping_add((r as u32).wrapping_mul(7919))
                    .wrapping_add((g as u32).wrapping_mul(104729));
                // half0 small [-1,1]
                for i in 0..128 {
                    w_f32[base + i] = prng(i, salt) * 2.0 - 1.0;
                }
                // half1 large [96,160]
                for i in 128..256 {
                    w_f32[base + i] = 96.0 + prng(i, salt ^ 0xA5A5_A5A5) * 64.0;
                }
            }
        }
        let blob = oracle::pack_blob(bits, m, k, &w_f32);
        weights.push(blob);
    }

    // X FP16 row-major [n x k] in [-1,1]
    let mut x_f16 = vec![0u16; n * k];
    for i in 0..n * k {
        let v = prng(i, (seed ^ 0xC0FFEE) as u32) * 2.0 - 1.0;
        x_f16[i] = f16::from_f32(v).to_bits();
    }

    // y_init and expected per projection
    for (proj_idx, &m_u) in shape.proj_m.iter().enumerate() {
        let m = m_u as usize;
        let blob = &weights[proj_idx];
        // y_init
        let y0: Vec<f32> = if desc.family.accumulates() {
            // deterministic canary: small pattern via prng, non-zero to exercise +=
            (0..n * m)
                .map(|i| {
                    let s = (seed ^ 0xDEADBEEF) as u32 ^ (proj_idx as u32).wrapping_mul(0x85EB_CA6B);
                    prng(i, s) * 2.0 - 1.5 // mix around -1.5..0.5
                })
                .collect()
        } else {
            vec![0.0f32; n * m]
        };

        // reference GEMM W*X
        let mut y_ref = oracle::reference_gemm(bits, m, k, n, blob, &x_f16);
        if desc.family.accumulates() {
            for (dst, &init) in y_ref.iter_mut().zip(&y0) {
                *dst += init;
            }
        }
        y_init.push(y0);
        expected_once.push(y_ref);
    }

    Fixture {
        shape: shape.clone(),
        bits,
        weights,
        x_f16,
        y_init,
        expected_once,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Family, Variant, KernelDesc, Arch};

    fn make_desc(family: Family, bits: u32) -> KernelDesc {
        let variant = Variant::Bt { bv: 4 };
        KernelDesc {
            symbol: format!("test_mq{}_bt4", bits),
            family,
            bits,
            variant,
            archs: vec![Arch::Gfx1151],
            source: "".into(),
        }
    }

    #[test]
    fn build_weights_sizes() {
        let desc = make_desc(Family::Qkv, 4);
        let shape = Shape { n_tokens: 16, k: 512, proj_m: vec![64, 32, 32] };
        let fx = build(&desc, &shape, 42);
        assert_eq!(fx.weights.len(), 3);
        // each blob size = m * gpr * group_bytes
        let gpr = shape.k / GROUP_SIZE;
        for (proj, &m) in fx.weights.iter().zip(&shape.proj_m) {
            assert_eq!(proj.len(), m as usize * gpr as usize * oracle::group_bytes(4));
        }
        assert_eq!(fx.x_f16.len(), 16 * 512);
        assert_eq!(fx.y_init.iter().all(|v| v.iter().all(|&x| x == 0.0)), true);
    }

    #[test]
    fn build_residual_canary_and_accumulation() {
        let desc = make_desc(Family::Residual, 5);
        let shape = Shape { n_tokens: 2, k: 256, proj_m: vec![4] };
        let fx = build(&desc, &shape, 123);
        // y_init non-zero
        assert!(fx.y_init[0].iter().any(|&v| v != 0.0));
        // expected_once = y_init + W*X, so delta = expected_once - y_init should be W*X
        // Verify via reference without y_init
        let y_wx = oracle::reference_gemm(5, 4, 256, 2, &fx.weights[0], &fx.x_f16);
        for (i, &y0) in fx.y_init[0].iter().enumerate() {
            let expected = y_wx[i] + y0;
            assert!((fx.expected_once[0][i] - expected).abs() < 1e-6);
        }
        // expected_after for 3 launches
        let after = fx.expected_after(Family::Residual, 3);
        for i in 0..fx.y_init[0].len() {
            let delta = fx.expected_once[0][i] as f64 - fx.y_init[0][i] as f64;
            let exp = fx.y_init[0][i] as f64 + delta * 3.0;
            // tolerance accounts for f32 rounding at magnitude ~ few thousand
            assert!((after[0][i] as f64 - exp).abs() < 1e-3, "i {i} after {} exp {} diff {}", after[0][i], exp, (after[0][i] as f64 - exp).abs());
        }
    }

    #[test]
    fn build_x_range() {
        let desc = make_desc(Family::GateUp, 3);
        let shape = Shape { n_tokens: 4, k: 256, proj_m: vec![8, 8] };
        let fx = build(&desc, &shape, 999);
        for &bits in &fx.x_f16 {
            let v = f16::from_bits(bits).to_f32();
            assert!(v >= -1.0 - 1e-6 && v <= 1.0 + 1e-6, "x {v}");
        }
    }

    #[test]
    fn deterministic_seed() {
        let desc = make_desc(Family::Qkv, 6);
        let shape = Shape { n_tokens: 2, k: 256, proj_m: vec![2, 2, 2] };
        let a = build(&desc, &shape, 777);
        let b = build(&desc, &shape, 777);
        assert_eq!(a.weights, b.weights);
        assert_eq!(a.x_f16, b.x_f16);
        assert_eq!(a.y_init, b.y_init);
        assert_eq!(a.expected_once, b.expected_once);
        let c = build(&desc, &shape, 778);
        assert_ne!(a.weights, c.weights);
    }
}
