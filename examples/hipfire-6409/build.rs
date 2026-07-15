// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

use anyhow::{bail, Context, Result};
use radiowave::{CompileRequest, Compiler, SchedulerProfile, Wavefront};
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

const SHADERS: &[&str] = &[
    "dispatch_tiny",
    "geometry_fma",
    "reduction_wave",
    "reduction_lds",
    "reduction_extra_barrier",
    "reduction_multi4",
    "reduction_multi8",
    "reduction_multi16",
    "memory_coalesced4",
    "memory_strided4",
    "memory_gather",
    "memory_interleave4",
    "dot_q8",
    "dot_q4",
    "dot_q6",
    "dot_scalar",
    "vopd_independent",
    "vopd_dependent",
    "vopd_mixed",
    "vopd_dequant",
    "sampler_argmax",
    "sampler_topk",
    "two_stage_partial",
    "two_stage_final",
    "q8_1_quantize_q4",
    "q8_1_quantize_q6",
    "q8_1_quantize_dense",
    "q4_selected_dual",
    "q6_x8",
    "dense_q8",
    "dense_q8_single",
];

fn run(mut command: Command, what: &str) -> Result<()> {
    let output = command
        .output()
        .with_context(|| format!("failed to start {what}"))?;
    if !output.status.success() {
        bail!(
            "{what} failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

fn find_glslc() -> PathBuf {
    if let Some(path) = env::var_os("GLSLC") {
        return path.into();
    }
    let pinned = Path::new("/tmp/shaderc-2025/root/usr/bin/glslc");
    if pinned.exists() {
        return pinned.to_owned();
    }
    PathBuf::from("glslc")
}

fn build_hip_code_object(
    hipcc: &Path,
    arch: &str,
    output: &Path,
    wave64: bool,
    scheduler_profile: SchedulerProfile,
) -> Result<()> {
    let wavefront = if wave64 {
        Wavefront::Wave64
    } else {
        Wavefront::Wave32
    };
    let mut request = CompileRequest::new("kernels/hipfire_6409.hip", output, arch)
        .wavefront(wavefront)
        .scheduler_profile(scheduler_profile)
        .hipcc(hipcc)
        .manifest(output.with_extension("radiowave.json"));
    if wave64 {
        request = request.define("HIPFIRE_BENCH_WAVE64=1");
    }
    if !arch.starts_with("gfx10") {
        request = request.define("HIPFIRE_BENCH_DOT8=1");
    }
    if let Ok(args) = env::var("RADIOWAVE_HIP_ARGS") {
        request
            .extra_args
            .extend(args.split_ascii_whitespace().map(Into::into));
    }
    Compiler.compile(&request).with_context(|| {
        format!(
            "Radiowave failed to build the wave{} HIP code object",
            wavefront.width(),
        )
    })?;
    Ok(())
}

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=kernels/hipfire_6409.hip");
    println!("cargo:rerun-if-changed=kernels/hipfire_6409.comp");
    println!(
        "cargo:rerun-if-changed={}",
        radiowave::support_header_path().display()
    );
    println!("cargo:rerun-if-env-changed=HIPFIRE_BENCH_ARCH");
    println!("cargo:rerun-if-env-changed=HIPCC");
    println!("cargo:rerun-if-env-changed=RADIOWAVE_HIP_ARGS");
    println!("cargo:rerun-if-env-changed=GLSLC");

    let out = PathBuf::from(env::var_os("OUT_DIR").context("OUT_DIR is unset")?);
    let arch = env::var("HIPFIRE_BENCH_ARCH").unwrap_or_else(|_| "gfx1201".to_owned());
    let hipcc = PathBuf::from(env::var_os("HIPCC").unwrap_or_else(|| "hipcc".into()));
    for wave64 in [false, true] {
        let width = if wave64 { 64 } else { 32 };
        for scheduler_profile in SchedulerProfile::ALL {
            let suffix = if scheduler_profile == SchedulerProfile::Default {
                String::new()
            } else {
                format!("_{}", scheduler_profile.as_str())
            };
            let output = out.join(format!("hipfire_6409_wave{width}{suffix}.hsaco"));
            build_hip_code_object(&hipcc, &arch, &output, wave64, scheduler_profile)?;
        }
    }

    let glslc = find_glslc();
    for shader in SHADERS {
        let output = out.join(format!("{shader}.spv"));
        let define = format!("-DKERNEL_{}=1", shader.to_ascii_uppercase());
        let mut glsl = Command::new(&glslc);
        if glslc.starts_with("/tmp/shaderc-2025/root") {
            glsl.env(
                "LD_LIBRARY_PATH",
                "/tmp/shaderc-2025/root/usr/lib/x86_64-linux-gnu",
            );
        }
        glsl.arg("--target-env=vulkan1.3")
            .arg("-O")
            .arg(define)
            .arg("-o")
            .arg(output)
            .arg("kernels/hipfire_6409.comp");
        run(glsl, &format!("glslc {shader}"))?;
    }
    Ok(())
}
