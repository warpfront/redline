// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

use std::error::Error;
use std::sync::Arc;

use redline_dispatch::aql::{
    Executable, GpuSelector, KernargPool, NodeDispatch, Runtime, load_symbols, lower_plan_to_pm4_ib,
};
use redline_dispatch::hipgraph::Graph;
use redline_dispatch::{Access, Dim3, KernelLaunch};

fn main() -> Result<(), Box<dyn Error>> {
    let hsaco = std::env::var("REDLINE_GRAPH_HSACO")
        .or_else(|_| std::env::var("REDLINE_FLOOR_HSACO"))
        .map_err(|_| "set REDLINE_GRAPH_HSACO (or REDLINE_FLOOR_HSACO)")?;
    let symbol = std::env::var("REDLINE_GRAPH_SYMBOL").unwrap_or_else(|_| "ctr_k".to_owned());

    let runtime = Runtime::initialize(load_symbols()?)?;
    let device = runtime.select_gpu(GpuSelector::Ordinal(0))?;
    let pool = KernargPool::discover(&device)?;
    let code: Arc<[u8]> = std::fs::read(hsaco)?.into();
    let executable = Executable::load(&device, code)?;
    let kernel = executable.kernel(&symbol).or_else(|error| {
        if symbol.ends_with(".kd") {
            Err(error)
        } else {
            executable.kernel(&format!("{symbol}.kd"))
        }
    })?;

    let mut counter = pool.allocate_executable_bytes(4)?;
    counter.as_mut_bytes().fill(0);
    if kernel.metadata().kernarg_segment_size < 8 {
        return Err(format!(
            "counter kernel {} has a {}-byte kernarg segment; expected at least 8 bytes",
            kernel.name(),
            kernel.metadata().kernarg_segment_size
        )
        .into());
    }
    let counter_address = counter.address() as usize as u64;
    let packed_kernarg = counter_address.to_le_bytes();

    let mut graph = Graph::new();
    let counter_resource = graph.buffer("counter", 4)?;
    let counter_region = graph.region(counter_resource, 0, 4)?;
    let grid = Dim3::x(1)?;
    let block = Dim3::x(1)?;
    let first = graph.kernel(
        KernelLaunch::new(kernel.name(), grid, block)?,
        [Access::write(counter_region)],
    )?;
    let second = graph.kernel_after(
        KernelLaunch::new(kernel.name(), grid, block)?,
        [Access::write(counter_region)],
        [first],
    )?;
    let graph_exec = graph.instantiate()?;

    let binding = NodeDispatch {
        kernel: &kernel,
        kernargs: &packed_kernarg,
        grid: [1, 1, 1],
        block: [1, 1, 1],
        dyn_group: 0,
    };
    let mut replay = lower_plan_to_pm4_ib(&device, &pool, graph_exec.plan(), |node| {
        (node == first || node == second).then_some(binding)
    })?;
    // SAFETY: the packed kernarg points to `counter`; both it and the loaded
    // executable outlive the replay and this synchronous completion wait.
    unsafe { replay.replay_and_wait()? };

    let observed = u32::from_le_bytes(counter.as_mut_bytes()[..4].try_into().unwrap());
    let expected = 2_u32;
    println!("OBSERVED={observed} EXPECTED={expected}");
    if observed != expected {
        return Err("PM4 graph replay counter mismatch".into());
    }
    Ok(())
}
