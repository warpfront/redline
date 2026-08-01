// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

#![allow(non_snake_case, non_camel_case_types)]
//! Topology-mutating hipGraph shims: add/remove nodes and edges, clone, enable.

use std::ffi::c_void;
use std::mem;
use std::ptr;

use crate::abi::{
    hipError_t, hipErrorInvalidHandle, hipErrorInvalidValue, hipErrorNotSupported, hipGraph_t,
    hipGraphExec_t, hipGraphNode_t, hipSuccess,
};
use crate::{
    NodeHandle, allocate_graph, allocate_node, exec_handle, finish_native_only_node, global,
    graph_handle, is_exec, is_graph, lock, node_snapshot, real_symbol,
};

/// `hipGraphNodeTypeGraph` — node which executes an embedded graph.
const HIP_GRAPH_NODE_TYPE_GRAPH: i32 = 4;

/// ROCm 7.14 `hipChildGraphNodeParams` — `{ hipGraph_t graph; }`.
#[repr(C)]
#[derive(Clone, Copy)]
struct HipChildGraphNodeParams {
    graph: hipGraph_t,
}

/// Payload union of `hipGraphNodeParams`. Only the child-graph arm is decoded;
/// other variants are forwarded as opaque bytes via `reserved1`.
#[repr(C)]
#[derive(Clone, Copy)]
union HipGraphNodeParamsPayload {
    reserved1: [i64; 29],
    graph: HipChildGraphNodeParams,
}

/// ROCm 7.14 `hipGraphNodeParams` mirror.
///
/// Layout verified against `/opt/rocm/core-7.14/include/hip/hip_runtime_api.h`
/// (~2076-2095) via a host C program linked against the ROCm headers:
///   `sizeof=256`, `align=8`, `type@0`, `reserved0@4`, union/`graph@16`,
///   `reserved2@248`. Compile-time asserts below lock the Rust mirror to that.
#[repr(C)]
#[derive(Clone, Copy)]
struct HipGraphNodeParams {
    type_: i32,
    reserved0: [i32; 3],
    u: HipGraphNodeParamsPayload,
    reserved2: i64,
}

const _: () = {
    assert!(mem::size_of::<HipGraphNodeParams>() == 256);
    assert!(mem::align_of::<HipGraphNodeParams>() == 8);
    assert!(mem::size_of::<HipChildGraphNodeParams>() == 8);
    assert!(mem::offset_of!(HipGraphNodeParams, type_) == 0);
    assert!(mem::offset_of!(HipGraphNodeParams, reserved0) == 4);
    assert!(mem::offset_of!(HipGraphNodeParams, u) == 16);
    assert!(mem::offset_of!(HipGraphNodeParams, reserved2) == 248);
};

/// If `node_params` describes a child-graph node, return a local copy whose
/// embedded `hipGraph_t` has been rewritten through `native_graph_or_passthrough`.
/// Never mutates the caller's struct. Non-child-graph params (and null) yield
/// `None` so the original pointer can be forwarded unchanged.
unsafe fn translate_child_graph_params(node_params: *mut c_void) -> Option<HipGraphNodeParams> {
    if node_params.is_null() {
        return None;
    }
    // SAFETY: HIP contracts this as `hipGraphNodeParams*`; we only read `type_`.
    let src = unsafe { &*node_params.cast::<HipGraphNodeParams>() };
    if src.type_ != HIP_GRAPH_NODE_TYPE_GRAPH {
        return None;
    }
    let mut local = *src;
    // SAFETY: `type_` is Graph so the `graph` arm is the live union member.
    unsafe {
        local.u.graph.graph = crate::native_graph_or_passthrough(local.u.graph.graph);
    }
    Some(local)
}

/// Pointer to forward for `nodeParams`: the rewritten local copy when present,
/// otherwise the caller's original pointer.
fn node_params_ptr(
    original: *mut c_void,
    translated: &mut Option<HipGraphNodeParams>,
) -> *mut c_void {
    match translated {
        Some(local) => ptr::from_mut(local).cast::<c_void>(),
        None => original,
    }
}

