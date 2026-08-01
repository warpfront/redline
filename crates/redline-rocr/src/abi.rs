// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! Minimal ROCr/HSA ABI used by the AQL backend.
//!
//! Values and layouts are copied from the MIT-licensed ROCm 7.2 public headers
//! `/opt/rocm/include/hsa/{hsa.h,hsa_ext_amd.h}`, cross-checked against
//! `rocm-systems` commit `c0430a50286200ab0562f4733445cdee6e48d416`.
//! Keeping this module small is intentional: it is an auditable dynamic-ABI
//! boundary, not a replacement for generated HSA bindings.

use std::ffi::{CStr, c_char, c_void};
use std::fmt;
use std::sync::Arc;

pub type Status = i32;
pub type SignalValue = i64;

pub const STATUS_SUCCESS: Status = 0;
pub const STATUS_INFO_BREAK: Status = 1;

pub const SYSTEM_INFO_TIMESTAMP_FREQUENCY: u32 = 3;

pub const DEVICE_TYPE_CPU: u32 = 0;
pub const DEVICE_TYPE_GPU: u32 = 1;

pub const AGENT_INFO_NAME: u32 = 0;
pub const AGENT_INFO_PROFILE: u32 = 4;
pub const AGENT_INFO_DEFAULT_FLOAT_ROUNDING_MODE: u32 = 5;
pub const AGENT_INFO_WORKGROUP_MAX_DIM: u32 = 7;
pub const AGENT_INFO_WORKGROUP_MAX_SIZE: u32 = 8;
pub const AGENT_INFO_GRID_MAX_DIM: u32 = 9;
pub const AGENT_INFO_GRID_MAX_SIZE: u32 = 10;
pub const AGENT_INFO_QUEUE_MIN_SIZE: u32 = 13;
pub const AGENT_INFO_QUEUE_MAX_SIZE: u32 = 14;
pub const AGENT_INFO_QUEUE_TYPE: u32 = 15;
pub const AGENT_INFO_DEVICE: u32 = 17;
pub const AMD_AGENT_INFO_BDFID: u32 = 0xA006;
pub const AMD_AGENT_INFO_DOMAIN: u32 = 0xA00F;
pub const AMD_AGENT_INFO_TIMESTAMP_FREQUENCY: u32 = 0xA016;
pub const AMD_AGENT_INFO_COMPUTE_UNIT_COUNT: u32 = 0xA002;
pub const AMD_AGENT_INFO_COOPERATIVE_COMPUTE_UNIT_COUNT: u32 = 0xA014;

/// `hsa_signal_condition_t` values (hsa.h).
pub const SIGNAL_CONDITION_EQ: u32 = 0;
pub const SIGNAL_CONDITION_NE: u32 = 1;
pub const SIGNAL_CONDITION_LT: u32 = 2;
pub const SIGNAL_CONDITION_GTE: u32 = 3;

/// `hsa_wait_state_t` values (hsa.h).
pub const WAIT_STATE_BLOCKED: u32 = 0;
pub const WAIT_STATE_ACTIVE: u32 = 1;

/// `hsa_amd_queue_priority_t` values (hsa_ext_amd.h).
pub const QUEUE_PRIORITY_LOW: u32 = 0;
pub const QUEUE_PRIORITY_NORMAL: u32 = 1;
pub const QUEUE_PRIORITY_HIGH: u32 = 2;

