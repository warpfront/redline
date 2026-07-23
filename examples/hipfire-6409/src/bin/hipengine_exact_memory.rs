// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! Exact-artifact parity probe for the pinned HipEngine #6409 memory/waitcnt rows.
//!
//! The runner loads the already-built Radiowave-certified HipEngine code
//! objects without recompiling them. It recreates HipEngine's deterministic
//! fixture and launch ABI, then submits the retained sequence through the Rust
//! Redline API used by the Hipfire microbenchmark.

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
use std::mem::size_of_val;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const N: u32 = 32_768;
const BODY_ITERS: u32 = 64;
const DEFAULT_REPS: usize = 10;
const DEFAULT_WARMUPS: usize = 3;
const DEFAULT_SAMPLES: usize = 9;
const DEFAULT_PREHEAT: usize = 2_000;
const SYMBOL: &str = "_ZN12_GLOBAL__N_121memory_waitcnt_kernelEPKfPKjPfjjj.kd";
const BLOCKS: [u32; 2] = [64, 256];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum MemoryVariant {
    Coalesced,
    Strided,
    Gather,
    Interleave,
}

impl MemoryVariant {
    const ALL: [Self; 4] = [
        Self::Coalesced,
        Self::Strided,
        Self::Gather,
        Self::Interleave,
    ];

    fn as_str(self) -> &'static str {
        match self {
            Self::Coalesced => "coalesced",
            Self::Strided => "strided",
            Self::Gather => "gather",
            Self::Interleave => "interleave",
        }
    }

    fn param(self) -> u32 {
        match self {
            Self::Gather => 1,
            _ => 4,
        }
    }

    fn artifact_dir(self) -> String {
        format!("{}_p{}", self.as_str(), self.param())
    }

    fn data_elems(self) -> usize {
        let base = N as usize * BODY_ITERS as usize;
        let required = if self == Self::Gather {
            base
        } else {
            base * self.param() as usize
        };
        required.max(1024).next_power_of_two()
    }
}

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
    /// Match the HipEngine adapter's retained tape: distinct kernargs, hot
    /// samples, and no dependency acquire inside the timed IB.
    HipengineCompatible,
    /// Reuse the identical serial kernarg address so stateful PM4 can omit
    /// redundant user-data writes. This is measured, never assumed better.
    HipfireReuseHot,
    /// Reset before every sample and complete a separate system acquire before
    /// entering the timed retained tape.
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
    fixture_policy: &'static str,
    rows: Vec<Row>,
}

#[derive(Serialize)]
struct Row {
    variant: MemoryVariant,
    param: u32,
    block: u32,
    n: u32,
    body_iters: u32,
    data_elems: usize,
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

struct Fixture {
    x_host: Vec<f32>,
    ids_host: Vec<u32>,
    x: DeviceBuffer,
    ids: DeviceBuffer,
    out: DeviceBuffer,
}

struct Tape {
    ib: SingleQueuePm4Ib,
    ownership: SingleQueuePm4Ib,
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
                "[hipengine-exact-memory] preheat {}/{}/wg{} replays={}",
                timing_mode.as_str(),
                variant.as_str(),
                block,
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
    for variant in MemoryVariant::ALL {
        for block in BLOCKS {
            for timing_mode in TimingMode::ALL {
                let key = format!("{}/{}/wg{}", timing_mode.as_str(), variant.as_str(), block);
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
                    eprintln!("[hipengine-exact-memory] {key} {}", replay_policy.as_str());
                    rows.push(probe.measure(&args, variant, block, timing_mode, replay_policy)?);
                }
            }
        }
    }