/// Map an application node handle to the native node inside an owned exec.
fn exec_native_node(
    state: &crate::ExecState,
    node: hipGraphNode_t,
) -> Result<hipGraphNode_t, hipError_t> {
    let key = node as usize;
    if let Some(&native) = state.native_nodes.get(&key) {
        if native != 0 {
            return Ok(native as hipGraphNode_t);
        }
        return Err(hipErrorNotSupported);
    }
    match node_snapshot(node) {
        Some(snap) if snap.native_node != 0 => Ok(snap.native_node as hipGraphNode_t),
        Some(_) => Err(hipErrorNotSupported),
        None => Ok(node),
    }
}

/// Upper bound on a dependency-array length we will allocate for. Each element
/// is a `hipGraphNode_t` the caller must own; a few million is already far past
/// any real graph. Larger counts are rejected as `hipErrorInvalidValue` rather
/// than risking an allocation failure that aborts through `extern "C"`.
const MAX_DEPENDENCY_COUNT: usize = 4_000_000;

/// hipGraphAddEmptyNode — mutating (adds a node).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphAddEmptyNode(
    pGraphNode: *mut hipGraphNode_t,
    graph: hipGraph_t,
    pDependencies: *const hipGraphNode_t,
    numDependencies: usize,
) -> hipError_t {
    type Function = unsafe extern "C" fn(
        *mut hipGraphNode_t,
        hipGraph_t,
        *const hipGraphNode_t,
        usize,
    ) -> hipError_t;

    if numDependencies > MAX_DEPENDENCY_COUNT {
        return hipErrorInvalidValue;
    }
    if !is_graph(graph) {
        let translated = match unsafe {
            crate::translate_dependencies_passthrough(pDependencies, numDependencies)
        } {
            Ok(value) => value,
            Err(status) => return status,
        };
        let deps_ptr = translated
            .as_ref()
            .map(|d| d.as_ptr())
            .unwrap_or(pDependencies);
        return match unsafe { real_symbol::<Function>(b"hipGraphAddEmptyNode\0") } {
            Some(function) => unsafe { function(pGraphNode, graph, deps_ptr, numDependencies) },
            None => hipErrorNotSupported,
        };
    }
    if pGraphNode.is_null() {
        return hipErrorInvalidValue;
    }
    let Some(handle) = graph_handle(graph) else {
        return hipErrorInvalidHandle;
    };
    let native_graph = lock(&handle.state).native_graph;
    if native_graph == 0 {
        return hipErrorNotSupported;
    }
    // Forwarding only — per-element passthrough so minted/foreign deps are not rejected.
    let translated = match unsafe {
        crate::translate_dependencies_passthrough(pDependencies, numDependencies)
    } {
        Ok(value) => value,
        Err(status) => return status,
    };
    let deps_ptr = translated
        .as_ref()
        .map(|d| d.as_ptr())
        .unwrap_or(pDependencies);
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphAddEmptyNode\0") }) else {
        return hipErrorNotSupported;
    };
    let mut native_node = ptr::null_mut();
    let status = unsafe {
        function(
            &mut native_node,
            native_graph as hipGraph_t,
            deps_ptr,
            numDependencies,
        )
    };
    if status == hipSuccess {
        finish_native_only_node(graph, native_node, pGraphNode)
    } else {
        status
    }
}

