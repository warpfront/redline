// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! Device selector grammar and pure resolution against [`DeviceIdentity`].
//!
//! # Why this is separate from `GpuSelector`
//!
//! The legacy [`crate::GpuSelector`] path only knows ordinals and name
//! substrings. Those are exactly the volatile handles that mis-pin devices on
//! multi-GPU hosts (rocm-smi order ≠ ROCr order; homogeneous boards share one
//! agent name). This module owns the **anchored** selector language —
//! `uuid:` / `bdf:` / `slot:` / `name:` / `index:` / `@alias` — so a pin copied
//! from a CLI flag, env var, or host manifest cannot silently become a wrong
//! ordinal.
//!
//! Parsing refuses unprefixed strings on purpose. Silently treating `"1"` as an
//! index or `"gfx1100"` as a name is the class of failure this feature exists
//! to prevent.

use std::fmt;
use std::str::FromStr;

use crate::identity::{DeviceIdentity, DeviceQuery};
use crate::runtime::{PciBusId, RuntimeError};

/// Parse a selector string into a [`DeviceQuery`].
///
/// Accepted forms (prefix is case-insensitive):
/// - `uuid:GPU-43390a851e296ee5`
/// - `bdf:0000:66:00.0` or `bdf:66:00.0` (bare bus assumes domain 0)
/// - `slot:3`
/// - `name:gfx1100`
/// - `index:1` — ROCr enumeration order; **volatile**
/// - `@alias` — resolved later by the host-manifest layer, not by [`resolve`]
///
/// An unprefixed string is always an error that lists the valid prefixes.
pub fn parse(input: &str) -> Result<DeviceQuery, RuntimeError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(RuntimeError::InvalidDeviceSelector {
            input: input.to_owned(),
            reason: "selector is empty; expected one of \
                     uuid:…, bdf:…, slot:…, name:…, index:…, or @alias"
                .to_owned(),
        });
    }

    if let Some(alias) = trimmed.strip_prefix('@') {
        if alias.is_empty() {
            return Err(RuntimeError::InvalidDeviceSelector {
                input: input.to_owned(),
                reason: "alias name after '@' is empty".to_owned(),
            });
        }
        return Ok(DeviceQuery::Alias(alias.to_owned()));
    }

    let Some((prefix, rest)) = trimmed.split_once(':') else {
        return Err(RuntimeError::InvalidDeviceSelector {
            input: input.to_owned(),
            reason: "unprefixed selector is refused; expected one of \
                     uuid:…, bdf:…, slot:…, name:…, index:…, or @alias"
                .to_owned(),
        });
    };

    if rest.is_empty() {
        return Err(RuntimeError::InvalidDeviceSelector {
            input: input.to_owned(),
            reason: format!("selector body after '{prefix}:' is empty"),
        });
    }

    match prefix.to_ascii_lowercase().as_str() {
        "uuid" => Ok(DeviceQuery::Uuid(rest.to_owned())),
        "bdf" => {
            let bdf =
                rest.parse::<PciBusId>()
                    .map_err(|err| RuntimeError::InvalidDeviceSelector {
                        input: input.to_owned(),
                        reason: err.to_string(),
                    })?;
            Ok(DeviceQuery::Bdf(bdf))
        }
        "slot" => Ok(DeviceQuery::Slot(rest.to_owned())),
        "name" => Ok(DeviceQuery::Name(rest.to_owned())),
        "index" => {
            let index = rest
                .parse::<usize>()
                .map_err(|_| RuntimeError::InvalidDeviceSelector {
                    input: input.to_owned(),
                    reason: format!("index body {rest:?} is not a non-negative integer"),
                })?;
            Ok(DeviceQuery::Index(index))
        }
        other => Err(RuntimeError::InvalidDeviceSelector {
            input: input.to_owned(),
            reason: format!(
                "unknown selector prefix {other:?}; expected one of \
                 uuid, bdf, slot, name, index, or @alias"
            ),
        }),
    }
}