    let report = Report {
        kind: "hipfire_hipengine_exact_memory_waitcnt_parity",
        artifact_root: args.artifact_root.display().to_string(),
        code_object_policy:
            "load exact HipEngine Radiowave-certified .redline.co without recompilation",
        fixture_policy: "byte-identical deterministic HipEngine memory/waitcnt input generator",
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
        variant: MemoryVariant,
        block: u32,
        timing_mode: TimingMode,
        replay_policy: ReplayPolicy,
    ) -> Result<Row> {
        let directory = args
            .artifact_root
            .join("memory-waitcnt/hip")
            .join(variant.artifact_dir())
            .join(format!("wg{block}"));
        let code_path = directory.join("hip_memory_waitcnt.redline.co");
        let manifest_path = directory.join("hip_memory_waitcnt.redline.radiowave.json");
        let code = fs::read(&code_path)
            .with_context(|| format!("read exact HipEngine artifact {}", code_path.display()))?;
        let manifest = fs::read_to_string(&manifest_path)
            .with_context(|| format!("read certification {}", manifest_path.display()))?;
        let certification = CodeObjectCertification::from_json(&code, &manifest)
            .context("verify exact HipEngine artifact certification")?;
        let mutable_read_cache = certification.mutable_read_cache(SYMBOL);
        let code_sha256 = hex_digest(&code);

        let fixture = self.make_fixture(variant, timing_mode, args.reps)?;
        let mut tape = self.build_tape(
            Arc::<[u8]>::from(code),
            &fixture,
            block,
            timing_mode,
            replay_policy,
            args.reps,
            mutable_read_cache,
        )?;

        // H2D input initialization happened through HIP. Establish ownership
        // once before any direct queue use; this is outside every timed sample.
        unsafe { tape.ownership.replay_and_wait()? };
        for _ in 0..args.warmups {
            if replay_policy.reset_each_sample() {
                self.reset_and_acquire(&fixture.out, &mut tape)?;
            }
            unsafe { tape.ib.replay_and_wait_profiled()? };
        }
        if !replay_policy.reset_each_sample() {
            self.reset(&fixture.out)?;
        }

        let mut samples = Vec::with_capacity(args.samples);
        for _ in 0..args.samples {
            if replay_policy.reset_each_sample() {
                self.reset_and_acquire(&fixture.out, &mut tape)?;
            }
            let timing = unsafe { tape.ib.replay_and_wait_profiled()? };
            samples.push(timing.span_microseconds() / args.reps as f64);
        }

        self.reset_and_acquire(&fixture.out, &mut tape)?;
        unsafe { tape.ib.replay_and_wait_profiled()? };
        let (correctness_pass, max_abs, max_rel) =
            self.check_output(&fixture, variant, timing_mode, args.reps)?;

        let command_dwords = tape.command_dwords;
        let command_sha256 = tape.command_sha256.clone();
        let mutable_read_cache = tape.mutable_read_cache;
        drop(tape);
        self.hip.free(fixture.x)?;
        self.hip.free(fixture.ids)?;
        self.hip.free(fixture.out)?;

        samples.sort_by(f64::total_cmp);
        Ok(Row {
            variant,
            param: variant.param(),
            block,
            n: N,
            body_iters: BODY_ITERS,
            data_elems: variant.data_elems(),
            timing_mode,
            replay_policy,
            reps: args.reps,
            samples: args.samples,
            median_us: percentile(&samples, 0.5),
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
                "one_initial_system_acquire_then_hot_samples"
            },
            mutable_read_cache,
            correctness_pass,
            max_abs,
            max_rel,
            code_object: code_path.display().to_string(),
            code_object_sha256: code_sha256,
        })
    }

    fn make_fixture(
        &self,
        variant: MemoryVariant,
        timing_mode: TimingMode,
        reps: usize,
    ) -> Result<Fixture> {
        let mut x_host = Vec::with_capacity(variant.data_elems());
        for index in 0..variant.data_elems() as u32 {
            x_host.push(data_value(index));
        }
        let mut ids_host = Vec::with_capacity(N as usize * BODY_ITERS as usize);
        let mask = variant.data_elems() as u32 - 1;
        for iteration in 0..BODY_ITERS {
            for index in 0..N {
                ids_host.push(
                    hash_u32(
                        index
                            .wrapping_mul(747_796_405)
                            .wrapping_add(iteration.wrapping_mul(2_891_336_453)),
                    ) & mask,
                );
            }
        }
        let output_slots = if timing_mode == TimingMode::IndependentThroughput {
            reps
        } else {
            1
        };
        let x = self.hip.malloc(size_of_val(x_host.as_slice()))?;
        let ids = self.hip.malloc(size_of_val(ids_host.as_slice()))?;
        let out = self
            .hip
            .malloc(output_slots * N as usize * size_of::<f32>())?;
        self.hip.memcpy_htod(&x, as_bytes(&x_host))?;
        self.hip.memcpy_htod(&ids, as_bytes(&ids_host))?;
        self.reset(&out)?;
        Ok(Fixture {
            x_host,
            ids_host,
            x,
            ids,
            out,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn build_tape(
        &self,
        code: Arc<[u8]>,
        fixture: &Fixture,
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
                fill_memory_kernarg(&mut kernarg, fixture, 0, variant_data_mask(fixture), block)?;
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
                    operation * N as usize * size_of::<f32>()
                } else {
                    0
                };
                fill_memory_kernarg(
                    &mut kernarg,
                    fixture,
                    output_offset,
                    variant_data_mask(fixture),
                    block,
                )?;
                let address = kernarg.address();
                kernargs.push(kernarg);
                address
            };
            commands.dispatch(&kernel, geometry, 0, address)?;
        }

        let command_dwords = commands.len_dwords();
        let command_sha256 = hex_digest(&commands.as_bytes());
        let ib = SingleQueuePm4Ib::create_profiled(&self.device, &self.pool, &commands)?;
        let mut acquire = Gfx12Pm4CommandBuffer::new();
        acquire.acquire_system_gfx12();
        let ownership = SingleQueuePm4Ib::create(&self.device, &self.pool, &acquire)?;
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
        unsafe { tape.ownership.replay_and_wait()? };
        Ok(())
    }

    fn check_output(
        &self,
        fixture: &Fixture,
        variant: MemoryVariant,
        timing_mode: TimingMode,
        reps: usize,
    ) -> Result<(bool, f64, f64)> {
        let mut bytes = vec![0u8; fixture.out.size()];
        self.hip.memcpy_dtoh(&mut bytes, &fixture.out)?;
        let values = bytes
            .chunks_exact(size_of::<f32>())
            .map(|chunk| f32::from_ne_bytes(chunk.try_into().unwrap()))
            .collect::<Vec<_>>();
        let mut max_abs = 0.0f64;
        let mut max_rel = 0.0f64;
        let slots = if timing_mode == TimingMode::IndependentThroughput {
            reps
        } else {
            1
        };
        let data_mask = fixture.x_host.len() as u32 - 1;
        for slot in 0..slots {
            for index in 0..64usize {
                let one = memory_value(
                    &fixture.x_host,
                    &fixture.ids_host,
                    variant,
                    index as u32,
                    data_mask,
                );
                let mut expected = 0.0f32;
                let logical_iterations = if timing_mode == TimingMode::SerialLatency {
                    reps
                } else {
                    1
                };
                for _ in 0..logical_iterations {
                    expected += one;
                }
                let observed = values[slot * N as usize + index];
                let abs = f64::from((observed - expected).abs());
                let rel = abs / f64::from(expected.abs().max(1.0e-6));
                max_abs = max_abs.max(abs);
                max_rel = max_rel.max(rel);
            }
        }
        Ok((max_abs <= 5.0e-3 || max_rel <= 5.0e-4, max_abs, max_rel))
    }
}

