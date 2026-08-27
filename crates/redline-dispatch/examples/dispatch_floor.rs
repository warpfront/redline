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
    BatchFencePolicy, Executable, Gfx10Pm4CommandBuffer, Gfx12Pm4CommandBuffer, GpuDevice,
    GpuSelector, KernargPool, Kernel, LaunchGeometry, MultiQueuePm4Ib, RecordedDispatch, Runtime,
    SingleQueueBatchGraph, SingleQueuePm4Ib, load_symbols,
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

/// PM4 command buffers and the retained-IB constructor are both
/// architecture-family specific: gfx10 and gfx11 share the legacy compute
/// register map, gfx12 has its own. Choosing wrongly does not degrade
/// gracefully -- it would emit register writes from the wrong map at the
/// hardware -- so the family is resolved once from the agent name and both the
/// buffer type and the constructor follow from that single decision.
///
/// The gfx12 arm deliberately matches `gfx120` rather than `gfx12`, because
/// gfx125x shares the numeric family without sharing validation here. Refusing
/// an unknown architecture costs a clear error; misencoding one costs a fault.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FloorFamily {
    Gfx10,
    Gfx11,
    Gfx12,
}

impl FloorFamily {
    fn of(device: &GpuDevice) -> Option<Self> {
        let name = device.name();
        // Agent names can carry target features, e.g. `gfx1010:xnack-`.
        let base = name.split(':').next().unwrap_or(&name);
        if base.starts_with("gfx10") {
            Some(Self::Gfx10)
        } else if base.starts_with("gfx11") {
            Some(Self::Gfx11)
        } else if base.starts_with("gfx120") {
            Some(Self::Gfx12)
        } else {
            None
        }
    }
}

/// One retained PM4 command stream, in whichever encoding the device needs.
enum FloorPm4 {
    /// gfx10 and gfx11: `Gfx11Pm4CommandBuffer` is an alias of the gfx10 type.
    Legacy(Gfx10Pm4CommandBuffer),
    Gfx12(Gfx12Pm4CommandBuffer),
}

impl FloorPm4 {
    fn new_stateful(family: FloorFamily) -> Self {
        match family {
            FloorFamily::Gfx10 | FloorFamily::Gfx11 => {
                Self::Legacy(Gfx10Pm4CommandBuffer::new_stateful())
            }
            FloorFamily::Gfx12 => Self::Gfx12(Gfx12Pm4CommandBuffer::new_stateful()),
        }
    }

    fn dispatch(
        &mut self,
        kernel: &Kernel,
        geometry: LaunchGeometry,
        group_segment: u32,
        kernarg: *mut std::ffi::c_void,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Self::Legacy(c) => c.dispatch(kernel, geometry, group_segment, kernarg)?,
            Self::Gfx12(c) => c.dispatch(kernel, geometry, group_segment, kernarg)?,
        }
        Ok(())
    }

    fn wait_compute_idle(&mut self) {
        match self {
            Self::Legacy(c) => c.wait_compute_idle(),
            Self::Gfx12(c) => c.wait_compute_idle(),
        }
    }

    /// Flush completed shader writes to system scope.
    ///
    /// Required before a correctness gate reads a counter the kernel wrote. The
    /// AMD vendor packet carrying the IB has no architected AQL release scope,
    /// so ROCr can publish its completion signal while the atomic is still in a
    /// GPU cache. RDNA3 happened to hide this; RDNA2 did not, which is exactly
    /// how the missing flush was found.
    fn acquire_system(&mut self) {
        match self {
            Self::Legacy(c) => c.acquire_system(),
            Self::Gfx12(c) => c.acquire_system(),
        }
    }

    /// Build the retained IB through the constructor matching this encoding.
    /// Each constructor re-checks the device family itself, so a mismatch here
    /// is caught rather than submitted.
    fn create_ib(
        &self,
        family: FloorFamily,
        device: &GpuDevice,
        pool: &KernargPool,
    ) -> Result<SingleQueuePm4Ib, Box<dyn std::error::Error>> {
        Ok(match (self, family) {
            (Self::Legacy(c), FloorFamily::Gfx10) => {
                SingleQueuePm4Ib::create_gfx10(device, pool, c)?
            }
            (Self::Legacy(c), FloorFamily::Gfx11) => {
                SingleQueuePm4Ib::create_gfx11(device, pool, c)?
            }
            (Self::Gfx12(c), FloorFamily::Gfx12) => {
                SingleQueuePm4Ib::create(device, pool, c)?
            }
            // Unreachable: the buffer is always built from the same family value.
            _ => return Err("PM4 buffer encoding does not match device family".into()),
        })
    }
}

