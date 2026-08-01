// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! Plan-staleness shims for exotic / never-accelerated hipGraph node types:
//! batch mem-op, external semaphore, and mem alloc/free.

#![allow(non_snake_case, non_camel_case_types)]

use std::ffi::c_void;

use crate::abi::{hipGraph_t, hipGraphExec_t, hipGraphNode_t};

/// Opaque HIP param blobs we only pass through.
type hipBatchMemOpNodeParams = c_void;
type hipMemAllocNodeParams = c_void;
type hipExternalSemaphoreSignalNodeParams = c_void;
type hipExternalSemaphoreWaitNodeParams = c_void;

// ---------------------------------------------------------------------------
// Batch mem-op nodes
// ---------------------------------------------------------------------------

graph_mutating_shim!(
    hipGraphAddBatchMemOpNode,
    b"hipGraphAddBatchMemOpNode\0",
    hGraph,
    (
        phGraphNode: *mut hipGraphNode_t,
        hGraph: hipGraph_t,
        dependencies: *const hipGraphNode_t,
        numDependencies: usize,
        nodeParams: *const hipBatchMemOpNodeParams,
    )
);

node_mutating_shim!(
    hipGraphBatchMemOpNodeSetParams,
    b"hipGraphBatchMemOpNodeSetParams\0",
    hNode,
    (
        hNode: hipGraphNode_t,
        nodeParams: *mut hipBatchMemOpNodeParams,
    )
);

exec_mutating_shim!(
    hipGraphExecBatchMemOpNodeSetParams,
    b"hipGraphExecBatchMemOpNodeSetParams\0",
    hGraphExec,
    (
        hGraphExec: hipGraphExec_t,
        hNode: hipGraphNode_t,
        nodeParams: *const hipBatchMemOpNodeParams,
    )
);

// ---------------------------------------------------------------------------
// External semaphore signal / wait nodes
// ---------------------------------------------------------------------------

graph_mutating_shim!(
    hipGraphAddExternalSemaphoresSignalNode,
    b"hipGraphAddExternalSemaphoresSignalNode\0",
    graph,
    (
        pGraphNode: *mut hipGraphNode_t,
        graph: hipGraph_t,
        pDependencies: *const hipGraphNode_t,
        numDependencies: usize,
        nodeParams: *const hipExternalSemaphoreSignalNodeParams,
    )
);

graph_mutating_shim!(
    hipGraphAddExternalSemaphoresWaitNode,
    b"hipGraphAddExternalSemaphoresWaitNode\0",
    graph,
    (
        pGraphNode: *mut hipGraphNode_t,
        graph: hipGraph_t,
        pDependencies: *const hipGraphNode_t,
        numDependencies: usize,
        nodeParams: *const hipExternalSemaphoreWaitNodeParams,
    )
);

node_mutating_shim!(
    hipGraphExternalSemaphoresSignalNodeSetParams,
    b"hipGraphExternalSemaphoresSignalNodeSetParams\0",
    hNode,
    (
        hNode: hipGraphNode_t,
        nodeParams: *const hipExternalSemaphoreSignalNodeParams,
    )
);

node_mutating_shim!(
    hipGraphExternalSemaphoresWaitNodeSetParams,
    b"hipGraphExternalSemaphoresWaitNodeSetParams\0",
    hNode,
    (
        hNode: hipGraphNode_t,
        nodeParams: *const hipExternalSemaphoreWaitNodeParams,
    )
);

exec_mutating_shim!(
    hipGraphExecExternalSemaphoresSignalNodeSetParams,
    b"hipGraphExecExternalSemaphoresSignalNodeSetParams\0",
    hGraphExec,
    (
        hGraphExec: hipGraphExec_t,
        hNode: hipGraphNode_t,
        nodeParams: *const hipExternalSemaphoreSignalNodeParams,
    )
);

exec_mutating_shim!(
    hipGraphExecExternalSemaphoresWaitNodeSetParams,
    b"hipGraphExecExternalSemaphoresWaitNodeSetParams\0",
    hGraphExec,
    (
        hGraphExec: hipGraphExec_t,
        hNode: hipGraphNode_t,
        nodeParams: *const hipExternalSemaphoreWaitNodeParams,
    )
);

// ---------------------------------------------------------------------------
// Mem alloc / free nodes
// ---------------------------------------------------------------------------

graph_mutating_shim!(
    hipGraphAddMemAllocNode,
    b"hipGraphAddMemAllocNode\0",
    graph,
    (
        pGraphNode: *mut hipGraphNode_t,
        graph: hipGraph_t,
        pDependencies: *const hipGraphNode_t,
        numDependencies: usize,
        pNodeParams: *mut hipMemAllocNodeParams,
    )
);

graph_mutating_shim!(
    hipGraphAddMemFreeNode,
    b"hipGraphAddMemFreeNode\0",
    graph,
    (
        pGraphNode: *mut hipGraphNode_t,
        graph: hipGraph_t,
        pDependencies: *const hipGraphNode_t,
        numDependencies: usize,
        dev_ptr: *mut c_void,
    )
);
