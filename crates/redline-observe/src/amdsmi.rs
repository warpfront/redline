// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! AMD SMI telemetry via `libamd_smi.so.26` (ROCm 7.14 / AMDSMI 26.5).
//!
//! Header provenance: `/opt/rocm/core/include/amd_smi/amdsmi.h`. Layouts and
//! sizes were measured with gcc against that header (e.g. `amdsmi_gpu_metrics_t`
//! = 4720 B). Missing library or symbol is a named error citing ROCm >= 7.14.
//!
//! Minimal snapshot path:
//! `amdsmi_init(AMDSMI_INIT_AMD_GPUS)` → socket/processor enumeration →
//! per-GPU clock/temp/power (+ optional fan RPM) → `amdsmi_shut_down` on Drop.

use std::ffi::c_void;
use std::mem::{align_of, size_of};
use std::ptr;
use std::sync::Arc;

use libloading::Library;
use thiserror::Error;

// ---------------------------------------------------------------------------
// Constants (amdsmi.h)
// ---------------------------------------------------------------------------

/// `AMDSMI_INIT_AMD_GPUS = (1 << 1)`.
pub const AMDSMI_INIT_AMD_GPUS: u64 = 1 << 1;

/// `AMDSMI_MAX_DEVICES`.
pub const AMDSMI_MAX_DEVICES: u32 = 32;

/// `AMDSMI_MAX_NUM_FREQUENCIES`.
pub const AMDSMI_MAX_NUM_FREQUENCIES: usize = 33;

/// `AMDSMI_NUM_HBM_INSTANCES`.
pub const AMDSMI_NUM_HBM_INSTANCES: usize = 4;
/// `AMDSMI_MAX_NUM_VCN`.
pub const AMDSMI_MAX_NUM_VCN: usize = 4;
/// `AMDSMI_MAX_NUM_CLKS`.
pub const AMDSMI_MAX_NUM_CLKS: usize = 4;
/// `AMDSMI_MAX_NUM_XGMI_LINKS`.
pub const AMDSMI_MAX_NUM_XGMI_LINKS: usize = 8;
/// `AMDSMI_MAX_NUM_GFX_CLKS`.
pub const AMDSMI_MAX_NUM_GFX_CLKS: usize = 8;
/// `AMDSMI_MAX_NUM_JPEG`.
pub const AMDSMI_MAX_NUM_JPEG: usize = 32;
/// `AMDSMI_MAX_NUM_JPEG_ENG_V1`.
pub const AMDSMI_MAX_NUM_JPEG_ENG_V1: usize = 40;
/// `AMDSMI_MAX_NUM_XCC`.
pub const AMDSMI_MAX_NUM_XCC: usize = 8;
/// `AMDSMI_MAX_NUM_XCP`.
pub const AMDSMI_MAX_NUM_XCP: usize = 8;
/// `AMDSMI_MAX_NUM_HBM_STACKS`.
pub const AMDSMI_MAX_NUM_HBM_STACKS: usize = 12;
/// `AMDSMI_MAX_NUM_AID`.
pub const AMDSMI_MAX_NUM_AID: usize = 2;
/// `AMDSMI_MAX_NUM_MID`.
pub const AMDSMI_MAX_NUM_MID: usize = 2;
/// `AMDSMI_MAX_NUM_CLKS_PER_AID`.
pub const AMDSMI_MAX_NUM_CLKS_PER_AID: usize = 2;
/// `AMDSMI_MAX_NUM_CLKS_PER_MID`.
pub const AMDSMI_MAX_NUM_CLKS_PER_MID: usize = 2;

/// `AMDSMI_STATUS_SUCCESS`.
pub const AMDSMI_STATUS_SUCCESS: Status = 0;
/// `AMDSMI_STATUS_NOT_SUPPORTED`.
pub const AMDSMI_STATUS_NOT_SUPPORTED: Status = 2;

/// `AMDSMI_PROCESSOR_TYPE_AMD_GPU`.
pub const AMDSMI_PROCESSOR_TYPE_AMD_GPU: ProcessorType = 1;

/// `AMDSMI_CLK_TYPE_SYS` / `AMDSMI_CLK_TYPE_GFX`.
pub const AMDSMI_CLK_TYPE_GFX: ClkType = 0;
/// `AMDSMI_CLK_TYPE_MEM`.
pub const AMDSMI_CLK_TYPE_MEM: ClkType = 4;

