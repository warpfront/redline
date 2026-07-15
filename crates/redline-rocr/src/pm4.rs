// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! GFX12 compute command construction for AMD's vendor-specific AQL PM4-IB packet.
//!
//! This is deliberately narrower than a general PM4 library. It lowers a
//! loader-resolved, zero-scratch HSA kernel into the register writes and
//! `DISPATCH_DIRECT` packet used by ROCr's own command builder. Unsupported
//! implicit-SGPR contracts fail closed instead of guessing queue internals.

use std::collections::BTreeMap;
use std::ffi::c_void;
use std::fmt;

use crate::{Kernel, LaunchGeometry};

const PACKET3_SET_SH_REG: u32 = 0x76;
const PACKET3_DISPATCH_DIRECT: u32 = 0x15;
const PACKET3_COPY_DATA: u32 = 0x40;
const PACKET3_RELEASE_MEM: u32 = 0x49;
const PACKET3_EVENT_WRITE: u32 = 0x46;
const PACKET3_ACQUIRE_MEM: u32 = 0x58;

// GFX12 SET_SH_REG offsets. The gfx12 register headers number COMPUTE
// registers from regCOMPUTE_DISPATCH_INITIATOR=0x1ba0; SET_SH_REG retains the
// architectural 0x200 COMPUTE window used by ROCr's PM4 builders.
const COMPUTE_NUM_THREAD_X: u32 = 0x207;
const COMPUTE_PGM_LO: u32 = 0x20c;
const COMPUTE_PGM_RSRC1: u32 = 0x212;
const COMPUTE_RESOURCE_LIMITS: u32 = 0x215;
const COMPUTE_TMPRING_SIZE: u32 = 0x216;
const COMPUTE_PGM_RSRC3_GFX12: u32 = 0x223;
const COMPUTE_STATIC_THREAD_MGMT_SE0: u32 = 0x230;
const COMPUTE_USER_DATA_0: u32 = 0x240;

const LDS_SIZE_MASK: u32 = 0x00ff_8000;
const LDS_SIZE_SHIFT: u32 = 15;
const GFX12_LDS_GRANULE: u32 = 512;

const ENABLE_SGPR_KERNARG_SEGMENT_PTR: u16 = 1 << 3;
const ENABLE_WAVEFRONT_SIZE32: u16 = 1 << 10;
const SUPPORTED_KERNEL_PROPERTIES: u16 = ENABLE_SGPR_KERNARG_SEGMENT_PTR | ENABLE_WAVEFRONT_SIZE32;

const GCR_GLK_INV: u32 = 1 << 7;
const GCR_GLV_INV: u32 = 1 << 8;
const GCR_GL1_INV: u32 = 1 << 9;
const GCR_GL2_INV: u32 = 1 << 14;
const GCR_GL2_WB: u32 = 1 << 15;
const GCR_SEQ_FORWARD: u32 = 1 << 16;

/// Cache actions to pair with a compute-idle wait between dependent gfx12
/// dispatches in one retained PM4 stream.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Gfx12RmwAcquirePolicy {
    /// Existing Redline/Hipfire policy: invalidate scalar/vector read caches
    /// and force forward cache sequencing.
    CurrentSequential,
    /// Conservative global-cache experiment. This is heavier than Mesa RADV
    /// 25.2.x's coherent-buffer compute barrier because it also writes back and
    /// invalidates L2.
    RadvGlobal,
    /// Same-agent candidate: retain scalar, vector, and merged L1
    /// invalidation, but omit RADV's global L2 action and forward sequencing.
    SameAgentParallelL1,
    /// Earlier scalar/vector-L0 experiment. It omits the merged L1 action and
    /// is retained for measurement compatibility, not as the generic default.
    SameAgentParallelL0,
    /// Radiowave-certified HIP/LLVM consumer: mutable resources are read only
    /// through VMEM, so invalidate vector L0 and merged L1 while retaining the
    /// unrelated scalar cache and coherent L2/MALL.
    HipLlvmVmemL1,
}

/// Backend-neutral description of one GFX12 compute program.
///
/// Public-HSA kernels are adapted with [`Self::from_hsa`], but the PM4 encoder
/// itself does not require HSA's kernarg ABI. A caller may supply an ACO,
/// assembly, or other compiler image together with that image's user-SGPR
/// payload.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Gfx12KernelImage {
    pub code_entry: u64,
    pub compute_pgm_rsrc1: u32,
    pub compute_pgm_rsrc2: u32,
    pub compute_pgm_rsrc3: u32,
    pub group_segment_size: u32,
    pub private_segment_size: u32,
    pub dynamic_callstack: bool,
    pub wave32: bool,
}

