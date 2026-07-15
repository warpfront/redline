// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

use crate::common::Measurement;
use crate::hip_backend::{embedded_code_object, HipBackend};
use crate::spec::{Fixture, RowSpec, TimingMode};
use anyhow::{Context, Result};
use radiowave::{
    CodeObjectCertification, CodeObjectInspection, MutableReadCache, SchedulerProfile, Wavefront,
};
use redline_dispatch::aql::{
    load_symbols, Executable, Gfx10Pm4CommandBuffer, Gfx12Pm4CommandBuffer, GpuDevice,
    GpuMultiQueueTiming, GpuSelector, KernargBuffer, KernargPool, Kernel, LaunchGeometry,
    MultiQueuePm4Ib, QueuePolicy, Runtime, SingleQueuePm4Ib,
};
use std::ffi::c_void;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RmwBoundary {
    RadvGlobal,
    SameAgent,
    RadiowaveVmem,
}

impl RmwBoundary {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "radv-global" | "radv_global" => Some(Self::RadvGlobal),
            "same-agent" | "same_agent" => Some(Self::SameAgent),
            "radiowave-vmem" | "radiowave_vmem" | "hip-llvm-vmem" => Some(Self::RadiowaveVmem),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::RadvGlobal => "radv_global",
            Self::SameAgent => "same_agent_shader_caches",
            Self::RadiowaveVmem => "radiowave_certified_vmem",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::RadvGlobal => {
                "Required RMW edges use CS_PARTIAL_FLUSH plus scalar/vector/L1 invalidation and global L2 writeback/invalidation. This is the historical conservative control."
            }
            Self::SameAgent => {
                "Required RMW edges use CS_PARTIAL_FLUSH plus same-agent scalar/vector/L1 invalidation while retaining coherent L2/MALL."
            }
            Self::RadiowaveVmem => {
                "Required RMW edges use CS_PARTIAL_FLUSH plus vector/L1 invalidation for Radiowave-certified VMEM consumers. Scalar or unknown consumers fail closed to scalar/vector/L1 invalidation."
            }
        }
    }
}

pub struct RedlineBackend {
    _runtime: Runtime,
    device: GpuDevice,
    profiles: Vec<ProfileExecutables>,
    pool: KernargPool,
    pm4_family: Pm4Family,
    queue_policy: QueuePolicy,
    independent_queue_count: usize,
    pub name: String,
    pub pci: String,
    pub rmw_boundary: RmwBoundary,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Pm4Family {
    Gfx10,
    Gfx11,
    Gfx12,
}

impl Pm4Family {
    fn from_device(name: &str) -> Result<Self> {
        if name.starts_with("gfx10") {
            Ok(Self::Gfx10)
        } else if name.starts_with("gfx11") {
            Ok(Self::Gfx11)
        } else if name.starts_with("gfx12") {
            Ok(Self::Gfx12)
        } else {
            anyhow::bail!("Redline PM4 benchmark does not support device architecture {name}")
        }
    }
}

enum RdnaPm4Commands {
    Legacy(Gfx10Pm4CommandBuffer),
    Gfx12(Gfx12Pm4CommandBuffer),
}

enum ProfiledPm4Replay {
    Single(SingleQueuePm4Ib),
    Multi(MultiQueuePm4Ib),
}

impl ProfiledPm4Replay {
    unsafe fn replay_and_wait_profiled(&mut self) -> Result<GpuMultiQueueTiming> {
        match self {
            Self::Single(ib) => {
                // SAFETY: forwarded from this method's caller.
                Ok(unsafe { ib.replay_and_wait_profiled()? })
            }
            Self::Multi(ib) => {
                // SAFETY: forwarded from this method's caller.
                Ok(unsafe { ib.replay_and_wait_profiled()? })
            }
        }
    }
}

impl RdnaPm4Commands {
    fn stateful(family: Pm4Family) -> Self {
        match family {
            Pm4Family::Gfx10 | Pm4Family::Gfx11 => {
                Self::Legacy(Gfx10Pm4CommandBuffer::new_stateful())
            }
            Pm4Family::Gfx12 => Self::Gfx12(Gfx12Pm4CommandBuffer::new_stateful()),
        }
    }

