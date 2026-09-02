// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

use crate::kernels;
use crate::types::{Arch, Backend, Fixture, KernargLayout, PtrBinding, RowSpec, RunOutput};
use anyhow::{Context, Result};
use hip_bridge::{DeviceBuffer, HipRuntime, Module, Stream};
use radiowave::SchedulerProfile;
use std::collections::HashMap;

/// Device buffers shared with Redline.
pub struct Buffers {
    pub weights: Vec<DeviceBuffer>,
    pub x: DeviceBuffer,
    /// y_sets[set_idx][proj_idx]
    pub y_sets: Vec<Vec<DeviceBuffer>>,
    /// staging copy of y_init for D2D reset path (discriminator 2)
    pub y_staging: Vec<Vec<DeviceBuffer>>,
    pub shape_proj_m: Vec<u32>,
    pub n_tokens: u32,
}
impl Buffers {
    pub fn y_device_ptr(&self, set_idx: usize, proj_idx: usize) -> u64 {
        self.y_sets[set_idx][proj_idx].as_ptr() as usize as u64
    }
    pub fn weights_ptr(&self, proj_idx: usize) -> u64 {
        self.weights[proj_idx].as_ptr() as usize as u64
    }
    pub fn x_ptr(&self) -> u64 {
        self.x.as_ptr() as usize as u64
    }
}

fn pack_kernarg(
    layout: &KernargLayout,
    desc: &crate::types::KernelDesc,
    shape: &crate::types::Shape,
    buffers: &Buffers,
    y_set_idx: usize,
    out: &mut [u8],
) -> Result<()> {
    anyhow::ensure!(out.len() >= layout.explicit_size as usize, "kernarg buffer too small");
    out.fill(0);
    let i32_vals = KernargLayout::i32_values(desc, shape);
    let mut i32_cursor = 0usize;
    for slot in &layout.slots {
        match slot.kind {
            crate::types::ArgKind::Ptr => {
                let binding = slot.binding.expect("ptr slot must have binding");
                let addr: u64 = match binding {
                    PtrBinding::Weights(i) => buffers.weights_ptr(i),
                    PtrBinding::X => buffers.x_ptr(),
                    PtrBinding::Y(i) => buffers.y_device_ptr(y_set_idx, i),
                };
                let bytes = addr.to_ne_bytes();
                let off = slot.offset as usize;
                out[off..off + 8].copy_from_slice(&bytes);
            }
            crate::types::ArgKind::I32 => {
                let v = i32_vals[i32_cursor];
                i32_cursor += 1;
                let off = slot.offset as usize;
                out[off..off + 4].copy_from_slice(&v.to_ne_bytes());
            }
        }
    }
    anyhow::ensure!(i32_cursor == i32_vals.len(), "i32 slot count mismatch");
    Ok(())
}

struct ProfileModules {
    scheduler_profile: SchedulerProfile,
    module: Module,
    functions: HashMap<String, hip_bridge::Function>,
}

pub struct HipBackend {
    pub hip: HipRuntime,
    pub arch: Arch,
    pub arch_str: String,
    device_ordinal: i32,
    profiles: Vec<ProfileModules>,
}

impl HipBackend {
    pub fn new(device_ordinal: i32) -> Result<Self> {
        let hip = HipRuntime::load().context("load HIP runtime")?;
        hip.set_device(device_ordinal).context("hipSetDevice")?;
        let arch_str = hip.get_arch(device_ordinal).context("get arch")?;
        let arch = Arch::parse(&arch_str).unwrap_or(Arch::Gfx1201);
        let mut profiles = Vec::new();
        for &profile in &[SchedulerProfile::Default] {
            let code = kernels::code_object(arch, profile);
            if code.is_empty() {
                continue;
            }
            let module = hip.module_load_data(code).with_context(|| {
                format!("hipModuleLoadData for arch {} profile {}", arch.as_str(), profile.as_str())
            })?;
            let descs = kernels::descriptors(arch);
            let mut functions = HashMap::new();
            for d in descs.iter().filter(|d| d.archs.contains(&arch)) {
                if let Ok(f) = hip.module_get_function(&module, &d.symbol) {
                    functions.insert(d.symbol.clone(), f);
                }
            }
            profiles.push(ProfileModules { scheduler_profile: profile, module, functions });
        }
        if profiles.is_empty() {
            let code = kernels::code_object(arch, SchedulerProfile::Default);
            if !code.is_empty() {
                let module = hip.module_load_data(code)?;
                profiles.push(ProfileModules {
                    scheduler_profile: SchedulerProfile::Default,
                    module,
                    functions: HashMap::new(),
                });
            }
        }
        Ok(Self { hip, arch, arch_str, device_ordinal, profiles })
    }

