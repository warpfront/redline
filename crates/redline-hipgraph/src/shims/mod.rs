// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! Plan-staleness shims for hipGraph entry points Redline does not accelerate.
//!
//! Application handles are real native HIP pointers. Side-table state is keyed
//! by those pointers; a registry miss simply means we do not model the object
//! and the call forwards unchanged. These shims exist only to mark a modeled
//! graph or exec plan stale (`force_native = true`) before forwarding when the
//! call mutates topology or parameters our lowered PM4 plan cannot represent.

/// Mark a modeled graph's plan stale, then forward every argument unchanged.
///
/// `$graph` is the state lookup key among `$($arg: $ty),*` (which must include
/// it). Used for `Add*` and other graph-level mutators whose first parameter is
/// not always the graph.
macro_rules! graph_mutating_shim {
    ($name:ident, $symbol:literal, $graph:ident, ($($arg:ident: $ty:ty),* $(,)?)) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name($($arg: $ty),*) -> crate::abi::hipError_t {
            type Function = unsafe extern "C" fn($($ty),*) -> crate::abi::hipError_t;
            if let Some(state) = crate::graph_state($graph) {
                crate::lock(&state).force_native = true;
            }
            match unsafe { crate::real_symbol::<Function>($symbol) } {
                Some(function) => unsafe { function($($arg),*) },
                None => crate::abi::hipErrorNotSupported,
            }
        }
    };
}

/// Mark the owning graph of a node stale, then forward every argument unchanged.
///
/// `$owner` is the `hipGraphNode_t` whose `node_record` supplies the graph key
/// (destination node for `hipGraphKernelNodeCopyAttributes`). The
/// `unregister = $owner` arm forwards first and calls `unregister_node` only
/// when the native status is `hipSuccess`.
macro_rules! node_mutating_shim {
    ($name:ident, $symbol:literal, unregister = $owner:ident, ($($arg:ident: $ty:ty),* $(,)?)) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name($($arg: $ty),*) -> crate::abi::hipError_t {
            type Function = unsafe extern "C" fn($($ty),*) -> crate::abi::hipError_t;
            if let Some(record) = crate::node_record($owner) {
                let graph = record.graph as crate::abi::hipGraph_t;
                if let Some(state) = crate::graph_state(graph) {
                    crate::lock(&state).force_native = true;
                }
            }
            let status = match unsafe { crate::real_symbol::<Function>($symbol) } {
                Some(function) => unsafe { function($($arg),*) },
                None => crate::abi::hipErrorNotSupported,
            };
            if status == crate::abi::hipSuccess {
                crate::unregister_node($owner as usize);
            }
            status
        }
    };
    ($name:ident, $symbol:literal, $owner:ident, ($($arg:ident: $ty:ty),* $(,)?)) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name($($arg: $ty),*) -> crate::abi::hipError_t {
            type Function = unsafe extern "C" fn($($ty),*) -> crate::abi::hipError_t;
            if let Some(record) = crate::node_record($owner) {
                let graph = record.graph as crate::abi::hipGraph_t;
                if let Some(state) = crate::graph_state(graph) {
                    crate::lock(&state).force_native = true;
                }
            }
            match unsafe { crate::real_symbol::<Function>($symbol) } {
                Some(function) => unsafe { function($($arg),*) },
                None => crate::abi::hipErrorNotSupported,
            }
        }
    };
}

/// Mark a modeled exec's plan stale, then forward every argument unchanged.
///
/// `$exec` is the state lookup key among `$($arg: $ty),*`.
macro_rules! exec_mutating_shim {
    ($name:ident, $symbol:literal, $exec:ident, ($($arg:ident: $ty:ty),* $(,)?)) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name($($arg: $ty),*) -> crate::abi::hipError_t {
            type Function = unsafe extern "C" fn($($ty),*) -> crate::abi::hipError_t;
            if let Some(state) = crate::exec_state($exec) {
                crate::lock(&state).force_native = true;
            }
            match unsafe { crate::real_symbol::<Function>($symbol) } {
                Some(function) => unsafe { function($($arg),*) },
                None => crate::abi::hipErrorNotSupported,
            }
        }
    };
}

mod events;
mod exotic;
mod instantiate;
mod kernel_params;
mod memcpy_params;
mod memset_host_params;
mod structure;