    fn ownership(family: Pm4Family) -> Self {
        match family {
            Pm4Family::Gfx10 | Pm4Family::Gfx11 => {
                let mut commands = Gfx10Pm4CommandBuffer::new();
                commands.acquire_system();
                Self::Legacy(commands)
            }
            Pm4Family::Gfx12 => {
                let mut commands = Gfx12Pm4CommandBuffer::new();
                commands.acquire_system_gfx12();
                Self::Gfx12(commands)
            }
        }
    }

    fn dispatch(
        &mut self,
        kernel: &Kernel,
        geometry: LaunchGeometry,
        kernarg_address: *mut c_void,
    ) -> Result<()> {
        match self {
            Self::Legacy(commands) => {
                commands.dispatch(kernel, geometry, 0, kernarg_address)?;
            }
            Self::Gfx12(commands) => {
                commands.dispatch(kernel, geometry, 0, kernarg_address)?;
            }
        }
        Ok(())
    }
}

struct ProfileExecutables {
    scheduler_profile: SchedulerProfile,
    exec_wave32: Executable,
    exec_wave64: Executable,
    inspection_wave32: CodeObjectInspection,
    inspection_wave64: CodeObjectInspection,
}

impl RedlineBackend {
    pub fn new(rmw_boundary: RmwBoundary, queue_policy: QueuePolicy) -> Result<Self> {
        let runtime = Runtime::initialize(load_symbols()?).context("initialize public ROCr")?;
        let device = runtime.select_gpu(GpuSelector::Ordinal(0))?;
        let name = device.name().to_owned();
        let independent_queue_count = queue_policy.resolve(&name, usize::MAX);
        let pm4_family = Pm4Family::from_device(&name)?;
        let pci = device.pci_bus_id().to_string();
        let mut profiles = Vec::with_capacity(SchedulerProfile::ALL.len());
        for scheduler_profile in SchedulerProfile::ALL {
            let wave32 = embedded_code_object(scheduler_profile, Wavefront::Wave32);
            let wave64 = embedded_code_object(scheduler_profile, Wavefront::Wave64);
            profiles.push(ProfileExecutables {
                scheduler_profile,
                exec_wave32: Executable::load(&device, Arc::<[u8]>::from(wave32.code))?,
                exec_wave64: Executable::load(&device, Arc::<[u8]>::from(wave64.code))?,
                inspection_wave32: inspection_from_manifest(
                    wave32.manifest,
                    wave32.code,
                    Wavefront::Wave32,
                    scheduler_profile,
                )?,
                inspection_wave64: inspection_from_manifest(
                    wave64.manifest,
                    wave64.code,
                    Wavefront::Wave64,
                    scheduler_profile,
                )?,
            });
        }
        let pool = KernargPool::discover(&device)?;
        Ok(Self {
            _runtime: runtime,
            device,
            profiles,
            pool,
            pm4_family,
            queue_policy,
            independent_queue_count,
            name,
            pci,
            rmw_boundary,
        })
    }