/// `hsa_queue_info_attribute_t` values (hsa_ext_amd.h).
pub const QUEUE_INFO_AGENT: u32 = 0;
pub const QUEUE_INFO_DOORBELL_ID: u32 = 1;
pub const QUEUE_INFO_USE_COUNT: u32 = 2;
pub const QUEUE_INFO_HW_ID: u32 = 3;
pub const QUEUE_INFO_PREFETCH_METADATA_DISPATCH_PKT_VERSION_MAJOR: u32 = 4;
pub const QUEUE_INFO_PREFETCH_METADATA_DISPATCH_PKT_VERSION_MINOR: u32 = 5;
pub const QUEUE_INFO_PREFETCH_METADATA_BARRIER_PKT_VERSION_MAJOR: u32 = 6;
pub const QUEUE_INFO_PREFETCH_METADATA_BARRIER_PKT_VERSION_MINOR: u32 = 7;
pub const QUEUE_INFO_PREFETCH_METADATA_RING_BUFFER: u32 = 8;
pub const QUEUE_INFO_PROPERTIES: u32 = 9;
pub const QUEUE_INFO_VM_FAULT_STATUS: u32 = 10;
pub const QUEUE_INFO_VM_FAULT_ADDRESS: u32 = 11;
pub const QUEUE_INFO_VM_FAULT_REASON: u32 = 12;

pub const PROFILE_BASE: u32 = 0;
pub const PROFILE_FULL: u32 = 1;
pub const DEFAULT_FLOAT_ROUNDING_MODE_ZERO: u32 = 1;
pub const DEFAULT_FLOAT_ROUNDING_MODE_NEAR: u32 = 2;

pub const QUEUE_TYPE_MULTI: u32 = 0;
pub const QUEUE_TYPE_SINGLE: u32 = 1;
pub const QUEUE_FEATURE_KERNEL_DISPATCH: u32 = 1;

pub const PACKET_TYPE_INVALID: u16 = 1;
pub const PACKET_TYPE_KERNEL_DISPATCH: u16 = 2;
pub const PACKET_TYPE_BARRIER_AND: u16 = 3;

pub const PACKET_HEADER_TYPE: u16 = 0;
pub const PACKET_HEADER_BARRIER: u16 = 8;
pub const PACKET_HEADER_SCACQUIRE_FENCE_SCOPE: u16 = 9;
pub const PACKET_HEADER_SCRELEASE_FENCE_SCOPE: u16 = 11;

pub const FENCE_SCOPE_NONE: u16 = 0;
pub const FENCE_SCOPE_AGENT: u16 = 1;
pub const FENCE_SCOPE_SYSTEM: u16 = 2;

pub const KERNEL_DISPATCH_SETUP_DIMENSIONS: u16 = 0;

pub const AMD_SEGMENT_GLOBAL: u32 = 0;
pub const AMD_MEMORY_POOL_GLOBAL_FLAG_KERNARG_INIT: u32 = 1;
pub const AMD_MEMORY_POOL_GLOBAL_FLAG_FINE_GRAINED: u32 = 2;
pub const AMD_MEMORY_POOL_GLOBAL_FLAG_COARSE_GRAINED: u32 = 4;
pub const AMD_MEMORY_POOL_INFO_SEGMENT: u32 = 0;
pub const AMD_MEMORY_POOL_INFO_GLOBAL_FLAGS: u32 = 1;
pub const AMD_MEMORY_POOL_INFO_RUNTIME_ALLOC_ALLOWED: u32 = 5;
pub const AMD_MEMORY_POOL_INFO_RUNTIME_ALLOC_GRANULE: u32 = 6;
pub const AMD_MEMORY_POOL_INFO_RUNTIME_ALLOC_ALIGNMENT: u32 = 7;
pub const AMD_MEMORY_POOL_INFO_LOCATION: u32 = 17;
pub const AMD_MEMORY_POOL_LOCATION_GPU: u32 = 1;
pub const AMD_MEMORY_POOL_STANDARD_FLAG: u32 = 0;
pub const AMD_MEMORY_POOL_EXECUTABLE_FLAG: u32 = 1 << 2;

