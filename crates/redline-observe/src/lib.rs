// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! ROCm Core SDK 7.14 observability for Redline.
//!
//! Three facilities, each loading its ROCm library from the TheRock layout
//! (`/opt/rocm/core/lib`) with a legacy `/opt/rocm/lib` fallback:
//!
//! - [`roctx`]: ROCTx markers and profiler control.
//!   - Nested **marker ranges** via `Roctx::range` / `RangeGuard` (push/pop) —
//!     naming only; not the selected-regions control path.
//!   - **Selected regions** for `rocprofv3 --selected-regions` via
//!     `Roctx::selected_region` / `SelectedRegionGuard` (Resume → Pause).
//! - [`amdsmi`]: clock/temperature/power telemetry via `libamd_smi`.
//! - [`rocprof`]: embedded rocprofiler-sdk dispatch counter collection.
//!
//! ROCm 7.14 is a hard requirement: missing libraries or symbols fail at
//! load with a named error; there is no silent no-op degradation.

pub mod amdsmi;
pub mod rocprof;
pub mod roctx;
