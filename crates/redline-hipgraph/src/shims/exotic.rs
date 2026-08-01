// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! Translating shims for exotic / never-accelerated hipGraph node types:
//! batch mem-op, external semaphore, mem alloc/free, and user-object retain.

#![allow(non_snake_case, non_camel_case_types)]

use std::ffi::c_void;
use std::ptr;

use crate::abi::{
    hipError_t, hipErrorInvalidHandle, hipErrorInvalidValue, hipErrorNotSupported, hipGraph_t,
    hipGraphExec_t, hipGraphNode_t, hipSuccess,
};
use crate::{
    exec_handle, finish_native_only_node, graph_handle, is_exec, is_graph, lock, node_snapshot,
    real_symbol,
};

/// Opaque HIP handles / param blobs we only pass through.
type hipUserObject_t = *mut c_void;
type hipBatchMemOpNodeParams = c_void;
type hipMemAllocNodeParams = c_void;
type hipExternalSemaphoreSignalNodeParams = c_void;
type hipExternalSemaphoreWaitNodeParams = c_void;

fn mark_graph_force_native(graph: hipGraph_t) {
    if let Some(handle) = graph_handle(graph) {
        lock(&handle.state).force_native = true;
    }
}

fn mark_owner_force_native(owner: usize) {
    mark_graph_force_native(owner as hipGraph_t);
}

fn mark_exec_force_native(exec: hipGraphExec_t) {
    if let Some(handle) = exec_handle(exec) {
        lock(&handle.state).force_native = true;
    }
}

// ---------------------------------------------------------------------------
// Batch mem-op nodes
// ---------------------------------------------------------------------------

/// mutating — Add* creates a native-only node; PM4 plan is stale.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphAddBatchMemOpNode(
    phGraphNode: *mut hipGraphNode_t,
    hGraph: hipGraph_t,
    dependencies: *const hipGraphNode_t,
    numDependencies: usize,
    nodeParams: *const hipBatchMemOpNodeParams,
) -> hipError_t {
    type Function = unsafe extern "C" fn(
        *mut hipGraphNode_t,
        hipGraph_t,
        *const hipGraphNode_t,
        usize,
        *const hipBatchMemOpNodeParams,
    ) -> hipError_t;
    if !is_graph(hGraph) {
        let translated = match unsafe {
            crate::translate_dependencies_passthrough(dependencies, numDependencies)
        } {
            Ok(value) => value,
            Err(status) => return status,
        };
        let deps_ptr = translated
            .as_ref()
            .map(|d| d.as_ptr())
            .unwrap_or(dependencies);
        return match unsafe { real_symbol::<Function>(b"hipGraphAddBatchMemOpNode\0") } {
            Some(function) => unsafe {
                function(phGraphNode, hGraph, deps_ptr, numDependencies, nodeParams)
            },
            None => hipErrorNotSupported,
        };
    }
    if phGraphNode.is_null() {
        return hipErrorInvalidValue;
    }
    let Some(handle) = graph_handle(hGraph) else {
        return hipErrorInvalidHandle;
    };
    let native_graph = lock(&handle.state).native_graph;
    if native_graph == 0 {
        return hipErrorNotSupported;
    }
    let translated =
        match unsafe { crate::translate_dependencies_passthrough(dependencies, numDependencies) } {
            Ok(value) => value,
            Err(status) => return status,
        };
    let deps_ptr = translated
        .as_ref()
        .map(|d| d.as_ptr())
        .unwrap_or(dependencies);
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphAddBatchMemOpNode\0") })
    else {
        return hipErrorNotSupported;
    };
    let mut native_node = ptr::null_mut();
    let status = unsafe {
        function(
            &mut native_node,
            native_graph as hipGraph_t,
            deps_ptr,
            numDependencies,
            nodeParams,
        )
    };
    if status == hipSuccess {
        finish_native_only_node(hGraph, native_node, phGraphNode)
    } else {
        status
    }
}

