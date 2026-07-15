// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! Architecture-portable public-ROCr/AQL dispatch smoke test.
//!
//! Compile `crates/radiowave/tests/fixtures/portable_buffer_probe.hip` for the
//! selected GPU, then provide the resulting code object through
//! `REDLINE_AQL_HSACO`. Select a GPU with `ROCR_VISIBLE_DEVICES` or
//! `REDLINE_GPU_ORDINAL`.

use std::sync::Arc;

use redline_dispatch::aql::{
    BatchFencePolicy, DevicePool, Executable, GpuSelector, KernargPool, LaunchGeometry,
    RecordedDispatch, Runtime, SingleQueueBatchGraph, load_symbols,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let code_path = std::env::var("REDLINE_AQL_HSACO")
        .map_err(|_| "set REDLINE_AQL_HSACO to a matching code object")?;
    let symbol = std::env::var("REDLINE_AQL_SYMBOL")
        .unwrap_or_else(|_| "radiowave_portable_buffer_probe.kd".to_owned());
    let ordinal = std::env::var("REDLINE_GPU_ORDINAL")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);

    let runtime = Runtime::initialize(load_symbols()?)?;
    let device = runtime.select_gpu(GpuSelector::Ordinal(ordinal))?;
    let kernarg_pool = KernargPool::discover(&device)?;
    let device_pool = DevicePool::discover(&device)?;
    let code: Arc<[u8]> = std::fs::read(&code_path)?.into();
    let executable = Executable::load(&device, code)?;

    let mut input = device_pool.allocate(4)?;
    let mut output = device_pool.allocate(4)?;
    unsafe {
        input.copy_from_host(&41_u32.to_le_bytes())?;
        output.copy_from_host(&0_u32.to_le_bytes())?;
    }
    let input_address = input.address() as usize as u64;
    let output_address = output.address() as usize as u64;

    // Two dispatches exercise retained packet publication and the internal
    // system-scope dependency without changing the expected value.
    let mut dispatches = Vec::with_capacity(2);
    for _ in 0..2 {
        let kernel = executable.kernel(&symbol)?;
        let mut kernarg = kernarg_pool.allocate_for(kernel.metadata())?;
        if kernarg.len() < 16 {
            return Err(format!(
                "kernel {} exposes a {}-byte kernarg segment; expected two pointers",
                kernel.name(),
                kernarg.len()
            )
            .into());
        }
        kernarg.as_mut_bytes()[0..8].copy_from_slice(&input_address.to_le_bytes());
        kernarg.as_mut_bytes()[8..16].copy_from_slice(&output_address.to_le_bytes());
        dispatches.push(RecordedDispatch::new(
            0,
            kernel,
            LaunchGeometry::new([1, 1, 1], [1, 1, 1])?,
            kernarg,
        )?);
    }

    let mut graph = SingleQueueBatchGraph::create(
        &device,
        *device.queue_size_range().start(),
        dispatches,
        BatchFencePolicy::SystemEveryDispatch,
    )?;
    let timing = unsafe { graph.replay_and_wait()? };
    let mut observed = [0_u8; 4];
    unsafe { output.copy_to_host(&mut observed)? };
    let observed = u32::from_le_bytes(observed);
    if observed != 42 {
        return Err(format!("AQL correctness failure: observed {observed}, expected 42").into());
    }
    println!(
        "aql-arch-smoke: pass device={} pci={:?} output={} gpu_span_us={:.4}",
        device.name(),
        device.pci_bus_id(),
        observed,
        timing.span_microseconds(),
    );
    Ok(())
}