fn dependency_boundary(commands: &mut Gfx12Pm4CommandBuffer, cache: MutableReadCache) {
    match cache {
        MutableReadCache::VmemOnly => commands.dependency_rmw_hip_llvm_vmem_gfx12(),
        MutableReadCache::ScalarOrUnknown => commands.dependency_rmw_same_agent_gfx12(),
    }
}

fn variant_data_mask(fixture: &Fixture) -> u32 {
    fixture.x_host.len() as u32 - 1
}

fn fill_memory_kernarg(
    kernarg: &mut KernargBuffer,
    fixture: &Fixture,
    output_offset: usize,
    data_mask: u32,
    block: u32,
) -> Result<()> {
    let bytes = kernarg.as_mut_bytes();
    if bytes.len() < 106 {
        bail!(
            "memory/waitcnt kernarg segment is only {} bytes",
            bytes.len()
        );
    }
    bytes.fill(0);
    let x = fixture.x.as_ptr() as usize as u64;
    let ids = fixture.ids.as_ptr() as usize as u64;
    let output = fixture.out.as_ptr() as usize as u64 + output_offset as u64;
    bytes[0..8].copy_from_slice(&x.to_ne_bytes());
    bytes[8..16].copy_from_slice(&ids.to_ne_bytes());
    bytes[16..24].copy_from_slice(&output.to_ne_bytes());
    bytes[24..28].copy_from_slice(&N.to_ne_bytes());
    bytes[28..32].copy_from_slice(&BODY_ITERS.to_ne_bytes());
    bytes[32..36].copy_from_slice(&data_mask.to_ne_bytes());
    let grid_groups = N.div_ceil(block);
    bytes[40..44].copy_from_slice(&grid_groups.to_ne_bytes());
    bytes[44..48].copy_from_slice(&1u32.to_ne_bytes());
    bytes[48..52].copy_from_slice(&1u32.to_ne_bytes());
    bytes[52..54].copy_from_slice(&(block as u16).to_ne_bytes());
    bytes[54..56].copy_from_slice(&1u16.to_ne_bytes());
    bytes[56..58].copy_from_slice(&1u16.to_ne_bytes());
    bytes[58..60].copy_from_slice(&(block as u16).to_ne_bytes());
    bytes[60..62].copy_from_slice(&1u16.to_ne_bytes());
    bytes[62..64].copy_from_slice(&1u16.to_ne_bytes());
    bytes[104..106].copy_from_slice(&1u16.to_ne_bytes());
    Ok(())
}

