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
    create_queue_set, load_symbols, Executable, Gfx10Pm4CommandBuffer, Gfx12DispatchMode,
    Gfx12KernelImage, Gfx12Pm4CommandBuffer, GpuDevice, GpuMultiQueueTiming, GpuSelector,
    KernargBuffer, KernargPool, Kernel, LaunchGeometry, MultiQueuePm4Ib, QueuePolicy, Runtime,
    SingleQueuePm4Ib,
};
use redline_dispatch::partition::{self, CuPartition, PartitionPolicy};
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

/// Effective per-lane CU mask after partitioned queue creation.
#[derive(Clone, Debug)]
pub struct EffectiveLaneCuMask {
    pub lane: u32,
    /// Effective affinity from [`redline_rocr::AqlQueue::cu_mask`] (one bool
    /// per device CU). That API returns the post-set cache after a successful
    /// `create_with_cu_mask`, not the raw partition config and not the stale
    /// ROCr get-mask all-CU vector. `None` only when probe create/read failed.
    pub cu_mask: Option<Vec<bool>>,
    pub enabled_cu_count: Option<u32>,
    pub cu_mask_was_reduced: Option<bool>,
    /// Populated only when `cu_mask` is `None`.
    pub reason: Option<String>,
}

pub struct RedlineBackend {
    _runtime: Runtime,
    device: GpuDevice,
    profiles: Vec<ProfileExecutables>,
    pool: KernargPool,
    pm4_family: Pm4Family,
    queue_policy: QueuePolicy,
    independent_queue_count: usize,
    partition_policy: PartitionPolicy,
    /// Validated CU slices from `partition_policy` (empty when policy is None).
    applied_partitions: Vec<CuPartition>,
    /// Effective CU masks from a post-create `AqlQueue::cu_mask` probe.
    effective_partition_masks: Vec<EffectiveLaneCuMask>,
    device_cu_count: u32,
    pub name: String,
    pub pci: String,
    pub rmw_boundary: RmwBoundary,
    dispatch_mode: Gfx12DispatchMode,
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
        mode: Gfx12DispatchMode,
    ) -> Result<()> {
        match self {
            Self::Legacy(commands) => {
                let _ = mode;
                commands.dispatch(kernel, geometry, 0, kernarg_address)?;
            }
            Self::Gfx12(commands) => {
                // Mirror Gfx12Pm4CommandBuffer::dispatch, then select initiator/dims.
                const ENABLE_SGPR_KERNARG_SEGMENT_PTR: u16 = 1 << 3;
                let image = Gfx12KernelImage::from_hsa(kernel)?;
                let pm4 = kernel
                    .pm4_metadata()
                    .context("missing kernel PM4 descriptor for Gfx12 dispatch")?;
                let needs_kernarg =
                    pm4.kernel_code_properties & ENABLE_SGPR_KERNARG_SEGMENT_PTR != 0;
                if needs_kernarg && kernarg_address.is_null() {
                    anyhow::bail!("null kernarg address for Gfx12 dispatch");
                }
                let mut user_sgprs = [0_u32; 2];
                let user_sgprs = if needs_kernarg {
                    let address = kernarg_address as usize as u64;
                    user_sgprs[0] = address as u32;
                    user_sgprs[1] = (address >> 32) as u32;
                    &user_sgprs[..]
                } else {
                    &[]
                };
                commands.dispatch_image_with_mode(
                    &image,
                    geometry,
                    0,
                    user_sgprs,
                    mode,
                )?;
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
    pub fn new(
        rmw_boundary: RmwBoundary,
        queue_policy: QueuePolicy,
        partition_policy: PartitionPolicy,
        dispatch_mode: Gfx12DispatchMode,
    ) -> Result<Self> {
        let runtime = Runtime::initialize(load_symbols()?).context("initialize public ROCr")?;
        let device = runtime.select_gpu(GpuSelector::Ordinal(0))?;
        let name = device.name().to_owned();
        let independent_queue_count = queue_policy.resolve(&name, usize::MAX);
        let pm4_family = Pm4Family::from_device(&name)?;
        let pci = device.pci_bus_id().to_string();

        // Fail closed on topology mismatch before any row runs. Serial /
        // single-lane IBs intentionally stay full-device; that is observable
        // via partition_applied and the notice below, not a silent downgrade
        // of an invalid multi-lane policy.
        let (applied_partitions, device_cu_count, effective_partition_masks) =
            if !matches!(partition_policy, PartitionPolicy::None) {
                let device_cu_count = device.compute_unit_count().with_context(|| {
                    format!("query compute unit count for partition policy on {name}")
                })?;
                let partitions =
                    partition::validate(&partition_policy, device_cu_count).with_context(|| {
                        format!(
                            "invalid partition policy configuration for device {name} ({device_cu_count} CUs)"
                        )
                    })?;
                if independent_queue_count <= 1 {
                    anyhow::bail!(
                        "partition policy configuration error: policy requires multi-queue lanes but queue policy resolved to {independent_queue_count} independent queue(s) on {name}"
                    );
                }
                if partitions.len() != independent_queue_count {
                    anyhow::bail!(
                        "partition policy configuration error: policy produces {} CU slices but independent queue count is {independent_queue_count} on {name}",
                        partitions.len()
                    );
                }
                eprintln!(
                    "note: --partition-policy applies only to multi-lane independent retained-PM4 IBs; serial and single-lane rows intentionally stay full-device (see per-row partition_applied)"
                );
                let effective =
                    Self::probe_effective_partition_masks(&device, &partitions, &partition_policy);
                (partitions, device_cu_count, effective)
            } else {
                let device_cu_count = device.compute_unit_count().unwrap_or(0);
                (Vec::new(), device_cu_count, Vec::new())
            };

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
            partition_policy,
            applied_partitions,
            effective_partition_masks,
            device_cu_count,
            name,
            pci,
            rmw_boundary,
            dispatch_mode,
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
                lane_commands.dispatch(&kernel, geometry, karg_address, self.dispatch_mode)?;

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
                    lane_commands.dispatch(&kernel, geometry, karg_address, self.dispatch_mode)?;
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

    pub fn dispatch_mode(&self) -> Gfx12DispatchMode {
        self.dispatch_mode
    }

    pub fn independent_queue_count(&self) -> usize {
        self.independent_queue_count
    }

    /// True when this row's retained multi-lane IB will install CU masks from
    /// the configured partition policy. Serial and single-lane rows stay
    /// full-device by design.
    pub fn partition_applied_for(&self, mode: TimingMode, iterations: usize) -> bool {
        let lane_count = self.queue_count_for(mode, iterations);
        self.partition_for_lanes(lane_count).is_some()
    }

    /// Validated CU slices from the configured partition policy (empty when
    /// policy is [`PartitionPolicy::None`]). Requested topology only — see
    /// [`Self::effective_partition_masks`] for HSA-reported affinity.
    pub fn applied_partitions(&self) -> &[CuPartition] {
        &self.applied_partitions
    }

    /// Effective per-lane CU masks after partitioned queue creation.
    ///
    /// Populated at backend init via a throwaway `create_queue_set` +
    /// `AqlQueue::cu_mask` / `cu_mask_was_reduced` probe (post-set effective
    /// affinity, not the partition config copy). Failed lanes carry
    /// `cu_mask: None` and a reason.
    pub fn effective_partition_masks(&self) -> &[EffectiveLaneCuMask] {
        &self.effective_partition_masks
    }

    /// Device CU count used when validating/applying the partition policy.
    pub fn device_cu_count(&self) -> u32 {
        self.device_cu_count
    }

    /// Create partitioned queues, read each lane's effective CU mask, then drop.
    fn probe_effective_partition_masks(
        device: &GpuDevice,
        partitions: &[CuPartition],
        policy: &PartitionPolicy,
    ) -> Vec<EffectiveLaneCuMask> {
        let queue_size = *device.queue_size_range().start();
        let queue_count = partitions.len();
        let queues = match create_queue_set(device, queue_count, queue_size, Some(policy)) {
            Ok(queues) => queues,
            Err(error) => {
                return partitions
                    .iter()
                    .map(|part| EffectiveLaneCuMask {
                        lane: part.index,
                        cu_mask: None,
                        enabled_cu_count: None,
                        cu_mask_was_reduced: None,
                        reason: Some(format!("partitioned queue create failed: {error}")),
                    })
                    .collect();
            }
        };
        (0..queue_count)
            .map(|lane| {
                let part_index = partitions
                    .get(lane)
                    .map(|part| part.index)
                    .unwrap_or(lane as u32);
                let reduced = queues.cu_mask_was_reduced(lane);
                match queues.cu_mask(lane, device) {
                    Some(Ok(mask)) => {
                        let enabled_cu_count = mask.iter().filter(|&&bit| bit).count() as u32;
                        EffectiveLaneCuMask {
                            lane: part_index,
                            cu_mask: Some(mask),
                            enabled_cu_count: Some(enabled_cu_count),
                            cu_mask_was_reduced: reduced,
                            reason: None,
                        }
                    }
                    Some(Err(error)) => EffectiveLaneCuMask {
                        lane: part_index,
                        cu_mask: None,
                        enabled_cu_count: None,
                        cu_mask_was_reduced: reduced,
                        reason: Some(format!("cu_mask read-back failed: {error}")),
                    },
                    None => EffectiveLaneCuMask {
                        lane: part_index,
                        cu_mask: None,
                        enabled_cu_count: None,
                        cu_mask_was_reduced: reduced,
                        reason: Some(format!("queue lane {lane} missing after create")),
                    },
                }
            })
            .collect()
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

    /// CU partition applies only when the retained IB owns multiple queues and
    /// the policy carves the same number of slices. Serial / ownership IBs stay
    /// on the full device so single-lane paths do not silently bind one half.
    fn partition_for_lanes(&self, lane_count: usize) -> Option<&PartitionPolicy> {
        if lane_count <= 1 || matches!(self.partition_policy, PartitionPolicy::None) {
            None
        } else {
            Some(&self.partition_policy)
        }
    }

    fn create_ib(&self, commands: &RdnaPm4Commands, profiled: bool) -> Result<SingleQueuePm4Ib> {
        // Ownership and serial IBs are always one queue on the full device.
        let partition = None;
        match (self.pm4_family, commands, profiled) {
            (Pm4Family::Gfx10, RdnaPm4Commands::Legacy(commands), false) => Ok(
                SingleQueuePm4Ib::create_gfx10_with_partition(
                    &self.device,
                    &self.pool,
                    commands,
                    partition,
                )?,
            ),
            (Pm4Family::Gfx10, RdnaPm4Commands::Legacy(commands), true) => Ok(
                SingleQueuePm4Ib::create_profiled_gfx10_with_partition(
                    &self.device,
                    &self.pool,
                    commands,
                    partition,
                )?,
            ),
            (Pm4Family::Gfx11, RdnaPm4Commands::Legacy(commands), false) => Ok(
                SingleQueuePm4Ib::create_gfx11_with_partition(
                    &self.device,
                    &self.pool,
                    commands,
                    partition,
                )?,
            ),
            (Pm4Family::Gfx11, RdnaPm4Commands::Legacy(commands), true) => Ok(
                SingleQueuePm4Ib::create_profiled_gfx11_with_partition(
                    &self.device,
                    &self.pool,
                    commands,
                    partition,
                )?,
            ),
            (Pm4Family::Gfx12, RdnaPm4Commands::Gfx12(commands), false) => Ok(
                SingleQueuePm4Ib::create_with_partition(
                    &self.device,
                    &self.pool,
                    commands,
                    partition,
                )?,
            ),
            (Pm4Family::Gfx12, RdnaPm4Commands::Gfx12(commands), true) => Ok(
                SingleQueuePm4Ib::create_profiled_with_partition(
                    &self.device,
                    &self.pool,
                    commands,
                    partition,
                )?,
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
        let partition = self.partition_for_lanes(commands.len());
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
                    Pm4Family::Gfx10 => MultiQueuePm4Ib::create_profiled_gfx10_with_partition(
                        &self.device,
                        &self.pool,
                        &encoded,
                        partition,
                    )?,
                    Pm4Family::Gfx11 => MultiQueuePm4Ib::create_profiled_gfx11_with_partition(
                        &self.device,
                        &self.pool,
                        &encoded,
                        partition,
                    )?,
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
                Ok(ProfiledPm4Replay::Multi(
                    MultiQueuePm4Ib::create_profiled_with_partition(
                        &self.device,
                        &self.pool,
                        &encoded,
                        partition,
                    )?,
                ))
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
    fn embedded_manifest_drives_fail_closed_cache_boundaries() {
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
                assert_eq!(
                    inspection
                        .kernel("memory_gather")
                        .unwrap()
                        .mutable_read_cache,
                    MutableReadCache::ScalarOrUnknown
                );
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
