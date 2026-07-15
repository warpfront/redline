// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! One real GFX10/GFX11 HIP kernel lowered to PM4 and carried by public ROCr AQL.
//!
//! Compile `crates/radiowave/tests/fixtures/portable_buffer_probe.hip` for the
//! selected target, then set `REDLINE_AQL_HSACO` to that code object.

use std::sync::Arc;

use redline_dispatch::aql::{
    DevicePool, Executable, Gfx10Pm4CommandBuffer, GpuSelector, KernargPool, LaunchGeometry,
    Runtime, SingleQueuePm4Ib, load_symbols,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let code_path = std::env::var("REDLINE_AQL_HSACO")
        .map_err(|_| "set REDLINE_AQL_HSACO to a matching gfx10/gfx11 code object")?;
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
    let kernel = executable.kernel(&symbol)?;

    let mut input = device_pool.allocate(4)?;
    let mut output = device_pool.allocate(4)?;
    unsafe {
        input.copy_from_host(&41_u32.to_le_bytes())?;
        output.copy_from_host(&0_u32.to_le_bytes())?;
    }
    let input_address = input.address() as usize as u64;
    let output_address = output.address() as usize as u64;
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

    let descriptor = kernel
        .pm4_metadata()
        .ok_or("loaded kernel descriptor has no PM4 metadata")?;
    let mut commands = Gfx10Pm4CommandBuffer::new();
    commands.acquire_system();
    commands.dispatch(
        &kernel,
        LaunchGeometry::new([1, 1, 1], [1, 1, 1])?,
        0,
        kernarg.address(),
    )?;
    commands.wait_compute_idle();
    // The vendor packet has no architected AQL release scope. Flush the
    // completed shader writes before ROCr publishes its completion signal.
    commands.acquire_system();

    let command_dwords = commands.len_dwords();
    if std::env::var_os("REDLINE_PM4_DRY_RUN").is_some() {
        println!(
            "pm4-gfx10-smoke: dry-run device={} metadata={:?} descriptor={:?} dwords={:#x?}",
            device.name(),
            kernel.metadata(),
            descriptor,
            commands.dwords(),
        );
        return Ok(());
    }
    let replays = std::env::var("REDLINE_PM4_REPLAYS")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(1)
        .max(1);
    let mut graph = if device.name().starts_with("gfx10") {
        SingleQueuePm4Ib::create_gfx10(&device, &kernarg_pool, &commands)?
    } else if device.name().starts_with("gfx11") {
        SingleQueuePm4Ib::create_gfx11(&device, &kernarg_pool, &commands)?
    } else {
        return Err(format!(
            "legacy PM4 smoke requires gfx10 or gfx11, selected {}",
            device.name()
        )
        .into());
    };
    for _ in 0..replays {
        unsafe { graph.replay_and_wait()? };
    }

    let mut observed = [0_u8; 4];
    unsafe { output.copy_to_host(&mut observed)? };
    let observed = u32::from_le_bytes(observed);
    if observed != 42 {
        return Err(
            format!("legacy PM4 correctness failure: observed {observed}, expected 42").into(),
        );
    }
    println!(
        "pm4-legacy-smoke: pass device={} pci={:?} output={} replays={} dwords={} properties=0x{:04x}",
        device.name(),
        device.pci_bus_id(),
        observed,
        replays,
        command_dwords,
        descriptor.kernel_code_properties,
    );
    Ok(())
}