    fn function_for(&self, row: &RowSpec) -> Result<&hip_bridge::Function> {
        let profile = row.scheduler_profile;
        let pm = self.profiles.iter().find(|p| p.scheduler_profile == profile).with_context(|| {
            format!("missing HIP module for profile {}", profile.as_str())
        })?;
        pm.functions.get(&row.kernel.symbol).with_context(|| {
            format!("HIP function {} not found in HSACO for arch {} profile {}", row.kernel.symbol, self.arch.as_str(), profile.as_str())
        })
    }

    pub fn allocate_buffers(&self, row: &RowSpec, fixture: &Fixture) -> Result<Buffers> {
        let num_sets = match row.mode {
            crate::types::TimingMode::SerialLatency => 1,
            crate::types::TimingMode::IndependentThroughput => row.iterations,
        };
        let mut weights = Vec::new();
        for w in fixture.weights.iter() {
            let size = w.len();
            let buf = self.hip.malloc(size.max(1))?;
            self.hip.memcpy_htod(&buf, w)?;
            weights.push(buf);
        }
        let x_bytes: Vec<u8> = fixture.x_f16.iter().flat_map(|v| v.to_ne_bytes()).collect();
        let x_buf = self.hip.malloc(x_bytes.len().max(1))?;
        self.hip.memcpy_htod(&x_buf, &x_bytes)?;
        let mut y_sets = Vec::new();
        let mut y_staging = Vec::new();
        for _ in 0..num_sets {
            let mut set = Vec::new();
            let mut staging_set = Vec::new();
            for init in fixture.y_init.iter() {
                let bytes: Vec<u8> = init.iter().flat_map(|f| f.to_ne_bytes()).collect();
                let buf = self.hip.malloc(bytes.len().max(4))?;
                self.hip.memcpy_htod(&buf, &bytes)?;
                set.push(buf);
                let staging = self.hip.malloc(bytes.len().max(4))?;
                self.hip.memcpy_htod(&staging, &bytes)?;
                staging_set.push(staging);
            }
            y_sets.push(set);
            y_staging.push(staging_set);
        }
        self.hip.device_synchronize()?;
        Ok(Buffers {
            weights,
            x: x_buf,
            y_sets,
            y_staging,
            shape_proj_m: row.shape.proj_m.clone(),
            n_tokens: row.shape.n_tokens,
        })
    }

    pub fn reset_buffers(&self, buffers: &Buffers, fixture: &Fixture) -> Result<()> {
        // Discriminator 2: if Y_RESET=d2d, use D2D from staging (blit kernel path when SDMA disabled, or D2D SDMA)
        if std::env::var("Y_RESET").as_deref() == Ok("d2d") {
            return self.reset_buffers_d2d(buffers);
        }
        for set in buffers.y_sets.iter() {
            for (proj_idx, buf) in set.iter().enumerate() {
                let init = &fixture.y_init[proj_idx];
                let bytes: Vec<u8> = init.iter().flat_map(|f| f.to_ne_bytes()).collect();
                self.hip.memcpy_htod(buf, &bytes)?;
            }
        }
        self.hip.device_synchronize()?;
        Ok(())
    }

    pub fn reset_buffers_d2d(&self, buffers: &Buffers) -> Result<()> {
        // D2D copy from staging (device) to y_sets (device) - discriminator 2
        for (set_idx, set) in buffers.y_sets.iter().enumerate() {
            for (proj_idx, buf) in set.iter().enumerate() {
                let staging = &buffers.y_staging[set_idx][proj_idx];
                self.hip.memcpy_dtod_at(buf, 0, staging, 0, buf.size())?;
            }
        }
        self.hip.device_synchronize()?;
        Ok(())
    }


