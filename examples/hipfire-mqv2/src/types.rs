// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
//
// Shared contracts for the mqv2 WMMA microbench. Every module in this crate
// builds against this file and nothing else across the kernel/oracle half
// (build.rs, kernels.rs, oracle.rs, fixture.rs) and the runtime/driver half
// (hip_backend.rs, redline_backend.rs, spec.rs, driver.rs, report.rs).
// Change it only by agreement between the two halves.

use radiowave::SchedulerProfile;
use serde::{Deserialize, Serialize};

/// mqv2 quantisation group: 256 weights, dual-half FP16 affine header.
pub const GROUP_SIZE: u32 = 256;

/// Bytes per G256 group on the wire: 8-byte header `[s0 z0 s1 z1]` (four
/// FP16) then a contiguous LSB-first payload of 256 codes of `bits` bits.
/// hipfire authority: crates/rdna-compute/src/dispatch.rs (MQ2V2=72,
/// MQ3V2=104, MQ4V2=136, MQ5V2=168, MQ6V2=200).
pub const fn group_bytes(bits: u32) -> u32 {
    8 + 32 * bits
}

/// hipfire's fused/WMMA correctness bound: relative RMS error of the whole
/// output against an f64 reference (`FUSED_TOL_RMS` in
/// crates/hipfire-runtime/examples/mqv2_family_parity.rs).
pub const REL_RMS_TOL: f64 = 0.05;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Arch {
    Gfx1100,
    Gfx1151,
    Gfx1201,
}

impl Arch {
    pub const ALL: [Arch; 3] = [Arch::Gfx1100, Arch::Gfx1151, Arch::Gfx1201];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Gfx1100 => "gfx1100",
            Self::Gfx1151 => "gfx1151",
            Self::Gfx1201 => "gfx1201",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|arch| arch.as_str() == value)
    }

    /// gfx11 kernels (`gemm_mqv2_wmma_gfx11_*.hip`) exist on these two.
    pub const fn is_gfx11(self) -> bool {
        matches!(self, Self::Gfx1100 | Self::Gfx1151)
    }
}

/// Fused projection family. The number of weight/output projections is the
/// only thing the harness needs to know about a family; the kernels own the
/// rest.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Family {
    /// q, k, v, z, beta, alpha style fused projection: 4 weight/output pairs
    /// (qkv, z, beta, alpha) in hipfire's `GEN_QKVZA_BT`.
    Qkvza,
    /// q, k, v: 3 pairs.
    Qkv,
    /// gate, up: 2 pairs.
    GateUp,
    /// single projection, `Y += W X` (accumulates into Y).
    Residual,
}

impl Family {
    pub const fn projections(self) -> usize {
        match self {
            Self::Qkvza => 4,
            Self::Qkv => 3,
            Self::GateUp => 2,
            Self::Residual => 1,
        }
    }

    /// `true` when the kernel accumulates into Y (`+=`) rather than
    /// overwriting it. Only `Residual` accumulates.
    pub const fn accumulates(self) -> bool {
        matches!(self, Self::Residual)
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Qkvza => "qkvza",
            Self::Qkv => "qkv",
            Self::GateUp => "gate_up",
            Self::Residual => "residual",
        }
    }
}

/// Kernel structural variant. Determines block size, static LDS, and the
/// token batch tile.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Variant {
    /// gfx11 batch-tiled single-wave WMMA: block 32, LDS 0, tile 16*bv tokens.
    Bt { bv: u32 },
    /// gfx11 multi-wave same-row LDS: block 32*nw, static LDS 8192, tile 16*nw.
    MwLds { nw: u32 },
    /// gfx1201 QKV BT8: block 32, LDS 0, tile 128 tokens.
    Gfx1201Bt8,
}

impl Variant {
    pub const fn block(self) -> u32 {
        match self {
            Self::Bt { .. } | Self::Gfx1201Bt8 => 32,
            Self::MwLds { nw } => 32 * nw,
        }
    }

    pub const fn static_lds_bytes(self) -> u32 {
        match self {
            Self::MwLds { .. } => 8192,
            _ => 0,
        }
    }

    pub const fn batch_tile_tokens(self) -> u32 {
        match self {
            Self::Bt { bv } => 16 * bv,
            Self::MwLds { nw } => 16 * nw,
            Self::Gfx1201Bt8 => 128,
        }
    }

    pub fn as_str(self) -> String {
        match self {
            Self::Bt { bv } => format!("bt{bv}"),
            Self::MwLds { nw } => format!("mw{nw}_lds"),
            Self::Gfx1201Bt8 => "gfx1201_bt8".to_owned(),
        }
    }
}

