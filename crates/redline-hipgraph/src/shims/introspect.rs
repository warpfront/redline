// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

#![allow(non_camel_case_types, non_snake_case)]
//! Read-only hipGraph introspection shims.
//!
//! These entry points never mutate the graph, so they translate handles and
//! forward without setting `force_native`. Callers (including llama.cpp via
//! `cudaGraphGetNodes` / `cudaGraphNodeGetType`) receive our node handles back
//! so subsequent calls stay on the interposer path.

use std::ffi::c_char;

use crate::abi::{
    hipError_t, hipErrorInvalidHandle, hipErrorInvalidValue, hipErrorNotSupported, hipGraph_t,
    hipGraphNode_t, hipSuccess,
};
use crate::{graph_handle, is_graph, lock, node_snapshot, real_symbol};

/// C `hipGraphNodeType` is a plain enum; size matches `i32` on this ABI.
type hipGraphNodeType = i32;

/// Rewrite `count` slots written by HIP from native node pointers to ours.
///
/// Uses `intern_native_node` so every non-null slot becomes one of our
/// `NodeHandle` wrappers (reused if already tracked, minted once otherwise).
/// Never leaves a raw native pointer in the caller's buffer.
unsafe fn rewrite_node_array(graph: hipGraph_t, nodes: *mut hipGraphNode_t, count: usize) {
    if nodes.is_null() {
        return;
    }
    for i in 0..count {
        let slot = unsafe { nodes.add(i) };
        let native = unsafe { *slot };
        unsafe {
            *slot = crate::intern_native_node(graph, native);
        }
    }
}

/// hipGraphDebugDotPrint — read-only. Translate graph handle only.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphDebugDotPrint(
    graph: hipGraph_t,
    path: *const c_char,
    flags: u32,
) -> hipError_t {
    type Function = unsafe extern "C" fn(hipGraph_t, *const c_char, u32) -> hipError_t;
    if !is_graph(graph) {
        return match unsafe { real_symbol::<Function>(b"hipGraphDebugDotPrint\0") } {
            Some(function) => unsafe { function(graph, path, flags) },
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
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphDebugDotPrint\0") }) else {
        return hipErrorNotSupported;
    };
    unsafe { function(native_graph as hipGraph_t, path, flags) }
}

/// hipGraphGetEdges — read-only. Two-call protocol on `from`/`to`/`numEdges`.
///
/// When `from` and `to` are null, only the edge count is reported. When non-null,
/// HIP fills up to the capacity in `*numEdges` and writes back the actual count;
/// both endpoint arrays are then translated native→ours.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphGetEdges(
    graph: hipGraph_t,
    from: *mut hipGraphNode_t,
    to: *mut hipGraphNode_t,
    numEdges: *mut usize,
) -> hipError_t {
    type Function = unsafe extern "C" fn(
        hipGraph_t,
        *mut hipGraphNode_t,
        *mut hipGraphNode_t,
        *mut usize,
    ) -> hipError_t;
    if !is_graph(graph) {
        return match unsafe { real_symbol::<Function>(b"hipGraphGetEdges\0") } {
            Some(function) => unsafe { function(graph, from, to, numEdges) },
            None => hipErrorNotSupported,
        };
    }
    if numEdges.is_null() {
        return hipErrorInvalidValue;
    }
    let Some(handle) = graph_handle(graph) else {
        return hipErrorInvalidHandle;
    };
    let native_graph = lock(&handle.state).native_graph;
    if native_graph == 0 {
        return hipErrorNotSupported;
    }
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphGetEdges\0") }) else {
        return hipErrorNotSupported;
    };
    let status = unsafe { function(native_graph as hipGraph_t, from, to, numEdges) };
    // Null array pointers: count-only query; nothing to rewrite.
    if status != hipSuccess || (from.is_null() && to.is_null()) {
        return status;
    }
    let count = unsafe { *numEdges };
    unsafe {
        rewrite_node_array(graph, from, count);
        rewrite_node_array(graph, to, count);
    }
    status
}

/// hipGraphGetNodes — read-only. Two-call protocol on `nodes`/`numNodes`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphGetNodes(
    graph: hipGraph_t,
    nodes: *mut hipGraphNode_t,
    numNodes: *mut usize,
) -> hipError_t {
    type Function = unsafe extern "C" fn(hipGraph_t, *mut hipGraphNode_t, *mut usize) -> hipError_t;
    if !is_graph(graph) {
        return match unsafe { real_symbol::<Function>(b"hipGraphGetNodes\0") } {
            Some(function) => unsafe { function(graph, nodes, numNodes) },
            None => hipErrorNotSupported,
        };
    }
    if numNodes.is_null() {
        return hipErrorInvalidValue;
    }
    let Some(handle) = graph_handle(graph) else {
        return hipErrorInvalidHandle;
    };
    let native_graph = lock(&handle.state).native_graph;
    if native_graph == 0 {
        return hipErrorNotSupported;
    }
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphGetNodes\0") }) else {
        return hipErrorNotSupported;
    };
    let status = unsafe { function(native_graph as hipGraph_t, nodes, numNodes) };
    if status != hipSuccess || nodes.is_null() {
        return status;
    }
    let count = unsafe { *numNodes };
    unsafe {
        rewrite_node_array(graph, nodes, count);
    }
    status
}

