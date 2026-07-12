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
    wait: bool,
    acquire: bool,
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
        if i + 1 < count {
            if wait {
                cmd.wait_compute_idle();
            }
            if acquire {
                cmd.acquire_inter_node_gfx12();
            }
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
    wait: bool,
    acquire: bool,
) -> Result<(f64, bool), Box<dyn std::error::Error>> {
    let mut out = pool.allocate_executable_bytes((n as usize) * 4)?;
    out.as_mut_bytes().fill(0);
    let out_addr = out.address() as usize as u64;
    let (mut ib, _karg) = build_ib(device, pool, exec, n, count, wait, acquire, out_addr)?;

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
        "{:>6} | {:>23} | {:>23} | {:>23}",
        "count", "conservative µs/disp", "tuned (acquire only)", "aggressive (no fence)"
    );
    println!("{}", "-".repeat(86));
    let pf = |b: bool| if b { "PASS" } else { "FAIL" };
    // (wait_compute_idle, acquire_inter_node) per dependency boundary
    let modes = [(true, true), (false, true), (false, false)];
    for &count in &counts {
        let mut r = Vec::with_capacity(modes.len());
        for (wait, acquire) in modes {
            r.push(measure(&device, &pool, &exec, n, count, reps, warmup, wait, acquire)?);
        }
        println!(
            "{count:>6} | {:>15.4} [{:>4}] | {:>15.4} [{:>4}] | {:>15.4} [{:>4}]",
            r[0].0, pf(r[0].1), r[1].0, pf(r[1].1), r[2].0, pf(r[2].1),
        );
    }
    Ok(())
}