/// Encoding of the dimensions and initiator for `DISPATCH_DIRECT`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Gfx12DispatchMode {
    /// ROCr/HSA convention: total work-items with `USE_THREAD_DIMENSIONS`.
    Workitems,
    /// RADV convention for aligned `vkCmdDispatch`: work-group counts with
    /// out-of-order wave launch enabled.
    RadvWorkgroups,
}

impl Gfx12KernelImage {
    /// Preserve the loader-reported program resources while shedding the HSA
    /// object type at the PM4 encoding boundary.
    pub fn from_hsa(kernel: &Kernel) -> Result<Self, Pm4BuildError> {
        let loader = kernel.metadata();
        let pm4 = kernel
            .pm4_metadata()
            .ok_or(Pm4BuildError::MissingKernelDescriptor)?;
        let unsupported = pm4.kernel_code_properties & !SUPPORTED_KERNEL_PROPERTIES;
        if unsupported != 0 {
            return Err(Pm4BuildError::UnsupportedKernelProperties(unsupported));
        }
        Ok(Self {
            code_entry: pm4.code_entry,
            compute_pgm_rsrc1: pm4.compute_pgm_rsrc1,
            compute_pgm_rsrc2: pm4.compute_pgm_rsrc2,
            compute_pgm_rsrc3: pm4.compute_pgm_rsrc3,
            group_segment_size: loader.group_segment_size,
            private_segment_size: loader.private_segment_size,
            dynamic_callstack: loader.dynamic_callstack,
            wave32: pm4.kernel_code_properties & ENABLE_WAVEFRONT_SIZE32 != 0,
        })
    }
}

impl Gfx12RmwAcquirePolicy {
    const fn gcr_cntl(self) -> u32 {
        match self {
            Self::CurrentSequential => GCR_GLK_INV | GCR_GLV_INV | GCR_SEQ_FORWARD,
            Self::RadvGlobal => GCR_GLK_INV | GCR_GLV_INV | GCR_GL1_INV | GCR_GL2_INV | GCR_GL2_WB,
            Self::SameAgentParallelL1 => GCR_GLK_INV | GCR_GLV_INV | GCR_GL1_INV,
            Self::SameAgentParallelL0 => GCR_GLK_INV | GCR_GLV_INV,
            Self::HipLlvmVmemL1 => GCR_GLV_INV | GCR_GL1_INV,
        }
    }
}

/// Retained GFX12 PM4 command words suitable for one PM4 indirect buffer.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Gfx12Pm4CommandBuffer {
    dwords: Vec<u32>,
    register_state: Option<BTreeMap<u32, u32>>,
    cache_dynamic_registers: bool,
}

