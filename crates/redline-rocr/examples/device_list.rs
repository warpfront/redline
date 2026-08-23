// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! List every GPU this host exposes, with the anchors needed to pin one.
//!
//! Read-only: it enumerates HSA agents and reads sysfs. It creates no queue and
//! dispatches nothing, so it is safe to run on a machine with live work on it.
//!
//! Run it once per host to write that host's section of `devices.toml`: the
//! `pin:` column is the durable selector to copy.
//!
//! # Resolve mode
//!
//! `device_list --resolve <selector>` pins one device through the anchored
//! selector grammar and prints a single machine-readable line for shell
//! consumption:
//!
//! ```text
//! <unfiltered_rocr_index>\t<bdf>\t<describe()>
//! ```
//!
//! Exit codes: 0 resolved, 3 denied/fragile-refused, 4 not-found/ambiguous/parse-error, 1 other.
//! Errors go to stderr as `redline: …`.

use redline_rocr::identity::DeviceQuery;
use redline_rocr::manifest::{HostManifest, RiskClass};
use redline_rocr::selector;
use redline_rocr::{Runtime, RuntimeError, load_symbols};
use std::env;
use std::process;

fn main() {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("--resolve") => {
            // Flexible parsing for --risk so both
            //   device_list --resolve SEL --risk reset
            // and
            //   device_list --resolve --risk reset SEL
            // work. Sh scripts that add --risk after the selector keep working.
            let remaining: Vec<String> = args.collect();
            let mut selector: Option<String> = None;
            let mut risk = RiskClass::Normal;
            let mut i = 0;
            while i < remaining.len() {
                let arg = &remaining[i];
                if arg == "--risk" {
                    let val = remaining.get(i + 1).cloned().unwrap_or_default();
                    if val.is_empty() {
                        eprintln!("redline: --risk requires an argument (normal|reset)");
                        process::exit(1);
                    }
                    risk = parse_risk(&val);
                    i += 2;
                } else if let Some(val) = arg.strip_prefix("--risk=") {
                    if val.is_empty() {
                        eprintln!("redline: --risk requires an argument (normal|reset)");
                        process::exit(1);
                    }
                    risk = parse_risk(val);
                    i += 1;
                } else if arg == "-h" || arg == "--help" {
                    print_help();
                    process::exit(0);
                } else if selector.is_none() && !arg.starts_with('-') {
                    selector = Some(arg.clone());
                    i += 1;
                } else {
                    eprintln!("redline: device_list --resolve: unexpected argument {arg:?}; try --help");
                    process::exit(1);
                }
            }
            let Some(sel) = selector else {
                eprintln!("redline: device_list --resolve requires a selector");
                process::exit(1);
            };
            resolve_mode(&sel, risk);
        }
        Some("-h" | "--help") => {
            print_help();
            process::exit(0);
        }
        Some(other) => {
            eprintln!("redline: unknown argument {other:?}; try --help");
            process::exit(1);
        }
        None => {
            if let Err(err) = list_mode() {
                eprintln!("redline: {err}");
                process::exit(1);
            }
        }
    }
}

fn parse_risk(s: &str) -> RiskClass {
    match s.to_ascii_lowercase().as_str() {
        "normal" => RiskClass::Normal,
        "reset" | "reset-provoking" | "reset_provoking" => RiskClass::ResetProvoking,
        other => {
            eprintln!("redline: unknown --risk value {other:?}; expected normal or reset");
            process::exit(1);
        }
    }
}

fn print_help() {
    eprintln!(
        "\
Usage:
  device_list                              List GPUs with durable pin anchors
  device_list --resolve SEL [--risk RISK]  Resolve SEL; print index\\tbdf\\tdescribe
  device_list -h|--help                    This help

Selector grammar: uuid:… | bdf:… | slot:… | name:… | index:… | @alias
Risk:  normal (default) — deny-list only; reset — deny + fragile
       --risk reset  refuses fragile devices (reset-provoking harnesses)
Exit codes for --resolve: 0 ok, 3 denied or fragile-refused (reset risk), 4 not-found/ambiguous/parse, 1 other

Manifest tiers (devices.toml [host.<hostname>]):
  deny    — never selectable, for any operation
  fragile — selectable for normal work; refused only with --risk reset
Both are evaluated on the resolved device so no selector form can bypass.
Trail a # comment on each entry with the human reason — it is quoted on refusal."
    );
}