impl FromStr for DeviceQuery {
    type Err = RuntimeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse(s)
    }
}

/// Resolve a parsed query against an already-joined device list.
///
/// Pure and side-effect free: alias lookup is intentionally not performed here
/// so unit tests and the manifest layer can compose cleanly. Returns a
/// reference into `devices` so callers can keep the owning list.
///
/// # Index volatility
///
/// [`DeviceQuery::Index`] matches [`DeviceIdentity::rocr_index`]. That field is
/// discovery-order under the current `ROCR_VISIBLE_DEVICES` filter and is
/// **not** stable across reboots, driver loads, or tool boundaries. Prefer
/// `uuid:` or `bdf:`.
pub fn resolve<'a>(
    query: &DeviceQuery,
    devices: &'a [DeviceIdentity],
) -> Result<&'a DeviceIdentity, RuntimeError> {
    if devices.is_empty() {
        return Err(RuntimeError::NoDevicesForSelector);
    }

    match query {
        DeviceQuery::Alias(alias) => Err(RuntimeError::AliasNotResolved {
            alias: alias.clone(),
        }),
        DeviceQuery::Uuid(want) => {
            let hits: Vec<_> = devices
                .iter()
                .filter(|d| {
                    d.uuid
                        .as_ref()
                        .is_some_and(|u| u.eq_ignore_ascii_case(want))
                })
                .collect();
            unique_hit(query, &hits)
        }
        DeviceQuery::Bdf(want) => {
            let hits: Vec<_> = devices.iter().filter(|d| d.bdf == *want).collect();
            unique_hit(query, &hits)
        }
        DeviceQuery::Slot(want) => {
            let any_slot = devices.iter().any(|d| d.pci_slot.is_some());
            if !any_slot {
                return Err(RuntimeError::NoPciSlotLabels { slot: want.clone() });
            }
            let hits: Vec<_> = devices
                .iter()
                .filter(|d| d.pci_slot.as_deref() == Some(want.as_str()))
                .collect();
            unique_hit(query, &hits)
        }
        DeviceQuery::Name(needle) => {
            let needle_l = needle.to_ascii_lowercase();
            let hits: Vec<_> = devices
                .iter()
                .filter(|d| d.agent_name.to_ascii_lowercase().contains(&needle_l))
                .collect();
            match hits.as_slice() {
                [] => Err(RuntimeError::DeviceNotFound {
                    query: format_query(query),
                }),
                [one] => Ok(*one),
                many => Err(RuntimeError::DeviceAmbiguous {
                    query: format_query(query),
                    matches: many.iter().map(|d| format_match(d)).collect(),
                }),
            }
        }
        DeviceQuery::Index(want) => {
            // Volatile: rocr_index is discovery order, not a durable pin.
            let hits: Vec<_> = devices.iter().filter(|d| d.rocr_index == *want).collect();
            unique_hit(query, &hits)
        }
    }
}

fn unique_hit<'a>(
    query: &DeviceQuery,
    hits: &[&'a DeviceIdentity],
) -> Result<&'a DeviceIdentity, RuntimeError> {
    match hits {
        [] => Err(RuntimeError::DeviceNotFound {
            query: format_query(query),
        }),
        [one] => Ok(*one),
        many => Err(RuntimeError::DeviceAmbiguous {
            query: format_query(query),
            matches: many.iter().map(|d| format_match(d)).collect(),
        }),
    }
}

/// Human-readable form of a query for error messages (round-trips the grammar).
fn format_query(query: &DeviceQuery) -> String {
    match query {
        DeviceQuery::Uuid(u) => format!("uuid:{u}"),
        DeviceQuery::Bdf(b) => format!("bdf:{b}"),
        DeviceQuery::Slot(s) => format!("slot:{s}"),
        DeviceQuery::Name(n) => format!("name:{n}"),
        DeviceQuery::Index(i) => format!("index:{i}"),
        DeviceQuery::Alias(a) => format!("@{a}"),
    }
}

