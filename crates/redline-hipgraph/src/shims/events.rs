// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! Translating shims for hipGraph event record/wait nodes.
//!
//! `hipEvent_t` is a native HIP handle — never translate it. Only graph, node,
//! and exec handles need translation.

#![allow(non_snake_case, non_camel_case_types)]

use std::ffi::c_void;
use std::ptr;

use crate::abi::{
    hipError_t, hipErrorInvalidHandle, hipErrorInvalidValue, hipErrorNotSupported, hipGraph_t,
    hipGraphExec_t, hipGraphNode_t, hipSuccess,
};
use crate::{
    exec_handle, finish_native_only_node, graph_handle, is_exec, is_graph, lock,
    native_exec_or_passthrough, native_graph_or_passthrough, native_node_or_passthrough,
    node_snapshot, real_symbol, translate_dependencies_passthrough,
};

/// Native HIP event handle — opaque to us; pass straight through.
type hipEvent_t = *mut c_void;

/// Mutating — creates a node; uses `finish_native_only_node` (sets force_native).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphAddEventRecordNode(
    pGraphNode: *mut hipGraphNode_t,
    graph: hipGraph_t,
    pDependencies: *const hipGraphNode_t,
    numDependencies: usize,
    event: hipEvent_t,
) -> hipError_t {
    type Function = unsafe extern "C" fn(
        *mut hipGraphNode_t,
        hipGraph_t,
        *const hipGraphNode_t,
        usize,
        hipEvent_t,
    ) -> hipError_t;

    if !is_graph(graph) {
        return match unsafe { real_symbol::<Function>(b"hipGraphAddEventRecordNode\0") } {
            Some(function) => {
                let translated = match unsafe {
                    translate_dependencies_passthrough(pDependencies, numDependencies)
                } {
                    Ok(value) => value,
                    Err(status) => return status,
                };
                let deps = translated
                    .as_ref()
                    .map(|deps| deps.as_ptr())
                    .unwrap_or(pDependencies);
                unsafe {
                    function(
                        pGraphNode,
                        native_graph_or_passthrough(graph),
                        deps,
                        numDependencies,
                        event,
                    )
                }
            }
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
    let translated =
        match unsafe { translate_dependencies_passthrough(pDependencies, numDependencies) } {
            Ok(value) => value,
            Err(status) => return status,
        };
    let deps = translated
        .as_ref()
        .map(|deps| deps.as_ptr())
        .unwrap_or(pDependencies);
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphAddEventRecordNode\0") })
    else {
        return hipErrorNotSupported;
    };
    let mut native_node = ptr::null_mut();
    let status = unsafe {
        function(
            &mut native_node,
            native_graph as hipGraph_t,
            deps,
            numDependencies,
            event,
        )
    };
    if status == hipSuccess {
        finish_native_only_node(graph, native_node, pGraphNode)
    } else {
        status
    }
}

/// Mutating — creates a node; uses `finish_native_only_node` (sets force_native).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphAddEventWaitNode(
    pGraphNode: *mut hipGraphNode_t,
    graph: hipGraph_t,
    pDependencies: *const hipGraphNode_t,
    numDependencies: usize,
    event: hipEvent_t,
) -> hipError_t {
    type Function = unsafe extern "C" fn(
        *mut hipGraphNode_t,
        hipGraph_t,
        *const hipGraphNode_t,
        usize,
        hipEvent_t,
    ) -> hipError_t;

    if !is_graph(graph) {
        return match unsafe { real_symbol::<Function>(b"hipGraphAddEventWaitNode\0") } {
            Some(function) => {
                let translated = match unsafe {
                    translate_dependencies_passthrough(pDependencies, numDependencies)
                } {
                    Ok(value) => value,
                    Err(status) => return status,
                };
                let deps = translated
                    .as_ref()
                    .map(|deps| deps.as_ptr())
                    .unwrap_or(pDependencies);
                unsafe {
                    function(
                        pGraphNode,
                        native_graph_or_passthrough(graph),
                        deps,
                        numDependencies,
                        event,
                    )
                }
            }
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
    let translated =
        match unsafe { translate_dependencies_passthrough(pDependencies, numDependencies) } {
            Ok(value) => value,
            Err(status) => return status,
        };
    let deps = translated
        .as_ref()
        .map(|deps| deps.as_ptr())
        .unwrap_or(pDependencies);
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphAddEventWaitNode\0") }) else {
        return hipErrorNotSupported;
    };
    let mut native_node = ptr::null_mut();
    let status = unsafe {
        function(
            &mut native_node,
            native_graph as hipGraph_t,
            deps,
            numDependencies,
            event,
        )
    };
    if status == hipSuccess {
        finish_native_only_node(graph, native_node, pGraphNode)
    } else {
        status
    }
}

/// Read-only — translate node, forward; do not set force_native.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphEventRecordNodeGetEvent(
    node: hipGraphNode_t,
    event_out: *mut hipEvent_t,
) -> hipError_t {
    type Function = unsafe extern "C" fn(hipGraphNode_t, *mut hipEvent_t) -> hipError_t;

    let Some(snapshot) = node_snapshot(node) else {
        return match unsafe { real_symbol::<Function>(b"hipGraphEventRecordNodeGetEvent\0") } {
            Some(function) => unsafe { function(node, event_out) },
            None => hipErrorNotSupported,
        };
    };
    if event_out.is_null() {
        return hipErrorInvalidValue;
    }
    if snapshot.native_node == 0 {
        return hipErrorNotSupported;
    }
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphEventRecordNodeGetEvent\0") })
    else {
        return hipErrorNotSupported;
    };
    unsafe { function(snapshot.native_node as hipGraphNode_t, event_out) }
}

/// Mutating — translate node, forward, force_native on owning graph.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphEventRecordNodeSetEvent(
    node: hipGraphNode_t,
    event: hipEvent_t,
) -> hipError_t {
    type Function = unsafe extern "C" fn(hipGraphNode_t, hipEvent_t) -> hipError_t;

    let Some(snapshot) = node_snapshot(node) else {
        return match unsafe { real_symbol::<Function>(b"hipGraphEventRecordNodeSetEvent\0") } {
            Some(function) => unsafe { function(node, event) },
            None => hipErrorNotSupported,
        };
    };
    if snapshot.native_node == 0 {
        return hipErrorNotSupported;
    }
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphEventRecordNodeSetEvent\0") })
    else {
        return hipErrorNotSupported;
    };
    let status = unsafe { function(snapshot.native_node as hipGraphNode_t, event) };
    if status == hipSuccess {
        if let Some(handle) = graph_handle(snapshot.owner as hipGraph_t) {
            lock(&handle.state).force_native = true;
        }
    }
    status
}