/// `AMDSMI_TEMPERATURE_TYPE_EDGE`.
pub const AMDSMI_TEMPERATURE_TYPE_EDGE: TemperatureType = 0;
/// `AMDSMI_TEMPERATURE_TYPE_HOTSPOT`.
pub const AMDSMI_TEMPERATURE_TYPE_HOTSPOT: TemperatureType = 1;
/// `AMDSMI_TEMPERATURE_TYPE_JUNCTION` — alias of HOTSPOT.
pub const AMDSMI_TEMPERATURE_TYPE_JUNCTION: TemperatureType = AMDSMI_TEMPERATURE_TYPE_HOTSPOT;

/// `AMDSMI_TEMP_CURRENT`.
pub const AMDSMI_TEMP_CURRENT: TemperatureMetric = 0;

// ---------------------------------------------------------------------------
// Opaque / scalar FFI types
// ---------------------------------------------------------------------------

pub type Status = i32;
pub type ClkType = u32;
pub type TemperatureType = u32;
pub type TemperatureMetric = u32;
pub type ProcessorType = u32;

/// `amdsmi_processor_handle` (`void*`).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ProcessorHandle(pub *mut c_void);

/// `amdsmi_socket_handle` (`void*`).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SocketHandle(pub *mut c_void);

// SAFETY: handles are opaque library tokens; the library mapping outlives them
// for as long as `AmdSmi` is alive. Cross-thread use matches the C API.
unsafe impl Send for ProcessorHandle {}
unsafe impl Sync for ProcessorHandle {}
unsafe impl Send for SocketHandle {}
unsafe impl Sync for SocketHandle {}

// ---------------------------------------------------------------------------
// Transcribed structs (amdsmi.h) + static layout asserts
// ---------------------------------------------------------------------------

/// `amd_metrics_table_header_t` (4 B).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct AmdMetricsTableHeader {
    pub structure_size: u16,
    pub format_revision: u8,
    pub content_revision: u8,
}

/// `amdsmi_clk_info_t` (32 B). Clocks in MHz.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AmdSmiClkInfo {
    pub clk: u32,
    pub min_clk: u32,
    pub max_clk: u32,
    pub clk_locked: u8,
    pub clk_deep_sleep: u8,
    pub _pad: [u8; 2],
    pub reserved: [u32; 4],
}

/// `amdsmi_frequencies_t` (280 B). Frequencies in Hz.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AmdSmiFrequencies {
    pub has_deep_sleep: bool,
    pub _pad0: [u8; 3],
    pub num_supported: u32,
    pub current: u32,
    pub _pad1: [u8; 4],
    pub frequency: [u64; AMDSMI_MAX_NUM_FREQUENCIES],
}

/// `amdsmi_power_info_t` (192 B).
///
/// On linux_bm: `socket_power` / current / average in W; `power_limit` in uW.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AmdSmiPowerInfo {
    pub socket_power: u64,
    pub current_socket_power: u32,
    pub average_socket_power: u32,
    pub gfx_voltage: u64,
    pub soc_voltage: u64,
    pub mem_voltage: u64,
    pub power_limit: u32,
    pub ubb_power: u32,
    pub reserved: [u64; 18],
}

/// `amdsmi_power_cap_info_t` (64 B). Cap fields in uW on linux_bm.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AmdSmiPowerCapInfo {
    pub power_cap: u64,
    pub default_power_cap: u64,
    pub dpm_cap: u64,
    pub min_power_cap: u64,
    pub max_power_cap: u64,
    pub reserved: [u64; 3],
}

/// `amdsmi_gpu_xcp_metrics_t` (520 B).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AmdSmiGpuXcpMetrics {
    pub gfx_busy_inst: [u32; AMDSMI_MAX_NUM_XCC],
    pub jpeg_busy: [u16; AMDSMI_MAX_NUM_JPEG_ENG_V1],
    pub vcn_busy: [u16; AMDSMI_MAX_NUM_VCN],
    pub gfx_busy_acc: [u64; AMDSMI_MAX_NUM_XCC],
    pub gfx_below_host_limit_acc: [u64; AMDSMI_MAX_NUM_XCC],
    pub gfx_below_host_limit_ppt_acc: [u64; AMDSMI_MAX_NUM_XCC],
    pub gfx_below_host_limit_thm_acc: [u64; AMDSMI_MAX_NUM_XCC],
    pub gfx_low_utilization_acc: [u64; AMDSMI_MAX_NUM_XCC],
    pub gfx_below_host_limit_total_acc: [u64; AMDSMI_MAX_NUM_XCC],
    pub temperature_xcd: [u16; AMDSMI_MAX_NUM_XCC],
}

