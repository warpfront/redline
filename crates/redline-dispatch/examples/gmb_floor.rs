// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! Pure-Rust #6409 dispatch-floor: the SAME `gmb_noop_kernel` serial-latency
//! chain as `examples/dispatch-floor-6409`, driven straight through
//! `SingleQueuePm4Ib` with NO Python/C FFI in the timed path. This isolates the
//! retained-PM4 replay cost from binding overhead.
//!
//! Two arms:
//!   * conservative — a correct dependency boundary between dispatches
//!     (`wait_compute_idle` + inter-node acquire), the only correct mode for a
//!     non-atomic read-modify-write chain; must leave every element == count.
//!   * aggressive — no inter-dispatch fence (races on the shared buffer, so it
//!     FAILS correctness); reported only as the fence-free timing ceiling.
//!
//! Env: `GMB_HSACO` (gmb_noop.co, required), `GMB_N` (256), `GMB_COUNTS`
//! (1,50,200,941), `GMB_REPS` (50), `GMB_WARMUP` (10). Pick the GPU with
//! `ROCR_VISIBLE_DEVICES`.

use std::sync::Arc;
use std::time::Instant;

use redline_dispatch::aql::{
    Executable, Gfx12Pm4CommandBuffer, GpuDevice, GpuSelector, KernargPool, LaunchGeometry, Runtime,
    SingleQueuePm4Ib, load_symbols,
};

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

fn env_usize(k: &str, d: usize) -> usize {
    std::env::var(k).ok().and_then(|s| s.parse().ok()).unwrap_or(d)
}

/// Build a retained PM4 IB of `count` gmb_noop dispatches against `out`.
/// `fence` inserts the correct RMW dependency boundary between dispatches.
fn build_ib(
    device: &GpuDevice,
    pool: &KernargPool,
    exec: &Executable,
    n: u32,
    count: usize,
    fence: bool,
    out_addr: u64,
) -> Result<(SingleQueuePm4Ib, redline_dispatch::aql::KernargBuffer), Box<dyn std::error::Error>> {
    let block = 256u32;
    let grid_blocks = n.div_ceil(block);
    let workitems = grid_blocks * block;
    let kernel = exec.kernel("gmb_noop_kernel.kd")?;
    let mut karg = pool.allocate_for(kernel.metadata())?;
    {
        let d = karg.as_mut_bytes();
        d.fill(0);
        d[..8].copy_from_slice(&out_addr.to_le_bytes());
        d[8..12].copy_from_slice(&n.to_le_bytes());
    }
    let mut cmd = Gfx12Pm4CommandBuffer::new();
    for i in 0..count {
        let geometry = LaunchGeometry::new([workitems, 1, 1], [block as u16, 1, 1])?;
        cmd.dispatch(&kernel, geometry, 0, karg.address())?;
        if fence && i + 1 < count {
            cmd.wait_compute_idle();
            cmd.acquire_inter_node_gfx12();
        }
    }
    let ib = SingleQueuePm4Ib::create(device, pool, &cmd)?;
    Ok((ib, karg))
}

fn measure(
    device: &GpuDevice,
    pool: &KernargPool,
    exec: &Executable,
    n: u32,
    count: usize,
    reps: usize,
    warmup: usize,
    fence: bool,
) -> Result<(f64, bool), Box<dyn std::error::Error>> {
    let mut out = pool.allocate_executable_bytes((n as usize) * 4)?;
    out.as_mut_bytes().fill(0);
    let out_addr = out.address() as usize as u64;
    let (mut ib, _karg) = build_ib(device, pool, exec, n, count, fence, out_addr)?;

    // correctness: one replay from zero -> element 0 must be count
    unsafe { ib.replay_and_wait()? };
    let v0 = f32::from_le_bytes(out.as_mut_bytes()[..4].try_into().unwrap());
    let correct = v0 == count as f32;

    for _ in 0..warmup {
        unsafe { ib.replay_and_wait()? };
    }
    let mut ts = Vec::with_capacity(reps);
    for _ in 0..reps {
        let t0 = Instant::now();
        unsafe { ib.replay_and_wait()? };
        ts.push(t0.elapsed().as_secs_f64() * 1e6);
    }
    Ok((median(ts) / count as f64, correct))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let hsaco = std::env::var("GMB_HSACO").map_err(|_| "set GMB_HSACO to gmb_noop.co")?;
    let n: u32 = std::env::var("GMB_N").ok().and_then(|s| s.parse().ok()).unwrap_or(256);
    let reps = env_usize("GMB_REPS", 50);
    let warmup = env_usize("GMB_WARMUP", 10);
    let counts: Vec<usize> = std::env::var("GMB_COUNTS")
        .unwrap_or_else(|_| "1,50,200,941".to_owned())
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    let runtime = Runtime::initialize(load_symbols()?)?;
    let device = runtime.select_gpu(GpuSelector::Ordinal(0))?;
    let pool = KernargPool::discover(&device)?;
    let code: Arc<[u8]> = std::fs::read(&hsaco)?.into();
    let exec = Executable::load(&device, code)?;

    println!("gmb_floor (pure Rust, no FFI)  n={n} block=256 reps={reps} (median)");
    println!(
        "{:>6} | {:>26} | {:>26}",
        "count", "PM4 conservative µs/disp", "PM4 aggressive µs/disp"
    );
    println!("{}", "-".repeat(66));
    for &count in &counts {
        let (cons, cons_ok) = measure(&device, &pool, &exec, n, count, reps, warmup, true)?;
        let (aggr, aggr_ok) = measure(&device, &pool, &exec, n, count, reps, warmup, false)?;
        println!(
            "{count:>6} | {cons:>18.4} [{:>4}] | {aggr:>18.4} [{:>4}]",
            if cons_ok { "PASS" } else { "FAIL" },
            if aggr_ok { "PASS" } else { "FAIL" },
        );
    }
    Ok(())
}
