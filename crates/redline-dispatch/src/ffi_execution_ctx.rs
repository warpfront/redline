// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! HIP 7.14 green-context / execution-context FFI (CuPartition L2 path).
//!
//! Layouts and entry points are transcribed from
//! `/opt/rocm/core/include/hip/hip_runtime_api.h` (ROCm 7.14). Missing symbols
//! hard-fail with a named error that requires ROCm >= 7.14.
//!
//! Flow: `hipDeviceGetDevResource(Sm)` → split → `hipDevResourceGenerateDesc`
//! → `hipGreenCtxCreate` → `hipExecutionCtxStreamCreate` per worker.

use std::ffi::{CStr, c_int, c_uint, c_ulonglong, c_void};
use std::fmt;
use std::mem::{align_of, offset_of, size_of};
use std::ptr;
use std::sync::Arc;

use libloading::Library;

/// HIP status code (`hipError_t`).
pub type HipError = c_int;

/// Device ordinal (`hipDevice_t` / `typedef int`).
pub type HipDevice = c_int;

/// Opaque stream handle (`hipStream_t`).
pub type HipStream = *mut c_void;

/// Opaque event handle (`hipEvent_t`).
pub type HipEvent = *mut c_void;

pub const HIP_SUCCESS: HipError = 0;

/// `hipStreamDefault` — usable with `hipExecutionCtxStreamCreate`.
pub const HIP_STREAM_DEFAULT: c_uint = 0x00;
/// `hipStreamNonBlocking` — usual choice for partition worker streams.
pub const HIP_STREAM_NON_BLOCKING: c_uint = 0x01;

/// Bytes reserved for workqueue / oversize union arms (`HIP_RESOURCE_ABI_BYTES`).
pub const HIP_RESOURCE_ABI_BYTES: usize = 40;

/// Opaque green / primary execution context (`hipExecutionCtx_t`).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct HipExecutionCtx(pub *mut c_void);

// SAFETY: HIP treats the handle as an opaque pointer token owned by the runtime.
unsafe impl Send for HipExecutionCtx {}
// SAFETY: same as Send — the pointed-to object is runtime-managed, not Rust data.
unsafe impl Sync for HipExecutionCtx {}

/// Opaque resource descriptor (`hipDevResourceDesc_t`).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct HipDevResourceDesc(pub *mut c_void);

// SAFETY: opaque HIP descriptor handle.
unsafe impl Send for HipDevResourceDesc {}
// SAFETY: opaque HIP descriptor handle.
unsafe impl Sync for HipDevResourceDesc {}

/// `hipDevResourceType` — CU/SM partition path uses [`HIP_DEV_RESOURCE_TYPE_SM`].
#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HipDevResourceType {
    Invalid = 0,
    /// CU/SM budget (`hipDevResourceTypeSm`).
    Sm = 1,
    WorkqueueConfig = 1000,
    Workqueue = 10000,
}

pub const HIP_DEV_RESOURCE_TYPE_INVALID: u32 = 0;
pub const HIP_DEV_RESOURCE_TYPE_SM: u32 = 1;
pub const HIP_DEV_RESOURCE_TYPE_WORKQUEUE_CONFIG: u32 = 1000;
pub const HIP_DEV_RESOURCE_TYPE_WORKQUEUE: u32 = 10000;

/// `hipDevSmResourceGroup_flags`.
#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HipDevSmResourceGroupFlags {
    Default = 0,
    Backfill = 0x1,
}

pub const HIP_DEV_SM_RESOURCE_GROUP_DEFAULT: u32 = 0;
pub const HIP_DEV_SM_RESOURCE_GROUP_BACKFILL: u32 = 0x1;

/// `hipDevSmResourceSplitByCount_flags`.
#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HipDevSmResourceSplitByCountFlags {
    IgnoreSmCoscheduling = 0x1,
    MaxPotentialClusterSize = 0x2,
}

pub const HIP_DEV_SM_RESOURCE_SPLIT_IGNORE_SM_COSCHEDULING: u32 = 0x1;
pub const HIP_DEV_SM_RESOURCE_SPLIT_MAX_POTENTIAL_CLUSTER_SIZE: u32 = 0x2;

