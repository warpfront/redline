// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! Best-effort sysfs enrichment for [`DeviceIdentity`].
//!
//! Slot labels and DRM card names are not available from HSA. Both are resolved
//! from a filesystem root so the walk can be unit-tested against fixture trees
//! that reproduce real host layouts without touching a GPU.
//!
//! Device danger (integrated APU whose reset takes the host down) is **not**
//! detected here: measured on a Strix Halo + discrete host, every plausible
//! sysfs/KFD signal is identical for the APU and the discrete boards. That
//! classification lives in the host manifest deny-list instead.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::identity::DeviceIdentity;
use crate::runtime::PciBusId;

/// Resolve the firmware/ACPI PCI slot label for `bdf`, if the host exposes one.
///
/// `/sys/bus/pci/slots/<label>/address` holds `domain:bus:dev` **without** the
/// function (e.g. `0000:6a:00`). The GPU itself is usually absent from that
/// table — it sits behind bridges — so this walks the canonical path of
/// `/sys/bus/pci/devices/<bdf>` and tests each PCI ancestor (nearest first)
/// against the slot table. Missing paths, unreadable entries, or no match all
/// yield `None`; nothing is invented.
pub(crate) fn resolve_pci_slot(sysfs_root: &Path, bdf: PciBusId) -> Option<String> {
    let slot_table = load_slot_table(sysfs_root)?;
    if slot_table.is_empty() {
        return None;
    }

    let device_link = sysfs_root.join("bus/pci/devices").join(bdf.to_string());
    let canonical = fs::canonicalize(&device_link).ok()?;

    // Nearest ancestor first: walk path components from the leaf toward root.
    for component in canonical
        .components()
        .rev()
        .filter_map(|c| c.as_os_str().to_str())
    {
        let Some(slot_key) = pci_slot_key(component) else {
            continue;
        };
        if let Some(label) = slot_table.get(slot_key) {
            return Some(label.clone());
        }
    }
    None
}

/// Map `bdf` to its primary DRM node name (`cardN`), if one is bound.
///
/// Compares the canonical target of each `/sys/class/drm/card*/device` symlink
/// to the canonical path of the PCI device. Connector entries (`card0-DP-1`
/// and friends) are ignored. Unreadable or missing trees yield `None`.
pub(crate) fn resolve_drm_card(sysfs_root: &Path, bdf: PciBusId) -> Option<String> {
    let device_link = sysfs_root.join("bus/pci/devices").join(bdf.to_string());
    let device_canon = fs::canonicalize(&device_link).ok()?;

    let drm_dir = sysfs_root.join("class/drm");
    let entries = fs::read_dir(&drm_dir).ok()?;

    let mut matches: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if !is_primary_drm_card(name) {
            continue;
        }
        let card_device = entry.path().join("device");
        let Ok(card_canon) = fs::canonicalize(&card_device) else {
            continue;
        };
        if card_canon == device_canon {
            matches.push(name.to_owned());
        }
    }

    // Stable pick if the tree is somehow duplicated; primary cards are unique
    // on real hosts.
    matches.sort();
    matches.into_iter().next()
}

/// Fill `pci_slot` and `drm_card` on `identity` from live `/sys`.
pub(crate) fn enrich_device_identity(identity: &mut DeviceIdentity) {
    enrich_device_identity_from(identity, Path::new("/sys"));
}

/// Same as [`enrich_device_identity`] but against an arbitrary sysfs root
/// (fixture trees in tests).
pub(crate) fn enrich_device_identity_from(identity: &mut DeviceIdentity, sysfs_root: &Path) {
    identity.pci_slot = resolve_pci_slot(sysfs_root, identity.bdf);
    identity.drm_card = resolve_drm_card(sysfs_root, identity.bdf);
}

/// Build `slot_address_without_function -> label` from `.../bus/pci/slots/*`.
fn load_slot_table(sysfs_root: &Path) -> Option<HashMap<String, String>> {
    let slots_dir = sysfs_root.join("bus/pci/slots");
    let entries = fs::read_dir(&slots_dir).ok()?;
    let mut table = HashMap::new();
    for entry in entries.flatten() {
        let label = entry.file_name();
        let Some(label) = label.to_str() else {
            continue;
        };
        let address_path = entry.path().join("address");
        let Ok(raw) = fs::read_to_string(&address_path) else {
            continue;
        };
        let address = raw.trim();
        if address.is_empty() {
            continue;
        }
        // Normalize to lowercase so path components and slot files compare.
        table.insert(address.to_ascii_lowercase(), label.to_owned());
    }
    Some(table)
}

