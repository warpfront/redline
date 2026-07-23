// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! Exact-artifact parity probe for the pinned HipEngine #6409 VOPD rows.
//!
//! This deliberately does not rebuild the kernel. It loads the
//! Radiowave-certified code object emitted by `examples/hipengine-6409`, then
//! records the same dispatch sequence through the Rust Redline API used by the
//! Hipfire microbench. Three replay policies isolate code/tape parity from the
//! cache state entering each sample.

use anyhow::{bail, Context, Result};
use hip_bridge::{DeviceBuffer, HipRuntime};
use radiowave::{CodeObjectCertification, MutableReadCache};
use redline_dispatch::aql::{
    load_symbols, Executable, Gfx12Pm4CommandBuffer, GpuDevice, GpuSelector, KernargBuffer,
    KernargPool, LaunchGeometry, Runtime, SingleQueuePm4Ib,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const N: u32 = 65_536;
const BODY_ITERS: u32 = 512;
const DEFAULT_REPS: usize = 10;
const DEFAULT_WARMUPS: usize = 3;
const DEFAULT_SAMPLES: usize = 9;
const DEFAULT_PREHEAT: usize = 2_000;
const SYMBOL: &str = "_ZN12_GLOBAL__N_117vopd_sweep_kernelEPfjj.kd";
const VARIANTS: [&str; 4] = [
    "independent_fma",
    "dependent_fma",
    "mixed_int_float",
    "dequant_like",
];
const BLOCKS: [u32; 2] = [64, 256];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum TimingMode {
    SerialLatency,
    IndependentThroughput,
}

impl TimingMode {
    const ALL: [Self; 2] = [Self::SerialLatency, Self::IndependentThroughput];

    fn as_str(self) -> &'static str {
        match self {
            Self::SerialLatency => "serial_latency",
            Self::IndependentThroughput => "independent_throughput",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ReplayPolicy {
    /// Match the HipEngine C-ABI adapter: distinct kernarg allocation for each
    /// recorded node, one output reset before all samples, and no ownership
    /// acquire in the retained tape.
    HipengineCompatible,
    /// Preserve HipEngine's hot sample state while reusing the serial kernarg
    /// address so stateful PM4 can elide redundant user-data writes.
    HipfireReuseHot,
    /// Reset before every replay and complete a separate system ownership
    /// acquire before entering the timed retained tape.
    HipfireSafe,
}

impl ReplayPolicy {
    const ALL: [Self; 3] = [
        Self::HipengineCompatible,
        Self::HipfireReuseHot,
        Self::HipfireSafe,
    ];

    fn as_str(self) -> &'static str {
        match self {
            Self::HipengineCompatible => "hipengine_compatible",
            Self::HipfireReuseHot => "hipfire_reuse_hot",
            Self::HipfireSafe => "hipfire_safe",
        }
    }

    fn reuse_serial_kernarg(self) -> bool {
        self != Self::HipengineCompatible
    }

    fn reset_each_sample(self) -> bool {
        self == Self::HipfireSafe
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "hipengine" | "hipengine_compatible" => Some(Self::HipengineCompatible),
            "reuse" | "hipfire_reuse_hot" => Some(Self::HipfireReuseHot),
            "safe" | "hipfire_safe" => Some(Self::HipfireSafe),
            _ => None,
        }
    }
}

#[derive(Debug)]
struct Args {
    artifact_root: PathBuf,
    out: Option<PathBuf>,
    reps: usize,
    warmups: usize,
    samples: usize,
    preheat: usize,
    filter: Option<String>,
    policy: Option<ReplayPolicy>,
}

#[derive(Serialize)]
struct Report {
    kind: &'static str,
    artifact_root: String,
    code_object_policy: &'static str,
    rows: Vec<Row>,
}

#[derive(Serialize)]
struct Row {
    variant: String,
    block: u32,
    timing_mode: TimingMode,
    replay_policy: ReplayPolicy,
    reps: usize,
    samples: usize,
    median_us: f64,
    min_us: f64,
    max_us: f64,
    command_dwords: u32,
    command_sha256: String,
    kernarg_policy: &'static str,
    ownership_policy: &'static str,
    mutable_read_cache: MutableReadCache,
    correctness_pass: bool,
    max_abs: f64,
    max_rel: f64,
    code_object: String,
    code_object_sha256: String,
}

struct ProbeRuntime {
    hip: HipRuntime,
    _runtime: Runtime,
    device: GpuDevice,
    pool: KernargPool,
}

struct Tape {
    ib: SingleQueuePm4Ib,
    ownership: Option<SingleQueuePm4Ib>,
    _executable: Executable,
    _kernargs: Vec<KernargBuffer>,
    command_dwords: u32,
    command_sha256: String,
    mutable_read_cache: MutableReadCache,
}

fn main() -> Result<()> {
    let args = parse_args()?;
    let probe = ProbeRuntime::new()?;
    if args.preheat > 0 {
        if let Some((variant, block, timing_mode)) = first_selected_case(&args) {
            eprintln!(
                "[hipengine-exact] preheat {}/{variant}/wg{block} replays={}",
                timing_mode.as_str(),
                args.preheat
            );
            let preheat_args = Args {
                artifact_root: args.artifact_root.clone(),
                out: None,
                reps: args.reps,
                warmups: args.preheat,
                samples: 1,
                preheat: 0,
                filter: None,
                policy: Some(ReplayPolicy::HipfireSafe),
            };
            let _ = probe.measure(
                &preheat_args,
                variant,
                block,
                timing_mode,
                ReplayPolicy::HipfireSafe,
            )?;
        }
    }
    let mut rows = Vec::new();
    for variant in VARIANTS {
        for block in BLOCKS {
            for timing_mode in TimingMode::ALL {
                let key = format!("{}/{variant}/wg{block}", timing_mode.as_str());
                if args
                    .filter
                    .as_deref()
                    .is_some_and(|filter| !key.contains(filter))
                {
                    continue;
                }
                for replay_policy in ReplayPolicy::ALL {
                    if args
                        .policy
                        .is_some_and(|selected| selected != replay_policy)
                    {
                        continue;
                    }
                    eprintln!("[hipengine-exact] {key} {}", replay_policy.as_str());
                    rows.push(probe.measure(&args, variant, block, timing_mode, replay_policy)?);
                }
            }
        }
    }
    let report = Report {
        kind: "hipfire_hipengine_exact_vopd_parity",
        artifact_root: args.artifact_root.display().to_string(),
        code_object_policy:
            "load exact HipEngine Radiowave-certified .redline.co without recompilation",
        rows,
    };
    let encoded = serde_json::to_string_pretty(&report)? + "\n";
    if let Some(path) = &args.out {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, &encoded)?;
    }
    print!("{encoded}");
    Ok(())
}

