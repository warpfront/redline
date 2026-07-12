// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! Dispatch-floor microbench (real GPU).
//!
//! Replays the SAME retained batch of `N` tiny kernel dispatches under two fence
//! policies and reports the GPU-timed span of each:
//!
//! * `SystemEveryDispatch` — a system-scope acquire/release fence on *every*
//!   dispatch. This is the HIP / HipGraph per-dispatch floor: each launch
//!   invalidates+flushes cache system-wide.
//! * `BoundarySerialized` — Redline's minimal fences: only where the dispatch
//!   order actually requires visibility.
//!
//! Same kernels, same retained indirect buffer, same queue — only the fence
//! policy differs. The ratio is the dispatch-floor win, measured on-device.
//!
//! Env:
//!   `REDLINE_FLOOR_HSACO`   path to the code object (required)
//!   `REDLINE_FLOOR_SYMBOL`  dispatched symbol (default `floor_k.kd`)
//!   `REDLINE_FLOOR_N`       dispatches per replay (default 64)
//!   `REDLINE_FLOOR_M`       timed replays (default 200)
//!   `REDLINE_FLOOR_WARMUP`  warmup replays (default 20)
//!
//! Select the GPU with `ROCR_VISIBLE_DEVICES` before running.

use std::sync::Arc;
use std::time::Instant;

use redline_dispatch::aql::{
    BatchFencePolicy, Executable, Gfx12Pm4CommandBuffer, GpuDevice, GpuSelector, KernargPool,
    LaunchGeometry, RecordedDispatch, Runtime, SingleQueueBatchGraph, SingleQueuePm4Ib, load_symbols,
};

fn env_usize(key: &str, default: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = v.len();
    if n == 0 {
        return 0.0;
    }
    if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    }
}

fn build_graph(
    device: &GpuDevice,
    exec: &Executable,
    symbol: &str,
    n: usize,
    policy: BatchFencePolicy,
) -> Result<SingleQueueBatchGraph, Box<dyn std::error::Error>> {
    let pool = KernargPool::discover(device)?;
    let geometry = LaunchGeometry::new([1, 1, 1], [1, 1, 1])?;
    let mut dispatches = Vec::with_capacity(n);
    for _ in 0..n {
        let kernel = exec.kernel(symbol)?;
        let kernarg = pool.allocate_for(kernel.metadata())?;
        dispatches.push(RecordedDispatch::new(0, kernel, geometry, kernarg)?);
    }
    // Size the queue to hold N dispatches plus fence/barrier/signal packets.
    let range = device.queue_size_range();
    let want = ((n as u32).saturating_add(16)).next_power_of_two();
    let queue_size = want.clamp(*range.start(), *range.end());
    Ok(SingleQueueBatchGraph::create(
        device, queue_size, dispatches, policy,
    )?)
}

fn measure(
    device: &GpuDevice,
    exec: &Executable,
    symbol: &str,
    n: usize,
    m: usize,
    warmup: usize,
    policy: BatchFencePolicy,
) -> Result<f64, Box<dyn std::error::Error>> {
    let mut graph = build_graph(device, exec, symbol, n, policy)?;
    for _ in 0..warmup {
        // SAFETY: the recorded batch owns its kernels, kernargs, and queue for
        // the lifetime of `graph`; no external pointers are referenced.
        unsafe { graph.replay_and_wait()? };
    }
    let mut spans = Vec::with_capacity(m);
    for _ in 0..m {
        let timing = unsafe { graph.replay_and_wait()? };
        spans.push(timing.span_microseconds());
    }
    Ok(median(spans))
}