fn list_mode() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = Runtime::initialize(load_symbols()?)?;
    let identities = runtime.device_identities()?;
    let manifest = HostManifest::load()?;

    println!(
        "host {}: {} GPU agent(s)",
        manifest.host(),
        identities.len()
    );
    for identity in &identities {
        let status = match manifest.check_denied(identity) {
            Ok(()) => {
                // Not denied — check fragile under reset risk for informational annotation.
                match manifest.check_with_risk(identity, RiskClass::ResetProvoking) {
                    Ok(()) => String::new(),
                    Err(error) => format!("\n      FRAGILE: {error}"),
                }
            }
            Err(error) => format!("\n      DENIED: {error}"),
        };
        // The anchor is what belongs in a manifest; everything else in
        // `describe` is either descriptive or explicitly volatile.
        println!(
            "  [{}] {}\n      pin: {}{}",
            identity.rocr_index,
            identity.describe(),
            identity.anchor(),
            status
        );
    }
    Ok(())
}

fn resolve_mode(selector: &str, risk: RiskClass) -> ! {
    // CRITICAL: the index we print is later used to SET ROCR_VISIBLE_DEVICES
    // for child processes. Resolving while that filter is already set would
    // yield an index relative to the filtered set and silently select a
    // different card. Clear it before HSA init so rocr_index is unfiltered.
    // SAFETY: single-threaded main; no other threads have observed the env.
    unsafe {
        env::remove_var("ROCR_VISIBLE_DEVICES");
    }

    let runtime = match Runtime::initialize(load_symbols().unwrap_or_else(|e| {
        eprintln!("redline: {e}");
        process::exit(1);
    })) {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("redline: {e}");
            process::exit(1);
        }
    };

    let identities = match runtime.device_identities() {
        Ok(ids) => ids,
        Err(e) => {
            eprintln!("redline: {e}");
            process::exit(1);
        }
    };

    let manifest = match HostManifest::load() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("redline: {e}");
            exit_for_error(&e);
        }
    };

    let query = match selector::parse(selector) {
        Ok(q) => q,
        Err(e) => {
            eprintln!("redline: {e}");
            exit_for_error(&e);
        }
    };

    let expanded;
    let query_ref: &DeviceQuery = match &query {
        DeviceQuery::Alias(alias) => match manifest.resolve_alias(alias) {
            Ok(q) => {
                expanded = q;
                &expanded
            }
            Err(e) => {
                eprintln!("redline: {e}");
                exit_for_error(&e);
            }
        },
        other => other,
    };

    let chosen = match selector::resolve(query_ref, &identities) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("redline: {e}");
            exit_for_error(&e);
        }
    };

    if let Err(e) = manifest.check_with_risk(chosen, risk) {
        eprintln!("redline: {e}");
        exit_for_error(&e);
    }

    // Machine-readable: unfiltered ROCr index, BDF, human describe — tabs.
    println!(
        "{}\t{}\t{}",
        chosen.rocr_index,
        chosen.bdf,
        chosen.describe()
    );
    process::exit(0);
}

fn exit_for_error(err: &RuntimeError) -> ! {
    let code = match err {
        RuntimeError::DeviceDenied { .. } | RuntimeError::DeviceFragile { .. } => 3,
        RuntimeError::InvalidDeviceSelector { .. }
        | RuntimeError::DeviceNotFound { .. }
        | RuntimeError::DeviceAmbiguous { .. }
        | RuntimeError::AliasNotFound { .. }
        | RuntimeError::AliasNotResolved { .. }
        | RuntimeError::NoDevicesForSelector
        | RuntimeError::NoPciSlotLabels { .. }
        | RuntimeError::GpuNameAmbiguous { .. } => 4,
        _ => 1,
    };
    process::exit(code);
}