/// One candidate line: anchor + PCI + rocr index. Enough to disambiguate on a
/// 4× identical-board host without forcing the operator to re-run tooling.
fn format_match(d: &DeviceIdentity) -> String {
    format!("{} {} rocr#{}(volatile)", d.anchor(), d.bdf, d.rocr_index)
}

/// Display helper kept public so callers can render a query the same way errors do.
pub fn query_label(query: &DeviceQuery) -> String {
    format_query(query)
}

impl fmt::Display for DeviceQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format_query(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{Anchor, DeviceIdentity};
    use crate::runtime::PciBusId;

    fn bdf(s: &str) -> PciBusId {
        s.parse().expect("test bdf")
    }

    /// Real hipx topology (4 heterogeneous GPUs). UUIDs and slots from measured host.
    fn hipx() -> Vec<DeviceIdentity> {
        vec![
            DeviceIdentity {
                uuid: Some("GPU-43390a851e296ee5".into()),
                bdf: bdf("0000:66:00.0"),
                agent_name: "gfx1100".into(),
                product_name: "Radeon RX 7900 XTX".into(),
                kfd_node: 1,
                rocr_index: 0,
                hip_ordinal: Some(0),
                pci_slot: None,
                drm_card: Some("card0".into()),
            },
            DeviceIdentity {
                uuid: None,
                bdf: bdf("0000:bf:00.0"),
                agent_name: "gfx1151".into(),
                product_name: "AMD Radeon Graphics".into(),
                kfd_node: 2,
                rocr_index: 1,
                hip_ordinal: Some(3),
                pci_slot: None,
                drm_card: Some("card1".into()),
            },
            DeviceIdentity {
                uuid: None,
                bdf: bdf("0000:6e:00.0"),
                agent_name: "gfx1010".into(),
                product_name: "Radeon RX 5700 XT".into(),
                kfd_node: 3,
                rocr_index: 2,
                hip_ordinal: Some(1),
                pci_slot: Some("1".into()),
                drm_card: Some("card2".into()),
            },
            DeviceIdentity {
                uuid: Some("GPU-c7ff6b154d0128bc".into()),
                bdf: bdf("0000:99:00.0"),
                agent_name: "gfx1030".into(),
                product_name: "Radeon RX 6800".into(),
                kfd_node: 4,
                rocr_index: 3,
                hip_ordinal: Some(2),
                pci_slot: Some("3".into()),
                drm_card: Some("card3".into()),
            },
        ]
    }

    /// Real hiptrx topology (4× gfx1201). No PCI slots resolvable.
    fn hiptrx() -> Vec<DeviceIdentity> {
        vec![
            DeviceIdentity {
                uuid: Some("GPU-9eb7aeda51c88ffd".into()),
                bdf: bdf("0000:03:00.0"),
                agent_name: "gfx1201".into(),
                product_name: "Radeon RX 9070 XT".into(),
                kfd_node: 1,
                rocr_index: 0,
                hip_ordinal: Some(0),
                pci_slot: None,
                drm_card: Some("card0".into()),
            },
            DeviceIdentity {
                uuid: Some("GPU-05f92432f2312a0e".into()),
                bdf: bdf("0000:c3:00.0"),
                agent_name: "gfx1201".into(),
                product_name: "Radeon RX 9070 XT".into(),
                kfd_node: 2,
                rocr_index: 1,
                hip_ordinal: Some(1),
                pci_slot: None,
                drm_card: Some("card1".into()),
            },
            DeviceIdentity {
                uuid: Some("GPU-085289909a86cc63".into()),
                bdf: bdf("0000:e3:00.0"),
                agent_name: "gfx1201".into(),
                product_name: "Radeon RX 9070 XT".into(),
                kfd_node: 3,
                rocr_index: 2,
                hip_ordinal: Some(2),
                pci_slot: None,
                drm_card: Some("card2".into()),
            },
            DeviceIdentity {
                uuid: Some("GPU-e475645fe0200397".into()),
                bdf: bdf("0000:13:00.0"),
                agent_name: "gfx1201".into(),
                product_name: "Radeon RX 9070 XT".into(),
                kfd_node: 4,
                rocr_index: 3,
                hip_ordinal: Some(3),
                pci_slot: None,
                drm_card: Some("card3".into()),
            },
        ]
    }

    #[test]
    fn parse_all_prefixed_forms() {
        assert_eq!(
            parse("uuid:GPU-43390a851e296ee5").unwrap(),
            DeviceQuery::Uuid("GPU-43390a851e296ee5".into())
        );
        assert_eq!(
            parse("BDF:0000:66:00.0").unwrap(),
            DeviceQuery::Bdf(bdf("0000:66:00.0"))
        );
        assert_eq!(
            parse("bdf:66:00.0").unwrap(),
            DeviceQuery::Bdf(bdf("0000:66:00.0"))
        );
        assert_eq!(parse("slot:3").unwrap(), DeviceQuery::Slot("3".into()));
        assert_eq!(
            parse("NAME:gfx1100").unwrap(),
            DeviceQuery::Name("gfx1100".into())
        );
        assert_eq!(parse("index:1").unwrap(), DeviceQuery::Index(1));
        assert_eq!(parse("@dev0").unwrap(), DeviceQuery::Alias("dev0".into()));
    }

    #[test]
    fn parse_rejects_unprefixed() {
        let err = parse("gfx1100").unwrap_err();
        let text = err.to_string();
        assert!(
            matches!(err, RuntimeError::InvalidDeviceSelector { .. }),
            "{err}"
        );
        assert!(text.contains("unprefixed"), "{text}");
        assert!(text.contains("uuid:"), "{text}");
        assert!(text.contains("@alias"), "{text}");
    }

    #[test]
    fn uuid_pin_selects_hipx_card() {
        let devices = hipx();
        let q = parse("uuid:GPU-43390a851e296ee5").unwrap();
        let d = resolve(&q, &devices).unwrap();
        assert_eq!(d.bdf, bdf("0000:66:00.0"));
        assert_eq!(d.rocr_index, 0);

        // Case-insensitive UUID compare.
        let q = parse("uuid:gpu-43390A851E296EE5").unwrap();
        let d = resolve(&q, &devices).unwrap();
        assert_eq!(d.agent_name, "gfx1100");
    }

    #[test]
    fn uuid_never_matches_uuid_less_hipx_gpus() {
        let devices = hipx();
        // The two GPU-XX devices must never match any uuid query.
        for bogus in [
            "uuid:GPU-XX",
            "uuid:GPU-0000000000000000",
            "uuid:GPU-c7ff6b154d0128bc-extra",
        ] {
            let q = parse(bogus).unwrap();
            let err = resolve(&q, &devices).unwrap_err();
            assert!(
                matches!(err, RuntimeError::DeviceNotFound { .. }),
                "query {bogus} => {err}"
            );
        }
        // Confirm the UUID-less devices themselves have no uuid.
        assert!(devices[1].uuid.is_none());
        assert!(devices[2].uuid.is_none());
        // Anchoring a uuid that equals nothing on those rows fails even if
        // someone stuffed the BDF into a uuid: string.
        let q = parse("uuid:0000:bf:00.0").unwrap();
        assert!(matches!(
            resolve(&q, &devices).unwrap_err(),
            RuntimeError::DeviceNotFound { .. }
        ));
    }

    #[test]
    fn name_gfx1201_on_hiptrx_is_ambiguous_and_lists_all_anchors() {
        let devices = hiptrx();
        let q = parse("name:gfx1201").unwrap();
        let err = resolve(&q, &devices).unwrap_err();
        let RuntimeError::DeviceAmbiguous { query, matches } = &err else {
            panic!("expected DeviceAmbiguous, got {err}");
        };
        assert_eq!(query, "name:gfx1201");
        assert_eq!(matches.len(), 4, "{matches:?}");
        let text = err.to_string();
        // Verbatim message captured for the acceptance paste.
        assert_eq!(
            text,
            "device selector \"name:gfx1201\" is ambiguous; matches \
             [uuid:GPU-9eb7aeda51c88ffd 0000:03:00.0 rocr#0(volatile), \
             uuid:GPU-05f92432f2312a0e 0000:c3:00.0 rocr#1(volatile), \
             uuid:GPU-085289909a86cc63 0000:e3:00.0 rocr#2(volatile), \
             uuid:GPU-e475645fe0200397 0000:13:00.0 rocr#3(volatile)]; \
             disambiguate with uuid:… or bdf:…"
        );
        for anchor in [
            "uuid:GPU-9eb7aeda51c88ffd",
            "uuid:GPU-05f92432f2312a0e",
            "uuid:GPU-085289909a86cc63",
            "uuid:GPU-e475645fe0200397",
        ] {
            assert!(text.contains(anchor), "missing {anchor} in {text}");
        }
    }

    #[test]
    fn slot_3_resolves_on_hipx_and_reports_no_labels_on_hiptrx() {
        let hipx = hipx();
        let q = parse("slot:3").unwrap();
        let d = resolve(&q, &hipx).unwrap();
        assert_eq!(d.bdf, bdf("0000:99:00.0"));
        assert_eq!(d.agent_name, "gfx1030");

        let hiptrx = hiptrx();
        let err = resolve(&q, &hiptrx).unwrap_err();
        let text = err.to_string();
        assert!(matches!(err, RuntimeError::NoPciSlotLabels { .. }), "{err}");
        assert!(
            text.contains("no slot labels") || text.contains("no PCI slot labels"),
            "{text}"
        );
    }

    #[test]
    fn index_1_resolves_volatile_rocr_order() {
        let devices = hipx();
        let q = parse("index:1").unwrap();
        let d = resolve(&q, &devices).unwrap();
        // hipx ROCr index 1 is the APU at bf:00.0 (danger is a host-manifest
        // deny-list concern; DeviceIdentity deliberately has no integrated flag).
        assert_eq!(d.rocr_index, 1);
        assert_eq!(d.bdf, bdf("0000:bf:00.0"));
        assert_eq!(d.agent_name, "gfx1151");
    }

    #[test]
    fn bdf_exact_match_including_short_form() {
        let devices = hipx();
        let q = parse("bdf:6e:00.0").unwrap();
        let d = resolve(&q, &devices).unwrap();
        assert_eq!(d.agent_name, "gfx1010");
        assert_eq!(d.pci_slot.as_deref(), Some("1"));
    }

    #[test]
    fn alias_is_not_resolved_here() {
        let devices = hipx();
        let q = parse("@primary").unwrap();
        let err = resolve(&q, &devices).unwrap_err();
        assert!(
            matches!(err, RuntimeError::AliasNotResolved { .. }),
            "{err}"
        );
    }

    #[test]
    fn empty_device_list_is_distinct_error() {
        let q = parse("index:0").unwrap();
        let err = resolve(&q, &[]).unwrap_err();
        assert!(matches!(err, RuntimeError::NoDevicesForSelector), "{err}");
    }

    #[test]
    fn zero_matches_is_not_found() {
        let devices = hipx();
        let q = parse("name:vega").unwrap();
        let err = resolve(&q, &devices).unwrap_err();
        assert!(matches!(err, RuntimeError::DeviceNotFound { .. }), "{err}");
    }

    #[test]
    fn format_match_includes_anchor() {
        let d = &hipx()[0];
        assert_eq!(d.anchor(), Anchor::Uuid("GPU-43390a851e296ee5".into()));
        let d = &hipx()[1];
        assert_eq!(d.anchor(), Anchor::Bdf(bdf("0000:bf:00.0")));
    }
}
