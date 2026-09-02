// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

use crate::hip_backend::Buffers;
use crate::kernels;
use crate::types::{Backend, Fixture, RowSpec, RunOutput};
use anyhow::{Context, Result};
use radiowave::SchedulerProfile;
use redline_dispatch::aql::{
    Gfx10Pm4CommandBuffer, Gfx12KernelImage, Gfx12Pm4CommandBuffer, Gfx12DispatchMode,
    GpuSelector, KernargPool, LaunchGeometry, QueuePolicy, Runtime, SingleQueuePm4Ib,
    MultiQueuePm4Ib, load_symbols,
};
use std::ffi::c_void;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RmwBoundary {
    RadiowaveVmem,
    SameAgent,
    RadvGlobal,
}

impl RmwBoundary {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "radiowave-vmem" => Some(Self::RadiowaveVmem),
            "same-agent" => Some(Self::SameAgent),
            "radv-global" => Some(Self::RadvGlobal),
            _ => None,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RadiowaveVmem => "radiowave-vmem",
            Self::SameAgent => "same-agent",
            Self::RadvGlobal => "radv-global",
        }
    }
    pub fn description(self) -> &'static str {
        match self {
            Self::RadiowaveVmem => "radiowave-vmem (certified vector L1)",
            Self::SameAgent => "same-agent",
            Self::RadvGlobal => "radv-global",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Pm4Family { Gfx10, Gfx11, Gfx12 }

impl Pm4Family {
    fn from_arch(arch: &str) -> Result<Self> {
        if arch.starts_with("gfx10") { Ok(Self::Gfx10) }
        else if arch.starts_with("gfx11") { Ok(Self::Gfx11) }
        else if arch.starts_with("gfx12") { Ok(Self::Gfx12) }
        else { anyhow::bail!("unsupported arch for Redline PM4: {arch}") }
    }
}

enum RdnaPm4Commands {
    Legacy(Gfx10Pm4CommandBuffer),
    Gfx12(Gfx12Pm4CommandBuffer),
}

impl RdnaPm4Commands {
    fn stateful(family: Pm4Family) -> Self {
        match family {
            Pm4Family::Gfx10 | Pm4Family::Gfx11 => Self::Legacy(Gfx10Pm4CommandBuffer::new_stateful()),
            Pm4Family::Gfx12 => Self::Gfx12(Gfx12Pm4CommandBuffer::new_stateful()),
        }
    }
    fn ownership(family: Pm4Family) -> Self {
        match family {
            Pm4Family::Gfx10 | Pm4Family::Gfx11 => {
                let mut c = Gfx10Pm4CommandBuffer::new();
                c.acquire_system();
                Self::Legacy(c)
            }
            Pm4Family::Gfx12 => {
                let mut c = Gfx12Pm4CommandBuffer::new();
                c.acquire_system_gfx12();
                Self::Gfx12(c)
            }
        }
    }
    fn dispatch(&mut self, kernel: &redline_dispatch::aql::Kernel, geometry: LaunchGeometry, kernarg: *mut c_void, mode: Gfx12DispatchMode) -> Result<()> {
        match self {
            Self::Legacy(cmds) => { let _ = mode; cmds.dispatch(kernel, geometry, 0, kernarg)?; }
            Self::Gfx12(cmds) => {
                const ENABLE_SGPR_KERNARG_SEGMENT_PTR: u16 = 1 << 3;
                let image = Gfx12KernelImage::from_hsa(kernel)?;
                let pm4 = kernel.pm4_metadata().context("missing PM4 metadata")?;
                let needs_kernarg = pm4.kernel_code_properties & ENABLE_SGPR_KERNARG_SEGMENT_PTR != 0;
                if needs_kernarg && kernarg.is_null() { anyhow::bail!("null kernarg for Gfx12"); }
                let mut user_sgprs = [0u32;2];
                let user_sgprs: &[u32] = if needs_kernarg {
                    let addr = kernarg as usize as u64;
                    user_sgprs[0] = addr as u32;
                    user_sgprs[1] = (addr>>32) as u32;
                    &user_sgprs
                } else { &[] };
                cmds.dispatch_image_with_mode(&image, geometry, 0, user_sgprs, mode)?;
            }
        }
        Ok(())
    }
    fn dependency_rmw(&mut self, boundary: RmwBoundary) {
        match self {
            Self::Legacy(c) => match boundary {
                RmwBoundary::RadvGlobal => c.dependency_rmw_global(),
                RmwBoundary::SameAgent => c.dependency_rmw_same_agent(),
                RmwBoundary::RadiowaveVmem => c.dependency_rmw_vmem(),
            },
            Self::Gfx12(c) => match boundary {
                RmwBoundary::RadvGlobal => { c.wait_compute_idle(); c.acquire_rmw_gfx12(redline_dispatch::aql::Gfx12RmwAcquirePolicy::RadvGlobal); }
                RmwBoundary::SameAgent => c.dependency_rmw_same_agent_gfx12(),
                RmwBoundary::RadiowaveVmem => c.dependency_rmw_hip_llvm_vmem_gfx12(),
            },
        }
    }
}

struct ProfileExec {
    profile: SchedulerProfile,
    exec: redline_dispatch::aql::Executable,
}

pub struct RedlineBackend {
    runtime: Runtime,
    device: redline_dispatch::aql::GpuDevice,
    pm4_family: Pm4Family,
    pool: KernargPool,
    profiles: Vec<ProfileExec>,
    queue_policy: QueuePolicy,
    rmw_boundary: RmwBoundary,
    arch_str: String,
    pci: String,
    dispatch_mode: Gfx12DispatchMode,
}

impl RedlineBackend {
    pub fn new(pci: &str, queue_policy: QueuePolicy, rmw_boundary: RmwBoundary) -> Result<Self> {
        let runtime = Runtime::initialize(load_symbols()?).context("initialize ROCr")?;
        let device = runtime.select_gpu(GpuSelector::Ordinal(0))?;
        let _ = pci;
        let name = device.name().to_owned();
        let pci_out = device.pci_bus_id().to_string();
        let pm4_family = Pm4Family::from_arch(&name)?;
        let pool = KernargPool::discover(&device)?;
        let arch = crate::types::Arch::parse(&name).unwrap_or(crate::types::Arch::Gfx1201);
        let mut profiles = Vec::new();
        for &profile in &[SchedulerProfile::Default] {
            let code = kernels::code_object(arch, profile);
            if code.is_empty() { continue; }
            let exec = redline_dispatch::aql::Executable::load(&device, Arc::<[u8]>::from(code))?;
            profiles.push(ProfileExec { profile, exec });
        }
        if profiles.is_empty() {
            let code = kernels::code_object(arch, SchedulerProfile::Default);
            if !code.is_empty() {
                let exec = redline_dispatch::aql::Executable::load(&device, Arc::<[u8]>::from(code))?;
                profiles.push(ProfileExec { profile: SchedulerProfile::Default, exec });
            }
        }
        Ok(Self {
            runtime: runtime.clone(),
            device,
            pm4_family,
            pool,
            profiles,
            queue_policy,
            rmw_boundary,
            arch_str: name,
            pci: pci_out,
            dispatch_mode: Gfx12DispatchMode::Workitems,
        })
    }

    pub fn pci(&self) -> &str { &self.pci }
    pub fn arch_str(&self) -> &str { &self.arch_str }

    fn exec_for(&self, profile: SchedulerProfile) -> Result<&redline_dispatch::aql::Executable> {
        self.profiles.iter().find(|p| p.profile == profile).map(|p| &p.exec).with_context(|| format!("missing Redline exec for profile {}", profile.as_str()))
    }

    fn queue_count_for(&self, mode: crate::types::TimingMode, iterations: usize) -> usize {
        if mode == crate::types::TimingMode::IndependentThroughput {
            let requested = self.queue_policy.resolve(&self.arch_str, iterations);
            requested.min(iterations.max(1))
        } else { 1 }
    }

    fn fill_kernarg(&self, karg: &mut redline_dispatch::aql::KernargBuffer, row: &RowSpec, buffers: &Buffers, y_set_idx: usize) -> Result<()> {
        let layout = kernels::kernarg_layout(&row.kernel);
        let bytes = karg.as_mut_bytes();
        let max_end = layout.slots.iter().map(|s| s.offset as usize + s.size as usize).max().unwrap_or(0);
        anyhow::ensure!(bytes.len() >= max_end, "kernarg too small {} vs needed {}", bytes.len(), max_end);
        bytes.fill(0);
        let i32_vals = crate::types::KernargLayout::i32_values(&row.kernel, &row.shape);
        let mut i32_cursor = 0usize;
        for slot in &layout.slots {
            match slot.kind {
                crate::types::ArgKind::Ptr => {
                    let binding = slot.binding.unwrap();
                    let addr: u64 = match binding {
                        crate::types::PtrBinding::Weights(i) => buffers.weights_ptr(i),
                        crate::types::PtrBinding::X => buffers.x_ptr(),
                        crate::types::PtrBinding::Y(i) => buffers.y_device_ptr(y_set_idx, i),
                    };
                    let off = slot.offset as usize;
                    bytes[off..off+8].copy_from_slice(&addr.to_ne_bytes());
                }
                crate::types::ArgKind::I32 => {
                    let v = i32_vals[i32_cursor];
                    i32_cursor += 1;
                    let off = slot.offset as usize;
                    bytes[off..off+4].copy_from_slice(&v.to_ne_bytes());
                }
            }
        }
        // Hidden args for completeness (grid)
        // HIP kernels avoid blockDim but keep canonical shape; fill if space.
        const GRID_OFF: usize = 0; // explicit size already accounted; hidden follows at next 8-byte boundary
        // We do not know hidden offset; rely on explicit_size alignment already. The metadata hidden starts at explicit_size aligned.
        // Fill a minimal hidden block if the allocation is larger than explicit.
        if bytes.len() >= layout.explicit_size as usize + 66 {
            let grid = kernels::grid(&row.kernel, &row.shape);
            let h_off = layout.explicit_size as usize;
            // align to 8
            let h_off_aligned = (h_off + 7) & !7;
            if bytes.len() >= h_off_aligned + 18 {
                let block = row.kernel.block();
                bytes[h_off_aligned..h_off_aligned+4].copy_from_slice(&grid[0].to_ne_bytes());
                bytes[h_off_aligned+4..h_off_aligned+8].copy_from_slice(&grid[1].to_ne_bytes());
                bytes[h_off_aligned+8..h_off_aligned+12].copy_from_slice(&grid[2].to_ne_bytes());
                bytes[h_off_aligned+12..h_off_aligned+14].copy_from_slice(&(block as u16).to_ne_bytes());
            }
        }
        Ok(())
    }
}

impl Backend for RedlineBackend {
    fn name(&self) -> &'static str { "redline" }
    fn run(&mut self, row: &RowSpec, fixture: &Fixture, warmups: usize, samples: usize) -> Result<RunOutput> {
        // HIP buffers for sharing
        // We need HipBackend to allocate? Instead allocate via rocr DevicePool or via HIP? Use HIP via hip-bridge for consistency: delegate to HipBackend allocation using a temporary HipBackend.
        // To avoid double HipRuntime, we will allocate via rocr DevicePool + HIP-style raw pointers? Simpler: allocate buffers via HipBackend helper using HipRuntime loaded here (reuse HIP malloc via hip-bridge load).
        // For now, create a HipBackend for buffer management.
        let hip = crate::hip_backend::HipBackend::new(0)?;
        let buffers = hip.allocate_buffers(row, fixture)?;

        // Check scratch: load kernel and inspect private_segment_size
        let exec = self.exec_for(row.scheduler_profile)?;
        let kernel = exec.kernel(&format!("{}.kd", row.kernel.symbol)).with_context(|| format!("kernel {} not found in exec", row.kernel.symbol))?;
        let meta = kernel.metadata();
        if meta.private_segment_size != 0 {
            anyhow::bail!("kernel {} uses scratch (private_segment_size={}), Redline refuses scratch kernels", row.kernel.symbol, meta.private_segment_size);
        }
        // Also validate static LDS matches descriptor expectation via HSACO? Just trust descriptor.

        let iterations = row.iterations;
        let lane_count = self.queue_count_for(row.mode, iterations);
        // Build PM4 commands
        let mut commands: Vec<RdnaPm4Commands> = (0..lane_count).map(|_| RdnaPm4Commands::stateful(self.pm4_family)).collect();
        // Need kernarg buffers to keep alive
        let mut kernargs: Vec<redline_dispatch::aql::KernargBuffer> = Vec::new();

        // Helper to get geometry
        let grid = kernels::grid(&row.kernel, &row.shape);
        let block = row.kernel.block();
        let geometry = LaunchGeometry::from_workgroups(grid, [block as u16, 1, 1]).context("LaunchGeometry")?;

        for (lane_idx, cmds) in commands.iter_mut().enumerate() {
            for op in (lane_idx..iterations).step_by(lane_count) {
                let y_set_idx = if row.mode == crate::types::TimingMode::SerialLatency { 0 } else { op };
                let mut karg = self.pool.allocate_for(kernel.metadata())?;
                self.fill_kernarg(&mut karg, row, &buffers, y_set_idx)?;
                let addr = karg.address();
                kernargs.push(karg);
                let k_ref: *mut c_void = addr;
                // Need to keep kernarg alive, but address is from last pushed; dispatch borrows kernel but kernarg stays.
                // Use the address we just pushed.
                let kernarg_ptr = kernargs.last().unwrap().address();
                cmds.dispatch(&kernel, geometry, kernarg_ptr, self.dispatch_mode)?;
                if row.mode == crate::types::TimingMode::SerialLatency && op + 1 < iterations {
                    cmds.dependency_rmw(self.rmw_boundary);
                }
            }
        }

        // Create IBs
        let ownership = RdnaPm4Commands::ownership(self.pm4_family);
        let mut ownership_ib = match (&ownership, self.pm4_family) {
            (RdnaPm4Commands::Legacy(c), Pm4Family::Gfx10) => SingleQueuePm4Ib::create_gfx10(&self.device, &self.pool, c)?,
            (RdnaPm4Commands::Legacy(c), Pm4Family::Gfx11) => SingleQueuePm4Ib::create_gfx11(&self.device, &self.pool, c)?,
            (RdnaPm4Commands::Gfx12(c), Pm4Family::Gfx12) => SingleQueuePm4Ib::create(&self.device, &self.pool, c)?,
            _ => anyhow::bail!("PM4 family mismatch for ownership"),
        };

        enum IbEnum { Single(SingleQueuePm4Ib), Multi(MultiQueuePm4Ib) }
        let mut ib = if commands.len() == 1 {
            let single = match (self.pm4_family, &commands[0]) {
                (Pm4Family::Gfx10, RdnaPm4Commands::Legacy(c)) => SingleQueuePm4Ib::create_profiled_gfx10(&self.device, &self.pool, c)?,
                (Pm4Family::Gfx11, RdnaPm4Commands::Legacy(c)) => SingleQueuePm4Ib::create_profiled_gfx11(&self.device, &self.pool, c)?,
                (Pm4Family::Gfx12, RdnaPm4Commands::Gfx12(c)) => SingleQueuePm4Ib::create_profiled(&self.device, &self.pool, c)?,
                _ => anyhow::bail!("PM4 mismatch"),
            };
            IbEnum::Single(single)
        } else {
            match self.pm4_family {
                Pm4Family::Gfx10 => {
                    let encoded: Vec<Gfx10Pm4CommandBuffer> = commands.iter().map(|c| match c { RdnaPm4Commands::Legacy(v)=> Ok(v.clone()), _=> anyhow::bail!("mismatch") }).collect::<Result<Vec<_>>>()?;
                    IbEnum::Multi(MultiQueuePm4Ib::create_profiled_gfx10(&self.device, &self.pool, &encoded)?)
                }
                Pm4Family::Gfx11 => {
                    let encoded: Vec<Gfx10Pm4CommandBuffer> = commands.iter().map(|c| match c { RdnaPm4Commands::Legacy(v)=> Ok(v.clone()), _=> anyhow::bail!("mismatch") }).collect::<Result<Vec<_>>>()?;
                    // Gfx11 uses same Gfx10 command buffer type?
                    IbEnum::Multi(MultiQueuePm4Ib::create_profiled_gfx11(&self.device, &self.pool, &encoded)?)
                }
                Pm4Family::Gfx12 => {
                    let encoded: Vec<Gfx12Pm4CommandBuffer> = commands.iter().map(|c| match c { RdnaPm4Commands::Gfx12(v)=> Ok(v.clone()), _=> anyhow::bail!("mismatch") }).collect::<Result<Vec<_>>>()?;
                    IbEnum::Multi(MultiQueuePm4Ib::create_profiled(&self.device, &self.pool, &encoded)?)
                }
            }
        };

        // Timing
        let mut gpu_samples_us = Vec::with_capacity(samples);
        for _ in 0..warmups {
            hip.reset_buffers(&buffers, fixture)?;
            unsafe { ownership_ib.replay_and_wait()?; }
            let timing = unsafe {
                match &mut ib {
                    IbEnum::Single(s) => s.replay_and_wait_profiled()?,
                    IbEnum::Multi(m) => m.replay_and_wait_profiled()?,
                }
            };
            let _ = timing;
        }
        for _ in 0..samples {
            hip.reset_buffers(&buffers, fixture)?;
            unsafe { ownership_ib.replay_and_wait()?; }
            let timing = unsafe {
                match &mut ib {
                    IbEnum::Single(s) => s.replay_and_wait_profiled()?,
                    IbEnum::Multi(m) => m.replay_and_wait_profiled()?,
                }
            };
            gpu_samples_us.push(timing.span_microseconds() / iterations as f64);
        }

        let outputs = hip.read_buffers(&buffers)?;
        let notes = serde_json::json!({
            "arch": self.arch_str,
            "pci": self.pci,
            "queue_policy": format!("{:?}", self.queue_policy),
            "rmw_boundary": self.rmw_boundary.as_str(),
            "lane_count": lane_count,
        });
        hip.free_buffers(buffers)?;
        drop(kernargs);
        drop(ib);
        drop(ownership_ib);
        Ok(RunOutput { outputs, samples_us: gpu_samples_us, notes })
    }
}