    pub fn measure(
        &self,
        hip: &HipBackend,
        spec: &RowSpec,
        fixture: &Fixture,
        mode: TimingMode,
        warmups: usize,
        samples: usize,
    ) -> Result<Measurement> {
        let buffers = hip.allocate(spec, fixture, mode)?;
        let iterations = spec.logical_iterations(mode);
        let profile = self
            .profiles
            .iter()
            .find(|profile| profile.scheduler_profile == spec.scheduler_profile)
            .with_context(|| {
                format!(
                    "missing Redline scheduler profile {}",
                    spec.scheduler_profile.as_str()
                )
            })?;
        let exec = match spec.wave_size {
            32 => &profile.exec_wave32,
            64 => &profile.exec_wave64,
            other => anyhow::bail!("unsupported Redline wave size {other}"),
        };
        let lane_count = self.queue_count_for(mode, iterations);
        let mut commands = (0..lane_count)
            .map(|_| RdnaPm4Commands::stateful(self.pm4_family))
            .collect::<Vec<_>>();
        let mut kernargs: Vec<KernargBuffer> = Vec::new();
        // Serial and one-shot tapes repeat identical arguments. Keep one
        // immutable kernarg block per kernel so its scalar prologue stays hot
        // and stateful PM4 can elide redundant COMPUTE_USER_DATA writes.
        // Independent rows encode a distinct output offset per operation.
        let reuse_kernargs = mode != TimingMode::IndependentThroughput;
        let primary_reused_address = if reuse_kernargs {
            let kernel = exec.kernel(&format!("{}.kd", spec.kernel))?;
            let mut karg = self.pool.allocate_for(kernel.metadata())?;
            fill_kernarg(&mut karg, &buffers, spec, 0, spec.grid_groups, false)?;
            let address = karg.address();
            kernargs.push(karg);
            Some(address)
        } else {
            None
        };
        let secondary_reused_address = if reuse_kernargs {
            if let Some(second) = spec.second_kernel {
                let kernel = exec.kernel(&format!("{second}.kd"))?;
                let mut karg = self.pool.allocate_for(kernel.metadata())?;
                fill_kernarg(
                    &mut karg,
                    &buffers,
                    spec,
                    spec.second_output_delta,
                    spec.second_grid_groups,
                    true,
                )?;
                let address = karg.address();
                kernargs.push(karg);
                Some(address)
            } else {
                None
            }
        } else {
            None
        };
        for (lane, lane_commands) in commands.iter_mut().enumerate() {
            for operation in lane_operations(iterations, lane_count, lane) {
                let offset = spec.stage_output_offset(mode, operation, false);
                let kernel = exec.kernel(&format!("{}.kd", spec.kernel))?;
                let karg_address = if let Some(address) = primary_reused_address {
                    address
                } else {
                    let mut karg = self.pool.allocate_for(kernel.metadata())?;
                    fill_kernarg(&mut karg, &buffers, spec, offset, spec.grid_groups, false)?;
                    let address = karg.address();
                    kernargs.push(karg);
                    address
                };
                let geometry = LaunchGeometry::new(
                    [spec.grid_groups * spec.block, 1, 1],
                    [spec.block as u16, 1, 1],
                )?;
                lane_commands.dispatch(&kernel, geometry, karg_address)?;

                if let Some(second) = spec.second_kernel {
                    self.dependency_boundary(
                        lane_commands,
                        second,
                        spec.wave_size,
                        spec.scheduler_profile,
                    );
                    let kernel = exec.kernel(&format!("{second}.kd"))?;
                    let karg_address = if let Some(address) = secondary_reused_address {
                        address
                    } else {
                        let mut karg = self.pool.allocate_for(kernel.metadata())?;
                        fill_kernarg(
                            &mut karg,
                            &buffers,
                            spec,
                            spec.stage_output_offset(mode, operation, true),
                            spec.second_grid_groups,
                            true,
                        )?;
                        let address = karg.address();
                        kernargs.push(karg);
                        address
                    };
                    let geometry = LaunchGeometry::new(
                        [spec.second_grid_groups * spec.stage_block(true), 1, 1],
                        [spec.stage_block(true) as u16, 1, 1],
                    )?;
                    lane_commands.dispatch(&kernel, geometry, karg_address)?;
                }
                if mode == TimingMode::SerialLatency && operation + 1 < iterations {
                    self.dependency_boundary(
                        lane_commands,
                        spec.kernel,
                        spec.wave_size,
                        spec.scheduler_profile,
                    );
                }
            }
        }
        // HIP initializes the resource before every replay. Transfer ownership
        // with a separate, completed acquire tape so the timed retained tape
        // contains only dispatches and their intra-tape dependency boundaries.
        // This is the same safe handoff previously used by the aggressive path.
        let acquire = RdnaPm4Commands::ownership(self.pm4_family);
        let mut ownership = self.create_ib(&acquire, false)?;
        let mut ib = self.create_profiled_ib(&commands)?;
        for _ in 0..warmups {
            hip.reset(&buffers.out)?;
            unsafe { ownership.replay_and_wait()? };
            unsafe { ib.replay_and_wait_profiled()? };
        }
        let mut gpu_samples_us = Vec::with_capacity(samples);
        for _ in 0..samples {
            hip.reset(&buffers.out)?;
            unsafe { ownership.replay_and_wait()? };
            let timing = unsafe { ib.replay_and_wait_profiled()? };
            gpu_samples_us.push(timing.span_microseconds() / iterations as f64);
        }
        let output = hip.read_output(&buffers.out, spec.output_words(mode))?;
        drop(ib);
        drop(ownership);
        drop(kernargs);
        hip.free_buffers(buffers)?;
        Ok(Measurement {
            gpu_samples_us,
            output,
        })
    }

