// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

use radiowave::{
    CompileRequest, Compiler, Inspector, SchedulerProfile, Wavefront, support_header_path,
};
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args_os().skip(1);
    match args.next().as_deref().and_then(|value| value.to_str()) {
        Some("compile") => compile(args.collect()),
        Some("inspect") => inspect(args.collect()),
        Some("oracle") => oracle(args.collect()),
        Some("recipes") => recipes(args.collect()),
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
    let mut arch = env::var("RADIOWAVE_ARCH").ok();
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
            Some("--arch") => {
                arch = Some(next(&mut iter, "--arch")?.to_string_lossy().into_owned())
            }
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
    let arch = arch.ok_or("compile requires --arch or RADIOWAVE_ARCH")?;
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
    let mut arch = env::var("RADIOWAVE_ARCH").ok();
    let mut hipcc = env::var_os("HIPCC")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("hipcc"));
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.to_str() {
            Some("--input") => input = Some(PathBuf::from(next(&mut iter, "--input")?)),
            Some("--arch") => {
                arch = Some(next(&mut iter, "--arch")?.to_string_lossy().into_owned())
            }
            Some("--hipcc") => hipcc = PathBuf::from(next(&mut iter, "--hipcc")?),
            Some("--help" | "-h") => {
                usage();
                return Ok(());
            }
            _ => return Err(format!("unknown inspect argument {}", arg.to_string_lossy()).into()),
        }
    }
    let input = input.ok_or("inspect requires --input")?;
    let arch = arch.ok_or("inspect requires --arch or RADIOWAVE_ARCH")?;
    let inspection = Inspector::from_hipcc(&hipcc).inspect(&input, &arch)?;
    println!("{}", serde_json::to_string_pretty(&inspection)?);
    Ok(())
}

