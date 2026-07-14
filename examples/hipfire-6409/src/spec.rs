// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

use half::f16;
use radiowave::SchedulerProfile;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WavePolicy {
    AllWave32,
    TargetedWave64,
    RadiowaveTuned,
    BlanketWave64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatrixProfile {
    HipEngineF2c,
    LegacyHipfire,
}

impl MatrixProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::HipEngineF2c => "hipengine_f2c",
            Self::LegacyHipfire => "legacy_hipfire",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "hipengine" | "hipengine_f2c" | "f2c" => Some(Self::HipEngineF2c),
            "legacy" | "legacy_hipfire" | "hipfire" => Some(Self::LegacyHipfire),
            _ => None,
        }
    }
}

impl WavePolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AllWave32 => "all_wave32",
            Self::TargetedWave64 => "targeted_wave64",
            Self::RadiowaveTuned => "radiowave_tuned",
            Self::BlanketWave64 => "blanket_wave64",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "all32" | "all_wave32" => Some(Self::AllWave32),
            "targeted64" | "targeted_wave64" => Some(Self::TargetedWave64),
            "radiowave" | "radiowave_tuned" => Some(Self::RadiowaveTuned),
            "blanket64" | "blanket_wave64" => Some(Self::BlanketWave64),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimingMode {
    SerialLatency,
    IndependentThroughput,
    SingleKernelAggressive,
}

impl TimingMode {
    pub const ALL: [Self; 3] = [
        Self::SerialLatency,
        Self::IndependentThroughput,
        Self::SingleKernelAggressive,
    ];

    pub const HIPENGINE_COMPARABLE: [Self; 2] = [Self::SerialLatency, Self::IndependentThroughput];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::SerialLatency => "serial_latency",
            Self::IndependentThroughput => "independent_throughput",
            Self::SingleKernelAggressive => "single_kernel_aggressive",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputKind {
    F32,
    U32,
    TwoStageF32,
    Q8Overwrite,
    Q8ThenU32,
}

#[derive(Clone, Debug)]
pub struct RowSpec {
    pub family: &'static str,
    pub name: String,
    pub kernel: &'static str,
    pub second_kernel: Option<&'static str>,
    pub second_n1: u32,
    pub second_aux: u32,
    pub second_output_delta: u32,
    pub second_block: u32,
    pub n0: u32,
    pub n1: u32,
    pub aux: u32,
    pub block: u32,
    pub wave_size: u32,
    pub scheduler_profile: SchedulerProfile,
    pub grid_groups: u32,
    pub second_grid_groups: u32,
    pub output_per_op: usize,
    pub iterations: usize,
    pub kind: OutputKind,
}

impl RowSpec {
    pub fn key(&self, mode: TimingMode) -> String {
        format!(
            "{}/{}/{};hip-wave={}",
            mode.as_str(),
            self.family,
            self.name,
            self.wave_size
        )
    }

    pub fn supports_mode(&self, mode: TimingMode) -> bool {
        mode != TimingMode::SingleKernelAggressive || self.second_kernel.is_none()
    }

    pub fn logical_iterations(&self, mode: TimingMode) -> usize {
        match mode {
            TimingMode::SingleKernelAggressive => 1,
            TimingMode::SerialLatency | TimingMode::IndependentThroughput => self.iterations,
        }
    }

    pub fn output_words(&self, mode: TimingMode) -> usize {
        match mode {
            TimingMode::SerialLatency | TimingMode::SingleKernelAggressive => self.output_per_op,
            TimingMode::IndependentThroughput => self.output_per_op * self.iterations,
        }
    }

    pub fn output_offset(&self, mode: TimingMode, operation: usize) -> u32 {
        match mode {
            TimingMode::SerialLatency | TimingMode::SingleKernelAggressive => 0,
            TimingMode::IndependentThroughput => (operation * self.output_per_op) as u32,
        }
    }

    pub fn stage_n1(&self, second: bool) -> u32 {
        if second && self.second_n1 != 0 {
            self.second_n1
        } else {
            self.n1
        }
    }

    pub fn stage_aux(&self, second: bool) -> u32 {
        if second && self.second_aux != 0 {
            self.second_aux
        } else {
            self.aux
        }
    }

    pub fn stage_output_offset(&self, mode: TimingMode, operation: usize, second: bool) -> u32 {
        self.output_offset(mode, operation) + if second { self.second_output_delta } else { 0 }
    }

