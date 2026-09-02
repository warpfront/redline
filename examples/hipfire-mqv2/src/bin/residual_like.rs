// SPDX-License-Identifier: Apache-2.0
use hip_bridge::HipRuntime;
use redline_dispatch::aql::{Gfx10Pm4CommandBuffer, Gfx12Pm4CommandBuffer, GpuSelector, KernargPool, LaunchGeometry, Runtime, SingleQueuePm4Ib, load_symbols};
use std::sync::Arc;
fn run_hip(gx: u32, gy: u32, n: u32, m: u32, iterations: usize, hsaco: &[u8]) -> anyhow::Result<Vec<f32>> {
    let hip = HipRuntime::load()?; hip.set_device(0)?;
    let module = hip.module_load_data(hsaco)?; let func = hip.module_get_function(&module, "residual_like")?;
    let total = (gx*gy) as usize;
    let y = hip.malloc(total*4)?;
    hip.memset(&y, 0, y.size())?; hip.device_synchronize()?;
    let mut kernarg = vec![0u8; 24];
    let y_ptr = y.as_ptr() as u64;
    kernarg[0..8].copy_from_slice(&y_ptr.to_ne_bytes()); kernarg[8..12].copy_from_slice(&(n as i32).to_ne_bytes()); kernarg[12..16].copy_from_slice(&(m as i32).to_ne_bytes()); kernarg[16..20].copy_from_slice(&(gx as i32).to_ne_bytes());
    let stream = hip.stream_create()?;
    for _ in 0..iterations { unsafe { hip.launch_kernel_blob(&func, [gx, gy, 1], [32,1,1], 0, Some(&stream), &mut kernarg)?; } }
    hip.stream_synchronize(&stream)?;
    let mut bytes = vec![0u8; total*4]; hip.memcpy_dtoh(&mut bytes, &y)?;
    let vals: Vec<f32> = bytes.chunks_exact(4).map(|c| f32::from_ne_bytes(c.try_into().unwrap())).collect();
    Ok(vals)
}
fn run_redline(gx: u32, gy: u32, n: u32, m: u32, iterations: usize, hsaco: &[u8], add_dep: bool) -> anyhow::Result<Vec<f32>> {
    let runtime = Runtime::initialize(load_symbols()?)?;
    let ordinal = std::env::var("HIP_VISIBLE_DEVICES").or_else(|_| std::env::var("ROCR_VISIBLE_DEVICES")).ok().and_then(|v| v.split(',').next().and_then(|s| s.trim().parse::<usize>().ok())).unwrap_or(0);
    let device = runtime.select_gpu(GpuSelector::Ordinal(ordinal)).or_else(|_| runtime.select_gpu(GpuSelector::Ordinal(0)))?;
    let exec = redline_dispatch::aql::Executable::load(&device, Arc::<[u8]>::from(hsaco))?;
    let kernel = exec.kernel("residual_like.kd")?;
    let pool = KernargPool::discover(&device)?;
    let hip = HipRuntime::load()?; hip.set_device(0)?;
    let total = (gx*gy) as usize;
    let y = hip.malloc(total*4)?;
    hip.memset(&y, 0, y.size())?; hip.device_synchronize()?;
    let mut karg = pool.allocate_for(kernel.metadata())?;
    let bytes = karg.as_mut_bytes(); bytes.fill(0);
    let y_ptr = y.as_ptr() as u64;
    bytes[0..8].copy_from_slice(&y_ptr.to_ne_bytes()); bytes[8..12].copy_from_slice(&(n as i32).to_ne_bytes()); bytes[12..16].copy_from_slice(&(m as i32).to_ne_bytes()); bytes[16..20].copy_from_slice(&(gx as i32).to_ne_bytes());
    let geometry = LaunchGeometry::from_workgroups([gx, gy, 1], [32,1,1])?;
    let is_gfx12 = device.name().contains("gfx12");
    let (mut ib, mut ownership) = if is_gfx12 {
        let mut cmds = Gfx12Pm4CommandBuffer::new_stateful();
        for i in 0..iterations { cmds.dispatch(&kernel, geometry, 0, karg.address())?; if add_dep && i + 1 < iterations { cmds.dependency_rmw_same_agent_gfx12(); } }
        let mut oc = Gfx12Pm4CommandBuffer::new(); oc.acquire_system_gfx12();
        (SingleQueuePm4Ib::create(&device, &pool, &cmds)?, SingleQueuePm4Ib::create(&device, &pool, &oc)?)
    } else {
        let mut cmds = Gfx10Pm4CommandBuffer::new_stateful();
        for i in 0..iterations { cmds.dispatch(&kernel, geometry, 0, karg.address())?; if add_dep && i + 1 < iterations { cmds.dependency_rmw_same_agent(); } }
        let mut oc = Gfx10Pm4CommandBuffer::new(); oc.acquire_system();
        let ib = if device.name().contains("gfx11") { SingleQueuePm4Ib::create_gfx11(&device, &pool, &cmds)? } else { SingleQueuePm4Ib::create_gfx10(&device, &pool, &cmds)? };
        let ownership = if device.name().contains("gfx11") { SingleQueuePm4Ib::create_gfx11(&device, &pool, &oc)? } else { SingleQueuePm4Ib::create_gfx10(&device, &pool, &oc)? };
        (ib, ownership)
    };
    let _keep = karg;
    unsafe { ownership.replay_and_wait()?; } unsafe { ib.replay_and_wait()?; }
    let mut bytes = vec![0u8; total*4]; hip.memcpy_dtoh(&mut bytes, &y)?;
    let vals: Vec<f32> = bytes.chunks_exact(4).map(|c| f32::from_ne_bytes(c.try_into().unwrap())).collect();
    Ok(vals)
}
fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let mut gx = 128; let mut gy = 2; let mut n = 128; let mut m = 2048; let mut iterations = 4; let mut arch = "gfx1151".to_string(); let mut dep = false;
    for i in 0..args.len() { match args[i].as_str() { "--gx"=>{gx=args[i+1].parse()?;} "--gy"=>{gy=args[i+1].parse()?;} "--n"=>{n=args[i+1].parse()?;} "--m"=>{m=args[i+1].parse()?;} "--iterations"=>{iterations=args[i+1].parse()?;} "--arch"=>{arch=args[i+1].clone();} "--dep"=>{dep=true;} _=>{} } }
    let hsaco_path = match arch.as_str() { "gfx1151" => "/tmp/residual_like_gfx1151.hsaco", "gfx1201" => "/tmp/residual_like_gfx1201.hsaco", _ => "/tmp/residual_like_gfx1151.hsaco", };
    let hsaco = std::fs::read(hsaco_path)?;
    println!("=== HIP gx={} gy={} n={} m={} iters={} arch={} ===", gx, gy, n, m, iterations, arch);
    match run_hip(gx, gy, n, m, iterations, &hsaco) { Ok(vals) => { println!("HIP vals (first 10): {:?}", &vals[..vals.len().min(10)]); let all1 = vals.iter().all(|&v| (v - iterations as f32).abs() < 1e-5); println!("HIP all == {}: {}", iterations, all1); } Err(e) => println!("HIP failed: {:#}", e), }
    println!("=== Redline gx={} gy={} n={} m={} iters={} arch={} dep={} ===", gx, gy, n, m, iterations, arch, dep);
    match run_redline(gx, gy, n, m, iterations, &hsaco, dep) { Ok(vals) => { println!("Redline vals (first 10): {:?}", &vals[..vals.len().min(10)]); let all1 = vals.iter().all(|&v| (v - iterations as f32).abs() < 1e-5); println!("Redline all == {}: {}", iterations, all1); let mut bad=0; for (i,&v) in vals.iter().enumerate() { if (v - iterations as f32).abs() > 1e-5 { println!("Redline elem {} val {}", i, v); bad+=1; if bad>20 {break;} } } } Err(e) => println!("Redline failed: {:#}", e), }
    Ok(())
}
