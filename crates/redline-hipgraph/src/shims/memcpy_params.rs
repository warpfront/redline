// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! Translating shims for hipGraph memcpy-node entry points.

#![allow(non_camel_case_types, non_snake_case)]

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

/// C `enum hipMemcpyKind` — ABI is a signed 32-bit enum. Values are only
/// forwarded; we never interpret them.
type hipMemcpyKind = i32;

/// Opaque stand-in for `hipMemcpy3DParms`. Every shim only passes the pointer
/// through to the real runtime, so the pointee layout is irrelevant here.
pub(crate) enum hipMemcpy3DParms {}

/// Mark the graph that owns `node` as force-native after a mutating node op.
fn force_native_owner(owner: usize) {
    if let Some(handle) = graph_handle(owner as hipGraph_t) {
        lock(&handle.state).force_native = true;
    }
}

// ---------------------------------------------------------------------------
// Add* — create a native-only node (mutating; finish_native_only_node sets
// force_native).
// ---------------------------------------------------------------------------

/// mutating
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphAddMemcpyNode1D(
    pGraphNode: *mut hipGraphNode_t,
    graph: hipGraph_t,
    pDependencies: *const hipGraphNode_t,
    numDependencies: usize,
    dst: *mut c_void,
    src: *const c_void,
    count: usize,
    kind: hipMemcpyKind,
) -> hipError_t {
    type Function = unsafe extern "C" fn(
        *mut hipGraphNode_t,
        hipGraph_t,
        *const hipGraphNode_t,
        usize,
        *mut c_void,
        *const c_void,
        usize,
        hipMemcpyKind,
    ) -> hipError_t;

    if !is_graph(graph) {
        return match unsafe { real_symbol::<Function>(b"hipGraphAddMemcpyNode1D\0") } {
            Some(function) => {
                let translated = match unsafe {
                    crate::translate_dependencies_passthrough(pDependencies, numDependencies)
                } {
                    Ok(value) => value,
                    Err(status) => return status,
                };
                let deps_ptr = translated
                    .as_ref()
                    .map(|v| v.as_ptr())
                    .unwrap_or(pDependencies);
                unsafe {
                    function(
                        pGraphNode,
                        crate::native_graph_or_passthrough(graph),
                        deps_ptr,
                        numDependencies,
                        dst,
                        src,
                        count,
                        kind,
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
    let translated = match unsafe {
        crate::translate_dependencies_passthrough(pDependencies, numDependencies)
    } {
        Ok(value) => value,
        Err(status) => return status,
    };
    let deps_ptr = translated
        .as_ref()
        .map(|v| v.as_ptr())
        .unwrap_or(pDependencies);
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphAddMemcpyNode1D\0") }) else {
        return hipErrorNotSupported;
    };
    let mut native_node = ptr::null_mut();
    let status = unsafe {
        function(
            &mut native_node,
            native_graph as hipGraph_t,
            deps_ptr,
            numDependencies,
            dst,
            src,
            count,
            kind,
        )
    };
    if status == hipSuccess {
        finish_native_only_node(graph, native_node, pGraphNode)
    } else {
        status
    }
}

/// mutating
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphAddMemcpyNodeFromSymbol(
    pGraphNode: *mut hipGraphNode_t,
    graph: hipGraph_t,
    pDependencies: *const hipGraphNode_t,
    numDependencies: usize,
    dst: *mut c_void,
    symbol: *const c_void,
    count: usize,
    offset: usize,
    kind: hipMemcpyKind,
) -> hipError_t {
    type Function = unsafe extern "C" fn(
        *mut hipGraphNode_t,
        hipGraph_t,
        *const hipGraphNode_t,
        usize,
        *mut c_void,
        *const c_void,
        usize,
        usize,
        hipMemcpyKind,
    ) -> hipError_t;

    if !is_graph(graph) {
        return match unsafe { real_symbol::<Function>(b"hipGraphAddMemcpyNodeFromSymbol\0") } {
            Some(function) => {
                let translated = match unsafe {
                    crate::translate_dependencies_passthrough(pDependencies, numDependencies)
                } {
                    Ok(value) => value,
                    Err(status) => return status,
                };
                let deps_ptr = translated
                    .as_ref()
                    .map(|v| v.as_ptr())
                    .unwrap_or(pDependencies);
                unsafe {
                    function(
                        pGraphNode,
                        crate::native_graph_or_passthrough(graph),
                        deps_ptr,
                        numDependencies,
                        dst,
                        symbol,
                        count,
                        offset,
                        kind,
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
    let translated = match unsafe {
        crate::translate_dependencies_passthrough(pDependencies, numDependencies)
    } {
        Ok(value) => value,
        Err(status) => return status,
    };
    let deps_ptr = translated
        .as_ref()
        .map(|v| v.as_ptr())
        .unwrap_or(pDependencies);
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphAddMemcpyNodeFromSymbol\0") })
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
            dst,
            symbol,
            count,
            offset,
            kind,
        )
    };
    if status == hipSuccess {
        finish_native_only_node(graph, native_node, pGraphNode)
    } else {
        status
    }
}

/// mutating
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphAddMemcpyNodeToSymbol(
    pGraphNode: *mut hipGraphNode_t,
    graph: hipGraph_t,
    pDependencies: *const hipGraphNode_t,
    numDependencies: usize,
    symbol: *const c_void,
    src: *const c_void,
    count: usize,
    offset: usize,
    kind: hipMemcpyKind,
) -> hipError_t {
    type Function = unsafe extern "C" fn(
        *mut hipGraphNode_t,
        hipGraph_t,
        *const hipGraphNode_t,
        usize,
        *const c_void,
        *const c_void,
        usize,
        usize,
        hipMemcpyKind,
    ) -> hipError_t;

    if !is_graph(graph) {
        return match unsafe { real_symbol::<Function>(b"hipGraphAddMemcpyNodeToSymbol\0") } {
            Some(function) => {
                let translated = match unsafe {
                    crate::translate_dependencies_passthrough(pDependencies, numDependencies)
                } {
                    Ok(value) => value,
                    Err(status) => return status,
                };
                let deps_ptr = translated
                    .as_ref()
                    .map(|v| v.as_ptr())
                    .unwrap_or(pDependencies);
                unsafe {
                    function(
                        pGraphNode,
                        crate::native_graph_or_passthrough(graph),
                        deps_ptr,
                        numDependencies,
                        symbol,
                        src,
                        count,
                        offset,
                        kind,
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
    let translated = match unsafe {
        crate::translate_dependencies_passthrough(pDependencies, numDependencies)
    } {
        Ok(value) => value,
        Err(status) => return status,
    };
    let deps_ptr = translated
        .as_ref()
        .map(|v| v.as_ptr())
        .unwrap_or(pDependencies);
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphAddMemcpyNodeToSymbol\0") })
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
            symbol,
            src,
            count,
            offset,
            kind,
        )
    };
    if status == hipSuccess {
        finish_native_only_node(graph, native_node, pGraphNode)
    } else {
        status
    }
}

// ---------------------------------------------------------------------------
// Node Get/SetParams* on hipGraphNode_t
// ---------------------------------------------------------------------------

/// read-only
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphMemcpyNodeGetParams(
    node: hipGraphNode_t,
    pNodeParams: *mut hipMemcpy3DParms,
) -> hipError_t {
    type Function = unsafe extern "C" fn(hipGraphNode_t, *mut hipMemcpy3DParms) -> hipError_t;

    let Some(snap) = node_snapshot(node) else {
        return match unsafe { real_symbol::<Function>(b"hipGraphMemcpyNodeGetParams\0") } {
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
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphMemcpyNodeGetParams\0") })
    else {
        return hipErrorNotSupported;
    };
    unsafe { function(snap.native_node as hipGraphNode_t, pNodeParams) }
}

/// mutating
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphMemcpyNodeSetParams(
    node: hipGraphNode_t,
    pNodeParams: *const hipMemcpy3DParms,
) -> hipError_t {
    type Function = unsafe extern "C" fn(hipGraphNode_t, *const hipMemcpy3DParms) -> hipError_t;

    let Some(snap) = node_snapshot(node) else {
        return match unsafe { real_symbol::<Function>(b"hipGraphMemcpyNodeSetParams\0") } {
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
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphMemcpyNodeSetParams\0") })
    else {
        return hipErrorNotSupported;
    };
    let status = unsafe { function(snap.native_node as hipGraphNode_t, pNodeParams) };
    if status == hipSuccess {
        force_native_owner(snap.owner);
    }
    status
}

/// mutating
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphMemcpyNodeSetParams1D(
    node: hipGraphNode_t,
    dst: *mut c_void,
    src: *const c_void,
    count: usize,
    kind: hipMemcpyKind,
) -> hipError_t {
    type Function = unsafe extern "C" fn(
        hipGraphNode_t,
        *mut c_void,
        *const c_void,
        usize,
        hipMemcpyKind,
    ) -> hipError_t;

    let Some(snap) = node_snapshot(node) else {
        return match unsafe { real_symbol::<Function>(b"hipGraphMemcpyNodeSetParams1D\0") } {
            Some(function) => unsafe { function(node, dst, src, count, kind) },
            None => hipErrorNotSupported,
        };
    };
    if snap.native_node == 0 {
        return hipErrorNotSupported;
    }
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphMemcpyNodeSetParams1D\0") })
    else {
        return hipErrorNotSupported;
    };
    let status = unsafe { function(snap.native_node as hipGraphNode_t, dst, src, count, kind) };
    if status == hipSuccess {
        force_native_owner(snap.owner);
    }
    status
}

/// mutating
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphMemcpyNodeSetParamsFromSymbol(
    node: hipGraphNode_t,
    dst: *mut c_void,
    symbol: *const c_void,
    count: usize,
    offset: usize,
    kind: hipMemcpyKind,
) -> hipError_t {
    type Function = unsafe extern "C" fn(
        hipGraphNode_t,
        *mut c_void,
        *const c_void,
        usize,
        usize,
        hipMemcpyKind,
    ) -> hipError_t;

    let Some(snap) = node_snapshot(node) else {
        return match unsafe { real_symbol::<Function>(b"hipGraphMemcpyNodeSetParamsFromSymbol\0") }
        {
            Some(function) => unsafe { function(node, dst, symbol, count, offset, kind) },
            None => hipErrorNotSupported,
        };
    };
    if snap.native_node == 0 {
        return hipErrorNotSupported;
    }
    let Some(function) =
        (unsafe { real_symbol::<Function>(b"hipGraphMemcpyNodeSetParamsFromSymbol\0") })
    else {
        return hipErrorNotSupported;
    };
    let status = unsafe {
        function(
            snap.native_node as hipGraphNode_t,
            dst,
            symbol,
            count,
            offset,
            kind,
        )
    };
    if status == hipSuccess {
        force_native_owner(snap.owner);
    }
    status
}

/// mutating
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphMemcpyNodeSetParamsToSymbol(
    node: hipGraphNode_t,
    symbol: *const c_void,
    src: *const c_void,
    count: usize,
    offset: usize,
    kind: hipMemcpyKind,
) -> hipError_t {
    type Function = unsafe extern "C" fn(
        hipGraphNode_t,
        *const c_void,
        *const c_void,
        usize,
        usize,
        hipMemcpyKind,
    ) -> hipError_t;

    let Some(snap) = node_snapshot(node) else {
        return match unsafe { real_symbol::<Function>(b"hipGraphMemcpyNodeSetParamsToSymbol\0") } {
            Some(function) => unsafe { function(node, symbol, src, count, offset, kind) },
            None => hipErrorNotSupported,
        };
    };
    if snap.native_node == 0 {
        return hipErrorNotSupported;
    }
    let Some(function) =
        (unsafe { real_symbol::<Function>(b"hipGraphMemcpyNodeSetParamsToSymbol\0") })
    else {
        return hipErrorNotSupported;
    };
    let status = unsafe {
        function(
            snap.native_node as hipGraphNode_t,
            symbol,
            src,
            count,
            offset,
            kind,
        )
    };
    if status == hipSuccess {
        force_native_owner(snap.owner);
    }
    status
}

// ---------------------------------------------------------------------------
// Exec*SetParams* on hipGraphExec_t (mutating)
// ---------------------------------------------------------------------------

/// mutating
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphExecMemcpyNodeSetParams(
    hGraphExec: hipGraphExec_t,
    node: hipGraphNode_t,
    pNodeParams: *mut hipMemcpy3DParms,
) -> hipError_t {
    type Function =
        unsafe extern "C" fn(hipGraphExec_t, hipGraphNode_t, *mut hipMemcpy3DParms) -> hipError_t;

    if !is_exec(hGraphExec) {
        return match unsafe { real_symbol::<Function>(b"hipGraphExecMemcpyNodeSetParams\0") } {
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
    let native_exec = state.native_exec;
    if native_exec == 0 {
        return hipErrorNotSupported;
    }
    let native_node = match state.native_nodes.get(&(node as usize)).copied() {
        Some(n) if n != 0 => n as hipGraphNode_t,
        _ => crate::native_node_or_passthrough(node),
    };
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphExecMemcpyNodeSetParams\0") })
    else {
        return hipErrorNotSupported;
    };
    let status = unsafe {
        function(
            native_exec as hipGraphExec_t,
            native_node as hipGraphNode_t,
            pNodeParams,
        )
    };
    if status == hipSuccess {
        state.force_native = true;
    }
    status
}

/// mutating
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphExecMemcpyNodeSetParams1D(
    hGraphExec: hipGraphExec_t,
    node: hipGraphNode_t,
    dst: *mut c_void,
    src: *const c_void,
    count: usize,
    kind: hipMemcpyKind,
) -> hipError_t {
    type Function = unsafe extern "C" fn(
        hipGraphExec_t,
        hipGraphNode_t,
        *mut c_void,
        *const c_void,
        usize,
        hipMemcpyKind,
    ) -> hipError_t;

    if !is_exec(hGraphExec) {
        return match unsafe { real_symbol::<Function>(b"hipGraphExecMemcpyNodeSetParams1D\0") } {
            Some(function) => unsafe {
                function(
                    crate::native_exec_or_passthrough(hGraphExec),
                    crate::native_node_or_passthrough(node),
                    dst,
                    src,
                    count,
                    kind,
                )
            },
            None => hipErrorNotSupported,
        };
    }
    let Some(handle) = exec_handle(hGraphExec) else {
        return hipErrorInvalidHandle;
    };
    let mut state = lock(&handle.state);
    let native_exec = state.native_exec;
    if native_exec == 0 {
        return hipErrorNotSupported;
    }
    let native_node = match state.native_nodes.get(&(node as usize)).copied() {
        Some(n) if n != 0 => n as hipGraphNode_t,
        _ => crate::native_node_or_passthrough(node),
    };
    let Some(function) =
        (unsafe { real_symbol::<Function>(b"hipGraphExecMemcpyNodeSetParams1D\0") })
    else {
        return hipErrorNotSupported;
    };
    let status = unsafe {
        function(
            native_exec as hipGraphExec_t,
            native_node as hipGraphNode_t,
            dst,
            src,
            count,
            kind,
        )
    };
    if status == hipSuccess {
        state.force_native = true;
    }
    status
}

/// mutating
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphExecMemcpyNodeSetParamsFromSymbol(
    hGraphExec: hipGraphExec_t,
    node: hipGraphNode_t,
    dst: *mut c_void,
    symbol: *const c_void,
    count: usize,
    offset: usize,
    kind: hipMemcpyKind,
) -> hipError_t {
    type Function = unsafe extern "C" fn(
        hipGraphExec_t,
        hipGraphNode_t,
        *mut c_void,
        *const c_void,
        usize,
        usize,
        hipMemcpyKind,
    ) -> hipError_t;

    if !is_exec(hGraphExec) {
        return match unsafe {
            real_symbol::<Function>(b"hipGraphExecMemcpyNodeSetParamsFromSymbol\0")
        } {
            Some(function) => unsafe {
                function(
                    crate::native_exec_or_passthrough(hGraphExec),
                    crate::native_node_or_passthrough(node),
                    dst,
                    symbol,
                    count,
                    offset,
                    kind,
                )
            },
            None => hipErrorNotSupported,
        };
    }
    let Some(handle) = exec_handle(hGraphExec) else {
        return hipErrorInvalidHandle;
    };
    let mut state = lock(&handle.state);
    let native_exec = state.native_exec;
    if native_exec == 0 {
        return hipErrorNotSupported;
    }
    let native_node = match state.native_nodes.get(&(node as usize)).copied() {
        Some(n) if n != 0 => n as hipGraphNode_t,
        _ => crate::native_node_or_passthrough(node),
    };
    let Some(function) =
        (unsafe { real_symbol::<Function>(b"hipGraphExecMemcpyNodeSetParamsFromSymbol\0") })
    else {
        return hipErrorNotSupported;
    };
    let status = unsafe {
        function(
            native_exec as hipGraphExec_t,
            native_node as hipGraphNode_t,
            dst,
            symbol,
            count,
            offset,
            kind,
        )
    };
    if status == hipSuccess {
        state.force_native = true;
    }
    status
}

/// mutating
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphExecMemcpyNodeSetParamsToSymbol(
    hGraphExec: hipGraphExec_t,
    node: hipGraphNode_t,
    symbol: *const c_void,
    src: *const c_void,
    count: usize,
    offset: usize,
    kind: hipMemcpyKind,
) -> hipError_t {
    type Function = unsafe extern "C" fn(
        hipGraphExec_t,
        hipGraphNode_t,
        *const c_void,
        *const c_void,
        usize,
        usize,
        hipMemcpyKind,
    ) -> hipError_t;

    if !is_exec(hGraphExec) {
        return match unsafe {
            real_symbol::<Function>(b"hipGraphExecMemcpyNodeSetParamsToSymbol\0")
        } {
            Some(function) => unsafe {
                function(
                    crate::native_exec_or_passthrough(hGraphExec),
                    crate::native_node_or_passthrough(node),
                    symbol,
                    src,
                    count,
                    offset,
                    kind,
                )
            },
            None => hipErrorNotSupported,
        };
    }
    let Some(handle) = exec_handle(hGraphExec) else {
        return hipErrorInvalidHandle;
    };
    let mut state = lock(&handle.state);
    let native_exec = state.native_exec;
    if native_exec == 0 {
        return hipErrorNotSupported;
    }
    let native_node = match state.native_nodes.get(&(node as usize)).copied() {
        Some(n) if n != 0 => n as hipGraphNode_t,
        _ => crate::native_node_or_passthrough(node),
    };
    let Some(function) =
        (unsafe { real_symbol::<Function>(b"hipGraphExecMemcpyNodeSetParamsToSymbol\0") })
    else {
        return hipErrorNotSupported;
    };
    let status = unsafe {
        function(
            native_exec as hipGraphExec_t,
            native_node as hipGraphNode_t,
            symbol,
            src,
            count,
            offset,
            kind,
        )
    };
    if status == hipSuccess {
        state.force_native = true;
    }
    status
}
