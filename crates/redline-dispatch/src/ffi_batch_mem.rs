// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! HIP 7.14 batch memory FFI (prefetch / discard / fused discard+prefetch).
//!
//! Layouts and entry points are transcribed from the MIT-licensed ROCm 7.14
//! public headers:
//! - `/opt/rocm/core/include/hip/hip_runtime_api.h` (batch APIs)
//! - `/opt/rocm/core/include/hip/driver_types.h` (`hipMemLocation`,
//!   `hipMemLocationType`, `hipDeviceptr_t`)
//!
//! There is **no** `hipDrvMemPrefetchBatchAsync` in HIP 7.14 — only the runtime
//! prefetch batch entry point exists. Driver-style twins exist solely for
//! discard and discard+prefetch. `flags` parameters are reserved and must be
//! zero.
//!
//! Missing symbols hard-fail with a named error that requires ROCm >= 7.14.

use std::ffi::{CStr, c_int, c_void};
use std::fmt;
use std::mem::{align_of, offset_of, size_of};
use std::ptr;
use std::sync::Arc;

use libloading::Library;

/// HIP status code (`hipError_t`).
pub type HipError = c_int;

/// Opaque stream handle (`hipStream_t`).
pub type HipStream = *mut c_void;

/// Driver device pointer (`hipDeviceptr_t` → `void*`).
pub type HipDeviceptr = *mut c_void;

pub const HIP_SUCCESS: HipError = 0;

/// Reserved batch-mem flags value — headers require zero today.
pub const HIP_MEM_BATCH_FLAGS_NONE: u64 = 0;

/// `hipMemLocationType` from `driver_types.h`.
///
/// `Invalid` and the C enumerator `hipMemLocationTypeNone` share value 0.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HipMemLocationType {
    Invalid = 0,
    Device = 1,
    Host = 2,
    HostNuma = 3,
    HostNumaCurrent = 4,
}

/// Alias for the C enumerator `hipMemLocationTypeNone` (value 0).
pub const HIP_MEM_LOCATION_TYPE_NONE: HipMemLocationType = HipMemLocationType::Invalid;
pub const HIP_MEM_LOCATION_TYPE_INVALID: HipMemLocationType = HipMemLocationType::Invalid;
pub const HIP_MEM_LOCATION_TYPE_DEVICE: HipMemLocationType = HipMemLocationType::Device;
pub const HIP_MEM_LOCATION_TYPE_HOST: HipMemLocationType = HipMemLocationType::Host;
pub const HIP_MEM_LOCATION_TYPE_HOST_NUMA: HipMemLocationType = HipMemLocationType::HostNuma;
pub const HIP_MEM_LOCATION_TYPE_HOST_NUMA_CURRENT: HipMemLocationType =
    HipMemLocationType::HostNumaCurrent;

/// `hipMemLocation` — prefetch destination descriptor.
///
/// Device: `type_ = Device`, `id =` HIP device ordinal.
/// Host: `type_ = Host`, `id` ignored.
/// Host NUMA: `type_ = HostNuma`, `id =` host NUMA node id.
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct HipMemLocation {
    pub type_: HipMemLocationType,
    pub id: c_int,
}

impl HipMemLocation {
    #[inline]
    pub const fn device(device_id: c_int) -> Self {
        Self {
            type_: HipMemLocationType::Device,
            id: device_id,
        }
    }

    #[inline]
    pub const fn host() -> Self {
        Self {
            type_: HipMemLocationType::Host,
            id: 0,
        }
    }

    #[inline]
    pub const fn host_numa(node_id: c_int) -> Self {
        Self {
            type_: HipMemLocationType::HostNuma,
            id: node_id,
        }
    }
}

// --- function pointer types -------------------------------------------------

/// `hipMemPrefetchBatchAsync`
pub type HipMemPrefetchBatchAsyncFn = unsafe extern "C" fn(
    dev_ptrs: *mut *mut c_void,
    sizes: *mut usize,
    count: usize,
    prefetch_locs: *mut HipMemLocation,
    prefetch_loc_idxs: *mut usize,
    num_prefetch_locs: usize,
    flags: u64,
    stream: HipStream,
) -> HipError;

/// `hipMemDiscardBatchAsync`
pub type HipMemDiscardBatchAsyncFn = unsafe extern "C" fn(
    dev_ptrs: *mut *mut c_void,
    sizes: *mut usize,
    count: usize,
    flags: u64,
    stream: HipStream,
) -> HipError;

/// `hipDrvMemDiscardBatchAsync`
pub type HipDrvMemDiscardBatchAsyncFn = unsafe extern "C" fn(
    dptrs: *mut HipDeviceptr,
    sizes: *mut usize,
    count: usize,
    flags: u64,
    stream: HipStream,
) -> HipError;

/// `hipMemDiscardAndPrefetchBatchAsync`
pub type HipMemDiscardAndPrefetchBatchAsyncFn = unsafe extern "C" fn(
    dptrs: *mut *mut c_void,
    sizes: *mut usize,
    count: usize,
    prefetch_locs: *mut HipMemLocation,
    prefetch_loc_idxs: *mut usize,
    num_prefetch_locs: usize,
    flags: u64,
    stream: HipStream,
) -> HipError;

/// `hipDrvMemDiscardAndPrefetchBatchAsync`
pub type HipDrvMemDiscardAndPrefetchBatchAsyncFn = unsafe extern "C" fn(
    dptrs: *mut HipDeviceptr,
    sizes: *mut usize,
    count: usize,
    prefetch_locs: *mut HipMemLocation,
    prefetch_loc_idxs: *mut usize,
    num_prefetch_locs: usize,
    flags: u64,
    stream: HipStream,
) -> HipError;