/// hipGraphAddNode — mutating (adds a node). Child-graph params are rewritten.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphAddNode(
    pGraphNode: *mut hipGraphNode_t,
    graph: hipGraph_t,
    pDependencies: *const hipGraphNode_t,
    numDependencies: usize,
    nodeParams: *mut c_void,
) -> hipError_t {
    type Function = unsafe extern "C" fn(
        *mut hipGraphNode_t,
        hipGraph_t,
        *const hipGraphNode_t,
        usize,
        *mut c_void,
    ) -> hipError_t;

    if numDependencies > MAX_DEPENDENCY_COUNT {
        return hipErrorInvalidValue;
    }

    let mut params_keep = unsafe { translate_child_graph_params(nodeParams) };
    let fwd_params = node_params_ptr(nodeParams, &mut params_keep);

    if !is_graph(graph) {
        let translated = match unsafe {
            crate::translate_dependencies_passthrough(pDependencies, numDependencies)
        } {
            Ok(value) => value,
            Err(status) => return status,
        };
        let deps_ptr = translated
            .as_ref()
            .map(|d| d.as_ptr())
            .unwrap_or(pDependencies);
        return match unsafe { real_symbol::<Function>(b"hipGraphAddNode\0") } {
            Some(function) => unsafe {
                function(pGraphNode, graph, deps_ptr, numDependencies, fwd_params)
            },
            None => hipErrorNotSupported,
        };
    }
    if pGraphNode.is_null() {
        return hipErrorInvalidValue;
    }
    let Some(handle) = graph_handle(graph) else {
        return hipErrorInvalidHandle;
    };
    let native_graph = lock(&handle.state).native_graph;
    if native_graph == 0 {
        return hipErrorNotSupported;
    }
    let translated = match unsafe {
        crate::translate_dependencies_passthrough(pDependencies, numDependencies)
    } {
        Ok(value) => value,
        Err(status) => return status,
    };
    let deps_ptr = translated
        .as_ref()
        .map(|d| d.as_ptr())
        .unwrap_or(pDependencies);
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphAddNode\0") }) else {
        return hipErrorNotSupported;
    };
    let mut native_node = ptr::null_mut();
    let status = unsafe {
        function(
            &mut native_node,
            native_graph as hipGraph_t,
            deps_ptr,
            numDependencies,
            fwd_params,
        )
    };
    if status == hipSuccess {
        finish_native_only_node(graph, native_node, pGraphNode)
    } else {
        status
    }
}

/// hipGraphClone — mutating for the new graph (no lowered plan yet).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphClone(
    pGraphClone: *mut hipGraph_t,
    originalGraph: hipGraph_t,
) -> hipError_t {
    type Function = unsafe extern "C" fn(*mut hipGraph_t, hipGraph_t) -> hipError_t;

    if !is_graph(originalGraph) {
        return match unsafe { real_symbol::<Function>(b"hipGraphClone\0") } {
            Some(function) => unsafe { function(pGraphClone, originalGraph) },
            None => hipErrorNotSupported,
        };
    }
    if pGraphClone.is_null() {
        return hipErrorInvalidValue;
    }
    let Some(handle) = graph_handle(originalGraph) else {
        return hipErrorInvalidHandle;
    };
    let native_graph = lock(&handle.state).native_graph;
    if native_graph == 0 {
        return hipErrorNotSupported;
    }
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphClone\0") }) else {
        return hipErrorNotSupported;
    };
    let mut native_clone = ptr::null_mut();
    let status = unsafe { function(&mut native_clone, native_graph as hipGraph_t) };
    if status != hipSuccess {
        return status;
    }
    // Caller must receive one of our handles. No PM4 plan exists for the clone.
    let clone = allocate_graph(native_clone as usize);
    if let Some(clone_handle) = graph_handle(clone) {
        lock(&clone_handle.state).force_native = true;
    }
    unsafe { *pGraphClone = clone };
    hipSuccess
}

