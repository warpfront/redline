// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! Translating shims for memset/host node Get/SetParams and their Exec variants.
//!
//! Host nodes carry a function pointer and user data — forwarded as-is; the
//! callback is never wrapped or intercepted.

#![allow(non_camel_case_types, non_snake_case)]

use std::ffi::c_void;

use crate::abi::{
    hipError_t, hipErrorInvalidHandle, hipErrorInvalidValue, hipErrorNotSupported, hipGraph_t,
    hipGraphExec_t, hipGraphNode_t, hipSuccess,
};
use crate::{exec_handle, graph_handle, is_exec, lock, node_snapshot, real_symbol};

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

/// Resolve our node handle to the native HIP node, or `None` if not ours.
fn native_node_of(node: hipGraphNode_t) -> Option<usize> {
    node_snapshot(node).map(|snap| snap.native_node)
}

/// Look up the native node for an exec update: prefer the exec's native_nodes
/// map, fall back to the live node snapshot.
fn exec_native_node(
    state: &crate::ExecState,
    node: hipGraphNode_t,
) -> Result<hipGraphNode_t, hipError_t> {
    if let Some(n) = state.native_nodes.get(&(node as usize)).copied() {
        if n != 0 {
            return Ok(n as hipGraphNode_t);
        }
        return Err(hipErrorNotSupported);
    }
    match node_snapshot(node) {
        Some(snap) if snap.native_node != 0 => Ok(snap.native_node as hipGraphNode_t),
        Some(_) => Err(hipErrorNotSupported),
        // Foreign node (e.g. minted wrapper for a native child/clone) — pass through.
        None => Ok(node),
    }
}

// --- memset node params -------------------------------------------------------

/// read-only: translate node → native, forward; do not set `force_native`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphMemsetNodeGetParams(
    node: hipGraphNode_t,
    pNodeParams: *mut hipMemsetParams,
) -> hipError_t {
    type Function = unsafe extern "C" fn(hipGraphNode_t, *mut hipMemsetParams) -> hipError_t;
    let Some(native) = native_node_of(node) else {
        return match unsafe { real_symbol::<Function>(b"hipGraphMemsetNodeGetParams\0") } {
            Some(function) => unsafe { function(node, pNodeParams) },
            None => hipErrorNotSupported,
        };
    };
    if pNodeParams.is_null() {
        return hipErrorInvalidValue;
    }
    if native == 0 {
        return hipErrorNotSupported;
    }
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphMemsetNodeGetParams\0") })
    else {
        return hipErrorNotSupported;
    };
    unsafe { function(native as hipGraphNode_t, pNodeParams) }
}

/// mutating: translate node → native, forward, mark owning graph `force_native`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphMemsetNodeSetParams(
    node: hipGraphNode_t,
    pNodeParams: *const hipMemsetParams,
) -> hipError_t {
    type Function = unsafe extern "C" fn(hipGraphNode_t, *const hipMemsetParams) -> hipError_t;
    let Some(snap) = node_snapshot(node) else {
        return match unsafe { real_symbol::<Function>(b"hipGraphMemsetNodeSetParams\0") } {
            Some(function) => unsafe { function(node, pNodeParams) },
            None => hipErrorNotSupported,
        };
    };
    if pNodeParams.is_null() {
        return hipErrorInvalidValue;
    }
    if snap.native_node == 0 {
        return hipErrorNotSupported;
    }
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphMemsetNodeSetParams\0") })
    else {
        return hipErrorNotSupported;
    };
    let status = unsafe { function(snap.native_node as hipGraphNode_t, pNodeParams) };
    if status == hipSuccess {
        if let Some(handle) = graph_handle(snap.owner as hipGraph_t) {
            lock(&handle.state).force_native = true;
        }
    }
    status
}

/// mutating (exec): translate exec + node → native, forward, mark exec `force_native`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphExecMemsetNodeSetParams(
    hGraphExec: hipGraphExec_t,
    node: hipGraphNode_t,
    pNodeParams: *const hipMemsetParams,
) -> hipError_t {
    type Function =
        unsafe extern "C" fn(hipGraphExec_t, hipGraphNode_t, *const hipMemsetParams) -> hipError_t;
    if !is_exec(hGraphExec) {
        return match unsafe { real_symbol::<Function>(b"hipGraphExecMemsetNodeSetParams\0") } {
            Some(function) => unsafe {
                function(
                    crate::native_exec_or_passthrough(hGraphExec),
                    crate::native_node_or_passthrough(node),
                    pNodeParams,
                )
            },
            None => hipErrorNotSupported,
        };
    }
    if pNodeParams.is_null() {
        return hipErrorInvalidValue;
    }
    let Some(handle) = exec_handle(hGraphExec) else {
        return hipErrorInvalidHandle;
    };
    let mut state = lock(&handle.state);
    if state.native_exec == 0 {
        return hipErrorNotSupported;
    }
    let native_node = match exec_native_node(&state, node) {
        Ok(n) => n,
        Err(status) => return status,
    };
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphExecMemsetNodeSetParams\0") })
    else {
        return hipErrorNotSupported;
    };
    let status = unsafe {
        function(
            state.native_exec as hipGraphExec_t,
            native_node,
            pNodeParams,
        )
    };
    if status == hipSuccess {
        state.force_native = true;
    }
    status
}