/// One `__global__` symbol in the imported kernel sources.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KernelDesc {
    /// Exact exported symbol, e.g. `gemm_qkv_mq5g256v2_wmma_gfx11_bt4`.
    pub symbol: String,
    pub family: Family,
    pub bits: u32,
    pub variant: Variant,
    /// Architectures whose HSACO contains this symbol.
    pub archs: Vec<Arch>,
    /// Source file under `kernels/` the symbol comes from (for provenance).
    pub source: String,
}

impl KernelDesc {
    pub fn block(&self) -> u32 {
        self.variant.block()
    }

    pub fn static_lds_bytes(&self) -> u32 {
        self.variant.static_lds_bytes()
    }

    /// Launch grid for a shape. `kernels.rs` owns the definitive rule read off
    /// the kernel's `blockIdx` usage; this is the shared contract for it:
    /// one workgroup per (16-row tile of the concatenated projection rows) x
    /// (batch tile of tokens). The implementation must return
    /// `[x, y, 1]` in the order the kernel indexes them.
    pub fn grid(&self, shape: &Shape) -> [u32; 3] {
        crate::kernels::grid(self, shape)
    }
}

/// Problem shape. `proj_m.len() == family.projections()`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Shape {
    /// Tokens (N in the kernel signature). Activations are `[n_tokens x k]`.
    pub n_tokens: u32,
    /// Reduction length; must be a multiple of `GROUP_SIZE`.
    pub k: u32,
    /// Output rows per projection (the `*_m` kernel arguments).
    pub proj_m: Vec<u32>,
}

impl Shape {
    pub fn total_m(&self) -> u32 {
        self.proj_m.iter().sum()
    }

    pub fn groups_per_row(&self) -> u32 {
        self.k / GROUP_SIZE
    }

    pub fn label(&self) -> String {
        let m = self
            .proj_m
            .iter()
            .map(|m| m.to_string())
            .collect::<Vec<_>>()
            .join("+");
        format!("n{}_k{}_m{}", self.n_tokens, self.k, m)
    }
}

