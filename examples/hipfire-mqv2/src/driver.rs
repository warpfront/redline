// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

use crate::hip_backend::{HipBackend, HipGraphBackend};
use crate::kernels;
use crate::oracle;
use crate::redline_backend::{RedlineBackend, RmwBoundary};
use crate::report::{self, RowReport};
use crate::rocm_provenance;
use crate::spec::{ShapeSet, matrix, shapes_for_set};
use crate::types::{Arch, Backend, BackendResult, Distribution, Fixture, RowSpec, TimingMode, Verdict};
use anyhow::{Context, Result, bail};
use radiowave::SchedulerProfile;
use redline_dispatch::aql::QueuePolicy;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug)]
struct Config {
    backends: Vec<String>,
    filter: Option<String>,
    shapes: ShapeSet,
    modes: Vec<TimingMode>,
    warmups: usize,
    samples: usize,
    iterations: usize,
    scheduler_profiles: Vec<SchedulerProfile>,
    device_ordinal: i32,
    redline_queues: String,
    redline_rmw: String,
    out: Option<PathBuf>,
    list: bool,
    dump_mismatch: Option<usize>,
}

fn parse_args() -> Result<Config> {
    let mut backends: Option<Vec<String>> = None;
    let mut filter: Option<String> = None;
    let mut shapes = ShapeSet::Smoke;
    let mut modes: Option<Vec<TimingMode>> = None;
    let mut warmups = 2usize;
    let mut samples = 5usize;
    let mut iterations = 4usize;
    let mut scheduler_profiles: Option<Vec<SchedulerProfile>> = None;
    let mut device_ordinal = 0i32;
    let mut redline_queues = "auto".to_string();
    let mut redline_rmw = "radiowave-vmem".to_string();
    let mut out: Option<PathBuf> = None;
    let mut list = false;
    let mut dump_mismatch: Option<usize> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--backends" => { i+=1; let v = args.get(i).context("missing --backends value")?.clone(); if v=="all" { backends = Some(vec!["hip".to_string(),"hipgraph".to_string(),"redline".to_string()]); } else { backends = Some(v.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()); } }
            "--filter" => { i+=1; filter = Some(args.get(i).context("missing --filter")?.clone()); }
            "--shapes" => { i+=1; let v = args.get(i).context("missing --shapes")?.clone(); shapes = ShapeSet::parse(&v).with_context(|| format!("unknown shapes {v}"))?; }
            "--modes" => { i+=1; let v = args.get(i).context("missing --modes")?.clone(); modes = Some(match v.as_str() { "serial" => vec![TimingMode::SerialLatency], "independent" => vec![TimingMode::IndependentThroughput], "all" => vec![TimingMode::SerialLatency, TimingMode::IndependentThroughput], _ => bail!("unknown modes {v}") }); }
            "--warmups" => { i+=1; warmups = args.get(i).context("missing --warmups")?.parse()?; }
            "--samples" => { i+=1; samples = args.get(i).context("missing --samples")?.parse()?; }
            "--iterations" => { i+=1; iterations = args.get(i).context("missing --iterations")?.parse()?; }
            "--scheduler-profile" => { i+=1; let v = args.get(i).context("missing --scheduler-profile")?.clone(); if v=="all" { scheduler_profiles = Some(SchedulerProfile::ALL.to_vec()); } else { let p = match v.as_str() { "default" => SchedulerProfile::Default, "max_ilp" => SchedulerProfile::MaxIlp, "iterative_ilp" => SchedulerProfile::IterativeIlp, "memory_clause" => SchedulerProfile::MemoryClause, "pipeline_ilp" => SchedulerProfile::PipelineIlp, _ => bail!("unknown scheduler profile {v}") }; scheduler_profiles = Some(vec![p]); } }
            "--device-ordinal" => { i+=1; device_ordinal = args.get(i).context("missing --device-ordinal")?.parse()?; }
            "--redline-queues" => { i+=1; redline_queues = args.get(i).context("missing --redline-queues")?.clone(); }
            "--redline-rmw" => { i+=1; redline_rmw = args.get(i).context("missing --redline-rmw")?.clone(); }
            "--dump-mismatch" => { i+=1; dump_mismatch = Some(args.get(i).context("missing --dump-mismatch")?.parse()?); }
            "--out" => { i+=1; out = Some(PathBuf::from(args.get(i).context("missing --out")?.clone())); }
            "--list" => { list = true; }
            "--help" | "-h" => { print_help(); std::process::exit(0); }
            other => { bail!("unknown arg {other}") }
        }
        i+=1;
    }
    let backends = backends.unwrap_or_else(|| vec!["hip".to_string(),"hipgraph".to_string(),"redline".to_string()]);
    let modes = modes.unwrap_or_else(|| vec![TimingMode::SerialLatency, TimingMode::IndependentThroughput]);
    let scheduler_profiles = scheduler_profiles.unwrap_or_else(|| vec![SchedulerProfile::Default]);

    // validate backends
    for b in &backends {
        if !["hip","hipgraph","redline"].contains(&b.as_str()) { bail!("unsupported backend {b}") }
    }
    Ok(Config { backends, filter, shapes, modes, warmups, samples, iterations, scheduler_profiles, device_ordinal, redline_queues, redline_rmw, out, list, dump_mismatch })
}

