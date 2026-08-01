// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! Minimal HIP ABI declarations used by the interposer.
//!
//! The field order of [`hipKernelNodeParams`] follows the ROCm 7.14 public
//! `hip_runtime_api.h`.  It is deliberately not the CUDA order: HIP places
//! `blockDim` and `extra` before `func`.

#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use std::ffi::c_void;

pub type hipError_t = i32;
pub type hipGraph_t = *mut c_void;
pub type hipGraphNode_t = *mut c_void;
pub type hipGraphExec_t = *mut c_void;
pub type hipStream_t = *mut c_void;
pub type hipFunction_t = *mut c_void;
pub type hipModule_t = *mut c_void;

pub const hipSuccess: hipError_t = 0;
pub const hipErrorInvalidValue: hipError_t = 1;
/// `hipErrorOutOfMemory = 2` per `hip_runtime_api.h:293`, aliased there as
/// `hipErrorMemoryAllocation`. Returned when a translation buffer cannot be
/// allocated, so allocation failure crosses the ABI as an error rather than
/// aborting the process inside the interposer.
pub const hipErrorOutOfMemory: hipError_t = 2;
pub const hipErrorNotInitialized: hipError_t = 3;
pub const hipErrorInvalidImage: hipError_t = 200;
pub const hipErrorInvalidHandle: hipError_t = 400;
pub const hipErrorIllegalState: hipError_t = 401;
pub const hipErrorLaunchFailure: hipError_t = 719;
pub const hipErrorNotSupported: hipError_t = 801;
pub const hipErrorGraphExecUpdateFailure: hipError_t = 910;
pub const hipErrorStreamCaptureInvalidated: hipError_t = 901;
pub const hipErrorStreamCaptureUnmatched: hipError_t = 903;
pub const hipErrorUnknown: hipError_t = 999;

pub const hipStreamCaptureStatusNone: i32 = 0;
pub const hipStreamCaptureStatusActive: i32 = 1;
pub const hipGraphExecUpdateErrorNotSupported: i32 = 6;

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct dim3 {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

impl dim3 {
    pub const fn new(x: u32, y: u32, z: u32) -> Self {
        Self { x, y, z }
    }
}

/// ROCm HIP's public `hipKernelNodeParams` layout.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct hipKernelNodeParams {
    pub blockDim: dim3,
    pub extra: *mut *mut c_void,
    pub func: *mut c_void,
    pub gridDim: dim3,
    pub kernelParams: *mut *mut c_void,
    pub sharedMemBytes: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kernel_node_params_matches_rocm_714_layout() {
        assert_eq!(std::mem::size_of::<dim3>(), 12);
        assert_eq!(std::mem::size_of::<hipKernelNodeParams>(), 64);
        assert_eq!(std::mem::offset_of!(hipKernelNodeParams, blockDim), 0);
        assert_eq!(std::mem::offset_of!(hipKernelNodeParams, extra), 16);
        assert_eq!(std::mem::offset_of!(hipKernelNodeParams, func), 24);
        assert_eq!(std::mem::offset_of!(hipKernelNodeParams, gridDim), 32);
        assert_eq!(std::mem::offset_of!(hipKernelNodeParams, kernelParams), 48);
        assert_eq!(
            std::mem::offset_of!(hipKernelNodeParams, sharedMemBytes),
            56
        );
    }
}
