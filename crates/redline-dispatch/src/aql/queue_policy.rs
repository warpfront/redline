// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

use std::fmt;
use std::str::FromStr;

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
    if architecture.starts_with("gfx12") {
        2
    } else if architecture.starts_with("gfx11") {
        4
    } else {
        // gfx10 queue fan-out has not been certified yet. Unknown future
        // architectures must also fail closed instead of inheriting a tuning.
        1
    }
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
        assert_eq!(QueuePolicy::Auto.resolve("gfx1100", 16), 4);
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
}