impl ProbeRuntime {
    fn new() -> Result<Self> {
        let hip = HipRuntime::load().context("load Hipfire HIP bridge")?;
        hip.set_device(0)?;
        let runtime = Runtime::initialize(load_symbols()?).context("initialize public ROCr")?;
        let device = runtime.select_gpu(GpuSelector::Ordinal(0))?;
        let pool = KernargPool::discover(&device)?;
        Ok(Self {
            hip,
            _runtime: runtime,
            device,
            pool,
        })
    }

    fn measure(
        &self,
        args: &Args,
        variant: &str,
        block: u32,
        timing_mode: TimingMode,
        replay_policy: ReplayPolicy,
    ) -> Result<Row> {
        let directory = args
            .artifact_root
            .join("vopd/hip")
            .join(format!("{variant}_a4"))
            .join(format!("wg{block}"));
        let code_path = directory.join("hip_vopd_sweep.redline.co");
        let manifest_path = directory.join("hip_vopd_sweep.redline.radiowave.json");
        let code = fs::read(&code_path)
            .with_context(|| format!("read exact HipEngine artifact {}", code_path.display()))?;
        let manifest = fs::read_to_string(&manifest_path)
            .with_context(|| format!("read certification {}", manifest_path.display()))?;
        let certification = CodeObjectCertification::from_json(&code, &manifest)
            .context("verify exact HipEngine artifact certification")?;
        let mutable_read_cache = certification.mutable_read_cache(SYMBOL);
        let code_sha256 = hex_digest(&code);

        let output_slots = if timing_mode == TimingMode::IndependentThroughput {
            args.reps
        } else {
            1
        };
        let out = self.hip.malloc(output_slots * N as usize * 4)?;
        self.reset(&out)?;
        let mut tape = self.build_tape(
            Arc::<[u8]>::from(code),
            &out,
            block,
            timing_mode,
            replay_policy,
            args.reps,
            mutable_read_cache,
        )?;

        for _ in 0..args.warmups {
            if replay_policy.reset_each_sample() {
                self.reset_and_acquire(&out, &mut tape)?;
            }
            unsafe { tape.ib.replay_and_wait_profiled()? };
        }
        if !replay_policy.reset_each_sample() {
            // HipEngine resets once between warmup and the complete sample set.
            self.reset(&out)?;
        }
        let mut samples = Vec::with_capacity(args.samples);
        for _ in 0..args.samples {
            if replay_policy.reset_each_sample() {
                self.reset_and_acquire(&out, &mut tape)?;
            }
            let timing = unsafe { tape.ib.replay_and_wait_profiled()? };
            samples.push(timing.span_microseconds() / args.reps as f64);
        }

        // Correctness is checked in a fresh, safe ownership epoch so the
        // hot-state timing policy cannot mask a bad launch ABI.
        self.reset_and_acquire(&out, &mut tape)?;
        unsafe { tape.ib.replay_and_wait_profiled()? };
        let (correctness_pass, max_abs, max_rel) =
            self.check_output(&out, variant, timing_mode, args.reps)?;
        let command_dwords = tape.command_dwords;
        let command_sha256 = tape.command_sha256.clone();
        let mutable_read_cache = tape.mutable_read_cache;
        // Retained kernargs encode `out`; destroy the tape before releasing the
        // allocation even though the final replay has already completed.
        drop(tape);
        self.hip.free(out)?;

        samples.sort_by(f64::total_cmp);
        let median_us = percentile(&samples, 0.5);
        Ok(Row {
            variant: variant.to_owned(),
            block,
            timing_mode,
            replay_policy,
            reps: args.reps,
            samples: args.samples,
            median_us,
            min_us: samples[0],
            max_us: *samples.last().unwrap(),
            command_dwords,
            command_sha256,
            kernarg_policy: if replay_policy.reuse_serial_kernarg()
                && timing_mode == TimingMode::SerialLatency
            {
                "reuse_identical_serial_kernarg"
            } else {
                "one_kernarg_per_dispatch"
            },
            ownership_policy: if replay_policy.reset_each_sample() {
                "hip_reset_then_separate_completed_gfx12_system_acquire"
            } else {
                "one_hip_reset_before_sample_set_no_redline_acquire"
            },
            mutable_read_cache,
            correctness_pass,
            max_abs,
            max_rel,
            code_object: code_path.display().to_string(),
            code_object_sha256: code_sha256,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn build_tape(
        &self,
        code: Arc<[u8]>,
        out: &DeviceBuffer,
        block: u32,
        timing_mode: TimingMode,
        replay_policy: ReplayPolicy,
        reps: usize,
        mutable_read_cache: MutableReadCache,
    ) -> Result<Tape> {
        let executable = Executable::load(&self.device, code)?;
        let kernel = executable.kernel(SYMBOL)?;
        let geometry = LaunchGeometry::new(
            [N.div_ceil(block) * block, 1, 1],
            [u16::try_from(block)?, 1, 1],
        )?;
        let mut commands = Gfx12Pm4CommandBuffer::new_stateful();
        let mut kernargs = Vec::new();
        let reused =
            if replay_policy.reuse_serial_kernarg() && timing_mode == TimingMode::SerialLatency {
                let mut kernarg = self.pool.allocate_for(kernel.metadata())?;
                fill_vopd_kernarg(&mut kernarg, out, 0, block)?;
                let address = kernarg.address();
                kernargs.push(kernarg);
                Some(address)
            } else {
                None
            };
        for operation in 0..reps {
            if operation > 0 && timing_mode == TimingMode::SerialLatency {
                dependency_boundary(&mut commands, mutable_read_cache);
            }
            let address = if let Some(address) = reused {
                address
            } else {
                let mut kernarg = self.pool.allocate_for(kernel.metadata())?;
                let output_offset = if timing_mode == TimingMode::IndependentThroughput {
                    operation * N as usize * 4
                } else {
                    0
                };
                fill_vopd_kernarg(&mut kernarg, out, output_offset, block)?;
                let address = kernarg.address();
                kernargs.push(kernarg);
                address
            };
            commands.dispatch(&kernel, geometry, 0, address)?;
        }
        let command_dwords = commands.len_dwords();
        let command_sha256 = hex_digest(&commands.as_bytes());
        let ib = SingleQueuePm4Ib::create_profiled(&self.device, &self.pool, &commands)?;
        let ownership = if replay_policy.reset_each_sample() {
            let mut acquire = Gfx12Pm4CommandBuffer::new();
            acquire.acquire_system_gfx12();
            Some(SingleQueuePm4Ib::create(
                &self.device,
                &self.pool,
                &acquire,
            )?)
        } else {
            None
        };
        Ok(Tape {
            ib,
            ownership,
            _executable: executable,
            _kernargs: kernargs,
            command_dwords,
            command_sha256,
            mutable_read_cache,
        })
    }

    fn reset(&self, out: &DeviceBuffer) -> Result<()> {
        self.hip.memset(out, 0, out.size())?;
        self.hip.device_synchronize()?;
        Ok(())
    }

    fn reset_and_acquire(&self, out: &DeviceBuffer, tape: &mut Tape) -> Result<()> {
        self.reset(out)?;
        if let Some(ownership) = tape.ownership.as_mut() {
            unsafe { ownership.replay_and_wait()? };
        } else {
            // Correctness checks also use an explicit acquire for hot policies.
            let mut acquire = Gfx12Pm4CommandBuffer::new();
            acquire.acquire_system_gfx12();
            let mut ownership = SingleQueuePm4Ib::create(&self.device, &self.pool, &acquire)?;
            unsafe { ownership.replay_and_wait()? };
        }
        Ok(())
    }

    fn check_output(
        &self,
        out: &DeviceBuffer,
        variant: &str,
        timing_mode: TimingMode,
        reps: usize,
    ) -> Result<(bool, f64, f64)> {
        let checked = 64usize;
        let mut bytes = vec![0u8; out.size()];
        self.hip.memcpy_dtoh(&mut bytes, out)?;
        let values = bytes
            .chunks_exact(4)
            .map(|chunk| f32::from_ne_bytes(chunk.try_into().unwrap()))
            .collect::<Vec<_>>();
        let mut max_abs = 0.0f64;
        let mut max_rel = 0.0f64;
        let slots = if timing_mode == TimingMode::IndependentThroughput {
            reps
        } else {
            1
        };
        for slot in 0..slots {
            for index in 0..checked {
                let one = vopd_value(index as u32, variant, BODY_ITERS);
                let expected = if timing_mode == TimingMode::SerialLatency {
                    one * reps as f32
                } else {
                    one
                };
                let observed = values[slot * N as usize + index];
                let abs = f64::from((observed - expected).abs());
                let rel = abs / f64::from(expected.abs().max(1.0e-6));
                max_abs = max_abs.max(abs);
                max_rel = max_rel.max(rel);
            }
        }
        Ok((max_abs <= 2.5e-3 || max_rel <= 2.5e-4, max_abs, max_rel))
    }
}

fn dependency_boundary(commands: &mut Gfx12Pm4CommandBuffer, cache: MutableReadCache) {
    match cache {
        MutableReadCache::VmemOnly => commands.dependency_rmw_hip_llvm_vmem_gfx12(),
        MutableReadCache::ScalarOrUnknown => commands.dependency_rmw_same_agent_gfx12(),
    }
}

fn fill_vopd_kernarg(
    kernarg: &mut KernargBuffer,
    out: &DeviceBuffer,
    output_offset: usize,
    block: u32,
) -> Result<()> {
    let bytes = kernarg.as_mut_bytes();
    if bytes.len() < 82 {
        bail!("VOPD kernarg segment is only {} bytes", bytes.len());
    }
    bytes.fill(0);
    let output = out.as_ptr() as usize as u64 + output_offset as u64;
    bytes[0..8].copy_from_slice(&output.to_ne_bytes());
    bytes[8..12].copy_from_slice(&N.to_ne_bytes());
    bytes[12..16].copy_from_slice(&BODY_ITERS.to_ne_bytes());
    let grid_groups = N.div_ceil(block);
    bytes[16..20].copy_from_slice(&grid_groups.to_ne_bytes());
    bytes[20..24].copy_from_slice(&1u32.to_ne_bytes());
    bytes[24..28].copy_from_slice(&1u32.to_ne_bytes());
    bytes[28..30].copy_from_slice(&(block as u16).to_ne_bytes());
    bytes[30..32].copy_from_slice(&1u16.to_ne_bytes());
    bytes[32..34].copy_from_slice(&1u16.to_ne_bytes());
    bytes[34..36].copy_from_slice(&(block as u16).to_ne_bytes());
    bytes[36..38].copy_from_slice(&1u16.to_ne_bytes());
    bytes[38..40].copy_from_slice(&1u16.to_ne_bytes());
    bytes[80..82].copy_from_slice(&1u16.to_ne_bytes());
    Ok(())
}

fn vopd_value(index: u32, variant: &str, body_iters: u32) -> f32 {
    let mut a = [0.0f32; 8];
    for (lane, value) in a.iter_mut().enumerate() {
        let bits = index.wrapping_mul(747_796_405)
            ^ (lane as u32).wrapping_mul(2_891_336_453)
            ^ 0x9e37_79b9;
        *value = 0.25 + (bits & 0xff) as f32 * 0.0009765625;
    }
    let mut u0 = index.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
    let mut u1 = index ^ 0xa5a5_a5a5;
    for iteration in 0..body_iters {
        match variant {
            "independent_fma" => {
                a[0] = a[0].mul_add(1.000001, lane_bias(index, iteration, 0));
                a[1] = a[1].mul_add(0.999999, lane_bias(index, iteration, 1));
                a[2] = a[2].mul_add(1.000002, lane_bias(index, iteration, 2));
                a[3] = a[3].mul_add(0.999998, lane_bias(index, iteration, 3));
            }
            "dependent_fma" => {
                a[0] = a[0].mul_add(1.000001, lane_bias(index, iteration, 0));
                a[0] = a[0].mul_add(0.999999, lane_bias(index, iteration, 1));
                a[0] = a[0].mul_add(1.000002, lane_bias(index, iteration, 2));
                a[0] = a[0].mul_add(0.999998, lane_bias(index, iteration, 3));
            }
            "mixed_int_float" => {
                u0 = hash_step(u0, iteration, 0);
                a[0] = a[0].mul_add(1.000001, (u0 & 0x1f) as f32 * 0.000001);
                u1 = hash_step(u1, iteration, 1);
                a[1] = a[1].mul_add(0.999999, ((u1 >> 3) & 0x1f) as f32 * 0.000001);
                u0 = hash_step(u0, iteration, 2);
                a[2] = a[2].mul_add(1.000002, ((u0 >> 7) & 0x1f) as f32 * 0.000001);
                u1 = hash_step(u1, iteration, 3);
                a[3] = a[3].mul_add(0.999998, ((u1 >> 11) & 0x1f) as f32 * 0.000001);
            }
            "dequant_like" => {
                u0 = hash_step(u0, iteration, 0);
                let q0 = ((u0 >> (iteration & 15)) & 0xff) as i32 - 128;
                a[0] = (q0 as f32 * 0.0078125).mul_add(0.03125, a[0]);
                u1 = hash_step(u1, iteration, 1);
                let q1 = ((u1 >> ((iteration + 3) & 15)) & 0xff) as i32 - 128;
                a[1] = (q1 as f32 * 0.0078125).mul_add(0.03125, a[1]);
                u0 = hash_step(u0, iteration, 2);
                let q2 = ((u0 >> ((iteration + 5) & 15)) & 0xff) as i32 - 128;
                a[2] = (q2 as f32 * 0.0078125).mul_add(0.03125, a[2]);
                u1 = hash_step(u1, iteration, 3);
                let q3 = ((u1 >> ((iteration + 7) & 15)) & 0xff) as i32 - 128;
                a[3] = (q3 as f32 * 0.0078125).mul_add(0.03125, a[3]);
            }
            _ => unreachable!(),
        }
    }
    a.into_iter().take(4).sum()
}

fn lane_bias(index: u32, iteration: u32, lane: u32) -> f32 {
    let bits = index
        .wrapping_mul(1_664_525)
        .wrapping_add(iteration.wrapping_mul(1_013_904_223))
        .wrapping_add(lane.wrapping_mul(2_246_822_519));
    ((bits >> 24) & 0x1f) as f32 * 0.000001 + 0.000003
}

fn hash_step(mut value: u32, iteration: u32, lane: u32) -> u32 {
    value ^= iteration
        .wrapping_mul(0x9e37_79b9)
        .wrapping_add(lane.wrapping_mul(0x85eb_ca6b));
    value = value.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    value ^ (value >> 16)
}

fn percentile(sorted: &[f64], quantile: f64) -> f64 {
    let position = quantile * (sorted.len() - 1) as f64;
    let low = position.floor() as usize;
    let high = position.ceil() as usize;
    if low == high {
        sorted[low]
    } else {
        sorted[low] * (high as f64 - position) + sorted[high] * (position - low as f64)
    }
}

fn hex_digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn parse_args() -> Result<Args> {
    let default_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../hipengine-6409/.artifacts/redline");
    let mut parsed = Args {
        artifact_root: default_root,
        out: None,
        reps: DEFAULT_REPS,
        warmups: DEFAULT_WARMUPS,
        samples: DEFAULT_SAMPLES,
        preheat: DEFAULT_PREHEAT,
        filter: None,
        policy: None,
    };
    let mut args = env::args().skip(1);
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--artifact-root" => {
                parsed.artifact_root = args
                    .next()
                    .context("--artifact-root requires a path")?
                    .into()
            }
            "--out" => parsed.out = Some(args.next().context("--out requires a path")?.into()),
            "--reps" => parsed.reps = args.next().context("--reps requires a value")?.parse()?,
            "--warmups" => {
                parsed.warmups = args.next().context("--warmups requires a value")?.parse()?
            }
            "--samples" => {
                parsed.samples = args.next().context("--samples requires a value")?.parse()?
            }
            "--preheat" => {
                parsed.preheat = args.next().context("--preheat requires a value")?.parse()?
            }
            "--filter" => parsed.filter = Some(args.next().context("--filter requires text")?),
            "--policy" => {
                let value = args.next().context("--policy requires a value")?;
                parsed.policy = Some(ReplayPolicy::parse(&value).with_context(|| {
                    format!("unknown policy {value}; expected hipengine, reuse, or safe")
                })?);
            }
            "--help" | "-h" => {
                println!(
                    "usage: hipengine_exact [--artifact-root PATH] [--out PATH] [--reps N] [--warmups N] [--samples N] [--preheat N] [--filter TEXT] [--policy hipengine|reuse|safe]"
                );
                std::process::exit(0);
            }
            other => bail!("unknown argument {other}"),
        }
    }
    if parsed.reps == 0 || parsed.samples == 0 {
        bail!("--reps and --samples must be positive");
    }
    Ok(parsed)
}

fn first_selected_case(args: &Args) -> Option<(&'static str, u32, TimingMode)> {
    for variant in VARIANTS {
        for block in BLOCKS {
            for timing_mode in TimingMode::ALL {
                let key = format!("{}/{variant}/wg{block}", timing_mode.as_str());
                if args
                    .filter
                    .as_deref()
                    .is_none_or(|filter| key.contains(filter))
                {
                    return Some((variant, block, timing_mode));
                }
            }
        }
    }
    None
}