    pub fn reset_buffers(&self, buffers: &Buffers, fixture: &Fixture) -> Result<()> {
        for set in buffers.y_sets.iter() {
            for (proj_idx, buf) in set.iter().enumerate() {
                let init = &fixture.y_init[proj_idx];
                let bytes: Vec<u8> = init.iter().flat_map(|f| f.to_ne_bytes()).collect();
                self.hip.memcpy_htod(buf, &bytes)?;
            }
        }
        self.hip.device_synchronize()?;
        Ok(())
    }

    pub fn read_buffers(&self, buffers: &Buffers) -> Result<Vec<Vec<Vec<f32>>>> {
        let mut out_sets = Vec::new();
        for set in &buffers.y_sets {
            let mut proj_out = Vec::new();
            for buf in set.iter() {
                let len = buf.size() / 4;
                let mut bytes = vec![0u8; len * 4];
                self.hip.memcpy_dtoh(&mut bytes, buf)?;
                let floats: Vec<f32> = bytes.chunks_exact(4).map(|c| f32::from_ne_bytes(c.try_into().unwrap())).collect();
                proj_out.push(floats);
            }
            out_sets.push(proj_out);
        }
        Ok(out_sets)
    }

    pub fn free_buffers(&self, buffers: Buffers) -> Result<()> {
        for w in buffers.weights { self.hip.free(w)?; }
        self.hip.free(buffers.x)?;
        for set in buffers.y_sets { for b in set { self.hip.free(b)?; } }
        Ok(())
    }

    fn launch_one(&self, row: &RowSpec, buffers: &Buffers, y_set_idx: usize, stream: &Stream) -> Result<()> {
        let grid = kernels::grid(&row.kernel, &row.shape);
        let block = row.kernel.block();
        let layout = kernels::kernarg_layout(&row.kernel);
        let mut blob = vec![0u8; layout.explicit_size as usize];
        pack_kernarg(&layout, &row.kernel, &row.shape, buffers, y_set_idx, &mut blob)?;
        let func = self.function_for(row)?;
        unsafe {
            self.hip.launch_kernel_blob(
                func,
                [grid[0], grid[1], grid[2]],
                [block, 1, 1],
                0,
                Some(stream),
                &mut blob,
            )?;
        }
        Ok(())
    }
}

impl Backend for HipBackend {
    fn name(&self) -> &'static str { "hip" }