/// Kernarg slot kinds. The explicit-argument ABI of every mqv2 kernel is, in
/// signature order: one `const char*` per projection (weights), one
/// `const _Float16*` (X), one `float*` per projection (Y), then one `int` per
/// projection (m), `int K`, `int N`. Pointers are 8-byte aligned, ints 4-byte,
/// packed with natural alignment in order (hipcc HSA explicit kernarg rule).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ArgKind {
    Ptr,
    I32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PtrBinding {
    /// Packed weights of projection `i`.
    Weights(usize),
    /// FP16 activations `[n_tokens x k]`.
    X,
    /// FP32 output of projection `i`, `[n_tokens x proj_m[i]]` row-major
    /// (`Y[token * proj_m + row]`).
    Y(usize),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArgSlot {
    pub name: String,
    pub kind: ArgKind,
    /// Byte offset inside the explicit kernarg segment.
    pub offset: u32,
    pub size: u32,
    /// Present for `ArgKind::Ptr` slots.
    pub binding: Option<PtrBinding>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KernargLayout {
    pub slots: Vec<ArgSlot>,
    /// Explicit segment size (hidden arguments, if any, follow at the offset
    /// the code object metadata reports; backends must honour the HSACO's
    /// `.kernarg_segment_size` and alignment when allocating).
    pub explicit_size: u32,
    pub align: u32,
}

impl KernargLayout {
    /// Values for the I32 slots in slot order for a shape:
    /// `proj_m[0..p]`, then `k`, then `n_tokens`.
    pub fn i32_values(desc: &KernelDesc, shape: &Shape) -> Vec<i32> {
        debug_assert_eq!(shape.proj_m.len(), desc.family.projections());
        let mut values = shape.proj_m.iter().map(|m| *m as i32).collect::<Vec<_>>();
        values.push(shape.k as i32);
        values.push(shape.n_tokens as i32);
        values
    }
}

/// Deterministic inputs plus the f64 reference for ONE launch.
#[derive(Clone, Debug)]
pub struct Fixture {
    pub shape: Shape,
    pub bits: u32,
    /// Per projection: `proj_m[i] * groups_per_row * group_bytes(bits)` bytes.
    pub weights: Vec<Vec<u8>>,
    /// `[n_tokens x k]` FP16 bit patterns, row-major by token.
    pub x_f16: Vec<u16>,
    /// Per projection, initial Y contents (`[n_tokens x proj_m[i]]`). Zero for
    /// overwriting families; a deterministic non-zero canary pattern for
    /// `Residual` so accumulation is exercised.
    pub y_init: Vec<Vec<f32>>,
    /// Per projection, the f64 reference of one launch rounded to f32
    /// (`W X` for overwriting families, `y_init + W X` for `Residual`).
    pub expected_once: Vec<Vec<f32>>,
}

impl Fixture {
    /// Expected Y after `launches` chained launches on the same Y.
    /// Overwriting families are idempotent; `Residual` accumulates.
    pub fn expected_after(&self, family: Family, launches: usize) -> Vec<Vec<f32>> {
        if !family.accumulates() || launches <= 1 {
            return self.expected_once.clone();
        }
        self.expected_once
            .iter()
            .zip(&self.y_init)
            .map(|(once, init)| {
                once.iter()
                    .zip(init)
                    .map(|(o, i)| {
                        let delta = f64::from(*o) - f64::from(*i);
                        (f64::from(*i) + delta * launches as f64) as f32
                    })
                    .collect()
            })
            .collect()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Verdict {
    pub pass: bool,
    pub rel_rms: f64,
    pub max_abs: f64,
    pub compared: usize,
    /// Set when the comparison could not run (missing output, NaN, error).
    pub note: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimingMode {
    /// `iterations` launches chained on one stream / one retained IB with a
    /// dependency boundary between them; one Y set.
    SerialLatency,
    /// `iterations` launches with no dependencies, each into its own Y set,
    /// spread across the backend's independent queues/streams.
    IndependentThroughput,
}

impl TimingMode {
    pub const ALL: [TimingMode; 2] = [TimingMode::SerialLatency, TimingMode::IndependentThroughput];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SerialLatency => "serial_latency",
            Self::IndependentThroughput => "independent_throughput",
        }
    }
}

/// One benchmark row: a kernel at a shape under a timing mode.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RowSpec {
    pub kernel: KernelDesc,
    pub shape: Shape,
    pub mode: TimingMode,
    /// Launches per timed sample (chain length or independent fan-out).
    pub iterations: usize,
    pub scheduler_profile: SchedulerProfile,
    /// Always 32 for the imported kernels (they use `_w32` WMMA builtins).
    pub wave_size: u32,
}

impl RowSpec {
    pub fn name(&self) -> String {
        format!(
            "{}/{}_mq{}_{}/{}",
            self.kernel.family.as_str(),
            self.kernel.symbol,
            self.kernel.bits,
            self.kernel.variant.as_str(),
            self.shape.label()
        )
    }

    pub fn key(&self) -> String {
        format!("{}/{}", self.mode.as_str(), self.name())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Distribution {
    pub min_us: f64,
    pub p05_us: f64,
    pub median_us: f64,
    pub p95_us: f64,
    pub max_us: f64,
    pub samples_us: Vec<f64>,
}

impl Distribution {
    pub fn from_samples(mut samples_us: Vec<f64>) -> Self {
        let mut sorted = samples_us.clone();
        sorted.sort_by(|a, b| a.total_cmp(b));
        let pick = |q: f64| -> f64 {
            if sorted.is_empty() {
                return f64::NAN;
            }
            let pos = q * (sorted.len() - 1) as f64;
            let lo = pos.floor() as usize;
            let hi = pos.ceil() as usize;
            let t = pos - lo as f64;
            sorted[lo] * (1.0 - t) + sorted[hi] * t
        };
        let dist = Self {
            min_us: pick(0.0),
            p05_us: pick(0.05),
            median_us: pick(0.5),
            p95_us: pick(0.95),
            max_us: pick(1.0),
            samples_us: Vec::new(),
        };
        samples_us.shrink_to_fit();
        Self { samples_us, ..dist }
    }
}

/// What a backend returns for one row.
#[derive(Clone, Debug)]
pub struct RunOutput {
    /// Per Y set (1 for serial, `iterations` for independent), per projection.
    pub outputs: Vec<Vec<Vec<f32>>>,
    /// Per-launch microseconds for each timed sample (`span / iterations`).
    pub samples_us: Vec<f64>,
    /// Backend-specific notes worth persisting (queue count, IB policy, ...).
    pub notes: serde_json::Value,
}

/// Backend contract. Implementations: `hip` (hipModuleLaunchKernel on one
/// stream, or N streams for independent), `hipgraph` (captured graph replay),
/// `redline` (retained PM4 IB, single or multi-queue).
pub trait Backend {
    fn name(&self) -> &'static str;

    /// Upload the fixture, run `warmups` untimed then `samples` timed rounds
    /// of `row.iterations` launches in `row.mode`, and read every Y set back.
    /// Y must be reset to `fixture.y_init` before every round so serial
    /// accumulation is measured from a known state.
    fn run(
        &mut self,
        row: &RowSpec,
        fixture: &Fixture,
        warmups: usize,
        samples: usize,
    ) -> anyhow::Result<RunOutput>;
}

/// Per-backend result persisted into the row JSON. `output_sha256` is over
/// the f32 bit patterns of every Y set so identical-HSACO backends can be
/// checked for bit-identity across dispatchers.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BackendResult {
    pub correctness: Verdict,
    pub distribution: Distribution,
    pub output_sha256: String,
    pub notes: serde_json::Value,
    pub error: Option<String>,
}
