// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! ROCm runtime provenance for result artifacts.
//! Copied verbatim from `examples/hipfire-6409/src/rocm_provenance.rs`.
//! See that file for the full rationale: ROCm 7.14 and 10.0 sit side by side
//! under `/opt/rocm/core-*` with `/opt/rocm/core` symlinking to one of them; a
//! binary built against one toolchain will silently load whichever runtime the
//! loader resolves first. The only trustworthy evidence is asked from inside
//! the process: `hipRuntimeGetVersion` plus the on-disk path of the HIP/HSA
//! objects actually mapped. This module captures that at measurement time.

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
    let (version_raw, version_tuple, version_error) = match hip.runtime_version() {
        Ok((major, minor)) => {
            let approx_raw = major * 10_000_000 + minor * 100_000;
            (Some(approx_raw), Some(json!([major, minor])), None)
        }
        Err(e) => (None, None, Some(format!("{e:#}"))),
    };

    let raw_direct = raw_hip_runtime_version_via_dlopen();

    let maps_entries = read_rocm_maps();
    let (libamdhip64_path, libhsa_path) = extract_hip_hsa_paths(&maps_entries);
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
/// libamdhip64 again and calling the symbol. Best-effort.
fn raw_hip_runtime_version_via_dlopen() -> Option<i32> {
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
        let Some(path) = line.split_whitespace().last() else {
            continue;
        };
        if !path.starts_with('/') {
            continue;
        }
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
            from_symlink += 1;
        } else {
            other += 1;
        }
    }
    (from_714, from_100, other, from_symlink)
}
