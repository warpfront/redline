// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! HIP 7.14 library load / global lookup FFI.
//!
//! Layouts and entry points are transcribed from
//! `/opt/rocm/core/include/hip/hip_runtime_api.h` and
//! `/opt/rocm/core/include/hip/linker_types.h` (ROCm 7.14). Missing symbols
//! hard-fail with a named error that requires ROCm >= 7.14.
//!
//! Pipeline: `hipLibraryLoadFromFile` | `hipLibraryLoadData` →
//! `hipLibraryGetGlobal` / `hipLibraryGetManaged` (and optional kernel
//! enumerate APIs) → `hipLibraryUnload`.

use std::ffi::{CStr, c_char, c_int, c_uint, c_void};
use std::fmt;
use std::mem::{align_of, size_of};
use std::ptr;
use std::sync::Arc;

use libloading::Library;

/// HIP status code (`hipError_t`).
pub type HipError = c_int;

pub const HIP_SUCCESS: HipError = 0;

/// Opaque code-object library handle (`hipLibrary_t` / `ihipLibrary_t*`).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct HipLibrary(pub *mut c_void);

// SAFETY: HIP treats the handle as an opaque pointer token owned by the runtime.
unsafe impl Send for HipLibrary {}
// SAFETY: same as Send — the pointed-to object is runtime-managed, not Rust data.
unsafe impl Sync for HipLibrary {}

/// Opaque kernel object from a library (`hipKernel_t` / `ihipKernel_t*`).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct HipKernel(pub *mut c_void);

// SAFETY: opaque HIP kernel handle.
unsafe impl Send for HipKernel {}
// SAFETY: opaque HIP kernel handle.
unsafe impl Sync for HipKernel {}

/// `hipLibraryOption` from `linker_types.h`.
#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HipLibraryOption {
    /// `hipLibraryHostUniversalFunctionAndDataTable`
    HostUniversalFunctionAndDataTable = 0,
    /// `hipLibraryBinaryIsPreserved`
    BinaryIsPreserved = 1,
}

pub const HIP_LIBRARY_HOST_UNIVERSAL_FUNCTION_AND_DATA_TABLE: u32 = 0;
pub const HIP_LIBRARY_BINARY_IS_PRESERVED: u32 = 1;

/// `hipJitOption` parameter type for library load (CUDA-oriented; size 4).
///
/// Full enumerators live in `linker_types.h`; load paths accept a pointer and
/// count. Callers that do not use JIT options pass null / zero.
pub type HipJitOption = c_int;

// --- function pointer types -------------------------------------------------

/// `hipLibraryLoadData`
pub type HipLibraryLoadDataFn = unsafe extern "C" fn(
    *mut HipLibrary,
    *const c_void,
    *mut HipJitOption,
    *mut *mut c_void,
    c_uint,
    *mut HipLibraryOption,
    *mut *mut c_void,
    c_uint,
) -> HipError;

/// `hipLibraryLoadFromFile`
pub type HipLibraryLoadFromFileFn = unsafe extern "C" fn(
    *mut HipLibrary,
    *const c_char,
    *mut HipJitOption,
    *mut *mut c_void,
    c_uint,
    *mut HipLibraryOption,
    *mut *mut c_void,
    c_uint,
) -> HipError;

/// `hipLibraryUnload`
pub type HipLibraryUnloadFn = unsafe extern "C" fn(HipLibrary) -> HipError;

/// `hipLibraryGetKernel`
pub type HipLibraryGetKernelFn =
    unsafe extern "C" fn(*mut HipKernel, HipLibrary, *const c_char) -> HipError;

/// `hipLibraryGetKernelCount`
pub type HipLibraryGetKernelCountFn = unsafe extern "C" fn(*mut c_uint, HipLibrary) -> HipError;

/// `hipLibraryGetGlobal` — `__device__` global lookup.
pub type HipLibraryGetGlobalFn =
    unsafe extern "C" fn(*mut *mut c_void, *mut usize, HipLibrary, *const c_char) -> HipError;

/// `hipLibraryGetManaged` — `__managed__` variable lookup.
pub type HipLibraryGetManagedFn =
    unsafe extern "C" fn(*mut *mut c_void, *mut usize, HipLibrary, *const c_char) -> HipError;

/// `hipLibraryEnumerateKernels`
pub type HipLibraryEnumerateKernelsFn =
    unsafe extern "C" fn(*mut HipKernel, c_uint, HipLibrary) -> HipError;

/// `hipKernelGetLibrary`
pub type HipKernelGetLibraryFn = unsafe extern "C" fn(*mut HipLibrary, HipKernel) -> HipError;