// --- host node params ---------------------------------------------------------

/// read-only: translate node → native, forward; do not set `force_native`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphHostNodeGetParams(
    node: hipGraphNode_t,
    pNodeParams: *mut hipHostNodeParams,
) -> hipError_t {
    type Function = unsafe extern "C" fn(hipGraphNode_t, *mut hipHostNodeParams) -> hipError_t;
    let Some(native) = native_node_of(node) else {
        return match unsafe { real_symbol::<Function>(b"hipGraphHostNodeGetParams\0") } {
            Some(function) => unsafe { function(node, pNodeParams) },
            None => hipErrorNotSupported,
        };
    };
    if pNodeParams.is_null() {
        return hipErrorInvalidValue;
    }
    if native == 0 {
        return hipErrorNotSupported;
    }
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphHostNodeGetParams\0") })
    else {
        return hipErrorNotSupported;
    };
    unsafe { function(native as hipGraphNode_t, pNodeParams) }
}

/// mutating: translate node → native, forward, mark owning graph `force_native`.
/// Host callback pointer and userData are forwarded as-is.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphHostNodeSetParams(
    node: hipGraphNode_t,
    pNodeParams: *const hipHostNodeParams,
) -> hipError_t {
    type Function = unsafe extern "C" fn(hipGraphNode_t, *const hipHostNodeParams) -> hipError_t;
    let Some(snap) = node_snapshot(node) else {
        return match unsafe { real_symbol::<Function>(b"hipGraphHostNodeSetParams\0") } {
            Some(function) => unsafe { function(node, pNodeParams) },
            None => hipErrorNotSupported,
        };
    };
    if pNodeParams.is_null() {
        return hipErrorInvalidValue;
    }
    if snap.native_node == 0 {
        return hipErrorNotSupported;
    }
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphHostNodeSetParams\0") })
    else {
        return hipErrorNotSupported;
    };
    let status = unsafe { function(snap.native_node as hipGraphNode_t, pNodeParams) };
    if status == hipSuccess {
        if let Some(handle) = graph_handle(snap.owner as hipGraph_t) {
            lock(&handle.state).force_native = true;
        }
    }
    status
}

/// mutating (exec): translate exec + node → native, forward, mark exec `force_native`.
/// Host callback pointer and userData are forwarded as-is.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphExecHostNodeSetParams(
    hGraphExec: hipGraphExec_t,
    node: hipGraphNode_t,
    pNodeParams: *const hipHostNodeParams,
) -> hipError_t {
    type Function = unsafe extern "C" fn(
        hipGraphExec_t,
        hipGraphNode_t,
        *const hipHostNodeParams,
    ) -> hipError_t;
    if !is_exec(hGraphExec) {
        return match unsafe { real_symbol::<Function>(b"hipGraphExecHostNodeSetParams\0") } {
            Some(function) => unsafe {
                function(
                    crate::native_exec_or_passthrough(hGraphExec),
                    crate::native_node_or_passthrough(node),
                    pNodeParams,
                )
            },
            None => hipErrorNotSupported,
        };
    }
    if pNodeParams.is_null() {
        return hipErrorInvalidValue;
    }
    let Some(handle) = exec_handle(hGraphExec) else {
        return hipErrorInvalidHandle;
    };
    let mut state = lock(&handle.state);
    if state.native_exec == 0 {
        return hipErrorNotSupported;
    }
    let native_node = match exec_native_node(&state, node) {
        Ok(n) => n,
        Err(status) => return status,
    };
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphExecHostNodeSetParams\0") })
    else {
        return hipErrorNotSupported;
    };
    let status = unsafe {
        function(
            state.native_exec as hipGraphExec_t,
            native_node,
            pNodeParams,
        )
    };
    if status == hipSuccess {
        state.force_native = true;
    }
    status
}
