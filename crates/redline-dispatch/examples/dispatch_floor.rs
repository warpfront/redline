// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Kaden Schutt

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

use redline_dispatch::aql::{
    BatchFencePolicy, Executable, GpuDevice, GpuSelector, KernargPool, LaunchGeometry,
    RecordedDispatch, Runtime, SingleQueueBatchGraph, load_symbols,
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
    Ok(())
}
