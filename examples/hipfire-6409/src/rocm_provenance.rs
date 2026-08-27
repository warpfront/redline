// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! ROCm runtime provenance for result artifacts.
//!
//! Why this exists: ROCm 7.14 and 10.0 are installed side-by-side at
//! `/opt/rocm/core-7.14` and `/opt/rocm/core-10.0` with `/opt/rocm/core`
//! symlinking to one of them. A binary built against one toolchain will
//! silently load whichever runtime the loader resolves first, so an A/B
//! that appears to compare compilers can actually compare the same runtime
//! twice. The only trustworthy evidence is asked from inside the process:
//! `hipRuntimeGetVersion` plus the on-disk path of the HIP/HSA objects
//! actually mapped. This module captures that at measurement time and
//! attaches it to every artifact so numbers are never again ambiguous
//! between releases.
//!
//! The pattern mirrors `bench/dispatch/rocm_ident.cpp`: runtime version
//! plus `dladdr` on a HIP entry point plus `/proc/self/maps` enumeration.
//! Pure Rust cannot call `dladdr` on the HIP symbol without extra FFI,
//! so we use `/proc/self/maps` as the ground truth for the resolved
//! `libamdhip64`/`libhsa-runtime64` paths. `hipRuntimeGetVersion` is
//! obtained from the loaded HIP runtime via `HipRuntime`.

use anyhow::Result;
use hip_bridge::HipRuntime;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::fs;

/// Collect provenance from the loaded HIP runtime and `/proc/self/maps`.
///
/// Never fails the benchmark: on error the fields are `null` or explain
/// the failure, but a `Value` is always returned.
pub fn collect(hip: &HipRuntime) -> Value {
    // hipRuntimeGetVersion via the dlopened runtime. HipRuntime exposes it
    // as (major, minor); we keep both the tuple and a best-effort raw
    // integer for direct comparison with the C++ probe (71460850 vs
    // 71526333).
    let (version_raw, version_tuple, version_error) = match hip.runtime_version() {
        Ok((major, minor)) => {
            // Reconstruct an approximate raw encoding for provenance display.
            // The exact raw is major*10_000_000 + minor*100_000 + patch;
            // we cannot recover patch from the tuple, so we emit the tuple
            // and the reconstructed major/minor raw as separate fields.
            let approx_raw = major * 10_000_000 + minor * 100_000;
            (Some(approx_raw), Some(json!([major, minor])), None)
        }
        Err(e) => (None, None, Some(format!("{e:#}"))),
    };

    // Attempt a direct raw hipRuntimeGetVersion via a second dlopen so the
    // full integer (including patch) is captured when the library is
    // resolvable. This is best-effort; the tuple above is the fallback.
    let raw_direct = raw_hip_runtime_version_via_dlopen();

    let maps_entries = read_rocm_maps();
    let (libamdhip64_path, libhsa_path) = extract_hip_hsa_paths(&maps_entries);

    // Summarize by tree, mirroring rocm_ident.cpp's tags.
    let (from_714, from_100, from_other, from_core_symlink) = summarize_by_tree(&maps_entries);

    json!({
        "hip_runtime_version_raw": raw_direct.or(version_raw),
        "hip_runtime_version_tuple": version_tuple,
        "hip_runtime_version_error": version_error,
        "hip_runtime_version_approx_raw": version_raw,
        "hip_runtime_version_raw_direct": raw_direct,
        "libamdhip64_path": libamdhip64_path,
        "libhsa_runtime_path": libhsa_path,
        "mapped_rocm_objects": maps_entries,
        "mapped_summary": {
            "core_7_14": from_714,
            "core_10_0": from_100,
            "core_symlink": from_core_symlink,
            "other": from_other,
        },
        "mixed_load_warning": from_714 > 0 && from_100 > 0,
        "notes": "hipRuntimeGetVersion plus resolved libamdhip64/libhsa paths from /proc/self/maps; compare with bench/dispatch/rocm_ident.cpp",
    })
}

/// Try to obtain the raw hipRuntimeGetVersion integer by dlopening
/// libamdhip64 again and calling the symbol. Best-effort: returns None
/// when the library cannot be opened or the symbol is missing.
fn raw_hip_runtime_version_via_dlopen() -> Option<i32> {
    // Use libloading directly to avoid touching the already-loaded HipRuntime.
    // Try the same SONAMEs HipRuntime tries.
    let candidates = [
        "libamdhip64.so",
        "libamdhip64.so.7",
        "libamdhip64.so.6",
        "libamdhip64.so.5",
    ];
    for cand in candidates {
        if let Ok(lib) = unsafe { libloading::Library::new(cand) } {
            unsafe {
                let sym: Result<libloading::Symbol<unsafe extern "C" fn(*mut i32) -> u32>, _> =
                    lib.get(b"hipRuntimeGetVersion\0");
                if let Ok(func) = sym {
                    let mut version: i32 = 0;
                    let code = func(&mut version as *mut i32);
                    if code == 0 {
                        return Some(version);
                    }
                }
            }
        }
    }
    None
}

fn read_rocm_maps() -> Vec<String> {
    let content = match fs::read_to_string("/proc/self/maps") {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for line in content.lines() {
        // /proc/self/maps: address perms offset dev inode pathname
        // pathname is after the last whitespace when present.
        let Some(path) = line.split_whitespace().last() else {
            continue;
        };
        if !path.starts_with('/') {
            continue;
        }
        // Keep ROCm objects plus the two HIP/HSA libraries wherever they live.
        let is_rocm = path.contains("/opt/rocm")
            || path.contains("rocm")
            || path.contains("libamdhip64")
            || path.contains("libhsa-runtime");
        if !is_rocm {
            continue;
        }
        if seen.insert(path.to_owned()) {
            out.push(path.to_owned());
        }
    }
    out.sort();
    out
}

fn extract_hip_hsa_paths(entries: &[String]) -> (Option<String>, Option<String>) {
    let hip = entries
        .iter()
        .find(|p| p.contains("libamdhip64"))
        .cloned();
    let hsa = entries
        .iter()
        .find(|p| p.contains("libhsa-runtime"))
        .cloned();
    (hip, hsa)
}

fn summarize_by_tree(entries: &[String]) -> (usize, usize, usize, usize) {
    let mut from_714 = 0usize;
    let mut from_100 = 0usize;
    let mut from_symlink = 0usize;
    let mut other = 0usize;
    for p in entries {
        if p.contains("core-7.14") {
            from_714 += 1;
        } else if p.contains("core-10.0") {
            from_100 += 1;
        } else if p.contains("/opt/rocm/core/") || p.contains("/opt/rocm/core/lib") {
            // The symlink tree without an explicit version suffix.
            from_symlink += 1;
        } else {
            other += 1;
        }
    }
    (from_714, from_100, other, from_symlink)
}