/// The PM4 champion: lower the same N dispatches into ONE retained GFX12 PM4
/// indirect buffer (`SingleQueuePm4Ib`) and replay it. Unlike the general
/// `RecordedGraph` (which re-arms N per-node completion signals per replay),
/// this tight single-IB path resets only ONE completion signal per replay, so
/// host latency reflects submission + GPU work, not signal re-arm.
///
/// `serialize` inserts a compute-idle wait between dispatches (conservative /
/// dependency-ordered); `false` leaves them back-to-back (aggressive / minimal).
/// Host-timed (submit -> wait), matched to the hipGraph host-latency baseline.
fn measure_pm4_ib_host(
    device: &GpuDevice,
    exec: &Executable,
    symbol: &str,
    n: usize,
    m: usize,
    warmup: usize,
    serialize: bool,
) -> Result<f64, Box<dyn std::error::Error>> {
    let pool = KernargPool::discover(device)?;
    let kernel = exec.kernel(symbol)?;
    let kernarg = pool.allocate_for(kernel.metadata())?;
    let mut cmd = Gfx12Pm4CommandBuffer::new();
    for i in 0..n {
        let geometry = LaunchGeometry::new([1, 1, 1], [1, 1, 1])?;
        cmd.dispatch(&kernel, geometry, 0, kernarg.address())?;
        if serialize && i + 1 < n {
            cmd.wait_compute_idle();
        }
    }
    let mut ib = SingleQueuePm4Ib::create(device, &pool, &cmd)?;
    for _ in 0..warmup {
        // SAFETY: no-op kernels reference no external pointees; the retained IB,
        // code object, and kernarg stay live for the lifetime of `ib`.
        unsafe { ib.replay_and_wait()? };
    }
    let mut per = Vec::with_capacity(m);
    for _ in 0..m {
        let t0 = Instant::now();
        unsafe { ib.replay_and_wait()? };
        per.push(t0.elapsed().as_secs_f64() * 1e6);
    }
    Ok(median(per))
}