/// hipGraphDestroyNode — mutating; drop our node bookkeeping after the native call.
///
/// Removal from the node registry is the atomic ownership claim: only the caller
/// that successfully removes the key may destroy the native node and free the
/// `Box`. A racing second destroy of an owned wrapper sees the key gone.
///
/// When the key is absent we forward to real HIP with the pointer unchanged so
/// genuine foreign (never-ours) nodes are destroyed. Residual tradeoff: an
/// application that double-destroys one of OUR nodes will have the second call
/// forwarded too, because after the wrapper is freed a bare pointer cannot be
/// distinguished from a foreign node. That is an application-level
/// use-after-destroy bug; silently swallowing every foreign destroy to hide it
/// would break correct programs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphDestroyNode(node: hipGraphNode_t) -> hipError_t {
    type Function = unsafe extern "C" fn(hipGraphNode_t) -> hipError_t;

    if node.is_null() {
        return hipSuccess;
    }

    let key = node as usize;

    // Atomic claim under the handles lock. Lock order: handles only — never hold
    // a GraphState/ExecState lock across this section (state is taken later).
    let claimed = {
        let mut handles = lock(&global().handles);
        if !handles.nodes.remove(&key) {
            // Key absent: never ours, or another call already claimed destruction
            // of our wrapper. Forward with the pointer unchanged so genuine
            // foreign nodes are destroyed by real HIP.
            //
            // Residual tradeoff: a double-destroy of one of OUR nodes also lands
            // here after the first claimer frees the wrapper, and will be
            // forwarded. Silently returning success for every missing key would
            // hide that UAF at the cost of leaking every never-ours native node
            // — the wrong trade for correct programs.
            drop(handles);
            return match unsafe { real_symbol::<Function>(b"hipGraphDestroyNode\0") } {
                Some(function) => unsafe { function(node) },
                None => hipErrorNotSupported,
            };
        }
        // SAFETY: key was registered; allocate_node paired Box::into_raw.
        // NodeHandle is Copy; leave the allocation in place until Box::from_raw.
        unsafe { *node.cast::<NodeHandle>() }
    };

    let status = if claimed.native_node != 0 {
        match unsafe { real_symbol::<Function>(b"hipGraphDestroyNode\0") } {
            Some(function) => unsafe { function(claimed.native_node as hipGraphNode_t) },
            None => hipErrorNotSupported,
        }
    } else {
        // No native mirror — still tear down our handle so the owner is not left stale.
        hipSuccess
    };
    if status != hipSuccess {
        // Native destroy failed: restore registry entry so the handle remains valid.
        lock(&global().handles).nodes.insert(key);
        return status;
    }

    if let Some(owner) = graph_handle(claimed.owner as hipGraph_t) {
        let mut state = lock(&owner.state);
        state.node_handles.retain(|&h| h != key);
        if let Some(id) = claimed.node {
            state.node_meta.remove(&id);
        }
        state.force_native = true;
    }

    // Free the heap NodeHandle. Registry entry already claimed above.
    drop(unsafe { Box::from_raw(node.cast::<NodeHandle>()) });
    hipSuccess
}

/// hipGraphExecNodeSetParams — mutating (exec node params).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphExecNodeSetParams(
    graphExec: hipGraphExec_t,
    node: hipGraphNode_t,
    nodeParams: *mut c_void,
) -> hipError_t {
    type Function = unsafe extern "C" fn(hipGraphExec_t, hipGraphNode_t, *mut c_void) -> hipError_t;

    let mut params_keep = unsafe { translate_child_graph_params(nodeParams) };
    let fwd_params = node_params_ptr(nodeParams, &mut params_keep);

    if !is_exec(graphExec) {
        return match unsafe { real_symbol::<Function>(b"hipGraphExecNodeSetParams\0") } {
            Some(function) => unsafe {
                function(
                    crate::native_exec_or_passthrough(graphExec),
                    crate::native_node_or_passthrough(node),
                    fwd_params,
                )
            },
            None => hipErrorNotSupported,
        };
    }
    let Some(handle) = exec_handle(graphExec) else {
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
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphExecNodeSetParams\0") })
    else {
        return hipErrorNotSupported;
    };
    let status = unsafe { function(state.native_exec as hipGraphExec_t, native_node, fwd_params) };
    if status == hipSuccess {
        state.force_native = true;
    }
    status
}

/// hipGraphNodeFindInClone — maps a node into a clone; wrap the result if the clone is ours.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphNodeFindInClone(
    pNode: *mut hipGraphNode_t,
    originalNode: hipGraphNode_t,
    clonedGraph: hipGraph_t,
) -> hipError_t {
    type Function =
        unsafe extern "C" fn(*mut hipGraphNode_t, hipGraphNode_t, hipGraph_t) -> hipError_t;

    let orig_snap = node_snapshot(originalNode);
    let clone_ours = is_graph(clonedGraph);
    if orig_snap.is_none() && !clone_ours {
        return match unsafe { real_symbol::<Function>(b"hipGraphNodeFindInClone\0") } {
            Some(function) => unsafe { function(pNode, originalNode, clonedGraph) },
            None => hipErrorNotSupported,
        };
    }
    if pNode.is_null() {
        return hipErrorInvalidValue;
    }

    let native_original = match orig_snap {
        None => originalNode,
        Some(snap) if snap.native_node != 0 => snap.native_node as hipGraphNode_t,
        Some(_) => return hipErrorNotSupported,
    };

    let native_clone = if clone_ours {
        let Some(handle) = graph_handle(clonedGraph) else {
            return hipErrorInvalidHandle;
        };
        let native_graph = lock(&handle.state).native_graph;
        if native_graph == 0 {
            return hipErrorNotSupported;
        }
        native_graph as hipGraph_t
    } else {
        clonedGraph
    };

    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphNodeFindInClone\0") }) else {
        return hipErrorNotSupported;
    };
    let mut native_found = ptr::null_mut();
    let status = unsafe { function(&mut native_found, native_original, native_clone) };
    if status != hipSuccess {
        return status;
    }

    if clone_ours {
        let Some(handle) = graph_handle(clonedGraph) else {
            return hipErrorInvalidHandle;
        };
        let mut state = lock(&handle.state);
        // Clone already has no lowered plan; keep force_native set.
        state.force_native = true;
        // Reuse an existing wrapper for the same native node if we already minted one.
        if let Some(&existing) =
            state
                .node_handles
                .iter()
                .find(|&&h| match node_snapshot(h as hipGraphNode_t) {
                    Some(s) => s.native_node == native_found as usize,
                    None => false,
                })
        {
            unsafe { *pNode = existing as hipGraphNode_t };
            return hipSuccess;
        }
        let wrapped = allocate_node(
            clonedGraph as usize,
            &mut state,
            None,
            native_found as usize,
        );
        unsafe { *pNode = wrapped };
        hipSuccess
    } else {
        unsafe { *pNode = native_found };
        hipSuccess
    }
}