fn oracle(args: Vec<OsString>) -> Result<(), Box<dyn Error>> {
    use radiowave::CompileManifest;
    use radiowave::oracle::{InputRelationship, OracleComparison, OracleMetadata, OracleReport};

    let mut iter = args.into_iter();
    match iter.next().as_deref().and_then(|value| value.to_str()) {
        Some("hip") => {
            let mut manifest = None;
            let mut kernel = None;
            let mut workgroup = None;
            let mut output = None;
            while let Some(arg) = iter.next() {
                match arg.to_str() {
                    Some("--manifest") => {
                        manifest = Some(PathBuf::from(next(&mut iter, "--manifest")?))
                    }
                    Some("--kernel") => {
                        kernel = Some(next(&mut iter, "--kernel")?.to_string_lossy().into_owned())
                    }
                    Some("--workgroup") => {
                        workgroup = Some(parse_workgroup(&next(&mut iter, "--workgroup")?)?)
                    }
                    Some("--output") => output = Some(PathBuf::from(next(&mut iter, "--output")?)),
                    Some("--help" | "-h") => {
                        usage();
                        return Ok(());
                    }
                    _ => {
                        return Err(format!(
                            "unknown oracle hip argument {}",
                            arg.to_string_lossy()
                        )
                        .into());
                    }
                }
            }
            let manifest = manifest.ok_or("oracle hip requires --manifest")?;
            let kernel = kernel.ok_or("oracle hip requires --kernel")?;
            let workgroup = workgroup.ok_or("oracle hip requires --workgroup")?;
            let manifest: CompileManifest = serde_json::from_str(&fs::read_to_string(manifest)?)?;
            let report = OracleReport::from_hip_manifest(&manifest, &kernel, workgroup)?;
            write_json(output.as_ref(), &report)
        }
        Some(compiler @ ("aco" | "llpc")) => {
            let mut input = None;
            let mut input_artifact = None;
            let mut kernel = None;
            let mut arch = env::var("RADIOWAVE_ARCH").ok();
            let mut workgroup = [0, 0, 0];
            let mut wavefront = 0;
            let mut compiler_version = String::new();
            let mut relationship = InputRelationship::Exact;
            let mut symbol = "_amdgpu_cs_main".to_owned();
            let mut output = None;
            while let Some(arg) = iter.next() {
                match arg.to_str() {
                    Some("--input") => input = Some(PathBuf::from(next(&mut iter, "--input")?)),
                    Some("--input-artifact") => {
                        input_artifact = Some(PathBuf::from(next(&mut iter, "--input-artifact")?))
                    }
                    Some("--kernel") => {
                        kernel = Some(next(&mut iter, "--kernel")?.to_string_lossy().into_owned())
                    }
                    Some("--arch") => {
                        arch = Some(next(&mut iter, "--arch")?.to_string_lossy().into_owned())
                    }
                    Some("--workgroup") => {
                        workgroup = parse_workgroup(&next(&mut iter, "--workgroup")?)?
                    }
                    Some("--wavefront") => {
                        wavefront = parse_wavefront(&next(&mut iter, "--wavefront")?)?
                    }
                    Some("--compiler-version") => {
                        compiler_version = next(&mut iter, "--compiler-version")?
                            .to_string_lossy()
                            .into_owned()
                    }
                    Some("--symbol") => {
                        symbol = next(&mut iter, "--symbol")?.to_string_lossy().into_owned()
                    }
                    Some("--semantic") => relationship = InputRelationship::Semantic,
                    Some("--output") => output = Some(PathBuf::from(next(&mut iter, "--output")?)),
                    Some("--help" | "-h") => {
                        usage();
                        return Ok(());
                    }
                    _ => {
                        return Err(format!(
                            "unknown oracle {compiler} argument {}",
                            arg.to_string_lossy()
                        )
                        .into());
                    }
                }
            }
            let input = input.ok_or_else(|| format!("oracle {compiler} requires --input"))?;
            let kernel = kernel.ok_or_else(|| format!("oracle {compiler} requires --kernel"))?;
            let arch =
                arch.ok_or_else(|| format!("oracle {compiler} requires --arch or RADIOWAVE_ARCH"))?;
            let mut metadata = OracleMetadata::new(kernel, arch, workgroup);
            metadata.compiler_version = compiler_version;
            metadata.wavefront_size = wavefront;
            metadata.input_relationship = relationship;
            if let Some(input_artifact) = input_artifact {
                metadata = metadata.input_artifact(&input_artifact)?;
            }
            let encoded = fs::read_to_string(input)?;
            let report = if compiler == "aco" {
                OracleReport::from_aco_dump(&encoded, metadata)?
            } else {
                OracleReport::from_llpc_assembly(&encoded, &symbol, metadata)?
            };
            write_json(output.as_ref(), &report)
        }
        Some("compare") => {
            let mut baseline = None;
            let mut candidate_paths = Vec::new();
            let mut output = None;
            while let Some(arg) = iter.next() {
                match arg.to_str() {
                    Some("--baseline") => {
                        baseline = Some(PathBuf::from(next(&mut iter, "--baseline")?))
                    }
                    Some("--candidate") => {
                        candidate_paths.push(PathBuf::from(next(&mut iter, "--candidate")?))
                    }
                    Some("--output") => output = Some(PathBuf::from(next(&mut iter, "--output")?)),
                    Some("--help" | "-h") => {
                        usage();
                        return Ok(());
                    }
                    _ => {
                        return Err(format!(
                            "unknown oracle compare argument {}",
                            arg.to_string_lossy()
                        )
                        .into());
                    }
                }
            }
            let baseline = baseline.ok_or("oracle compare requires --baseline")?;
            if candidate_paths.is_empty() {
                return Err("oracle compare requires at least one --candidate".into());
            }
            let baseline: OracleReport = serde_json::from_str(&fs::read_to_string(baseline)?)?;
            let mut candidates = Vec::with_capacity(candidate_paths.len());
            for path in candidate_paths {
                candidates.push(serde_json::from_str(&fs::read_to_string(path)?)?);
            }
            let comparison = OracleComparison::new(baseline, candidates)?;
            write_json(output.as_ref(), &comparison)
        }
        Some("--help" | "-h") | None => {
            usage();
            Ok(())
        }
        Some(command) => Err(format!("unknown oracle command {command}").into()),
    }
}