/// read-only — translate node, forward params out.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphBatchMemOpNodeGetParams(
    hNode: hipGraphNode_t,
    nodeParams_out: *mut hipBatchMemOpNodeParams,
) -> hipError_t {
    type Function =
        unsafe extern "C" fn(hipGraphNode_t, *mut hipBatchMemOpNodeParams) -> hipError_t;
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphBatchMemOpNodeGetParams\0") })
    else {
        return hipErrorNotSupported;
    };
    let Some(snap) = node_snapshot(hNode) else {
        return unsafe { function(hNode, nodeParams_out) };
    };
    if nodeParams_out.is_null() {
        return hipErrorInvalidValue;
    }
    if snap.native_node == 0 {
        return hipErrorNotSupported;
    }
    unsafe { function(snap.native_node as hipGraphNode_t, nodeParams_out) }
}

/// mutating — node params change; force native on owning graph.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphBatchMemOpNodeSetParams(
    hNode: hipGraphNode_t,
    nodeParams: *mut hipBatchMemOpNodeParams,
) -> hipError_t {
    type Function =
        unsafe extern "C" fn(hipGraphNode_t, *mut hipBatchMemOpNodeParams) -> hipError_t;
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphBatchMemOpNodeSetParams\0") })
    else {
        return hipErrorNotSupported;
    };
    let Some(snap) = node_snapshot(hNode) else {
        return unsafe { function(hNode, nodeParams) };
    };
    if snap.native_node == 0 {
        return hipErrorNotSupported;
    }
    let status = unsafe { function(snap.native_node as hipGraphNode_t, nodeParams) };
    if status == hipSuccess {
        mark_owner_force_native(snap.owner);
    }
    status
}

/// mutating — exec node params change; force native on exec.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphExecBatchMemOpNodeSetParams(
    hGraphExec: hipGraphExec_t,
    hNode: hipGraphNode_t,
    nodeParams: *const hipBatchMemOpNodeParams,
) -> hipError_t {
    type Function = unsafe extern "C" fn(
        hipGraphExec_t,
        hipGraphNode_t,
        *const hipBatchMemOpNodeParams,
    ) -> hipError_t;
    let Some(function) =
        (unsafe { real_symbol::<Function>(b"hipGraphExecBatchMemOpNodeSetParams\0") })
    else {
        return hipErrorNotSupported;
    };
    if !is_exec(hGraphExec) {
        return unsafe {
            function(
                crate::native_exec_or_passthrough(hGraphExec),
                crate::native_node_or_passthrough(hNode),
                nodeParams,
            )
        };
    }
    let Some(handle) = exec_handle(hGraphExec) else {
        return hipErrorInvalidHandle;
    };
    let state = lock(&handle.state);
    if state.native_exec == 0 {
        return hipErrorNotSupported;
    }
    let native_exec = state.native_exec as hipGraphExec_t;
    let native_node = match node_snapshot(hNode) {
        Some(snap) if snap.native_node != 0 => snap.native_node as hipGraphNode_t,
        Some(_) => return hipErrorNotSupported,
        None => state
            .native_nodes
            .get(&(hNode as usize))
            .copied()
            .map(|n| n as hipGraphNode_t)
            .unwrap_or(hNode),
    };
    drop(state);
    let status = unsafe { function(native_exec, native_node, nodeParams) };
    if status == hipSuccess {
        mark_exec_force_native(hGraphExec);
    }
    status
}

// ---------------------------------------------------------------------------
// External semaphore signal / wait nodes
// ---------------------------------------------------------------------------