    pub fn queue_count_for(&self, mode: TimingMode, iterations: usize) -> usize {
        active_queue_count(mode, self.independent_queue_count, iterations)
    }

    pub fn queue_policy(&self) -> QueuePolicy {
        self.queue_policy
    }

    pub fn independent_queue_count(&self) -> usize {
        self.independent_queue_count
    }

    pub fn dependency_policy_name(
        &self,
        kernel: &str,
        wave_size: u32,
        scheduler_profile: SchedulerProfile,
    ) -> &'static str {
        match self.rmw_boundary {
            RmwBoundary::RadvGlobal => "global_l2_wb_inv_0x0c380",
            RmwBoundary::SameAgent => "scalar_vector_l1_0x00380",
            RmwBoundary::RadiowaveVmem => {
                match self.mutable_read_cache(kernel, wave_size, scheduler_profile) {
                    MutableReadCache::VmemOnly => "certified_vector_l1_0x00300",
                    MutableReadCache::ScalarOrUnknown => "fallback_scalar_vector_l1_0x00380",
                }
            }
        }
    }

    fn dependency_boundary(
        &self,
        commands: &mut RdnaPm4Commands,
        consumer: &str,
        wave_size: u32,
        scheduler_profile: SchedulerProfile,
    ) {
        let cache = self.mutable_read_cache(consumer, wave_size, scheduler_profile);
        match commands {
            RdnaPm4Commands::Legacy(commands) => match self.rmw_boundary {
                RmwBoundary::RadvGlobal => commands.dependency_rmw_global(),
                RmwBoundary::SameAgent => commands.dependency_rmw_same_agent(),
                RmwBoundary::RadiowaveVmem => match cache {
                    MutableReadCache::VmemOnly => commands.dependency_rmw_vmem(),
                    MutableReadCache::ScalarOrUnknown => commands.dependency_rmw_same_agent(),
                },
            },
            RdnaPm4Commands::Gfx12(commands) => match self.rmw_boundary {
                RmwBoundary::RadvGlobal => {
                    commands.wait_compute_idle();
                    commands.acquire_rmw_gfx12(
                        redline_dispatch::aql::Gfx12RmwAcquirePolicy::RadvGlobal,
                    );
                }
                RmwBoundary::SameAgent => commands.dependency_rmw_same_agent_gfx12(),
                RmwBoundary::RadiowaveVmem => match cache {
                    MutableReadCache::VmemOnly => {
                        commands.dependency_rmw_hip_llvm_vmem_gfx12();
                    }
                    MutableReadCache::ScalarOrUnknown => {
                        commands.dependency_rmw_same_agent_gfx12();
                    }
                },
            },
        }
    }

    fn create_ib(&self, commands: &RdnaPm4Commands, profiled: bool) -> Result<SingleQueuePm4Ib> {
        match (self.pm4_family, commands, profiled) {
            (Pm4Family::Gfx10, RdnaPm4Commands::Legacy(commands), false) => Ok(
                SingleQueuePm4Ib::create_gfx10(&self.device, &self.pool, commands)?,
            ),
            (Pm4Family::Gfx10, RdnaPm4Commands::Legacy(commands), true) => Ok(
                SingleQueuePm4Ib::create_profiled_gfx10(&self.device, &self.pool, commands)?,
            ),
            (Pm4Family::Gfx11, RdnaPm4Commands::Legacy(commands), false) => Ok(
                SingleQueuePm4Ib::create_gfx11(&self.device, &self.pool, commands)?,
            ),
            (Pm4Family::Gfx11, RdnaPm4Commands::Legacy(commands), true) => Ok(
                SingleQueuePm4Ib::create_profiled_gfx11(&self.device, &self.pool, commands)?,
            ),
            (Pm4Family::Gfx12, RdnaPm4Commands::Gfx12(commands), false) => Ok(
                SingleQueuePm4Ib::create(&self.device, &self.pool, commands)?,
            ),
            (Pm4Family::Gfx12, RdnaPm4Commands::Gfx12(commands), true) => Ok(
                SingleQueuePm4Ib::create_profiled(&self.device, &self.pool, commands)?,
            ),
            _ => anyhow::bail!("PM4 command family does not match selected device"),
        }
    }