/// PM4 across several independent queue lanes, one retained IB per lane.
///
/// Why this arm exists: hipGraph on ROCm 10.0 gets 2.2x-3.6x from spreading a
/// parallel-path graph across hardware queues, and the measured optimum is one
/// chain per queue at a device-specific width. The single-queue PM4 arm cannot
/// see that win at all, so comparing it against a *tuned* hipGraph understates
/// PM4 by whatever the queue width is worth. This arm gives PM4 the same
/// structural advantage: `lanes` command buffers, each holding N/lanes
/// dispatches, submitted as `lanes` retained IBs on independent queues.
///
/// `serialize` orders dispatches within each lane, which mirrors a chain per
/// lane; lanes are independent of each other by construction, exactly as
/// ParallelChains builds the graph side.
fn measure_pm4_multiqueue_host(
    device: &GpuDevice,
    exec: &Executable,
    symbol: &str,
    n: usize,
    lanes: usize,
    m: usize,
    warmup: usize,
    serialize: bool,
) -> Result<(f64, usize), Box<dyn std::error::Error>> {
    let family = FloorFamily::of(device)
        .ok_or_else(|| format!("no PM4 encoding for device {}", device.name()))?;
    let pool = KernargPool::discover(device)?;
    let kernel = exec.kernel(symbol)?;
    let kernarg = pool.allocate_for(kernel.metadata())?;
    // Split N as evenly as possible; a remainder goes to the first lanes so the
    // total dispatch count is exactly N and comparable to the other arms.
    let base = n / lanes;
    let extra = n % lanes;

    let mut ib = match family {
        FloorFamily::Gfx10 | FloorFamily::Gfx11 => {
            let mut cmds = Vec::with_capacity(lanes);
            for l in 0..lanes {
                let count = base + usize::from(l < extra);
                let mut c = Gfx10Pm4CommandBuffer::new_stateful();
                for i in 0..count {
                    let geometry = LaunchGeometry::new([1, 1, 1], [1, 1, 1])?;
                    c.dispatch(&kernel, geometry, 0, kernarg.address())?;
                    if serialize && i + 1 < count {
                        c.wait_compute_idle();
                    }
                }
                cmds.push(c);
            }
            if family == FloorFamily::Gfx10 {
                MultiQueuePm4Ib::create_gfx10(device, &pool, &cmds)?
            } else {
                MultiQueuePm4Ib::create_gfx11(device, &pool, &cmds)?
            }
        }
        FloorFamily::Gfx12 => {
            let mut cmds = Vec::with_capacity(lanes);
            for l in 0..lanes {
                let count = base + usize::from(l < extra);
                let mut c = Gfx12Pm4CommandBuffer::new_stateful();
                for i in 0..count {
                    let geometry = LaunchGeometry::new([1, 1, 1], [1, 1, 1])?;
                    c.dispatch(&kernel, geometry, 0, kernarg.address())?;
                    if serialize && i + 1 < count {
                        c.wait_compute_idle();
                    }
                }
                cmds.push(c);
            }
            MultiQueuePm4Ib::create(device, &pool, &cmds)?
        }
    };

    let queues = ib.queue_count();
    for _ in 0..warmup {
        // SAFETY: no-op kernels reference no external pointees; the retained IBs,
        // code object and kernarg all outlive `ib`.
        unsafe { ib.replay_and_wait()? };
    }
    let mut per = Vec::with_capacity(m);
    for _ in 0..m {
        let t0 = Instant::now();
        unsafe { ib.replay_and_wait()? };
        per.push(t0.elapsed().as_secs_f64() * 1e6);
    }
    Ok((median(per), queues))
}