/// mutating — Add* creates a native-only node.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphAddExternalSemaphoresSignalNode(
    pGraphNode: *mut hipGraphNode_t,
    graph: hipGraph_t,
    pDependencies: *const hipGraphNode_t,
    numDependencies: usize,
    nodeParams: *const hipExternalSemaphoreSignalNodeParams,
) -> hipError_t {
    type Function = unsafe extern "C" fn(
        *mut hipGraphNode_t,
        hipGraph_t,
        *const hipGraphNode_t,
        usize,
        *const hipExternalSemaphoreSignalNodeParams,
    ) -> hipError_t;
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
        return match unsafe {
            real_symbol::<Function>(b"hipGraphAddExternalSemaphoresSignalNode\0")
        } {
            Some(function) => unsafe {
                function(pGraphNode, graph, deps_ptr, numDependencies, nodeParams)
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
    let Some(function) =
        (unsafe { real_symbol::<Function>(b"hipGraphAddExternalSemaphoresSignalNode\0") })
    else {
        return hipErrorNotSupported;
    };
    let mut native_node = ptr::null_mut();
    let status = unsafe {
        function(
            &mut native_node,
            native_graph as hipGraph_t,
            deps_ptr,
            numDependencies,
            nodeParams,
        )
    };
    if status == hipSuccess {
        finish_native_only_node(graph, native_node, pGraphNode)
    } else {
        status
    }
}

/// mutating — Add* creates a native-only node.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphAddExternalSemaphoresWaitNode(
    pGraphNode: *mut hipGraphNode_t,
    graph: hipGraph_t,
    pDependencies: *const hipGraphNode_t,
    numDependencies: usize,
    nodeParams: *const hipExternalSemaphoreWaitNodeParams,
) -> hipError_t {
    type Function = unsafe extern "C" fn(
        *mut hipGraphNode_t,
        hipGraph_t,
        *const hipGraphNode_t,
        usize,
        *const hipExternalSemaphoreWaitNodeParams,
    ) -> hipError_t;
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
        return match unsafe { real_symbol::<Function>(b"hipGraphAddExternalSemaphoresWaitNode\0") }
        {
            Some(function) => unsafe {
                function(pGraphNode, graph, deps_ptr, numDependencies, nodeParams)
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
    let Some(function) =
        (unsafe { real_symbol::<Function>(b"hipGraphAddExternalSemaphoresWaitNode\0") })
    else {
        return hipErrorNotSupported;
    };
    let mut native_node = ptr::null_mut();
    let status = unsafe {
        function(
            &mut native_node,
            native_graph as hipGraph_t,
            deps_ptr,
            numDependencies,
            nodeParams,
        )
    };
    if status == hipSuccess {
        finish_native_only_node(graph, native_node, pGraphNode)
    } else {
        status
    }
}

/// read-only.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphExternalSemaphoresSignalNodeGetParams(
    hNode: hipGraphNode_t,
    params_out: *mut hipExternalSemaphoreSignalNodeParams,
) -> hipError_t {
    type Function = unsafe extern "C" fn(
        hipGraphNode_t,
        *mut hipExternalSemaphoreSignalNodeParams,
    ) -> hipError_t;
    let Some(function) =
        (unsafe { real_symbol::<Function>(b"hipGraphExternalSemaphoresSignalNodeGetParams\0") })
    else {
        return hipErrorNotSupported;
    };
    let Some(snap) = node_snapshot(hNode) else {
        return unsafe { function(hNode, params_out) };
    };
    if params_out.is_null() {
        return hipErrorInvalidValue;
    }
    if snap.native_node == 0 {
        return hipErrorNotSupported;
    }
    unsafe { function(snap.native_node as hipGraphNode_t, params_out) }
}

/// read-only.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphExternalSemaphoresWaitNodeGetParams(
    hNode: hipGraphNode_t,
    params_out: *mut hipExternalSemaphoreWaitNodeParams,
) -> hipError_t {
    type Function =
        unsafe extern "C" fn(hipGraphNode_t, *mut hipExternalSemaphoreWaitNodeParams) -> hipError_t;
    let Some(function) =
        (unsafe { real_symbol::<Function>(b"hipGraphExternalSemaphoresWaitNodeGetParams\0") })
    else {
        return hipErrorNotSupported;
    };
    let Some(snap) = node_snapshot(hNode) else {
        return unsafe { function(hNode, params_out) };
    };
    if params_out.is_null() {
        return hipErrorInvalidValue;
    }
    if snap.native_node == 0 {
        return hipErrorNotSupported;
    }
    unsafe { function(snap.native_node as hipGraphNode_t, params_out) }
}

/// mutating.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphExternalSemaphoresSignalNodeSetParams(
    hNode: hipGraphNode_t,
    nodeParams: *const hipExternalSemaphoreSignalNodeParams,
) -> hipError_t {
    type Function = unsafe extern "C" fn(
        hipGraphNode_t,
        *const hipExternalSemaphoreSignalNodeParams,
    ) -> hipError_t;
    let Some(function) =
        (unsafe { real_symbol::<Function>(b"hipGraphExternalSemaphoresSignalNodeSetParams\0") })
    else {
        return hipErrorNotSupported;
    };
    let Some(snap) = node_snapshot(hNode) else {
        return unsafe { function(hNode, nodeParams) };
    };
    if snap.native_node == 0 {
        return hipErrorNotSupported;
    }
    let status = unsafe { function(snap.native_node as hipGraphNode_t, nodeParams) };
    if status == hipSuccess {
        mark_owner_force_native(snap.owner);
    }
    status
}