impl Gfx12Pm4CommandBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Construct a command buffer which omits writes to SH registers whose
    /// values are already live earlier in this same retained indirect buffer.
    /// The first write to every register is always emitted.
    pub fn new_stateful() -> Self {
        Self {
            dwords: Vec::new(),
            register_state: Some(BTreeMap::new()),
            cache_dynamic_registers: true,
        }
    }

    /// Retain only queue-global invariant register values. Program, resource,
    /// workgroup, user-data, and dispatch state are still written exactly as
    /// in the legacy encoder.
    pub fn new_static_stateful() -> Self {
        Self {
            dwords: Vec::new(),
            register_state: Some(BTreeMap::new()),
            cache_dynamic_registers: false,
        }
    }

    /// Invalidate the agent caches at the HIP/HSA-to-PM4 ownership boundary.
    /// Encoding matches ROCr's gfx10+ `AcquireMemTemplate`, which remains the
    /// command shape used on gfx12.
    pub fn acquire_system(&mut self) {
        self.dwords.extend_from_slice(&[
            packet3(PACKET3_ACQUIRE_MEM, 7, false),
            0,
            u32::MAX,
            0xff,
            0,
            0,
            4,
            (1 << 16)
                | (1 << 15)
                | (1 << 14)
                | (1 << 9)
                | (1 << 8)
                | (1 << 7)
                | (1 << 6)
                | (1 << 5)
                | (1 << 4)
                | 1,
        ]);
    }

    /// GFX12 ownership-boundary acquire derived from the gfx12 GCR fields.
    /// This preserves system-scope L2 writeback/invalidate plus instruction,
    /// scalar, and vector cache visibility without carrying removed gfx11
    /// GL1/metadata bits into the merged RDNA4 hierarchy.
    pub fn acquire_system_gfx12(&mut self) {
        self.emit_acquire_gcr(0x1c1d1);
    }

    /// Return a copy bracketed by GPU-clock writes. The end timestamp follows
    /// a compute-idle event, matching a bottom-of-pipe timestamp query; the
    /// start uses the same top-of-pipe `COPY_DATA` form as RADV.
    pub fn with_gpu_timestamps(&self, start_address: u64, end_address: u64) -> Self {
        let mut timed = Self::new();
        timed.copy_gpu_timestamp(start_address);
        timed.dwords.extend_from_slice(&self.dwords);
        timed.release_gpu_timestamp(end_address);
        timed
    }

    fn copy_gpu_timestamp(&mut self, address: u64) {
        // COPY_DATA: 64-bit timestamp source -> memory, with write confirm.
        // This is RADV's TOP_OF_PIPE timestamp encoding on gfx12.
        const COPY_DATA_TIMESTAMP_TO_MEMORY_64: u32 = 9 | (5 << 8) | (1 << 16) | (1 << 20);
        self.dwords.extend_from_slice(&[
            packet3(PACKET3_COPY_DATA, 5, false),
            COPY_DATA_TIMESTAMP_TO_MEMORY_64,
            0,
            0,
            address as u32,
            (address >> 32) as u32,
        ]);
    }

    fn release_gpu_timestamp(&mut self, address: u64) {
        // RADV's gfx9+ bottom-of-pipe timestamp: RELEASE_MEM waits for all
        // earlier compute work and writes the 64-bit GPU clock to memory.
        const BOTTOM_OF_PIPE_TS_EVENT: u32 = 40 | (5 << 8);
        const TIMESTAMP_AFTER_WRITE_CONFIRM: u32 = (3 << 24) | (3 << 29);
        self.dwords.extend_from_slice(&[
            packet3(PACKET3_RELEASE_MEM, 7, false),
            BOTTOM_OF_PIPE_TS_EVENT,
            TIMESTAMP_AFTER_WRITE_CONFIRM,
            address as u32,
            (address >> 32) as u32,
            0,
            0,
            0,
        ]);
    }

    /// Same-agent inter-node acquire for one retained gfx12 tape. Kernel code
    /// is immutable and L2/MALL remains coherent, so only scalar/vector read
    /// caches plus forward sequencing are invalidated.
    pub fn acquire_inter_node_gfx12(&mut self) {
        self.acquire_rmw_gfx12(Gfx12RmwAcquirePolicy::CurrentSequential);
    }

    /// Generic same-agent GFX12 shader-write to shader-read/write boundary.
    ///
    /// `CS_PARTIAL_FLUSH` prevents a later dispatch from overlapping its
    /// producer. The acquire then invalidates scalar, vector, and merged L1
    /// shader read caches while retaining coherent L2/MALL contents. This is
    /// the fail-closed path for consumers whose scalar-memory behavior is not
    /// certified.
    ///
    /// This boundary is valid only when producer and consumer execute on the
    /// same gfx12 agent and the resource remains a coherent shader buffer.
    /// Host/device ownership changes or non-coherent resources require the
    /// broader system/global acquire path.
    pub fn dependency_rmw_same_agent_gfx12(&mut self) {
        self.wait_compute_idle();
        self.acquire_rmw_gfx12(Gfx12RmwAcquirePolicy::SameAgentParallelL1);
    }

    /// Same-agent boundary for a HIP/LLVM consumer certified to read mutable
    /// resources through VMEM only.
    ///
    /// The completion edge remains identical to the generic boundary. Only
    /// the unrelated scalar-cache invalidation is omitted; vector L0 and the
    /// merged L1 are still invalidated. Unknown or scalar-reading consumers
    /// must use [`Self::dependency_rmw_same_agent_gfx12`].
    pub fn dependency_rmw_hip_llvm_vmem_gfx12(&mut self) {
        self.wait_compute_idle();
        self.acquire_rmw_gfx12(Gfx12RmwAcquirePolicy::HipLlvmVmemL1);
    }

    /// Emit the cache half of a gfx12 read-modify-write dependency boundary.
    /// Callers must emit `wait_compute_idle` first when the later dispatch
    /// reads or overwrites memory written by the earlier dispatch.
    pub fn acquire_rmw_gfx12(&mut self, policy: Gfx12RmwAcquirePolicy) {
        match policy {
            Gfx12RmwAcquirePolicy::CurrentSequential => {
                self.emit_acquire_gcr(policy.gcr_cntl());
            }
            _ => self.emit_acquire_gcr_radv(policy.gcr_cntl()),
        }
    }

    fn emit_acquire_gcr(&mut self, gcr_cntl: u32) {
        self.dwords.extend_from_slice(&[
            packet3(PACKET3_ACQUIRE_MEM, 7, false),
            0,
            u32::MAX,
            0xff,
            0,
            0,
            4,
            gcr_cntl,
        ]);
    }

    fn emit_acquire_gcr_radv(&mut self, gcr_cntl: u32) {
        self.dwords.extend_from_slice(&[
            packet3(PACKET3_ACQUIRE_MEM, 7, false),
            0,
            u32::MAX,
            0x00ff_ffff,
            0,
            0,
            10,
            gcr_cntl,
        ]);
    }

    /// Append one zero-scratch wave32 or wave64 dispatch using the exact loaded
    /// code entry and descriptor resources reported by the HSA loader.
    pub fn dispatch(
        &mut self,
        kernel: &Kernel,
        geometry: LaunchGeometry,
        dynamic_group_bytes: u32,
        kernarg_address: *mut c_void,
    ) -> Result<(), Pm4BuildError> {
        let image = Gfx12KernelImage::from_hsa(kernel)?;
        let pm4 = kernel
            .pm4_metadata()
            .ok_or(Pm4BuildError::MissingKernelDescriptor)?;
        let needs_kernarg = pm4.kernel_code_properties & ENABLE_SGPR_KERNARG_SEGMENT_PTR != 0;
        if needs_kernarg && kernarg_address.is_null() {
            return Err(Pm4BuildError::NullKernarg);
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
        self.dispatch_image(&image, geometry, dynamic_group_bytes, user_sgprs)
    }

    /// Append one dispatch of an ABI-neutral kernel image.
    ///
    /// `user_sgprs` are written verbatim starting at `COMPUTE_USER_DATA_0`.
    /// HSA's kernarg pointer is just the two-dword payload supplied by
    /// [`Self::dispatch`]; Vulkan/ACO-style descriptor table pointers and push
    /// constants can use this entry point directly.
    pub fn dispatch_image(
        &mut self,
        image: &Gfx12KernelImage,
        geometry: LaunchGeometry,
        dynamic_group_bytes: u32,
        user_sgprs: &[u32],
    ) -> Result<(), Pm4BuildError> {
        self.dispatch_image_with_mode(
            image,
            geometry,
            dynamic_group_bytes,
            user_sgprs,
            Gfx12DispatchMode::Workitems,
        )
    }

    pub fn dispatch_image_with_mode(
        &mut self,
        image: &Gfx12KernelImage,
        geometry: LaunchGeometry,
        dynamic_group_bytes: u32,
        user_sgprs: &[u32],
        mode: Gfx12DispatchMode,
    ) -> Result<(), Pm4BuildError> {
        if image.private_segment_size != 0 || image.dynamic_callstack {
            return Err(Pm4BuildError::ScratchUnsupported {
                private_bytes: image.private_segment_size,
                dynamic_callstack: image.dynamic_callstack,
            });
        }
        if image.code_entry == 0 || image.code_entry & 0xff != 0 {
            return Err(Pm4BuildError::InvalidCodeEntry(image.code_entry));
        }
        if user_sgprs.len() > 16 {
            return Err(Pm4BuildError::TooManyUserSgprs(user_sgprs.len()));
        }
        let total_group_bytes = image
            .group_segment_size
            .checked_add(dynamic_group_bytes)
            .ok_or(Pm4BuildError::GroupSegmentOverflow)?;
        let lds_blocks = total_group_bytes.div_ceil(GFX12_LDS_GRANULE);
        if lds_blocks > LDS_SIZE_MASK >> LDS_SIZE_SHIFT {
            return Err(Pm4BuildError::GroupSegmentTooLarge(total_group_bytes));
        }
        let rsrc2 = (image.compute_pgm_rsrc2 & !LDS_SIZE_MASK) | (lds_blocks << LDS_SIZE_SHIFT);

        self.set_sh_regs(
            COMPUTE_PGM_LO,
            &[
                (image.code_entry >> 8) as u32,
                (image.code_entry >> 40) as u32,
            ],
        );
        self.set_sh_regs(COMPUTE_PGM_RSRC1, &[image.compute_pgm_rsrc1, rsrc2]);
        self.set_sh_regs(COMPUTE_PGM_RSRC3_GFX12, &[image.compute_pgm_rsrc3]);
        self.set_sh_regs(COMPUTE_TMPRING_SIZE, &[0]);
        self.set_sh_regs(
            COMPUTE_NUM_THREAD_X,
            &[
                u32::from(geometry.workgroup[0]),
                u32::from(geometry.workgroup[1]),
                u32::from(geometry.workgroup[2]),
            ],
        );
        // Match ROCr's direct-dispatch template: all waves per SH are allowed
        // and every shader engine remains eligible.
        self.set_sh_regs(COMPUTE_RESOURCE_LIMITS, &[0x3ff]);
        self.set_sh_regs(COMPUTE_STATIC_THREAD_MGMT_SE0, &[u32::MAX; 4]);
        if !user_sgprs.is_empty() {
            self.set_sh_regs(COMPUTE_USER_DATA_0, user_sgprs);
        }

        let wave_mode = if image.wave32 { 1 << 15 } else { 0 };
        let (dimensions, initiator) = match mode {
            Gfx12DispatchMode::Workitems => (
                geometry.grid_workitems,
                (1 << 0) | (1 << 2) | (1 << 5) | wave_mode,
            ),
            Gfx12DispatchMode::RadvWorkgroups => {
                let mut workgroups = [0_u32; 3];
                for (axis, count) in workgroups.iter_mut().enumerate() {
                    let workgroup = u32::from(geometry.workgroup[axis]);
                    if !geometry.grid_workitems[axis].is_multiple_of(workgroup) {
                        return Err(Pm4BuildError::PartialWorkgroupUnsupported { axis });
                    }
                    *count = geometry.grid_workitems[axis] / workgroup;
                }
                (
                    workgroups,
                    // COMPUTE_SHADER_EN | FORCE_START_AT_000 | ORDER_MODE |
                    // TUNNEL_ENABLE, plus CS_W32_EN for wave32 images.
                    (1 << 0) | (1 << 2) | (1 << 6) | (1 << 13) | wave_mode,
                )
            }
        };
        self.dwords.push(packet3(PACKET3_DISPATCH_DIRECT, 4, true));
        self.dwords.extend_from_slice(&dimensions);
        self.dwords.push(initiator);
        Ok(())
    }

    /// Wait until all earlier compute waves have finished before the PM4 IB
    /// itself completes and its enclosing AQL packet publishes its signal.
    pub fn wait_compute_idle(&mut self) {
        self.dwords.push(packet3(PACKET3_EVENT_WRITE, 1, false));
        self.dwords.push(0x407); // CS_PARTIAL_FLUSH, event-index 4.
    }

    pub fn len_dwords(&self) -> u32 {
        self.dwords.len() as u32
    }

    pub fn is_empty(&self) -> bool {
        self.dwords.is_empty()
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        self.dwords
            .iter()
            .flat_map(|word| word.to_le_bytes())
            .collect()
    }

    pub fn dwords(&self) -> &[u32] {
        &self.dwords
    }

    fn set_sh_regs(&mut self, first: u32, values: &[u32]) {
        debug_assert!(!values.is_empty());
        let static_registers = matches!(
            first,
            COMPUTE_TMPRING_SIZE | COMPUTE_RESOURCE_LIMITS | COMPUTE_STATIC_THREAD_MGMT_SE0
        );
        if !self.cache_dynamic_registers && !static_registers {
            self.emit_set_sh_regs(first, values);
            return;
        }
        let Some(register_state) = self.register_state.as_mut() else {
            self.emit_set_sh_regs(first, values);
            return;
        };

        let mut changed_runs = Vec::<(u32, Vec<u32>)>::new();
        let mut run_first = None;
        let mut run_values = Vec::new();
        for (offset, value) in values.iter().copied().enumerate() {
            let register = first + offset as u32;
            if register_state.get(&register).copied() == Some(value) {
                if let Some(run_first) = run_first.take() {
                    changed_runs.push((run_first, std::mem::take(&mut run_values)));
                }
                continue;
            }
            register_state.insert(register, value);
            run_first.get_or_insert(register);
            run_values.push(value);
        }
        if let Some(run_first) = run_first {
            changed_runs.push((run_first, run_values));
        }

        for (run_first, run_values) in changed_runs {
            self.emit_set_sh_regs(run_first, &run_values);
        }
    }

    fn emit_set_sh_regs(&mut self, first: u32, values: &[u32]) {
        self.dwords
            .push(packet3(PACKET3_SET_SH_REG, 1 + values.len() as u32, true));
        self.dwords.push(first);
        self.dwords.extend_from_slice(values);
    }
}

