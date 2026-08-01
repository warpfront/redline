// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! Plan-staleness shims for memset/host SetParams and related exec variants.
//!
//! Host nodes carry a function pointer and user data — forwarded as-is; the
//! callback is never wrapped or intercepted.

#![allow(non_camel_case_types, non_snake_case)]

use std::ffi::c_void;

use crate::abi::{hipGraphExec_t, hipGraphNode_t};

/// ROCm 7.14 `hipMemsetParams` — pointer payload only; we never inspect fields.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct hipMemsetParams {
    pub dst: *mut c_void,
    pub elementSize: u32,
    pub height: usize,
    pub pitch: usize,
    pub value: u32,
    pub width: usize,
}

/// ROCm 7.14 `hipHostFn_t`.
pub type hipHostFn_t = Option<unsafe extern "C" fn(userData: *mut c_void)>;

/// ROCm 7.14 `hipHostNodeParams` — callback + userData, forwarded untouched.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct hipHostNodeParams {
    pub fn_: hipHostFn_t,
    pub userData: *mut c_void,
}

node_mutating_shim!(
    hipGraphMemsetNodeSetParams,
    b"hipGraphMemsetNodeSetParams\0",
    node,
    (node: hipGraphNode_t, pNodeParams: *const hipMemsetParams)
);

node_mutating_shim!(
    hipGraphHostNodeSetParams,
    b"hipGraphHostNodeSetParams\0",
    node,
    (node: hipGraphNode_t, pNodeParams: *const hipHostNodeParams)
);

exec_mutating_shim!(
    hipGraphExecMemsetNodeSetParams,
    b"hipGraphExecMemsetNodeSetParams\0",
    hGraphExec,
    (
        hGraphExec: hipGraphExec_t,
        node: hipGraphNode_t,
        pNodeParams: *const hipMemsetParams
    )
);

exec_mutating_shim!(
    hipGraphExecHostNodeSetParams,
    b"hipGraphExecHostNodeSetParams\0",
    hGraphExec,
    (
        hGraphExec: hipGraphExec_t,
        node: hipGraphNode_t,
        pNodeParams: *const hipHostNodeParams
    )
);
