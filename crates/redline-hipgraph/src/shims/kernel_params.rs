// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! Translating shims for kernel-node params/attributes.
//!
//! llama.cpp remaps `cudaGraphKernelNodeGetParams` /
//! `cudaGraphKernelNodeSetParams` onto two of these via ggml's HIP vendor
//! header, so this surface is load-bearing for a live integration.

#![allow(non_camel_case_types, non_snake_case)]

use std::ffi::c_void;
use std::os::raw::c_int;

use crate::abi::{
    hipError_t, hipErrorInvalidValue, hipErrorNotSupported, hipKernelNodeParams, hipSuccess,
};
use crate::{graph_handle, hipGraph_t, hipGraphNode_t, lock, node_snapshot, real_symbol};

/// C `hipKernelNodeAttrID` (`typedef enum hipLaunchAttributeID` → int).
type hipKernelNodeAttrID = c_int;

/// Opaque stand-in for C `hipKernelNodeAttrValue*` (64-byte union). Only ever
/// passed by pointer through the FFI boundary.
type hipKernelNodeAttrValue = c_void;

/// Mark the owning graph's PM4 plan stale after a successful mutating call.
fn force_native_owner(owner: usize) {
    if let Some(handle) = graph_handle(owner as hipGraph_t) {
        lock(&handle.state).force_native = true;
    }
}

/// Resolve a node handle to the native HIP node pointer, or an error if ours
/// but has no native shadow. `None` owner means the handle is not ours.
fn native_or_passthrough(
    node: hipGraphNode_t,
) -> Result<(hipGraphNode_t, Option<usize>), hipError_t> {
    match node_snapshot(node) {
        None => Ok((node, None)),
        Some(snap) if snap.native_node == 0 => Err(hipErrorNotSupported),
        Some(snap) => Ok((snap.native_node as hipGraphNode_t, Some(snap.owner))),
    }
}

/// read-only — translate + forward; acceleration survives.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphKernelNodeGetParams(
    node: hipGraphNode_t,
    pNodeParams: *mut hipKernelNodeParams,
) -> hipError_t {
    type Function = unsafe extern "C" fn(hipGraphNode_t, *mut hipKernelNodeParams) -> hipError_t;

    let Some(snap) = node_snapshot(node) else {
        return match unsafe { real_symbol::<Function>(b"hipGraphKernelNodeGetParams\0") } {
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
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphKernelNodeGetParams\0") })
    else {
        return hipErrorNotSupported;
    };
    unsafe { function(snap.native_node as hipGraphNode_t, pNodeParams) }
}

/// mutating — translate + forward + force_native (kernel identity / launch geometry).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphKernelNodeSetParams(
    node: hipGraphNode_t,
    pNodeParams: *const hipKernelNodeParams,
) -> hipError_t {
    type Function = unsafe extern "C" fn(hipGraphNode_t, *const hipKernelNodeParams) -> hipError_t;

    let Some(snap) = node_snapshot(node) else {
        return match unsafe { real_symbol::<Function>(b"hipGraphKernelNodeSetParams\0") } {
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
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphKernelNodeSetParams\0") })
    else {
        return hipErrorNotSupported;
    };
    let status = unsafe { function(snap.native_node as hipGraphNode_t, pNodeParams) };
    if status == hipSuccess {
        force_native_owner(snap.owner);
    }
    status
}

/// read-only — translate + forward; acceleration survives.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphKernelNodeGetAttribute(
    hNode: hipGraphNode_t,
    attr: hipKernelNodeAttrID,
    value: *mut hipKernelNodeAttrValue,
) -> hipError_t {
    type Function = unsafe extern "C" fn(
        hipGraphNode_t,
        hipKernelNodeAttrID,
        *mut hipKernelNodeAttrValue,
    ) -> hipError_t;

    let Some(snap) = node_snapshot(hNode) else {
        return match unsafe { real_symbol::<Function>(b"hipGraphKernelNodeGetAttribute\0") } {
            Some(function) => unsafe { function(hNode, attr, value) },
            None => hipErrorNotSupported,
        };
    };
    if value.is_null() {
        return hipErrorInvalidValue;
    }
    if snap.native_node == 0 {
        return hipErrorNotSupported;
    }
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphKernelNodeGetAttribute\0") })
    else {
        return hipErrorNotSupported;
    };
    unsafe { function(snap.native_node as hipGraphNode_t, attr, value) }
}

/// mutating — translate + forward + force_native.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphKernelNodeSetAttribute(
    hNode: hipGraphNode_t,
    attr: hipKernelNodeAttrID,
    value: *const hipKernelNodeAttrValue,
) -> hipError_t {
    type Function = unsafe extern "C" fn(
        hipGraphNode_t,
        hipKernelNodeAttrID,
        *const hipKernelNodeAttrValue,
    ) -> hipError_t;

    let Some(snap) = node_snapshot(hNode) else {
        return match unsafe { real_symbol::<Function>(b"hipGraphKernelNodeSetAttribute\0") } {
            Some(function) => unsafe { function(hNode, attr, value) },
            None => hipErrorNotSupported,
        };
    };
    if value.is_null() {
        return hipErrorInvalidValue;
    }
    if snap.native_node == 0 {
        return hipErrorNotSupported;
    }
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphKernelNodeSetAttribute\0") })
    else {
        return hipErrorNotSupported;
    };
    let status = unsafe { function(snap.native_node as hipGraphNode_t, attr, value) };
    if status == hipSuccess {
        force_native_owner(snap.owner);
    }
    status
}

/// mutating (destination) — translate both handles independently; either may
/// not be ours. force_native only on the destination owner's graph.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphKernelNodeCopyAttributes(
    hSrc: hipGraphNode_t,
    hDst: hipGraphNode_t,
) -> hipError_t {
    type Function = unsafe extern "C" fn(hipGraphNode_t, hipGraphNode_t) -> hipError_t;

    let src_ours = node_snapshot(hSrc).is_some();
    let dst_ours = node_snapshot(hDst).is_some();
    if !src_ours && !dst_ours {
        return match unsafe { real_symbol::<Function>(b"hipGraphKernelNodeCopyAttributes\0") } {
            Some(function) => unsafe { function(hSrc, hDst) },
            None => hipErrorNotSupported,
        };
    }

    let (native_src, _) = match native_or_passthrough(hSrc) {
        Ok(v) => v,
        Err(status) => return status,
    };
    let (native_dst, dst_owner) = match native_or_passthrough(hDst) {
        Ok(v) => v,
        Err(status) => return status,
    };

    let Some(function) =
        (unsafe { real_symbol::<Function>(b"hipGraphKernelNodeCopyAttributes\0") })
    else {
        return hipErrorNotSupported;
    };
    let status = unsafe { function(native_src, native_dst) };
    if status == hipSuccess {
        if let Some(owner) = dst_owner {
            force_native_owner(owner);
        }
    }
    status
}