fn packet3(opcode: u32, body_dwords: u32, compute: bool) -> u32 {
    debug_assert!(body_dwords > 0);
    (3 << 30) | ((body_dwords - 1) << 16) | (opcode << 8) | if compute { 1 << 1 } else { 0 }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Pm4BuildError {
    MissingKernelDescriptor,
    InvalidCodeEntry(u64),
    ScratchUnsupported {
        private_bytes: u32,
        dynamic_callstack: bool,
    },
    UnsupportedKernelProperties(u16),
    NullKernarg,
    TooManyUserSgprs(usize),
    PartialWorkgroupUnsupported {
        axis: usize,
    },
    GroupSegmentOverflow,
    GroupSegmentTooLarge(u32),
}

impl fmt::Display for Pm4BuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingKernelDescriptor => {
                write!(formatter, "kernel descriptor PM4 metadata is unavailable")
            }
            Self::InvalidCodeEntry(address) => write!(
                formatter,
                "kernel code entry {address:#x} is null or not 256-byte aligned"
            ),
            Self::ScratchUnsupported {
                private_bytes,
                dynamic_callstack,
            } => write!(
                formatter,
                "PM4 dispatch does not support scratch (private={private_bytes}, dynamic_callstack={dynamic_callstack})"
            ),
            Self::UnsupportedKernelProperties(bits) => write!(
                formatter,
                "kernel requires unsupported implicit SGPR properties 0x{bits:04x}"
            ),
            Self::NullKernarg => write!(formatter, "kernel requires a non-null kernarg pointer"),
            Self::TooManyUserSgprs(count) => write!(
                formatter,
                "kernel image requests {count} user-SGPR dwords; GFX12 exposes at most 16"
            ),
            Self::PartialWorkgroupUnsupported { axis } => write!(
                formatter,
                "RADV work-group dispatch requires an integral grid on axis {axis}"
            ),
            Self::GroupSegmentOverflow => {
                write!(formatter, "static plus dynamic group segment overflowed")
            }
            Self::GroupSegmentTooLarge(bytes) => write!(
                formatter,
                "group segment size {bytes} cannot be encoded in GFX12 COMPUTE_PGM_RSRC2"
            ),
        }
    }
}

