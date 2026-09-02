// SPDX-License-Identifier: Apache-2.0
use hip_bridge::HipRuntime;
use redline_dispatch::aql::{Gfx10Pm4CommandBuffer, Gfx12Pm4CommandBuffer, GpuSelector, KernargPool, LaunchGeometry, Runtime, SingleQueuePm4Ib, load_symbols};
use std::sync::Arc;
fn run_hip(gx: u32, gy: u32, hsaco: &[u8]) -> anyhow::Result<(Vec<(u32,u32)>, Vec<u32>)> {
    let hip = HipRuntime::load()?; hip.set_device(0)?;
    let module = hip.module_load_data(hsaco)?; let func = hip.module_get_function(&module, "debug_grid")?;
    let total_wg = (gx * gy) as usize;
    let out = hip.malloc(total_wg * 4)?; let hits = hip.malloc((gx*gy) as usize *4)?; let counter = hip.malloc(4)?;
    hip.memset(&out, 0, out.size())?; hip.memset(&hits, 0, hits.size())?; hip.memset(&counter, 0, counter.size())?; hip.device_synchronize()?;
    let mut kernarg = vec![0u8; 32];
    let out_ptr = out.as_ptr() as u64; let hits_ptr = hits.as_ptr() as u64; let counter_ptr = counter.as_ptr() as u64;
    kernarg[0..8].copy_from_slice(&out_ptr.to_ne_bytes()); kernarg[8..16].copy_from_slice(&hits_ptr.to_ne_bytes()); kernarg[16..24].copy_from_slice(&counter_ptr.to_ne_bytes()); kernarg[24..28].copy_from_slice(&(gx as i32).to_ne_bytes());
    let stream = hip.stream_create()?;
    unsafe { hip.launch_kernel_blob(&func, [gx, gy, 1], [32,1,1], 0, Some(&stream), &mut kernarg)?; }
    hip.stream_synchronize(&stream)?;
    let mut out_bytes = vec![0u8; total_wg*4]; let mut hits_bytes = vec![0u8; (gx*gy) as usize*4]; let mut counter_bytes = vec![0u8; 4];
    hip.memcpy_dtoh(&mut out_bytes, &out)?; hip.memcpy_dtoh(&mut hits_bytes, &hits)?; hip.memcpy_dtoh(&mut counter_bytes, &counter)?;
    let counter_val = u32::from_ne_bytes(counter_bytes[0..4].try_into().unwrap());
    let mut out_pairs = Vec::new(); for i in 0..counter_val as usize { let v = u32::from_ne_bytes(out_bytes[i*4..i*4+4].try_into().unwrap()); out_pairs.push((v & 0xFFFF, v >> 16)); }
    let mut hits_vals = Vec::new(); for i in 0..(gx*gy) as usize { hits_vals.push(u32::from_ne_bytes(hits_bytes[i*4..i*4+4].try_into().unwrap())); }
    Ok((out_pairs, hits_vals))
}
fn run_redline(gx: u32, gy: u32, hsaco: &[u8]) -> anyhow::Result<(Vec<(u32,u32)>, Vec<u32>)> {
    let runtime = Runtime::initialize(load_symbols()?)?;
    let ordinal = std::env::var("HIP_VISIBLE_DEVICES").or_else(|_| std::env::var("ROCR_VISIBLE_DEVICES")).ok().and_then(|v| v.split(',').next().and_then(|s| s.trim().parse::<usize>().ok())).unwrap_or(0);
    let device = runtime.select_gpu(GpuSelector::Ordinal(ordinal)).or_else(|_| runtime.select_gpu(GpuSelector::Ordinal(0)))?;
    let exec = redline_dispatch::aql::Executable::load(&device, Arc::<[u8]>::from(hsaco))?;
    let kernel = exec.kernel("debug_grid.kd")?;
    let pool = KernargPool::discover(&device)?;
    let hip = HipRuntime::load()?; hip.set_device(0)?;
    let total_wg = (gx * gy) as usize;
    let out = hip.malloc(total_wg * 4)?; let hits = hip.malloc((gx*gy) as usize *4)?; let counter = hip.malloc(4)?;
    hip.memset(&out, 0, out.size())?; hip.memset(&hits, 0, hits.size())?; hip.memset(&counter, 0, counter.size())?; hip.device_synchronize()?;
    let mut karg = pool.allocate_for(kernel.metadata())?;
    let bytes = karg.as_mut_bytes(); bytes.fill(0);
    let out_ptr = out.as_ptr() as u64; let hits_ptr = hits.as_ptr() as u64; let counter_ptr = counter.as_ptr() as u64;
    bytes[0..8].copy_from_slice(&out_ptr.to_ne_bytes()); bytes[8..16].copy_from_slice(&hits_ptr.to_ne_bytes()); bytes[16..24].copy_from_slice(&counter_ptr.to_ne_bytes()); bytes[24..28].copy_from_slice(&(gx as i32).to_ne_bytes());
    let geometry = LaunchGeometry::from_workgroups([gx, gy, 1], [32,1,1])?;
    let is_gfx12 = device.name().contains("gfx12");
    let (mut ib, mut ownership) = if is_gfx12 {
        let mut cmds = Gfx12Pm4CommandBuffer::new_stateful(); cmds.dispatch(&kernel, geometry, 0, karg.address())?;
        let mut oc = Gfx12Pm4CommandBuffer::new(); oc.acquire_system_gfx12();
        (SingleQueuePm4Ib::create(&device, &pool, &cmds)?, SingleQueuePm4Ib::create(&device, &pool, &oc)?)
    } else {
        let mut cmds = Gfx10Pm4CommandBuffer::new_stateful(); cmds.dispatch(&kernel, geometry, 0, karg.address())?;
        let mut oc = Gfx10Pm4CommandBuffer::new(); oc.acquire_system();
        let ib = if device.name().contains("gfx11") { SingleQueuePm4Ib::create_gfx11(&device, &pool, &cmds)? } else { SingleQueuePm4Ib::create_gfx10(&device, &pool, &cmds)? };
        let ownership = if device.name().contains("gfx11") { SingleQueuePm4Ib::create_gfx11(&device, &pool, &oc)? } else { SingleQueuePm4Ib::create_gfx10(&device, &pool, &oc)? };
        (ib, ownership)
    };
    let _keep = karg;
    unsafe { ownership.replay_and_wait()?; } unsafe { ib.replay_and_wait()?; }
    let mut out_bytes = vec![0u8; total_wg*4]; let mut hits_bytes = vec![0u8; (gx*gy) as usize*4]; let mut counter_bytes = vec![0u8; 4];
    hip.memcpy_dtoh(&mut out_bytes, &out)?; hip.memcpy_dtoh(&mut hits_bytes, &hits)?; hip.memcpy_dtoh(&mut counter_bytes, &counter)?;
    let counter_val = u32::from_ne_bytes(counter_bytes[0..4].try_into().unwrap());
    let mut out_pairs = Vec::new(); for i in 0..counter_val as usize { let v = u32::from_ne_bytes(out_bytes[i*4..i*4+4].try_into().unwrap()); out_pairs.push((v & 0xFFFF, v >> 16)); }
    let mut hits_vals = Vec::new(); for i in 0..(gx*gy) as usize { hits_vals.push(u32::from_ne_bytes(hits_bytes[i*4..i*4+4].try_into().unwrap())); }
    Ok((out_pairs, hits_vals))
}
fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let mut gx = 128; let mut gy = 2; let mut arch = "gfx1151".to_string();
    for i in 0..args.len() { if args[i]=="--gx" { gx=args[i+1].parse()?; } if args[i]=="--gy" { gy=args[i+1].parse()?; } if args[i]=="--arch" { arch=args[i+1].clone(); } }
    let hsaco_path = match arch.as_str() { "gfx1151" => "/tmp/debug_grid_gfx1151.hsaco", "gfx1201" => "/tmp/debug_grid_gfx1201.hsaco", "gfx1100" => "/tmp/debug_grid_gfx1151.hsaco", _ => "/tmp/debug_grid.hsaco", };
    let hsaco = std::fs::read(hsaco_path)?;
    println!("=== HIP gx={} gy={} arch={} ===", gx, gy, arch);
    match run_hip(gx, gy, &hsaco) { Ok((pairs, hits)) => { println!("HIP total {}", pairs.len()); let mut counts = std::collections::BTreeMap::new(); for (x,y) in &pairs { *counts.entry((*x,*y)).or_insert(0) +=1; } println!("HIP unique {}", counts.len()); for ((x,y),c) in &counts { println!("  ({},{}) x{}", x,y,c); } println!("HIP hits {:?}", hits); if counts.values().all(|&c| c==1) { println!("HIP: all once"); } }, Err(e) => println!("HIP failed: {:#}", e), }
    println!("=== Redline gx={} gy={} arch={} ===", gx, gy, arch);
    match run_redline(gx, gy, &hsaco) { Ok((pairs, hits)) => { println!("Redline total {}", pairs.len()); let mut counts = std::collections::BTreeMap::new(); for (x,y) in &pairs { *counts.entry((*x,*y)).or_insert(0) +=1; } println!("Redline unique {}", counts.len()); for ((x,y),c) in &counts { println!("  ({},{}) x{}", x,y,c); } println!("Redline hits {:?}", hits); if counts.values().all(|&c| c==1) { println!("Redline: all once"); } for y in 0..gy { for x in 0..gx { if hits[(y*gx+x) as usize]!=1 { println!("Redline missing at ({},{}) hits {}", x,y, hits[(y*gx+x) as usize]); } } } }, Err(e) => println!("Redline failed: {:#}", e), }
    Ok(())
}
