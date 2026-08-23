// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! Pure-Rust #6409 dispatch-floor: the SAME `gmb_noop_kernel` serial-latency
//! chain as `examples/dispatch-floor-6409`, driven straight through
//! `SingleQueuePm4Ib` with NO Python/C FFI in the timed path. This isolates the
//! retained-PM4 replay cost from binding overhead.
//!
//! Dependency-boundary arms separate completion ordering from cache
//! invalidation. Every arm must leave every output element exactly equal to the
//! dispatch count before its timing can be trusted.
//!
//! Env: `GMB_HSACO` (required), `GMB_KERNEL_ABI` (`flat` or `buffer`), `GMB_N`
//! (256), `GMB_COUNTS` (1,50,200,941), `GMB_REPS` (50), `GMB_WARMUP` (10),
//! `GMB_ONLY` (optional mode id). Pick the GPU with `ROCR_VISIBLE_DEVICES`.

use std::sync::Arc;
use std::time::Instant;

use redline_dispatch::aql::{
    BatchFencePolicy, DevicePool, Executable, Gfx12DispatchMode, Gfx12KernelImage,
    Gfx12Pm4CommandBuffer, Gfx12RmwAcquirePolicy, GpuDevice, GpuSelector, KernargPool,
    LaunchGeometry, RecordedDispatch, Runtime, SingleQueueBatchGraph, SingleQueuePm4Ib,
    load_symbols,
};

#[derive(Clone, Copy)]
struct Mode {
    id: &'static str,
    encoder: &'static str,
    boundary: &'static str,
    wait: bool,
    acquire: Option<Gfx12RmwAcquirePolicy>,
    stateful: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum GmbKernelAbi {
    Flat,
    Buffer,
}

impl GmbKernelAbi {
    fn from_env() -> Result<Self, &'static str> {
        match std::env::var("GMB_KERNEL_ABI")
            .unwrap_or_else(|_| "flat".to_owned())
            .as_str()
        {
            "flat" => Ok(Self::Flat),
            "buffer" | "srd" => Ok(Self::Buffer),
            _ => Err("GMB_KERNEL_ABI must be flat or buffer"),
        }
    }

    fn symbol(self) -> &'static str {
        match self {
            Self::Flat => "gmb_noop_kernel.kd",
            Self::Buffer => "gmb_buffer_kernel.kd",
        }
    }

    fn populate_kernarg(
        self,
        bytes: &mut [u8],
        out_addr: u64,
        n: u32,
        grid_blocks: u32,
        block: u32,
    ) -> Result<(), &'static str> {
        bytes.fill(0);
        match self {
            Self::Flat => {
                // gmb_noop's kernarg segment is 272B: two explicit args then the
                // AMDGPU hidden arguments. gmb_noop computes
                // `idx = blockIdx.x*blockDim.x + threadIdx.x`, and blockDim.x is
                // read from `hidden_group_size_x`. Without it that term is 0, so
                // every workgroup collapses onto out[0..n] and grid>1 races —
                // the "no distinct block ID" failure. Offsets are the code
                // object's own metadata (llvm-readobj --notes on the unbundled
                // ELF): block_count_x/y/z @16/20/24 (u32), group_size_x/y/z
                // @28/30/32 (u16), grid_dims @80 (u16).
                if bytes.len() < 82 {
                    return Err("flat gmb kernarg is shorter than its 272-byte segment");
                }
                bytes[..8].copy_from_slice(&out_addr.to_le_bytes());
                bytes[8..12].copy_from_slice(&n.to_le_bytes());
                bytes[16..20].copy_from_slice(&grid_blocks.to_le_bytes());
                bytes[20..24].copy_from_slice(&1u32.to_le_bytes());
                bytes[24..28].copy_from_slice(&1u32.to_le_bytes());
                bytes[28..30].copy_from_slice(&(block as u16).to_le_bytes());
                bytes[30..32].copy_from_slice(&1u16.to_le_bytes());
                bytes[32..34].copy_from_slice(&1u16.to_le_bytes());
                bytes[80..82].copy_from_slice(&1u16.to_le_bytes());
            }
            Self::Buffer => {
                if bytes.len() < 20 {
                    return Err("buffer gmb kernarg is shorter than 20 bytes");
                }
                // GFX11/GFX12 raw-buffer SRD: 64-bit base, byte range, and
                // CK/RADV-compatible float32 buffer configuration.
                bytes[..8].copy_from_slice(&out_addr.to_le_bytes());
                bytes[8..12].copy_from_slice(&(n * 4).to_le_bytes());
                bytes[12..16].copy_from_slice(&0x3100_4000_u32.to_le_bytes());
                bytes[16..20].copy_from_slice(&n.to_le_bytes());
            }
        }
        Ok(())
    }
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