/// Correctness gate: build a PM4 IB of N dispatches of an atomic-increment
/// kernel (`ctr_k(unsigned int*)`), replay once, and read the counter back. It
/// must equal N iff every dispatch executed AND the replay's completion waited
/// for wave retirement (else the host-latency numbers are measuring skipped or
/// unfinished dispatches). Returns (observed, expected).
fn verify_pm4_execution(
    device: &GpuDevice,
    exec: &Executable,
    symbol: &str,
    n: usize,
    serialize: bool,
) -> Result<(u32, u32), Box<dyn std::error::Error>> {
    let pool = KernargPool::discover(device)?;
    let kernel = exec.kernel(symbol)?;
    let mut counter = pool.allocate_executable_bytes(4)?;
    counter.as_mut_bytes().fill(0);
    let counter_addr = counter.address() as usize as u64;
    let mut kernarg = pool.allocate_for(kernel.metadata())?;
    kernarg.as_mut_bytes()[..8].copy_from_slice(&counter_addr.to_le_bytes());

    let mut cmd = Gfx12Pm4CommandBuffer::new();
    for i in 0..n {
        let geometry = LaunchGeometry::new([1, 1, 1], [1, 1, 1])?;
        cmd.dispatch(&kernel, geometry, 0, kernarg.address())?;
        if serialize && i + 1 < n {
            cmd.wait_compute_idle(); // serialize the atomic increments
        }
    }
    let mut ib = SingleQueuePm4Ib::create(device, &pool, &cmd)?;
    // SAFETY: kernarg points at `counter`, which outlives `ib`.
    unsafe { ib.replay_and_wait()? };
    let observed = u32::from_le_bytes(counter.as_mut_bytes()[..4].try_into().unwrap());
    Ok((observed, n as u32))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let hsaco = std::env::var("REDLINE_FLOOR_HSACO")
        .map_err(|_| "set REDLINE_FLOOR_HSACO to the code-object path")?;
    let symbol = std::env::var("REDLINE_FLOOR_SYMBOL").unwrap_or_else(|_| "floor_k.kd".to_owned());
    let n = env_usize("REDLINE_FLOOR_N", 64);
    let m = env_usize("REDLINE_FLOOR_M", 200);
    let warmup = env_usize("REDLINE_FLOOR_WARMUP", 20);

    let runtime = Runtime::initialize(load_symbols()?)?;
    let device = runtime.select_gpu(GpuSelector::Ordinal(0))?;
    let code: Arc<[u8]> = std::fs::read(&hsaco)?.into();
    let exec = Executable::load(&device, code)?;

    println!("dispatch-floor: N={n} dispatches/replay, M={m} timed replays, {warmup} warmup");

    // The full fence-policy spectrum, most-conservative first. The floor is a
    // system-scope fence on every dispatch (HIP default); BoundarySerialized is
    // Redline's safe policy for a dependency chain (decode); BoundaryIndependent
    // is the aggressive policy — correct ONLY when dispatches are genuinely
    // independent (disjoint writable state), as the no-op kernel here is.
    let policies = [
        ("SystemEveryDispatch      (HIP per-dispatch floor)", BatchFencePolicy::SystemEveryDispatch),
        ("SystemAcquireAgentRelease", BatchFencePolicy::SystemAcquireAgentRelease),
        ("AgentEveryInternalDispatch", BatchFencePolicy::AgentEveryInternalDispatch),
        ("BoundarySerialized       (redline safe / decode)", BatchFencePolicy::BoundarySerialized),
        ("BoundaryIndependent      (redline aggressive / indep-only)", BatchFencePolicy::BoundaryIndependent),
    ];

    let mut results = Vec::with_capacity(policies.len());
    for (name, policy) in policies {
        results.push((name, measure(&device, &exec, &symbol, n, m, warmup, policy)?));
    }
    let floor = results[0].1;

    println!("  {:<52} {:>10} {:>10} {:>9}", "policy", "us", "us/disp", "vs floor");
    for (name, us) in &results {
        let per = us / n as f64;
        let ratio = if *us > 0.0 { floor / us } else { 0.0 };
        println!("  {name:<52} {us:>10.3} {per:>10.4} {ratio:>8.3}x");
    }

    // Correctness gate: prove the PM4 IB actually executes N dispatches to
    // completion before trusting its host-latency numbers.
    if let Ok(vpath) = std::env::var("REDLINE_FLOOR_VERIFY_HSACO") {
        let vsym =
            std::env::var("REDLINE_FLOOR_VERIFY_SYMBOL").unwrap_or_else(|_| "ctr_k.kd".to_owned());
        let vcode: Arc<[u8]> = std::fs::read(&vpath)?.into();
        let vexec = Executable::load(&device, vcode)?;
        for (label, serialize) in [("serialized", true), ("minimal-fence", false)] {
            let (observed, expected) = verify_pm4_execution(&device, &vexec, &vsym, n, serialize)?;
            let status = if observed == expected { "PASS" } else { "FAIL" };
            println!(
                "  PM4 correctness gate ({label:<13}): counter = {observed} / {expected}  [{status}]"
            );
        }
    }

    // The champion: single-stream retained PM4 indirect buffer (SingleQueuePm4Ib),
    // O(1) signal reset per replay. Host-timed (submit -> wait), matched to the
    // hipGraph host-latency baseline. Conservative serializes each dispatch;
    // aggressive leaves them minimal/back-to-back.
    let pm4_cons = measure_pm4_ib_host(&device, &exec, &symbol, n, m, warmup, true)?;
    let pm4_aggr = measure_pm4_ib_host(&device, &exec, &symbol, n, m, warmup, false)?;
    println!(
        "  {:<52} {:>10.3} {:>10.4}   host latency/replay",
        "PM4 retained IB — conservative (serialized)", pm4_cons, pm4_cons / n as f64
    );
    println!(
        "  {:<52} {:>10.3} {:>10.4}   host latency/replay",
        "PM4 retained IB — aggressive (minimal fence)", pm4_aggr, pm4_aggr / n as f64
    );
    Ok(())
}