fn parse_workgroup(value: &OsString) -> Result<[u32; 3], Box<dyn Error>> {
    let encoded = value.to_string_lossy().replace('x', ",");
    let values = encoded
        .split(',')
        .map(|value| value.trim().parse::<u32>())
        .collect::<Result<Vec<_>, _>>()?;
    let dimensions = match values.as_slice() {
        [x] => [*x, 1, 1],
        [x, y, z] => [*x, *y, *z],
        _ => return Err("workgroup must be X or X,Y,Z".into()),
    };
    if dimensions.contains(&0) {
        return Err("workgroup dimensions must be non-zero".into());
    }
    Ok(dimensions)
}

fn parse_wavefront(value: &OsString) -> Result<u32, Box<dyn Error>> {
    let value = value.to_string_lossy().parse::<u32>()?;
    if matches!(value, 32 | 64) {
        Ok(value)
    } else {
        Err("wavefront must be 32 or 64".into())
    }
}

fn write_json<T: serde::Serialize>(
    output: Option<&PathBuf>,
    value: &T,
) -> Result<(), Box<dyn Error>> {
    let encoded = serde_json::to_string_pretty(value)?;
    if let Some(output) = output {
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(output, format!("{encoded}\n"))?;
    } else {
        println!("{encoded}");
    }
    Ok(())
}

fn recipes(args: Vec<OsString>) -> Result<(), Box<dyn Error>> {
    use radiowave::recipes::{RecipeCatalog, SelectionMode, WorkloadDescriptor};

    let mut iter = args.into_iter();
    match iter.next().as_deref().and_then(|value| value.to_str()) {
        Some("builtin") => {
            let mut output = None;
            while let Some(arg) = iter.next() {
                match arg.to_str() {
                    Some("--output") => output = Some(PathBuf::from(next(&mut iter, "--output")?)),
                    Some("--help" | "-h") => {
                        usage();
                        return Ok(());
                    }
                    _ => {
                        return Err(format!(
                            "unknown recipes builtin argument {}",
                            arg.to_string_lossy()
                        )
                        .into());
                    }
                }
            }
            let encoded = RecipeCatalog::builtin_hipfire_6409().to_json_pretty()?;
            if let Some(output) = output {
                if let Some(parent) = output.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(output, encoded)?;
            } else {
                print!("{encoded}");
            }
            Ok(())
        }
        Some("select") => {
            let mut catalog = None;
            let mut arch = env::var("RADIOWAVE_ARCH").ok();
            let mut kernel = None;
            let mut family = String::new();
            let mut tags = Vec::new();
            let mut mode = SelectionMode::Certified;
            while let Some(arg) = iter.next() {
                match arg.to_str() {
                    Some("--catalog") => {
                        catalog = Some(PathBuf::from(next(&mut iter, "--catalog")?))
                    }
                    Some("--arch") => {
                        arch = Some(next(&mut iter, "--arch")?.to_string_lossy().into_owned())
                    }
                    Some("--kernel") => {
                        kernel = Some(next(&mut iter, "--kernel")?.to_string_lossy().into_owned())
                    }
                    Some("--family") => {
                        family = next(&mut iter, "--family")?.to_string_lossy().into_owned()
                    }
                    Some("--tag") => {
                        tags.push(next(&mut iter, "--tag")?.to_string_lossy().into_owned())
                    }
                    Some("--candidates") => mode = SelectionMode::Candidates,
                    Some("--help" | "-h") => {
                        usage();
                        return Ok(());
                    }
                    _ => {
                        return Err(format!(
                            "unknown recipes select argument {}",
                            arg.to_string_lossy()
                        )
                        .into());
                    }
                }
            }
            let catalog = load_catalog(catalog.as_ref())?;
            let arch = arch.ok_or("recipes select requires --arch or RADIOWAVE_ARCH")?;
            let kernel = kernel.ok_or("recipes select requires --kernel")?;
            let mut workload = WorkloadDescriptor::new(kernel, family);
            for tag in tags {
                workload = workload.tag(tag);
            }
            let selection = catalog.select(arch, workload, mode)?;
            println!("{}", serde_json::to_string_pretty(&selection)?);
            Ok(())
        }
        Some("ingest") => {
            let mut catalog = None;
            let mut ledgers = Vec::new();
            let mut output = None;
            while let Some(arg) = iter.next() {
                match arg.to_str() {
                    Some("--catalog") => {
                        catalog = Some(PathBuf::from(next(&mut iter, "--catalog")?))
                    }
                    Some("--ledger") => ledgers.push(PathBuf::from(next(&mut iter, "--ledger")?)),
                    Some("--output") => output = Some(PathBuf::from(next(&mut iter, "--output")?)),
                    Some("--help" | "-h") => {
                        usage();
                        return Ok(());
                    }
                    _ => {
                        return Err(format!(
                            "unknown recipes ingest argument {}",
                            arg.to_string_lossy()
                        )
                        .into());
                    }
                }
            }
            if ledgers.is_empty() {
                return Err("recipes ingest requires at least one --ledger".into());
            }
            let output = output.ok_or("recipes ingest requires --output")?;
            let mut catalog = load_catalog(catalog.as_ref())?;
            let mut inserted = 0;
            for ledger in ledgers {
                inserted += catalog.ingest_autoresearch_jsonl(&fs::read_to_string(&ledger)?)?;
            }
            if let Some(parent) = output.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&output, catalog.to_json_pretty()?)?;
            eprintln!(
                "ingested {inserted} promoted recipe evidence rows into {}",
                output.display()
            );
            Ok(())
        }
        Some("--help" | "-h") | None => {
            usage();
            Ok(())
        }
        Some(command) => Err(format!("unknown recipes command {command}").into()),
    }
}