/// hipGraphNodeGetEnabled — read-only (does not invalidate the PM4 plan).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphNodeGetEnabled(
    hGraphExec: hipGraphExec_t,
    hNode: hipGraphNode_t,
    isEnabled: *mut u32,
) -> hipError_t {
    type Function = unsafe extern "C" fn(hipGraphExec_t, hipGraphNode_t, *mut u32) -> hipError_t;

    if !is_exec(hGraphExec) {
        return match unsafe { real_symbol::<Function>(b"hipGraphNodeGetEnabled\0") } {
            Some(function) => unsafe {
                function(
                    crate::native_exec_or_passthrough(hGraphExec),
                    crate::native_node_or_passthrough(hNode),
                    isEnabled,
                )
            },
            None => hipErrorNotSupported,
        };
    }
    if isEnabled.is_null() {
        return hipErrorInvalidValue;
    }
    let Some(handle) = exec_handle(hGraphExec) else {
        return hipErrorInvalidHandle;
    };
    let state = lock(&handle.state);
    if state.native_exec == 0 {
        return hipErrorNotSupported;
    }
    let native_node = match exec_native_node(&state, hNode) {
        Ok(n) => n,
        Err(status) => return status,
    };
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphNodeGetEnabled\0") }) else {
        return hipErrorNotSupported;
    };
    unsafe { function(state.native_exec as hipGraphExec_t, native_node, isEnabled) }
}

/// hipGraphNodeSetEnabled — mutating (exec node enable bit).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphNodeSetEnabled(
    hGraphExec: hipGraphExec_t,
    hNode: hipGraphNode_t,
    isEnabled: u32,
) -> hipError_t {
    type Function = unsafe extern "C" fn(hipGraphExec_t, hipGraphNode_t, u32) -> hipError_t;

    if !is_exec(hGraphExec) {
        return match unsafe { real_symbol::<Function>(b"hipGraphNodeSetEnabled\0") } {
            Some(function) => unsafe {
                function(
                    crate::native_exec_or_passthrough(hGraphExec),
                    crate::native_node_or_passthrough(hNode),
                    isEnabled,
                )
            },
            None => hipErrorNotSupported,
        };
    }
    let Some(handle) = exec_handle(hGraphExec) else {
        return hipErrorInvalidHandle;
    };
    let mut state = lock(&handle.state);
    if state.native_exec == 0 {
        return hipErrorNotSupported;
    }
    let native_node = match exec_native_node(&state, hNode) {
        Ok(n) => n,
        Err(status) => return status,
    };
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphNodeSetEnabled\0") }) else {
        return hipErrorNotSupported;
    };
    let status = unsafe { function(state.native_exec as hipGraphExec_t, native_node, isEnabled) };
    if status == hipSuccess {
        state.force_native = true;
    }
    status
}