/// Opaque stand-in for `amdsmi_apu_metrics_t` (344 B). Only the pointer slot in
/// [`AmdSmiGpuMetrics`] is used by this crate.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AmdSmiApuMetrics {
    pub _opaque: [u8; 344],
}

/// `amdsmi_gpu_metrics_t` (4720 B). Bulk device / partition metrics blob.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AmdSmiGpuMetrics {
    pub common_header: AmdMetricsTableHeader,
    pub temperature_edge: u16,
    pub temperature_hotspot: u16,
    pub temperature_mem: u16,
    pub temperature_vrgfx: u16,
    pub temperature_vrsoc: u16,
    pub temperature_vrmem: u16,
    pub average_gfx_activity: u16,
    pub average_umc_activity: u16,
    pub average_mm_activity: u16,
    pub average_socket_power: u16,
    pub energy_accumulator: u64,
    pub system_clock_counter: u64,
    pub average_gfxclk_frequency: u16,
    pub average_socclk_frequency: u16,
    pub average_uclk_frequency: u16,
    pub average_vclk0_frequency: u16,
    pub average_dclk0_frequency: u16,
    pub average_vclk1_frequency: u16,
    pub average_dclk1_frequency: u16,
    pub current_gfxclk: u16,
    pub current_socclk: u16,
    pub current_uclk: u16,
    pub current_vclk0: u16,
    pub current_dclk0: u16,
    pub current_vclk1: u16,
    pub current_dclk1: u16,
    pub throttle_status: u32,
    pub current_fan_speed: u16,
    pub pcie_link_width: u16,
    pub pcie_link_speed: u16,
    pub _pad_after_pcie_link_speed: [u8; 2],
    pub gfx_activity_acc: u32,
    pub mem_activity_acc: u32,
    pub temperature_hbm: [u16; AMDSMI_NUM_HBM_INSTANCES],
    pub firmware_timestamp: u64,
    pub voltage_soc: u16,
    pub voltage_gfx: u16,
    pub voltage_mem: u16,
    pub _pad_after_voltage_mem: [u8; 2],
    pub indep_throttle_status: u64,
    pub current_socket_power: u16,
    pub vcn_activity: [u16; AMDSMI_MAX_NUM_VCN],
    pub _pad_after_vcn_activity: [u8; 2],
    pub gfxclk_lock_status: u32,
    pub xgmi_link_width: u16,
    pub xgmi_link_speed: u16,
    pub _pad_after_xgmi_link_speed: [u8; 4],
    pub pcie_bandwidth_acc: u64,
    pub pcie_bandwidth_inst: u64,
    pub pcie_l0_to_recov_count_acc: u64,
    pub pcie_replay_count_acc: u64,
    pub pcie_replay_rover_count_acc: u64,
    pub xgmi_read_data_acc: [u64; AMDSMI_MAX_NUM_XGMI_LINKS],
    pub xgmi_write_data_acc: [u64; AMDSMI_MAX_NUM_XGMI_LINKS],
    pub current_gfxclks: [u16; AMDSMI_MAX_NUM_GFX_CLKS],
    pub current_socclks: [u16; AMDSMI_MAX_NUM_CLKS],
    pub current_vclk0s: [u16; AMDSMI_MAX_NUM_CLKS],
    pub current_dclk0s: [u16; AMDSMI_MAX_NUM_CLKS],
    pub jpeg_activity: [u16; AMDSMI_MAX_NUM_JPEG],
    pub pcie_nak_sent_count_acc: u32,
    pub pcie_nak_rcvd_count_acc: u32,
    pub accumulation_counter: u64,
    pub prochot_residency_acc: u64,
    pub ppt_residency_acc: u64,
    pub socket_thm_residency_acc: u64,
    pub vr_thm_residency_acc: u64,
    pub hbm_thm_residency_acc: u64,
    pub num_partition: u16,
    pub _pad_after_num_partition: [u8; 6],
    pub xcp_stats: [AmdSmiGpuXcpMetrics; AMDSMI_MAX_NUM_XCP],
    pub pcie_lc_perf_other_end_recovery: u32,
    pub _pad_after_pcie_lc: [u8; 4],
    pub vram_max_bandwidth: u64,
    pub xgmi_link_status: [u16; AMDSMI_MAX_NUM_XGMI_LINKS],
    pub temperature_hbm_stacks: [u16; AMDSMI_MAX_NUM_HBM_STACKS],
    pub temperature_mid: [u16; AMDSMI_MAX_NUM_MID],
    pub temperature_aid: [u16; AMDSMI_MAX_NUM_AID],
    pub current_uclk_aid: [u16; AMDSMI_MAX_NUM_CLKS_PER_AID],
    pub current_socclks_mid: [u16; AMDSMI_MAX_NUM_CLKS_PER_MID],
    /// Thread-local library storage; invalidated by any subsequent metrics call.
    pub apu_metrics: *mut AmdSmiApuMetrics,
}