    fn create_profiled_ib(&self, commands: &[RdnaPm4Commands]) -> Result<ProfiledPm4Replay> {
        anyhow::ensure!(
            !commands.is_empty(),
            "profiled PM4 replay requires one lane"
        );
        if commands.len() == 1 {
            return Ok(ProfiledPm4Replay::Single(
                self.create_ib(&commands[0], true)?,
            ));
        }
        match self.pm4_family {
            Pm4Family::Gfx10 | Pm4Family::Gfx11 => {
                let encoded = commands
                    .iter()
                    .map(|commands| match commands {
                        RdnaPm4Commands::Legacy(commands) => Ok(commands.clone()),
                        RdnaPm4Commands::Gfx12(_) => {
                            anyhow::bail!("PM4 command family does not match selected device")
                        }
                    })
                    .collect::<Result<Vec<_>>>()?;
                let ib = match self.pm4_family {
                    Pm4Family::Gfx10 => {
                        MultiQueuePm4Ib::create_profiled_gfx10(&self.device, &self.pool, &encoded)?
                    }
                    Pm4Family::Gfx11 => {
                        MultiQueuePm4Ib::create_profiled_gfx11(&self.device, &self.pool, &encoded)?
                    }
                    Pm4Family::Gfx12 => unreachable!(),
                };
                Ok(ProfiledPm4Replay::Multi(ib))
            }
            Pm4Family::Gfx12 => {
                let encoded = commands
                    .iter()
                    .map(|commands| match commands {
                        RdnaPm4Commands::Gfx12(commands) => Ok(commands.clone()),
                        RdnaPm4Commands::Legacy(_) => {
                            anyhow::bail!("PM4 command family does not match selected device")
                        }
                    })
                    .collect::<Result<Vec<_>>>()?;
                Ok(ProfiledPm4Replay::Multi(MultiQueuePm4Ib::create_profiled(
                    &self.device,
                    &self.pool,
                    &encoded,
                )?))
            }
        }
    }

    fn mutable_read_cache(
        &self,
        kernel: &str,
        wave_size: u32,
        scheduler_profile: SchedulerProfile,
    ) -> MutableReadCache {
        let Some(profile) = self
            .profiles
            .iter()
            .find(|profile| profile.scheduler_profile == scheduler_profile)
        else {
            return MutableReadCache::ScalarOrUnknown;
        };
        let inspection = match wave_size {
            32 => &profile.inspection_wave32,
            64 => &profile.inspection_wave64,
            _ => return MutableReadCache::ScalarOrUnknown,
        };
        inspection
            .kernel(kernel)
            .map_or(MutableReadCache::ScalarOrUnknown, |report| {
                report.mutable_read_cache
            })
    }
}

fn active_queue_count(mode: TimingMode, requested: usize, iterations: usize) -> usize {
    if mode == TimingMode::IndependentThroughput {
        requested.min(iterations.max(1))
    } else {
        1
    }
}

fn lane_operations(
    iterations: usize,
    lane_count: usize,
    lane: usize,
) -> impl Iterator<Item = usize> {
    (lane..iterations).step_by(lane_count)
}

fn inspection_from_manifest(
    encoded: &str,
    code_object: &[u8],
    expected_wavefront: Wavefront,
    expected_scheduler_profile: SchedulerProfile,
) -> Result<CodeObjectInspection> {
    let certified = CodeObjectCertification::from_json(code_object, encoded)
        .context("certify embedded Radiowave code object")?;
    let manifest = certified.manifest();
    anyhow::ensure!(
        manifest.wavefront == expected_wavefront,
        "Radiowave manifest wavefront mismatch: expected {:?}, found {:?}",
        expected_wavefront,
        manifest.wavefront
    );
    anyhow::ensure!(
        manifest.scheduler_profile == expected_scheduler_profile,
        "Radiowave manifest scheduler-profile mismatch: expected {:?}, found {:?}",
        expected_scheduler_profile,
        manifest.scheduler_profile
    );
    Ok(certified.inspection().clone())
}