/// Dynamically resolved HIP 7.14 batch-memory entry points.
///
/// `_keepalive` retains the `libamdhip64` mapping for every stored function
/// pointer. There is no `hipDrvMemPrefetchBatchAsync` symbol in this set.
pub struct BatchMemSymbols {
    _keepalive: Arc<dyn Send + Sync>,
    pub mem_prefetch_batch_async: HipMemPrefetchBatchAsyncFn,
    pub mem_discard_batch_async: HipMemDiscardBatchAsyncFn,
    pub drv_mem_discard_batch_async: HipDrvMemDiscardBatchAsyncFn,
    pub mem_discard_and_prefetch_batch_async: HipMemDiscardAndPrefetchBatchAsyncFn,
    pub drv_mem_discard_and_prefetch_batch_async: HipDrvMemDiscardAndPrefetchBatchAsyncFn,
}

impl fmt::Debug for BatchMemSymbols {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BatchMemSymbols").finish_non_exhaustive()
    }
}

impl BatchMemSymbols {
    /// Resolve the public batch-mem symbols used by this module.
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
            mem_prefetch_batch_async: symbol!(
                "hipMemPrefetchBatchAsync",
                HipMemPrefetchBatchAsyncFn
            ),
            mem_discard_batch_async: symbol!("hipMemDiscardBatchAsync", HipMemDiscardBatchAsyncFn),
            drv_mem_discard_batch_async: symbol!(
                "hipDrvMemDiscardBatchAsync",
                HipDrvMemDiscardBatchAsyncFn
            ),
            mem_discard_and_prefetch_batch_async: symbol!(
                "hipMemDiscardAndPrefetchBatchAsync",
                HipMemDiscardAndPrefetchBatchAsyncFn
            ),
            drv_mem_discard_and_prefetch_batch_async: symbol!(
                "hipDrvMemDiscardAndPrefetchBatchAsync",
                HipDrvMemDiscardAndPrefetchBatchAsyncFn
            ),
        }))
    }
}

/// A required HIP 7.14 batch-mem symbol was absent from `libamdhip64`.
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

/// Failure opening `libamdhip64` or resolving a 7.14 batch-mem symbol.
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

/// Load batch-memory symbols from the installed HIP runtime.
pub fn load_batch_mem_symbols() -> Result<Arc<BatchMemSymbols>, LoadError> {
    let mut failures = Vec::new();
    let library = HIP_LIBRARY_CANDIDATES
        .iter()
        .find_map(|candidate| {
            // SAFETY: loading the installed HIP runtime is the purpose of this
            // module. `BatchMemSymbols` retains the successful mapping.
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
    // `keepalive`; `BatchMemSymbols::load` assigns each name its header ABI.
    unsafe {
        BatchMemSymbols::load(keepalive, |name| {
            library
                .get::<*const c_void>(name.to_bytes_with_nul())
                .map_or(ptr::null(), |symbol| *symbol)
        })
        .map_err(LoadError::Symbol)
    }
}

const _: () = {
    assert!(size_of::<usize>() == 8);
    assert!(size_of::<HipDeviceptr>() == 8);
    assert!(size_of::<HipStream>() == 8);
    assert!(size_of::<HipMemLocationType>() == 4);
    assert!(align_of::<HipMemLocationType>() == 4);
    assert!(size_of::<HipMemLocation>() == 8);
    assert!(align_of::<HipMemLocation>() == 4);
    assert!(offset_of!(HipMemLocation, type_) == 0);
    assert!(offset_of!(HipMemLocation, id) == 4);
    assert!(HipMemLocationType::Invalid as i32 == 0);
    assert!(HipMemLocationType::Device as i32 == 1);
    assert!(HipMemLocationType::Host as i32 == 2);
    assert!(HipMemLocationType::HostNuma as i32 == 3);
    assert!(HipMemLocationType::HostNumaCurrent as i32 == 4);
};

#[cfg(test)]
mod selfcheck {
    use super::*;

    #[test]
    fn missing_symbol_names_rocm_requirement() {
        let err = MissingSymbol("hipMemPrefetchBatchAsync");
        let text = err.to_string();
        assert!(text.contains("hipMemPrefetchBatchAsync"), "{text}");
        assert!(text.contains("requires ROCm >= 7.14"), "{text}");
    }

    #[test]
    fn load_error_from_missing_symbol_displays() {
        let err = LoadError::from(MissingSymbol("hipMemDiscardBatchAsync"));
        let text = err.to_string();
        assert!(text.contains("hipMemDiscardBatchAsync"), "{text}");
        assert!(text.contains("requires ROCm >= 7.14"), "{text}");
    }

    #[test]
    fn load_fails_when_resolver_returns_null() {
        let keepalive: Arc<dyn Send + Sync> = Arc::new(());
        // SAFETY: resolver always returns null — exercises MissingSymbol only.
        let err = unsafe { BatchMemSymbols::load(keepalive, |_| ptr::null()) }
            .expect_err("null resolver must fail");
        assert_eq!(err, MissingSymbol("hipMemPrefetchBatchAsync"));
        let text = err.to_string();
        assert!(text.contains("requires ROCm >= 7.14"), "{text}");
    }

    #[test]
    fn mem_location_helpers() {
        let d = HipMemLocation::device(1);
        assert_eq!(d.type_, HipMemLocationType::Device);
        assert_eq!(d.id, 1);
        let h = HipMemLocation::host();
        assert_eq!(h.type_, HipMemLocationType::Host);
        let n = HipMemLocation::host_numa(2);
        assert_eq!(n.type_, HipMemLocationType::HostNuma);
        assert_eq!(n.id, 2);
        assert_eq!(HIP_MEM_LOCATION_TYPE_NONE, HipMemLocationType::Invalid);
    }
}
