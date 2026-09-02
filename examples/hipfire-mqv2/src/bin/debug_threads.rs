// SPDX-License-Identifier: Apache-2.0
use hip_bridge::HipRuntime;
use redline_dispatch::aql::{Gfx10Pm4CommandBuffer, Gfx12Pm4CommandBuffer, GpuSelector, KernargPool, LaunchGeometry, Runtime, SingleQueuePm4Ib, load_symbols};
use std::sync::Arc;
fn run_hip(gx: u32, gy: u32, hsaco: &[u8]) -> anyhow::Result<(Vec<i32>, Vec<i32>, Vec<i32>)> {
    let hip = HipRuntime::load()?; hip.set_device(0)?;
    let module = hip.module_load_data(hsaco)?; let func = hip.module_get_function(&module, "debug_threads")?;
    let total = (gx * gy) as usize;
    let maxtid = hip.malloc(total * 4)?; let threads = hip.malloc(total * 4)?; let lane_max = hip.malloc(total * 4)?;
    hip.memset(&maxtid, 0, maxtid.size())?; hip.memset(&threads, 0, threads.size())?; hip.memset(&lane_max, 0, lane_max.size())?; hip.device_synchronize()?;
    let mut kernarg = vec![0u8; 32];
    let a = maxtid.as_ptr() as u64; let b = threads.as_ptr() as u64; let c = lane_max.as_ptr() as u64;
    kernarg[0..8].copy_from_slice(&a.to_ne_bytes()); kernarg[8..16].copy_from_slice(&b.to_ne_bytes()); kernarg[16..24].copy_from_slice(&c.to_ne_bytes()); kernarg[24..28].copy_from_slice(&(gx as i32).to_ne_bytes());
    let stream = hip.stream_create()?;
    unsafe { hip.launch_kernel_blob(&func, [gx, gy, 1], [32,1,1], 0, Some(&stream), &mut kernarg)?; }
    hip.stream_synchronize(&stream)?;
    let mut a_b = vec![0u8; total*4]; let mut b_b = vec![0u8; total*4]; let mut c_b = vec![0u8; total*4];
    hip.memcpy_dtoh(&mut a_b, &maxtid)?; hip.memcpy_dtoh(&mut b_b, &threads)?; hip.memcpy_dtoh(&mut c_b, &lane_max)?;
    let maxtid_v: Vec<i32> = a_b.chunks_exact(4).map(|c| i32::from_ne_bytes(c.try_into().unwrap())).collect();
    let threads_v: Vec<i32> = b_b.chunks_exact(4).map(|c| i32::from_ne_bytes(c.try_into().unwrap())).collect();
    let lane_max_v: Vec<i32> = c_b.chunks_exact(4).map(|c| i32::from_ne_bytes(c.try_into().unwrap())).collect();
    Ok((maxtid_v, threads_v, lane_max_v))
}
fn run_redline(gx: u32, gy: u32, hsaco: &[u8]) -> anyhow::Result<(Vec<i32>, Vec<i32>, Vec<i32>)> {
    let runtime = Runtime::initialize(load_symbols()?)?;
    let ordinal = std::env::var("HIP_VISIBLE_DEVICES").or_else(|_| std::env::var("ROCR_VISIBLE_DEVICES")).ok().and_then(|v| v.split(',').next().and_then(|s| s.trim().parse::<usize>().ok())).unwrap_or(0);
    let device = runtime.select_gpu(GpuSelector::Ordinal(ordinal)).or_else(|_| runtime.select_gpu(GpuSelector::Ordinal(0)))?;
    let exec = redline_dispatch::aql::Executable::load(&device, Arc::<[u8]>::from(hsaco))?;
    let kernel = exec.kernel("debug_threads.kd")?;
    let pool = KernargPool::discover(&device)?;
    let hip = HipRuntime::load()?; hip.set_device(0)?;
    let total = (gx * gy) as usize;
    let maxtid = hip.malloc(total * 4)?; let threads = hip.malloc(total * 4)?; let lane_max = hip.malloc(total * 4)?;
    hip.memset(&maxtid, 0, maxtid.size())?; hip.memset(&threads, 0, threads.size())?; hip.memset(&lane_max, 0, lane_max.size())?; hip.device_synchronize()?;
    let mut karg = pool.allocate_for(kernel.metadata())?;
    let bytes = karg.as_mut_bytes(); bytes.fill(0);
    let a = maxtid.as_ptr() as u64; let b = threads.as_ptr() as u64; let c = lane_max.as_ptr() as u64;
    bytes[0..8].copy_from_slice(&a.to_ne_bytes()); bytes[8..16].copy_from_slice(&b.to_ne_bytes()); bytes[16..24].copy_from_slice(&c.to_ne_bytes()); bytes[24..28].copy_from_slice(&(gx as i32).to_ne_bytes());
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
    let mut a_b = vec![0u8; total*4]; let mut b_b = vec![0u8; total*4]; let mut c_b = vec![0u8; total*4];
    hip.memcpy_dtoh(&mut a_b, &maxtid)?; hip.memcpy_dtoh(&mut b_b, &threads)?; hip.memcpy_dtoh(&mut c_b, &lane_max)?;
    let maxtid_v: Vec<i32> = a_b.chunks_exact(4).map(|c| i32::from_ne_bytes(c.try_into().unwrap())).collect();
    let threads_v: Vec<i32> = b_b.chunks_exact(4).map(|c| i32::from_ne_bytes(c.try_into().unwrap())).collect();
    let lane_max_v: Vec<i32> = c_b.chunks_exact(4).map(|c| i32::from_ne_bytes(c.try_into().unwrap())).collect();
    Ok((maxtid_v, threads_v, lane_max_v))
}
fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let mut gx = 128; let mut gy = 2; let mut arch = "gfx1151".to_string();
    for i in 0..args.len() { if args[i]=="--gx" { gx=args[i+1].parse()?; } if args[i]=="--gy" { gy=args[i+1].parse()?; } if args[i]=="--arch" { arch=args[i+1].clone(); } }
    let hsaco_path = match arch.as_str() { "gfx1151" => "/tmp/debug_threads_gfx1151.hsaco", "gfx1201" => "/tmp/debug_threads_gfx1201.hsaco", _ => "/tmp/debug_threads_gfx1151.hsaco", };
    let hsaco = std::fs::read(hsaco_path)?;
    println!("=== HIP gx={} gy={} arch={} ===", gx, gy, arch);
    match run_hip(gx, gy, &hsaco) { Ok((a,b,c)) => { println!("HIP maxtid {:?}", &a[..a.len().min(10)]); println!("HIP threads {:?}", &b[..b.len().min(10)]); println!("HIP lane_max {:?}", &c[..c.len().min(10)]); let mut cnt=std::collections::BTreeMap::new(); for &v in &b { *cnt.entry(v).or_insert(0)+=1; } println!("HIP thread count histogram {:?}", cnt); } Err(e) => println!("HIP failed: {:#}", e), }
    println!("=== Redline gx={} gy={} arch={} ===", gx, gy, arch);
    match run_redline(gx, gy, &hsaco) { Ok((a,b,c)) => { println!("Redline maxtid {:?}", &a[..a.len().min(10)]); println!("Redline threads {:?}", &b[..b.len().min(10)]); println!("Redline lane_max {:?}", &c[..c.len().min(10)]); let mut cnt=std::collections::BTreeMap::new(); for &v in &b { *cnt.entry(v).or_insert(0)+=1; } println!("Redline thread count histogram {:?}", cnt); } Err(e) => println!("Redline failed: {:#}", e), }
    Ok(())
}