    pub fn stage_block(&self, second: bool) -> u32 {
        if second && self.second_block != 0 {
            self.second_block
        } else {
            self.block
        }
    }
}

#[derive(Clone, Debug)]
pub struct Fixture {
    pub a: Vec<u32>,
    pub b: Vec<u32>,
    pub one_op: Vec<u32>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Correctness {
    pub pass: bool,
    pub mismatches: usize,
    pub checked_words: usize,
    pub max_abs: f64,
    pub max_rel: f64,
}

#[allow(clippy::too_many_arguments)] // Compact declarative matrix constructor.
fn row(
    family: &'static str,
    name: impl Into<String>,
    kernel: &'static str,
    n0: u32,
    n1: u32,
    aux: u32,
    block: u32,
    grid_groups: u32,
    output_per_op: usize,
    iterations: usize,
    kind: OutputKind,
) -> RowSpec {
    RowSpec {
        family,
        name: name.into(),
        kernel,
        second_kernel: None,
        second_n1: 0,
        second_aux: 0,
        second_output_delta: 0,
        second_block: 0,
        n0,
        n1,
        aux,
        block,
        wave_size: 32,
        scheduler_profile: SchedulerProfile::Default,
        grid_groups,
        second_grid_groups: 0,
        output_per_op,
        iterations,
        kind,
    }
}

pub fn matrix(profile: MatrixProfile, wave_policy: WavePolicy) -> Vec<RowSpec> {
    match profile {
        MatrixProfile::HipEngineF2c => hipengine_f2c_matrix(wave_policy),
        MatrixProfile::LegacyHipfire => legacy_hipfire_matrix(wave_policy),
    }
}

/// The exact family/parameter grid selected by `examples/hipengine-6409/run_matrix.py`
/// at hipEngine commit f2c3ad6, plus the separately-run eight-row dispatch grid.
///
/// These remain Hipfire-native kernels and retain this harness's RMW correctness
/// contract.  The purpose of this table is denominator and shape parity: no
/// HipEngine configuration is silently omitted from the Rust control harness.
fn hipengine_f2c_matrix(wave_policy: WavePolicy) -> Vec<RowSpec> {
    let mut rows = Vec::new();

    for count in [1usize, 50, 200, 941] {
        rows.push(row(
            "dispatch-grid",
            format!("sweep=count,count={count},grid=1"),
            "dispatch_tiny",
            1,
            0,
            0,
            256,
            1,
            1,
            count,
            OutputKind::U32,
        ));
    }
    for grid in [1u32, 128, 1024, 8192] {
        rows.push(row(
            "dispatch-grid",
            format!("sweep=grid,count=941,grid={grid}"),
            "dispatch_tiny",
            grid,
            0,
            0,
            256,
            grid,
            grid as usize,
            941,
            OutputKind::U32,
        ));
    }

    for k in [512u32, 2048] {
        for row_count in [1u32, 4] {
            for wg in [64u32, 256] {
                rows.push(row(
                    "geometry",
                    format!("k={k},rows={row_count},wg={wg},body=32"),
                    "geometry_fma",
                    k,
                    row_count,
                    32,
                    wg,
                    row_count,
                    row_count as usize,
                    10,
                    OutputKind::F32,
                ));
            }
        }
    }

    for (kernel, variant) in [
        ("reduction_lds", "lds_tree"),
        ("reduction_extra_barrier", "extra_barrier"),
        ("reduction_wave", "wave_shuffle"),
        ("reduction_multi4", "multi_accum4"),
        ("reduction_multi8", "multi_accum8"),
        ("reduction_multi16", "multi_accum16"),
    ] {
        for k in [512u32, 2048] {
            for wg in [64u32, 256] {
                rows.push(row(
                    "reduction",
                    format!("variant={variant},k={k},rows=1,wg={wg},body=32"),
                    kernel,
                    k,
                    1,
                    32,
                    wg,
                    1,
                    1,
                    10,
                    OutputKind::F32,
                ));
            }
        }
    }

    for (kernel, variant) in [
        ("memory_coalesced4", "coalesced4"),
        ("memory_strided4", "strided4"),
        ("memory_gather", "gather1"),
        ("memory_interleave4", "interleave4"),
    ] {
        for wg in [64u32, 256] {
            rows.push(row(
                "memory-waitcnt",
                format!("variant={variant},n=32768,body=64,wg={wg}"),
                kernel,
                32768,
                64,
                0,
                wg,
                32768u32.div_ceil(wg),
                32768,
                10,
                OutputKind::F32,
            ));
        }
    }

    for (kernel, variant) in [
        ("dot_q8", "q8_signed"),
        ("dot_q4", "q4_unsigned"),
        ("dot_q6", "q6_zero"),
        ("dot_scalar", "scalar_dequant"),
    ] {
        for wg in [64u32, 256] {
            rows.push(row(
                "packed-dot",
                format!("variant={variant},groups=16,n=32768,body=64,wg={wg}"),
                kernel,
                32768,
                64,
                0,
                wg,
                32768u32.div_ceil(wg),
                32768,
                10,
                OutputKind::U32,
            ));
        }
    }

    for (kernel, variant) in [
        ("vopd_independent", "independent_fma"),
        ("vopd_dependent", "dependent_fma"),
        ("vopd_mixed", "mixed_int_float"),
        ("vopd_dequant", "dequant_like"),
    ] {
        for wg in [64u32, 256] {
            rows.push(row(
                "vopd",
                format!("variant={variant},accums=4,n=65536,body=512,wg={wg}"),
                kernel,
                65536,
                0,
                512,
                wg,
                65536u32.div_ceil(wg),
                65536,
                10,
                OutputKind::F32,
            ));
        }
    }

    for row_count in [1u32, 4, 8] {
        for wg in [64u32, 256] {
            for top_k in [1u32, 8] {
                rows.push(row(
                    "sampler",
                    format!("top-k={top_k},vocab=32768,rows={row_count},wg={wg}"),
                    "sampler_topk",
                    32768,
                    row_count,
                    top_k,
                    wg,
                    row_count,
                    (row_count * top_k) as usize,
                    10,
                    OutputKind::U32,
                ));
            }
        }
    }

    for k in [8192u32, 32768] {
        for row_count in [1u32, 4] {
            for wg in [128u32, 256] {
                for splits in [2u32, 4] {
                    let mut spec = row(
                        "two-stage-reduction",
                        format!("k={k},rows={row_count},splits={splits},wg={wg},body=16"),
                        "two_stage_partial",
                        k,
                        row_count,
                        splits,
                        wg,
                        row_count * splits,
                        (row_count * (splits + 1)) as usize,
                        10,
                        OutputKind::TwoStageF32,
                    );
                    spec.second_kernel = Some("two_stage_final");
                    spec.second_grid_groups = row_count;
                    rows.push(spec);
                }
            }
        }
    }

    // The production-slice rows retain Hipfire's packed-dot implementations,
    // but enumerate HipEngine's operation and launch grid exactly.  Quantize
    // and combined rows use dedicated kernels below rather than being aliases.
    rows.push(row(
        "q4-selected-dual",
        "operation=q8_1_quantize,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=32",
        "q8_1_quantize_q4",
        2048,
        4,
        0,
        32,
        4 * (2048 / 32),
        4 * (2048 / 32) * 9,
        10,
        OutputKind::Q8Overwrite,
    ));
    for wg in [64u32, 128] {
        rows.push(row(
            "q4-selected-dual",
            format!("operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg={wg}"),
            "q4_selected_dual",
            2048,
            32 * 2 * 512,
            0,
            wg,
            32 * 512,
            32 * 2 * 512,
            10,
            OutputKind::U32,
        ));
        let scratch_words = 4 * (2048 / 32) * 9;
        let mut spec = row(
            "q4-selected-dual",
            format!("operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg={wg}"),
            "q8_1_quantize_q4",
            2048,
            4,
            0,
            32,
            4 * (2048 / 32),
            (scratch_words + 32 * 2 * 512) as usize,
            10,
            OutputKind::Q8ThenU32,
        );
        spec.second_kernel = Some("q4_selected_dual");
        spec.second_grid_groups = 32 * 512;
        spec.second_n1 = 32 * 2 * 512;
        spec.second_aux = scratch_words;
        spec.second_output_delta = scratch_words;
        spec.second_block = wg;
        rows.push(spec);
    }

    rows.push(row(
        "q6-x8-selected-down",
        "operation=q8_1_quantize,rows=8,experts=256,in=512,out=2048,wg=32",
        "q8_1_quantize_q6",
        512,
        8,
        0,
        32,
        8 * (512 / 32),
        8 * (512 / 32) * 9,
        10,
        OutputKind::Q8Overwrite,
    ));
    rows.push(row(
        "q6-x8-selected-down",
        "operation=x8_selected_dp4a_dot_prequantized,rows=8,experts=256,in=512,out=2048,wg=64",
        "q6_x8",
        512,
        8 * 2048,
        0,
        64,
        2048,
        8 * 2048,
        10,
        OutputKind::U32,
    ));
    let q6_scratch_words = 8 * (512 / 32) * 9;
    let mut q6_combined = row(
        "q6-x8-selected-down",
        "operation=x8_selected_dp4a_quantize_plus_dot,rows=8,experts=256,in=512,out=2048,wg=64",
        "q8_1_quantize_q6",
        512,
        8,
        0,
        32,
        8 * (512 / 32),
        (q6_scratch_words + 8 * 2048) as usize,
        10,
        OutputKind::Q8ThenU32,
    );
    q6_combined.second_kernel = Some("q6_x8");
    q6_combined.second_grid_groups = 2048;
    q6_combined.second_n1 = 8 * 2048;
    q6_combined.second_aux = q6_scratch_words;
    q6_combined.second_output_delta = q6_scratch_words;
    q6_combined.second_block = 64;
    rows.push(q6_combined);

    for in_features in [768u32, 2048] {
        for row_count in [1u32, 4] {
            rows.push(row(
                "dense-q8",
                format!("operation=q8_1_quantize,in={in_features},out=2048,rows={row_count},wg=32"),
                "q8_1_quantize_dense",
                in_features,
                row_count,
                0,
                32,
                row_count * (in_features / 32),
                (row_count * (in_features / 32) * 9) as usize,
                10,
                OutputKind::Q8Overwrite,
            ));
            for row_tile in [1u32, 4] {
                let kernel = if row_tile == 1 {
                    "dense_q8_single"
                } else {
                    "dense_q8"
                };
                rows.push(row(
                    "dense-q8",
                    format!("operation=q8_0_dense_dp4a_dot_prequantized,in={in_features},out=2048,rows={row_count},row_tile={row_tile},wg=32"),
                    kernel,
                    in_features,
                    row_count * 2048,
                    0,
                    32,
                    (row_count * 2048).div_ceil(row_tile),
                    (row_count * 2048) as usize,
                    10,
                    OutputKind::U32,
                ));
                let scratch_words = row_count * (in_features / 32) * 9;
                let mut combined = row(
                    "dense-q8",
                    format!("operation=q8_0_dense_dp4a_quantize_plus_dot,in={in_features},out=2048,rows={row_count},row_tile={row_tile},wg=32"),
                    "q8_1_quantize_dense",
                    in_features,
                    row_count,
                    row_tile,
                    32,
                    row_count * (in_features / 32),
                    (scratch_words + row_count * 2048) as usize,
                    10,
                    OutputKind::Q8ThenU32,
                );
                combined.second_kernel = Some(kernel);
                combined.second_grid_groups = (row_count * 2048).div_ceil(row_tile);
                combined.second_n1 = row_count * 2048;
                combined.second_aux = scratch_words;
                combined.second_output_delta = scratch_words;
                rows.push(combined);
            }
        }
    }

    apply_wave_policy(&mut rows, wave_policy, true);
    debug_assert_eq!(rows.len(), 120);
    rows
}

fn legacy_hipfire_matrix(wave_policy: WavePolicy) -> Vec<RowSpec> {
    let mut rows = Vec::new();

    for count in [1usize, 50, 200, 941] {
        rows.push(row(
            "dispatch-grid",
            format!("count={count},grid=1"),
            "dispatch_tiny",
            1,
            0,
            0,
            256,
            1,
            1,
            count,
            OutputKind::U32,
        ));
    }
    for grid in [128u32, 1024, 8192] {
        rows.push(row(
            "dispatch-grid",
            format!("count=64,grid={grid}"),
            "dispatch_tiny",
            grid,
            0,
            0,
            256,
            grid,
            grid as usize,
            64,
            OutputKind::U32,
        ));
    }

    for k in [512u32, 2048] {
        for r in [1u32, 4] {
            rows.push(row(
                "geometry",
                format!("k={k},rows={r},wg=256"),
                "geometry_fma",
                k,
                r,
                32,
                256,
                r,
                r as usize,
                64,
                OutputKind::F32,
            ));
        }
    }

    for (kernel, variant) in [("reduction_wave", "wave"), ("reduction_lds", "lds-tree")] {
        for r in [1u32, 4] {
            rows.push(row(
                "reduction",
                format!("variant={variant},k=8192,rows={r}"),
                kernel,
                8192,
                r,
                16,
                256,
                r,
                r as usize,
                64,
                OutputKind::F32,
            ));
        }
    }

    for (kernel, variant) in [
        ("memory_coalesced4", "coalesced4"),
        ("memory_gather", "gather"),
        ("memory_interleave4", "interleave4"),
    ] {
        for n in [4096u32, 32768] {
            rows.push(row(
                "memory-waitcnt",
                format!("variant={variant},n={n},body=16"),
                kernel,
                n,
                16,
                0, // fixture fills the data mask
                256,
                n.div_ceil(256),
                n as usize,
                if n > 4096 { 32 } else { 64 },
                OutputKind::F32,
            ));
        }
    }

    for (kernel, variant) in [
        ("dot_q8", "q8-signed"),
        ("dot_q4", "q4-unsigned"),
        ("dot_q6", "q6-zero"),
        ("dot_scalar", "q4-scalar"),
    ] {
        rows.push(row(
            "packed-dot",
            format!("variant={variant},n=4096,body=16"),
            kernel,
            4096,
            16,
            0,
            256,
            16,
            4096,
            64,
            OutputKind::U32,
        ));
    }

    for (kernel, variant) in [
        ("vopd_independent", "independent-fma"),
        ("vopd_dependent", "dependent-fma"),
        ("vopd_mixed", "mixed-int-float"),
        ("vopd_dequant", "dequant-like"),
    ] {
        rows.push(row(
            "vopd",
            format!("variant={variant},n=32768,body=64"),
            kernel,
            32768,
            0,
            64,
            256,
            128,
            32768,
            32,
            OutputKind::F32,
        ));
    }

    for (vocab, r) in [(32768u32, 1u32), (32768, 4), (131072, 1), (131072, 4)] {
        rows.push(row(
            "sampler",
            format!("argmax,vocab={vocab},rows={r}"),
            "sampler_argmax",
            vocab,
            r,
            0,
            256,
            r,
            r as usize,
            64,
            OutputKind::U32,
        ));
    }

    for (k, splits) in [(8192u32, 2u32), (32768, 8)] {
        let r = 4u32;
        let mut spec = row(
            "two-stage-reduction",
            format!("k={k},rows={r},splits={splits}"),
            "two_stage_partial",
            k,
            r,
            splits,
            256,
            r * splits,
            (r * (splits + 1)) as usize,
            if k > 8192 { 32 } else { 64 },
            OutputKind::TwoStageF32,
        );
        spec.second_kernel = Some("two_stage_final");
        spec.second_grid_groups = r;
        rows.push(spec);
    }

    for (m, k) in [(512u32, 768u32), (2048, 2048), (4096, 8192)] {
        rows.push(row(
            "q4-selected-dual",
            format!("m={m},k={k},tile=2"),
            "q4_selected_dual",
            k,
            m,
            0,
            32,
            m.div_ceil(2),
            m as usize,
            if k >= 8192 { 16 } else { 32 },
            OutputKind::U32,
        ));
    }

    for (m, k) in [(512u32, 2048u32), (2048, 4096), (4096, 8192)] {
        rows.push(row(
            "q6-x8-selected-down",
            format!("m={m},k={k},tile=8"),
            "q6_x8",
            k,
            m,
            0,
            32,
            m.div_ceil(8),
            m as usize,
            if k >= 8192 { 16 } else { 32 },
            OutputKind::U32,
        ));
    }

    for (m, k) in [(512u32, 768u32), (2048, 2048), (2048, 8192), (4096, 8192)] {
        rows.push(row(
            "dense-q8",
            format!("m={m},k={k},tile=4"),
            "dense_q8",
            k,
            m,
            0,
            32,
            m.div_ceil(4),
            m as usize,
            if k >= 8192 { 16 } else { 32 },
            OutputKind::U32,
        ));
    }
    apply_wave_policy(&mut rows, wave_policy, false);
    rows
}

fn apply_wave_policy(rows: &mut [RowSpec], wave_policy: WavePolicy, preserve_block: bool) {
    for spec in rows {
        let parity_requires_wave32 = preserve_block
            && matches!(
                spec.kernel,
                "dense_q8" | "dense_q8_single" | "q8_1_quantize_dense"
            );
        if uses_wave64(wave_policy, spec.kernel) && !parity_requires_wave32 {
            spec.wave_size = 64;
            if !preserve_block && matches!(spec.kernel, "q4_selected_dual" | "q6_x8" | "dense_q8") {
                spec.block = 64;
            }
        }
        if wave_policy == WavePolicy::RadiowaveTuned && spec.kernel == "dispatch_tiny" {
            // This workload has one live lane per workgroup.  A 32-thread HIP
            // workgroup preserves the dispatch count and oracle while avoiding
            // deliberately idle lanes. The Vulkan pipeline still records the
            // row's native specialization separately.
            spec.block = 32;
        }
    }
}

fn uses_wave64(policy: WavePolicy, kernel: &str) -> bool {
    let targeted = matches!(
        kernel,
        "q4_selected_dual" | "q6_x8" | "dense_q8" | "dense_q8_single" | "vopd_dependent"
    );
    let radiowave_tuned = targeted
        || matches!(
            kernel,
            "memory_interleave4" | "vopd_independent" | "vopd_mixed" | "vopd_dequant"
        );
    let blanket = matches!(
        kernel,
        "dispatch_tiny"
            | "memory_coalesced4"
            | "memory_gather"
            | "memory_interleave4"
            | "memory_strided4"
            | "dot_q8"
            | "dot_q4"
            | "dot_q6"
            | "vopd_independent"
            | "vopd_dependent"
            | "vopd_mixed"
            | "vopd_dequant"
            | "sampler_argmax"
            | "sampler_topk"
            | "two_stage_partial"
            | "q4_selected_dual"
            | "q6_x8"
            | "dense_q8"
            | "dense_q8_single"
    );
    match policy {
        WavePolicy::AllWave32 => false,
        WavePolicy::TargetedWave64 => targeted,
        WavePolicy::RadiowaveTuned => radiowave_tuned,
        WavePolicy::BlanketWave64 => blanket,
    }
}

fn hash_u32(mut v: u32) -> u32 {
    v ^= v >> 16;
    v = v.wrapping_mul(0x7feb352d);
    v ^= v >> 15;
    v = v.wrapping_mul(0x846ca68b);
    v ^ (v >> 16)
}

fn f32_word(v: f32) -> u32 {
    v.to_bits()
}

fn fixture_geometry(spec: &RowSpec) -> Fixture {
    let a = (0..spec.n0)
        .map(|i| f32_word(((i * 17 + 5) % 29) as f32 * 0.03125 - 0.4375))
        .collect::<Vec<_>>();
    let b = (0..spec.n1)
        .flat_map(|r| {
            (0..spec.n0)
                .map(move |i| f32_word(((r * 11 + i * 13 + 3) % 31) as f32 * 0.015625 - 0.234375))
        })
        .collect::<Vec<_>>();
    let mut one = vec![0.0f32; spec.n1 as usize];
    for r in 0..spec.n1 as usize {
        let mut sum = 0.0f32;
        for repeat in 0..spec.aux as usize {
            for i in 0..spec.n0 as usize {
                let j = (i + repeat) % spec.n0 as usize;
                sum =
                    f32::from_bits(a[j]).mul_add(f32::from_bits(b[r * spec.n0 as usize + j]), sum);
            }
        }
        one[r] = sum;
    }
    Fixture {
        a,
        b,
        one_op: one.into_iter().map(f32_word).collect(),
    }
}

fn packed_word(seed: u32, mask: u32) -> u32 {
    let mut word = 0u32;
    for lane in 0..4 {
        word |= (hash_u32(seed.wrapping_add(lane * 0x9e3779b9)) & mask) << (lane * 8);
    }
    word
}

fn unpack_i8(word: u32, lane: usize) -> i32 {
    ((word >> (lane * 8)) as u8 as i8) as i32
}

fn unpack_u8(word: u32, lane: usize) -> i32 {
    ((word >> (lane * 8)) & 255) as i32
}

fn dot_word_signed(a: u32, b: u32) -> i32 {
    (0..4)
        .map(|lane| unpack_i8(a, lane) * unpack_i8(b, lane))
        .sum()
}

fn dot_word_unsigned(a: u32, b: u32) -> i32 {
    (0..4)
        .map(|lane| unpack_u8(a, lane) * unpack_i8(b, lane))
        .sum()
}

fn fixture_dot(spec: &mut RowSpec) -> Fixture {
    let len = (spec.n0 as usize * spec.n1 as usize * 16).next_power_of_two();
    spec.aux = (len - 1) as u32;
    let weight_mask = if spec.kernel == "dot_q8" {
        255
    } else if spec.kernel == "dot_q6" {
        63
    } else {
        15
    };
    let a = (0..len)
        .map(|i| packed_word(i as u32 * 17 + 3, weight_mask))
        .collect::<Vec<_>>();
    let b = (0..len)
        .map(|i| packed_word(i as u32 * 29 + 7, 255))
        .collect::<Vec<_>>();
    let mut one = vec![0u32; spec.n0 as usize];
    for (idx, output) in one.iter_mut().enumerate() {
        let mut sum = 0i32;
        for iter in 0..spec.n1 as usize {
            let base = ((iter * spec.n0 as usize + idx) * 16) & spec.aux as usize;
            for g in 0..16 {
                let w = a[(base + g) & spec.aux as usize];
                let x = b[(base + g) & spec.aux as usize];
                sum = sum.wrapping_add(match spec.kernel {
                    "dot_q8" => dot_word_signed(w, x),
                    "dot_q4" => dot_word_unsigned(w, x),
                    "dot_q6" => {
                        dot_word_unsigned(w, x) - 32 * (0..4).map(|l| unpack_i8(x, l)).sum::<i32>()
                    }
                    _ => (0..4)
                        .map(|l| (unpack_u8(w, l) - 8) * unpack_i8(x, l))
                        .sum(),
                });
            }
        }
        *output = sum as u32;
    }
    Fixture { a, b, one_op: one }
}

fn fixture_memory(spec: &mut RowSpec) -> Fixture {
    let multiplier = if spec.kernel.starts_with("memory_gather") {
        1
    } else {
        4
    };
    let len = (spec.n0 as usize * spec.n1 as usize * multiplier).next_power_of_two();
    spec.aux = (len - 1) as u32;
    let a = (0..len)
        .map(|i| f32_word(((hash_u32(i as u32) & 1023) as f32 - 511.0) * 0.0009765625))
        .collect::<Vec<_>>();
    let b = (0..spec.n0 as usize * spec.n1 as usize)
        .map(|i| hash_u32(i as u32 * 31 + 9) & spec.aux)
        .collect::<Vec<_>>();
    let mut one = vec![0.0f32; spec.n0 as usize];
    for (idx, output) in one.iter_mut().enumerate() {
        let mut sum = 0.0f32;
        let mut state = (idx as u32)
            .wrapping_mul(747796405)
            .wrapping_add(2891336453);
        for iter in 0..spec.n1 as usize {
            if spec.kernel.starts_with("memory_gather") {
                let address = b[iter * spec.n0 as usize + idx] as usize & spec.aux as usize;
                sum = f32::from_bits(a[address]).mul_add(1.000001, sum);
            } else if spec.kernel == "memory_strided4" {
                let address = (iter * spec.n0 as usize * 4 + idx * 4) & spec.aux as usize;
                sum = f32::from_bits(a[address]).mul_add(1.000001, sum);
            } else {
                let base = (iter * spec.n0 as usize * 4 + idx * 4) & spec.aux as usize;
                for j in 0..4usize {
                    let mut v = f32::from_bits(a[(base + j) & spec.aux as usize]);
                    let scale;
                    if spec.kernel.starts_with("memory_interleave4") {
                        state = hash_u32(state.wrapping_add(iter as u32).wrapping_add(j as u32));
                        v += (state & 255) as f32 * 0.0000001;
                        scale = 0.999999 + j as f32 * 0.000001;
                    } else {
                        scale = 1.000001 + j as f32 * 0.000001;
                    }
                    sum = v.mul_add(scale, sum);
                }
            }
        }
        *output = sum;
    }
    Fixture {
        a,
        b,
        one_op: one.into_iter().map(f32_word).collect(),
    }
}

fn vopd_cpu(idx: u32, mode: u32, iters: u32) -> f32 {
    let mut v0 = 0.25 + ((idx.wrapping_mul(747796405)) & 255) as f32 * 0.0009765625;
    let mut v1 = 0.25 + ((idx.wrapping_mul(747796405) ^ 2891336453) & 255) as f32 * 0.0009765625;
    let mut v2 = v0 + 0.03125;
    let mut v3 = v1 + 0.046875;
    let mut u0 = idx.wrapping_mul(747796405).wrapping_add(2891336453);
    let mut u1 = idx ^ 0xa5a5a5a5;
    let bias = |lane: u32, iter: u32| {
        let bits = idx
            .wrapping_mul(1664525)
            .wrapping_add(iter.wrapping_mul(1013904223))
            .wrapping_add(lane.wrapping_mul(2246822519));
        ((bits >> 24) & 31) as f32 * 0.000001 + 0.000003
    };
    for iter in 0..iters {
        match mode {
            0 => {
                v0 = v0.mul_add(1.000001, bias(0, iter));
                v1 = v1.mul_add(0.999999, bias(1, iter));
                v2 = v2.mul_add(1.000002, bias(2, iter));
                v3 = v3.mul_add(0.999998, bias(3, iter));
            }
            1 => {
                v0 = v0.mul_add(1.000001, bias(0, iter));
                v0 = v0.mul_add(0.999999, bias(1, iter));
                v0 = v0.mul_add(1.000002, bias(2, iter));
                v0 = v0.mul_add(0.999998, bias(3, iter));
            }
            2 => {
                u0 = hash_u32(u0.wrapping_add(iter));
                u1 = hash_u32(u1.wrapping_add(iter).wrapping_add(1));
                v0 = v0.mul_add(1.000001, (u0 & 31) as f32 * 0.000001);
                v1 = v1.mul_add(0.999999, (u1 & 31) as f32 * 0.000001);
                v2 = v2.mul_add(1.000002, ((u0 >> 7) & 31) as f32 * 0.000001);
                v3 = v3.mul_add(0.999998, ((u1 >> 11) & 31) as f32 * 0.000001);
            }
            _ => {
                u0 = hash_u32(u0.wrapping_add(iter));
                u1 = hash_u32(u1.wrapping_add(iter).wrapping_add(1));
                let q0 = ((u0 >> (iter & 15)) & 255) as i32 - 128;
                let q1 = ((u1 >> ((iter + 3) & 15)) & 255) as i32 - 128;
                v0 = (q0 as f32 * 0.0078125).mul_add(0.03125, v0);
                v1 = (q1 as f32 * 0.0078125).mul_add(0.03125, v1);
            }
        }
    }
    v0 + v1 + v2 + v3
}

fn bf16_bits(value: f32) -> u16 {
    let bits = value.to_bits();
    let lsb = (bits >> 16) & 1;
    ((bits.wrapping_add(0x7fff + lsb)) >> 16) as u16
}

fn bf16_value(bits: u16) -> f32 {
    f32::from_bits((bits as u32) << 16)
}

fn quant_source(rows: usize, in_features: usize) -> (Vec<u32>, Vec<Vec<f32>>) {
    let mut packed = vec![0u32; (rows * in_features).div_ceil(2)];
    let mut values = vec![vec![0.0f32; in_features]; rows];
    for (row, row_values) in values.iter_mut().enumerate() {
        for (k, value) in row_values.iter_mut().enumerate() {
            let hash = hash_u32(
                (row as u32)
                    .wrapping_mul(1315423911)
                    .wrapping_add((k as u32).wrapping_mul(2654435761))
                    .wrapping_add(0x9e3779b9),
            );
            // Keep the Q8_1 oracle exact across LLVM and SPIR-V: every source
            // value is an exactly representable multiple of 1/128, and every
            // 32-value block contains both extrema. The resulting scale is
            // exactly 1/128 and integer rounding is unambiguous.
            let lane = k & 31;
            let quant = if lane == 30 {
                -127
            } else if lane == 31 {
                127
            } else {
                (hash % 255) as i32 - 127
            };
            let raw = quant as f32 * (1.0 / 128.0);
            let bits = bf16_bits(raw);
            *value = bf16_value(bits);
            let element = row * in_features + k;
            packed[element >> 1] |= (bits as u32) << ((element & 1) * 16);
        }
    }
    (packed, values)
}

fn quantize_q8_1(values: &[Vec<f32>], in_features: usize) -> Vec<u32> {
    let blocks = in_features.div_ceil(32);
    let mut out = vec![0u32; values.len() * blocks * 9];
    for (row, row_values) in values.iter().enumerate() {
        for block in 0..blocks {
            let mut lanes = [0.0f32; 32];
            for lane in 0..32 {
                let k = block * 32 + lane;
                if k < in_features {
                    lanes[lane] = row_values[k];
                }
            }
            let mut sums = lanes;
            let mut maxima = lanes.map(f32::abs);
            for offset in [16usize, 8, 4, 2, 1] {
                for lane in 0..offset {
                    sums[lane] += sums[lane + offset];
                    maxima[lane] = maxima[lane].max(maxima[lane + offset]);
                }
            }
            let d = if maxima[0] == 0.0 {
                0.0
            } else {
                maxima[0] * (1.0 / 127.0)
            };
            let base = (row * blocks + block) * 9;
            out[base] = f16::from_f32(d).to_bits() as u32
                | ((f16::from_f32(sums[0]).to_bits() as u32) << 16);
            for (lane, &value) in lanes.iter().enumerate() {
                let q = if d == 0.0 {
                    0
                } else {
                    (value / d).round().clamp(-128.0, 127.0) as i32
                };
                out[base + 1 + lane / 4] |= ((q as u32) & 255) << ((lane & 3) * 8);
            }
        }
    }
    out
}

fn q8_packed_word(q8: &[u32], row: usize, blocks: usize, group: usize) -> u32 {
    q8[(row * blocks + group / 8) * 9 + 1 + group % 8]
}

fn fixture_quant(spec: &RowSpec) -> Fixture {
    let source_rows = match spec.family {
        "q4-selected-dual" => 4usize,
        "q6-x8-selected-down" => 8usize,
        "dense-q8" => {
            if spec.kernel == "q8_1_quantize_dense" {
                spec.n1 as usize
            } else {
                (spec.n1 as usize / 2048).max(1)
            }
        }
        _ => 1,
    };
    let in_features = spec.n0 as usize;
    let (bf16_input, source_values) = quant_source(source_rows, in_features);
    let q8 = quantize_q8_1(&source_values, in_features);
    let output_count = if spec.second_kernel.is_some() {
        spec.second_n1 as usize
    } else if spec.kernel.starts_with("q8_1_quantize") {
        0
    } else {
        spec.n1 as usize
    };
    let groups = in_features / 4;
    let mask = match spec.family {
        "q4-selected-dual" => 15,
        "q6-x8-selected-down" => 63,
        _ => 255,
    };
    let a = if output_count == 0 {
        vec![0]
    } else {
        (0..output_count * groups)
            .map(|i| packed_word(i as u32 * 37 + 11, mask))
            .collect::<Vec<_>>()
    };
    let blocks = in_features.div_ceil(32);
    let mut dot = vec![0u32; output_count];
    for (output, result) in dot.iter_mut().enumerate() {
        let x_row = match spec.family {
            "q4-selected-dual" => (output / 1024) & 3,
            "q6-x8-selected-down" | "dense-q8" => output / 2048,
            _ => 0,
        };
        let mut sum = 0i32;
        for group in 0..groups {
            let weight = a[output * groups + group];
            let activation = q8_packed_word(&q8, x_row, blocks, group);
            sum = sum.wrapping_add(match spec.family {
                "q4-selected-dual" => {
                    dot_word_unsigned(weight, activation)
                        - 8 * (0..4).map(|lane| unpack_i8(activation, lane)).sum::<i32>()
                }
                "q6-x8-selected-down" => {
                    dot_word_unsigned(weight, activation)
                        - 32 * (0..4).map(|lane| unpack_i8(activation, lane)).sum::<i32>()
                }
                _ => dot_word_signed(weight, activation),
            });
        }
        *result = sum as u32;
    }
    let quant_only = output_count == 0;
    let combined = spec.second_kernel.is_some();
    Fixture {
        a,
        b: if quant_only || combined {
            bf16_input
        } else {
            q8.clone()
        },
        one_op: if quant_only {
            q8
        } else if combined {
            q8.into_iter().chain(dot).collect()
        } else {
            dot
        },
    }
}

pub fn fixture(spec: &mut RowSpec) -> Fixture {
    match spec.family {
        "dispatch-grid" => Fixture {
            a: vec![0],
            b: vec![0],
            one_op: vec![1; spec.output_per_op],
        },
        "geometry" | "reduction" => fixture_geometry(spec),
        "memory-waitcnt" => fixture_memory(spec),
        "packed-dot" => fixture_dot(spec),
        "vopd" => {
            let mode = match spec.kernel {
                "vopd_independent" => 0,
                "vopd_dependent" => 1,
                "vopd_mixed" | "vopd_mixed_pair" => 2,
                _ => 3,
            };
            Fixture {
                a: vec![0],
                b: vec![0],
                one_op: (0..spec.n0)
                    .map(|i| f32_word(vopd_cpu(i, mode, spec.aux)))
                    .collect(),
            }
        }
        "sampler" => {
            let mut a = vec![0u32; spec.n0 as usize * spec.n1 as usize];
            let top_k = if spec.kernel == "sampler_topk" {
                spec.aux as usize
            } else {
                1
            };
            let mut one = vec![0u32; spec.n1 as usize * top_k];
            for r in 0..spec.n1 as usize {
                let peak =
                    hash_u32((r as u32).wrapping_mul(747796405).wrapping_add(2891336453)) % spec.n0;
                for c in 0..spec.n0 as usize {
                    let v = if c == peak as usize {
                        64.0 + r as f32 * 0.25
                    } else {
                        ((hash_u32(
                            (r as u32)
                                .wrapping_mul(1664525)
                                .wrapping_add((c as u32).wrapping_mul(1013904223)),
                        ) & 0xffff) as i32
                            - 32768) as f32
                            * (1.0 / 32768.0)
                    };
                    a[r * spec.n0 as usize + c] = f32_word(v);
                }
                let mut order = (0..spec.n0 as usize).collect::<Vec<_>>();
                order.select_nth_unstable_by(top_k - 1, |&lhs, &rhs| {
                    let left = f32::from_bits(a[r * spec.n0 as usize + lhs]);
                    let right = f32::from_bits(a[r * spec.n0 as usize + rhs]);
                    right.total_cmp(&left).then_with(|| lhs.cmp(&rhs))
                });
                order[..top_k].sort_unstable_by(|&lhs, &rhs| {
                    let left = f32::from_bits(a[r * spec.n0 as usize + lhs]);
                    let right = f32::from_bits(a[r * spec.n0 as usize + rhs]);
                    right.total_cmp(&left).then_with(|| lhs.cmp(&rhs))
                });
                for (rank, &index) in order[..top_k].iter().enumerate() {
                    one[r * top_k + rank] = index as u32 + 1;
                }
            }
            Fixture {
                a,
                b: vec![0],
                one_op: one,
            }
        }
        "two-stage-reduction" => {
            let base = fixture_geometry(spec);
            let splits = spec.aux as usize;
            let rows = spec.n1 as usize;
            let mut one = vec![f32_word(0.0); rows * (splits + 1)];
            for r in 0..rows {
                let mut total = 0.0f32;
                for s in 0..splits {
                    let begin = spec.n0 as usize * s / splits;
                    let end = spec.n0 as usize * (s + 1) / splits;
                    let mut part = 0.0f32;
                    for i in begin..end {
                        part = f32::from_bits(base.a[i])
                            .mul_add(f32::from_bits(base.b[r * spec.n0 as usize + i]), part);
                    }
                    one[r * splits + s] = f32_word(part);
                    total += part;
                }
                one[rows * splits + r] = f32_word(total);
            }
            Fixture {
                a: base.a,
                b: base.b,
                one_op: one,
            }
        }
        "q4-selected-dual" | "q6-x8-selected-down" | "dense-q8" => fixture_quant(spec),
        _ => unreachable!("unknown family {}", spec.family),
    }
}

pub fn expected(spec: &RowSpec, fixture: &Fixture, mode: TimingMode) -> Vec<u32> {
    if mode == TimingMode::SingleKernelAggressive {
        return fixture.one_op.clone();
    }
    if mode == TimingMode::IndependentThroughput {
        return (0..spec.iterations)
            .flat_map(|_| fixture.one_op.iter().copied())
            .collect();
    }
    if spec.kind == OutputKind::TwoStageF32 {
        let rows = spec.n1 as usize;
        let splits = spec.aux as usize;
        let mut values = vec![0.0f32; spec.output_per_op];
        for _ in 0..spec.iterations {
            for r in 0..rows {
                for s in 0..splits {
                    values[r * splits + s] += f32::from_bits(fixture.one_op[r * splits + s]);
                }
                let mut total = 0.0f32;
                for s in 0..splits {
                    total += values[r * splits + s];
                }
                values[rows * splits + r] += total;
            }
        }
        return values.into_iter().map(f32_word).collect();
    }
    match spec.kind {
        OutputKind::U32 => fixture
            .one_op
            .iter()
            .map(|&v| v.wrapping_mul(spec.iterations as u32))
            .collect(),
        OutputKind::F32 => fixture
            .one_op
            .iter()
            .map(|&bits| {
                let base = f32::from_bits(bits);
                let mut value = 0.0f32;
                for _ in 0..spec.iterations {
                    value += base;
                }
                f32_word(value)
            })
            .collect(),
        OutputKind::TwoStageF32 => unreachable!(),
        OutputKind::Q8Overwrite => fixture.one_op.clone(),
        OutputKind::Q8ThenU32 => {
            let split = spec.second_output_delta as usize;
            fixture.one_op[..split]
                .iter()
                .copied()
                .chain(
                    fixture.one_op[split..]
                        .iter()
                        .map(|&value| value.wrapping_mul(spec.iterations as u32)),
                )
                .collect()
        }
    }
}

pub fn validate(
    spec: &RowSpec,
    fixture: &Fixture,
    mode: TimingMode,
    actual: &[u32],
) -> Correctness {
    let expected = expected(spec, fixture, mode);
    let mut mismatches = 0usize;
    let mut max_abs = 0.0f64;
    let mut max_rel = 0.0f64;
    for (got, want) in actual.iter().zip(&expected) {
        match spec.kind {
            OutputKind::U32 | OutputKind::Q8Overwrite | OutputKind::Q8ThenU32 => {
                if got != want {
                    mismatches += 1;
                }
            }
            OutputKind::F32 | OutputKind::TwoStageF32 => {
                let g = f32::from_bits(*got) as f64;
                let w = f32::from_bits(*want) as f64;
                let abs = (g - w).abs();
                let rel = abs / w.abs().max(1.0e-12);
                max_abs = max_abs.max(abs);
                max_rel = max_rel.max(rel);
                let tolerance = 0.01 + w.abs() * 0.002;
                if !g.is_finite() || abs > tolerance {
                    mismatches += 1;
                }
            }
        }
    }
    mismatches += actual.len().abs_diff(expected.len());
    Correctness {
        pass: mismatches == 0,
        mismatches,
        checked_words: expected.len(),
        max_abs,
        max_rel,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn legacy_matrix_retains_45_unique_shapes_and_133_supported_mode_rows() {
        let rows = matrix(MatrixProfile::LegacyHipfire, WavePolicy::BlanketWave64);
        assert_eq!(rows.len(), 45);
        let keys = TimingMode::ALL
            .into_iter()
            .flat_map(|mode| {
                rows.iter()
                    .filter(move |row| row.supports_mode(mode))
                    .map(move |row| row.key(mode))
            })
            .collect::<HashSet<_>>();
        assert_eq!(keys.len(), 133);
        assert_eq!(rows.iter().filter(|row| row.wave_size == 64).count(), 36);
        assert_eq!(
            matrix(MatrixProfile::LegacyHipfire, WavePolicy::TargetedWave64)
                .iter()
                .filter(|row| row.wave_size == 64)
                .count(),
            11
        );
        assert_eq!(
            matrix(MatrixProfile::LegacyHipfire, WavePolicy::RadiowaveTuned)
                .iter()
                .filter(|row| row.wave_size == 64)
                .count(),
            16
        );
        assert!(
            matrix(MatrixProfile::LegacyHipfire, WavePolicy::RadiowaveTuned)
                .iter()
                .filter(|row| row.kernel == "dispatch_tiny")
                .all(|row| row.block == 32)
        );
        assert!(matrix(MatrixProfile::LegacyHipfire, WavePolicy::AllWave32)
            .iter()
            .all(|row| row.wave_size == 32 && row.block != 64));
    }

    #[test]
    fn hipengine_f2c_matrix_is_120_shapes_and_240_comparable_rows() {
        let rows = matrix(MatrixProfile::HipEngineF2c, WavePolicy::RadiowaveTuned);
        assert_eq!(rows.len(), 120);
        let keys = TimingMode::HIPENGINE_COMPARABLE
            .into_iter()
            .flat_map(|mode| rows.iter().map(move |row| row.key(mode)))
            .collect::<HashSet<_>>();
        assert_eq!(keys.len(), 240);
        let family_counts = rows
            .iter()
            .fold(std::collections::BTreeMap::new(), |mut acc, row| {
                *acc.entry(row.family).or_insert(0usize) += 1;
                acc
            });
        assert_eq!(family_counts["dispatch-grid"], 8);
        assert_eq!(family_counts["geometry"], 8);
        assert_eq!(family_counts["reduction"], 24);
        assert_eq!(family_counts["memory-waitcnt"], 8);
        assert_eq!(family_counts["packed-dot"], 8);
        assert_eq!(family_counts["vopd"], 8);
        assert_eq!(family_counts["sampler"], 12);
        assert_eq!(family_counts["two-stage-reduction"], 16);
        assert_eq!(family_counts["q4-selected-dual"], 5);
        assert_eq!(family_counts["q6-x8-selected-down"], 3);
        assert_eq!(family_counts["dense-q8"], 20);
        assert!(rows
            .iter()
            .filter(|row| row.family == "dense-q8"
                && row.second_kernel.is_none()
                && row.kernel.starts_with("dense_q8"))
            .all(|row| row.aux == 0));
    }

    #[test]
    fn dispatch_oracles_cover_serial_rmw_and_independent_slices() {
        let mut row = matrix(MatrixProfile::LegacyHipfire, WavePolicy::BlanketWave64)
            .into_iter()
            .find(|row| row.family == "dispatch-grid" && row.iterations == 50)
            .unwrap();
        let input = fixture(&mut row);
        assert_eq!(expected(&row, &input, TimingMode::SerialLatency), vec![50]);
        assert_eq!(
            expected(&row, &input, TimingMode::IndependentThroughput),
            vec![1; 50]
        );
        assert_eq!(
            expected(&row, &input, TimingMode::SingleKernelAggressive),
            vec![1]
        );
    }
}