pub const EXECUTABLE_SYMBOL_INFO_TYPE: u32 = 0;
pub const SYMBOL_KIND_KERNEL: u32 = 1;
pub const EXECUTABLE_SYMBOL_INFO_KERNEL_KERNARG_SEGMENT_SIZE: u32 = 11;
pub const EXECUTABLE_SYMBOL_INFO_KERNEL_KERNARG_SEGMENT_ALIGNMENT: u32 = 12;
pub const EXECUTABLE_SYMBOL_INFO_KERNEL_GROUP_SEGMENT_SIZE: u32 = 13;
pub const EXECUTABLE_SYMBOL_INFO_KERNEL_PRIVATE_SEGMENT_SIZE: u32 = 14;
pub const EXECUTABLE_SYMBOL_INFO_KERNEL_DYNAMIC_CALLSTACK: u32 = 15;
pub const EXECUTABLE_SYMBOL_INFO_KERNEL_OBJECT: u32 = 22;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Agent(pub u64);

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Signal(pub u64);

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct MemoryPool(pub u64);

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CodeObjectReader(pub u64);

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Executable(pub u64);

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct LoadedCodeObject(pub u64);

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ExecutableSymbol(pub u64);

/// Public `hsa_queue_t` layout for the 64-bit HSA machine model.
#[repr(C)]
#[derive(Debug)]
pub struct Queue {
    pub queue_type: u32,
    pub features: u32,
    pub base_address: *mut c_void,
    pub doorbell_signal: Signal,
    pub size: u32,
    pub reserved1: u32,
    pub id: u64,
}

/// Public `hsa_amd_profiling_dispatch_time_t` layout.
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProfilingDispatchTime {
    pub start: u64,
    pub end: u64,
}

/// Fixed-width stand-in for `hsa_signal_condition_t` (C enum, 4 bytes).
pub type SignalCondition = u32;

/// Fixed-width stand-in for `hsa_wait_state_t` (C enum, 4 bytes).
pub type WaitState = u32;

/// Fixed-width stand-in for `hsa_amd_queue_priority_t` (C enum, 4 bytes).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct QueuePriority(pub u32);

/// Fixed-width stand-in for `hsa_queue_info_attribute_t` (C enum, 4 bytes).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct QueueInfoAttribute(pub u32);

/// Fixed-width stand-in for `hsa_queue_type_t` (C enum, 4 bytes).
pub type QueueType = u32;

pub type AgentCallback = unsafe extern "C" fn(Agent, *mut c_void) -> Status;
pub type MemoryPoolCallback = unsafe extern "C" fn(MemoryPool, *mut c_void) -> Status;
pub type QueueErrorCallback = unsafe extern "C" fn(Status, *mut Queue, *mut c_void);

pub type InitFn = unsafe extern "C" fn() -> Status;
pub type ShutDownFn = unsafe extern "C" fn() -> Status;
pub type SystemGetInfoFn = unsafe extern "C" fn(u32, *mut c_void) -> Status;
pub type StatusStringFn = unsafe extern "C" fn(Status, *mut *const c_char) -> Status;
pub type IterateAgentsFn = unsafe extern "C" fn(Option<AgentCallback>, *mut c_void) -> Status;
pub type AgentGetInfoFn = unsafe extern "C" fn(Agent, u32, *mut c_void) -> Status;

pub type SignalCreateFn =
    unsafe extern "C" fn(SignalValue, u32, *const Agent, *mut Signal) -> Status;
pub type SignalDestroyFn = unsafe extern "C" fn(Signal) -> Status;
pub type SignalLoadScAcquireFn = unsafe extern "C" fn(Signal) -> SignalValue;
pub type SignalStoreRelaxedFn = unsafe extern "C" fn(Signal, SignalValue);
pub type SignalStoreScReleaseFn = unsafe extern "C" fn(Signal, SignalValue);

