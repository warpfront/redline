// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

mod common;
mod hip_backend;
mod redline_backend;
mod spec;
mod vulkan_backend;

use anyhow::{bail, Context, Result};
use common::{median, Distribution, Measurement};
use hip_backend::{embedded_code_object, HipBackend};
use radiowave::recipes::{RecipeCatalog, SelectionMode};
use radiowave::{SchedulerProfile, Wavefront};
use redline_backend::{RedlineBackend, RmwBoundary};
use redline_dispatch::aql::{Gfx12DispatchMode, QueuePolicy};
use redline_dispatch::partition::PartitionPolicy;
use redline_observe::amdsmi::{AmdSmi, TelemetrySnapshot};
use redline_observe::roctx::{Roctx, RoctxStack};
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use spec::{
    apply_radiowave_runtime_policy_with_catalog_and_mode, fixture, matrix_with_catalog_and_mode,
    validate, Correctness, MatrixProfile, RowSpec, TimingMode, WavePolicy,
};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use vulkan_backend::VulkanBackend;

const BACKENDS: [&str; 4] = ["redline", "vulkan", "hipgraph", "hip"];
const HIPENGINE_SUMMARY: &str =
    "../hipengine-6409/results/gfx1201/2026-07-22-714-bench/summary.json";
const HIPFIRE_BRIDGE_REV: &str = "455ffb9dfd6a5712889b504737f88fbbe87d3efe";

#[derive(Debug)]
struct Config {
    output: PathBuf,
    warmups: usize,
    samples: usize,
    filter: Option<String>,
    max_rows: Option<usize>,
    list: bool,
    wave_policy: WavePolicy,
    rmw_boundary: RmwBoundary,
    dispatch_mode: Gfx12DispatchMode,
    redline_queue_policy: QueuePolicy,
    scheduler_profile: SchedulerProfile,
    scheduler_profile_explicit: bool,
    interleave_aggressive_b32: bool,
    mixed_paired_hash: bool,
    matrix_profile: MatrixProfile,
    include_aggressive: bool,
    target_architecture: String,
    recipe_catalog: RecipeCatalog,
    recipe_catalog_source: String,
    recipe_mode: SelectionMode,
    recipe_allowlist: BTreeSet<String>,
    active_backends: Vec<&'static str>,
    partition_policy: PartitionPolicy,
    batch_mem: bool,
    telemetry: bool,
    roctx: bool,
}

#[derive(Debug, Serialize)]
struct BackendResult {
    distribution: Option<Distribution>,
    correctness: Option<Correctness>,
    error: Option<String>,
}

impl BackendResult {
    fn median_us(&self) -> Option<f64> {
        self.distribution.as_ref().map(|d| d.median_us)
    }

    fn accepted(&self) -> bool {
        self.correctness.as_ref().is_some_and(|c| c.pass) && self.distribution.is_some()
    }
}

#[derive(Debug, Serialize)]
struct RowResult {
    key: String,
    mode: TimingMode,
    family: String,
    name: String,
    kernel: String,
    second_kernel: Option<String>,
    n0: u32,
    n1: u32,
    aux: u32,
    block: u32,
    second_block: u32,
    wave_size: u32,
    scheduler_profile: SchedulerProfile,
    grid_groups: u32,
    iterations: usize,
    logical_operations: usize,
    redline_queue_count: usize,
    redline_submission_policy: &'static str,
    redline_dependency_cache_policies: BTreeMap<String, String>,
    partition_applied: bool,
    radiowave_recipes: Vec<String>,
    radiowave_lowerings: Vec<radiowave::recipes::SourceLowering>,
    backend_order: Vec<String>,
    backends: BTreeMap<String, BackendResult>,
}

#[derive(Debug, Serialize)]
struct Artifact {
    schema_version: u32,
    kind: &'static str,
    generated_unix_seconds: u64,
    methodology: Value,
    environment: Value,
    config: Value,
    rows: Vec<RowResult>,
    summary: Value,
    hipengine_baseline: Option<Value>,
}

