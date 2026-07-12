// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Kaden Schutt

//! Python bindings for `redline-dispatch` — a leaner, safer HipGraph.
//!
//! ```python
//! import redline_dispatch as rl
//! g = rl.Graph(mode="latency")
//! acts = g.buffer("activations", 4096)
//! project = g.kernel("project", (32, 1, 1), (64, 1, 1),
//!                     accesses=[(acts, 0, 2048, False), (acts, 2048, 2048, True)])
//! g.kernel("consume", (32, 1, 1), (64, 1, 1),
//!          accesses=[(acts, 2048, 2048, False)], deps=[project])
//! exec = g.instantiate()          # == hipGraphInstantiate
//! exec.launch_mock()              # validate ordering/fences without a GPU
//! print(exec.lane_count, exec.fingerprint())
//! ```

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use redline_core::hipgraph::{Graph, GraphExec, Tuning};
use redline_core::mock::MockBackend;
use redline_core::{Access, Dim3, KernelLaunch, NodeId, ReplayToken, ResourceId};

fn to_py<E: std::fmt::Debug>(err: E) -> PyErr {
    PyValueError::new_err(format!("{err:?}"))
}

/// A capturing graph — the HipGraph-shaped surface over the record/replay core.
#[pyclass(name = "Graph")]
struct PyGraph {
    graph: Graph,
    resources: Vec<ResourceId>,
    nodes: Vec<NodeId>,
}

#[pymethods]
impl PyGraph {
    /// `Graph(lanes=1, mode="latency", max_in_flight=1)`.
    /// `mode` is `"latency"`, `"overlap"`, or `"throughput"`.
    #[new]
    #[pyo3(signature = (lanes=1, mode="latency", max_in_flight=1))]
    fn new(lanes: usize, mode: &str, max_in_flight: usize) -> PyResult<Self> {
        let tuning = match mode {
            "latency" => Tuning::latency(),
            "overlap" => Tuning::overlap(lanes),
            "throughput" => Tuning::throughput(lanes, max_in_flight),
            other => return Err(PyValueError::new_err(format!("unknown mode {other:?}"))),
        };
        Ok(Self {
            graph: Graph::with_tuning(tuning),
            resources: Vec::new(),
            nodes: Vec::new(),
        })
    }

    /// Declare a device buffer; returns an opaque buffer handle.
    fn buffer(&mut self, label: &str, size: u64) -> PyResult<u32> {
        let id = self.graph.buffer(label, size).map_err(to_py)?;
        let handle = self.resources.len() as u32;
        self.resources.push(id);
        Ok(handle)
    }

    /// Add a kernel node. `accesses` is a list of `(buffer, offset, len, write)`
    /// tuples; `deps` is a list of node handles. Returns the new node handle.
    #[pyo3(signature = (name, grid, block, accesses=None, deps=None))]
    fn kernel(
        &mut self,
        name: &str,
        grid: (u32, u32, u32),
        block: (u32, u32, u32),
        accesses: Option<Vec<(u32, u64, u64, bool)>>,
        deps: Option<Vec<u32>>,
    ) -> PyResult<u32> {
        let grid = Dim3::new(grid.0, grid.1, grid.2).map_err(to_py)?;
        let block = Dim3::new(block.0, block.1, block.2).map_err(to_py)?;
        let launch = KernelLaunch::new(name, grid, block).map_err(to_py)?;

        let mut acc = Vec::new();
        for (buffer, offset, len, write) in accesses.unwrap_or_default() {
            let res = *self
                .resources
                .get(buffer as usize)
                .ok_or_else(|| PyValueError::new_err(format!("bad buffer handle {buffer}")))?;
            let region = self.graph.region(res, offset, len).map_err(to_py)?;
            acc.push(if write {
                Access::write(region)
            } else {
                Access::read(region)
            });
        }

        let mut dep = Vec::new();
        for d in deps.unwrap_or_default() {
            let node = *self
                .nodes
                .get(d as usize)
                .ok_or_else(|| PyValueError::new_err(format!("bad node handle {d}")))?;
            dep.push(node);
        }

        let node = if dep.is_empty() {
            self.graph.kernel(launch, acc)
        } else {
            self.graph.kernel_after(launch, acc, dep)
        }
        .map_err(to_py)?;

        let handle = self.nodes.len() as u32;
        self.nodes.push(node);
        Ok(handle)
    }

    /// Instantiate into a replayable [`PyGraphExec`] (== `hipGraphInstantiate`).
    fn instantiate(&self) -> PyResult<PyGraphExec> {
        let exec = self.graph.instantiate().map_err(to_py)?;
        Ok(PyGraphExec { exec })
    }
}

/// An instantiated, replayable graph (== `hipGraphExec_t`).
#[pyclass(name = "GraphExec")]
struct PyGraphExec {
    exec: GraphExec,
}

#[pymethods]
impl PyGraphExec {
    /// Logical replay lanes the plan compiled to.
    #[getter]
    fn lane_count(&self) -> usize {
        self.exec.plan().lane_count().get()
    }

    /// Hex-encoded 32-byte plan fingerprint.
    fn fingerprint(&self) -> String {
        self.exec
            .plan()
            .fingerprint()
            .as_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }

    /// Validate ordering/fences by replaying once against the in-process mock
    /// backend (no GPU). Real HIP / public-AQL replay is bound at integration.
    fn launch_mock(&self) -> PyResult<()> {
        let mut backend = MockBackend::default();
        self.exec
            .launch(&mut backend, ReplayToken(0))
            .map_err(to_py)?;
        Ok(())
    }
}

#[pymodule]
fn redline_dispatch(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyGraph>()?;
    m.add_class::<PyGraphExec>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
