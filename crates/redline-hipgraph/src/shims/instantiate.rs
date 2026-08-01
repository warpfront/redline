// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! Translating shims for graph instantiation, upload, and child-graph access.
//!
//! These entry points create or return handles rather than only translating
//! them. Exec creation mirrors `hipGraphInstantiate` in `lib.rs`. Child graphs
//! returned from a node are wrapped with `allocate_graph`, reusing an existing
//! wrapper when the same native graph is requested again.

#![allow(non_snake_case, non_camel_case_types)]

use std::collections::HashMap;
use std::ptr;
use std::sync::LazyLock;

use crate::abi::{
    hipError_t, hipErrorInvalidHandle, hipErrorInvalidValue, hipErrorNotSupported, hipGraph_t,
    hipGraphExec_t, hipGraphNode_t, hipStream_t, hipSuccess,
};
use crate::{
    ExecState, allocate_exec, allocate_graph, exec_handle, global, graph_handle, is_exec, is_graph,
    lock, native_exec_or_passthrough, native_graph_or_passthrough, native_node_or_passthrough,
    node_snapshot, real_symbol,
};

/// Serializes [`graph_for_native`] scan-and-mint. Outer lock only — never held
/// while inverted against `GraphState` (callers must not hold state when entering).
static GRAPH_NATIVE_MINT: LazyLock<std::sync::Mutex<()>> =
    LazyLock::new(|| std::sync::Mutex::new(()));

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

/// Return an existing wrapper for `native_graph`, or allocate a new one.
///
/// `hipGraphChildGraphNodeGetGraph` may be called repeatedly for the same node;
/// handing out two of our handles for one native graph would split ownership.
/// Scan-and-mint runs under [`GRAPH_NATIVE_MINT`] so concurrent callers cannot
/// both miss and allocate. Inside the critical section the handles guard is
/// still dropped before any `GraphState` lock (order remains state → handles).
fn graph_for_native(native_graph: usize) -> hipGraph_t {
    if native_graph == 0 {
        return ptr::null_mut();
    }
    let _mint = lock(&GRAPH_NATIVE_MINT);
    // Do not hold `handles` while locking graph state: other paths take state
    // then handles (e.g. `allocate_node`), so the opposite order deadlocks.
    let keys: Vec<usize> = lock(&global().handles).graphs.iter().copied().collect();
    for key in keys {
        let candidate = key as hipGraph_t;
        if !is_graph(candidate) {
            continue;
        }
        let Some(handle) = graph_handle(candidate) else {
            continue;
        };
        if lock(&handle.state).native_graph == native_graph {
            return candidate;
        }
    }
    // Native child topology was never modelled — force native so instantiate
    // cannot try to PM4-compile the empty retained graph.
    let graph = allocate_graph(native_graph);
    if let Some(handle) = graph_handle(graph) {
        lock(&handle.state).force_native = true;
    }
    graph
}

/// Snapshot node maps the same way `hipGraphInstantiate` does.
fn exec_node_maps(
    graph: hipGraph_t,
) -> (
    HashMap<usize, redline_dispatch::NodeId>,
    HashMap<usize, usize>,
) {
    let mut nodes = HashMap::new();
    let mut native_nodes = HashMap::new();
    let Some(handle) = graph_handle(graph) else {
        return (nodes, native_nodes);
    };
    let state = lock(&handle.state);
    for &node_key in &state.node_handles {
        if let Some(node) = node_snapshot(node_key as hipGraphNode_t) {
            if let Some(id) = node.node {
                nodes.insert(node_key, id);
            }
            if node.native_node != 0 {
                native_nodes.insert(node_key, node.native_node);
            }
        }
    }
    (nodes, native_nodes)
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

/// Mutating — creates an exec. Instantiates natively and registers `force_native`
/// so flags/upload/result_out stay with HIP; we only wrap the exec handle.
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

    let Some(handle) = graph_handle(graph) else {
        return hipErrorInvalidHandle;
    };
    let native_graph = lock(&handle.state).native_graph;
    if native_graph == 0 {
        return hipErrorNotSupported;
    }

    // Copy params for the native call. `errNode_out` comes back as a native
    // node pointer and is translated to one of ours when we track it.
    let mut native_params = unsafe { *instantiateParams };
    native_params.errNode_out = ptr::null_mut();

    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphInstantiateWithParams\0") })
    else {
        return hipErrorNotSupported;
    };
    let mut native_exec_ptr = ptr::null_mut();
    let status = unsafe {
        function(
            &mut native_exec_ptr,
            native_graph as hipGraph_t,
            &mut native_params,
        )
    };

    // Always publish result_out / errNode_out / flags echo from native.
    // errNode_out is application-visible: intern so only our handles escape.
    unsafe {
        (*instantiateParams).result_out = native_params.result_out;
        (*instantiateParams).flags = native_params.flags;
        (*instantiateParams).errNode_out =
            crate::intern_native_node(graph, native_params.errNode_out);
        // uploadStream is an in-param; leave caller's value alone.
    }

    if status != hipSuccess {
        return status;
    }

    let native_exec = native_exec_ptr as usize;
    let (nodes, native_nodes) = exec_node_maps(graph);
    let node_meta = lock(&handle.state).node_meta.clone();
    let exec = allocate_exec(ExecState {
        exec: None,
        replay: None,
        dirty: false,
        node_meta,
        nodes,
        native_nodes,
        native_exec,
        force_native: true,
    });
    unsafe { *pGraphExec = exec };
    hipSuccess
}