fn print_help() {
    eprintln!("hipfire-mqv2-bench [--backends all|hip,hipgraph,redline] [--filter TEXT] [--shapes smoke|prefill|all] [--modes serial|independent|all] [--warmups N] [--samples N] [--iterations N] [--scheduler-profile <p>|all] [--device-ordinal N] [--redline-queues auto|N] [--redline-rmw radiowave-vmem|same-agent|radv-global] [--out PATH] [--list] [--dump-mismatch N]");
}

fn command_output(program: &str, args: &[&str]) -> String {
    Command::new(program).args(args).output().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_else(|e| format!("failed to run {program}: {e}"))
}

fn sha256_hex(bytes: &[u8]) -> String { format!("{:x}", Sha256::digest(bytes)) }

fn flatten_f32(proj: &[Vec<f32>]) -> Vec<f32> {
    let mut out = Vec::new();
    for p in proj { out.extend_from_slice(p); }
    out
}

fn hash_outputs(outputs: &[Vec<Vec<f32>>]) -> String {
    let mut hasher = Sha256::new();
    for set in outputs {
        for proj in set {
            for &v in proj { hasher.update(v.to_bits().to_ne_bytes()); }
        }
    }
    format!("{:x}", hasher.finalize())
}

fn verify_all(fixture: &Fixture, row: &RowSpec, outputs: &[Vec<Vec<f32>>]) -> Verdict {
    // outputs: per Y set, per projection.
    // For serial: 1 set with iterations launches; for independent: iterations sets each 1 launch.
    let is_serial = row.mode == TimingMode::SerialLatency;
    let mut all_expected: Vec<f32> = Vec::new();
    let mut all_actual: Vec<f32> = Vec::new();
    if is_serial {
        // Single Y set, expected after iterations launches
        let expected = fixture.expected_after(row.kernel.family, row.iterations);
        let actual_set = &outputs[0];
        for (proj_idx, exp_proj) in expected.iter().enumerate() {
            all_expected.extend_from_slice(exp_proj);
            all_actual.extend_from_slice(&actual_set[proj_idx]);
        }
    } else {
        // Each set 1 launch
        let expected_once = fixture.expected_after(row.kernel.family, 1);
        for set in outputs {
            for (proj_idx, exp_proj) in expected_once.iter().enumerate() {
                all_expected.extend_from_slice(exp_proj);
                all_actual.extend_from_slice(&set[proj_idx]);
            }
        }
    }
    oracle::verify(&all_expected, &all_actual)
}