/// Dynamically resolved HIP 7.14 library entry points.
///
/// `_keepalive` retains the `libamdhip64` mapping for every stored function
/// pointer.
pub struct LibrarySymbols {
    _keepalive: Arc<dyn Send + Sync>,
    pub library_load_data: HipLibraryLoadDataFn,
    pub library_load_from_file: HipLibraryLoadFromFileFn,
    pub library_unload: HipLibraryUnloadFn,
    pub library_get_kernel: HipLibraryGetKernelFn,
    pub library_get_kernel_count: HipLibraryGetKernelCountFn,
    pub library_get_global: HipLibraryGetGlobalFn,
    pub library_get_managed: HipLibraryGetManagedFn,
    pub library_enumerate_kernels: HipLibraryEnumerateKernelsFn,
    pub kernel_get_library: HipKernelGetLibraryFn,
}

impl fmt::Debug for LibrarySymbols {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LibrarySymbols").finish_non_exhaustive()
    }
}

impl LibrarySymbols {
    /// Resolve the public library symbols used for code-object load and globals.
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
            library_load_data: symbol!("hipLibraryLoadData", HipLibraryLoadDataFn),
            library_load_from_file: symbol!("hipLibraryLoadFromFile", HipLibraryLoadFromFileFn),
            library_unload: symbol!("hipLibraryUnload", HipLibraryUnloadFn),
            library_get_kernel: symbol!("hipLibraryGetKernel", HipLibraryGetKernelFn),
            library_get_kernel_count: symbol!(
                "hipLibraryGetKernelCount",
                HipLibraryGetKernelCountFn
            ),
            library_get_global: symbol!("hipLibraryGetGlobal", HipLibraryGetGlobalFn),
            library_get_managed: symbol!("hipLibraryGetManaged", HipLibraryGetManagedFn),
            library_enumerate_kernels: symbol!(
                "hipLibraryEnumerateKernels",
                HipLibraryEnumerateKernelsFn
            ),
            kernel_get_library: symbol!("hipKernelGetLibrary", HipKernelGetLibraryFn),
        }))
    }
}

/// A required HIP 7.14 library symbol was absent from `libamdhip64`.
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

/// Failure opening `libamdhip64` or resolving a 7.14 library symbol.
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

/// Load library-management symbols from the installed HIP runtime.
pub fn load_library_symbols() -> Result<Arc<LibrarySymbols>, LoadError> {
    let mut failures = Vec::new();
    let library = HIP_LIBRARY_CANDIDATES
        .iter()
        .find_map(|candidate| {
            // SAFETY: loading the installed HIP runtime is the purpose of this
            // module. `LibrarySymbols` retains the successful mapping.
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
    // `keepalive`; `LibrarySymbols::load` assigns each name its header ABI.
    unsafe {
        LibrarySymbols::load(keepalive, |name| {
            library
                .get::<*const c_void>(name.to_bytes_with_nul())
                .map_or(ptr::null(), |symbol| *symbol)
        })
        .map_err(LoadError::Symbol)
    }
}

const _: () = {
    assert!(size_of::<usize>() == 8);
    assert!(size_of::<HipLibrary>() == 8);
    assert!(align_of::<HipLibrary>() == 8);
    assert!(size_of::<HipKernel>() == 8);
    assert!(align_of::<HipKernel>() == 8);
    assert!(size_of::<HipLibraryOption>() == 4);
    assert!(align_of::<HipLibraryOption>() == 4);
    assert!(size_of::<HipJitOption>() == 4);
    assert!(size_of::<HipError>() == 4);
};

#[cfg(test)]
mod selfcheck {
    use super::*;

    #[test]
    fn missing_symbol_names_rocm_requirement() {
        let err = MissingSymbol("hipLibraryGetGlobal");
        let text = err.to_string();
        assert!(text.contains("hipLibraryGetGlobal"), "{text}");
        assert!(text.contains("requires ROCm >= 7.14"), "{text}");
    }

    #[test]
    fn load_error_from_missing_symbol_displays() {
        let err = LoadError::from(MissingSymbol("hipLibraryLoadFromFile"));
        let text = err.to_string();
        assert!(text.contains("hipLibraryLoadFromFile"), "{text}");
        assert!(text.contains("requires ROCm >= 7.14"), "{text}");
    }

    #[test]
    fn missing_resolve_reports_named_symbol() {
        let keepalive: Arc<dyn Send + Sync> = Arc::new(());
        // SAFETY: resolve always returns null; load must not transmute.
        let err = unsafe { LibrarySymbols::load(keepalive, |_| ptr::null()) }.unwrap_err();
        assert_eq!(err, MissingSymbol("hipLibraryLoadData"));
        let text = err.to_string();
        assert!(text.contains("requires ROCm >= 7.14"), "{text}");
    }
}