const _: () = {
    assert!(size_of::<AmdMetricsTableHeader>() == 4);
    assert!(align_of::<AmdMetricsTableHeader>() == 2);

    assert!(size_of::<AmdSmiClkInfo>() == 32);
    assert!(align_of::<AmdSmiClkInfo>() == 4);

    assert!(size_of::<AmdSmiFrequencies>() == 280);
    assert!(align_of::<AmdSmiFrequencies>() == 8);

    assert!(size_of::<AmdSmiPowerInfo>() == 192);
    assert!(align_of::<AmdSmiPowerInfo>() == 8);

    assert!(size_of::<AmdSmiPowerCapInfo>() == 64);
    assert!(align_of::<AmdSmiPowerCapInfo>() == 8);

    assert!(size_of::<AmdSmiGpuXcpMetrics>() == 520);
    assert!(align_of::<AmdSmiGpuXcpMetrics>() == 8);

    assert!(size_of::<AmdSmiApuMetrics>() == 344);

    assert!(size_of::<AmdSmiGpuMetrics>() == 4720);
    assert!(align_of::<AmdSmiGpuMetrics>() == 8);

    assert!(size_of::<ProcessorHandle>() == size_of::<usize>());
    assert!(size_of::<SocketHandle>() == size_of::<usize>());
    assert!(size_of::<Status>() == 4);
};

// ---------------------------------------------------------------------------
// Function pointer types
// ---------------------------------------------------------------------------

type InitFn = unsafe extern "C" fn(u64) -> Status;
type ShutDownFn = unsafe extern "C" fn() -> Status;
type GetSocketHandlesFn = unsafe extern "C" fn(*mut u32, *mut SocketHandle) -> Status;
type GetProcessorHandlesFn =
    unsafe extern "C" fn(SocketHandle, *mut u32, *mut ProcessorHandle) -> Status;
type GetProcessorTypeFn = unsafe extern "C" fn(ProcessorHandle, *mut ProcessorType) -> Status;
type GetClockInfoFn = unsafe extern "C" fn(ProcessorHandle, ClkType, *mut AmdSmiClkInfo) -> Status;
type GetClkFreqFn =
    unsafe extern "C" fn(ProcessorHandle, ClkType, *mut AmdSmiFrequencies) -> Status;
type GetTempMetricFn =
    unsafe extern "C" fn(ProcessorHandle, TemperatureType, TemperatureMetric, *mut i64) -> Status;
type GetPowerInfoFn = unsafe extern "C" fn(ProcessorHandle, *mut AmdSmiPowerInfo) -> Status;
type GetPowerCapInfoFn =
    unsafe extern "C" fn(ProcessorHandle, u32, *mut AmdSmiPowerCapInfo) -> Status;
type GetGpuMetricsInfoFn = unsafe extern "C" fn(ProcessorHandle, *mut AmdSmiGpuMetrics) -> Status;
type GetGpuPartitionMetricsInfoFn =
    unsafe extern "C" fn(ProcessorHandle, *mut AmdSmiGpuMetrics) -> Status;
type GetGpuFanRpmsFn = unsafe extern "C" fn(ProcessorHandle, u32, *mut i64) -> Status;

struct Symbols {
    _lib: Arc<Library>,
    /// Candidate path/soname that successfully opened.
    library_path: String,
    init: InitFn,
    shut_down: ShutDownFn,
    get_socket_handles: GetSocketHandlesFn,
    get_processor_handles: GetProcessorHandlesFn,
    get_processor_type: GetProcessorTypeFn,
    get_clock_info: GetClockInfoFn,
    get_clk_freq: GetClkFreqFn,
    get_temp_metric: GetTempMetricFn,
    get_power_info: GetPowerInfoFn,
    get_power_cap_info: GetPowerCapInfoFn,
    get_gpu_metrics_info: GetGpuMetricsInfoFn,
    get_gpu_partition_metrics_info: GetGpuPartitionMetricsInfoFn,
    get_gpu_fan_rpms: GetGpuFanRpmsFn,
}