/// `hipDevWorkqueueConfigScope`.
#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HipDevWorkqueueConfigScope {
    DeviceCtx = 0,
    GreenCtxBalanced = 1,
}

pub const HIP_DEV_WORKQUEUE_CONFIG_SCOPE_DEVICE_CTX: u32 = 0;
pub const HIP_DEV_WORKQUEUE_CONFIG_SCOPE_GREEN_CTX_BALANCED: u32 = 1;

/// `hipDevSmResource` — SM/CU budget and split constraints (16 bytes).
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HipDevSmResource {
    pub sm_count: c_uint,
    pub min_sm_partition_size: c_uint,
    pub sm_coscheduled_alignment: c_uint,
    pub flags: c_uint,
}

/// `hipDevWorkqueueConfigResource` (12 bytes).
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HipDevWorkqueueConfigResource {
    pub device: c_int,
    pub wq_concurrency_limit: c_uint,
    pub sharing_scope: c_uint, // HipDevWorkqueueConfigScope
}

/// `hipDevWorkqueueResource` — opaque reserved blob (40 bytes).
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HipDevWorkqueueResource {
    pub reserved: [u8; HIP_RESOURCE_ABI_BYTES],
}

/// Union arms of `hipDevResource` (40-byte oversize).
#[repr(C)]
#[derive(Clone, Copy)]
pub union HipDevResourcePayload {
    pub sm: HipDevSmResource,
    pub wq_config: HipDevWorkqueueConfigResource,
    pub wq: HipDevWorkqueueResource,
    pub oversize: [u8; HIP_RESOURCE_ABI_BYTES],
}

// SAFETY: all union arms are plain POD bytes / integers.
unsafe impl Send for HipDevResourcePayload {}
// SAFETY: all union arms are plain POD bytes / integers.
unsafe impl Sync for HipDevResourcePayload {}

impl fmt::Debug for HipDevResourcePayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // SAFETY: reading the oversize byte view is always valid for a POD union.
        let bytes = unsafe { self.oversize };
        f.debug_struct("HipDevResourcePayload")
            .field("oversize", &bytes)
            .finish()
    }
}

/// `hipDevResource` — 144 bytes on amd64 (type@0, pad@4, union@96, next@136).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct HipDevResource {
    pub type_: c_uint, // HipDevResourceType
    pub _internal_padding: [u8; 92],
    pub payload: HipDevResourcePayload,
    pub next_resource: *mut HipDevResource,
}

impl Default for HipDevResource {
    fn default() -> Self {
        Self {
            type_: HIP_DEV_RESOURCE_TYPE_INVALID,
            _internal_padding: [0; 92],
            payload: HipDevResourcePayload {
                oversize: [0; HIP_RESOURCE_ABI_BYTES],
            },
            next_resource: ptr::null_mut(),
        }
    }
}

/// `hipDevSmResourceGroupParams` — per-group split request (64 bytes).
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HipDevSmResourceGroupParams {
    pub sm_count: c_uint,
    pub coscheduled_sm_count: c_uint,
    pub preferred_coscheduled_sm_count: c_uint,
    pub flags: c_uint, // HipDevSmResourceGroupFlags
    pub reserved: [c_uint; 12],
}

impl Default for HipDevSmResourceGroupParams {
    fn default() -> Self {
        Self {
            sm_count: 0,
            coscheduled_sm_count: 0,
            preferred_coscheduled_sm_count: 0,
            flags: HIP_DEV_SM_RESOURCE_GROUP_DEFAULT,
            reserved: [0; 12],
        }
    }
}

// --- function pointer types -------------------------------------------------

pub type HipDeviceGetDevResourceFn = unsafe extern "C" fn(
    HipDevice,
    *mut HipDevResource,
    c_uint, // HipDevResourceType
) -> HipError;

pub type HipDevSmResourceSplitByCountFn = unsafe extern "C" fn(
    *mut HipDevResource,
    *mut c_uint,
    *const HipDevResource,
    *mut HipDevResource,
    c_uint,
    c_uint,
) -> HipError;

