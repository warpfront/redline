// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Kaden Schutt

//! HipGraph-shaped migration surface over the Redline record/replay core.
//!
//! If you already drive kernels through `hipGraph_t` / `hipGraphExec_t` — or a
//! Vulkan command buffer you record once and resubmit — this module gives you
//! the same **capture → instantiate → launch** shape, mapping 1:1 onto
//! [`Recorder`]:
//!
//! | HipGraph / Vulkan                  | Redline (`hipgraph` adapter)            |
//! |------------------------------------|-----------------------------------------|
//! | `hipGraphCreate`                   | [`Graph::new`] / [`Graph::with_tuning`] |
//! | buffer a node reads/writes         | [`Graph::buffer`] + [`Graph::region`]   |
//! | `hipGraphAddKernelNode`            | [`Graph::kernel`] / [`Graph::kernel_after`] |
//! | node dependency array              | `deps` arg to [`Graph::kernel_after`]   |
//! | `hipGraphInstantiate` → `hipGraphExec_t` | [`Graph::instantiate`] → [`GraphExec`] |
//! | `hipGraphLaunch(exec, stream)`     | [`GraphExec::launch`]                   |
//! | re-record buffer bindings, re-instantiate | rebind + [`Graph::instantiate`]   |
//!
//! **The one thing HipGraph infers that Redline asks you to state:** each kernel
//! node's buffer reads and writes, via [`Access::read`] / [`Access::write`].
//! That single declaration is what lets Redline derive the *minimal* correct
//! fence set instead of HipGraph's blanket per-dispatch system-scope
//! acquire/release — it is the source of both the safety (hazards are checked,
//! not assumed) and the lower dispatch floor. If you already know your kernel's
//! I/O — and a HipGraph author does — this is a few extra tokens per node.
//!
//! ```
//! use redline_dispatch::hipgraph::{Graph, Tuning};
//! use redline_dispatch::mock::MockBackend;
//! use redline_dispatch::{Access, Dim3, KernelLaunch, ReplayToken};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut graph = Graph::with_tuning(Tuning::latency());
//! let acts = graph.buffer("activations", 4096)?;
//! let input = graph.region(acts, 0, 2048)?;
//! let output = graph.region(acts, 2048, 2048)?;
//!
//! let project = graph.kernel(
//!     KernelLaunch::new("project", Dim3::x(32)?, Dim3::x(64)?)?,
//!     [Access::read(input), Access::write(output)],
//! )?;
//! let _consume = graph.kernel_after(
//!     KernelLaunch::new("consume", Dim3::x(32)?, Dim3::x(64)?)?,
//!     [Access::read(output)],
//!     [project],
//! )?;
//!
//! let exec = graph.instantiate()?;
//! let mut backend = MockBackend::default();
//! exec.launch(&mut backend, ReplayToken(0))?; // == hipGraphLaunch
//! # Ok(())
//! # }
//! ```

use std::num::NonZeroUsize;

use crate::{
    Access, CompileError, CompileOptions, CompiledPlan, DeviceRegion, DispatchBackend, KernelLaunch,
    NodeId, RecordError, Recorder, ReplayMode, ReplayToken, ResourceId,
};

/// Replay tuning knobs.
///
/// The default is the safest HipGraph-equivalent behavior: one serial lane with
/// terminal-signal (latency) completion — every launch completes before the
/// next begins, exactly like replaying one captured stream. Raise `lanes` to let
/// independent branches of the graph overlap; switch to
/// [`Tuning::throughput`] to permit whole-token overlap with a bounded
/// in-flight window.
#[derive(Clone, Copy, Debug)]
pub struct Tuning {
    /// Independent logical replay lanes. `1` behaves like a single serial
    /// stream (the default). Higher values let disjoint graph branches run
    /// concurrently on the backend.
    pub lanes: usize,
    /// Completion policy for one graph launch.
    pub mode: ReplayMode,
}

impl Default for Tuning {
    fn default() -> Self {
        Self {
            lanes: 1,
            mode: ReplayMode::TokenLatency,
        }
    }
}

impl Tuning {
    /// Latency-first (default): one lane; every terminal signal completes before
    /// the launch returns. The closest match to a single `hipStream`/graph.
    pub fn latency() -> Self {
        Self::default()
    }

    /// Overlap-first: `lanes` independent lanes, terminal-signal completion.
    /// `lanes` is clamped to at least 1.
    pub fn overlap(lanes: usize) -> Self {
        Self {
            lanes: lanes.max(1),
            mode: ReplayMode::TokenLatency,
        }
    }