/// hipGraphGetRootNodes — read-only. Two-call protocol on `pRootNodes`/`pNumRootNodes`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphGetRootNodes(
    graph: hipGraph_t,
    pRootNodes: *mut hipGraphNode_t,
    pNumRootNodes: *mut usize,
) -> hipError_t {
    type Function = unsafe extern "C" fn(hipGraph_t, *mut hipGraphNode_t, *mut usize) -> hipError_t;
    if !is_graph(graph) {
        return match unsafe { real_symbol::<Function>(b"hipGraphGetRootNodes\0") } {
            Some(function) => unsafe { function(graph, pRootNodes, pNumRootNodes) },
            None => hipErrorNotSupported,
        };
    }
    if pNumRootNodes.is_null() {
        return hipErrorInvalidValue;
    }
    let Some(handle) = graph_handle(graph) else {
        return hipErrorInvalidHandle;
    };
    let native_graph = lock(&handle.state).native_graph;
    if native_graph == 0 {
        return hipErrorNotSupported;
    }
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphGetRootNodes\0") }) else {
        return hipErrorNotSupported;
    };
    let status = unsafe { function(native_graph as hipGraph_t, pRootNodes, pNumRootNodes) };
    if status != hipSuccess || pRootNodes.is_null() {
        return status;
    }
    let count = unsafe { *pNumRootNodes };
    unsafe {
        rewrite_node_array(graph, pRootNodes, count);
    }
    status
}

/// hipGraphNodeGetDependencies — read-only. Two-call protocol.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphNodeGetDependencies(
    node: hipGraphNode_t,
    pDependencies: *mut hipGraphNode_t,
    pNumDependencies: *mut usize,
) -> hipError_t {
    type Function =
        unsafe extern "C" fn(hipGraphNode_t, *mut hipGraphNode_t, *mut usize) -> hipError_t;
    let Some(snap) = node_snapshot(node) else {
        return match unsafe { real_symbol::<Function>(b"hipGraphNodeGetDependencies\0") } {
            Some(function) => unsafe { function(node, pDependencies, pNumDependencies) },
            None => hipErrorNotSupported,
        };
    };
    if pNumDependencies.is_null() {
        return hipErrorInvalidValue;
    }
    if snap.native_node == 0 {
        return hipErrorNotSupported;
    }
    if graph_handle(snap.owner as hipGraph_t).is_none() {
        return hipErrorInvalidHandle;
    }
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphNodeGetDependencies\0") })
    else {
        return hipErrorNotSupported;
    };
    let status = unsafe {
        function(
            snap.native_node as hipGraphNode_t,
            pDependencies,
            pNumDependencies,
        )
    };
    if status != hipSuccess || pDependencies.is_null() {
        return status;
    }
    let count = unsafe { *pNumDependencies };
    let owner = snap.owner as hipGraph_t;
    unsafe {
        rewrite_node_array(owner, pDependencies, count);
    }
    status
}

/// hipGraphNodeGetDependentNodes — read-only. Two-call protocol.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphNodeGetDependentNodes(
    node: hipGraphNode_t,
    pDependentNodes: *mut hipGraphNode_t,
    pNumDependentNodes: *mut usize,
) -> hipError_t {
    type Function =
        unsafe extern "C" fn(hipGraphNode_t, *mut hipGraphNode_t, *mut usize) -> hipError_t;
    let Some(snap) = node_snapshot(node) else {
        return match unsafe { real_symbol::<Function>(b"hipGraphNodeGetDependentNodes\0") } {
            Some(function) => unsafe { function(node, pDependentNodes, pNumDependentNodes) },
            None => hipErrorNotSupported,
        };
    };
    if pNumDependentNodes.is_null() {
        return hipErrorInvalidValue;
    }
    if snap.native_node == 0 {
        return hipErrorNotSupported;
    }
    if graph_handle(snap.owner as hipGraph_t).is_none() {
        return hipErrorInvalidHandle;
    }
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphNodeGetDependentNodes\0") })
    else {
        return hipErrorNotSupported;
    };
    let status = unsafe {
        function(
            snap.native_node as hipGraphNode_t,
            pDependentNodes,
            pNumDependentNodes,
        )
    };
    if status != hipSuccess || pDependentNodes.is_null() {
        return status;
    }
    let count = unsafe { *pNumDependentNodes };
    let owner = snap.owner as hipGraph_t;
    unsafe {
        rewrite_node_array(owner, pDependentNodes, count);
    }
    status
}

/// hipGraphNodeGetType — read-only. Translate node handle only.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphNodeGetType(
    node: hipGraphNode_t,
    pType: *mut hipGraphNodeType,
) -> hipError_t {
    type Function = unsafe extern "C" fn(hipGraphNode_t, *mut hipGraphNodeType) -> hipError_t;
    let Some(snap) = node_snapshot(node) else {
        return match unsafe { real_symbol::<Function>(b"hipGraphNodeGetType\0") } {
            Some(function) => unsafe { function(node, pType) },
            None => hipErrorNotSupported,
        };
    };
    if pType.is_null() {
        return hipErrorInvalidValue;
    }
    if snap.native_node == 0 {
        return hipErrorNotSupported;
    }
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphNodeGetType\0") }) else {
        return hipErrorNotSupported;
    };
    unsafe { function(snap.native_node as hipGraphNode_t, pType) }
}