pub type QueueCreateFn = unsafe extern "C" fn(
    Agent,
    u32,
    u32,
    Option<QueueErrorCallback>,
    *mut c_void,
    u32,
    u32,
    *mut *mut Queue,
) -> Status;
pub type QueueDestroyFn = unsafe extern "C" fn(*mut Queue) -> Status;
pub type QueueInactivateFn = unsafe extern "C" fn(*mut Queue) -> Status;
pub type QueueLoadReadIndexRelaxedFn = unsafe extern "C" fn(*const Queue) -> u64;
pub type QueueLoadReadIndexScAcquireFn = unsafe extern "C" fn(*const Queue) -> u64;
pub type QueueLoadWriteIndexRelaxedFn = unsafe extern "C" fn(*const Queue) -> u64;
pub type QueueAddWriteIndexRelaxedFn = unsafe extern "C" fn(*const Queue, u64) -> u64;
pub type ProfilingSetProfilerEnabledFn = unsafe extern "C" fn(*mut Queue, i32) -> Status;
pub type ProfilingGetDispatchTimeFn =
    unsafe extern "C" fn(Agent, Signal, *mut ProfilingDispatchTime) -> Status;

pub type AgentIterateMemoryPoolsFn =
    unsafe extern "C" fn(Agent, Option<MemoryPoolCallback>, *mut c_void) -> Status;
pub type MemoryPoolGetInfoFn = unsafe extern "C" fn(MemoryPool, u32, *mut c_void) -> Status;
pub type MemoryPoolAllocateFn =
    unsafe extern "C" fn(MemoryPool, usize, u32, *mut *mut c_void) -> Status;
pub type MemoryPoolFreeFn = unsafe extern "C" fn(*mut c_void) -> Status;
pub type MemoryCopyFn = unsafe extern "C" fn(*mut c_void, *const c_void, usize) -> Status;
pub type AgentsAllowAccessFn =
    unsafe extern "C" fn(u32, *const Agent, *const u32, *const c_void) -> Status;

pub type CodeObjectReaderCreateFromMemoryFn =
    unsafe extern "C" fn(*const c_void, usize, *mut CodeObjectReader) -> Status;
pub type CodeObjectReaderDestroyFn = unsafe extern "C" fn(CodeObjectReader) -> Status;
pub type ExecutableCreateAltFn =
    unsafe extern "C" fn(u32, u32, *const c_char, *mut Executable) -> Status;
pub type ExecutableDestroyFn = unsafe extern "C" fn(Executable) -> Status;
pub type ExecutableLoadAgentCodeObjectFn = unsafe extern "C" fn(
    Executable,
    Agent,
    CodeObjectReader,
    *const c_char,
    *mut LoadedCodeObject,
) -> Status;
pub type ExecutableFreezeFn = unsafe extern "C" fn(Executable, *const c_char) -> Status;
pub type ExecutableGetSymbolByNameFn =
    unsafe extern "C" fn(Executable, *const c_char, *const Agent, *mut ExecutableSymbol) -> Status;
pub type ExecutableSymbolGetInfoFn =
    unsafe extern "C" fn(ExecutableSymbol, u32, *mut c_void) -> Status;

// ROCm 7.14 / HSA_AMD_INTERFACE_VERSION 1.26 entry points (hsa_ext_amd.h).
pub type QueueCuSetMaskFn = unsafe extern "C" fn(*const Queue, u32, *const u32) -> Status;
pub type QueueCuGetMaskFn = unsafe extern "C" fn(*const Queue, u32, *mut u32) -> Status;
pub type QueueSetPriorityFn = unsafe extern "C" fn(*mut Queue, QueuePriority) -> Status;
pub type CountedQueueAcquireFn = unsafe extern "C" fn(
    Agent,
    QueueType,
    QueuePriority,
    Option<QueueErrorCallback>,
    *mut c_void,
    u64,
    *mut *mut Queue,
) -> Status;
pub type CountedQueueReleaseFn = unsafe extern "C" fn(*mut Queue) -> Status;
pub type QueueGetInfoFn =
    unsafe extern "C" fn(*mut Queue, QueueInfoAttribute, *mut c_void) -> Status;
