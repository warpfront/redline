// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! KEEP-CREATE instantiate shims plus exec mutators that lived in this file.
//!
//! Application holds real native graph/exec pointers. The two instantiate
//! entry points create an exec we may accelerate, so they mirror
//! `hipGraphInstantiate` bookkeeping and `register_exec` the native result.
//! Upload / child-graph exec set-params only mark the exec plan stale.

#![allow(non_snake_case, non_camel_case_types)]

use std::collections::HashMap;
use std::ptr;

use crate::abi::{
    hipError_t, hipErrorInvalidHandle, hipErrorInvalidValue, hipErrorNotSupported, hipErrorUnknown,
    hipGraph_t, hipGraphExec_t, hipGraphNode_t, hipStream_t, hipSuccess,
};
use crate::{ExecState, graph_state, is_graph, lock, real_symbol, register_exec};

/// `unsigned long long` on the HIP C ABI.
type c_ull = u64;

/// C `hipGraphInstantiateResult` (enum stored as `int`).
type hipGraphInstantiateResult = i32;

/// Matches `hipGraphInstantiateParams` in `hip_runtime_api.h`.
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct hipGraphInstantiateParams {
    errNode_out: hipGraphNode_t,
    flags: c_ull,
    result_out: hipGraphInstantiateResult,
    uploadStream: hipStream_t,
}

/// Mutating — creates an exec. ROCm documents flags as ignored and equivalent
/// to plain `hipGraphInstantiate`; delegate so handle bookkeeping (PM4 replay,
/// node maps, `force_native`) stays identical to the already-exported path.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphInstantiateWithFlags(
    pGraphExec: *mut hipGraphExec_t,
    graph: hipGraph_t,
    flags: c_ull,
) -> hipError_t {
    type Function = unsafe extern "C" fn(*mut hipGraphExec_t, hipGraph_t, c_ull) -> hipError_t;

    if pGraphExec.is_null() {
        return hipErrorInvalidValue;
    }
    if !is_graph(graph) {
        return match unsafe { real_symbol::<Function>(b"hipGraphInstantiateWithFlags\0") } {
            Some(function) => unsafe { function(pGraphExec, graph, flags) },
            None => hipErrorNotSupported,
        };
    }
    // `flags` is unsupported on this ROCm and behaves as plain instantiate.
    let _ = flags;
    unsafe { crate::hipGraphInstantiate(pGraphExec, graph, ptr::null_mut(), ptr::null_mut(), 0) }
}

/// Mutating — creates an exec via the params form. Instantiates natively (flags /
/// upload / result_out stay with HIP) and registers the native exec once.
///
/// This params form carries semantics Redline does not model, so the exec is
/// force-native from birth. Its native-node map is deliberately empty because
/// no PM4 path can consult it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphInstantiateWithParams(
    pGraphExec: *mut hipGraphExec_t,
    graph: hipGraph_t,
    instantiateParams: *mut hipGraphInstantiateParams,
) -> hipError_t {
    type Function = unsafe extern "C" fn(
        *mut hipGraphExec_t,
        hipGraph_t,
        *mut hipGraphInstantiateParams,
    ) -> hipError_t;

    if pGraphExec.is_null() || instantiateParams.is_null() {
        return hipErrorInvalidValue;
    }
    if !is_graph(graph) {
        return match unsafe { real_symbol::<Function>(b"hipGraphInstantiateWithParams\0") } {
            Some(function) => unsafe { function(pGraphExec, graph, instantiateParams) },
            None => hipErrorNotSupported,
        };
    }
    let Some(graph_state) = graph_state(graph) else {
        return hipErrorInvalidHandle;
    };

    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphInstantiateWithParams\0") })
    else {
        return hipErrorNotSupported;
    };

    // Native identity: `graph` is already HIP's pointer. Pass params through so
    // HIP fills result_out / errNode_out in the caller's struct.
    let mut native_exec_ptr = ptr::null_mut();
    let status = unsafe { function(&mut native_exec_ptr, graph, instantiateParams) };
    if status != hipSuccess {
        return status;
    }
    if native_exec_ptr.is_null() {
        return hipErrorUnknown;
    }

    let node_meta = lock(&graph_state).node_meta.clone();
    let native_exec = native_exec_ptr as usize;
    register_exec(
        native_exec,
        ExecState {
            exec: None,
            replay: None,
            dirty: false,
            node_meta,
            // This params form is force-native from birth: no PM4 path can
            // consult native-node mappings, so leave them deliberately empty.
            nodes: HashMap::new(),
            force_native: true,
        },
    );
    unsafe { *pGraphExec = native_exec_ptr };
    hipSuccess
}

exec_mutating_shim!(
    hipGraphUpload,
    b"hipGraphUpload\0",
    graphExec,
    (graphExec: hipGraphExec_t, stream: hipStream_t,)
);

exec_mutating_shim!(
    hipGraphExecChildGraphNodeSetParams,
    b"hipGraphExecChildGraphNodeSetParams\0",
    hGraphExec,
    (
        hGraphExec: hipGraphExec_t,
        node: hipGraphNode_t,
        childGraph: hipGraph_t,
    )
);
