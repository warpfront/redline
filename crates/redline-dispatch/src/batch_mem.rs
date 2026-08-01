// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! Pure plan types for HIP 7.14 batch memory prefetch/discard around replay.
//!
//! A [`BatchMemPlan`] describes fixture ranges to bring onto the device before
//! a replay batch and ranges to discard after it. Execution is **not** done
//! here: Wave 3 wiring runs the plan through `ffi_batch_mem` with
//! **prefetch-before-replay** and **discard-after** semantics (design spec L2 /
//! `hipMemPrefetchBatchAsync` then replay then `hipMemDiscardBatchAsync`).
//!
//! # Alignment
//!
//! Ranges are **not** required to be page-aligned. The HIP batch APIs
//! (`hipMemPrefetchBatchAsync`, `hipMemDiscardBatchAsync`, and the fused /
//! driver variants) take arbitrary base pointers and byte lengths with no
//! documented page-granularity constraint in `hip_runtime_api.h`. Validation
//! therefore only enforces non-zero sizes and non-overlapping ranges within
//! each list; page rounding is left to the runtime/driver if needed.
//!
//! # Empty plan
//!
//! [`BatchMemPlan::empty`] (and a builder with no ranges) is a validated no-op:
//! both lists are empty, so the backend skips batch-mem work around replay.

/// One device/managed address range for batch prefetch or discard.
///
/// `device_ptr` is the numeric device (or managed) base address; `size` is the
/// byte length. Pointers are stored as `u64` so this crate stays free of FFI
/// types until Wave 3 binds them to `void*` / `hipDeviceptr_t`.
///
/// Fields are private so raw addresses cannot be injected through a safe
/// struct literal. Use [`BatchMemPlan::try_new`] / [`BatchMemPlanBuilder`] for
/// validated plan assembly, or [`BatchMemRange::new`] only when the caller can
/// uphold the device-pointer contract.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BatchMemRange {
    device_ptr: u64,
    size: usize,
}

impl BatchMemRange {
    /// Construct a range from a raw device/managed address.
    ///
    /// Prefer [`BatchMemPlanBuilder`] / [`BatchMemPlan::try_new`] so size and
    /// overlap validation runs once for the whole plan. Those safe builders
    /// still require each range to satisfy the pointer contract below.
    ///
    /// # Safety
    ///
    /// - `device_ptr` must be a HIP device pointer or managed/unified pointer
    ///   returned by the HIP runtime (or an equivalent driver allocation).
    /// - The allocation must remain live and cover at least `size` bytes for
    ///   the entire lifetime of every [`BatchMemPlan`] that embeds this range
    ///   and through every prefetch/discard that consumes it.
    /// - `device_ptr + size` must not overflow the address space used by the
    ///   HIP batch APIs (checked again by plan validation when `size > 0`).
    pub const unsafe fn new(device_ptr: u64, size: usize) -> Self {
        Self { device_ptr, size }
    }

    /// Device/managed base address as a numeric pointer value.
    pub const fn device_ptr(self) -> u64 {
        self.device_ptr
    }

    /// Byte length of the range.
    pub const fn size(self) -> usize {
        self.size
    }

    /// Exclusive end address in the unsigned address space used for overlap
    /// checks. Returns `None` if `device_ptr + size` overflows `u64`.
    pub fn end_exclusive(self) -> Option<u64> {
        let size = u64::try_from(self.size).ok()?;
        self.device_ptr.checked_add(size)
    }
}

/// Prefetch-before / discard-after range lists for one replay batch.
///
/// Fields are private so callers cannot inject unvalidated device addresses.
/// Construct only via [`BatchMemPlan::empty`], [`BatchMemPlan::try_new`], or
/// [`BatchMemPlanBuilder`]. Invalid inputs are rejected up front so the
/// backend can treat a held plan as already validated.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BatchMemPlan {
    /// Ranges to prefetch onto the device **before** replay.
    prefetch: Vec<BatchMemRange>,
    /// Ranges to discard **after** replay completes.
    discard: Vec<BatchMemRange>,
}

impl BatchMemPlan {
    /// Empty plan: no prefetch and no discard. Backend treats this as a no-op.
    pub const fn empty() -> Self {
        Self {
            prefetch: Vec::new(),
            discard: Vec::new(),
        }
    }

    /// True when both lists are empty (no batch-mem work around replay).
    pub fn is_noop(&self) -> bool {
        self.prefetch.is_empty() && self.discard.is_empty()
    }

    /// Validated prefetch ranges (may be empty).
    pub fn prefetch(&self) -> &[BatchMemRange] {
        &self.prefetch
    }