// SAFETY: C entry points are process-global; library mapping is retained.
unsafe impl Send for Symbols {}
unsafe impl Sync for Symbols {}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// High-level telemetry sample for one GPU.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TelemetrySnapshot {
    /// Graphics (SCLK / GFX) clock in MHz.
    pub sclk_mhz: u32,
    /// Memory (MCLK) clock in MHz.
    pub mclk_mhz: u32,
    /// Edge temperature in °C.
    pub edge_temp_c: i64,
    /// Junction / hotspot temperature in °C.
    pub junction_temp_c: i64,
    /// Instantaneous socket power in watts.
    pub power_w: f64,
    /// Current power cap in watts.
    pub power_cap_w: f64,
    /// Fan speed in RPM when the sensor is present.
    pub fan_rpm: Option<i64>,
}

/// Loaded AMD SMI session: init on construct, shut_down on [`Drop`].
pub struct AmdSmi {
    symbols: Arc<Symbols>,
    gpus: Vec<ProcessorHandle>,
    /// When false, Drop skips `amdsmi_shut_down` (init never succeeded).
    initialized: bool,
}

/// Errors from loading or querying AMD SMI.
#[derive(Debug, Error)]
pub enum AmdSmiError {
    #[error("could not load libamd_smi (requires ROCm >= 7.14); tried {candidates}: {detail}")]
    Library { candidates: String, detail: String },