/// Read-only — translate node, forward; do not set force_native.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphEventWaitNodeGetEvent(
    node: hipGraphNode_t,
    event_out: *mut hipEvent_t,
) -> hipError_t {
    type Function = unsafe extern "C" fn(hipGraphNode_t, *mut hipEvent_t) -> hipError_t;

    let Some(snapshot) = node_snapshot(node) else {
        return match unsafe { real_symbol::<Function>(b"hipGraphEventWaitNodeGetEvent\0") } {
            Some(function) => unsafe { function(node, event_out) },
            None => hipErrorNotSupported,
        };
    };
    if event_out.is_null() {
        return hipErrorInvalidValue;
    }
    if snapshot.native_node == 0 {
        return hipErrorNotSupported;
    }
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphEventWaitNodeGetEvent\0") })
    else {
        return hipErrorNotSupported;
    };
    unsafe { function(snapshot.native_node as hipGraphNode_t, event_out) }
}

/// Mutating — translate node, forward, force_native on owning graph.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphEventWaitNodeSetEvent(
    node: hipGraphNode_t,
    event: hipEvent_t,
) -> hipError_t {
    type Function = unsafe extern "C" fn(hipGraphNode_t, hipEvent_t) -> hipError_t;

    let Some(snapshot) = node_snapshot(node) else {
        return match unsafe { real_symbol::<Function>(b"hipGraphEventWaitNodeSetEvent\0") } {
            Some(function) => unsafe { function(node, event) },
            None => hipErrorNotSupported,
        };
    };
    if snapshot.native_node == 0 {
        return hipErrorNotSupported;
    }
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphEventWaitNodeSetEvent\0") })
    else {
        return hipErrorNotSupported;
    };
    let status = unsafe { function(snapshot.native_node as hipGraphNode_t, event) };
    if status == hipSuccess {
        if let Some(handle) = graph_handle(snapshot.owner as hipGraph_t) {
            lock(&handle.state).force_native = true;
        }
    }
    status
}