pub type HipDevSmResourceSplitFn = unsafe extern "C" fn(
    *mut HipDevResource,
    c_uint,
    *const HipDevResource,
    *mut HipDevResource,
    c_uint,
    *mut HipDevSmResourceGroupParams,
) -> HipError;

pub type HipDevResourceGenerateDescFn =
    unsafe extern "C" fn(*mut HipDevResourceDesc, *mut HipDevResource, c_uint) -> HipError;

pub type HipGreenCtxCreateFn =
    unsafe extern "C" fn(*mut HipExecutionCtx, HipDevResourceDesc, c_int, c_uint) -> HipError;

pub type HipExecutionCtxDestroyFn = unsafe extern "C" fn(HipExecutionCtx) -> HipError;

pub type HipDeviceGetExecutionCtxFn = unsafe extern "C" fn(*mut HipExecutionCtx, c_int) -> HipError;

pub type HipExecutionCtxStreamCreateFn =
    unsafe extern "C" fn(*mut HipStream, HipExecutionCtx, c_uint, c_int) -> HipError;

pub type HipExecutionCtxGetDevResourceFn = unsafe extern "C" fn(
    HipExecutionCtx,
    *mut HipDevResource,
    c_uint, // HipDevResourceType
) -> HipError;

pub type HipExecutionCtxGetDeviceFn = unsafe extern "C" fn(*mut c_int, HipExecutionCtx) -> HipError;

pub type HipExecutionCtxGetIdFn =
    unsafe extern "C" fn(HipExecutionCtx, *mut c_ulonglong) -> HipError;

pub type HipStreamGetDevResourceFn = unsafe extern "C" fn(
    HipStream,
    *mut HipDevResource,
    c_uint, // HipDevResourceType
) -> HipError;

pub type HipExecutionCtxRecordEventFn = unsafe extern "C" fn(HipExecutionCtx, HipEvent) -> HipError;

pub type HipExecutionCtxSynchronizeFn = unsafe extern "C" fn(HipExecutionCtx) -> HipError;

pub type HipExecutionCtxWaitEventFn = unsafe extern "C" fn(HipExecutionCtx, HipEvent) -> HipError;

/// Dynamically resolved HIP 7.14 execution-context entry points.
///
/// `_keepalive` retains the `libamdhip64` mapping for every stored function
/// pointer.
pub struct ExecutionCtxSymbols {
    _keepalive: Arc<dyn Send + Sync>,
    pub device_get_dev_resource: HipDeviceGetDevResourceFn,
    pub dev_sm_resource_split_by_count: HipDevSmResourceSplitByCountFn,
    pub dev_sm_resource_split: HipDevSmResourceSplitFn,
    pub dev_resource_generate_desc: HipDevResourceGenerateDescFn,
    pub green_ctx_create: HipGreenCtxCreateFn,
    pub execution_ctx_destroy: HipExecutionCtxDestroyFn,
    pub device_get_execution_ctx: HipDeviceGetExecutionCtxFn,
    pub execution_ctx_stream_create: HipExecutionCtxStreamCreateFn,
    pub execution_ctx_get_dev_resource: HipExecutionCtxGetDevResourceFn,
    pub execution_ctx_get_device: HipExecutionCtxGetDeviceFn,
    pub execution_ctx_get_id: HipExecutionCtxGetIdFn,
    pub stream_get_dev_resource: HipStreamGetDevResourceFn,
    pub execution_ctx_record_event: HipExecutionCtxRecordEventFn,
    pub execution_ctx_synchronize: HipExecutionCtxSynchronizeFn,
    pub execution_ctx_wait_event: HipExecutionCtxWaitEventFn,
}

impl fmt::Debug for ExecutionCtxSymbols {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExecutionCtxSymbols")
            .finish_non_exhaustive()
    }
}