    #[error("missing libamd_smi symbol `{symbol}` (requires ROCm >= 7.14)")]
    MissingSymbol { symbol: &'static str },

    #[error("amdsmi_{api} failed with status {status} (requires ROCm >= 7.14)")]
    Status { api: &'static str, status: Status },

    #[error("GPU index {index} out of range (have {count} AMD GPU processor(s))")]
    GpuIndexOutOfRange { index: u32, count: u32 },
}

/// Loader candidate order: bare soname, TheRock core path, legacy path.
pub const LIBRARY_CANDIDATES: &[&str] = &[
    "libamd_smi.so.26",
    "/opt/rocm/core/lib/libamd_smi.so.26",
    "/opt/rocm/lib/libamd_smi.so.26",
];

impl AmdSmi {
    /// Open `libamd_smi.so.26`, resolve symbols, init AMD GPUs, enumerate handles.
    pub fn new() -> Result<Self, AmdSmiError> {
        Self::new_from_candidates(LIBRARY_CANDIDATES)
    }

    /// Load from an explicit candidate list (tests / diagnostics).
    pub fn new_from_candidates(candidates: &[&str]) -> Result<Self, AmdSmiError> {
        let symbols = Arc::new(Symbols::open(candidates)?);
        let mut session = Self {
            symbols,
            gpus: Vec::new(),
            initialized: false,
        };
        session.init_and_enumerate()?;
        Ok(session)
    }

    /// Number of AMD GPU processors discovered at init.
    pub fn gpu_count(&self) -> u32 {
        self.gpus.len() as u32
    }

    /// Path or soname of the shared library that was opened.
    pub fn library_path(&self) -> &str {
        &self.symbols.library_path
    }

    /// Snapshot clocks, temperatures, power, and optional fan RPM for `gpu_index`.
    pub fn snapshot(&self, gpu_index: u32) -> Result<TelemetrySnapshot, AmdSmiError> {
        let handle = self.gpu_handle(gpu_index)?;

        let mut gfx = zeroed_clk();
        check(
            "get_clock_info",
            // SAFETY: handle from this session; out-struct is valid stack memory.
            unsafe { (self.symbols.get_clock_info)(handle, AMDSMI_CLK_TYPE_GFX, &mut gfx) },
        )?;

        let mut mem = zeroed_clk();
        check("get_clock_info", unsafe {
            (self.symbols.get_clock_info)(handle, AMDSMI_CLK_TYPE_MEM, &mut mem)
        })?;

        let mut edge_temp_c: i64 = 0;
        check("get_temp_metric", unsafe {
            (self.symbols.get_temp_metric)(
                handle,
                AMDSMI_TEMPERATURE_TYPE_EDGE,
                AMDSMI_TEMP_CURRENT,
                &mut edge_temp_c,
            )
        })?;

        let mut junction_temp_c: i64 = 0;
        check("get_temp_metric", unsafe {
            (self.symbols.get_temp_metric)(
                handle,
                AMDSMI_TEMPERATURE_TYPE_JUNCTION,
                AMDSMI_TEMP_CURRENT,
                &mut junction_temp_c,
            )
        })?;

        let mut power = zeroed_power();
        check("get_power_info", unsafe {
            (self.symbols.get_power_info)(handle, &mut power)
        })?;

        let mut cap = zeroed_power_cap();
        check(
            "get_power_cap_info",
            // sensor_ind 0 is the conventional primary sensor.
            unsafe { (self.symbols.get_power_cap_info)(handle, 0, &mut cap) },
        )?;

        let fan_rpm = {
            let mut rpm: i64 = 0;
            // SAFETY: optional sensor; only NOT_SUPPORTED → None.
            let st = unsafe { (self.symbols.get_gpu_fan_rpms)(handle, 0, &mut rpm) };
            if st == AMDSMI_STATUS_SUCCESS {
                Some(rpm)
            } else if st == AMDSMI_STATUS_NOT_SUPPORTED {
                None
            } else {
                return Err(AmdSmiError::Status {
                    api: "get_gpu_fan_rpms",
                    status: st,
                });
            }
        };

        Ok(TelemetrySnapshot {
            sclk_mhz: gfx.clk,
            mclk_mhz: mem.clk,
            edge_temp_c,
            junction_temp_c,
            power_w: power_watts(&power),
            power_cap_w: uw_to_w(cap.power_cap),
            fan_rpm,
        })
    }

    /// Bulk GPU metrics (`amdsmi_get_gpu_metrics_info`).
    pub fn gpu_metrics(&self, gpu_index: u32) -> Result<AmdSmiGpuMetrics, AmdSmiError> {
        let handle = self.gpu_handle(gpu_index)?;
        let mut metrics = zeroed_metrics();
        check("get_gpu_metrics_info", unsafe {
            (self.symbols.get_gpu_metrics_info)(handle, &mut metrics)
        })?;
        Ok(metrics)
    }

    /// Partition-scoped metrics (`amdsmi_get_gpu_partition_metrics_info`).
    ///
    /// Same blob shape as [`Self::gpu_metrics`], scoped to the active partition
    /// (CPX/DPX/… when applicable).
    pub fn partition_metrics(&self, gpu_index: u32) -> Result<AmdSmiGpuMetrics, AmdSmiError> {
        let handle = self.gpu_handle(gpu_index)?;
        let mut metrics = zeroed_metrics();
        check("get_gpu_partition_metrics_info", unsafe {
            (self.symbols.get_gpu_partition_metrics_info)(handle, &mut metrics)
        })?;
        Ok(metrics)
    }

    /// Discrete frequency table (`amdsmi_get_clk_freq`); values in Hz.
    pub fn clk_freq(
        &self,
        gpu_index: u32,
        clk_type: ClkType,
    ) -> Result<AmdSmiFrequencies, AmdSmiError> {
        let handle = self.gpu_handle(gpu_index)?;
        let mut freq = zeroed_freq();
        check("get_clk_freq", unsafe {
            (self.symbols.get_clk_freq)(handle, clk_type, &mut freq)
        })?;
        Ok(freq)
    }

    fn gpu_handle(&self, gpu_index: u32) -> Result<ProcessorHandle, AmdSmiError> {
        self.gpus
            .get(gpu_index as usize)
            .copied()
            .ok_or(AmdSmiError::GpuIndexOutOfRange {
                index: gpu_index,
                count: self.gpu_count(),
            })
    }

    fn init_and_enumerate(&mut self) -> Result<(), AmdSmiError> {
        check(
            "init",
            // SAFETY: process-global AMDSMI init; matched by Drop → shut_down.
            unsafe { (self.symbols.init)(AMDSMI_INIT_AMD_GPUS) },
        )?;
        self.initialized = true;

        let mut socket_count: u32 = 0;
        check("get_socket_handles", unsafe {
            (self.symbols.get_socket_handles)(&mut socket_count, ptr::null_mut())
        })?;

        let mut sockets = vec![SocketHandle(ptr::null_mut()); socket_count as usize];
        if socket_count > 0 {
            check("get_socket_handles", unsafe {
                (self.symbols.get_socket_handles)(&mut socket_count, sockets.as_mut_ptr())
            })?;
            sockets.truncate(socket_count as usize);
        }

        let mut gpus = Vec::new();
        for socket in sockets {
            let mut processor_count: u32 = 0;
            check("get_processor_handles", unsafe {
                (self.symbols.get_processor_handles)(socket, &mut processor_count, ptr::null_mut())
            })?;

            let mut processors = vec![ProcessorHandle(ptr::null_mut()); processor_count as usize];
            if processor_count > 0 {
                check("get_processor_handles", unsafe {
                    (self.symbols.get_processor_handles)(
                        socket,
                        &mut processor_count,
                        processors.as_mut_ptr(),
                    )
                })?;
                processors.truncate(processor_count as usize);
            }

            for proc in processors {
                let mut ty: ProcessorType = 0;
                check("get_processor_type", unsafe {
                    (self.symbols.get_processor_type)(proc, &mut ty)
                })?;
                if ty == AMDSMI_PROCESSOR_TYPE_AMD_GPU {
                    gpus.push(proc);
                }
            }
        }

        self.gpus = gpus;
        Ok(())
    }
}

impl Drop for AmdSmi {
    fn drop(&mut self) {
        if self.initialized {
            // SAFETY: pairs with successful amdsmi_init in new().
            let _ = unsafe { (self.symbols.shut_down)() };
            self.initialized = false;
        }
    }
}

impl Symbols {
    fn open(candidates: &[&str]) -> Result<Self, AmdSmiError> {
        let mut failures = Vec::new();
        let (library, library_path) = candidates
            .iter()
            .find_map(|candidate| {
                // SAFETY: loading installed libamd_smi is the purpose of this module.
                match unsafe { Library::new(candidate) } {
                    Ok(lib) => Some((Arc::new(lib), (*candidate).to_owned())),
                    Err(error) => {
                        failures.push(format!("{candidate}: {error}"));
                        None
                    }
                }
            })
            .ok_or_else(|| AmdSmiError::Library {
                candidates: candidates.join(", "),
                detail: failures.join("; "),
            })?;

        // SAFETY: each lookup is a public C symbol from the retained mapping.
        unsafe {
            Ok(Self {
                _lib: library.clone(),
                library_path,
                init: resolve(&library, b"amdsmi_init\0", "amdsmi_init")?,
                shut_down: resolve(&library, b"amdsmi_shut_down\0", "amdsmi_shut_down")?,
                get_socket_handles: resolve(
                    &library,
                    b"amdsmi_get_socket_handles\0",
                    "amdsmi_get_socket_handles",
                )?,
                get_processor_handles: resolve(
                    &library,
                    b"amdsmi_get_processor_handles\0",
                    "amdsmi_get_processor_handles",
                )?,
                get_processor_type: resolve(
                    &library,
                    b"amdsmi_get_processor_type\0",
                    "amdsmi_get_processor_type",
                )?,
                get_clock_info: resolve(
                    &library,
                    b"amdsmi_get_clock_info\0",
                    "amdsmi_get_clock_info",
                )?,
                get_clk_freq: resolve(&library, b"amdsmi_get_clk_freq\0", "amdsmi_get_clk_freq")?,
                get_temp_metric: resolve(
                    &library,
                    b"amdsmi_get_temp_metric\0",
                    "amdsmi_get_temp_metric",
                )?,
                get_power_info: resolve(
                    &library,
                    b"amdsmi_get_power_info\0",
                    "amdsmi_get_power_info",
                )?,
                get_power_cap_info: resolve(
                    &library,
                    b"amdsmi_get_power_cap_info\0",
                    "amdsmi_get_power_cap_info",
                )?,
                get_gpu_metrics_info: resolve(
                    &library,
                    b"amdsmi_get_gpu_metrics_info\0",
                    "amdsmi_get_gpu_metrics_info",
                )?,
                get_gpu_partition_metrics_info: resolve(
                    &library,
                    b"amdsmi_get_gpu_partition_metrics_info\0",
                    "amdsmi_get_gpu_partition_metrics_info",
                )?,
                get_gpu_fan_rpms: resolve(
                    &library,
                    b"amdsmi_get_gpu_fan_rpms\0",
                    "amdsmi_get_gpu_fan_rpms",
                )?,
            })
        }
    }
}

unsafe fn resolve<T: Copy>(
    library: &Library,
    name: &'static [u8],
    symbol: &'static str,
) -> Result<T, AmdSmiError> {
    // SAFETY: `name` is a static NUL-terminated public C symbol; T is an
    // extern "C" fn pointer matching the header declaration.
    let sym = unsafe {
        library
            .get::<T>(name)
            .map_err(|_| AmdSmiError::MissingSymbol { symbol })?
    };
    Ok(*sym)
}

fn check(api: &'static str, status: Status) -> Result<(), AmdSmiError> {
    if status == AMDSMI_STATUS_SUCCESS {
        Ok(())
    } else {
        Err(AmdSmiError::Status { api, status })
    }
}

fn uw_to_w(uw: u64) -> f64 {
    (uw as f64) / 1_000_000.0
}

/// Prefer MI300+ current_socket_power, then average, then socket_power (W).
fn power_watts(info: &AmdSmiPowerInfo) -> f64 {
    const U32_MAX: u32 = u32::MAX;
    if info.current_socket_power != U32_MAX {
        f64::from(info.current_socket_power)
    } else if info.average_socket_power != U32_MAX {
        f64::from(info.average_socket_power)
    } else if info.socket_power != u64::from(U32_MAX) && info.socket_power != u64::MAX {
        info.socket_power as f64
    } else {
        f64::NAN
    }
}

fn zeroed_clk() -> AmdSmiClkInfo {
    AmdSmiClkInfo {
        clk: 0,
        min_clk: 0,
        max_clk: 0,
        clk_locked: 0,
        clk_deep_sleep: 0,
        _pad: [0; 2],
        reserved: [0; 4],
    }
}

fn zeroed_freq() -> AmdSmiFrequencies {
    AmdSmiFrequencies {
        has_deep_sleep: false,
        _pad0: [0; 3],
        num_supported: 0,
        current: 0,
        _pad1: [0; 4],
        frequency: [0; AMDSMI_MAX_NUM_FREQUENCIES],
    }
}

fn zeroed_power() -> AmdSmiPowerInfo {
    AmdSmiPowerInfo {
        socket_power: 0,
        current_socket_power: 0,
        average_socket_power: 0,
        gfx_voltage: 0,
        soc_voltage: 0,
        mem_voltage: 0,
        power_limit: 0,
        ubb_power: 0,
        reserved: [0; 18],
    }
}

fn zeroed_power_cap() -> AmdSmiPowerCapInfo {
    AmdSmiPowerCapInfo {
        power_cap: 0,
        default_power_cap: 0,
        dpm_cap: 0,
        min_power_cap: 0,
        max_power_cap: 0,
        reserved: [0; 3],
    }
}

fn zeroed_metrics() -> AmdSmiGpuMetrics {
    // SAFETY: all-zero bit pattern is valid for this POD metrics blob (pointer
    // field becomes null, which the C API documents for non-APU devices).
    unsafe { std::mem::zeroed() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_nonexistent_path_yields_library_error() {
        let result = AmdSmi::new_from_candidates(&["/nonexistent/libamd_smi-redline-test.so.26"]);
        let err = match result {
            Err(e) => e,
            Ok(_) => panic!("nonexistent path must fail"),
        };
        match err {
            AmdSmiError::Library { candidates, detail } => {
                assert!(
                    candidates.contains("nonexistent"),
                    "candidates={candidates}"
                );
                assert!(!detail.is_empty(), "detail should describe dlopen failure");
                let msg = format!(
                    "could not load libamd_smi (requires ROCm >= 7.14); tried {candidates}: {detail}"
                );
                assert!(
                    msg.contains("requires ROCm >= 7.14"),
                    "library error must cite ROCm >= 7.14: {msg}"
                );
            }
            other => panic!("expected Library error, got {other:?}"),
        }
    }

    #[test]
    fn missing_symbol_error_cites_rocm_714() {
        let err = AmdSmiError::MissingSymbol {
            symbol: "amdsmi_init",
        };
        let msg = err.to_string();
        assert!(
            msg.contains("requires ROCm >= 7.14"),
            "missing-symbol error must cite ROCm >= 7.14: {msg}"
        );
        assert!(
            msg.contains("amdsmi_init"),
            "missing-symbol error must name the symbol: {msg}"
        );
    }

    #[test]
    fn junction_aliases_hotspot() {
        assert_eq!(
            AMDSMI_TEMPERATURE_TYPE_JUNCTION,
            AMDSMI_TEMPERATURE_TYPE_HOTSPOT
        );
    }

    #[test]
    fn layout_sizes_match_header() {
        assert_eq!(size_of::<AmdSmiClkInfo>(), 32);
        assert_eq!(size_of::<AmdSmiFrequencies>(), 280);
        assert_eq!(size_of::<AmdSmiPowerInfo>(), 192);
        assert_eq!(size_of::<AmdSmiPowerCapInfo>(), 64);
        assert_eq!(size_of::<AmdSmiGpuXcpMetrics>(), 520);
        assert_eq!(size_of::<AmdSmiGpuMetrics>(), 4720);
        assert_eq!(size_of::<AmdSmiApuMetrics>(), 344);
    }
}