    /// Validated discard ranges (may be empty).
    pub fn discard(&self) -> &[BatchMemRange] {
        &self.discard
    }

    /// True when the discard list is non-empty (extra host sync after replay).
    pub fn has_discard(&self) -> bool {
        !self.discard.is_empty()
    }

    /// Validate and build a plan from the given lists.
    ///
    /// This is a fully-validating safe constructor; prefer
    /// [`BatchMemPlanBuilder`] for incremental assembly.
    pub fn try_new(
        prefetch: Vec<BatchMemRange>,
        discard: Vec<BatchMemRange>,
    ) -> Result<Self, BatchMemError> {
        validate_range_list(&prefetch, BatchMemListKind::Prefetch)?;
        validate_range_list(&discard, BatchMemListKind::Discard)?;
        Ok(Self { prefetch, discard })
    }

    /// Start a builder for incremental construction.
    pub fn builder() -> BatchMemPlanBuilder {
        BatchMemPlanBuilder::new()
    }
}

/// Which plan list a validation error refers to.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BatchMemListKind {
    Prefetch,
    Discard,
}

impl core::fmt::Display for BatchMemListKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Prefetch => f.write_str("prefetch"),
            Self::Discard => f.write_str("discard"),
        }
    }
}

/// Validation failures for [`BatchMemPlan`] construction.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum BatchMemError {
    #[error("{list} range at index {index} has zero size")]
    ZeroSize {
        list: BatchMemListKind,
        index: usize,
    },
    #[error(
        "{list} ranges at indices {first_index} and {second_index} overlap \
         ([{first_ptr:#x}, +{first_size}) and [{second_ptr:#x}, +{second_size}))"
    )]
    Overlap {
        list: BatchMemListKind,
        first_index: usize,
        second_index: usize,
        first_ptr: u64,
        first_size: usize,
        second_ptr: u64,
        second_size: usize,
    },
    #[error(
        "{list} range at index {index} overflows address space \
         (device_ptr={device_ptr:#x}, size={size})"
    )]
    AddressOverflow {
        list: BatchMemListKind,
        index: usize,
        device_ptr: u64,
        size: usize,
    },
}

/// Incremental builder with the same validation rules as [`BatchMemPlan::try_new`].
#[derive(Clone, Debug, Default)]
pub struct BatchMemPlanBuilder {
    prefetch: Vec<BatchMemRange>,
    discard: Vec<BatchMemRange>,
}

impl BatchMemPlanBuilder {
    pub const fn new() -> Self {
        Self {
            prefetch: Vec::new(),
            discard: Vec::new(),
        }
    }

    pub fn prefetch(mut self, range: BatchMemRange) -> Self {
        self.prefetch.push(range);
        self
    }

    pub fn discard(mut self, range: BatchMemRange) -> Self {
        self.discard.push(range);
        self
    }

    pub fn prefetch_all<I>(mut self, ranges: I) -> Self
    where
        I: IntoIterator<Item = BatchMemRange>,
    {
        self.prefetch.extend(ranges);
        self
    }

    pub fn discard_all<I>(mut self, ranges: I) -> Self
    where
        I: IntoIterator<Item = BatchMemRange>,
    {
        self.discard.extend(ranges);
        self
    }

    /// Validate and produce a [`BatchMemPlan`].
    pub fn build(self) -> Result<BatchMemPlan, BatchMemError> {
        BatchMemPlan::try_new(self.prefetch, self.discard)
    }
}