/// mutating.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphExternalSemaphoresWaitNodeSetParams(
    hNode: hipGraphNode_t,
    nodeParams: *const hipExternalSemaphoreWaitNodeParams,
) -> hipError_t {
    type Function = unsafe extern "C" fn(
        hipGraphNode_t,
        *const hipExternalSemaphoreWaitNodeParams,
    ) -> hipError_t;
    let Some(function) =
        (unsafe { real_symbol::<Function>(b"hipGraphExternalSemaphoresWaitNodeSetParams\0") })
    else {
        return hipErrorNotSupported;
    };
    let Some(snap) = node_snapshot(hNode) else {
        return unsafe { function(hNode, nodeParams) };
    };
    if snap.native_node == 0 {
        return hipErrorNotSupported;
    }
    let status = unsafe { function(snap.native_node as hipGraphNode_t, nodeParams) };
    if status == hipSuccess {
        mark_owner_force_native(snap.owner);
    }
    status
}

/// mutating — exec node params.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphExecExternalSemaphoresSignalNodeSetParams(
    hGraphExec: hipGraphExec_t,
    hNode: hipGraphNode_t,
    nodeParams: *const hipExternalSemaphoreSignalNodeParams,
) -> hipError_t {
    type Function = unsafe extern "C" fn(
        hipGraphExec_t,
        hipGraphNode_t,
        *const hipExternalSemaphoreSignalNodeParams,
    ) -> hipError_t;
    let Some(function) = (unsafe {
        real_symbol::<Function>(b"hipGraphExecExternalSemaphoresSignalNodeSetParams\0")
    }) else {
        return hipErrorNotSupported;
    };
    if !is_exec(hGraphExec) {
        return unsafe {
            function(
                crate::native_exec_or_passthrough(hGraphExec),
                crate::native_node_or_passthrough(hNode),
                nodeParams,
            )
        };
    }
    let Some(handle) = exec_handle(hGraphExec) else {
        return hipErrorInvalidHandle;
    };
    let state = lock(&handle.state);
    if state.native_exec == 0 {
        return hipErrorNotSupported;
    }
    let native_exec = state.native_exec as hipGraphExec_t;
    let native_node = match node_snapshot(hNode) {
        Some(snap) if snap.native_node != 0 => snap.native_node as hipGraphNode_t,
        Some(_) => return hipErrorNotSupported,
        None => state
            .native_nodes
            .get(&(hNode as usize))
            .copied()
            .map(|n| n as hipGraphNode_t)
            .unwrap_or(hNode),
    };
    drop(state);
    let status = unsafe { function(native_exec, native_node, nodeParams) };
    if status == hipSuccess {
        mark_exec_force_native(hGraphExec);
    }
    status
}