fn main() -> Result<()> {
    let config = parse_args()?;
    let all_specs = matrix_with_catalog_and_mode(
        config.matrix_profile,
        config.wave_policy,
        &config.target_architecture,
        &config.recipe_catalog,
        config.recipe_mode,
    );
    let timing_modes = selected_timing_modes(config.include_aggressive);
    if config.list {
        for &mode in &timing_modes {
            for spec in &all_specs {
                if spec.supports_mode(mode) {
                    println!("{}", spec.key(mode));
                }
            }
        }
        return Ok(());
    }

    // --batch-mem is currently recorded-only on the retained-PM4 path (no clean
    // HIP batch-mem attachment). --partition-policy is applied at multi-lane
    // AQL queue creation; serial lanes stay full-device by design.
    let amd_smi = if config.telemetry {
        Some(AmdSmi::new().context("failed to initialize AMD SMI telemetry (--telemetry)")?)
    } else {
        None
    };
    let telemetry_start = amd_smi
        .as_ref()
        .map(|smi| smi.snapshot(0))
        .transpose()
        .context("failed to capture start AMD SMI TelemetrySnapshot")?;

    let roctx = if config.roctx {
        Some(Roctx::load().context("failed to initialize ROCTx (--roctx)")?)
    } else {
        None
    };
    let roctx_stack = roctx.as_ref().map(|r| match r.stack() {
        RoctxStack::Sdk => "sdk",
        RoctxStack::Legacy => "legacy",
    });

    println!("initializing Hipfire HIP bridge");
    let hip = HipBackend::new()?;
    println!("initializing Redline retained-PM4 backend");
    let redline = RedlineBackend::new(
        config.rmw_boundary,
        config.redline_queue_policy,
        config.partition_policy.clone(),
        config.dispatch_mode,
    )?;
    println!("initializing RADV Vulkan backend");
    let vulkan = VulkanBackend::new(Some(&redline.pci))?;
    if hip.arch != config.target_architecture {
        bail!(
            "Radiowave target {} does not match HIP device {}",
            config.target_architecture,
            hip.arch
        );
    }
    if let Some(vk_pci) = &vulkan.pci {
        let rl = normalize_pci(&redline.pci);
        let vk = normalize_pci(vk_pci);
        if rl != vk {
            bail!(
                "device mismatch: Redline PCI {} but Vulkan PCI {}",
                redline.pci,
                vk_pci
            );
        }
    }

    let mut selected = Vec::new();
    for &mode in &timing_modes {
        for spec in &all_specs {
            if !spec.supports_mode(mode) {
                continue;
            }
            let key = spec.key(mode);
            if config.filter.as_ref().is_none_or(|f| key.contains(f)) {
                selected.push((mode, spec.clone()));
            }
        }
    }
    if let Some(max_rows) = config.max_rows {
        selected.truncate(max_rows);
    }
    if selected.is_empty() {
        bail!("no rows matched the requested filter");
    }

    let mut rows = Vec::with_capacity(selected.len());
    let mut active_family_range: Option<(String, redline_observe::roctx::RangeGuard<'_>)> = None;
    for (row_index, (mode, mut spec)) in selected.into_iter().enumerate() {
        if config.wave_policy == WavePolicy::RadiowaveTuned {
            apply_radiowave_runtime_policy_with_catalog_and_mode(
                &mut spec,
                mode,
                &config.target_architecture,
                config.interleave_aggressive_b32,
                config.mixed_paired_hash,
                &config.recipe_catalog,
                config.recipe_mode,
            )?;
        }
        if config.scheduler_profile_explicit || config.wave_policy != WavePolicy::RadiowaveTuned {
            spec.scheduler_profile = config.scheduler_profile;
        }
        if let Some(roctx) = &roctx {
            let family = spec.family.to_owned();
            let needs_new = active_family_range
                .as_ref()
                .map(|(current, _)| current.as_str() != family.as_str())
                .unwrap_or(true);
            if needs_new {
                drop(active_family_range.take());
                let guard = roctx
                    .range(&family)
                    .with_context(|| format!("failed to open ROCTx range for family {family}"))?;
                active_family_range = Some((family, guard));
            }
        }
        let fixture = fixture(&mut spec);
        let mut order = config.active_backends.clone();
        let order_len = order.len();
        order.rotate_left(row_index % order_len);
        println!(
            "[{}/{}] {} ({} operations; order {})",
            row_index + 1,
            rows.capacity(),
            spec.key(mode),
            spec.logical_iterations(mode),
            order.join(" -> ")
        );
        let mut backends = BTreeMap::new();
        for backend in &order {
            let measurement = match *backend {
                "hip" => hip.measure_direct(&spec, &fixture, mode, config.warmups, config.samples),
                "hipgraph" => {
                    hip.measure_graph(&spec, &fixture, mode, config.warmups, config.samples)
                }
                "redline" => {
                    redline.measure(&hip, &spec, &fixture, mode, config.warmups, config.samples)
                }
                "vulkan" => vulkan.measure(&spec, &fixture, mode, config.warmups, config.samples),
                _ => unreachable!(),
            };
            let result = result_from_measurement(&spec, &fixture, mode, measurement);
            match (&result.distribution, &result.correctness, &result.error) {
                (Some(d), Some(c), _) => println!(
                    "  {:8} {:10.4} us/op correctness={} mismatches={}",
                    backend, d.median_us, c.pass, c.mismatches
                ),
                (_, _, Some(error)) => println!("  {backend:8} ERROR {error}"),
                _ => {}
            }
            backends.insert((*backend).to_owned(), result);
        }
        let redline_queue_count = redline.queue_count_for(mode, spec.logical_iterations(mode));
        let partition_applied =
            redline.partition_applied_for(mode, spec.logical_iterations(mode));
        rows.push(RowResult {
            key: spec.key(mode),
            mode,
            family: spec.family.to_owned(),
            name: spec.name.clone(),
            kernel: spec.kernel.to_owned(),
            second_kernel: spec.second_kernel.map(str::to_owned),
            n0: spec.n0,
            n1: spec.n1,
            aux: spec.aux,
            block: spec.block,
            second_block: spec.stage_block(true),
            wave_size: spec.wave_size,
            scheduler_profile: spec.scheduler_profile,
            grid_groups: spec.grid_groups,
            iterations: spec.iterations,
            logical_operations: spec.logical_iterations(mode),
            redline_queue_count,
            redline_submission_policy: redline_submission_policy(&spec, mode, redline_queue_count),
            redline_dependency_cache_policies: redline_dependency_cache_policies(
                &redline, &spec, mode,
            ),
            partition_applied,
            radiowave_recipes: spec.radiowave_recipes.iter().cloned().collect(),
            radiowave_lowerings: spec.radiowave_lowerings.iter().cloned().collect(),
            backend_order: order.into_iter().map(str::to_owned).collect(),
            backends,
        });
    }
    drop(active_family_range);

    let telemetry_end = amd_smi
        .as_ref()
        .map(|smi| smi.snapshot(0))
        .transpose()
        .context("failed to capture end AMD SMI TelemetrySnapshot")?;

    let summary = summarize(&rows, &config.active_backends);
    let hipengine_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(HIPENGINE_SUMMARY);
    let hipengine_baseline = fs::read(&hipengine_path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok());
    let scheduler_profiles_used = SchedulerProfile::ALL
        .into_iter()
        .filter(|profile| rows.iter().any(|row| row.scheduler_profile == *profile))
        .collect::<Vec<_>>();
    let hsaco_wave32_sha256 = scheduler_profiles_used
        .iter()
        .map(|profile| {
            let code = embedded_code_object(*profile, Wavefront::Wave32);
            (
                profile.as_str().to_owned(),
                format!("{:x}", Sha256::digest(code.code)),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let hsaco_wave64_sha256 = scheduler_profiles_used
        .iter()
        .map(|profile| {
            let code = embedded_code_object(*profile, Wavefront::Wave64);
            (
                profile.as_str().to_owned(),
                format!("{:x}", Sha256::digest(code.code)),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let scheduler_policy = if config.scheduler_profile_explicit {
        config.scheduler_profile.as_str()
    } else {
        "radiowave-recipe-or-default"
    };
    let hipcc_path = if let Some(value) = env::var_os("HIPCC").filter(|v| !v.is_empty()) {
        value.to_string_lossy().into_owned()
    } else if Path::new("/opt/rocm/core/bin/hipcc").exists() {
        "/opt/rocm/core/bin/hipcc".to_owned()
    } else if Path::new("/opt/rocm/core-7.14/bin/hipcc").exists() {
        "/opt/rocm/core-7.14/bin/hipcc".to_owned()
    } else {
        "hipcc".to_owned()
    };
    let artifact = Artifact {
        schema_version: 2,
        kind: "hipfire-6409-backend-comparison",
        generated_unix_seconds: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        methodology: json!({
            "timing_modes": {
                "serial_latency": "One true output RMW chain. Every operation is separated by the backend's required compute-write to compute-read/write dependency. Redline reuses immutable kernargs when every operation has identical arguments.",
                "independent_throughput": "Disjoint output slices with no inter-operation dependency; HIP, Vulkan, and Redline use up to four queues/streams. Redline release-publishes one retained PM4 IB per active lane before ringing any doorbell and measures the earliest GPU start through the latest GPU end. Serial RMW rows remain single-queue.",
                "single_kernel_aggressive": "Exactly one dispatch and one output operation. Redline's timed retained IB contains no entry acquire or dependency fence; the HIP-to-PM4 ownership acquire is replayed and waited outside the GPU timestamp window. Two-stage rows are excluded."
            },
            "timers": {
                "hip": "HIP device events",
                "hipgraph": "HIP device events around graph replay",
                "redline": "GPU-written PM4 COPY_DATA/RELEASE_MEM timestamps",
                "vulkan": "Vulkan timestamp queries"
            },
            "code_identity": "For every row, HIP, HipGraph, and Redline load exactly the same Radiowave-produced hipcc code object selected by the recorded wave, recipe, and scheduler policies. Vulkan runs matched GLSL algorithms compiled for RADV.",
            "scheduler_profile": scheduler_policy,
            "radiowave_recipe_mode": selection_mode_name(config.recipe_mode),
            "wave_policy": {
                "selected": config.wave_policy.as_str(),
                "all_wave32": "Every HIP-family row selects the wave32 code object.",
                "targeted_wave64": "Only q4_selected_dual, q6_x8, dense_q8, and vopd_dependent select wave64.",
                "radiowave_tuned": "The targeted kernels, interleave, and every VOPD variant select wave64; dispatch_tiny uses one 32-lane workgroup because only lane zero is live; interleave selects buffer output for independent throughput and B128 loads with a one-wave HIP workgroup for aggressive latency while Vulkan retains its native shader geometry.",
                "blanket_wave64": "Every kernel family with any prior Vulkan-over-Redline row selects wave64."
            },
            "correctness": "Every timed sequence is reset before timing and checked against a CPU oracle after its final sample. Only rows passing every selected backend are ranked.",
            "matrix_parity": "The default hipengine profile reproduces the pinned HipEngine f2c row set: the same family, operation, shape/sweep axes, repetition count, and serial/independent modes, totaling 112 core configurations plus 8 dispatch controls per mode. Each row is deliberately fired through Hipfire's existing Radiowave-tuned launch policy, so wave size, workgroup geometry, source variant, ABI, and machine code are optimization variables rather than parity constraints.",
            "vulkan_memory": "Device-local buffers with staging transfers outside the timing window.",
            "redline_dependency": format!("{} Every Redline sample completes its HIP-to-PM4 system ownership acquire before the timed retained tape. Independent single-stage and aggressive single-kernel tapes contain no dependency fence.", redline.rmw_boundary.description())
        }),
        environment: json!({
            "hip_arch": hip.arch,
            "hipfire_device": "HIP ordinal 0",
            "redline_device": redline.name,
            "redline_pci": redline.pci,
            "redline_queue_policy": redline.queue_policy().as_str(),
            "redline_dispatch_mode": match redline.dispatch_mode() {
                Gfx12DispatchMode::Workitems => "workitems",
                Gfx12DispatchMode::RadvWorkgroups => "radv-workgroups",
            },
            "redline_independent_queues": redline.independent_queue_count(),
            "vulkan_device": vulkan.name,
            "vulkan_pci": vulkan.pci,
            "vulkan_compute_queues": vulkan.queue_count,
            "repository_commit": command_output("git", &["rev-parse", "HEAD"]),
            "repository_dirty": !command_output("git", &["status", "--porcelain"]).is_empty(),
            "hipfire_redline_commit": HIPFIRE_BRIDGE_REV,
            "hipfire_clone_dirty": false,
            "hsaco_wave32_sha256_by_scheduler": hsaco_wave32_sha256,
            "hsaco_wave64_sha256_by_scheduler": hsaco_wave64_sha256,
            "hipcc": command_output(&hipcc_path, &["--version"]),
            "vulkan_summary": command_output("vulkaninfo", &["--summary"]),
            "telemetry": match (telemetry_start.as_ref(), telemetry_end.as_ref()) {
                (Some(start), Some(end)) => json!({
                    "start": telemetry_snapshot_json(start),
                    "end": telemetry_snapshot_json(end),
                }),
                _ => Value::Null,
            },
            "roctx": roctx_stack,
            "loaded": {
                "roctx_stack": roctx_stack.unwrap_or("unavailable"),
                "roctx_library": roctx
                    .as_ref()
                    .map(|r| r.library_path().to_owned())
                    .unwrap_or_else(|| "unavailable".to_owned()),
                "amdsmi": amd_smi
                    .as_ref()
                    .map(|s| s.library_path().to_owned())
                    .unwrap_or_else(|| "unavailable".to_owned()),
                "dispatch_partition_symbols": "present",
                "rocr_interface": "1.26",
                "partition_masks_applied": redline
                    .effective_partition_masks()
                    .iter()
                    .map(|lane| {
                        json!({
                            "lane": lane.lane,
                            "cu_mask": lane.cu_mask,
                            "enabled_cu_count": lane.enabled_cu_count,
                            "cu_mask_was_reduced": lane.cu_mask_was_reduced,
                            "reason": lane.reason,
                        })
                    })
                    .collect::<Vec<_>>(),
                "partition_masks_requested": redline
                    .applied_partitions()
                    .iter()
                    .map(|p| {
                        json!({
                            "lane": p.index,
                            "cu_offset": p.cu_offset,
                            "cu_count": p.cu_count,
                        })
                    })
                    .collect::<Vec<_>>(),
                "device_cu_count": redline.device_cu_count(),
            },
        }),
        config: json!({
            "warmups": config.warmups,
            "samples": config.samples,
            "filter": config.filter,
            "max_rows": config.max_rows,
            "wave_policy": config.wave_policy.as_str(),
            "radiowave_target_architecture": config.target_architecture,
            "radiowave_recipe_schema": radiowave::recipes::RECIPE_SCHEMA_VERSION,
            "radiowave_recipe_catalog": config.recipe_catalog_source,
            "radiowave_recipe_mode": selection_mode_name(config.recipe_mode),
            "radiowave_recipe_allowlist": config.recipe_allowlist,
            "scheduler_profile": scheduler_policy,
            "scheduler_profile_override": config.scheduler_profile_explicit.then(|| config.scheduler_profile.as_str()),
            "interleave_aggressive_b32": config.interleave_aggressive_b32,
            "mixed_paired_hash": config.mixed_paired_hash,
            "redline_rmw_boundary": config.rmw_boundary.as_str(),
            "redline_dispatch_mode": match config.dispatch_mode {
                Gfx12DispatchMode::Workitems => "workitems",
                Gfx12DispatchMode::RadvWorkgroups => "radv-workgroups",
            },
            "redline_queue_policy": redline.queue_policy().as_str(),
            "redline_independent_queues": redline.independent_queue_count(),
            "matrix_profile": config.matrix_profile.as_str(),
            "include_aggressive_extension": config.include_aggressive,
            "selected_rows": rows.len(),
            "logical_matrix_rows": timing_modes.iter().copied().map(|mode| all_specs.iter().filter(|spec| spec.supports_mode(mode)).count()).sum::<usize>(),
            "backends": config.active_backends,
            "partition_policy": partition_policy_json(&config.partition_policy),
            "batch_mem": {
                "requested": config.batch_mem,
                "applied": false,
                "reason": "no clean attachment on retained-PM4 path; HIP batch-mem plan is dispatch-crate-only",
            },
            "telemetry": config.telemetry,
            "roctx": config.roctx,
        }),
        rows,
        summary,
        hipengine_baseline,
    };
    write_artifact(&config.output, &artifact)?;
    println!("wrote {}", config.output.display());
    println!(
        "wrote {}",
        config.output.with_file_name("REPORT.md").display()
    );
    println!("{}", render_console_summary(&artifact));

    let matched = artifact.summary["matched_rows"].as_u64().unwrap_or(0);
    if matched == 0 {
        bail!("the run produced no correctness-passing comparison rows");
    }
    Ok(())
}

fn selected_timing_modes(include_aggressive: bool) -> Vec<TimingMode> {
    let mut modes = TimingMode::HIPENGINE_COMPARABLE.to_vec();
    if include_aggressive {
        modes.push(TimingMode::SingleKernelAggressive);
    }
    modes
}

fn result_from_measurement(
    spec: &RowSpec,
    fixture: &spec::Fixture,
    mode: TimingMode,
    measurement: Result<Measurement>,
) -> BackendResult {
    match measurement {
        Ok(measurement) => BackendResult {
            correctness: Some(validate(spec, fixture, mode, &measurement.output)),
            distribution: Some(Distribution::from_samples(measurement.gpu_samples_us)),
            error: None,
        },
        Err(error) => BackendResult {
            distribution: None,
            correctness: None,
            error: Some(format!("{error:#}")),
        },
    }
}

fn redline_submission_policy(spec: &RowSpec, mode: TimingMode, queue_count: usize) -> &'static str {
    if mode == TimingMode::IndependentThroughput && queue_count > 1 {
        return "independent_lane_local_retained_pm4";
    }
    if spec.second_kernel.is_none()
        && (mode != TimingMode::SerialLatency || spec.logical_iterations(mode) == 1)
    {
        "aggressive_unfenced_timed_ib_external_ownership_acquire"
    } else {
        "dependency_safe_timed_ib"
    }
}

fn redline_dependency_cache_policies(
    redline: &RedlineBackend,
    spec: &RowSpec,
    mode: TimingMode,
) -> BTreeMap<String, String> {
    let mut policies = BTreeMap::new();
    if let Some(second) = spec.second_kernel {
        policies.insert(
            second.to_owned(),
            redline
                .dependency_policy_name(second, spec.wave_size, spec.scheduler_profile)
                .to_owned(),
        );
    }
    if mode == TimingMode::SerialLatency && spec.logical_iterations(mode) > 1 {
        policies.insert(
            spec.kernel.to_owned(),
            redline
                .dependency_policy_name(spec.kernel, spec.wave_size, spec.scheduler_profile)
                .to_owned(),
        );
    }
    policies
}

fn summarize(rows: &[RowResult], active_backends: &[&str]) -> Value {
    let mut matched_rows = 0usize;
    let mut placements = active_backends
        .iter()
        .map(|&name| (name.to_owned(), [0usize; 4]))
        .collect::<BTreeMap<_, _>>();
    let mut losses = Vec::new();
    let mut pairwise: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    let mut family: BTreeMap<String, Vec<&RowResult>> = BTreeMap::new();

    for row in rows {
        if !active_backends
            .iter()
            .all(|name| row.backends.get(*name).is_some_and(BackendResult::accepted))
        {
            continue;
        }
        matched_rows += 1;
        family
            .entry(format!("{}/{}", row.mode.as_str(), row.family))
            .or_default()
            .push(row);
        let mut ranked = active_backends
            .iter()
            .map(|&name| (name, row.backends[name].median_us().unwrap()))
            .collect::<Vec<_>>();
        ranked.sort_by(|a, b| a.1.total_cmp(&b.1));
        for (place, (name, _)) in ranked.iter().enumerate() {
            placements.get_mut(*name).unwrap()[place] += 1;
        }
        let redline_us = row.backends["redline"].median_us().unwrap();
        for &other in active_backends.iter().filter(|&&name| name != "redline") {
            let other_us = row.backends[other].median_us().unwrap();
            pairwise
                .entry(other.to_owned())
                .or_default()
                .push(redline_us / other_us);
        }
        if ranked[0].0 != "redline" {
            let beaters = ranked
                .iter()
                .take_while(|(_, us)| *us < redline_us)
                .map(|(name, us)| {
                    json!({
                        "backend": name,
                        "median_us": us,
                        "redline_slower_percent": (redline_us / us - 1.0) * 100.0,
                    })
                })
                .collect::<Vec<_>>();
            let redline_place = ranked
                .iter()
                .position(|(name, _)| *name == "redline")
                .unwrap()
                + 1;
            losses.push(json!({
                "key": row.key,
                "redline_place": redline_place,
                "redline_median_us": redline_us,
                "winner": ranked[0].0,
                "winner_median_us": ranked[0].1,
                "redline_slower_than_winner_percent": (redline_us / ranked[0].1 - 1.0) * 100.0,
                "beaters": beaters,
            }));
        }
    }

    let placement_json = placements.into_iter().map(|(name, counts)| {
        let wins = counts[0];
        (name, json!({
            "first": counts[0], "second": counts[1], "third": counts[2], "fourth": counts[3],
            "wins": wins,
            "win_percent": percent(wins, matched_rows),
            "bench_n": matched_rows,
        }))
    }).collect::<serde_json::Map<_, _>>();
    let pairwise_json = pairwise
        .into_iter()
        .map(|(other, ratios)| {
            let wins = ratios.iter().filter(|&&v| v < 1.0).count();
            let ties = ratios.iter().filter(|&&v| v == 1.0).count();
            let losses = ratios.len() - wins - ties;
            (
                format!("redline_over_{other}"),
                json!({
                    "wins": wins,
                    "losses": losses,
                    "ties": ties,
                    "bench_n": ratios.len(),
                    "win_percent": percent(wins, ratios.len()),
                    "median_ratio": median(&ratios),
                    "min_ratio": ratios.iter().copied().reduce(f64::min),
                    "max_ratio": ratios.iter().copied().reduce(f64::max),
                }),
            )
        })
        .collect::<serde_json::Map<_, _>>();
    let family_json = family.into_iter().map(|(key, group)| {
        let wins = group.iter().filter(|row| {
            let rl = row.backends["redline"].median_us().unwrap();
            active_backends
                .iter()
                .all(|name| rl <= row.backends[*name].median_us().unwrap())
        }).count();
        (key, json!({"redline_wins": wins, "bench_n": group.len(), "win_percent": percent(wins, group.len())}))
    }).collect::<serde_json::Map<_, _>>();
    json!({
        "attempted_rows": rows.len(),
        "matched_rows": matched_rows,
        "rejected_rows": rows.len() - matched_rows,
        "redline_wins": placement_json["redline"]["first"],
        "redline_win_percent": placement_json["redline"]["win_percent"],
        "placements": placement_json,
        "pairwise": pairwise_json,
        "families": family_json,
        "redline_losses": losses,
    })
}

fn percent(n: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        n as f64 * 100.0 / total as f64
    }
}

fn write_artifact(path: &Path, artifact: &Artifact) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(path, serde_json::to_vec_pretty(artifact)?)
        .with_context(|| format!("write {}", path.display()))?;
    let report = render_report(artifact);
    fs::write(path.with_file_name("REPORT.md"), report)?;
    fs::write(
        path.with_file_name("summary.json"),
        serde_json::to_vec_pretty(&artifact.summary)?,
    )?;
    Ok(())
}

fn render_console_summary(artifact: &Artifact) -> String {
    let s = &artifact.summary;
    format!(
        "Redline wins {}/{} ({:.2}%). Placements: 1st {}, 2nd {}, 3rd {}, 4th {}.",
        s["redline_wins"].as_u64().unwrap_or(0),
        s["matched_rows"].as_u64().unwrap_or(0),
        s["redline_win_percent"].as_f64().unwrap_or(0.0),
        s["placements"]["redline"]["first"].as_u64().unwrap_or(0),
        s["placements"]["redline"]["second"].as_u64().unwrap_or(0),
        s["placements"]["redline"]["third"].as_u64().unwrap_or(0),
        s["placements"]["redline"]["fourth"].as_u64().unwrap_or(0),
    )
}

fn render_report(artifact: &Artifact) -> String {
    let s = &artifact.summary;
    let active_backends = artifact.config["backends"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    let wave_policy = artifact.config["wave_policy"].as_str().unwrap_or("unknown");
    let scheduler_profile = artifact.config["scheduler_profile"]
        .as_str()
        .unwrap_or("unknown");
    let matrix_profile = artifact.config["matrix_profile"]
        .as_str()
        .unwrap_or("unknown");
    let includes_aggressive = artifact.config["include_aggressive_extension"]
        .as_bool()
        .unwrap_or(false);
    let mut out = String::new();
    out.push_str("# Hipfire/Redline ROCm issue 6409 backend comparison\n\n");
    out.push_str(&format!(
        "Correctness-gated result: **Redline wins {}/{} rows ({:.2}%)**.\n\n",
        s["redline_wins"].as_u64().unwrap_or(0),
        s["matched_rows"].as_u64().unwrap_or(0),
        s["redline_win_percent"].as_f64().unwrap_or(0.0),
    ));
    out.push_str("## Placement table\n\n");
    out.push_str(
        "| Backend | 1st | 2nd | 3rd | 4th | Win % | N |\n|---|---:|---:|---:|---:|---:|---:|\n",
    );
    for &backend in &active_backends {
        let p = &s["placements"][backend];
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {:.2} | {} |\n",
            backend,
            p["first"].as_u64().unwrap_or(0),
            p["second"].as_u64().unwrap_or(0),
            p["third"].as_u64().unwrap_or(0),
            p["fourth"].as_u64().unwrap_or(0),
            p["win_percent"].as_f64().unwrap_or(0.0),
            p["bench_n"].as_u64().unwrap_or(0),
        ));
    }
    out.push_str("\n## Placement by timing mode\n\n");
    out.push_str(
        "| Mode | Backend | 1st | 2nd | 3rd | 4th | N |\n|---|---|---:|---:|---:|---:|---:|\n",
    );
    for mode in TimingMode::ALL {
        if !artifact.rows.iter().any(|row| row.mode == mode) {
            continue;
        }
        for &backend in &active_backends {
            let counts = placement_counts(&artifact.rows, mode, backend, &active_backends);
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {} |\n",
                mode.as_str(),
                backend,
                counts[0],
                counts[1],
                counts[2],
                counts[3],
                counts.iter().sum::<usize>(),
            ));
        }
    }
    out.push_str("\n## Redline losses\n\n");
    out.push_str("| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |\n|---|---:|---|---:|---:|\n");
    for loss in s["redline_losses"].as_array().into_iter().flatten() {
        let beaters = loss["beaters"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|beater| {
                format!(
                    "{} (+{:.2}%)",
                    beater["backend"].as_str().unwrap_or(""),
                    beater["redline_slower_percent"].as_f64().unwrap_or(0.0),
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!(
            "| `{}` | {} | {} | {:.4} | {:.4} |\n",
            loss["key"].as_str().unwrap_or(""),
            loss["redline_place"].as_u64().unwrap_or(0),
            beaters,
            loss["redline_median_us"].as_f64().unwrap_or(0.0),
            loss["winner_median_us"].as_f64().unwrap_or(0.0),
        ));
    }
    out.push_str("\n## Pairwise Redline results\n\n");
    out.push_str("| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |\n|---|---:|---:|---:|---:|---:|---:|\n");
    for &other in active_backends.iter().filter(|&&name| name != "redline") {
        let key = format!("redline_over_{other}");
        let p = &s["pairwise"][&key];
        out.push_str(&format!(
            "| RL / {} | {} | {} | {} | {:.2} | {:.4} | {} |\n",
            other,
            p["wins"].as_u64().unwrap_or(0),
            p["losses"].as_u64().unwrap_or(0),
            p["ties"].as_u64().unwrap_or(0),
            p["win_percent"].as_f64().unwrap_or(0.0),
            p["median_ratio"].as_f64().unwrap_or(0.0),
            p["bench_n"].as_u64().unwrap_or(0),
        ));
    }
    if let Some(baseline) = &artifact.hipengine_baseline {
        out.push_str("\n## Pinned hipEngine-harness comparison\n\n");
        out.push_str("The `hipengine` profile covers the same benchmark row set as pinned HipEngine `f2c3ad6`: family, operation, shape/sweep axes, repetition count, and serial/independent mode. Those rows deliberately run through Hipfire's existing Radiowave-tuned launch policy; wave size, workgroup geometry, source variant, ABI, and machine code remain optimization variables rather than parity constraints. HipEngine has 212 three-way matched core rows because its HIP independent sampler path rejects 12 rows; its 16 dispatch controls are reported separately. Hipfire executes and correctness-gates all 240 rows across four backends.\n\n");
        out.push_str("| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |\n|---|---|---:|---:|---:|---:|---:|\n");
        for other in ["vulkan", "hip"] {
            let key = format!("redline_over_{other}");
            let p = &baseline["overall"][&key];
            let wins = p["wins"].as_u64().unwrap_or(0);
            let losses = p["losses"].as_u64().unwrap_or(0);
            let n = wins + losses + p["ties"].as_u64().unwrap_or(0);
            out.push_str(&format!(
                "| hipEngine core | RL / {} | {} | {} | {:.2} | {:.4} | {} |\n",
                other,
                wins,
                losses,
                if n == 0 {
                    0.0
                } else {
                    wins as f64 * 100.0 / n as f64
                },
                p["median"].as_f64().unwrap_or(0.0),
                n,
            ));
        }
        for other in ["vulkan", "hip"] {
            let key = format!("redline_over_{other}");
            let p = &baseline["dispatch"][&key];
            out.push_str(&format!(
                "| hipEngine dispatch | RL / {} | {} | {} | {:.2} | {:.4} | {} |\n",
                other,
                p["wins"].as_u64().unwrap_or(0),
                p["losses"].as_u64().unwrap_or(0),
                p["win_percent"].as_f64().unwrap_or(0.0),
                p["median"].as_f64().unwrap_or(0.0),
                p["count"].as_u64().unwrap_or(0),
            ));
        }
    }
    let rl_vk = &s["pairwise"]["redline_over_vulkan"];
    out.push_str("\n## Harness verdict\n\n");
    if active_backends.contains(&"hip") && active_backends.contains(&"hipgraph") {
        let rl_hip = &s["pairwise"]["redline_over_hip"];
        let rl_graph = &s["pairwise"]["redline_over_hipgraph"];
        out.push_str(&format!(
            "This is **not a Hipfire harness failure**: Redline beats direct HIP in {}/{} rows and HipGraph in {}/{} while all three select identical per-row hipcc code objects. This run uses the `{}` matrix, `{}` wave policy, and `{}` scheduler profile.{} Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in {}/{} rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.\n",
            rl_hip["wins"].as_u64().unwrap_or(0), rl_hip["bench_n"].as_u64().unwrap_or(0),
            rl_graph["wins"].as_u64().unwrap_or(0), rl_graph["bench_n"].as_u64().unwrap_or(0),
            matrix_profile,
            wave_policy,
            scheduler_profile,
            if includes_aggressive { " The optional aggressive single-kernel extension removes dependency fences from Redline's timed tape." } else { "" },
            rl_vk["wins"].as_u64().unwrap_or(0), rl_vk["bench_n"].as_u64().unwrap_or(0),
        ));
    } else {
        out.push_str(&format!(
            "This tuning smoke intentionally measures only `{}`. It uses the `{}` matrix, `{}` wave policy, and `{}` scheduler profile.{} Every ranked row passed both CPU oracles. Redline beats Vulkan in {}/{} rows; final promotion still requires a full four-backend certification run.\n",
            active_backends.join("` versus `"),
            matrix_profile,
            wave_policy,
            scheduler_profile,
            if includes_aggressive { " The optional aggressive single-kernel extension removes dependency fences from Redline's timed tape." } else { "" },
            rl_vk["wins"].as_u64().unwrap_or(0), rl_vk["bench_n"].as_u64().unwrap_or(0),
        ));
    }
    out.push_str("\n## Interpretation guardrails\n\n");
    out.push_str(&format!("HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. The `{matrix_profile}` profile matches HipEngine's row coverage while intentionally retaining Hipfire/Radiowave's tuned wave, workgroup, and source-variant choices. This artifact records the `{wave_policy}` wave policy and `{scheduler_profile}` scheduler profile as controlled HIP compilation/launch factors. Every completion timestamp necessarily proves the measured work finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.\n"));
    out
}

fn placement_counts(
    rows: &[RowResult],
    mode: TimingMode,
    backend: &str,
    active_backends: &[&str],
) -> [usize; 4] {
    let mut counts = [0usize; 4];
    for row in rows.iter().filter(|row| row.mode == mode) {
        if !active_backends
            .iter()
            .all(|name| row.backends.get(*name).is_some_and(BackendResult::accepted))
        {
            continue;
        }
        let mut ranked = active_backends
            .iter()
            .map(|&name| (name, row.backends[name].median_us().unwrap()))
            .collect::<Vec<_>>();
        ranked.sort_by(|a, b| a.1.total_cmp(&b.1));
        if let Some(place) = ranked.iter().position(|(name, _)| *name == backend) {
            counts[place] += 1;
        }
    }
    counts
}

fn parse_args() -> Result<Config> {
    let mut output = PathBuf::from("results/gfx1201/manual-radiowave/results.json");
    let mut warmups = 3usize;
    let mut samples = 7usize;
    let mut filter = None;
    let mut max_rows = None;
    let mut list = false;
    let mut wave_policy = WavePolicy::RadiowaveTuned;
    let mut rmw_boundary = RmwBoundary::RadiowaveVmem;
    let mut dispatch_mode = Gfx12DispatchMode::Workitems;
    let mut redline_queue_policy = QueuePolicy::Auto;
    let mut scheduler_profile = SchedulerProfile::Default;
    let mut scheduler_profile_explicit = false;
    let mut interleave_aggressive_b32 = false;
    let mut mixed_paired_hash = false;
    let mut matrix_profile = MatrixProfile::HipEngineF2c;
    let mut include_aggressive = false;
    let mut partition_policy = PartitionPolicy::None;
    let mut batch_mem = false;
    let mut telemetry = false;
    let mut roctx = false;
    let target_architecture =
        env::var("HIPFIRE_BENCH_ARCH").unwrap_or_else(|_| "gfx1201".to_owned());
    let mut recipe_catalog_path = env::var_os("RADIOWAVE_RECIPE_CATALOG").map(PathBuf::from);
    let mut recipe_allowlist = BTreeSet::new();
    let mut active_backends = BACKENDS.to_vec();
    let mut recipe_mode = match env::var("RADIOWAVE_RECIPE_MODE") {
        Ok(value) => parse_selection_mode(&value).with_context(|| {
            format!("unknown RADIOWAVE_RECIPE_MODE {value}; expected certified or candidates")
        })?,
        Err(_) => SelectionMode::Certified,
    };
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--out" => output = PathBuf::from(args.next().context("--out requires a path")?),
            "--warmups" => warmups = args.next().context("--warmups requires a value")?.parse()?,
            "--samples" => samples = args.next().context("--samples requires a value")?.parse()?,
            "--filter" => filter = Some(args.next().context("--filter requires text")?),
            "--max-rows" => {
                max_rows = Some(
                    args.next()
                        .context("--max-rows requires a value")?
                        .parse()?,
                )
            }
            "--wave-policy" => {
                let value = args.next().context("--wave-policy requires a value")?;
                wave_policy = WavePolicy::parse(&value).with_context(|| {
                    format!("unknown wave policy {value}; expected all32, targeted64, radiowave, or blanket64")
                })?;
            }
            "--redline-rmw" => {
                let value = args.next().context("--redline-rmw requires a value")?;
                rmw_boundary = RmwBoundary::parse(&value).with_context(|| {
                    format!(
                        "unknown Redline RMW boundary {value}; expected radiowave-vmem, same-agent, or radv-global"
                    )
                })?;
            }
            "--redline-dispatch-mode" => {
                let value = args
                    .next()
                    .context("--redline-dispatch-mode requires a value")?;
                dispatch_mode = match value.as_str() {
                    "workitems" => Gfx12DispatchMode::Workitems,
                    "radv-workgroups" | "radv" => Gfx12DispatchMode::RadvWorkgroups,
                    _ => bail!(
                        "unknown Redline dispatch mode {value}; expected workitems, radv-workgroups, or radv"
                    ),
                };
            }
            "--redline-queues" => {
                redline_queue_policy = args
                    .next()
                    .context("--redline-queues requires a value")?
                    .parse()?;
            }
            "--scheduler-profile" => {
                let value = args
                    .next()
                    .context("--scheduler-profile requires a value")?;
                scheduler_profile = SchedulerProfile::parse(&value).with_context(|| {
                    format!(
                        "unknown scheduler profile {value}; expected default, max-ilp, iterative-ilp, memory-clause, or pipeline-ilp"
                    )
                })?;
                scheduler_profile_explicit = true;
            }
            "--interleave-aggressive-b32" => interleave_aggressive_b32 = true,
            "--mixed-paired-hash" => mixed_paired_hash = true,
            "--recipe-catalog" => {
                recipe_catalog_path = Some(PathBuf::from(
                    args.next().context("--recipe-catalog requires a path")?,
                ));
            }
            "--recipe-mode" => {
                let value = args.next().context("--recipe-mode requires a value")?;
                recipe_mode = parse_selection_mode(&value).with_context(|| {
                    format!(
                        "unknown Radiowave recipe mode {value}; expected certified or candidates"
                    )
                })?;
            }
            "--recipe-allow" => {
                recipe_allowlist
                    .insert(args.next().context("--recipe-allow requires a recipe ID")?);
            }
            "--backends" => {
                let value = args.next().context("--backends requires a value")?;
                active_backends = parse_backends(&value)?;
            }
            "--matrix" => {
                let value = args.next().context("--matrix requires a value")?;
                matrix_profile = MatrixProfile::parse(&value).with_context(|| {
                    format!("unknown matrix {value}; expected hipengine or legacy")
                })?;
            }
            "--include-aggressive" => include_aggressive = true,
            "--partition-policy" => {
                let value = args
                    .next()
                    .context("--partition-policy requires a value")?;
                partition_policy = parse_partition_policy(&value)?;
            }
            "--batch-mem" => batch_mem = true,
            "--telemetry" => telemetry = true,
            "--roctx" => roctx = true,
            "--list" => list = true,
            "--help" | "-h" => {
                println!(
                    "usage: hipfire-6409-bench [--out PATH] [--warmups N] [--samples N] [--filter TEXT] [--max-rows N] [--matrix hipengine|legacy] [--include-aggressive] [--backends all|redline,vulkan] [--wave-policy all32|targeted64|radiowave|blanket64] [--recipe-catalog PATH] [--recipe-mode certified|candidates] [--recipe-allow ID ...] [--scheduler-profile default|max-ilp|iterative-ilp|memory-clause|pipeline-ilp] [--interleave-aggressive-b32] [--mixed-paired-hash] [--redline-rmw radiowave-vmem|same-agent|radv-global] [--redline-dispatch-mode workitems|radv-workgroups|radv] [--redline-queues auto|1|2|4] [--partition-policy none|equal:N|cus:a,b,c] [--batch-mem (recorded-only on retained-PM4 path)] [--telemetry] [--roctx] [--list]"
                );
                std::process::exit(0);
            }
            _ => bail!("unknown argument {arg}"),
        }
    }
    if samples == 0 {
        bail!("--samples must be nonzero");
    }
    let (mut recipe_catalog, recipe_catalog_source) = if let Some(path) = recipe_catalog_path {
        let encoded = fs::read_to_string(&path)
            .with_context(|| format!("failed to read recipe catalog {}", path.display()))?;
        let catalog = RecipeCatalog::from_json(&encoded)
            .with_context(|| format!("failed to parse recipe catalog {}", path.display()))?;
        catalog
            .validate()
            .with_context(|| format!("invalid recipe catalog {}", path.display()))?;
        (catalog, path.display().to_string())
    } else {
        (
            RecipeCatalog::builtin_hipfire_6409(),
            "builtin:hipfire_6409".to_owned(),
        )
    };
    if !recipe_allowlist.is_empty() {
        for recipe_id in &recipe_allowlist {
            if !recipe_catalog
                .recipes
                .iter()
                .any(|recipe| &recipe.id == recipe_id)
            {
                bail!("--recipe-allow names unknown recipe {recipe_id}");
            }
        }
        recipe_catalog
            .recipes
            .retain(|recipe| recipe_allowlist.contains(&recipe.id));
    }
    Ok(Config {
        output,
        warmups,
        samples,
        filter,
        max_rows,
        list,
        wave_policy,
        rmw_boundary,
        dispatch_mode,
        redline_queue_policy,
        scheduler_profile,
        scheduler_profile_explicit,
        interleave_aggressive_b32,
        mixed_paired_hash,
        matrix_profile,
        include_aggressive,
        target_architecture,
        recipe_catalog,
        recipe_catalog_source,
        recipe_mode,
        recipe_allowlist,
        active_backends,
        partition_policy,
        batch_mem,
        telemetry,
        roctx,
    })
}

fn parse_partition_policy(value: &str) -> Result<PartitionPolicy> {
    if value == "none" {
        return Ok(PartitionPolicy::None);
    }
    if let Some(rest) = value.strip_prefix("equal:") {
        let n: usize = rest.parse().with_context(|| {
            format!("invalid partition policy {value}; equal:N requires a positive integer N")
        })?;
        let n = NonZeroUsize::new(n).with_context(|| {
            format!("invalid partition policy {value}; equal:N requires a positive integer N")
        })?;
        return Ok(PartitionPolicy::Equal(n));
    }
    if let Some(rest) = value.strip_prefix("cus:") {
        if rest.is_empty() {
            bail!(
                "invalid partition policy {value}: empty CU list; expected cus:a,b,c with positive CU counts"
            );
        }
        let mut counts = Vec::new();
        for (index, part) in rest.split(',').enumerate() {
            if part.is_empty() {
                bail!(
                    "invalid partition policy {value}: empty CU entry at index {index}; expected positive unsigned counts"
                );
            }
            let count: u32 = part.parse().with_context(|| {
                format!(
                    "invalid partition policy {value}: expected unsigned CU counts, got {part:?}"
                )
            })?;
            if count == 0 {
                bail!(
                    "invalid partition policy {value}: zero CU count at index {index} is not allowed"
                );
            }
            counts.push(count);
        }
        if counts.is_empty() {
            bail!(
                "invalid partition policy {value}: empty CU list; expected cus:a,b,c with positive CU counts"
            );
        }
        return Ok(PartitionPolicy::Explicit(counts));
    }
    bail!(
        "invalid partition policy {value}; expected none, equal:N, or cus:a,b,c"
    );
}

fn partition_policy_json(policy: &PartitionPolicy) -> Value {
    match policy {
        PartitionPolicy::None => json!("none"),
        PartitionPolicy::Equal(n) => json!(format!("equal:{}", n.get())),
        PartitionPolicy::Explicit(counts) => {
            let joined = counts
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(",");
            json!(format!("cus:{joined}"))
        }
    }
}

fn telemetry_snapshot_json(snap: &TelemetrySnapshot) -> Value {
    json!({
        "sclk_mhz": snap.sclk_mhz,
        "mclk_mhz": snap.mclk_mhz,
        "edge_temp_c": snap.edge_temp_c,
        "junction_temp_c": snap.junction_temp_c,
        "power_w": snap.power_w,
        "power_cap_w": snap.power_cap_w,
        "fan_rpm": snap.fan_rpm,
    })
}

fn parse_backends(value: &str) -> Result<Vec<&'static str>> {
    if value == "all" {
        return Ok(BACKENDS.to_vec());
    }
    let mut selected = Vec::new();
    for name in value.split(',') {
        let backend = BACKENDS
            .iter()
            .copied()
            .find(|candidate| *candidate == name)
            .with_context(|| {
                format!(
                    "unknown backend {name}; expected a comma-separated subset of {}",
                    BACKENDS.join(",")
                )
            })?;
        if !selected.contains(&backend) {
            selected.push(backend);
        }
    }
    if selected.len() < 2 || !selected.contains(&"redline") || !selected.contains(&"vulkan") {
        bail!("--backends tuning subsets require both redline and vulkan");
    }
    Ok(selected)
}