fn fill_kernarg(
    karg: &mut KernargBuffer,
    buffers: &crate::hip_backend::Buffers,
    spec: &RowSpec,
    offset: u32,
    grid_groups: u32,
    second: bool,
) -> Result<()> {
    let bytes = karg.as_mut_bytes();
    anyhow::ensure!(
        bytes.len() >= 40,
        "kernel kernarg segment is only {} bytes",
        bytes.len()
    );
    bytes.fill(0);
    let a = buffers.a.as_ptr() as usize as u64;
    let b = buffers.b.as_ptr() as usize as u64;
    let out = buffers.out.as_ptr() as usize as u64;
    bytes[0..8].copy_from_slice(&a.to_ne_bytes());
    bytes[8..16].copy_from_slice(&b.to_ne_bytes());
    bytes[16..24].copy_from_slice(&out.to_ne_bytes());
    bytes[24..28].copy_from_slice(&spec.n0.to_ne_bytes());
    bytes[28..32].copy_from_slice(&spec.stage_n1(second).to_ne_bytes());
    bytes[32..36].copy_from_slice(&offset.to_ne_bytes());
    bytes[36..40].copy_from_slice(&spec.stage_aux(second).to_ne_bytes());

    // hipcc's AMDGPU hidden-argument block begins at the next 8-byte boundary.
    // Direct PM4 bypasses the HIP runtime that normally fills these fields.
    // The kernels avoid blockDim/blockIdx and consume hardware IDs directly,
    // but populating the canonical shape keeps the code object ABI honest.
    const HIDDEN: usize = 40;
    if bytes.len() >= HIDDEN + 66 {
        bytes[HIDDEN..HIDDEN + 4].copy_from_slice(&grid_groups.to_ne_bytes());
        bytes[HIDDEN + 4..HIDDEN + 8].copy_from_slice(&1u32.to_ne_bytes());
        bytes[HIDDEN + 8..HIDDEN + 12].copy_from_slice(&1u32.to_ne_bytes());
        bytes[HIDDEN + 12..HIDDEN + 14]
            .copy_from_slice(&(spec.stage_block(second) as u16).to_ne_bytes());
        bytes[HIDDEN + 14..HIDDEN + 16].copy_from_slice(&1u16.to_ne_bytes());
        bytes[HIDDEN + 16..HIDDEN + 18].copy_from_slice(&1u16.to_ne_bytes());
        bytes[HIDDEN + 64..HIDDEN + 66].copy_from_slice(&1u16.to_ne_bytes());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn independent_operations_are_striped_exactly_once() {
        for lane_count in [1, 2, 4] {
            let lanes = (0..lane_count)
                .map(|lane| lane_operations(17, lane_count, lane).collect::<Vec<_>>())
                .collect::<Vec<_>>();
            let mut flattened = lanes.into_iter().flatten().collect::<Vec<_>>();
            flattened.sort_unstable();
            assert_eq!(flattened, (0..17).collect::<Vec<_>>());
        }
    }

    #[test]
    fn only_independent_mode_activates_multiple_queues() {
        assert_eq!(
            active_queue_count(TimingMode::IndependentThroughput, 4, 100),
            4
        );
        assert_eq!(
            active_queue_count(TimingMode::IndependentThroughput, 4, 2),
            2
        );
        assert_eq!(active_queue_count(TimingMode::SerialLatency, 4, 100), 1);
        assert_eq!(
            active_queue_count(TimingMode::SingleKernelAggressive, 4, 1),
            1
        );
    }

    #[test]
    fn embedded_manifest_certifies_all_explicit_vmem_consumers() {
        for scheduler_profile in SchedulerProfile::ALL {
            for wavefront in [Wavefront::Wave32, Wavefront::Wave64] {
                let embedded = embedded_code_object(scheduler_profile, wavefront);
                let inspection = inspection_from_manifest(
                    embedded.manifest,
                    embedded.code,
                    wavefront,
                    scheduler_profile,
                )
                .unwrap();
                assert!(inspection
                    .kernel("memory_gather")
                    .unwrap()
                    .certifies_vmem_only_rmw());
                assert!(inspection
                    .kernel("dispatch_tiny")
                    .unwrap()
                    .certifies_vmem_only_rmw());
                assert!(inspection
                    .kernel("dense_q8")
                    .unwrap()
                    .certifies_vmem_only_rmw());
            }
        }
    }

    #[test]
    fn embedded_manifest_identity_check_fails_closed() {
        let embedded = embedded_code_object(SchedulerProfile::Default, Wavefront::Wave32);
        let mut wrong_code = embedded.code.to_vec();
        wrong_code[0] ^= 1;
        assert!(inspection_from_manifest(
            embedded.manifest,
            &wrong_code,
            Wavefront::Wave32,
            SchedulerProfile::Default,
        )
        .is_err());
    }
}