fn load_catalog(
    path: Option<&PathBuf>,
) -> Result<radiowave::recipes::RecipeCatalog, Box<dyn Error>> {
    let catalog = if let Some(path) = path {
        radiowave::recipes::RecipeCatalog::from_json(&fs::read_to_string(path)?)?
    } else {
        radiowave::recipes::RecipeCatalog::builtin_hipfire_6409()
    };
    catalog.validate()?;
    Ok(catalog)
}

fn next(iter: &mut impl Iterator<Item = OsString>, option: &str) -> Result<OsString, String> {
    iter.next()
        .ok_or_else(|| format!("{option} requires a value"))
}

fn usage() {
    println!(
        "radiowave compile --source KERNEL.hip --output KERNEL.hsaco --arch TARGET [--wave32|--wave64] [--scheduler-profile default|max-ilp|iterative-ilp|memory-clause|pipeline-ilp] [--define NAME=VALUE] [--arg FLAG] [--manifest PATH] [--no-inspect]\n\
         radiowave inspect --input KERNEL.hsaco --arch TARGET\n\
         radiowave oracle hip --manifest MANIFEST.json --kernel NAME --workgroup X[,Y,Z] [--output REPORT.json]\n\
         radiowave oracle aco --input ACO.dump --input-artifact SHADER.spv --kernel NAME --arch TARGET --wavefront 32|64 [--workgroup X,Y,Z] [--compiler-version TEXT] [--output REPORT.json]\n\
         radiowave oracle llpc --input LLPC.s --input-artifact SHADER.spv --kernel NAME --arch TARGET [--symbol NAME] [--workgroup X,Y,Z] [--wavefront 32|64] [--compiler-version TEXT] [--output REPORT.json]\n\
         radiowave oracle compare --baseline REPORT.json --candidate REPORT.json [--candidate REPORT.json ...] [--output COMPARISON.json]\n\
         radiowave recipes builtin [--output CATALOG.json]\n\
         radiowave recipes select --arch TARGET --kernel NAME [--family FAMILY] [--tag TAG] [--catalog CATALOG.json] [--candidates]\n\
         radiowave recipes ingest [--catalog CATALOG.json] --ledger WIN_ROWS.jsonl --output CATALOG.json\n\
         radiowave header"
    );
}