pub type SignalWaitAllFn = unsafe extern "C" fn(
    u32,
    *mut Signal,
    *mut SignalCondition,
    *mut SignalValue,
    u64,
    WaitState,
    *mut SignalValue,
) -> u32;
pub type SignalWaitAnyFn = unsafe extern "C" fn(
    u32,
    *mut Signal,
    *mut SignalCondition,
    *mut SignalValue,
    u64,
    WaitState,
    *mut SignalValue,
) -> u32;
pub type SvmPrefetchAsyncFn =
    unsafe extern "C" fn(*mut c_void, usize, Agent, u32, *const Signal, Signal) -> Status;
pub type SvmDiscardBatchAsyncFn =
    unsafe extern "C" fn(*mut *mut c_void, *mut usize, u32, u32, *const Signal, Signal) -> Status;
pub type ProfilingConvertTickToSystemDomainFn =
    unsafe extern "C" fn(Agent, u64, *mut u64) -> Status;

/// Dynamically resolved public ROCr entry points.
///
/// The `keepalive` object normally contains an `Arc<libloading::Library>`.
/// Keeping it here guarantees that no stored function pointer outlives the
/// loaded `libhsa-runtime64.so` image.
pub struct Symbols {
    _keepalive: Arc<dyn Send + Sync>,
    pub init: InitFn,
    pub shut_down: ShutDownFn,
    pub system_get_info: SystemGetInfoFn,
    pub status_string: StatusStringFn,
    pub iterate_agents: IterateAgentsFn,
    pub agent_get_info: AgentGetInfoFn,
    pub signal_create: SignalCreateFn,
    pub signal_destroy: SignalDestroyFn,
    pub signal_load_scacquire: SignalLoadScAcquireFn,
    pub signal_store_relaxed: SignalStoreRelaxedFn,
    pub signal_store_screlease: SignalStoreScReleaseFn,
    pub queue_create: QueueCreateFn,
    pub queue_destroy: QueueDestroyFn,
    pub queue_inactivate: QueueInactivateFn,
    pub queue_load_read_index_relaxed: QueueLoadReadIndexRelaxedFn,
    pub queue_load_read_index_scacquire: QueueLoadReadIndexScAcquireFn,
    pub queue_load_write_index_relaxed: QueueLoadWriteIndexRelaxedFn,
    pub queue_add_write_index_relaxed: QueueAddWriteIndexRelaxedFn,
    pub profiling_set_profiler_enabled: ProfilingSetProfilerEnabledFn,
    pub profiling_get_dispatch_time: ProfilingGetDispatchTimeFn,
    pub agent_iterate_memory_pools: AgentIterateMemoryPoolsFn,
    pub memory_pool_get_info: MemoryPoolGetInfoFn,
    pub memory_pool_allocate: MemoryPoolAllocateFn,
    pub memory_pool_free: MemoryPoolFreeFn,
    pub memory_copy: MemoryCopyFn,
    pub agents_allow_access: AgentsAllowAccessFn,
    pub code_object_reader_create_from_memory: CodeObjectReaderCreateFromMemoryFn,
    pub code_object_reader_destroy: CodeObjectReaderDestroyFn,
    pub executable_create_alt: ExecutableCreateAltFn,
    pub executable_destroy: ExecutableDestroyFn,
    pub executable_load_agent_code_object: ExecutableLoadAgentCodeObjectFn,
    pub executable_freeze: ExecutableFreezeFn,
    pub executable_get_symbol_by_name: ExecutableGetSymbolByNameFn,
    pub executable_symbol_get_info: ExecutableSymbolGetInfoFn,
    // ROCm >= 7.14 / interface 1.26
    pub queue_cu_set_mask: QueueCuSetMaskFn,
    pub queue_cu_get_mask: QueueCuGetMaskFn,
    pub queue_set_priority: QueueSetPriorityFn,
    pub counted_queue_acquire: CountedQueueAcquireFn,
    pub counted_queue_release: CountedQueueReleaseFn,
    pub queue_get_info: QueueGetInfoFn,
    pub signal_wait_all: SignalWaitAllFn,
    pub signal_wait_any: SignalWaitAnyFn,
    pub svm_prefetch_async: SvmPrefetchAsyncFn,
    pub svm_discard_batch_async: SvmDiscardBatchAsyncFn,
    pub profiling_convert_tick_to_system_domain: ProfilingConvertTickToSystemDomainFn,
}

