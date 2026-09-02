// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

use anyhow::{bail, Context, Result};
use radiowave::{CompileRequest, SchedulerProfile, Wavefront};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=kernels/gemm_mqv2_wmma_gfx11_bt.hip");
    println!("cargo:rerun-if-changed=kernels/gemm_mqv2_wmma_gfx11_mw_lds.hip");
    println!("cargo:rerun-if-changed=kernels/gemm_qkv_mqv2_wmma_gfx1201_bt.hip");
    println!("cargo:rerun-if-changed={}", radiowave::support_header_path().display());
    println!("cargo:rerun-if-env-changed=HIPFIRE_BENCH_ARCH");
    println!("cargo:rerun-if-env-changed=HIPCC");
    println!("cargo:rerun-if-env-changed=ROCM_PATH");
    println!("cargo:rerun-if-env-changed=RADIOWAVE_HIP_ARGS");
    println!("cargo:rerun-if-env-changed=RADIOWAVE_BUNDLER");
    println!("cargo:rerun-if-env-changed=RADIOWAVE_READOBJ");
    println!("cargo:rerun-if-env-changed=RADIOWAVE_OBJDUMP");

    let out = PathBuf::from(env::var_os("OUT_DIR").context("OUT_DIR is unset")?);
    let arch = env::var("HIPFIRE_BENCH_ARCH").unwrap_or_else(|_| "gfx1201".to_owned());
    if !matches!(arch.as_str(), "gfx1100" | "gfx1151" | "gfx1201") {
        bail!("HIPFIRE_BENCH_ARCH must be gfx1100, gfx1151 or gfx1201, got {arch}");
    }

    // Resolve hipcc with readable failure.
    let hipcc = resolve_hipcc()?;
    if !hipcc.exists() {
        let probe = std::process::Command::new(&hipcc).arg("--version").output();
        if probe.is_err() {
            bail!(
                "hipcc not found at {} (HIPCC env / /opt/rocm/core/bin/hipcc). Set HIPCC=/opt/rocm/core-10.0/bin/hipcc or install ROCm.",
                hipcc.display()
            );
        }
    }

    // Radiowave injects -include radiowave/hip.h and -DRADIOWAVE_ACTIVE=1 via
    // compile_args(). That header includes <hip/hip_runtime.h> and defines
    // namespace radiowave helpers (buffer_resource etc.). The mqv2 kernels only
    // include <hip/hip_runtime.h> / <hip/hip_fp16.h> and use
    // __builtin_amdgcn_wmma_*_w32 / _gfx12 intrinsics; they do not reference
    // radiowave symbols, and include guards make the double hip_runtime include
    // harmless. Verified: neither BT nor MW nor gfx1201 file defines
    // RADIOWAVE_ACTIVE guards or conflicts with the injected header.
    // No extra_args workaround is required; we keep the plain Radiowave path.

    let is_gfx11 = arch == "gfx1100" || arch == "gfx1151";

    // For gfx11 we must deliver a single code object containing all BT+MW
    // symbols. Radiowave's CompileRequest takes one source file, so we generate
    // an umbrella under OUT_DIR that #includes both kernel files via absolute
    // paths. gfx1201 builds only its single file.
    let cwd = env::current_dir().context("current_dir")?;
    let bt = cwd.join("kernels/gemm_mqv2_wmma_gfx11_bt.hip");
    let mw = cwd.join("kernels/gemm_mqv2_wmma_gfx11_mw_lds.hip");
    let gfx12 = cwd.join("kernels/gemm_qkv_mqv2_wmma_gfx1201_bt.hip");

    let selected_source: PathBuf;
    if is_gfx11 {
        // Umbrella must avoid duplicate decode_tile: both BT and MW define
        // the same template decode_tile<BITS>. Including both verbatim triggers
        // duplicate-definition / substitution failures. We concatenate BT
        // verbatim and MW with its decode_tile stripped.
        let umbrella = out.join("mqv2_gfx11_umbrella.hip");
        let bt_content = fs::read_to_string(&bt).unwrap_or_default();
        let mw_content = fs::read_to_string(&mw).unwrap_or_default();
        // Strip the decode_tile function from MW (first occurrence of template <int BITS> ... decode_tile)
        let mw_stripped = if let Some(start) = mw_content.find("template <int BITS>") {
            if let Some(brace) = mw_content[start..].find("return out;") {
                let end = start + brace + "return out;".len();
                // Find closing brace after return
                if let Some(close) = mw_content[end..].find("\n}") {
                    let cut_end = end + close + 2;
                    format!("{}{}", &mw_content[..start], &mw_content[cut_end..])
                } else { mw_content.clone() }
            } else { mw_content.clone() }
        } else { mw_content.clone() };
        let combined = format!("{}\n{}\n", bt_content, mw_stripped);
        fs::write(&umbrella, combined)?;
        selected_source = umbrella;
    } else {
        selected_source = gfx12;
    }

    // Attempt real HSACO for selected arch across all SchedulerProfile::ALL;
    // LLVM misched profiles can crash clang (e.g. gfx1201 iterative_ilp
    // RAGreedy segfault in clang-23 10.0). Per-profile failures emit an empty
    // placeholder so code_object() returns empty slice and the driver skips.
    let all_arches = ["gfx1100", "gfx1151", "gfx1201"];
    for a in all_arches {
        for profile in SchedulerProfile::ALL {
            let output = out.join(format!("mqv2_{}_{}.hsaco", a, profile.as_str()));
            if a == arch {
                if let Err(e) = build_one(&hipcc, &selected_source, a, &output, profile) {
                    eprintln!(
                        "warning: mqv2 HSACO build failed for {a} {:?}: {e:#}; emitting empty placeholder",
                        profile
                    );
                    let _ = fs::remove_file(&output);
                    fs::write(&output, &[] as &[u8])?;
                    let placeholder_manifest = output.with_extension("radiowave.json");
                    if !placeholder_manifest.exists() {
                        fs::write(&placeholder_manifest, b"{}")?;
                    }
                }
            } else if !output.exists() {
                fs::write(&output, &[] as &[u8])?;
                let placeholder_manifest = output.with_extension("radiowave.json");
                if !placeholder_manifest.exists() {
                    fs::write(&placeholder_manifest, b"{}")?;
                }
            }
        }
    }

    Ok(())
}