pub fn run() -> Result<()> {
    let config = parse_args()?;

    // Init HIP to discover arch
    let hip_probe = HipBackend::new(config.device_ordinal);
    let (arch, arch_str) = match &hip_probe {
        Ok(h) => (h.arch, h.arch_str.clone()),
        Err(_) => {
            // fallback to gfx1201 if no GPU present (for --list)
            (Arch::Gfx1201, "gfx1201".to_string())
        }
    };
    // For --list, need matrix without initializing backends? Use discovered arch.
    let rows = matrix(arch, config.shapes, &config.modes, &config.scheduler_profiles, config.iterations);
    let filtered_rows: Vec<RowSpec> = rows.into_iter().filter(|r| {
        if let Some(f) = &config.filter { r.key().contains(f) } else { true }
    }).collect();

    if config.list {
        for r in &filtered_rows {
            println!("{}", r.key());
        }
        return Ok(());
    }

    if filtered_rows.is_empty() { bail!("no rows matched filter"); }

    // Print provenance
    if let Ok(hip) = HipBackend::new(config.device_ordinal) {
        let prov = rocm_provenance::collect(&hip.hip);
        println!("ROCm provenance hip_runtime_version_raw={} libamdhip64={} libhsa={} mixed={}", prov["hip_runtime_version_raw"], prov["libamdhip64_path"], prov["libhsa_runtime_path"], prov["mixed_load_warning"]);
        println!("arch={} hip_device={} queue_policy_redline={} rmw={}", arch_str, arch.as_str(), config.redline_queues, config.redline_rmw);
    } else {
        println!("ROCm provenance unavailable (no HIP device)");
    }
    // HIPCC version
    let hipcc_path = std::env::var("HIPCC").unwrap_or_else(|_| "/opt/rocm/core-10.0/bin/hipcc".to_string());
    let hipcc_version = command_output(&hipcc_path, &["--version"]);
    println!("hipcc: {}", hipcc_version.lines().next().unwrap_or(""));

    // HSACO sha256 per profile
    let mut hsaco_sha: BTreeMap<String,String> = BTreeMap::new();
    for &p in &config.scheduler_profiles {
        let code = kernels::code_object(arch, p);
        if !code.is_empty() {
            hsaco_sha.insert(p.as_str().to_string(), sha256_hex(code));
        }
    }
    println!("hsaco sha256: {:?}", hsaco_sha);

    // Build backends
    // We will lazily create backends per need.
    let queue_policy: QueuePolicy = config.redline_queues.parse().unwrap_or(QueuePolicy::Auto);
    let rmw_boundary = RmwBoundary::parse(&config.redline_rmw).unwrap_or(RmwBoundary::RadiowaveVmem);

    // For incremental JSON, keep RowReports
    let mut reports: Vec<RowReport> = Vec::new();

    // Prepare environment json
    let hip_for_env = HipBackend::new(config.device_ordinal).ok();
    let redline_for_env = RedlineBackend::new("", queue_policy, rmw_boundary).ok();
    let environment = json!({
        "arch": arch.as_str(),
        "arch_str": arch_str,
        "hip_arch": hip_for_env.as_ref().map(|h| h.arch_str.clone()).unwrap_or(arch_str.clone()),
        "hip_device_ordinal": config.device_ordinal,
        "redline_arch": redline_for_env.as_ref().map(|r| r.arch_str().to_string()).unwrap_or_default(),
        "redline_pci": redline_for_env.as_ref().map(|r| r.pci().to_string()).unwrap_or_default(),
        "rocm_provenance": hip_for_env.as_ref().map(|h| rocm_provenance::collect(&h.hip)).unwrap_or(Value::Null),
        "hipcc_version": hipcc_version,
        "hsaco_sha256_by_profile": hsaco_sha,
        "device_names": {
            "hip": hip_for_env.as_ref().map(|h| h.arch_str.clone()),
            "redline": redline_for_env.as_ref().map(|r| r.arch_str().to_string()),
        }
    });

    let config_json = json!({
        "backends": config.backends,
        "filter": config.filter,
        "shapes": config.shapes.as_str(),
        "modes": config.modes.iter().map(|m| m.as_str()).collect::<Vec<_>>(),
        "warmups": config.warmups,
        "samples": config.samples,
        "iterations": config.iterations,
        "scheduler_profiles": config.scheduler_profiles.iter().map(|p| p.as_str()).collect::<Vec<_>>(),
        "device_ordinal": config.device_ordinal,
        "redline_queues": config.redline_queues,
        "redline_rmw": config.redline_rmw,
        "arch": arch.as_str(),
    });

    // Instantiate backends once (hip, hipgraph, redline) for reuse
    let mut hip_backend: Option<HipBackend> = if config.backends.contains(&"hip".to_string()) { HipBackend::new(config.device_ordinal).ok() } else { None };
    let mut hipgraph_backend: Option<HipGraphBackend> = if config.backends.contains(&"hipgraph".to_string()) { HipGraphBackend::new(config.device_ordinal).ok() } else { None };
    let mut redline_backend: Option<RedlineBackend> = if config.backends.contains(&"redline".to_string()) {
        // Need PCI: use hip arch pci? Use empty (ordinal 0) if none
        let pci = hip_for_env.as_ref().map(|_| "".to_string()).unwrap_or_default();
        match RedlineBackend::new(&pci, queue_policy, rmw_boundary) {
            Ok(b) => Some(b),
            Err(e) => { eprintln!("redline init failed: {e:#}"); None },
        }
    } else { None };

    for (row_idx, row) in filtered_rows.iter().enumerate() {
        let fixture = crate::fixture::build(&row.kernel, &row.shape, row_idx as u64);
        let mut backends_map: BTreeMap<String, BackendResult> = BTreeMap::new();
        let mut output_hashes: Vec<String> = Vec::new();

        for backend_name in &config.backends {
            let result = match backend_name.as_str() {
                "hip" => {
                    if let Some(b) = hip_backend.as_mut() {
                        run_one(b, row, &fixture, config.warmups, config.samples, config.dump_mismatch)
                    } else { BackendResult { correctness: Verdict { pass: false, rel_rms: f64::NAN, max_abs: f64::NAN, compared: 0, note: Some("hip backend init failed".to_string()) }, distribution: Distribution { min_us: f64::NAN, p05_us: f64::NAN, median_us: f64::NAN, p95_us: f64::NAN, max_us: f64::NAN, samples_us: vec![] }, output_sha256: String::new(), notes: Value::Null, error: Some("hip init failed".to_string()) } }
                }
                "hipgraph" => {
                    if let Some(b) = hipgraph_backend.as_mut() {
                        run_one(b, row, &fixture, config.warmups, config.samples, config.dump_mismatch)
                    } else { BackendResult { correctness: Verdict { pass: false, rel_rms: f64::NAN, max_abs: f64::NAN, compared: 0, note: Some("hipgraph init failed".to_string()) }, distribution: Distribution { min_us: f64::NAN, p05_us: f64::NAN, median_us: f64::NAN, p95_us: f64::NAN, max_us: f64::NAN, samples_us: vec![] }, output_sha256: String::new(), notes: Value::Null, error: Some("hipgraph init failed".to_string()) } }
                }
                "redline" => {
                    if let Some(b) = redline_backend.as_mut() {
                        run_one(b, row, &fixture, config.warmups, config.samples, config.dump_mismatch)
                    } else { BackendResult { correctness: Verdict { pass: false, rel_rms: f64::NAN, max_abs: f64::NAN, compared: 0, note: Some("redline init failed".to_string()) }, distribution: Distribution { min_us: f64::NAN, p05_us: f64::NAN, median_us: f64::NAN, p95_us: f64::NAN, max_us: f64::NAN, samples_us: vec![] }, output_sha256: String::new(), notes: Value::Null, error: Some("redline init failed".to_string()) } }
                }
                _ => continue,
            };
            // Print one-line per row/backend
            if let Some(err) = &result.error {
                println!("[{}/{}] {} backend={} ERROR {}", row_idx+1, filtered_rows.len(), row.key(), backend_name, err);
            } else {
                println!("[{}/{}] {} backend={} median {:.2} us gate={} max_abs {:.3} rel_rms {:.4}", row_idx+1, filtered_rows.len(), row.key(), backend_name, result.distribution.median_us, if result.correctness.pass { "pass" } else { "FAIL" }, result.correctness.max_abs, result.correctness.rel_rms);
            }
            output_hashes.push(result.output_sha256.clone());
            backends_map.insert(backend_name.clone(), result);
        }

        let bit_identical = if output_hashes.len() > 1 && output_hashes.iter().all(|h| !h.is_empty()) {
            output_hashes.windows(2).all(|w| w[0]==w[1])
        } else if output_hashes.len()==1 { true } else { false };

        reports.push(RowReport { spec: row.clone(), backends: backends_map, bit_identical_across_backends: bit_identical });

        // Incremental write
        if let Some(out_path) = &config.out {
            let artifact = report::build_artifact(&reports, config_json.clone(), environment.clone());
            let partial = out_path.with_extension("partial.json");
            // Ensure parent exists
            if let Some(parent) = out_path.parent() { std::fs::create_dir_all(parent).ok(); }
            std::fs::write(&partial, serde_json::to_string_pretty(&artifact).unwrap()).ok();
            std::fs::write(out_path, serde_json::to_string_pretty(&artifact).unwrap()).ok();
        }
    }

    // Final write
    if let Some(out_path) = &config.out {
        let artifact = report::build_artifact(&reports, config_json, environment);
        if let Some(parent) = out_path.parent() { std::fs::create_dir_all(parent).ok(); }
        std::fs::write(out_path, serde_json::to_string_pretty(&artifact).unwrap()).context("write out")?;
        println!("wrote {}", out_path.display());
    } else if reports.is_empty() {
        // just print summary
    }

    Ok(())
}