/// mutating — exec node params.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphExecExternalSemaphoresWaitNodeSetParams(
    hGraphExec: hipGraphExec_t,
    hNode: hipGraphNode_t,
    nodeParams: *const hipExternalSemaphoreWaitNodeParams,
) -> hipError_t {
    type Function = unsafe extern "C" fn(
        hipGraphExec_t,
        hipGraphNode_t,
        *const hipExternalSemaphoreWaitNodeParams,
    ) -> hipError_t;
    let Some(function) =
        (unsafe { real_symbol::<Function>(b"hipGraphExecExternalSemaphoresWaitNodeSetParams\0") })
    else {
        return hipErrorNotSupported;
    };
    if !is_exec(hGraphExec) {
        return unsafe {
            function(
                crate::native_exec_or_passthrough(hGraphExec),
                crate::native_node_or_passthrough(hNode),
                nodeParams,
            )
        };
    }
    let Some(handle) = exec_handle(hGraphExec) else {
        return hipErrorInvalidHandle;
    };
    let state = lock(&handle.state);
    if state.native_exec == 0 {
        return hipErrorNotSupported;
    }
    let native_exec = state.native_exec as hipGraphExec_t;
    let native_node = match node_snapshot(hNode) {
        Some(snap) if snap.native_node != 0 => snap.native_node as hipGraphNode_t,
        Some(_) => return hipErrorNotSupported,
        None => state
            .native_nodes
            .get(&(hNode as usize))
            .copied()
            .map(|n| n as hipGraphNode_t)
            .unwrap_or(hNode),
    };
    drop(state);
    let status = unsafe { function(native_exec, native_node, nodeParams) };
    if status == hipSuccess {
        mark_exec_force_native(hGraphExec);
    }
    status
}

// ---------------------------------------------------------------------------
// Mem alloc / free nodes
// ---------------------------------------------------------------------------

/// mutating — Add* creates a native-only node.
/// `pNodeParams` is non-const: HIP writes the allocated device pointer back.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphAddMemAllocNode(
    pGraphNode: *mut hipGraphNode_t,
    graph: hipGraph_t,
    pDependencies: *const hipGraphNode_t,
    numDependencies: usize,
    pNodeParams: *mut hipMemAllocNodeParams,
) -> hipError_t {
    type Function = unsafe extern "C" fn(
        *mut hipGraphNode_t,
        hipGraph_t,
        *const hipGraphNode_t,
        usize,
        *mut hipMemAllocNodeParams,
    ) -> hipError_t;
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
        return match unsafe { real_symbol::<Function>(b"hipGraphAddMemAllocNode\0") } {
            Some(function) => unsafe {
                function(pGraphNode, graph, deps_ptr, numDependencies, pNodeParams)
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
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphAddMemAllocNode\0") }) else {
        return hipErrorNotSupported;
    };
    let mut native_node = ptr::null_mut();
    let status = unsafe {
        function(
            &mut native_node,
            native_graph as hipGraph_t,
            deps_ptr,
            numDependencies,
            pNodeParams,
        )
    };
    if status == hipSuccess {
        finish_native_only_node(graph, native_node, pGraphNode)
    } else {
        status
    }
}

/// read-only.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphMemAllocNodeGetParams(
    node: hipGraphNode_t,
    pNodeParams: *mut hipMemAllocNodeParams,
) -> hipError_t {
    type Function = unsafe extern "C" fn(hipGraphNode_t, *mut hipMemAllocNodeParams) -> hipError_t;
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphMemAllocNodeGetParams\0") })
    else {
        return hipErrorNotSupported;
    };
    let Some(snap) = node_snapshot(node) else {
        return unsafe { function(node, pNodeParams) };
    };
    if pNodeParams.is_null() {
        return hipErrorInvalidValue;
    }
    if snap.native_node == 0 {
        return hipErrorNotSupported;
    }
    unsafe { function(snap.native_node as hipGraphNode_t, pNodeParams) }
}

