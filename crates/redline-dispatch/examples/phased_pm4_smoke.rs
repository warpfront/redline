// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! Live gfx12 smoke for ordered multi-queue retained PM4.
//!
//! Build the matching code object and run under the shared GPU lock:
//!
//! ```text
//! hipcc --genco --offload-arch=gfx1201 -O3 bench/phased_pm4_smoke.hip \
//!   -o /tmp/phased_pm4_smoke.hsaco
//! REDLINE_PHASED_HSACO=/tmp/phased_pm4_smoke.hsaco \
//!   cargo run -p redline-dispatch --example phased_pm4_smoke
//! ```

use std::sync::Arc;

use redline_dispatch::aql::{
    Executable, Gfx12Pm4CommandBuffer, GpuSelector, KernargBuffer, KernargPool, LaunchGeometry,
    PhasedMultiQueuePm4Ib, Runtime, load_symbols,
};

fn put_u64(kernarg: &mut KernargBuffer, offset: usize, value: u64) {
    kernarg.as_mut_bytes()[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

fn put_u32(kernarg: &mut KernargBuffer, offset: usize, value: u32) {
    kernarg.as_mut_bytes()[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn read_u32(buffer: &mut KernargBuffer) -> u32 {
    u32::from_le_bytes(buffer.as_mut_bytes()[..4].try_into().unwrap())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let hsaco = std::env::var("REDLINE_PHASED_HSACO")
        .map_err(|_| "set REDLINE_PHASED_HSACO to bench/phased_pm4_smoke.hip's code object")?;
    let runtime = Runtime::initialize(load_symbols()?)?;
    let device = runtime.select_gpu(GpuSelector::Ordinal(0))?;
    if !device.name().starts_with("gfx12") {
        return Err(format!(
            "phased_pm4_smoke requires gfx12, selected {}",
            device.name()
        )
        .into());
    }
    let pool = KernargPool::discover(&device)?;
    let code: Arc<[u8]> = std::fs::read(hsaco)?.into();
    let executable = Executable::load(&device, code)?;
    let write = executable.kernel("phased_write.kd")?;
    let consume = executable.kernel("phased_consume.kd")?;
    let geometry = LaunchGeometry::new([64, 1, 1], [64, 1, 1])?;

    let mut source = pool.allocate_executable_bytes(4)?;
    let mut peer = pool.allocate_executable_bytes(4)?;
    let mut output = pool.allocate_executable_bytes(4)?;
    let mut source_arg = pool.allocate_for(write.metadata())?;
    let mut peer_arg = pool.allocate_for(write.metadata())?;
    let mut consumer_arg = pool.allocate_for(consume.metadata())?;
    for kernarg in [&mut source_arg, &mut peer_arg, &mut consumer_arg] {
        kernarg.as_mut_bytes().fill(0);
    }
    put_u64(&mut source_arg, 0, source.address() as usize as u64);
    put_u32(&mut source_arg, 8, 41);
    put_u32(&mut source_arg, 12, 100_000);
    put_u64(&mut peer_arg, 0, peer.address() as usize as u64);
    put_u32(&mut peer_arg, 8, 7);
    put_u32(&mut peer_arg, 12, 10_000);
    put_u64(&mut consumer_arg, 0, source.address() as usize as u64);
    put_u64(&mut consumer_arg, 8, output.address() as usize as u64);

    let producer =
        |kernarg: &KernargBuffer| -> Result<Gfx12Pm4CommandBuffer, Box<dyn std::error::Error>> {
            let mut commands = Gfx12Pm4CommandBuffer::new_stateful();
            commands.acquire_system_gfx12();
            commands.dispatch(&write, geometry, 0, kernarg.address())?;
            commands.wait_compute_idle();
            Ok(commands)
        };
    let source_commands = producer(&source_arg)?;
    let peer_commands = producer(&peer_arg)?;
    let mut consumer_commands = Gfx12Pm4CommandBuffer::new_stateful();
    consumer_commands.acquire_system_gfx12();
    consumer_commands.dispatch(&consume, geometry, 0, consumer_arg.address())?;
    consumer_commands.wait_compute_idle();
    let phases = vec![
        vec![source_commands, peer_commands],
        vec![consumer_commands],
    ];
    let mut replay = PhasedMultiQueuePm4Ib::create(&device, &pool, &phases)?;

    for iteration in 0..3 {
        source.as_mut_bytes().fill(0);
        peer.as_mut_bytes().fill(0);
        output.as_mut_bytes().fill(0);
        // SAFETY: code, kernargs, and all pointees remain live through completion;
        // the two producer lanes write disjoint allocations.
        unsafe { replay.replay_and_wait()? };
        let observed = (
            read_u32(&mut source),
            read_u32(&mut peer),
            read_u32(&mut output),
        );
        if observed != (41, 7, 42) {
            return Err(
                format!("iteration {iteration}: expected (41, 7, 42), got {observed:?}").into(),
            );
        }
    }
    println!(
        "PASS {}: {} queues, {} phases, repeated result (41, 7, 42)",
        device.name(),
        replay.queue_count(),
        replay.phase_count()
    );
    Ok(())
}