fn memory_value(x: &[f32], ids: &[u32], variant: MemoryVariant, index: u32, data_mask: u32) -> f32 {
    let mut sum = 0.0f32;
    let mut state = index.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
    for iteration in 0..BODY_ITERS {
        match variant {
            MemoryVariant::Coalesced => {
                let base = (iteration
                    .wrapping_mul(N)
                    .wrapping_mul(variant.param())
                    .wrapping_add(index.wrapping_mul(variant.param())))
                    & data_mask;
                for lane in 0..variant.param() {
                    let value = x[((base + lane) & data_mask) as usize];
                    sum = value.mul_add(1.000001 + lane as f32 * 0.000001, sum);
                }
            }
            MemoryVariant::Strided => {
                let address = (iteration
                    .wrapping_mul(N)
                    .wrapping_mul(variant.param())
                    .wrapping_add(index.wrapping_mul(variant.param())))
                    & data_mask;
                sum = x[address as usize].mul_add(1.000001, sum);
            }
            MemoryVariant::Gather => {
                let id = ids[iteration as usize * N as usize + index as usize] & data_mask;
                sum = x[id as usize].mul_add(1.000001, sum);
            }
            MemoryVariant::Interleave => {
                let base = (iteration
                    .wrapping_mul(N)
                    .wrapping_mul(variant.param())
                    .wrapping_add(index.wrapping_mul(variant.param())))
                    & data_mask;
                for lane in 0..variant.param() {
                    state = hash_u32(state.wrapping_add(iteration).wrapping_add(lane));
                    let value = x[((base + lane) & data_mask) as usize];
                    let bias = (state & 0xff) as f32 * 0.0000001;
                    sum = (value + bias).mul_add(0.999999 + lane as f32 * 0.000001, sum);
                }
            }
        }
    }
    sum
}

fn hash_u32(mut value: u32) -> u32 {
    value ^= value >> 16;
    value = value.wrapping_mul(0x7feb_352d);
    value ^= value >> 15;
    value = value.wrapping_mul(0x846c_a68b);
    value ^ (value >> 16)
}

fn data_value(index: u32) -> f32 {
    let bits = hash_u32(index.wrapping_mul(1_664_525).wrapping_add(1_013_904_223));
    let value = (bits & 0x3ff) as i32 - 512;
    value as f32 * 0.0009765625
}

fn as_bytes<T>(values: &[T]) -> &[u8] {
    // SAFETY: a byte slice has alignment one, spans exactly the initialized
    // source slice, and is used only for a synchronous H2D copy.
    unsafe { std::slice::from_raw_parts(values.as_ptr().cast::<u8>(), size_of_val(values)) }
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
                    "usage: hipengine_exact_memory [--artifact-root PATH] [--out PATH] [--reps N] [--warmups N] [--samples N] [--preheat N] [--filter TEXT] [--policy hipengine|reuse|safe]"
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

fn first_selected_case(args: &Args) -> Option<(MemoryVariant, u32, TimingMode)> {
    for variant in MemoryVariant::ALL {
        for block in BLOCKS {
            for timing_mode in TimingMode::ALL {
                let key = format!("{}/{}/wg{}", timing_mode.as_str(), variant.as_str(), block);
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