    fn run(&mut self, row: &RowSpec, fixture: &Fixture, warmups: usize, samples: usize) -> Result<RunOutput> {
        let code = kernels::code_object(self.arch, row.scheduler_profile);
        if code.is_empty() {
            anyhow::bail!("HSACO missing for arch {} profile {}", self.arch.as_str(), row.scheduler_profile.as_str());
        }
        let buffers = self.allocate_buffers(row, fixture)?;
        let iterations = row.iterations;
        let is_serial = row.mode == crate::types::TimingMode::SerialLatency;
        let mut gpu_samples_us = Vec::with_capacity(samples);
        if is_serial {
            let stream = self.hip.stream_create()?;
            let start = self.hip.event_create()?;
            let stop = self.hip.event_create()?;
            for _ in 0..warmups {
                self.reset_buffers(&buffers, fixture)?;
                self.hip.event_record(&start, Some(&stream))?;
                for _ in 0..iterations {
                    self.launch_one(row, &buffers, 0, &stream)?;
                }
                self.hip.event_record(&stop, Some(&stream))?;
                self.hip.event_synchronize(&stop)?;
                let _ = self.hip.event_elapsed_ms(&start, &stop)?;
            }
            for _ in 0..samples {
                self.reset_buffers(&buffers, fixture)?;
                self.hip.event_record(&start, Some(&stream))?;
                for _ in 0..iterations {
                    self.launch_one(row, &buffers, 0, &stream)?;
                }
                self.hip.event_record(&stop, Some(&stream))?;
                self.hip.event_synchronize(&stop)?;
                let ms = self.hip.event_elapsed_ms(&start, &stop)? as f64;
                gpu_samples_us.push(ms * 1000.0 / iterations as f64);
            }
            self.hip.event_destroy(start)?;
            self.hip.event_destroy(stop)?;
            self.hip.stream_destroy(stream)?;
        } else {
            let lanes = iterations.min(4);
            let streams: Vec<Stream> = (0..lanes).map(|_| self.hip.stream_create()).collect::<Result<Vec<_>, _>>()?;
            let coord = self.hip.stream_create()?;
            let start = self.hip.event_create()?;
            let stop = self.hip.event_create()?;
            let done_events: Vec<hip_bridge::Event> = (0..lanes).map(|_| self.hip.event_create()).collect::<Result<Vec<_>, _>>()?;
            for _ in 0..warmups {
                self.reset_buffers(&buffers, fixture)?;
                self.hip.event_record(&start, Some(&coord))?;
                for s in &streams { self.hip.stream_wait_event(s, &start)?; }
                for op in 0..iterations {
                    let lane = op % lanes;
                    self.launch_one(row, &buffers, op, &streams[lane])?;
                }
                for (lane, stream) in streams.iter().enumerate() {
                    self.hip.event_record(&done_events[lane], Some(stream))?;
                    self.hip.stream_wait_event(&coord, &done_events[lane])?;
                }
                self.hip.event_record(&stop, Some(&coord))?;
                self.hip.event_synchronize(&stop)?;
                let _ = self.hip.event_elapsed_ms(&start, &stop)?;
            }
            for _ in 0..samples {
                self.reset_buffers(&buffers, fixture)?;
                self.hip.event_record(&start, Some(&coord))?;
                for s in &streams { self.hip.stream_wait_event(s, &start)?; }
                for op in 0..iterations {
                    let lane = op % lanes;
                    self.launch_one(row, &buffers, op, &streams[lane])?;
                }
                for (lane, stream) in streams.iter().enumerate() {
                    self.hip.event_record(&done_events[lane], Some(stream))?;
                    self.hip.stream_wait_event(&coord, &done_events[lane])?;
                }
                self.hip.event_record(&stop, Some(&coord))?;
                self.hip.event_synchronize(&stop)?;
                let ms = self.hip.event_elapsed_ms(&start, &stop)? as f64;
                gpu_samples_us.push(ms * 1000.0 / iterations as f64);
            }
            for e in done_events { self.hip.event_destroy(e)?; }
            self.hip.event_destroy(start)?;
            self.hip.event_destroy(stop)?;
            for s in streams { self.hip.stream_destroy(s)?; }
            self.hip.stream_destroy(coord)?;
        }
        let outputs = self.read_buffers(&buffers)?;
        let notes = serde_json::json!({
            "arch": self.arch_str,
            "device_ordinal": self.device_ordinal,
        });
        self.free_buffers(buffers)?;
        Ok(RunOutput { outputs, samples_us: gpu_samples_us, notes })
    }
}

pub struct HipGraphBackend {
    inner: HipBackend,
}

impl HipGraphBackend {
    pub fn new(device_ordinal: i32) -> Result<Self> {
        Ok(Self { inner: HipBackend::new(device_ordinal)? })
    }
}

impl Backend for HipGraphBackend {
    fn name(&self) -> &'static str { "hipgraph" }