impl ExecutionCtxSymbols {
    /// Resolve the public symbols used by the CuPartition path.
    ///
    /// # Safety
    ///
    /// Each non-null pointer returned by `resolve` must name the C function
    /// requested by the supplied symbol name, with the exact HIP 7.14 ABI.
    /// `keepalive` must keep the containing shared object mapped for its own
    /// lifetime.
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
                    return Err(MissingSymbol($name));
                }
                // SAFETY: guaranteed by this function's contract.
                unsafe { std::mem::transmute::<*const c_void, $ty>(pointer) }
            }};
        }

        Ok(Arc::new(Self {
            _keepalive: keepalive,
            device_get_dev_resource: symbol!("hipDeviceGetDevResource", HipDeviceGetDevResourceFn),
            dev_sm_resource_split_by_count: symbol!(
                "hipDevSmResourceSplitByCount",
                HipDevSmResourceSplitByCountFn
            ),
            dev_sm_resource_split: symbol!("hipDevSmResourceSplit", HipDevSmResourceSplitFn),
            dev_resource_generate_desc: symbol!(
                "hipDevResourceGenerateDesc",
                HipDevResourceGenerateDescFn
            ),
            green_ctx_create: symbol!("hipGreenCtxCreate", HipGreenCtxCreateFn),
            execution_ctx_destroy: symbol!("hipExecutionCtxDestroy", HipExecutionCtxDestroyFn),
            device_get_execution_ctx: symbol!(
                "hipDeviceGetExecutionCtx",
                HipDeviceGetExecutionCtxFn
            ),
            execution_ctx_stream_create: symbol!(
                "hipExecutionCtxStreamCreate",
                HipExecutionCtxStreamCreateFn
            ),
            execution_ctx_get_dev_resource: symbol!(
                "hipExecutionCtxGetDevResource",
                HipExecutionCtxGetDevResourceFn
            ),
            execution_ctx_get_device: symbol!(
                "hipExecutionCtxGetDevice",
                HipExecutionCtxGetDeviceFn
            ),
            execution_ctx_get_id: symbol!("hipExecutionCtxGetId", HipExecutionCtxGetIdFn),
            stream_get_dev_resource: symbol!("hipStreamGetDevResource", HipStreamGetDevResourceFn),
            execution_ctx_record_event: symbol!(
                "hipExecutionCtxRecordEvent",
                HipExecutionCtxRecordEventFn
            ),
            execution_ctx_synchronize: symbol!(
                "hipExecutionCtxSynchronize",
                HipExecutionCtxSynchronizeFn
            ),
            execution_ctx_wait_event: symbol!(
                "hipExecutionCtxWaitEvent",
                HipExecutionCtxWaitEventFn
            ),
        }))
    }
}

/// A required HIP 7.14 execution-context symbol was absent from `libamdhip64`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MissingSymbol(pub &'static str);

impl fmt::Display for MissingSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "libamdhip64 is missing {} (requires ROCm >= 7.14)",
            self.0
        )
    }
}

impl std::error::Error for MissingSymbol {}

/// Failure opening `libamdhip64` or resolving a 7.14 execution-context symbol.
#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("failed to load libamdhip64 (tried {candidates}): {detail}")]
    Library { candidates: String, detail: String },
    #[error(transparent)]
    Symbol(#[from] MissingSymbol),
}

const HIP_LIBRARY_CANDIDATES: &[&str] = &[
    "libamdhip64.so",
    "libamdhip64.so.7",
    "/opt/rocm/core/lib/libamdhip64.so",
    "/opt/rocm/lib/libamdhip64.so",
];

