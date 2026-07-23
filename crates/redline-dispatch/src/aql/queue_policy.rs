// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

use std::fmt;
use std::str::FromStr;

use crate::partition::{self, CuPartition, PartitionError, PartitionPolicy};
use redline_rocr::{GpuDevice, QueueSet, RuntimeError};

/// Public-queue fan-out policy for independent retained work.
///
/// `Auto` is deliberately conservative on unmeasured architectures. The
/// architecture table records only queue counts established by the #6409
/// same-HSACO queue sweeps; explicit variants remain available for diagnosis.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(u32)]
pub enum QueuePolicy {
    #[default]
    Auto = 0,
    One = 1,
    Two = 2,
    Four = 4,
}

impl QueuePolicy {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::One => "1",
            Self::Two => "2",
            Self::Four => "4",
        }
    }

    pub const fn explicit_lanes(self) -> Option<usize> {
        match self {
            Self::Auto => None,
            Self::One => Some(1),
            Self::Two => Some(2),
            Self::Four => Some(4),
        }
    }

    /// Resolve this policy for an architecture and independent antichain.
    /// Serial callers pass an independent width of one and therefore remain
    /// single-queue regardless of policy.
    pub fn resolve(self, architecture: &str, independent_width: usize) -> usize {
        let available = independent_width.max(1);
        let requested = self
            .explicit_lanes()
            .unwrap_or_else(|| automatic_lane_limit(architecture));
        requested.min(available)
    }
}

fn automatic_lane_limit(architecture: &str) -> usize {
    let architecture = architecture.to_ascii_lowercase();
    if architecture.starts_with("gfx12") || architecture.starts_with("gfx1100") {
        2
    } else if architecture.starts_with("gfx11") {
        4
    } else {
        // gfx10 queue fan-out has not been certified yet. Unknown future
        // architectures must also fail closed instead of inheriting a tuning.
        1
    }
}

/// Pack a contiguous CU slice into a bool mask for `hsa_amd_queue_cu_set_mask`.
///
/// Bits `cu_offset .. cu_offset+cu_count` are true; the vector length is the
/// device CU count so the HSA bit-vector covers the full agent.
pub fn cu_mask_for_partition(device_cu_count: u32, part: CuPartition) -> Vec<bool> {
    let len = device_cu_count as usize;
    let start = part.cu_offset as usize;
    let end = start.saturating_add(part.cu_count as usize);
    let mut mask = vec![false; len];
    let end = end.min(len);
    let start = start.min(end);
    for slot in mask.iter_mut().take(end).skip(start) {
        *slot = true;
    }
    mask
}

/// Failures while materializing a CU-partitioned [`QueueSet`].
#[derive(Debug, thiserror::Error)]
pub enum PartitionedQueueError {
    #[error(transparent)]
    Partition(#[from] PartitionError),
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
    #[error(
        "partition policy produced {partitions} CU slices but queue creation requested {queues} lanes"
    )]
    LaneCountMismatch { partitions: usize, queues: usize },
}

/// Create `queue_count` public queues, optionally CU-masked from `policy`.
///
/// When `policy` is `None` or [`PartitionPolicy::None`], this is byte-equivalent
/// to [`QueueSet::create`] (no CU mask API is invoked). Any other policy is
/// validated against `GpuDevice::compute_unit_count()` and each lane receives
/// `create_with_cu_mask` covering its [`CuPartition`] slice. If the validated
/// partition count differs from `queue_count`, returns
/// [`PartitionedQueueError::LaneCountMismatch`].
pub fn create_queue_set(
    device: &GpuDevice,
    queue_count: usize,
    queue_size: u32,
    policy: Option<&PartitionPolicy>,
) -> Result<QueueSet, PartitionedQueueError> {
    let Some(policy) = policy else {
        return Ok(QueueSet::create(device, queue_count, queue_size)?);
    };
    if matches!(policy, PartitionPolicy::None) {
        return Ok(QueueSet::create(device, queue_count, queue_size)?);
    }
    let device_cu_count = device.compute_unit_count()?;
    let partitions = partition::validate(policy, device_cu_count)?;
    if partitions.len() != queue_count {
        return Err(PartitionedQueueError::LaneCountMismatch {
            partitions: partitions.len(),
            queues: queue_count,
        });
    }
    let masks = partitions
        .into_iter()
        .map(|part| cu_mask_for_partition(device_cu_count, part))
        .collect::<Vec<_>>();
    Ok(QueueSet::create_with_cu_masks(device, queue_size, &masks)?)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueuePolicyParseError(String);

impl fmt::Display for QueuePolicyParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unknown queue policy {:?}; expected auto, 1, 2, or 4",
            self.0
        )
    }
}

impl std::error::Error for QueuePolicyParseError {}

impl FromStr for QueuePolicy {
    type Err = QueuePolicyParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "1" | "one" => Ok(Self::One),
            "2" | "two" => Ok(Self::Two),
            "4" | "four" => Ok(Self::Four),
            _ => Err(QueuePolicyParseError(value.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn automatic_policy_uses_certified_architecture_caps() {
        assert_eq!(QueuePolicy::Auto.resolve("gfx1100", 16), 2);
        assert_eq!(QueuePolicy::Auto.resolve("gfx1151", 16), 4);
        assert_eq!(QueuePolicy::Auto.resolve("gfx1201", 16), 2);
        assert_eq!(QueuePolicy::Auto.resolve("gfx1030", 16), 1);
        assert_eq!(QueuePolicy::Auto.resolve("gfx1010", 16), 1);
        assert_eq!(QueuePolicy::Auto.resolve("gfx9999", 16), 1);
    }

    #[test]
    fn policy_never_exceeds_independent_width() {
        assert_eq!(QueuePolicy::Four.resolve("gfx1100", 2), 2);
        assert_eq!(QueuePolicy::Two.resolve("gfx1201", 1), 1);
        assert_eq!(QueuePolicy::Auto.resolve("gfx1151", 0), 1);
    }

    #[test]
    fn parsing_preserves_explicit_diagnostic_overrides() {
        assert_eq!("auto".parse(), Ok(QueuePolicy::Auto));
        assert_eq!("1".parse(), Ok(QueuePolicy::One));
        assert_eq!("2".parse(), Ok(QueuePolicy::Two));
        assert_eq!("4".parse(), Ok(QueuePolicy::Four));
        assert!("8".parse::<QueuePolicy>().is_err());
    }

    #[test]
    fn cu_mask_for_partition_sets_contiguous_bits() {
        let mask = cu_mask_for_partition(
            8,
            CuPartition {
                index: 1,
                cu_offset: 2,
                cu_count: 3,
            },
        );
        assert_eq!(
            mask,
            vec![false, false, true, true, true, false, false, false]
        );
    }

    #[test]
    fn equal_halves_cover_full_device_without_overlap() {
        let parts = partition::validate(
            &PartitionPolicy::Equal(std::num::NonZeroUsize::new(2).unwrap()),
            64,
        )
        .unwrap();
        assert_eq!(parts.len(), 2);
        let left = cu_mask_for_partition(64, parts[0]);
        let right = cu_mask_for_partition(64, parts[1]);
        assert_eq!(left.iter().filter(|&&b| b).count(), 32);
        assert_eq!(right.iter().filter(|&&b| b).count(), 32);
        assert!(left.iter().zip(right.iter()).all(|(a, b)| !(*a && *b)));
        assert_eq!(
            left.iter()
                .zip(right.iter())
                .filter(|(a, b)| **a || **b)
                .count(),
            64
        );
    }
}