    fn run(&mut self, row: &RowSpec, fixture: &Fixture, warmups: usize, samples: usize) -> Result<RunOutput> {
        let code = kernels::code_object(self.inner.arch, row.scheduler_profile);
        if code.is_empty() {
            anyhow::bail!("HSACO missing for arch {} profile {}", self.inner.arch.as_str(), row.scheduler_profile.as_str());
        }
        self.inner.function_for(row)?;
        let buffers = self.inner.allocate_buffers(row, fixture)?;
        let iterations = row.iterations;
        let is_serial = row.mode == crate::types::TimingMode::SerialLatency;
        let mut gpu_samples_us = Vec::with_capacity(samples);
        if is_serial {
            let stream = self.inner.hip.stream_create()?;
            let start = self.inner.hip.event_create()?;
            let stop = self.inner.hip.event_create()?;
            self.inner.hip.stream_begin_capture(&stream, 0)?;
            for _ in 0..iterations {
                self.inner.launch_one(row, &buffers, 0, &stream)?;
            }
            let graph = self.inner.hip.stream_end_capture(&stream)?;
            let exec = self.inner.hip.graph_instantiate(&graph)?;
            let time_once = |hip: &HipRuntime| -> Result<f64> {
                hip.event_record(&start, Some(&stream))?;
                hip.graph_launch(&exec, &stream)?;
                hip.event_record(&stop, Some(&stream))?;
                hip.event_synchronize(&stop)?;
                let ms = hip.event_elapsed_ms(&start, &stop)? as f64;
                Ok(ms * 1000.0 / iterations as f64)
            };
            for _ in 0..warmups {
                self.inner.reset_buffers(&buffers, fixture)?;
                time_once(&self.inner.hip)?;
            }
            for _ in 0..samples {
                self.inner.reset_buffers(&buffers, fixture)?;
                gpu_samples_us.push(time_once(&self.inner.hip)?);
            }
            self.inner.hip.graph_exec_destroy(exec)?;
            self.inner.hip.graph_destroy(graph)?;
            self.inner.hip.event_destroy(start)?;
            self.inner.hip.event_destroy(stop)?;
            self.inner.hip.stream_destroy(stream)?;
        } else {
            let lanes = iterations.min(4);
            let streams: Vec<Stream> = (0..lanes).map(|_| self.inner.hip.stream_create()).collect::<Result<Vec<_>, _>>()?;
            let coord = self.inner.hip.stream_create()?;
            let start = self.inner.hip.event_create()?;
            let stop = self.inner.hip.event_create()?;
            let done_events: Vec<hip_bridge::Event> = (0..lanes).map(|_| self.inner.hip.event_create()).collect::<Result<Vec<_>, _>>()?;
            let mut graphs = Vec::new();
            let mut execs = Vec::new();
            for lane in 0..lanes {
                self.inner.hip.stream_begin_capture(&streams[lane], 0)?;
                for op in (lane..iterations).step_by(lanes) {
                    self.inner.launch_one(row, &buffers, op, &streams[lane])?;
                }
                let g = self.inner.hip.stream_end_capture(&streams[lane])?;
                let e = self.inner.hip.graph_instantiate(&g)?;
                graphs.push(g);
                execs.push(e);
            }
            let time_once = |hip: &HipRuntime| -> Result<f64> {
                hip.event_record(&start, Some(&coord))?;
                for s in &streams { hip.stream_wait_event(s, &start)?; }
                for lane in 0..lanes {
                    hip.graph_launch(&execs[lane], &streams[lane])?;
                    hip.event_record(&done_events[lane], Some(&streams[lane]))?;
                    hip.stream_wait_event(&coord, &done_events[lane])?;
                }
                hip.event_record(&stop, Some(&coord))?;
                hip.event_synchronize(&stop)?;
                let ms = hip.event_elapsed_ms(&start, &stop)? as f64;
                Ok(ms * 1000.0 / iterations as f64)
            };
            for _ in 0..warmups {
                self.inner.reset_buffers(&buffers, fixture)?;
                time_once(&self.inner.hip)?;
            }
            for _ in 0..samples {
                self.inner.reset_buffers(&buffers, fixture)?;
                gpu_samples_us.push(time_once(&self.inner.hip)?);
            }
            for e in execs { self.inner.hip.graph_exec_destroy(e)?; }
            for g in graphs { self.inner.hip.graph_destroy(g)?; }
            for e in done_events { self.inner.hip.event_destroy(e)?; }
            self.inner.hip.event_destroy(start)?;
            self.inner.hip.event_destroy(stop)?;
            for s in streams { self.inner.hip.stream_destroy(s)?; }
            self.inner.hip.stream_destroy(coord)?;
        }
        let outputs = self.inner.read_buffers(&buffers)?;
        let notes = serde_json::json!({
            "arch": self.inner.arch_str,
            "device_ordinal": self.inner.device_ordinal,
        });
        self.inner.free_buffers(buffers)?;
        Ok(RunOutput { outputs, samples_us: gpu_samples_us, notes })
    }
}