    /// Throughput: permit whole-token overlap while bounding the number of
    /// launches in flight. Both arguments are clamped to at least 1.
    pub fn throughput(lanes: usize, max_tokens_in_flight: usize) -> Self {
        let max = NonZeroUsize::new(max_tokens_in_flight.max(1)).expect("clamped to >= 1");
        Self {
            lanes: lanes.max(1),
            mode: ReplayMode::Throughput {
                max_tokens_in_flight: max,
            },
        }
    }

    fn compile_options(self) -> CompileOptions {
        let lanes = NonZeroUsize::new(self.lanes.max(1)).expect("clamped to >= 1");
        CompileOptions::new(lanes, self.mode)
    }
}

/// A capturing graph — a [`Recorder`] behind a HipGraph-shaped surface.
///
/// Build it up with [`buffer`](Self::buffer) / [`region`](Self::region) /
/// [`kernel`](Self::kernel) / [`kernel_after`](Self::kernel_after), then call
/// [`instantiate`](Self::instantiate) to get a replayable [`GraphExec`].
pub struct Graph {
    recorder: Recorder,
    tuning: Tuning,
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

impl Graph {
    /// Create an empty graph with default ([`Tuning::latency`]) tuning.
    pub fn new() -> Self {
        Self {
            recorder: Recorder::new(),
            tuning: Tuning::default(),
        }
    }

    /// Create an empty graph with explicit replay [`Tuning`].
    pub fn with_tuning(tuning: Tuning) -> Self {
        Self {
            recorder: Recorder::new(),
            tuning,
        }
    }

    /// Change the replay [`Tuning`] before [`instantiate`](Self::instantiate).
    pub fn set_tuning(&mut self, tuning: Tuning) -> &mut Self {
        self.tuning = tuning;
        self
    }

    /// The current tuning.
    pub fn tuning(&self) -> Tuning {
        self.tuning
    }

    /// Declare a device allocation that participates in the graph (label +
    /// byte size) — analogous to a buffer a HipGraph kernel node reads/writes.
    pub fn buffer(
        &mut self,
        label: impl Into<String>,
        size: u64,
    ) -> Result<ResourceId, RecordError> {
        self.recorder.resource(label, size)
    }

    /// A sub-region `[offset, offset + len)` of a [`buffer`](Self::buffer), used
    /// to declare per-node [`Access`].
    pub fn region(
        &self,
        buffer: ResourceId,
        offset: u64,
        len: u64,
    ) -> Result<DeviceRegion, RecordError> {
        self.recorder.region(buffer, offset, len)
    }

    /// Add a kernel node with no prerequisites (a graph root).
    ///
    /// `accesses` declares the node's buffer reads/writes — the dependency
    /// information HipGraph infers, made explicit so Redline can derive minimal
    /// fences.
    pub fn kernel(
        &mut self,
        launch: KernelLaunch,
        accesses: impl IntoIterator<Item = Access>,
    ) -> Result<NodeId, RecordError> {
        self.recorder.dispatch(launch, accesses)
    }

    /// Add a kernel node that runs after `deps` (≈ `hipGraphAddKernelNode` with
    /// a dependency array). `accesses` declares the node's buffer reads/writes.
    pub fn kernel_after(
        &mut self,
        launch: KernelLaunch,
        accesses: impl IntoIterator<Item = Access>,
        deps: impl IntoIterator<Item = NodeId>,
    ) -> Result<NodeId, RecordError> {
        let node = self.recorder.dispatch(launch, accesses)?;
        for dep in deps {
            self.recorder.depends_on(node, dep)?;
        }
        Ok(node)
    }

    /// Instantiate the captured graph into a replayable [`GraphExec`]
    /// (≈ `hipGraphInstantiate` → `hipGraphExec_t`). Hazard and memory-ordering
    /// validation happen here; a bad graph fails closed with [`CompileError`].
    pub fn instantiate(&self) -> Result<GraphExec, CompileError> {
        let plan = self.recorder.compile(self.tuning.compile_options())?;
        Ok(GraphExec { plan })
    }

    /// Borrow the underlying [`Recorder`] for lower-level APIs not surfaced by
    /// this adapter (explicit `depends_on`, etc.).
    pub fn recorder(&mut self) -> &mut Recorder {
        &mut self.recorder
    }
}

/// An instantiated, replayable graph — the analogue of `hipGraphExec_t`.
pub struct GraphExec {
    plan: CompiledPlan,
}

impl GraphExec {
    /// Launch (replay) the graph on `backend` (≈ `hipGraphLaunch(exec, stream)`).
    /// Replay is allocation-free; call it as many times as you like.
    pub fn launch<B: DispatchBackend>(
        &self,
        backend: &mut B,
        token: ReplayToken,
    ) -> Result<B::Completion, B::Error> {
        self.plan.replay(backend, token)
    }

    /// The underlying compiled plan (fingerprint, lanes, hazards, …).
    pub fn plan(&self) -> &CompiledPlan {
        &self.plan
    }
}
