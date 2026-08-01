// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! Plan-staleness shims for hipGraph event record/wait nodes.
//!
//! `hipEvent_t` is a native HIP handle — pass straight through. GetEvent
//! entry points are intentionally unexported (HIP answers correctly).

#![allow(non_snake_case, non_camel_case_types)]

use std::ffi::c_void;

use crate::abi::{hipGraph_t, hipGraphExec_t, hipGraphNode_t};

/// Native HIP event handle — opaque to us; pass straight through.
type hipEvent_t = *mut c_void;

graph_mutating_shim!(
    hipGraphAddEventRecordNode,
    b"hipGraphAddEventRecordNode\0",
    graph,
    (
        pGraphNode: *mut hipGraphNode_t,
        graph: hipGraph_t,
        pDependencies: *const hipGraphNode_t,
        numDependencies: usize,
        event: hipEvent_t,
    )
);

graph_mutating_shim!(
    hipGraphAddEventWaitNode,
    b"hipGraphAddEventWaitNode\0",
    graph,
    (
        pGraphNode: *mut hipGraphNode_t,
        graph: hipGraph_t,
        pDependencies: *const hipGraphNode_t,
        numDependencies: usize,
        event: hipEvent_t,
    )
);

node_mutating_shim!(
    hipGraphEventRecordNodeSetEvent,
    b"hipGraphEventRecordNodeSetEvent\0",
    node,
    (node: hipGraphNode_t, event: hipEvent_t,)
);

node_mutating_shim!(
    hipGraphEventWaitNodeSetEvent,
    b"hipGraphEventWaitNodeSetEvent\0",
    node,
    (node: hipGraphNode_t, event: hipEvent_t,)
);

exec_mutating_shim!(
    hipGraphExecEventRecordNodeSetEvent,
    b"hipGraphExecEventRecordNodeSetEvent\0",
    hGraphExec,
    (
        hGraphExec: hipGraphExec_t,
        hNode: hipGraphNode_t,
        event: hipEvent_t,
    )
);

exec_mutating_shim!(
    hipGraphExecEventWaitNodeSetEvent,
    b"hipGraphExecEventWaitNodeSetEvent\0",
    hGraphExec,
    (
        hGraphExec: hipGraphExec_t,
        hNode: hipGraphNode_t,
        event: hipEvent_t,
    )
);