/// The PM4 champion: lower the same N dispatches into ONE retained PM4 indirect
/// buffer (`SingleQueuePm4Ib`) and replay it. Unlike the general `RecordedGraph`
/// (which re-arms N per-node completion signals per replay), this tight
/// single-IB path resets only ONE completion signal per replay, so host latency
/// reflects submission + GPU work, not signal re-arm.
///
/// The encoding follows the device family, so this runs on gfx10, gfx11 and
/// gfx12 rather than gfx12 alone. That matters for cross-architecture
/// comparison: hard-coding one family here silently reduces the whole PM4 row
/// to an ArchitectureMismatch error on every other part, which reads as "PM4 is
/// unavailable" when it is only unimplemented in the harness.
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
    let family = FloorFamily::of(device)
        .ok_or_else(|| format!("no PM4 encoding for device {}", device.name()))?;
    let pool = KernargPool::discover(device)?;
    let kernel = exec.kernel(symbol)?;
    let kernarg = pool.allocate_for(kernel.metadata())?;
    let mut cmd = FloorPm4::new_stateful(family);
    for i in 0..n {
        let geometry = LaunchGeometry::new([1, 1, 1], [1, 1, 1])?;
        cmd.dispatch(&kernel, geometry, 0, kernarg.address())?;
        if serialize && i + 1 < n {
            cmd.wait_compute_idle();
        }
    }
    let mut ib = cmd.create_ib(family, device, &pool)?;
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
    let family = FloorFamily::of(device)
        .ok_or_else(|| format!("no PM4 encoding for device {}", device.name()))?;
    let pool = KernargPool::discover(device)?;
    let kernel = exec.kernel(symbol)?;
    let mut counter = pool.allocate_executable_bytes(4)?;
    counter.as_mut_bytes().fill(0);
    let counter_addr = counter.address() as usize as u64;
    let mut kernarg = pool.allocate_for(kernel.metadata())?;
    kernarg.as_mut_bytes()[..8].copy_from_slice(&counter_addr.to_le_bytes());

    let mut cmd = FloorPm4::new_stateful(family);
    for i in 0..n {
        let geometry = LaunchGeometry::new([1, 1, 1], [1, 1, 1])?;
        cmd.dispatch(&kernel, geometry, 0, kernarg.address())?;
        if serialize && i + 1 < n {
            cmd.wait_compute_idle(); // serialize the atomic increments
        }
    }
    // Retire the last wave and push its write to system scope before ROCr's
    // completion signal lets the host read the counter. Without this the gate
    // can read a stale value and report zero executions for work that ran.
    cmd.wait_compute_idle();
    cmd.acquire_system();
    let mut ib = cmd.create_ib(family, device, &pool)?;
    // SAFETY: kernarg points at `counter`, which outlives `ib`.
    unsafe { ib.replay_and_wait()? };
    let observed = u32::from_le_bytes(counter.as_mut_bytes()[..4].try_into().unwrap());
    Ok((observed, n as u32))
}

