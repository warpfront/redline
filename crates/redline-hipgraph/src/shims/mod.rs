// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! Translating shims for the hipGraph surface Redline does not accelerate.
//!
//! `hipGraphCreate` hands the application a pointer to our own [`GraphHandle`],
//! not a native `ihipGraph*`. Any hipGraph entry point the interposer does not
//! export therefore reaches the real HIP runtime holding our heap pointer and
//! dereferences it as its own type. That is memory corruption in an otherwise
//! working application, and it is silent.
//!
//! Every node we accept is mirrored into a real native graph (`native_graph` /
//! `native_node`), so the fix is uniform: intercept the entry point, translate
//! our handles to the native ones, forward, and translate any handle coming
//! back. The three cases, matching the idiom already used by
//! `unsupported_pointer_node!`:
//!
//! 1. Handle is not ours (`!is_graph`) — forward untouched. Applications may
//!    mix native graphs with ours in one process.
//! 2. Read-only introspection — translate and forward. The retained PM4 plan
//!    stays valid, so acceleration is preserved.
//! 3. Mutating — translate, forward, and set `force_native = true`. The native
//!    graph is now authoritative and our lowered plan is stale, so replay must
//!    fall back rather than execute a plan that no longer matches the graph.
//!
//! Losing acceleration is acceptable. Executing a stale PM4 plan is not.

mod events;
mod exotic;
mod instantiate;
mod introspect;
mod kernel_params;
mod memcpy_params;
mod memset_host_params;
mod structure;
