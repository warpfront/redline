// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

use crate::types::{BackendResult, RowSpec};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct RowReport {
    pub spec: RowSpec,
    pub backends: BTreeMap<String, BackendResult>,
    pub bit_identical_across_backends: bool,
}

pub fn build_artifact(
    rows: &[RowReport],
    config: Value,
    environment: Value,
) -> Value {
    let generated_unix_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut json_rows = Vec::new();
    for row in rows {
        let mut backends_json = serde_json::Map::new();
        for (name, result) in &row.backends {
            let v = serde_json::to_value(result).unwrap_or(Value::Null);
            backends_json.insert(name.clone(), v);
        }
        let r = json!({
            "key": row.spec.key(),
            "mode": row.spec.mode.as_str(),
            "family": row.spec.kernel.family.as_str(),
            "name": row.spec.name(),
            "kernel": row.spec.kernel.symbol,
            "bits": row.spec.kernel.bits,
            "variant": row.spec.kernel.variant.as_str(),
            "shape": row.spec.shape,
            "wave_size": row.spec.wave_size,
            "scheduler_profile": row.spec.scheduler_profile.as_str(),
            "iterations": row.spec.iterations,
            "backends": Value::Object(backends_json),
            "bit_identical_across_backends": row.bit_identical_across_backends,
        });
        json_rows.push(r);
    }

    let summary = build_summary(rows);

    json!({
        "schema_version": 2,
        "kind": "hipfire-mqv2",
        "generated_unix_seconds": generated_unix_seconds,
        "config": config,
        "environment": environment,
        "rows": json_rows,
        "summary": summary,
    })
}

fn build_summary(rows: &[RowReport]) -> Value {
    let total = rows.len();
    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut bit_identical = 0usize;
    for row in rows {
        // Row passes if every backend that has a result is gate-passed
        let all_pass = row.backends.values().all(|r| r.correctness.pass);
        if all_pass {
            passed += 1;
        } else {
            failed += 1;
        }
        if row.bit_identical_across_backends {
            bit_identical += 1;
        }
    }
    json!({
        "total_rows": total,
        "passed_rows": passed,
        "failed_rows": failed,
        "bit_identical_rows": bit_identical,
    })
}