/// hipGraphNodeSetParams — mutating (graph node params).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphNodeSetParams(
    node: hipGraphNode_t,
    nodeParams: *mut c_void,
) -> hipError_t {
    type Function = unsafe extern "C" fn(hipGraphNode_t, *mut c_void) -> hipError_t;

    let mut params_keep = unsafe { translate_child_graph_params(nodeParams) };
    let fwd_params = node_params_ptr(nodeParams, &mut params_keep);

    let Some(snap) = node_snapshot(node) else {
        return match unsafe { real_symbol::<Function>(b"hipGraphNodeSetParams\0") } {
            Some(function) => unsafe {
                function(crate::native_node_or_passthrough(node), fwd_params)
            },
            None => hipErrorNotSupported,
        };
    };
    if snap.native_node == 0 {
        return hipErrorNotSupported;
    }
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphNodeSetParams\0") }) else {
        return hipErrorNotSupported;
    };
    let status = unsafe { function(snap.native_node as hipGraphNode_t, fwd_params) };
    if status == hipSuccess {
        if let Some(owner) = graph_handle(snap.owner as hipGraph_t) {
            lock(&owner.state).force_native = true;
        }
    }
    status
}

/// hipGraphRemoveDependencies — mutating; from/to elements may mix ours and native.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphRemoveDependencies(
    graph: hipGraph_t,
    from: *const hipGraphNode_t,
    to: *const hipGraphNode_t,
    numDependencies: usize,
) -> hipError_t {
    type Function = unsafe extern "C" fn(
        hipGraph_t,
        *const hipGraphNode_t,
        *const hipGraphNode_t,
        usize,
    ) -> hipError_t;

    if numDependencies > MAX_DEPENDENCY_COUNT {
        return hipErrorInvalidValue;
    }
    if !is_graph(graph) {
        let native_from =
            match unsafe { crate::translate_dependencies_passthrough(from, numDependencies) } {
                Ok(value) => value,
                Err(status) => return status,
            };
        let native_to =
            match unsafe { crate::translate_dependencies_passthrough(to, numDependencies) } {
                Ok(value) => value,
                Err(status) => return status,
            };
        let from_ptr = native_from.as_ref().map(|v| v.as_ptr()).unwrap_or(from);
        let to_ptr = native_to.as_ref().map(|v| v.as_ptr()).unwrap_or(to);
        return match unsafe { real_symbol::<Function>(b"hipGraphRemoveDependencies\0") } {
            Some(function) => unsafe { function(graph, from_ptr, to_ptr, numDependencies) },
            None => hipErrorNotSupported,
        };
    }
    if numDependencies != 0 && (from.is_null() || to.is_null()) {
        return hipErrorInvalidValue;
    }
    let Some(handle) = graph_handle(graph) else {
        return hipErrorInvalidHandle;
    };
    let mut state = lock(&handle.state);
    if state.native_graph == 0 {
        return hipErrorNotSupported;
    }

    // Element-wise passthrough; count already bounded above so the Vecs inside
    // translate_dependencies_passthrough cannot be caller-forced unbounded.
    let native_from =
        match unsafe { crate::translate_dependencies_passthrough(from, numDependencies) } {
            Ok(value) => value,
            Err(status) => return status,
        };
    let native_to = match unsafe { crate::translate_dependencies_passthrough(to, numDependencies) }
    {
        Ok(value) => value,
        Err(status) => return status,
    };
    let from_ptr = native_from.as_ref().map(|v| v.as_ptr()).unwrap_or(from);
    let to_ptr = native_to.as_ref().map(|v| v.as_ptr()).unwrap_or(to);

    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphRemoveDependencies\0") })
    else {
        return hipErrorNotSupported;
    };
    let status = unsafe {
        function(
            state.native_graph as hipGraph_t,
            from_ptr,
            to_ptr,
            numDependencies,
        )
    };
    if status == hipSuccess {
        state.force_native = true;
    }
    status
}