/// Mutating — translate exec + node, forward, force_native on exec state.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphExecEventRecordNodeSetEvent(
    hGraphExec: hipGraphExec_t,
    hNode: hipGraphNode_t,
    event: hipEvent_t,
) -> hipError_t {
    type Function = unsafe extern "C" fn(hipGraphExec_t, hipGraphNode_t, hipEvent_t) -> hipError_t;

    if !is_exec(hGraphExec) {
        return match unsafe { real_symbol::<Function>(b"hipGraphExecEventRecordNodeSetEvent\0") } {
            Some(function) => unsafe {
                function(
                    native_exec_or_passthrough(hGraphExec),
                    native_node_or_passthrough(hNode),
                    event,
                )
            },
            None => hipErrorNotSupported,
        };
    }
    let Some(exec) = exec_handle(hGraphExec) else {
        return hipErrorInvalidHandle;
    };
    let native_node = match node_snapshot(hNode) {
        Some(snapshot) => {
            if snapshot.native_node == 0 {
                return hipErrorNotSupported;
            }
            snapshot.native_node as hipGraphNode_t
        }
        None => hNode,
    };
    let native_exec = {
        let state = lock(&exec.state);
        if state.native_exec == 0 {
            return hipErrorNotSupported;
        }
        state.native_exec as hipGraphExec_t
    };
    let Some(function) =
        (unsafe { real_symbol::<Function>(b"hipGraphExecEventRecordNodeSetEvent\0") })
    else {
        return hipErrorNotSupported;
    };
    let status = unsafe { function(native_exec, native_node, event) };
    if status == hipSuccess {
        lock(&exec.state).force_native = true;
    }
    status
}

/// Mutating — translate exec + node, forward, force_native on exec state.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphExecEventWaitNodeSetEvent(
    hGraphExec: hipGraphExec_t,
    hNode: hipGraphNode_t,
    event: hipEvent_t,
) -> hipError_t {
    type Function = unsafe extern "C" fn(hipGraphExec_t, hipGraphNode_t, hipEvent_t) -> hipError_t;

    if !is_exec(hGraphExec) {
        return match unsafe { real_symbol::<Function>(b"hipGraphExecEventWaitNodeSetEvent\0") } {
            Some(function) => unsafe {
                function(
                    native_exec_or_passthrough(hGraphExec),
                    native_node_or_passthrough(hNode),
                    event,
                )
            },
            None => hipErrorNotSupported,
        };
    }
    let Some(exec) = exec_handle(hGraphExec) else {
        return hipErrorInvalidHandle;
    };
    let native_node = match node_snapshot(hNode) {
        Some(snapshot) => {
            if snapshot.native_node == 0 {
                return hipErrorNotSupported;
            }
            snapshot.native_node as hipGraphNode_t
        }
        None => hNode,
    };
    let native_exec = {
        let state = lock(&exec.state);
        if state.native_exec == 0 {
            return hipErrorNotSupported;
        }
        state.native_exec as hipGraphExec_t
    };
    let Some(function) =
        (unsafe { real_symbol::<Function>(b"hipGraphExecEventWaitNodeSetEvent\0") })
    else {
        return hipErrorNotSupported;
    };
    let status = unsafe { function(native_exec, native_node, event) };
    if status == hipSuccess {
        lock(&exec.state).force_native = true;
    }
    status
}