fn run_one<B: Backend>(backend: &mut B, row: &RowSpec, fixture: &Fixture, warmups: usize, samples: usize, dump_n: Option<usize>) -> BackendResult {
    match backend.run(row, fixture, warmups, samples) {
        Ok(out) => {
            let verdict = verify_all(fixture, row, &out.outputs);
            if let Some(n) = dump_n {
                if !verdict.pass && n > 0 {
                    // Dump first n mismatched elements for serial arm (first Y set)
                    let is_serial = row.mode == TimingMode::SerialLatency;
                    let expected = if is_serial {
                        fixture.expected_after(row.kernel.family, row.iterations)
                    } else {
                        fixture.expected_after(row.kernel.family, 1)
                    };
                    // Compare per projection
                    let mut printed = 0;
                    for (proj_idx, (exp_proj, act_set)) in expected.iter().zip(out.outputs[0].iter()).enumerate() {
                        for (elem_idx, (exp, act)) in exp_proj.iter().zip(act_set.iter()).enumerate() {
                            if (exp - act).abs() > 1e-5 && printed < n {
                                eprintln!("  dump [proj {} elem {}] expected {:.6} actual {:.6} diff {:.6} y_init {:.6}", proj_idx, elem_idx, exp, act, exp-act, fixture.y_init[proj_idx][elem_idx]);
                                printed += 1;
                            }
                        }
                        if printed >= n { break; }
                    }
                    // Also dump first few expected vs actual for independent mode per set
                    if !is_serial && printed < n {
                        for set_idx in 0..out.outputs.len() {
                            for (proj_idx, exp_proj) in expected.iter().enumerate() {
                                let act_proj = &out.outputs[set_idx][proj_idx];
                                for (elem_idx, (exp, act)) in exp_proj.iter().zip(act_proj.iter()).enumerate() {
                                    if (exp - act).abs() > 1e-5 && printed < n {
                                        eprintln!("  dump indep set {} [proj {} elem {}] expected {:.6} actual {:.6} diff {:.6}", set_idx, proj_idx, elem_idx, exp, act, exp-act);
                                        printed += 1;
                                    }
                                }
                                if printed >= n { break; }
                            }
                            if printed >= n { break; }
                        }
                    }
                }
            }
            let sha = hash_outputs(&out.outputs);
            let dist = Distribution::from_samples(out.samples_us.clone());
            BackendResult { correctness: verdict, distribution: dist, output_sha256: sha, notes: out.notes, error: None }
        }
        Err(e) => {
            BackendResult {
                correctness: Verdict { pass: false, rel_rms: f64::NAN, max_abs: f64::NAN, compared: 0, note: Some(format!("{e:#}")) },
                distribution: Distribution { min_us: f64::NAN, p05_us: f64::NAN, median_us: f64::NAN, p95_us: f64::NAN, max_us: f64::NAN, samples_us: vec![] },
                output_sha256: String::new(),
                notes: Value::Null,
                error: Some(format!("{e:#}")),
            }
        }
    }
}