/// `0000:6a:00.0` → `Some("0000:6a:00")`; non-BDF path components → `None`.
fn pci_slot_key(component: &str) -> Option<&str> {
    let (domain_bus_dev, function) = component.rsplit_once('.')?;
    if function.len() != 1 || !function.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let mut parts = domain_bus_dev.split(':');
    let domain = parts.next()?;
    let bus = parts.next()?;
    let dev = parts.next()?;
    if parts.next().is_some()
        || domain.len() != 4
        || bus.len() != 2
        || dev.len() != 2
        || !domain.chars().all(|c| c.is_ascii_hexdigit())
        || !bus.chars().all(|c| c.is_ascii_hexdigit())
        || !dev.chars().all(|c| c.is_ascii_hexdigit())
    {
        return None;
    }
    // Slot files are lowercase hex; path components already are on Linux.
    // Return the borrowed prefix only when it is already lowercase-compatible
    // for table lookup — callers lower the table keys; path comps are lower.
    Some(domain_bus_dev)
}

/// Primary DRM nodes are `card` + decimal digits only (`card0`, not `card0-DP-1`).
fn is_primary_drm_card(name: &str) -> bool {
    let Some(rest) = name.strip_prefix("card") else {
        return false;
    };
    !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::path::PathBuf;
    use std::str::FromStr;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static FIXTURE_SEQ: AtomicU64 = AtomicU64::new(0);

    struct Fixture {
        root: PathBuf,
    }

    impl Fixture {
        fn create(label: &str) -> io::Result<Self> {
            let seq = FIXTURE_SEQ.fetch_add(1, Ordering::Relaxed);
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0);
            let root = std::env::temp_dir()
                .join(format!("redline-rocr-sysfs-{}-{}-{}", label, nanos, seq));
            fs::create_dir_all(&root)?;
            Ok(Self { root })
        }

        fn path(&self) -> &Path {
            &self.root
        }

        /// Create a PCI device at `chain` (leaf last) and a
        /// `bus/pci/devices/<leaf>` symlink whose canonicalize walks that chain.
        fn install_pci_chain(&self, chain: &[&str]) -> io::Result<PathBuf> {
            assert!(!chain.is_empty());
            let leaf = *chain.last().unwrap();
            // Root complex folder name is derived from the first domain segment.
            let domain = chain[0].split(':').next().unwrap_or("0000");
            let mut device_dir = self.root.join("devices").join(format!("pci{domain}:00"));
            for comp in chain {
                device_dir.push(comp);
            }
            fs::create_dir_all(&device_dir)?;

            let link_dir = self.root.join("bus/pci/devices");
            fs::create_dir_all(&link_dir)?;
            let link_path = link_dir.join(leaf);

            // Relative symlink from bus/pci/devices/<bdf> → devices/.../<bdf>.
            let rel = pathdiff_from_to(&link_dir, &device_dir);
            std::os::unix::fs::symlink(&rel, &link_path)?;
            Ok(device_dir)
        }

        fn install_slot(&self, label: &str, address: &str) -> io::Result<()> {
            let dir = self.root.join("bus/pci/slots").join(label);
            fs::create_dir_all(&dir)?;
            fs::write(dir.join("address"), format!("{address}\n"))
        }

        fn install_drm_card(&self, card: &str, device_dir: &Path) -> io::Result<()> {
            let card_dir = self.root.join("class/drm").join(card);
            fs::create_dir_all(&card_dir)?;
            let link = card_dir.join("device");
            let rel = pathdiff_from_to(&card_dir, device_dir);
            std::os::unix::fs::symlink(&rel, &link)
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    /// Minimal relative path from `from_dir` to `to` (both absolute under root).
    fn pathdiff_from_to(from_dir: &Path, to: &Path) -> PathBuf {
        let from_comps: Vec<_> = from_dir.components().collect();
        let to_comps: Vec<_> = to.components().collect();
        let mut common = 0;
        for (a, b) in from_comps.iter().zip(to_comps.iter()) {
            if a == b {
                common += 1;
            } else {
                break;
            }
        }
        let mut rel = PathBuf::new();
        for _ in common..from_comps.len() {
            rel.push("..");
        }
        for c in &to_comps[common..] {
            rel.push(c.as_os_str());
        }
        if rel.as_os_str().is_empty() {
            PathBuf::from(".")
        } else {
            rel
        }
    }

    fn bdf(s: &str) -> PciBusId {
        PciBusId::from_str(s).unwrap_or_else(|e| panic!("bdf {s}: {e}"))
    }

    fn blank_identity(bus: &str) -> DeviceIdentity {
        DeviceIdentity {
            uuid: None,
            bdf: bdf(bus),
            agent_name: "gfxTEST".into(),
            product_name: "test".into(),
            kfd_node: 0,
            rocr_index: 0,
            hip_ordinal: None,
            pci_slot: None,
            drm_card: None,
        }
    }

    /// hipx discrete in slot 1: GPU is NOT in the slot table; ancestor is.
    ///
    /// Canonical path (from real host):
    /// `.../0000:68:01.0/0000:6a:00.0/0000:6b:01.0/0000:6c:00.0/0000:6d:00.0/0000:6e:00.0`
    /// Slot table: `{0: 0000:01:00, 0-1: 0000:31:00, 1: 0000:6a:00, 3: 0000:95:00}`
    /// → `Some("1")`.
    #[test]
    fn pci_slot_walks_ancestors_hipx_slot1() {
        let fx = Fixture::create("hipx-slot1").unwrap();
        let chain = [
            "0000:68:01.0",
            "0000:6a:00.0",
            "0000:6b:01.0",
            "0000:6c:00.0",
            "0000:6d:00.0",
            "0000:6e:00.0",
        ];
        fx.install_pci_chain(&chain).unwrap();
        fx.install_slot("0", "0000:01:00").unwrap();
        fx.install_slot("0-1", "0000:31:00").unwrap();
        fx.install_slot("1", "0000:6a:00").unwrap();
        fx.install_slot("3", "0000:95:00").unwrap();

        let got = resolve_pci_slot(fx.path(), bdf("0000:6e:00.0"));
        assert_eq!(got.as_deref(), Some("1"));
    }

    /// hipx board with no ancestor in the slot table → `None`.
    ///
    /// Path: `.../0000:00:03.1/0000:64:00.0/0000:65:00.0/0000:66:00.0`
    #[test]
    fn pci_slot_none_when_no_ancestor_matches() {
        let fx = Fixture::create("hipx-noslot").unwrap();
        let chain = [
            "0000:00:03.1",
            "0000:64:00.0",
            "0000:65:00.0",
            "0000:66:00.0",
        ];
        fx.install_pci_chain(&chain).unwrap();
        fx.install_slot("0", "0000:01:00").unwrap();
        fx.install_slot("0-1", "0000:31:00").unwrap();
        fx.install_slot("1", "0000:6a:00").unwrap();
        fx.install_slot("3", "0000:95:00").unwrap();

        let got = resolve_pci_slot(fx.path(), bdf("0000:66:00.0"));
        assert_eq!(got, None);
    }

    /// Nearest ancestor wins when more than one matches.
    #[test]
    fn pci_slot_nearest_ancestor_wins() {
        let fx = Fixture::create("nearest").unwrap();
        let chain = ["0000:10:00.0", "0000:20:00.0", "0000:30:00.0"];
        fx.install_pci_chain(&chain).unwrap();
        fx.install_slot("far", "0000:10:00").unwrap();
        fx.install_slot("near", "0000:20:00").unwrap();

        let got = resolve_pci_slot(fx.path(), bdf("0000:30:00.0"));
        assert_eq!(got.as_deref(), Some("near"));
    }

    #[test]
    fn drm_card_maps_bdf_ignoring_connectors() {
        let fx = Fixture::create("drm").unwrap();
        let device_dir = fx.install_pci_chain(&["0000:3d:00.0"]).unwrap();
        fx.install_drm_card("card0", &device_dir).unwrap();
        // Connector entry pointing at the same device must not be chosen.
        fx.install_drm_card("card0-DP-1", &device_dir).unwrap();
        // Unrelated card.
        let other = fx.install_pci_chain(&["0000:aa:00.0"]).unwrap();
        fx.install_drm_card("card1", &other).unwrap();

        let got = resolve_drm_card(fx.path(), bdf("0000:3d:00.0"));
        assert_eq!(got.as_deref(), Some("card0"));
    }

    #[test]
    fn missing_sysfs_yields_none() {
        let fx = Fixture::create("empty").unwrap();
        // No devices, no slots, no drm.
        assert_eq!(resolve_pci_slot(fx.path(), bdf("0000:66:00.0")), None);
        assert_eq!(resolve_drm_card(fx.path(), bdf("0000:66:00.0")), None);

        let mut id = blank_identity("0000:66:00.0");
        enrich_device_identity_from(&mut id, fx.path());
        assert_eq!(id.pci_slot, None);
        assert_eq!(id.drm_card, None);
    }

    #[test]
    fn enrich_sets_both_fields() {
        let fx = Fixture::create("enrich").unwrap();
        let chain = [
            "0000:68:01.0",
            "0000:6a:00.0",
            "0000:6b:01.0",
            "0000:6c:00.0",
            "0000:6d:00.0",
            "0000:6e:00.0",
        ];
        let device_dir = fx.install_pci_chain(&chain).unwrap();
        fx.install_slot("1", "0000:6a:00").unwrap();
        fx.install_drm_card("card2", &device_dir).unwrap();

        let mut id = blank_identity("0000:6e:00.0");
        enrich_device_identity_from(&mut id, fx.path());
        assert_eq!(id.pci_slot.as_deref(), Some("1"));
        assert_eq!(id.drm_card.as_deref(), Some("card2"));
    }
}