/// mutating — Add* creates a native-only node.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphAddMemFreeNode(
    pGraphNode: *mut hipGraphNode_t,
    graph: hipGraph_t,
    pDependencies: *const hipGraphNode_t,
    numDependencies: usize,
    dev_ptr: *mut c_void,
) -> hipError_t {
    type Function = unsafe extern "C" fn(
        *mut hipGraphNode_t,
        hipGraph_t,
        *const hipGraphNode_t,
        usize,
        *mut c_void,
    ) -> hipError_t;
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
        return match unsafe { real_symbol::<Function>(b"hipGraphAddMemFreeNode\0") } {
            Some(function) => unsafe {
                function(pGraphNode, graph, deps_ptr, numDependencies, dev_ptr)
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
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphAddMemFreeNode\0") }) else {
        return hipErrorNotSupported;
    };
    let mut native_node = ptr::null_mut();
    let status = unsafe {
        function(
            &mut native_node,
            native_graph as hipGraph_t,
            deps_ptr,
            numDependencies,
            dev_ptr,
        )
    };
    if status == hipSuccess {
        finish_native_only_node(graph, native_node, pGraphNode)
    } else {
        status
    }
}

/// read-only — `dev_ptr` is an out-slot for the device pointer value.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphMemFreeNodeGetParams(
    node: hipGraphNode_t,
    dev_ptr: *mut c_void,
) -> hipError_t {
    type Function = unsafe extern "C" fn(hipGraphNode_t, *mut c_void) -> hipError_t;
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphMemFreeNodeGetParams\0") })
    else {
        return hipErrorNotSupported;
    };
    let Some(snap) = node_snapshot(node) else {
        return unsafe { function(node, dev_ptr) };
    };
    if dev_ptr.is_null() {
        return hipErrorInvalidValue;
    }
    if snap.native_node == 0 {
        return hipErrorNotSupported;
    }
    unsafe { function(snap.native_node as hipGraphNode_t, dev_ptr) }
}

// ---------------------------------------------------------------------------
// User object retain / release on a graph
// ---------------------------------------------------------------------------

/// mutating — graph ownership of a user object changes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphRetainUserObject(
    graph: hipGraph_t,
    object: hipUserObject_t,
    count: u32,
    flags: u32,
) -> hipError_t {
    type Function = unsafe extern "C" fn(hipGraph_t, hipUserObject_t, u32, u32) -> hipError_t;
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphRetainUserObject\0") }) else {
        return hipErrorNotSupported;
    };
    if !is_graph(graph) {
        return unsafe { function(graph, object, count, flags) };
    }
    let Some(handle) = graph_handle(graph) else {
        return hipErrorInvalidHandle;
    };
    let native_graph = lock(&handle.state).native_graph;
    if native_graph == 0 {
        return hipErrorNotSupported;
    }
    let status = unsafe { function(native_graph as hipGraph_t, object, count, flags) };
    if status == hipSuccess {
        mark_graph_force_native(graph);
    }
    status
}

/// mutating — graph ownership of a user object changes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphReleaseUserObject(
    graph: hipGraph_t,
    object: hipUserObject_t,
    count: u32,
) -> hipError_t {
    type Function = unsafe extern "C" fn(hipGraph_t, hipUserObject_t, u32) -> hipError_t;
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphReleaseUserObject\0") })
    else {
        return hipErrorNotSupported;
    };
    if !is_graph(graph) {
        return unsafe { function(graph, object, count) };
    }
    let Some(handle) = graph_handle(graph) else {
        return hipErrorInvalidHandle;
    };
    let native_graph = lock(&handle.state).native_graph;
    if native_graph == 0 {
        return hipErrorNotSupported;
    }
    let status = unsafe { function(native_graph as hipGraph_t, object, count) };
    if status == hipSuccess {
        mark_graph_force_native(graph);
    }
    status
}