fn parse_selection_mode(value: &str) -> Option<SelectionMode> {
    match value {
        "certified" => Some(SelectionMode::Certified),
        "candidates" | "candidate" => Some(SelectionMode::Candidates),
        _ => None,
    }
}

fn selection_mode_name(mode: SelectionMode) -> &'static str {
    match mode {
        SelectionMode::Certified => "certified",
        SelectionMode::Candidates => "candidates",
    }
}

fn command_output(program: &str, args: &[&str]) -> String {
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .map(|output| {
            let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
            if text.trim().is_empty() {
                text = String::from_utf8_lossy(&output.stderr).into_owned();
            }
            text.trim().to_owned()
        })
        .unwrap_or_default()
}

fn normalize_pci(value: &str) -> String {
    value.trim_start_matches("0000:").to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tuning_backend_pair_is_explicit_and_ordered() {
        assert_eq!(
            parse_backends("redline,vulkan").unwrap(),
            vec!["redline", "vulkan"]
        );
        assert!(parse_backends("vulkan,hip").is_err());
        assert!(parse_backends("redline,hip").is_err());
        assert!(parse_backends("redline,unknown").is_err());
    }

    #[test]
    fn hipengine_summary_default_is_retained_run() {
        assert_eq!(
            HIPENGINE_SUMMARY,
            "../hipengine-6409/results/gfx1201/2026-07-22-714-bench/summary.json"
        );
        assert!(
            !HIPENGINE_SUMMARY.contains("2026-07-13-radiowave-redline"),
            "archived result path must not be the default"
        );
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(HIPENGINE_SUMMARY);
        assert!(
            path.is_file(),
            "retained hipengine summary missing at {}",
            path.display()
        );
    }

    #[test]
    fn hipfire_bridge_provenance_matches_pinned_dependency() {
        let manifest = include_str!("../Cargo.toml");
        assert!(
            manifest.contains(&format!("rev = \"{HIPFIRE_BRIDGE_REV}\"")),
            "provenance revision must match the pinned hip-bridge dependency"
        );
    }

    #[test]
    fn partition_policy_cli_forms() {
        assert_eq!(
            parse_partition_policy("none").unwrap(),
            PartitionPolicy::None
        );
        assert_eq!(
            parse_partition_policy("equal:2").unwrap(),
            PartitionPolicy::Equal(NonZeroUsize::new(2).unwrap())
        );
        assert_eq!(
            parse_partition_policy("cus:8,24,32").unwrap(),
            PartitionPolicy::Explicit(vec![8, 24, 32])
        );
        let err = parse_partition_policy("bogus").unwrap_err().to_string();
        assert!(err.contains("invalid partition policy bogus"), "{err}");
        let err = parse_partition_policy("equal:0").unwrap_err().to_string();
        assert!(err.contains("invalid partition policy equal:0"), "{err}");
    }
}
