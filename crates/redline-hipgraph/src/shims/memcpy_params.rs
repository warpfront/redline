// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! Plan-staleness shims for hipGraph memcpy-node entry points.
//!
//! Under native handle identity these only mark modelled graphs/execs stale
//! and forward the call unchanged.

#![allow(non_camel_case_types, non_snake_case)]

use std::ffi::c_void;

use crate::abi::{hipGraph_t, hipGraphExec_t, hipGraphNode_t};

/// C `enum hipMemcpyKind` — ABI is a signed 32-bit enum. Values are only
/// forwarded; we never interpret them.
type hipMemcpyKind = i32;

/// Opaque stand-in for `hipMemcpy3DParms`. Every shim only passes the pointer
/// through to the real runtime, so the pointee layout is irrelevant here.
pub(crate) enum hipMemcpy3DParms {}

// ---------------------------------------------------------------------------
// Add* — create a native-only node (plan stale).
// ---------------------------------------------------------------------------

graph_mutating_shim!(
    hipGraphAddMemcpyNode1D,
    b"hipGraphAddMemcpyNode1D\0",
    graph,
    (
        pGraphNode: *mut hipGraphNode_t,
        graph: hipGraph_t,
        pDependencies: *const hipGraphNode_t,
        numDependencies: usize,
        dst: *mut c_void,
        src: *const c_void,
        count: usize,
        kind: hipMemcpyKind,
    )
);

graph_mutating_shim!(
    hipGraphAddMemcpyNodeFromSymbol,
    b"hipGraphAddMemcpyNodeFromSymbol\0",
    graph,
    (
        pGraphNode: *mut hipGraphNode_t,
        graph: hipGraph_t,
        pDependencies: *const hipGraphNode_t,
        numDependencies: usize,
        dst: *mut c_void,
        symbol: *const c_void,
        count: usize,
        offset: usize,
        kind: hipMemcpyKind,
    )
);

graph_mutating_shim!(
    hipGraphAddMemcpyNodeToSymbol,
    b"hipGraphAddMemcpyNodeToSymbol\0",
    graph,
    (
        pGraphNode: *mut hipGraphNode_t,
        graph: hipGraph_t,
        pDependencies: *const hipGraphNode_t,
        numDependencies: usize,
        symbol: *const c_void,
        src: *const c_void,
        count: usize,
        offset: usize,
        kind: hipMemcpyKind,
    )
);

// ---------------------------------------------------------------------------
// Node SetParams* on hipGraphNode_t
// ---------------------------------------------------------------------------

node_mutating_shim!(
    hipGraphMemcpyNodeSetParams,
    b"hipGraphMemcpyNodeSetParams\0",
    node,
    (
        node: hipGraphNode_t,
        pNodeParams: *const hipMemcpy3DParms,
    )
);

node_mutating_shim!(
    hipGraphMemcpyNodeSetParams1D,
    b"hipGraphMemcpyNodeSetParams1D\0",
    node,
    (
        node: hipGraphNode_t,
        dst: *mut c_void,
        src: *const c_void,
        count: usize,
        kind: hipMemcpyKind,
    )
);

node_mutating_shim!(
    hipGraphMemcpyNodeSetParamsFromSymbol,
    b"hipGraphMemcpyNodeSetParamsFromSymbol\0",
    node,
    (
        node: hipGraphNode_t,
        dst: *mut c_void,
        symbol: *const c_void,
        count: usize,
        offset: usize,
        kind: hipMemcpyKind,
    )
);

node_mutating_shim!(
    hipGraphMemcpyNodeSetParamsToSymbol,
    b"hipGraphMemcpyNodeSetParamsToSymbol\0",
    node,
    (
        node: hipGraphNode_t,
        symbol: *const c_void,
        src: *const c_void,
        count: usize,
        offset: usize,
        kind: hipMemcpyKind,
    )
);

// ---------------------------------------------------------------------------
// Exec*SetParams* on hipGraphExec_t
// ---------------------------------------------------------------------------

exec_mutating_shim!(
    hipGraphExecMemcpyNodeSetParams,
    b"hipGraphExecMemcpyNodeSetParams\0",
    hGraphExec,
    (
        hGraphExec: hipGraphExec_t,
        node: hipGraphNode_t,
        pNodeParams: *mut hipMemcpy3DParms,
    )
);

exec_mutating_shim!(
    hipGraphExecMemcpyNodeSetParams1D,
    b"hipGraphExecMemcpyNodeSetParams1D\0",
    hGraphExec,
    (
        hGraphExec: hipGraphExec_t,
        node: hipGraphNode_t,
        dst: *mut c_void,
        src: *const c_void,
        count: usize,
        kind: hipMemcpyKind,
    )
);

exec_mutating_shim!(
    hipGraphExecMemcpyNodeSetParamsFromSymbol,
    b"hipGraphExecMemcpyNodeSetParamsFromSymbol\0",
    hGraphExec,
    (
        hGraphExec: hipGraphExec_t,
        node: hipGraphNode_t,
        dst: *mut c_void,
        symbol: *const c_void,
        count: usize,
        offset: usize,
        kind: hipMemcpyKind,
    )
);

exec_mutating_shim!(
    hipGraphExecMemcpyNodeSetParamsToSymbol,
    b"hipGraphExecMemcpyNodeSetParamsToSymbol\0",
    hGraphExec,
    (
        hGraphExec: hipGraphExec_t,
        node: hipGraphNode_t,
        symbol: *const c_void,
        src: *const c_void,
        count: usize,
        offset: usize,
        kind: hipMemcpyKind,
    )
);
