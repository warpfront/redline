// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! Device identity: one record that joins every view of a GPU, and the query
//! language for pinning one deterministically.
//!
//! # Why this exists
//!
//! Every index a human can read is volatile, and they disagree with each other
//! on real hardware. Measured on a four-GPU host (`hipx`):
//!
//! | ROCr index | rocm-smi | agent   | UUID                    | PCI          |
//! | ---------- | -------- | ------- | ----------------------- | ------------ |
//! | 0          | GPU\[0\] | gfx1100 | GPU-43390a851e296ee5    | 0000:66:00.0 |
//! | 1          | GPU\[3\] | gfx1151 | *(none)*                | 0000:bf:00.0 |
//! | 2          | GPU\[1\] | gfx1010 | *(none)*                | 0000:6e:00.0 |
//! | 3          | GPU\[2\] | gfx1030 | GPU-c7ff6b154d0128bc    | 0000:99:00.0 |
//!
//! ROCr and KFD enumerate in discovery order; rocm-smi sorts by PCI bus. So
//! rocm-smi `GPU[3]` and ROCr index 1 are the same device, and on that host it
//! is an integrated APU whose device reset takes the whole machine down. An
//! ordinal copied from the wrong tool is not a mislabeled benchmark row, it is
//! a dead host.
//!
//! A second host of four identical `gfx1201` boards makes name matching
//! useless, and two of the four `hipx` GPUs report no UUID at all. Hence:
//!
//! - The **anchor** is the UUID when the device has one, otherwise the PCI
//!   address. Never an index. [`DeviceIdentity::anchor`] says which one a
//!   device actually got, so callers can warn that a BDF-anchored device moves
//!   if it is re-slotted.
//! - The PCI address is the **join key** across HSA, HIP, sysfs, rocm-smi, and
//!   `amdgpu` fault lines in dmesg — it is the only field present everywhere.
//! - Indices are carried for reporting and never matched against.

use std::fmt;

use crate::runtime::PciBusId;

/// The stable handle for a device, in priority order.
///
/// A device that reports no UUID (integrated parts and some older discrete
/// boards report `GPU-XX`) can only be anchored by its PCI address, which is
/// stable across reboots but not across re-slotting or a board swap.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Anchor {
    /// Tied to the silicon; survives re-slotting and PCI renumbering.
    Uuid(String),
    /// Tied to the topology slot; survives reboots, not physical moves.
    Bdf(PciBusId),
}

impl fmt::Display for Anchor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Uuid(uuid) => write!(f, "uuid:{uuid}"),
            Self::Bdf(bdf) => write!(f, "bdf:{bdf}"),
        }
    }
}

/// Everything known about one GPU, joined across HSA, sysfs, and (optionally)
/// HIP.
///
/// Fields are grouped by trustworthiness: anchors first, description second,
/// volatile indices last. Nothing here is ever invented — a view that cannot be
/// established is `None`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeviceIdentity {
    /// `GPU-43390a851e296ee5`, or `None` when the device reports `GPU-XX`.
    pub uuid: Option<String>,
    /// Always present. The join key across every tool and dmesg.
    pub bdf: PciBusId,
    /// HSA agent name, e.g. `gfx1100`. Not unique on homogeneous hosts.
    pub agent_name: String,
    /// Marketing name, e.g. `Radeon RX 7900 XTX`. Human-facing only.
    pub product_name: String,
    /// KFD topology node id.
    pub kfd_node: u32,
    /// Position in `hsa_iterate_agents` order under the current
    /// `ROCR_VISIBLE_DEVICES`. Volatile: reported, never matched.
    pub rocr_index: usize,
    /// HIP device ordinal, when a HIP runtime was loaded and joined by PCI
    /// address. Volatile, and filtered by `HIP_VISIBLE_DEVICES` independently
    /// of ROCr.
    pub hip_ordinal: Option<i32>,
    /// Firmware/ACPI slot label, resolved by walking PCI ancestors. Frequently
    /// unavailable: absent for every GPU on some hosts.
    pub pci_slot: Option<String>,
    /// `card0`-style DRM node name, when one is bound.
    pub drm_card: Option<String>,
    // There is deliberately no `integrated` / `is_dangerous` field. Both
    // plausible sysfs signals were measured against a host carrying a Strix
    // Halo APU alongside three discrete boards and both fail: the APU's KFD
    // node reports `cpu_cores_count = 0` exactly like the discrete cards, and
    // every node reports `heap_type = 1` because the APU's 96 GiB of unified
    // memory presents as public framebuffer. A bool that silently reads
    // `false` for the one device whose reset takes the host down is worse than
    // no bool, so device danger is declared in the host manifest's deny-list
    // instead, where a human states the reason.
}

impl DeviceIdentity {
    /// The stable handle for this device: UUID when it has one, else PCI.
    pub fn anchor(&self) -> Anchor {
        match &self.uuid {
            Some(uuid) => Anchor::Uuid(uuid.clone()),
            None => Anchor::Bdf(self.bdf),
        }
    }

    /// One-line provenance suitable for a benchmark artifact or a fault report.
    ///
    /// Every volatile view is labeled as such so a number lifted out of this
    /// line cannot be mistaken for a durable handle.
    pub fn describe(&self) -> String {
        let mut text = format!("{} {} [{}]", self.agent_name, self.bdf, self.anchor());
        if let Some(slot) = &self.pci_slot {
            text.push_str(&format!(" slot={slot}"));
        }
        if let Some(card) = &self.drm_card {
            text.push_str(&format!(" {card}"));
        }
        text.push_str(&format!(" kfd_node={}", self.kfd_node));
        text.push_str(&format!(" rocr#{}(volatile)", self.rocr_index));
        if let Some(ordinal) = self.hip_ordinal {
            text.push_str(&format!(" hip#{ordinal}(volatile)"));
        }
        text
    }
}

/// A parsed device selector.
///
/// The string form is the same in an environment variable, a CLI flag, and a
/// manifest entry, so a pin can be copied between them without translation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeviceQuery {
    /// `uuid:GPU-43390a851e296ee5` — the strongest form.
    Uuid(String),
    /// `bdf:0000:66:00.0` (a bare `66:00.0` assumes domain 0).
    Bdf(PciBusId),
    /// `slot:3` — firmware slot label; only usable where the host exposes one.
    Slot(String),
    /// `name:gfx1100` — case-insensitive substring of the agent name. Refused
    /// when it matches more than one device.
    Name(String),
    /// `index:1` — ROCr enumeration order. Volatile by construction; accepted
    /// so existing call sites keep working, and reported as unsafe.
    Index(usize),
    /// `@dev0` — an alias resolved through the host manifest.
    Alias(String),
}
