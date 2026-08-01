// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! Plan-staleness shims for kernel-node params/attributes.
//!
//! llama.cpp remaps `cudaGraphKernelNodeGetParams` /
//! `cudaGraphKernelNodeSetParams` onto two of these via ggml's HIP vendor
//! header, so this surface is load-bearing for a live integration. Getters are
//! deleted under native identity — HIP answers correctly without us. Setters
//! only mark the owning graph's plan stale before forwarding.

#![allow(non_camel_case_types, non_snake_case)]

use std::ffi::c_void;
use std::os::raw::c_int;

use crate::abi::hipKernelNodeParams;
use crate::hipGraphNode_t;

/// C `hipKernelNodeAttrID` (`typedef enum hipLaunchAttributeID` → int).
type hipKernelNodeAttrID = c_int;

/// Opaque stand-in for C `hipKernelNodeAttrValue*` (64-byte union). Only ever
/// passed by pointer through the FFI boundary.
type hipKernelNodeAttrValue = c_void;

node_mutating_shim!(
    hipGraphKernelNodeSetParams,
    b"hipGraphKernelNodeSetParams\0",
    node,
    (node: hipGraphNode_t, pNodeParams: *const hipKernelNodeParams)
);

node_mutating_shim!(
    hipGraphKernelNodeSetAttribute,
    b"hipGraphKernelNodeSetAttribute\0",
    hNode,
    (
        hNode: hipGraphNode_t,
        attr: hipKernelNodeAttrID,
        value: *const hipKernelNodeAttrValue
    )
);

// Destination owns the mutated graph even though source is first in the ABI.
node_mutating_shim!(
    hipGraphKernelNodeCopyAttributes,
    b"hipGraphKernelNodeCopyAttributes\0",
    hDst,
    (hSrc: hipGraphNode_t, hDst: hipGraphNode_t)
);