fn env_usize(k: &str, d: usize) -> usize {
    std::env::var(k)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(d)
}

fn grid_blocks(n: u32) -> u32 {
    std::env::var("GMB_GRID_BLOCKS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| n.div_ceil(256))
}

fn dispatch_gmb(
    cmd: &mut Gfx12Pm4CommandBuffer,
    kernel: &redline_dispatch::aql::Kernel,
    geometry: LaunchGeometry,
    kernarg_address: *mut std::ffi::c_void,
) -> Result<(), redline_dispatch::aql::Pm4BuildError> {
    let address = kernarg_address as usize as u64;
    let user_sgprs = [address as u32, (address >> 32) as u32];
    // GMB_DISPATCH_MODE selects the DISPATCH_DIRECT dimension encoding.
    // `workitems` sets USE_THREAD_DIMENSIONS (initiator bit 5) and passes the
    // grid as work-items; the CP divides by COMPUTE_NUM_THREAD_X, which is what
    // delivers distinct hardware workgroup IDs (blockIdx) to hipcc kernels.
    // `radv` uses workgroup-count dims (RADV/ACO style) and does NOT supply a
    // distinct block ID to this kernel on gfx1201 — kept only for A/B.
    let mode = match std::env::var("GMB_DISPATCH_MODE").as_deref() {
        Ok("radv") | Ok("workgroups") => Gfx12DispatchMode::RadvWorkgroups,
        _ => Gfx12DispatchMode::Workitems,
    };
    cmd.dispatch_image_with_mode(
        &Gfx12KernelImage::from_hsa(kernel)?,
        geometry,
        0,
        &user_sgprs,
        mode,
    )
}

/// Build a retained PM4 IB of `count` gmb_noop dispatches against `out`.
/// `fence` inserts the correct RMW dependency boundary between dispatches.
// Argument list mirrors the PM4 IB build ABI (device/pool/exec + launch params).
#[allow(clippy::too_many_arguments)]
fn build_ib(
    device: &GpuDevice,
    pool: &KernargPool,
    exec: &Executable,
    n: u32,
    count: usize,
    mode: Mode,
    kernel_abi: GmbKernelAbi,
    out_addr: u64,
    profiling: bool,
) -> Result<(SingleQueuePm4Ib, redline_dispatch::aql::KernargBuffer), Box<dyn std::error::Error>> {
    let block = 256u32;
    let grid_blocks = grid_blocks(n);
    let workitems = grid_blocks * block;
    let kernel = exec.kernel(kernel_abi.symbol())?;
    let mut karg = pool.allocate_for(kernel.metadata())?;
    kernel_abi.populate_kernarg(karg.as_mut_bytes(), out_addr, n, grid_blocks, block)?;
    let mut cmd = if mode.stateful {
        Gfx12Pm4CommandBuffer::new_stateful()
    } else {
        Gfx12Pm4CommandBuffer::new()
    };
    for i in 0..count {
        let geometry = LaunchGeometry::new([workitems, 1, 1], [block as u16, 1, 1])?;
        dispatch_gmb(&mut cmd, &kernel, geometry, karg.address())?;
        if i + 1 < count {
            if mode.wait {
                cmd.wait_compute_idle();
            }
            if let Some(policy) = mode.acquire {
                cmd.acquire_rmw_gfx12(policy);
            }
        }
    }
    let ib = if profiling {
        SingleQueuePm4Ib::create_profiled(device, pool, &cmd)?
    } else {
        SingleQueuePm4Ib::create(device, pool, &cmd)?
    };
    Ok((ib, karg))
}
// Mirrors the gmb_floor microbench ABI (device resources + launch params + reps).
#[allow(clippy::too_many_arguments)]
fn measure(
    device: &GpuDevice,
    device_pool: &DevicePool,
    pool: &KernargPool,
    exec: &Executable,
    n: u32,
    count: usize,
    reps: usize,
    warmup: usize,
    mode: Mode,
    kernel_abi: GmbKernelAbi,
) -> Result<(f64, bool), Box<dyn std::error::Error>> {
    let mut out = device_pool.allocate((n as usize) * 4)?;
    let zeros = vec![0_u8; out.len()];
    // SAFETY: the allocation has no GPU users before the retained IB is built.
    unsafe { out.copy_from_host(&zeros)? };
    let out_addr = out.address() as usize as u64;
    let (mut ib, _karg) = build_ib(
        device, pool, exec, n, count, mode, kernel_abi, out_addr, false,
    )?;

    // Correctness: one replay from zero must update every element exactly once
    // per dispatch. Checking only element zero can hide a partial-grid failure.
    unsafe { ib.replay_and_wait()? };
    let expected = (count as f32).to_bits();
    let mut observed = vec![0_u8; out.len()];
    // SAFETY: replay completion proves the GPU is no longer using `out`.
    unsafe { out.copy_to_host(&mut observed)? };
    let (chunks, _) = observed.as_chunks::<4>();
    let correct = chunks
        .iter()
        .all(|bytes| u32::from_le_bytes(*bytes) == expected);
    if !correct && std::env::var_os("GMB_DEBUG_MISMATCH").is_some() {
        let (chunks, _) = observed.as_chunks::<4>();
        let values: Vec<f32> = chunks
            .iter()
            .take(16)
            .map(|bytes| f32::from_le_bytes(*bytes))
            .collect();
        eprintln!(
            "serial mismatch grid={} count={count}: {values:?}",
            grid_blocks(n)
        );
    }

    for _ in 0..warmup {
        unsafe { ib.replay_and_wait()? };
    }
    let mut ts = Vec::with_capacity(reps);
    for _ in 0..reps {
        let t0 = Instant::now();
        unsafe { ib.replay_and_wait()? };
        ts.push(t0.elapsed().as_secs_f64() * 1e6);
    }
    let final_expected = (((1 + warmup + reps) * count) as f32).to_bits();
    unsafe { out.copy_to_host(&mut observed)? };
    let (chunks, _) = observed.as_chunks::<4>();
    let final_correct = chunks
        .iter()
        .all(|bytes| u32::from_le_bytes(*bytes) == final_expected);
    Ok((median(ts) / count as f64, correct && final_correct))
}
// Mirrors the gmb_floor microbench ABI (device resources + launch params + reps).
#[allow(clippy::too_many_arguments)]
fn measure_profiled_pm4(
    device: &GpuDevice,
    device_pool: &DevicePool,
    pool: &KernargPool,
    exec: &Executable,
    n: u32,
    count: usize,
    reps: usize,
    warmup: usize,
    mode: Mode,
    kernel_abi: GmbKernelAbi,
) -> Result<(f64, bool), Box<dyn std::error::Error>> {
    let mut out = device_pool.allocate((n as usize) * 4)?;
    let zeros = vec![0_u8; out.len()];
    unsafe { out.copy_from_host(&zeros)? };
    let out_addr = out.address() as usize as u64;
    let (mut ib, _karg) = build_ib(
        device, pool, exec, n, count, mode, kernel_abi, out_addr, true,
    )?;

    let _ = unsafe { ib.replay_and_wait_profiled()? };
    let expected = (count as f32).to_bits();
    let mut observed = vec![0_u8; out.len()];
    unsafe { out.copy_to_host(&mut observed)? };
    let (chunks, _) = observed.as_chunks::<4>();
    let correct = chunks
        .iter()
        .all(|bytes| u32::from_le_bytes(*bytes) == expected);
    if !correct && std::env::var_os("GMB_DEBUG_MISMATCH").is_some() {
        let (chunks, _) = observed.as_chunks::<4>();
        let values: Vec<f32> = chunks
            .iter()
            .take(16)
            .map(|bytes| f32::from_le_bytes(*bytes))
            .collect();
        eprintln!(
            "profiled mismatch grid={} count={count}: {values:?}",
            grid_blocks(n)
        );
    }

    for _ in 0..warmup {
        let _ = unsafe { ib.replay_and_wait_profiled()? };
    }
    let mut samples = Vec::with_capacity(reps);
    for _ in 0..reps {
        samples.push(unsafe { ib.replay_and_wait_profiled()? }.span_microseconds());
    }
    Ok((median(samples) / count as f64, correct))
}
// Mirrors the gmb_floor microbench ABI (device resources + launch params + reps).
#[allow(clippy::too_many_arguments)]
fn measure_profiled_pm4_independent(
    device: &GpuDevice,
    device_pool: &DevicePool,
    pool: &KernargPool,
    exec: &Executable,
    n: u32,
    count: usize,
    reps: usize,
    warmup: usize,
    kernel_abi: GmbKernelAbi,
) -> Result<(f64, bool), Box<dyn std::error::Error>> {
    let slice_bytes = (n as usize) * 4;
    let mut out = device_pool.allocate(slice_bytes * count)?;
    let zeros = vec![0_u8; out.len()];
    unsafe { out.copy_from_host(&zeros)? };
    let base = out.address() as usize as u64;
    let block = 256u32;
    let workitems = grid_blocks(n) * block;
    let kernel = exec.kernel(kernel_abi.symbol())?;
    let mut kernargs = Vec::with_capacity(count);
    let mut cmd = Gfx12Pm4CommandBuffer::new_stateful();
    for index in 0..count {
        let mut karg = pool.allocate_for(kernel.metadata())?;
        kernel_abi.populate_kernarg(
            karg.as_mut_bytes(),
            base + (index * slice_bytes) as u64,
            n,
            grid_blocks(n),
            block,
        )?;
        let geometry = LaunchGeometry::new([workitems, 1, 1], [block as u16, 1, 1])?;
        dispatch_gmb(&mut cmd, &kernel, geometry, karg.address())?;
        kernargs.push(karg);
    }
    let mut ib = SingleQueuePm4Ib::create_profiled(device, pool, &cmd)?;
    let _ = unsafe { ib.replay_and_wait_profiled()? };
    let mut observed = vec![0_u8; out.len()];
    unsafe { out.copy_to_host(&mut observed)? };
    let (chunks, _) = observed.as_chunks::<4>();
    let correct = chunks
        .iter()
        .all(|bytes| u32::from_le_bytes(*bytes) == 1.0_f32.to_bits());
    if !correct && std::env::var_os("GMB_DEBUG_MISMATCH").is_some() {
        let (chunks, _) = observed.as_chunks::<4>();
        let values: Vec<f32> = chunks
            .iter()
            .take(16)
            .map(|bytes| f32::from_le_bytes(*bytes))
            .collect();
        eprintln!(
            "independent mismatch grid={} count={count}: {values:?}",
            grid_blocks(n)
        );
    }
    for _ in 0..warmup {
        let _ = unsafe { ib.replay_and_wait_profiled()? };
    }
    let mut samples = Vec::with_capacity(reps);
    for _ in 0..reps {
        samples.push(unsafe { ib.replay_and_wait_profiled()? }.span_microseconds());
    }
    drop(kernargs);
    Ok((median(samples) / count as f64, correct))
}

/// Redline's GPU-execution span for the gmb_noop dependency chain via the AQL
/// `SingleQueueBatchGraph` (BoundarySerialized), which exposes GPU timestamps.
/// This isolates GPU dispatch time from the host submit+wait overhead of the
/// PM4 host-latency arms — the fair basis versus Vulkan's GPU timestamp.
// Mirrors the gmb_floor microbench ABI (device resources + launch params + reps).
#[allow(clippy::too_many_arguments)]
fn measure_gpuspan(
    device: &GpuDevice,
    device_pool: &DevicePool,
    pool: &KernargPool,
    exec: &Executable,
    n: u32,
    count: usize,
    reps: usize,
    warmup: usize,
    kernel_abi: GmbKernelAbi,
) -> Result<(f64, bool), Box<dyn std::error::Error>> {
    let mut out = device_pool.allocate((n as usize) * 4)?;
    let zeros = vec![0_u8; out.len()];
    // SAFETY: no dispatch can access `out` before graph construction completes.
    unsafe { out.copy_from_host(&zeros)? };
    let out_addr = out.address() as usize as u64;
    let block = 256u32;
    let grid_blocks = grid_blocks(n);
    let workitems = grid_blocks * block;
    let mut dispatches = Vec::with_capacity(count);
    for i in 0..count {
        let kernel = exec.kernel(kernel_abi.symbol())?;
        let mut karg = pool.allocate_for(kernel.metadata())?;
        kernel_abi.populate_kernarg(karg.as_mut_bytes(), out_addr, n, grid_blocks, block)?;
        let geometry = LaunchGeometry::new([workitems, 1, 1], [block as u16, 1, 1])?;
        // BoundarySerialized serializes the batch itself; no explicit deps.
        let _ = i;
        dispatches.push(RecordedDispatch::new(0, kernel, geometry, karg)?);
    }
    let range = device.queue_size_range();
    let want = ((count as u32).saturating_add(16)).next_power_of_two();
    let queue_size = want.clamp(*range.start(), *range.end());
    // AgentEveryInternalDispatch fences every dispatch at agent scope (no
    // access/dep declaration needed) — the minimal policy that correctly
    // serializes the RMW chain in the RecordedDispatch path.
    let mut graph = SingleQueueBatchGraph::create(
        device,
        queue_size,
        dispatches,
        BatchFencePolicy::AgentEveryInternalDispatch,
    )?;

    let _ = unsafe { graph.replay_and_wait()? };
    let expected = (count as f32).to_bits();
    let mut observed = vec![0_u8; out.len()];
    // SAFETY: replay completion proves the GPU is no longer using `out`.
    unsafe { out.copy_to_host(&mut observed)? };
    let (chunks, _) = observed.as_chunks::<4>();
    let correct = chunks
        .iter()
        .all(|bytes| u32::from_le_bytes(*bytes) == expected);

    for _ in 0..warmup {
        let _ = unsafe { graph.replay_and_wait()? };
    }
    let mut spans = Vec::with_capacity(reps);
    for _ in 0..reps {
        let t = unsafe { graph.replay_and_wait()? };
        spans.push(t.span_microseconds());
    }
    Ok((median(spans) / count as f64, correct))
}

/// redline "serial": submit each dispatch as its OWN retained 1-dispatch IB
/// (N submit+wait cycles), host-timed. This is redline with no batching — the
/// raw per-submission cost, the direct analogue of hip_direct (no graph).
// Mirrors the gmb_floor microbench ABI (device resources + launch params + reps).
#[allow(clippy::too_many_arguments)]
fn measure_serial(
    device: &GpuDevice,
    device_pool: &DevicePool,
    pool: &KernargPool,
    exec: &Executable,
    n: u32,
    count: usize,
    reps: usize,
    warmup: usize,
    mode: Mode,
    kernel_abi: GmbKernelAbi,
) -> Result<(f64, bool), Box<dyn std::error::Error>> {
    let mut out = device_pool.allocate((n as usize) * 4)?;
    let zeros = vec![0_u8; out.len()];
    unsafe { out.copy_from_host(&zeros)? };
    let out_addr = out.address() as usize as u64;
    // ONE retained 1-dispatch IB, submitted `count` times in sequence. Because
    // the boundary in a batched IB lands BETWEEN dispatches, a lone dispatch
    // gets none — so each independent submit needs a LEADING acquire to read
    // the prior submit's result coherently (its completion signal already
    // drained compute and flushed writes; this invalidates stale read caches).
    let block = 256u32;
    let grid_blocks = grid_blocks(n);
    let workitems = grid_blocks * block;
    let kernel = exec.kernel(kernel_abi.symbol())?;
    let mut karg = pool.allocate_for(kernel.metadata())?;
    kernel_abi.populate_kernarg(karg.as_mut_bytes(), out_addr, n, grid_blocks, block)?;
    let mut cmd = Gfx12Pm4CommandBuffer::new_stateful();
    if mode.wait {
        cmd.wait_compute_idle();
    }
    if let Some(policy) = mode.acquire {
        cmd.acquire_rmw_gfx12(policy);
    }
    let geometry = LaunchGeometry::new([workitems, 1, 1], [block as u16, 1, 1])?;
    dispatch_gmb(&mut cmd, &kernel, geometry, karg.address())?;
    let mut ib = SingleQueuePm4Ib::create(device, pool, &cmd)?;
    let _karg = karg;

    // Correctness: from zero, `count` host-serialized submits -> out == count.
    unsafe { out.copy_from_host(&zeros)? };
    for _ in 0..count {
        unsafe { ib.replay_and_wait()? };
    }
    let expected = (count as f32).to_bits();
    let mut observed = vec![0_u8; out.len()];
    unsafe { out.copy_to_host(&mut observed)? };
    let (chunks, _) = observed.as_chunks::<4>();
    let correct = chunks.iter().all(|b| u32::from_le_bytes(*b) == expected);

    for _ in 0..warmup {
        unsafe { ib.replay_and_wait()? };
    }
    let mut ts = Vec::with_capacity(reps);
    for _ in 0..reps {
        let t0 = Instant::now();
        for _ in 0..count {
            unsafe { ib.replay_and_wait()? };
        }
        ts.push(t0.elapsed().as_secs_f64() * 1e6);
    }
    Ok((median(ts) / count as f64, correct))
}

/// redline "AQL": the same dispatches as ONE queue submission of N AQL
/// kernel-dispatch packets (`SingleQueueBatchGraph`), host-timed. Same batching
/// as the PM4 IB, but standard AQL packets instead of a CP-streamed PM4 IB —
/// isolates the AQL-packet-processing overhead the PM4 IB removes.
// Mirrors the gmb_floor microbench ABI (device resources + launch params + reps).
#[allow(clippy::too_many_arguments)]
fn measure_aql_host(
    device: &GpuDevice,
    device_pool: &DevicePool,
    pool: &KernargPool,
    exec: &Executable,
    n: u32,
    count: usize,
    reps: usize,
    warmup: usize,
    kernel_abi: GmbKernelAbi,
) -> Result<(f64, bool), Box<dyn std::error::Error>> {
    let mut out = device_pool.allocate((n as usize) * 4)?;
    let zeros = vec![0_u8; out.len()];
    unsafe { out.copy_from_host(&zeros)? };
    let out_addr = out.address() as usize as u64;
    let block = 256u32;
    let grid_blocks = grid_blocks(n);
    let workitems = grid_blocks * block;
    let mut dispatches = Vec::with_capacity(count);
    for _ in 0..count {
        let kernel = exec.kernel(kernel_abi.symbol())?;
        let mut karg = pool.allocate_for(kernel.metadata())?;
        kernel_abi.populate_kernarg(karg.as_mut_bytes(), out_addr, n, grid_blocks, block)?;
        let geometry = LaunchGeometry::new([workitems, 1, 1], [block as u16, 1, 1])?;
        dispatches.push(RecordedDispatch::new(0, kernel, geometry, karg)?);
    }
    let range = device.queue_size_range();
    let want = ((count as u32).saturating_add(16)).next_power_of_two();
    let queue_size = want.clamp(*range.start(), *range.end());
    let mut graph = SingleQueueBatchGraph::create(
        device,
        queue_size,
        dispatches,
        BatchFencePolicy::AgentEveryInternalDispatch,
    )?;

    let _ = unsafe { graph.replay_and_wait()? };
    let expected = (count as f32).to_bits();
    let mut observed = vec![0_u8; out.len()];
    unsafe { out.copy_to_host(&mut observed)? };
    let (chunks, _) = observed.as_chunks::<4>();
    let correct = chunks.iter().all(|b| u32::from_le_bytes(*b) == expected);

    for _ in 0..warmup {
        let _ = unsafe { graph.replay_and_wait()? };
    }
    let mut ts = Vec::with_capacity(reps);
    for _ in 0..reps {
        let t0 = Instant::now();
        let _ = unsafe { graph.replay_and_wait()? };
        ts.push(t0.elapsed().as_secs_f64() * 1e6);
    }
    Ok((median(ts) / count as f64, correct))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let hsaco = std::env::var("GMB_HSACO").map_err(|_| "set GMB_HSACO to gmb_noop.co")?;
    let n: u32 = std::env::var("GMB_N")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(256);
    let reps = env_usize("GMB_REPS", 50);
    let warmup = env_usize("GMB_WARMUP", 10);
    let kernel_abi = GmbKernelAbi::from_env()?;
    let counts: Vec<usize> = std::env::var("GMB_COUNTS")
        .unwrap_or_else(|_| "1,50,200,941".to_owned())
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    let runtime = Runtime::initialize(load_symbols()?)?;
    let device = runtime.select_gpu(GpuSelector::Ordinal(0))?;
    if !device.name().starts_with("gfx12") {
        return Err(format!("gmb_floor requires gfx12, selected {}", device.name()).into());
    }
    let pool = KernargPool::discover(&device)?;
    let device_pool = DevicePool::discover(&device)?;
    let code: Arc<[u8]> = std::fs::read(&hsaco)?.into();
    let exec = Executable::load(&device, code)?;

    println!(
        "gmb_floor (pure Rust, no FFI)  abi={kernel_abi:?} n={n} block=256 reps={reps} (median)"
    );
    println!(
        "{:>6} | {:>8} | {:>24} | {:>15} | {:>7}",
        "count", "encoder", "boundary", "host us/disp", "correct",
    );
    println!("{}", "-".repeat(75));
    let pf = |b: bool| if b { "PASS" } else { "FAIL" };
    // The first two rows preserve the late legacy experiment and the working
    // Hipfire policy. The next three isolate GCR policy. The final rows prove
    // that completion ordering and cache visibility are independently needed.
    let modes = [
        Mode {
            id: "legacy",
            encoder: "legacy",
            boundary: "current-seq 0x10180",
            wait: true,
            acquire: Some(Gfx12RmwAcquirePolicy::CurrentSequential),
            stateful: false,
        },
        Mode {
            id: "current",
            encoder: "stateful",
            boundary: "current-seq 0x10180",
            wait: true,
            acquire: Some(Gfx12RmwAcquirePolicy::CurrentSequential),
            stateful: true,
        },
        Mode {
            id: "radv",
            encoder: "stateful",
            boundary: "RADV-global 0x0c380",
            wait: true,
            acquire: Some(Gfx12RmwAcquirePolicy::RadvGlobal),
            stateful: true,
        },
        Mode {
            id: "same-l1",
            encoder: "stateful",
            boundary: "same-agent-L1 0x00380",
            wait: true,
            acquire: Some(Gfx12RmwAcquirePolicy::SameAgentParallelL1),
            stateful: true,
        },
        Mode {
            id: "same-l0",
            encoder: "stateful",
            boundary: "same-agent-L0 0x00180",
            wait: true,
            acquire: Some(Gfx12RmwAcquirePolicy::SameAgentParallelL0),
            stateful: true,
        },
        Mode {
            id: "hip-llvm-vmem",
            encoder: "stateful",
            boundary: "HIP/LLVM-VMEM 0x00300",
            wait: true,
            acquire: Some(Gfx12RmwAcquirePolicy::HipLlvmVmemL1),
            stateful: true,
        },
        Mode {
            id: "wait",
            encoder: "stateful",
            boundary: "wait only",
            wait: true,
            acquire: None,
            stateful: true,
        },
        Mode {
            id: "acquire",
            encoder: "stateful",
            boundary: "acquire only",
            wait: false,
            acquire: Some(Gfx12RmwAcquirePolicy::CurrentSequential),
            stateful: true,
        },
        Mode {
            id: "none",
            encoder: "stateful",
            boundary: "none",
            wait: false,
            acquire: None,
            stateful: true,
        },
    ];
    let only = std::env::var("GMB_ONLY").ok();
    let profile_only = std::env::var_os("GMB_PROFILE_ONLY").is_some();
    if !profile_only {
        for &count in &counts {
            for mode in modes {
                if only.as_deref().is_some_and(|id| id != mode.id) {
                    continue;
                }
                let result = measure(
                    &device,
                    &device_pool,
                    &pool,
                    &exec,
                    n,
                    count,
                    reps,
                    warmup,
                    mode,
                    kernel_abi,
                )?;
                println!(
                    "{count:>6} | {:>8} | {:>24} | {:>15.4} | {:>7}",
                    mode.encoder,
                    mode.boundary,
                    result.0,
                    pf(result.1),
                );
            }
            println!("{}", "-".repeat(75));
        }
    }
    println!();
    println!("redline retained-PM4 GPU-span (one profiled vendor AQL packet):");
    let profile_mode = only
        .as_deref()
        .and_then(|id| modes.into_iter().find(|mode| mode.id == id))
        .or_else(|| modes.into_iter().find(|mode| mode.id == "radv"))
        .expect("profile mode");
    for &count in &counts {
        let (gpu, ok) =
            if std::env::var("GMB_TIMING_MODE").as_deref() == Ok("independent_throughput") {
                measure_profiled_pm4_independent(
                    &device,
                    &device_pool,
                    &pool,
                    &exec,
                    n,
                    count,
                    reps,
                    warmup,
                    kernel_abi,
                )?
            } else {
                measure_profiled_pm4(
                    &device,
                    &device_pool,
                    &pool,
                    &exec,
                    n,
                    count,
                    reps,
                    warmup,
                    profile_mode,
                    kernel_abi,
                )?
            };
        println!(
            "  count={count:>4}   {gpu:>10.4} µs/dispatch   [{}]",
            pf(ok)
        );
    }
    if !profile_only {
        println!();
        println!(
            "redline GPU-span (AQL AgentEveryInternalDispatch, GPU timestamps — GPU exec, no host):"
        );
        for &count in &counts {
            if count < 2 {
                println!("  count={count:>4}   (n/a — profiling needs >=2 dispatches)");
                continue;
            }
            let (gpu, ok) = measure_gpuspan(
                &device,
                &device_pool,
                &pool,
                &exec,
                n,
                count,
                reps,
                warmup,
                kernel_abi,
            )?;
            println!(
                "  count={count:>4}   {gpu:>10.4} µs/dispatch   [{}]",
                pf(ok)
            );
        }
    }
    if std::env::var_os("GMB_VARIANTS").is_some() {
        // Machine-parseable host-latency rows for the redline submission
        // variants, using the conservative correctness-passing fence.
        let cur = modes
            .into_iter()
            .find(|m| m.id == "current")
            .expect("current mode");
        println!();
        for &count in &counts {
            let (serial, s_ok) = measure_serial(
                &device,
                &device_pool,
                &pool,
                &exec,
                n,
                count,
                reps,
                warmup,
                cur,
                kernel_abi,
            )?;
            let (pm4, p_ok) = measure(
                &device,
                &device_pool,
                &pool,
                &exec,
                n,
                count,
                reps,
                warmup,
                cur,
                kernel_abi,
            )?;
            let (aql, a_ok) = if count >= 2 {
                measure_aql_host(
                    &device,
                    &device_pool,
                    &pool,
                    &exec,
                    n,
                    count,
                    reps,
                    warmup,
                    kernel_abi,
                )?
            } else {
                (f64::NAN, true)
            };
            println!(
                "VARIANT count={count} serial={serial:.4} aql={aql:.4} pm4={pm4:.4} \
                 serial_ok={} aql_ok={} pm4_ok={}",
                s_ok as u8, a_ok as u8, p_ok as u8
            );
        }
    }
    Ok(())
}