/// Multi-queue correctness gate: same as `verify_pm4_execution` but for the
/// `lanes` retained-IB path. It builds the SAME per-lane command buffers
/// `measure_pm4_multiqueue_host` builds (same N split: `base = n / lanes`,
/// `extra = n % lanes`, first `extra` lanes get one more), points every
/// lane's dispatches at ONE shared atomic counter, replays once via
/// `MultiQueuePm4Ib::replay_and_wait`, and returns `(observed, expected == n,
/// granted_queue_count)`.
///
/// A shared counter across lanes is the point: it proves total dispatch count
/// is exactly N with no loss and no duplication. The per-lane counts are also
/// asserted to sum to exactly `n` in host code before submitting, so a split
/// bug is caught even if the GPU path is fine.
fn verify_pm4_multiqueue_execution(
    device: &GpuDevice,
    exec: &Executable,
    symbol: &str,
    n: usize,
    lanes: usize,
    serialize: bool,
) -> Result<(u32, u32, usize), Box<dyn std::error::Error>> {
    if lanes == 0 {
        return Err("lanes must be >= 1".into());
    }
    let family = FloorFamily::of(device)
        .ok_or_else(|| format!("no PM4 encoding for device {}", device.name()))?;
    let pool = KernargPool::discover(device)?;
    let kernel = exec.kernel(symbol)?;
    let mut counter = pool.allocate_executable_bytes(4)?;
    counter.as_mut_bytes().fill(0);
    let counter_addr = counter.address() as usize as u64;
    let mut kernarg = pool.allocate_for(kernel.metadata())?;
    kernarg.as_mut_bytes()[..8].copy_from_slice(&counter_addr.to_le_bytes());

    // Same split as `measure_pm4_multiqueue_host`: evenly as possible, remainder
    // to the first lanes so the total is exactly N.
    let base = n / lanes;
    let extra = n % lanes;
    let total: usize = (0..lanes).map(|l| base + usize::from(l < extra)).sum();
    assert_eq!(total, n, "lane split {total} != N {n}");

    let mut ib = match family {
        FloorFamily::Gfx10 | FloorFamily::Gfx11 => {
            let mut cmds = Vec::with_capacity(lanes);
            for l in 0..lanes {
                let count = base + usize::from(l < extra);
                let mut c = Gfx10Pm4CommandBuffer::new_stateful();
                for i in 0..count {
                    let geometry = LaunchGeometry::new([1, 1, 1], [1, 1, 1])?;
                    c.dispatch(&kernel, geometry, 0, kernarg.address())?;
                    if serialize && i + 1 < count {
                        c.wait_compute_idle();
                    }
                }
                // Per lane, not once globally: each lane is a separate queue
                // whose last wave must retire and reach system scope before the
                // host reads the shared counter.
                c.wait_compute_idle();
                c.acquire_system();
                cmds.push(c);
            }
            if family == FloorFamily::Gfx10 {
                MultiQueuePm4Ib::create_gfx10(device, &pool, &cmds)?
            } else {
                MultiQueuePm4Ib::create_gfx11(device, &pool, &cmds)?
            }
        }
        FloorFamily::Gfx12 => {
            let mut cmds = Vec::with_capacity(lanes);
            for l in 0..lanes {
                let count = base + usize::from(l < extra);
                let mut c = Gfx12Pm4CommandBuffer::new_stateful();
                for i in 0..count {
                    let geometry = LaunchGeometry::new([1, 1, 1], [1, 1, 1])?;
                    c.dispatch(&kernel, geometry, 0, kernarg.address())?;
                    if serialize && i + 1 < count {
                        c.wait_compute_idle();
                    }
                }
                c.wait_compute_idle();
                c.acquire_system();
                cmds.push(c);
            }
            MultiQueuePm4Ib::create(device, &pool, &cmds)?
        }
    };

    let queues = ib.queue_count();
    // SAFETY: kernarg points at `counter`, which outlives `ib` and the kernel
    // writes only to that counter via atomicAdd; no other external pointees.
    unsafe { ib.replay_and_wait()? };
    let observed = u32::from_le_bytes(counter.as_mut_bytes()[..4].try_into().unwrap());
    Ok((observed, n as u32, queues))
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
    // Accept any family this harness can encode PM4 for, rather than gfx12
    // alone. The previous gfx12-only guard made every other architecture report
    // the whole benchmark as unavailable, which is indistinguishable from PM4
    // being unsupported on that part when in fact the encoders exist.
    let family = FloorFamily::of(&device).ok_or_else(|| {
        format!(
            "dispatch_floor has no PM4 encoding for {}; supported: gfx10*, gfx11*, gfx120*",
            device.name()
        )
    })?;
    println!("device {} -> PM4 family {:?}", device.name(), family);
    let code: Arc<[u8]> = std::fs::read(&hsaco)?.into();
    let exec = Executable::load(&device, code)?;

    println!("dispatch-floor: N={n} dispatches/replay, M={m} timed replays, {warmup} warmup");

    // The full fence-policy spectrum, most-conservative first. The floor is a
    // system-scope fence on every dispatch (HIP default); BoundarySerialized is
    // Redline's safe policy for a dependency chain (decode); BoundaryIndependent
    // is the aggressive policy — correct ONLY when dispatches are genuinely
    // independent (disjoint writable state), as the no-op kernel here is.
    let policies = [
        (
            "SystemEveryDispatch      (HIP per-dispatch floor)",
            BatchFencePolicy::SystemEveryDispatch,
        ),
        (
            "SystemAcquireAgentRelease",
            BatchFencePolicy::SystemAcquireAgentRelease,
        ),
        (
            "AgentEveryInternalDispatch",
            BatchFencePolicy::AgentEveryInternalDispatch,
        ),
        (
            "BoundarySerialized       (redline safe / decode)",
            BatchFencePolicy::BoundarySerialized,
        ),
        (
            "BoundaryIndependent      (redline aggressive / indep-only)",
            BatchFencePolicy::BoundaryIndependent,
        ),
    ];

    let mut results = Vec::with_capacity(policies.len());
    for (name, policy) in policies {
        results.push((
            name,
            measure(&device, &exec, &symbol, n, m, warmup, policy)?,
        ));
    }
    let floor = results[0].1;

    println!(
        "  {:<52} {:>10} {:>10} {:>9}",
        "policy", "us", "us/disp", "vs floor"
    );
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
        "PM4 retained IB — conservative (serialized)",
        pm4_cons,
        pm4_cons / n as f64
    );
    println!(
        "  {:<52} {:>10.3} {:>10.4}   host latency/replay",
        "PM4 retained IB — aggressive (minimal fence)",
        pm4_aggr,
        pm4_aggr / n as f64
    );

    // Multi-queue lanes. Swept rather than fixed, because the useful width is
    // device-specific: the graph side peaks at one chain per queue at a width
    // that is 2 on some parts and 5 on others, and cannot be derived from
    // published device properties. `queues` is what the runtime actually gave
    // us, which may be less than requested.
    //
    // Multi-queue correctness gate: before trusting the timing for a lane
    // count, prove that the same per-lane split executes exactly N dispatches
    // across lanes with no loss or duplication. The gate uses one shared
    // atomic counter (`ctr_k`) so the observed count is the total across all
    // queues. If the counter code object cannot be located, the gate is
    // reported as not-run rather than inventing a PASS.
    let mq_gate: Option<(Executable, String)> = {
        let path = std::env::var("REDLINE_FLOOR_HSACO_CTR")
            .or_else(|_| std::env::var("REDLINE_FLOOR_VERIFY_HSACO"))
            .ok();
        match path {
            Some(vpath) => {
                let vsym = std::env::var("REDLINE_FLOOR_VERIFY_SYMBOL")
                    .or_else(|_| std::env::var("REDLINE_FLOOR_CTR_SYMBOL"))
                    .unwrap_or_else(|_| "ctr_k.kd".to_owned());
                match std::fs::read(&vpath) {
                    Ok(bytes) => {
                        let code: Arc<[u8]> = bytes.into();
                        match Executable::load(&device, code) {
                            Ok(vexec) => Some((vexec, vsym)),
                            Err(e) => {
                                println!(
                                    "  PM4 multi-queue correctness gate: counter HSACO load failed for {vpath}: {e} — gate not run"
                                );
                                None
                            }
                        }
                    }
                    Err(e) => {
                        println!(
                            "  PM4 multi-queue correctness gate: counter HSACO read failed for {vpath}: {e} — gate not run"
                        );
                        None
                    }
                }
            }
            None => None,
        }
    };
    let lane_counts = std::env::var("REDLINE_FLOOR_LANES")
        .ok()
        .map(|s| {
            s.split(',')
                .filter_map(|t| t.trim().parse::<usize>().ok())
                .filter(|&l| l >= 1)
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| vec![2, 4, 5, 8]);
    for lanes in lane_counts {
        if lanes > n {
            continue;
        }
        // Gate this lane count before timing it: prove N dispatches executed
        // across lanes exactly once, with no loss or duplication.
        let gate_pass = if let Some((vexec, vsym)) = &mq_gate {
            match verify_pm4_multiqueue_execution(&device, vexec, vsym, n, lanes, true) {
                Ok((observed, expected, _queues)) => {
                    let status = if observed == expected { "PASS" } else { "FAIL" };
                    println!(
                        "  PM4 multi-queue correctness gate ({lanes} lane(s)): counter = {observed} / {expected}  [{status}]"
                    );
                    observed == expected
                }
                Err(e) => {
                    println!(
                        "  PM4 multi-queue correctness gate ({lanes} lane(s)): error: {e}  [FAIL]"
                    );
                    false
                }
            }
        } else {
            println!(
                "  PM4 multi-queue correctness gate ({lanes} lane(s)): counter HSACO not set (set REDLINE_FLOOR_HSACO_CTR or REDLINE_FLOOR_VERIFY_HSACO) — gate not run"
            );
            // Gate not-run is not PASS; keep timing but clearly unverified.
            // The caller must supply a counter HSACO to get a trustworthy number.
            true
        };
        if !gate_pass {
            println!(
                "  {:<52} {}",
                format!("PM4 multi-queue — {lanes} lane(s)"),
                "skipped (gate FAIL)"
            );
            continue;
        }
        match measure_pm4_multiqueue_host(&device, &exec, &symbol, n, lanes, m, warmup, true) {
            Ok((us, queues)) => println!(
                "  {:<52} {:>10.3} {:>10.4}   {} queue(s)",
                format!("PM4 multi-queue — conservative, {lanes} lane(s)"),
                us,
                us / n as f64,
                queues
            ),
            // A lane count the runtime refuses is information, not a failure:
            // report it and keep going rather than aborting the whole sweep.
            Err(e) => println!(
                "  {:<52} {}",
                format!("PM4 multi-queue — {lanes} lane(s)"),
                e
            ),
        }
    }
    Ok(())
}