impl fmt::Debug for Symbols {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Symbols").finish_non_exhaustive()
    }
}

impl Symbols {
    /// Resolve the public symbols used by this backend.
    ///
    /// # Safety
    ///
    /// Each non-null pointer returned by `resolve` must name the C function
    /// requested by the supplied symbol name, with the exact ROCm 7.2 ABI.
    /// `keepalive` must keep the containing shared object mapped for its own
    /// lifetime. Loading symbols from a different, ABI-incompatible runtime is
    /// undefined behavior.
    pub unsafe fn load(
        keepalive: Arc<dyn Send + Sync>,
        mut resolve: impl FnMut(&CStr) -> *const c_void,
    ) -> Result<Arc<Self>, MissingSymbol> {
        macro_rules! symbol {
            ($name:literal, $ty:ty) => {{
                let name = CStr::from_bytes_with_nul(concat!($name, "\0").as_bytes())
                    .expect("symbol name has one trailing NUL");
                let pointer = resolve(name);
                if pointer.is_null() {
                    return Err(MissingSymbol {
                        name: $name,
                        requires_rocm_714: false,
                    });
                }
                // SAFETY: guaranteed by this function's contract. The macro
                // names a concrete function-pointer type, which is pointer-sized.
                unsafe { std::mem::transmute::<*const c_void, $ty>(pointer) }
            }};
            (@rocm714 $name:literal, $ty:ty) => {{
                let name = CStr::from_bytes_with_nul(concat!($name, "\0").as_bytes())
                    .expect("symbol name has one trailing NUL");
                let pointer = resolve(name);
                if pointer.is_null() {
                    return Err(MissingSymbol {
                        name: $name,
                        requires_rocm_714: true,
                    });
                }
                // SAFETY: guaranteed by this function's contract. The macro
                // names a concrete function-pointer type, which is pointer-sized.
                unsafe { std::mem::transmute::<*const c_void, $ty>(pointer) }
            }};
        }

        Ok(Arc::new(Self {
            _keepalive: keepalive,
            init: symbol!("hsa_init", InitFn),
            shut_down: symbol!("hsa_shut_down", ShutDownFn),
            system_get_info: symbol!("hsa_system_get_info", SystemGetInfoFn),
            status_string: symbol!("hsa_status_string", StatusStringFn),
            iterate_agents: symbol!("hsa_iterate_agents", IterateAgentsFn),
            agent_get_info: symbol!("hsa_agent_get_info", AgentGetInfoFn),
            signal_create: symbol!("hsa_signal_create", SignalCreateFn),
            signal_destroy: symbol!("hsa_signal_destroy", SignalDestroyFn),
            signal_load_scacquire: symbol!("hsa_signal_load_scacquire", SignalLoadScAcquireFn),
            signal_store_relaxed: symbol!("hsa_signal_store_relaxed", SignalStoreRelaxedFn),
            signal_store_screlease: symbol!("hsa_signal_store_screlease", SignalStoreScReleaseFn),
            queue_create: symbol!("hsa_queue_create", QueueCreateFn),
            queue_destroy: symbol!("hsa_queue_destroy", QueueDestroyFn),
            queue_inactivate: symbol!("hsa_queue_inactivate", QueueInactivateFn),
            queue_load_read_index_relaxed: symbol!(
                "hsa_queue_load_read_index_relaxed",
                QueueLoadReadIndexRelaxedFn
            ),
            queue_load_read_index_scacquire: symbol!(
                "hsa_queue_load_read_index_scacquire",
                QueueLoadReadIndexScAcquireFn
            ),
            queue_load_write_index_relaxed: symbol!(
                "hsa_queue_load_write_index_relaxed",
                QueueLoadWriteIndexRelaxedFn
            ),
            queue_add_write_index_relaxed: symbol!(
                "hsa_queue_add_write_index_relaxed",
                QueueAddWriteIndexRelaxedFn
            ),
            profiling_set_profiler_enabled: symbol!(
                "hsa_amd_profiling_set_profiler_enabled",
                ProfilingSetProfilerEnabledFn
            ),
            profiling_get_dispatch_time: symbol!(
                "hsa_amd_profiling_get_dispatch_time",
                ProfilingGetDispatchTimeFn
            ),
            agent_iterate_memory_pools: symbol!(
                "hsa_amd_agent_iterate_memory_pools",
                AgentIterateMemoryPoolsFn
            ),
            memory_pool_get_info: symbol!("hsa_amd_memory_pool_get_info", MemoryPoolGetInfoFn),
            memory_pool_allocate: symbol!("hsa_amd_memory_pool_allocate", MemoryPoolAllocateFn),
            memory_pool_free: symbol!("hsa_amd_memory_pool_free", MemoryPoolFreeFn),
            memory_copy: symbol!("hsa_memory_copy", MemoryCopyFn),
            agents_allow_access: symbol!("hsa_amd_agents_allow_access", AgentsAllowAccessFn),
            code_object_reader_create_from_memory: symbol!(
                "hsa_code_object_reader_create_from_memory",
                CodeObjectReaderCreateFromMemoryFn
            ),
            code_object_reader_destroy: symbol!(
                "hsa_code_object_reader_destroy",
                CodeObjectReaderDestroyFn
            ),
            executable_create_alt: symbol!("hsa_executable_create_alt", ExecutableCreateAltFn),
            executable_destroy: symbol!("hsa_executable_destroy", ExecutableDestroyFn),
            executable_load_agent_code_object: symbol!(
                "hsa_executable_load_agent_code_object",
                ExecutableLoadAgentCodeObjectFn
            ),
            executable_freeze: symbol!("hsa_executable_freeze", ExecutableFreezeFn),
            executable_get_symbol_by_name: symbol!(
                "hsa_executable_get_symbol_by_name",
                ExecutableGetSymbolByNameFn
            ),
            executable_symbol_get_info: symbol!(
                "hsa_executable_symbol_get_info",
                ExecutableSymbolGetInfoFn
            ),
            queue_cu_set_mask: symbol!(
                @rocm714 "hsa_amd_queue_cu_set_mask",
                QueueCuSetMaskFn
            ),
            queue_cu_get_mask: symbol!(
                @rocm714 "hsa_amd_queue_cu_get_mask",
                QueueCuGetMaskFn
            ),
            queue_set_priority: symbol!(
                @rocm714 "hsa_amd_queue_set_priority",
                QueueSetPriorityFn
            ),
            counted_queue_acquire: symbol!(
                @rocm714 "hsa_amd_counted_queue_acquire",
                CountedQueueAcquireFn
            ),
            counted_queue_release: symbol!(
                @rocm714 "hsa_amd_counted_queue_release",
                CountedQueueReleaseFn
            ),
            queue_get_info: symbol!(@rocm714 "hsa_amd_queue_get_info", QueueGetInfoFn),
            signal_wait_all: symbol!(@rocm714 "hsa_amd_signal_wait_all", SignalWaitAllFn),
            signal_wait_any: symbol!(@rocm714 "hsa_amd_signal_wait_any", SignalWaitAnyFn),
            svm_prefetch_async: symbol!(
                @rocm714 "hsa_amd_svm_prefetch_async",
                SvmPrefetchAsyncFn
            ),
            svm_discard_batch_async: symbol!(
                @rocm714 "hsa_amd_svm_discard_batch_async",
                SvmDiscardBatchAsyncFn
            ),
            profiling_convert_tick_to_system_domain: symbol!(
                @rocm714 "hsa_amd_profiling_convert_tick_to_system_domain",
                ProfilingConvertTickToSystemDomainFn
            ),
        }))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MissingSymbol {
    pub name: &'static str,
    /// When true, the symbol is part of the ROCm >= 7.14 / interface-1.26 surface.
    pub requires_rocm_714: bool,
}

impl fmt::Display for MissingSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.requires_rocm_714 {
            write!(
                f,
                "libhsa-runtime64 is missing {} (requires ROCm >= 7.14)",
                self.name
            )
        } else {
            write!(f, "libhsa-runtime64 is missing {}", self.name)
        }
    }
}

