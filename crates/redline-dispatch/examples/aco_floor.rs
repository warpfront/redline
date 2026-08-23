// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! Run an ABI-neutral GFX12 kernel image through Redline's retained PM4 path.
//! The bundled `gmb_aco_style.s` image uses the same compact descriptor-based
//! instruction shape as the RADV/ACO dispatch-floor shader while retaining
//! Redline's public-HSA queue and memory ownership.

use std::sync::Arc;
use std::time::Instant;

use redline_dispatch::aql::{
    DevicePool, Executable, Gfx12DispatchMode, Gfx12KernelImage, Gfx12Pm4CommandBuffer,
    GpuSelector, KernargPool, LaunchGeometry, Runtime, SingleQueuePm4Ib, load_symbols,
};

const GFX12_BUFFER_CONFIG: u32 = 0x3100_4000;
const USER_SGPR_MASK: u32 = 0x1f << 1;
const RAW_USER_SGPRS: u32 = 5;

#[derive(Clone, Copy, Debug)]
enum BindingMode {
    DescriptorTable,
    DirectSrd,
}

fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(|left, right| left.partial_cmp(right).unwrap());
    values[values.len() / 2]
}

fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let code_path =
        std::env::var("ACO_FLOOR_CODE").map_err(|_| "set ACO_FLOOR_CODE to gmb_aco_style.bin")?;
    let reference_path = std::env::var("ACO_FLOOR_REFERENCE_HSACO")
        .map_err(|_| "set ACO_FLOOR_REFERENCE_HSACO to gmb_buffer.co")?;
    let counts = std::env::var("ACO_FLOOR_COUNTS")
        .unwrap_or_else(|_| "1,50,200,941".to_owned())
        .split(',')
        .filter_map(|value| value.trim().parse::<usize>().ok())
        .collect::<Vec<_>>();
    let n = env_usize("ACO_FLOOR_N", 256) as u32;
    let reps = env_usize("ACO_FLOOR_REPS", 100);
    let warmups = env_usize("ACO_FLOOR_WARMUPS", 100);
    let dispatch_mode = match std::env::var("ACO_FLOOR_DISPATCH_MODE")
        .unwrap_or_else(|_| "radv".to_owned())
        .as_str()
    {
        "radv" | "workgroups" => Gfx12DispatchMode::RadvWorkgroups,
        "hsa" | "workitems" => Gfx12DispatchMode::Workitems,
        _ => return Err("ACO_FLOOR_DISPATCH_MODE must be radv or workitems".into()),
    };
    let binding_mode = match std::env::var("ACO_FLOOR_BINDING")
        .unwrap_or_else(|_| "table".to_owned())
        .as_str()
    {
        "table" | "aco" => BindingMode::DescriptorTable,
        "direct" | "srd" => BindingMode::DirectSrd,
        _ => return Err("ACO_FLOOR_BINDING must be table or direct".into()),
    };

    let runtime = Runtime::initialize(load_symbols()?)?;
    let device = runtime.select_gpu(GpuSelector::Ordinal(0))?;
    if !device.name().starts_with("gfx12") {
        return Err(format!("aco_floor requires gfx12, selected {}", device.name()).into());
    }
    let kernarg_pool = KernargPool::discover(&device)?;
    let device_pool = DevicePool::discover(&device)?;

    let reference_bytes: Arc<[u8]> = std::fs::read(reference_path)?.into();
    let reference_executable = Executable::load(&device, reference_bytes)?;
    let reference_kernel = reference_executable.kernel("gmb_buffer_kernel.kd")?;
    let mut image = Gfx12KernelImage::from_hsa(&reference_kernel)?;

    let raw_bytes = std::fs::read(code_path)?;
    let mut raw_code = kernarg_pool.allocate_executable_bytes(raw_bytes.len())?;
    raw_code.write_exact(&raw_bytes)?;
    image.code_entry = raw_code.address() as usize as u64;
    image.compute_pgm_rsrc2 = (image.compute_pgm_rsrc2 & !USER_SGPR_MASK) | (RAW_USER_SGPRS << 1);

    println!(
        "aco_floor  code={} bytes mode={dispatch_mode:?} binding={binding_mode:?} rsrc1={:#x} rsrc2={:#x} rsrc3={:#x}",
        raw_bytes.len(),
        image.compute_pgm_rsrc1,
        image.compute_pgm_rsrc2,
        image.compute_pgm_rsrc3,
    );
    println!(" count | host us/disp | correct");
    println!("--------------------------------");

    for count in counts {
        let mut output = device_pool.allocate(n as usize * 4)?;
        let zeros = vec![0_u8; output.len()];
        // SAFETY: no command buffer refers to `output` yet.
        unsafe { output.copy_from_host(&zeros)? };

        let mut descriptor = kernarg_pool.allocate_executable_bytes(16)?;
        let output_address = output.address() as usize as u64;
        let descriptor_words = [
            output_address as u32,
            (output_address >> 32) as u32,
            n * 4,
            GFX12_BUFFER_CONFIG,
        ];
        for (index, word) in descriptor_words.into_iter().enumerate() {
            descriptor.as_mut_bytes()[index * 4..index * 4 + 4]
                .copy_from_slice(&word.to_le_bytes());
        }
        let user_sgprs = match binding_mode {
            BindingMode::DescriptorTable => {
                let descriptor_address = descriptor.address() as usize as u64;
                [
                    descriptor_address as u32,
                    (descriptor_address >> 32) as u32,
                    n,
                    0,
                    0,
                ]
            }
            BindingMode::DirectSrd => [
                descriptor_words[0],
                descriptor_words[1],
                descriptor_words[2],
                descriptor_words[3],
                n,
            ],
        };

        let geometry = LaunchGeometry::new([n.div_ceil(256) * 256, 1, 1], [256, 1, 1])?;
        let mut commands = Gfx12Pm4CommandBuffer::new_stateful();
        commands.acquire_system_gfx12();
        for index in 0..count {
            commands.dispatch_image_with_mode(&image, geometry, 0, &user_sgprs, dispatch_mode)?;
            if index + 1 < count {
                commands.dependency_rmw_same_agent_gfx12();
            }
        }
        commands.wait_compute_idle();
        let mut graph = SingleQueuePm4Ib::create(&device, &kernarg_pool, &commands)?;

        // SAFETY: raw code, descriptor, output, and the retained IB remain live.
        unsafe { graph.replay_and_wait()? };
        let mut observed = vec![0_u8; output.len()];
        // SAFETY: replay completion excludes concurrent GPU access.
        unsafe { output.copy_to_host(&mut observed)? };
        let expected = (count as f32).to_bits();
        let correct = observed
            .chunks_exact(4)
            .all(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()) == expected);
        if !correct {
            return Err(format!("raw image failed full-grid correctness at count {count}").into());
        }

        for _ in 0..warmups {
            // SAFETY: all image and binding storage remains live.
            unsafe { graph.replay_and_wait()? };
        }
        let mut samples = Vec::with_capacity(reps);
        for _ in 0..reps {
            let started = Instant::now();
            // SAFETY: all image and binding storage remains live.
            unsafe { graph.replay_and_wait()? };
            samples.push(started.elapsed().as_secs_f64() * 1_000_000.0);
        }
        let mut final_observed = vec![0_u8; output.len()];
        // SAFETY: the last replay completed before this copy.
        unsafe { output.copy_to_host(&mut final_observed)? };
        let final_expected = ((count * (1 + warmups + reps)) as f32).to_bits();
        let final_correct = final_observed
            .chunks_exact(4)
            .all(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()) == final_expected);
        if !final_correct {
            return Err(
                format!("raw image failed measured-burst correctness at count {count}").into(),
            );
        }
        println!(
            "{count:>6} | {:>12.4} | PASS",
            median(samples) / count as f64
        );
    }

    Ok(())
}