fn resolve_hipcc() -> Result<PathBuf> {
    if let Some(v) = env::var_os("HIPCC") {
        if !v.is_empty() {
            return Ok(PathBuf::from(v));
        }
    }
    for cand in [
        "/opt/rocm/core/bin/hipcc",
        "/opt/rocm/core-10.0/bin/hipcc",
        "/opt/rocm/core-7.14/bin/hipcc",
    ] {
        let p = PathBuf::from(cand);
        if p.exists() {
            return Ok(p);
        }
    }
    Ok(PathBuf::from("hipcc"))
}

fn build_one(
    hipcc: &Path,
    source: &Path,
    arch: &str,
    output: &Path,
    profile: SchedulerProfile,
) -> Result<()> {
    let mut req = CompileRequest::new(source, output, arch)
        .wavefront(Wavefront::Wave32)
        .scheduler_profile(profile)
        .hipcc(hipcc)
        .manifest(output.with_extension("radiowave.json"));
    if let Ok(args) = env::var("RADIOWAVE_HIP_ARGS") {
        req.extra_args.extend(args.split_ascii_whitespace().map(Into::into));
    }
    radiowave::Compiler.compile(&req).with_context(|| {
        format!(
            "Radiowave failed to build mqv2 HSACO for {arch} {:?} (hipcc={}, source={})",
            profile,
            hipcc.display(),
            source.display()
        )
    })?;
    Ok(())
}
