// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

#![allow(non_snake_case, non_camel_case_types)]
//! Topology-mutating hipGraph shims: mark plan stale, forward native handles.

use std::ffi::c_void;

use crate::abi::{hipGraph_t, hipGraphExec_t, hipGraphNode_t};

graph_mutating_shim!(
    hipGraphAddEmptyNode,
    b"hipGraphAddEmptyNode\0",
    graph,
    (
        pGraphNode: *mut hipGraphNode_t,
        graph: hipGraph_t,
        pDependencies: *const hipGraphNode_t,
        numDependencies: usize,
    )
);

graph_mutating_shim!(
    hipGraphAddNode,
    b"hipGraphAddNode\0",
    graph,
    (
        pGraphNode: *mut hipGraphNode_t,
        graph: hipGraph_t,
        pDependencies: *const hipGraphNode_t,
        numDependencies: usize,
        nodeParams: *mut c_void,
    )
);

graph_mutating_shim!(
    hipGraphRemoveDependencies,
    b"hipGraphRemoveDependencies\0",
    graph,
    (
        graph: hipGraph_t,
        from: *const hipGraphNode_t,
        to: *const hipGraphNode_t,
        numDependencies: usize,
    )
);

node_mutating_shim!(
    hipGraphNodeSetParams,
    b"hipGraphNodeSetParams\0",
    node,
    (node: hipGraphNode_t, nodeParams: *mut c_void,)
);

node_mutating_shim!(
    hipGraphDestroyNode,
    b"hipGraphDestroyNode\0",
    unregister = node,
    (node: hipGraphNode_t,)
);

exec_mutating_shim!(
    hipGraphExecNodeSetParams,
    b"hipGraphExecNodeSetParams\0",
    graphExec,
    (
        graphExec: hipGraphExec_t,
        node: hipGraphNode_t,
        nodeParams: *mut c_void,
    )
);

exec_mutating_shim!(
    hipGraphNodeSetEnabled,
    b"hipGraphNodeSetEnabled\0",
    hGraphExec,
    (hGraphExec: hipGraphExec_t, hNode: hipGraphNode_t, isEnabled: u32,)
);