/// Mutating — upload makes the native exec authoritative for subsequent launch.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphUpload(
    graphExec: hipGraphExec_t,
    stream: hipStream_t,
) -> hipError_t {
    type Function = unsafe extern "C" fn(hipGraphExec_t, hipStream_t) -> hipError_t;

    if !is_exec(graphExec) {
        return match unsafe { real_symbol::<Function>(b"hipGraphUpload\0") } {
            Some(function) => unsafe { function(graphExec, stream) },
            None => hipErrorNotSupported,
        };
    }
    let Some(exec) = exec_handle(graphExec) else {
        return hipErrorInvalidHandle;
    };
    let native_exec = {
        let state = lock(&exec.state);
        if state.native_exec == 0 {
            return hipErrorNotSupported;
        }
        state.native_exec as hipGraphExec_t
    };
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphUpload\0") }) else {
        return hipErrorNotSupported;
    };
    let status = unsafe { function(native_exec, stream) };
    if status == hipSuccess {
        lock(&exec.state).force_native = true;
    }
    status
}

/// Read-only — translate exec, forward flags query; do not set force_native.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphExecGetFlags(
    graphExec: hipGraphExec_t,
    flags: *mut c_ull,
) -> hipError_t {
    type Function = unsafe extern "C" fn(hipGraphExec_t, *mut c_ull) -> hipError_t;

    if !is_exec(graphExec) {
        return match unsafe { real_symbol::<Function>(b"hipGraphExecGetFlags\0") } {
            Some(function) => unsafe { function(graphExec, flags) },
            None => hipErrorNotSupported,
        };
    }
    if flags.is_null() {
        return hipErrorInvalidValue;
    }
    let Some(exec) = exec_handle(graphExec) else {
        return hipErrorInvalidHandle;
    };
    let native_exec = {
        let state = lock(&exec.state);
        if state.native_exec == 0 {
            return hipErrorNotSupported;
        }
        state.native_exec as hipGraphExec_t
    };
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphExecGetFlags\0") }) else {
        return hipErrorNotSupported;
    };
    unsafe { function(native_exec, flags) }
}

/// Read-only w.r.t. graph topology — returns a GRAPH handle for the child.
/// Wraps the native child so the caller never holds a raw native graph pointer
/// against our API. Repeat calls reuse the same wrapper.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphChildGraphNodeGetGraph(
    node: hipGraphNode_t,
    pGraph: *mut hipGraph_t,
) -> hipError_t {
    type Function = unsafe extern "C" fn(hipGraphNode_t, *mut hipGraph_t) -> hipError_t;

    let Some(snapshot) = node_snapshot(node) else {
        return match unsafe { real_symbol::<Function>(b"hipGraphChildGraphNodeGetGraph\0") } {
            Some(function) => unsafe { function(node, pGraph) },
            None => hipErrorNotSupported,
        };
    };
    if pGraph.is_null() {
        return hipErrorInvalidValue;
    }
    if snapshot.native_node == 0 {
        return hipErrorNotSupported;
    }
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphChildGraphNodeGetGraph\0") })
    else {
        return hipErrorNotSupported;
    };
    let mut native_graph = ptr::null_mut();
    let status = unsafe { function(snapshot.native_node as hipGraphNode_t, &mut native_graph) };
    if status != hipSuccess {
        return status;
    }
    unsafe { *pGraph = graph_for_native(native_graph as usize) };
    hipSuccess
}

/// Mutating — updates child graph params on an exec; force_native on the exec.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphExecChildGraphNodeSetParams(
    hGraphExec: hipGraphExec_t,
    node: hipGraphNode_t,
    childGraph: hipGraph_t,
) -> hipError_t {
    type Function = unsafe extern "C" fn(hipGraphExec_t, hipGraphNode_t, hipGraph_t) -> hipError_t;

    if !is_exec(hGraphExec) {
        return match unsafe { real_symbol::<Function>(b"hipGraphExecChildGraphNodeSetParams\0") } {
            Some(function) => unsafe {
                function(
                    native_exec_or_passthrough(hGraphExec),
                    native_node_or_passthrough(node),
                    native_graph_or_passthrough(childGraph),
                )
            },
            None => hipErrorNotSupported,
        };
    }
    let Some(exec) = exec_handle(hGraphExec) else {
        return hipErrorInvalidHandle;
    };

    let native_node = match node_snapshot(node) {
        Some(snapshot) => {
            if snapshot.native_node == 0 {
                return hipErrorNotSupported;
            }
            snapshot.native_node as hipGraphNode_t
        }
        None => node,
    };

    let native_child = if is_graph(childGraph) {
        let Some(child) = graph_handle(childGraph) else {
            return hipErrorInvalidHandle;
        };
        let native = lock(&child.state).native_graph;
        if native == 0 {
            return hipErrorNotSupported;
        }
        native as hipGraph_t
    } else {
        childGraph
    };

    let native_exec = {
        let state = lock(&exec.state);
        if state.native_exec == 0 {
            return hipErrorNotSupported;
        }
        state.native_exec as hipGraphExec_t
    };

    let Some(function) =
        (unsafe { real_symbol::<Function>(b"hipGraphExecChildGraphNodeSetParams\0") })
    else {
        return hipErrorNotSupported;
    };
    let status = unsafe { function(native_exec, native_node, native_child) };
    if status == hipSuccess {
        lock(&exec.state).force_native = true;
    }
    status
}