fn validate_range_list(
    ranges: &[BatchMemRange],
    list: BatchMemListKind,
) -> Result<(), BatchMemError> {
    for (index, range) in ranges.iter().enumerate() {
        if range.size == 0 {
            return Err(BatchMemError::ZeroSize { list, index });
        }
        if range.end_exclusive().is_none() {
            return Err(BatchMemError::AddressOverflow {
                list,
                index,
                device_ptr: range.device_ptr,
                size: range.size,
            });
        }
    }

    // Pairwise half-open overlap: [a, a+sa) ∩ [b, b+sb) ≠ ∅.
    // Order-independent; original indices are reported for diagnostics.
    for (i, a) in ranges.iter().enumerate() {
        let a_end = a.end_exclusive().expect("validated non-overflow");
        for (j, b) in ranges.iter().enumerate().skip(i + 1) {
            let b_end = b.end_exclusive().expect("validated non-overflow");
            let overlaps = a.device_ptr < b_end && b.device_ptr < a_end;
            if overlaps {
                return Err(BatchMemError::Overlap {
                    list,
                    first_index: i,
                    second_index: j,
                    first_ptr: a.device_ptr,
                    first_size: a.size,
                    second_ptr: b.device_ptr,
                    second_size: b.size,
                });
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test-only range construction; production callers use the unsafe contract.
    fn range(device_ptr: u64, size: usize) -> BatchMemRange {
        // SAFETY: unit tests use synthetic addresses solely for validation logic.
        unsafe { BatchMemRange::new(device_ptr, size) }
    }

    #[test]
    fn empty_plan_is_noop() {
        let plan = BatchMemPlan::empty();
        assert!(plan.is_noop());
        assert!(plan.prefetch().is_empty());
        assert!(plan.discard().is_empty());

        let built = BatchMemPlan::builder().build().expect("empty builder ok");
        assert!(built.is_noop());
        assert_eq!(built, BatchMemPlan::empty());
    }

    #[test]
    fn zero_size_rejected_in_prefetch() {
        let err = BatchMemPlan::try_new(vec![range(0x1000, 0)], vec![]).expect_err("zero size");
        assert_eq!(
            err,
            BatchMemError::ZeroSize {
                list: BatchMemListKind::Prefetch,
                index: 0,
            }
        );
    }

    #[test]
    fn zero_size_rejected_in_discard() {
        let err = BatchMemPlan::builder()
            .discard(range(0x2000, 0))
            .build()
            .expect_err("zero size");
        assert_eq!(
            err,
            BatchMemError::ZeroSize {
                list: BatchMemListKind::Discard,
                index: 0,
            }
        );
    }

    #[test]
    fn overlap_detected_within_prefetch() {
        // [0x1000, 0x1800) and [0x1400, 0x1c00)
        let err = BatchMemPlan::try_new(vec![range(0x1000, 0x800), range(0x1400, 0x800)], vec![])
            .expect_err("overlap");
        match err {
            BatchMemError::Overlap {
                list: BatchMemListKind::Prefetch,
                first_index: 0,
                second_index: 1,
                first_ptr: 0x1000,
                first_size: 0x800,
                second_ptr: 0x1400,
                second_size: 0x800,
            } => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn overlap_detected_within_discard() {
        let err = BatchMemPlan::builder()
            .discard(range(0x10, 16))
            .discard(range(0x18, 8))
            .build()
            .expect_err("overlap");
        match err {
            BatchMemError::Overlap {
                list: BatchMemListKind::Discard,
                first_index: 0,
                second_index: 1,
                ..
            } => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn adjacent_ranges_are_not_overlap() {
        // [0x1000, 0x1800) and [0x1800, 0x2000) touch at the boundary only.
        let plan = BatchMemPlan::try_new(vec![range(0x1000, 0x800), range(0x1800, 0x800)], vec![])
            .expect("adjacent ok");
        assert!(!plan.is_noop());
        assert_eq!(plan.prefetch().len(), 2);
    }

    #[test]
    fn disjoint_ranges_ok_in_both_lists() {
        let plan = BatchMemPlan::builder()
            .prefetch(range(0x1000, 0x100))
            .prefetch(range(0x2000, 0x100))
            .discard(range(0x3000, 0x40))
            .discard(range(0x4000, 0x40))
            .build()
            .expect("disjoint ok");
        assert!(!plan.is_noop());
        assert_eq!(plan.prefetch().len(), 2);
        assert_eq!(plan.discard().len(), 2);
    }

    #[test]
    fn same_address_in_prefetch_and_discard_is_allowed() {
        // Overlap is only checked within each list, not across lists.
        let r = range(0x1000, 64);
        let plan = BatchMemPlan::try_new(vec![r], vec![r]).expect("cross-list same range ok");
        assert_eq!(plan.prefetch(), plan.discard());
    }

    #[test]
    fn unaligned_sizes_and_ptrs_accepted() {
        // Page alignment is intentionally not required (see module docs).
        let plan = BatchMemPlan::try_new(vec![range(0x1003, 17)], vec![range(0xabcd, 3)])
            .expect("unaligned ok");
        assert_eq!(plan.prefetch()[0].size(), 17);
        assert_eq!(plan.discard()[0].device_ptr(), 0xabcd);
    }

    #[test]
    fn address_overflow_rejected() {
        let err =
            BatchMemPlan::try_new(vec![range(u64::MAX - 8, 16)], vec![]).expect_err("overflow");
        match err {
            BatchMemError::AddressOverflow {
                list: BatchMemListKind::Prefetch,
                index: 0,
                device_ptr,
                size: 16,
            } if device_ptr == u64::MAX - 8 => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