/// Load execution-context symbols from the installed HIP runtime.
pub fn load_execution_ctx_symbols() -> Result<Arc<ExecutionCtxSymbols>, LoadError> {
    let mut failures = Vec::new();
    let library = HIP_LIBRARY_CANDIDATES
        .iter()
        .find_map(|candidate| {
            // SAFETY: loading the installed HIP runtime is the purpose of this
            // module. `ExecutionCtxSymbols` retains the successful mapping.
            match unsafe { Library::new(candidate) } {
                Ok(library) => Some(Arc::new(library)),
                Err(error) => {
                    failures.push(format!("{candidate}: {error}"));
                    None
                }
            }
        })
        .ok_or_else(|| LoadError::Library {
            candidates: HIP_LIBRARY_CANDIDATES.join(", "),
            detail: failures.join("; "),
        })?;
    let keepalive: Arc<dyn Send + Sync> = library.clone();
    // SAFETY: every lookup is resolved from the public HIP library retained by
    // `keepalive`; `ExecutionCtxSymbols::load` assigns each name its header ABI.
    unsafe {
        ExecutionCtxSymbols::load(keepalive, |name| {
            library
                .get::<*const c_void>(name.to_bytes_with_nul())
                .map_or(ptr::null(), |symbol| *symbol)
        })
        .map_err(LoadError::Symbol)
    }
}

const _: () = {
    assert!(size_of::<usize>() == 8);
    assert!(size_of::<HipExecutionCtx>() == 8);
    assert!(align_of::<HipExecutionCtx>() == 8);
    assert!(size_of::<HipDevResourceDesc>() == 8);
    assert!(align_of::<HipDevResourceDesc>() == 8);

    assert!(size_of::<HipDevResourceType>() == 4);
    assert!(size_of::<HipDevWorkqueueConfigScope>() == 4);

    assert!(size_of::<HipDevSmResource>() == 16);
    assert!(align_of::<HipDevSmResource>() == 4);
    assert!(offset_of!(HipDevSmResource, sm_count) == 0);
    assert!(offset_of!(HipDevSmResource, min_sm_partition_size) == 4);
    assert!(offset_of!(HipDevSmResource, sm_coscheduled_alignment) == 8);
    assert!(offset_of!(HipDevSmResource, flags) == 12);

    assert!(size_of::<HipDevWorkqueueConfigResource>() == 12);
    assert!(align_of::<HipDevWorkqueueConfigResource>() == 4);
    assert!(offset_of!(HipDevWorkqueueConfigResource, device) == 0);
    assert!(offset_of!(HipDevWorkqueueConfigResource, wq_concurrency_limit) == 4);
    assert!(offset_of!(HipDevWorkqueueConfigResource, sharing_scope) == 8);

    assert!(size_of::<HipDevWorkqueueResource>() == 40);
    assert!(align_of::<HipDevWorkqueueResource>() == 1);

    assert!(size_of::<HipDevResourcePayload>() == 40);

    assert!(size_of::<HipDevResource>() == 144);
    assert!(align_of::<HipDevResource>() == 8);
    assert!(offset_of!(HipDevResource, type_) == 0);
    assert!(offset_of!(HipDevResource, _internal_padding) == 4);
    assert!(offset_of!(HipDevResource, payload) == 96);
    assert!(offset_of!(HipDevResource, next_resource) == 136);

    assert!(size_of::<HipDevSmResourceGroupParams>() == 64);
    assert!(align_of::<HipDevSmResourceGroupParams>() == 4);
    assert!(offset_of!(HipDevSmResourceGroupParams, sm_count) == 0);
    assert!(offset_of!(HipDevSmResourceGroupParams, coscheduled_sm_count) == 4);
    assert!(offset_of!(HipDevSmResourceGroupParams, preferred_coscheduled_sm_count) == 8);
    assert!(offset_of!(HipDevSmResourceGroupParams, flags) == 12);
    assert!(offset_of!(HipDevSmResourceGroupParams, reserved) == 16);
};

#[cfg(test)]
mod selfcheck {
    use super::*;

    #[test]
    fn missing_symbol_names_rocm_requirement() {
        let err = MissingSymbol("hipGreenCtxCreate");
        let text = err.to_string();
        assert!(text.contains("hipGreenCtxCreate"), "{text}");
        assert!(text.contains("requires ROCm >= 7.14"), "{text}");
    }

    #[test]
    fn load_error_from_missing_symbol_displays() {
        let err = LoadError::from(MissingSymbol("hipExecutionCtxDestroy"));
        let text = err.to_string();
        assert!(text.contains("hipExecutionCtxDestroy"), "{text}");
        assert!(text.contains("requires ROCm >= 7.14"), "{text}");
    }
}