impl std::error::Error for Pm4BuildError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packet3_count_and_shader_type_match_gfx12_headers() {
        assert_eq!(packet3(PACKET3_SET_SH_REG, 3, true), 0xc002_7602);
        assert_eq!(packet3(PACKET3_DISPATCH_DIRECT, 4, true), 0xc003_1502);
        assert_eq!(packet3(PACKET3_EVENT_WRITE, 1, false), 0xc000_4600);
        assert_eq!(packet3(PACKET3_ACQUIRE_MEM, 7, false), 0xc006_5800);
    }

    #[test]
    fn acquire_and_compute_idle_have_stable_rocr_encodings() {
        let mut commands = Gfx12Pm4CommandBuffer::new();
        commands.acquire_system();
        commands.acquire_system_gfx12();
        commands.acquire_inter_node_gfx12();
        commands.wait_compute_idle();
        assert_eq!(commands.dwords()[0], 0xc006_5800);
        assert_eq!(commands.dwords()[7], 0x1c3f1);
        assert_eq!(commands.dwords()[8], 0xc006_5800);
        assert_eq!(commands.dwords()[15], 0x1c1d1);
        assert_eq!(commands.dwords()[16], 0xc006_5800);
        assert_eq!(commands.dwords()[23], 0x10180);
        assert_eq!(&commands.dwords()[24..], &[0xc000_4600, 0x407]);
    }

    #[test]
    fn same_agent_rmw_boundary_matches_compute_flush_and_all_shader_cache_invalidate() {
        let mut commands = Gfx12Pm4CommandBuffer::new();
        commands.dependency_rmw_same_agent_gfx12();
        assert_eq!(
            commands.dwords(),
            &[
                0xc000_4600,
                0x407,
                0xc006_5800,
                0,
                u32::MAX,
                0x00ff_ffff,
                0,
                0,
                10,
                0x00380,
            ]
        );
    }

    #[test]
    fn hip_llvm_vmem_boundary_retains_scalar_and_l2_caches() {
        let mut commands = Gfx12Pm4CommandBuffer::new();
        commands.dependency_rmw_hip_llvm_vmem_gfx12();
        assert_eq!(
            commands.dwords(),
            &[
                0xc000_4600,
                0x407,
                0xc006_5800,
                0,
                u32::MAX,
                0x00ff_ffff,
                0,
                0,
                10,
                0x00300,
            ]
        );
    }

    #[test]
    fn gfx12_rmw_acquire_policies_have_expected_gcr_words() {
        let cases = [
            (Gfx12RmwAcquirePolicy::CurrentSequential, 0x10180),
            (Gfx12RmwAcquirePolicy::RadvGlobal, 0x0c380),
            (Gfx12RmwAcquirePolicy::SameAgentParallelL1, 0x00380),
            (Gfx12RmwAcquirePolicy::SameAgentParallelL0, 0x00180),
            (Gfx12RmwAcquirePolicy::HipLlvmVmemL1, 0x00300),
        ];
        for (policy, expected) in cases {
            let mut commands = Gfx12Pm4CommandBuffer::new();
            commands.acquire_rmw_gfx12(policy);
            assert_eq!(commands.dwords()[0], 0xc006_5800);
            let expected_size_hi = if policy == Gfx12RmwAcquirePolicy::CurrentSequential {
                0xff
            } else {
                0x00ff_ffff
            };
            let expected_poll = if policy == Gfx12RmwAcquirePolicy::CurrentSequential {
                4
            } else {
                10
            };
            assert_eq!(commands.dwords()[3], expected_size_hi);
            assert_eq!(commands.dwords()[6], expected_poll);
            assert_eq!(commands.dwords()[7], expected);
        }
    }

    #[test]
    fn stateful_register_writes_emit_only_changed_contiguous_runs() {
        let mut commands = Gfx12Pm4CommandBuffer::new_stateful();
        commands.set_sh_regs(0x210, &[1, 2, 3, 4]);
        let first_len = commands.len_dwords();
        commands.set_sh_regs(0x210, &[1, 2, 3, 4]);
        assert_eq!(commands.len_dwords(), first_len);

        commands.set_sh_regs(0x210, &[5, 2, 6, 4]);
        assert_eq!(
            &commands.dwords()[first_len as usize..],
            &[
                packet3(PACKET3_SET_SH_REG, 2, true),
                0x210,
                5,
                packet3(PACKET3_SET_SH_REG, 2, true),
                0x212,
                6,
            ]
        );
    }

    #[test]
    fn legacy_register_writes_remain_byte_stable() {
        let mut commands = Gfx12Pm4CommandBuffer::new();
        commands.set_sh_regs(0x210, &[1, 2]);
        let once = commands.dwords().to_vec();
        commands.set_sh_regs(0x210, &[1, 2]);
        assert_eq!(commands.dwords().len(), once.len() * 2);
        assert_eq!(&commands.dwords()[once.len()..], once);
    }

    #[test]
    fn static_stateful_caches_only_queue_global_registers() {
        let mut commands = Gfx12Pm4CommandBuffer::new_static_stateful();
        commands.set_sh_regs(COMPUTE_RESOURCE_LIMITS, &[0x3ff]);
        let static_len = commands.len_dwords();
        commands.set_sh_regs(COMPUTE_RESOURCE_LIMITS, &[0x3ff]);
        assert_eq!(commands.len_dwords(), static_len);

        commands.set_sh_regs(COMPUTE_PGM_LO, &[1, 2]);
        let dynamic_len = commands.len_dwords();
        commands.set_sh_regs(COMPUTE_PGM_LO, &[1, 2]);
        assert_eq!(
            commands.len_dwords() - dynamic_len,
            dynamic_len - static_len
        );
    }

    #[test]
    fn abi_neutral_image_writes_declared_user_sgprs() {
        let image = Gfx12KernelImage {
            code_entry: 0x1_0000,
            compute_pgm_rsrc1: 0x11,
            compute_pgm_rsrc2: 0x22,
            compute_pgm_rsrc3: 0x33,
            group_segment_size: 0,
            private_segment_size: 0,
            dynamic_callstack: false,
            wave32: true,
        };
        let geometry = LaunchGeometry::new([256, 1, 1], [256, 1, 1]).unwrap();
        let user_data = [0xaaaa_0000, 0xbbbb_0001, 0xcccc_0002, 0xdddd_0003];
        let mut commands = Gfx12Pm4CommandBuffer::new_stateful();
        commands
            .dispatch_image(&image, geometry, 0, &user_data)
            .unwrap();
        let user_data_offset = commands
            .dwords()
            .windows(2)
            .position(|words| words[1] == COMPUTE_USER_DATA_0)
            .expect("USER_DATA register write");
        assert_eq!(
            &commands.dwords()[user_data_offset + 2..user_data_offset + 6],
            &user_data
        );
    }

    #[test]
    fn radv_dispatch_mode_uses_workgroup_counts_and_initiator() {
        let image = Gfx12KernelImage {
            code_entry: 0x1_0000,
            compute_pgm_rsrc1: 0,
            compute_pgm_rsrc2: 0,
            compute_pgm_rsrc3: 0,
            group_segment_size: 0,
            private_segment_size: 0,
            dynamic_callstack: false,
            wave32: true,
        };
        let geometry = LaunchGeometry::new([512, 1, 1], [256, 1, 1]).unwrap();
        let mut commands = Gfx12Pm4CommandBuffer::new();
        commands
            .dispatch_image_with_mode(&image, geometry, 0, &[], Gfx12DispatchMode::RadvWorkgroups)
            .unwrap();
        let dispatch = commands
            .dwords()
            .iter()
            .position(|word| *word == packet3(PACKET3_DISPATCH_DIRECT, 4, true))
            .unwrap();
        assert_eq!(
            &commands.dwords()[dispatch + 1..dispatch + 5],
            &[2, 1, 1, 0xa045]
        );
    }

    #[test]
    fn wave64_dispatch_clears_cs_w32_in_both_dimension_modes() {
        let image = Gfx12KernelImage {
            code_entry: 0x1_0000,
            compute_pgm_rsrc1: 0,
            compute_pgm_rsrc2: 0,
            compute_pgm_rsrc3: 0,
            group_segment_size: 0,
            private_segment_size: 0,
            dynamic_callstack: false,
            wave32: false,
        };
        let geometry = LaunchGeometry::new([512, 1, 1], [256, 1, 1]).unwrap();
        for (mode, expected) in [
            (Gfx12DispatchMode::Workitems, [512, 1, 1, 0x25]),
            (Gfx12DispatchMode::RadvWorkgroups, [2, 1, 1, 0x2045]),
        ] {
            let mut commands = Gfx12Pm4CommandBuffer::new();
            commands
                .dispatch_image_with_mode(&image, geometry, 0, &[], mode)
                .unwrap();
            let dispatch = commands
                .dwords()
                .iter()
                .position(|word| *word == packet3(PACKET3_DISPATCH_DIRECT, 4, true))
                .unwrap();
            assert_eq!(&commands.dwords()[dispatch + 1..dispatch + 5], &expected);
        }
    }

    #[test]
    fn abi_neutral_image_rejects_invalid_entry_and_user_data_width() {
        let mut image = Gfx12KernelImage {
            code_entry: 0x1_0001,
            compute_pgm_rsrc1: 0,
            compute_pgm_rsrc2: 0,
            compute_pgm_rsrc3: 0,
            group_segment_size: 0,
            private_segment_size: 0,
            dynamic_callstack: false,
            wave32: true,
        };
        let geometry = LaunchGeometry::new([1, 1, 1], [1, 1, 1]).unwrap();
        let mut commands = Gfx12Pm4CommandBuffer::new();
        assert_eq!(
            commands.dispatch_image(&image, geometry, 0, &[]),
            Err(Pm4BuildError::InvalidCodeEntry(0x1_0001))
        );
        image.code_entry = 0x1_0000;
        assert_eq!(
            commands.dispatch_image(&image, geometry, 0, &[0; 17]),
            Err(Pm4BuildError::TooManyUserSgprs(17))
        );
    }
}
