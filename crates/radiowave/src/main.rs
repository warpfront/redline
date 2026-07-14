// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

use radiowave::{
    CompileRequest, Compiler, Inspector, SchedulerProfile, Wavefront, support_header_path,
};
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args_os().skip(1);
    match args.next().as_deref().and_then(|value| value.to_str()) {
        Some("compile") => compile(args.collect()),
        Some("inspect") => inspect(args.collect()),
        Some("header") => {
            println!("{}", support_header_path().display());
            Ok(())
        }
        Some("--help" | "-h") | None => {
            usage();
            Ok(())
        }
        Some(command) => Err(format!("unknown command {command}").into()),
    }
}

fn compile(args: Vec<OsString>) -> Result<(), Box<dyn Error>> {
    let mut source = None;
    let mut output = None;
    let mut arch = "gfx1201".to_owned();
    let mut wavefront = Wavefront::Wave32;
    let mut scheduler_profile = SchedulerProfile::Default;
    let mut hipcc = env::var_os("HIPCC").map(PathBuf::from);
    let mut manifest = None;
    let mut defines = Vec::new();
    let mut extra_args = Vec::new();
    let mut inspect = true;
    let mut fast_math = true;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.to_str() {
            Some("--source") => source = Some(PathBuf::from(next(&mut iter, "--source")?)),
            Some("--output") => output = Some(PathBuf::from(next(&mut iter, "--output")?)),
            Some("--arch") => arch = next(&mut iter, "--arch")?.to_string_lossy().into_owned(),
            Some("--wave32") => wavefront = Wavefront::Wave32,
            Some("--wave64") => wavefront = Wavefront::Wave64,
            Some("--scheduler-profile") => {
                let value = next(&mut iter, "--scheduler-profile")?;
                scheduler_profile = SchedulerProfile::parse(&value.to_string_lossy())
                    .ok_or_else(|| {
                        format!(
                            "unknown scheduler profile {}; expected default, max-ilp, iterative-ilp, memory-clause, or pipeline-ilp",
                            value.to_string_lossy()
                        )
                    })?;
            }
            Some("--hipcc") => hipcc = Some(PathBuf::from(next(&mut iter, "--hipcc")?)),
            Some("--manifest") => manifest = Some(PathBuf::from(next(&mut iter, "--manifest")?)),
            Some("--define") => {
                defines.push(next(&mut iter, "--define")?.to_string_lossy().into_owned())
            }
            Some("--arg") => extra_args.push(next(&mut iter, "--arg")?),
            Some("--no-inspect") => inspect = false,
            Some("--no-fast-math") => fast_math = false,
            Some("--help" | "-h") => {
                usage();
                return Ok(());
            }
            _ => return Err(format!("unknown compile argument {}", arg.to_string_lossy()).into()),
        }
    }
    let source = source.ok_or("compile requires --source")?;
    let output = output.ok_or("compile requires --output")?;
    let manifest = manifest.unwrap_or_else(|| output.with_extension("radiowave.json"));
    let mut request = CompileRequest::new(source, output, arch)
        .wavefront(wavefront)
        .scheduler_profile(scheduler_profile)
        .manifest(manifest);
    if let Some(hipcc) = hipcc {
        request = request.hipcc(hipcc);
    }
    request.defines = defines;
    request.extra_args = extra_args;
    request.inspect = inspect;
    request.fast_math = fast_math;
    let artifact = Compiler.compile(&request)?;
    println!(
        "compiled {} sha256={} kernels={} manifest={}",
        artifact.output.display(),
        artifact.output_sha256,
        artifact
            .inspection
            .as_ref()
            .map_or(0, |inspection| inspection.kernels.len()),
        artifact
            .manifest
            .as_ref()
            .map_or_else(|| "none".to_owned(), |path| path.display().to_string())
    );
    Ok(())
}

fn inspect(args: Vec<OsString>) -> Result<(), Box<dyn Error>> {
    let mut input = None;
    let mut arch = "gfx1201".to_owned();
    let mut hipcc = env::var_os("HIPCC")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("hipcc"));
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.to_str() {
            Some("--input") => input = Some(PathBuf::from(next(&mut iter, "--input")?)),
            Some("--arch") => arch = next(&mut iter, "--arch")?.to_string_lossy().into_owned(),
            Some("--hipcc") => hipcc = PathBuf::from(next(&mut iter, "--hipcc")?),
            Some("--help" | "-h") => {
                usage();
                return Ok(());
            }
            _ => return Err(format!("unknown inspect argument {}", arg.to_string_lossy()).into()),
        }
    }
    let input = input.ok_or("inspect requires --input")?;
    let inspection = Inspector::from_hipcc(&hipcc).inspect(&input, &arch)?;
    println!("{}", serde_json::to_string_pretty(&inspection)?);
    Ok(())
}

fn next(iter: &mut impl Iterator<Item = OsString>, option: &str) -> Result<OsString, String> {
    iter.next()
        .ok_or_else(|| format!("{option} requires a value"))
}

fn usage() {
    println!(
        "radiowave compile --source KERNEL.hip --output KERNEL.hsaco [--arch gfx1201] [--wave32|--wave64] [--scheduler-profile default|max-ilp|iterative-ilp|memory-clause|pipeline-ilp] [--define NAME=VALUE] [--arg FLAG] [--manifest PATH] [--no-inspect]\n\
         radiowave inspect --input KERNEL.hsaco [--arch gfx1201]\n\
         radiowave header"
    );
}