impl std::error::Error for MissingSymbol {}

const _: () = {
    assert!(std::mem::size_of::<usize>() == 8);
    assert!(std::mem::size_of::<Agent>() == 8);
    assert!(std::mem::size_of::<Signal>() == 8);
    assert!(std::mem::size_of::<MemoryPool>() == 8);
    assert!(std::mem::size_of::<Queue>() == 40);
    assert!(std::mem::align_of::<Queue>() == 8);
    assert!(std::mem::size_of::<ProfilingDispatchTime>() == 16);
    // ROCm 7.14 / interface 1.26 fixed-width enum stand-ins (C enums → int).
    assert!(std::mem::size_of::<QueuePriority>() == 4);
    assert!(std::mem::align_of::<QueuePriority>() == 4);
    assert!(std::mem::size_of::<QueueInfoAttribute>() == 4);
    assert!(std::mem::align_of::<QueueInfoAttribute>() == 4);
    assert!(std::mem::size_of::<SignalCondition>() == 4);
    assert!(std::mem::size_of::<WaitState>() == 4);
    assert!(std::mem::size_of::<QueueType>() == 4);
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;
    use std::sync::Arc;

    #[test]
    fn missing_rocm714_symbol_names_requirement() {
        let err = MissingSymbol {
            name: "hsa_amd_queue_cu_set_mask",
            requires_rocm_714: true,
        };
        let text = err.to_string();
        assert!(text.contains("hsa_amd_queue_cu_set_mask"), "{text}");
        assert!(text.contains("requires ROCm >= 7.14"), "{text}");
    }

    #[test]
    fn null_resolver_fails_first_symbol_without_714_tag() {
        let keepalive: Arc<dyn Send + Sync> = Arc::new(());
        // SAFETY: resolver always returns null; no function pointers are formed.
        let err = unsafe { Symbols::load(keepalive, |_| ptr::null()) }.unwrap_err();
        assert_eq!(err.name, "hsa_init");
        assert!(!err.requires_rocm_714);
        assert!(!err.to_string().contains("requires ROCm >= 7.14"));
    }

    #[test]
    fn null_after_legacy_symbols_tags_714_requirement() {
        let keepalive: Arc<dyn Send + Sync> = Arc::new(());
        // SAFETY: non-null pointers are never called; only used to pass null-checks.
        let err = unsafe {
            Symbols::load(keepalive, |name| {
                let bytes = name.to_bytes();
                if bytes.starts_with(b"hsa_amd_queue_cu_set_mask")
                    || bytes.starts_with(b"hsa_amd_queue_cu_get_mask")
                    || bytes.starts_with(b"hsa_amd_queue_set_priority")
                    || bytes.starts_with(b"hsa_amd_counted_queue")
                    || bytes.starts_with(b"hsa_amd_queue_get_info")
                    || bytes.starts_with(b"hsa_amd_signal_wait_")
                    || bytes.starts_with(b"hsa_amd_svm_")
                    || bytes.starts_with(b"hsa_amd_profiling_convert_tick")
                {
                    ptr::null()
                } else {
                    // Any non-null stand-in; never invoked.
                    1usize as *const c_void
                }
            })
        }
        .unwrap_err();
        assert_eq!(err.name, "hsa_amd_queue_cu_set_mask");
        assert!(err.requires_rocm_714);
        assert!(err.to_string().contains("requires ROCm >= 7.14"));
    }
}
