// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! Python bindings for Redline — lightning-fast kernel dispatch for ROCm.
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

use radiowave::{CodeObjectCertification, MutableReadCache};
use redline_core::aql::{
    DeviceBuffer as HsaDeviceBuffer, DevicePool, Executable, Gfx12Pm4CommandBuffer, GpuDevice,
    GpuSelector, KernargBuffer, KernargPool, LaunchGeometry, QueuePolicy, Runtime,
    SingleQueuePm4Ib, load_symbols,
};
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

// ---------------------------------------------------------------------------
// Real-GPU retained-PM4 replay: the fast path for an engine to drive with its
// own kernels + kernargs (the SingleQueuePm4Ib champion behind a Python surface).
// ---------------------------------------------------------------------------

/// A GPU binding (ROCr runtime + device + kernarg pool). `Gpu(ordinal)`.
#[pyclass(unsendable)]
struct Gpu {
    device: GpuDevice,
    pool: KernargPool,
    device_pool: DevicePool,
    _runtime: Runtime,
}

/// A loaded code object; kernels looked up by symbol.
#[pyclass(unsendable)]
struct Module {
    executable: Executable,
    certification: Option<CodeObjectCertification>,
}

/// A GPU-accessible buffer (for a demo counter / any device data). Its
/// `address()` goes into a kernarg; `read_u32()` reads it back after replay.
#[pyclass(unsendable)]
struct Buffer {
    buf: KernargBuffer,
}

/// Coarse-grained GPU-local storage with explicit host copies.
#[pyclass(unsendable)]
struct DeviceBuffer {
    buf: HsaDeviceBuffer,
}

/// A finalized retained PM4 indirect buffer; `replay()` submits + waits.
#[pyclass(unsendable)]
struct Pm4Ib {
    ib: SingleQueuePm4Ib,
    kernargs: Vec<KernargBuffer>,
    _modules: Vec<Executable>,
}

type DispatchSpec = (String, (u32, u32, u32), (u32, u32, u32), u32, Vec<u8>, bool);

#[pymethods]
impl Gpu {
    /// Bind ROCr GPU `ordinal` (of the `ROCR_VISIBLE_DEVICES` set).
    #[new]
    fn new(ordinal: i32) -> PyResult<Self> {
        let runtime = Runtime::initialize(load_symbols().map_err(to_py)?).map_err(to_py)?;
        let ord = usize::try_from(ordinal)
            .map_err(|_| PyValueError::new_err("negative device ordinal"))?;
        let device = runtime
            .select_gpu(GpuSelector::Ordinal(ord))
            .map_err(to_py)?;
        let pool = KernargPool::discover(&device).map_err(to_py)?;
        let device_pool = DevicePool::discover(&device).map_err(to_py)?;
        Ok(Self {
            device,
            pool,
            device_pool,
            _runtime: runtime,
        })
    }

    /// Resolve `auto`, `1`, `2`, `3`, or `4` for this GPU and an independent phase.
    /// The returned lane count never exceeds `independent_width`.
    #[pyo3(signature = (independent_width, policy="auto"))]
    fn pm4_queue_count(&self, independent_width: usize, policy: &str) -> PyResult<usize> {
        let policy = policy.parse::<QueuePolicy>().map_err(to_py)?;
        Ok(policy.resolve(self.device.name(), independent_width))
    }

    /// Load a code object. Supplying a Radiowave manifest verifies the exact
    /// bytes and enables per-consumer VMEM-only dependency boundaries. Omitting
    /// it retains the fail-closed generic same-agent boundary.
    #[pyo3(signature = (code, manifest=None))]
    fn load_module(&self, code: &[u8], manifest: Option<&str>) -> PyResult<Module> {
        let certification = manifest
            .map(|encoded| CodeObjectCertification::from_json(code, encoded))
            .transpose()
            .map_err(to_py)?;
        let bytes: std::sync::Arc<[u8]> = code.into();
        Ok(Module {
            executable: Executable::load(&self.device, bytes).map_err(to_py)?,
            certification,
        })
    }

    /// Allocate a zeroed, GPU-accessible buffer of `nbytes`.
    fn alloc(&self, nbytes: usize) -> PyResult<Buffer> {
        let mut buf = self.pool.allocate_executable_bytes(nbytes).map_err(to_py)?;
        buf.as_mut_bytes().fill(0);
        Ok(Buffer { buf })
    }

    /// Allocate zeroed coarse-grained memory in the GPU-local pool.
    fn alloc_device(&self, nbytes: usize) -> PyResult<DeviceBuffer> {
        let mut buf = self.device_pool.allocate(nbytes).map_err(to_py)?;
        let zeros = vec![0_u8; nbytes];
        // SAFETY: the newly allocated buffer has no GPU users.
        unsafe { buf.copy_from_host(&zeros) }.map_err(to_py)?;
        Ok(DeviceBuffer { buf })
    }

    /// Build a retained PM4 IB from `dispatches` — a list of
    /// `(symbol, grid, block, dynamic_group_bytes, kernarg_bytes, serialize)`
    /// tuples (grid/block in workitems). `serialize` inserts the safe RMW
    /// boundary selected from the verified next consumer.
    fn build(&self, module: &Module, dispatches: Vec<DispatchSpec>) -> PyResult<Pm4Ib> {
        // Match the certified Hipfire retained-tape policy: preserve SH-register
        // state within the IB and omit writes whose values have not changed.
        let mut cmd = Gfx12Pm4CommandBuffer::new_stateful();
        // Leading same-agent acquire: invalidate scalar/vector read caches at the
        // start of every replay, so in-place kernarg mutation (`set_kernargs`)
        // between replays is observed fresh instead of read stale from the scalar
        // cache. Required for the per-token decode update pattern.
        cmd.acquire_inter_node_gfx12();
        let mut kernargs = Vec::with_capacity(dispatches.len());
        for (i, (symbol, grid, block, dyn_group, karg_bytes, serialize)) in
            dispatches.iter().enumerate()
        {
            let kernel = module.executable.kernel(symbol).map_err(to_py)?;
            let block16 = [
                u16::try_from(block.0).map_err(|_| PyValueError::new_err("block.x > 65535"))?,
                u16::try_from(block.1).map_err(|_| PyValueError::new_err("block.y > 65535"))?,
                u16::try_from(block.2).map_err(|_| PyValueError::new_err("block.z > 65535"))?,
            ];
            let geometry = LaunchGeometry::new([grid.0, grid.1, grid.2], block16).map_err(to_py)?;
            let mut karg = self.pool.allocate_for(kernel.metadata()).map_err(to_py)?;
            {
                let dst = karg.as_mut_bytes();
                dst.fill(0);
                let n = karg_bytes.len().min(dst.len());
                dst[..n].copy_from_slice(&karg_bytes[..n]);
            }
            cmd.dispatch(&kernel, geometry, *dyn_group, karg.address())
                .map_err(to_py)?;
            kernargs.push(karg);
            if *serialize && i + 1 < dispatches.len() {
                let consumer = &dispatches[i + 1].0;
                match module
                    .certification
                    .as_ref()
                    .map_or(MutableReadCache::ScalarOrUnknown, |certification| {
                        certification.mutable_read_cache(consumer)
                    }) {
                    MutableReadCache::VmemOnly => {
                        cmd.dependency_rmw_hip_llvm_vmem_gfx12();
                    }
                    MutableReadCache::ScalarOrUnknown => {
                        cmd.dependency_rmw_same_agent_gfx12();
                    }
                }
            }
        }
        let ib = SingleQueuePm4Ib::create(&self.device, &self.pool, &cmd).map_err(to_py)?;
        Ok(Pm4Ib {
            ib,
            kernargs,
            _modules: vec![module.executable.clone()],
        })
    }
}

#[pymethods]
impl Module {
    /// The kernarg segment size (bytes) to supply for `symbol`.
    fn kernarg_size(&self, symbol: &str) -> PyResult<u32> {
        Ok(self
            .executable
            .kernel(symbol)
            .map_err(to_py)?
            .metadata()
            .kernarg_segment_size)
    }

    /// True when this module's exact bytes were verified against a Radiowave
    /// manifest containing code-object inspection evidence.
    #[getter]
    fn radiowave_certified(&self) -> bool {
        self.certification.is_some()
    }

    /// `"vmem_only"` only for a verified kernel; every other case is
    /// `"scalar_or_unknown"` and uses the fail-closed boundary.
    fn mutable_read_cache(&self, symbol: &str) -> &'static str {
        match self
            .certification
            .as_ref()
            .map_or(MutableReadCache::ScalarOrUnknown, |certification| {
                certification.mutable_read_cache(symbol)
            }) {
            MutableReadCache::VmemOnly => "vmem_only",
            MutableReadCache::ScalarOrUnknown => "scalar_or_unknown",
        }
    }

    /// Selected scheduler profile recorded by the verified manifest.
    #[getter]
    fn scheduler_profile(&self) -> Option<&str> {
        self.certification
            .as_ref()
            .map(|certification| certification.manifest().scheduler_profile.as_str())
    }

    /// Selected wavefront width recorded by the verified manifest.
    #[getter]
    fn wavefront_size(&self) -> Option<u32> {
        self.certification
            .as_ref()
            .map(|certification| certification.manifest().wavefront.width())
    }
}

#[pymethods]
impl Buffer {
    /// The device address (put its little-endian u64 into a kernarg).
    fn address(&self) -> u64 {
        self.buf.address() as usize as u64
    }

    /// Read a little-endian u32 at `offset`.
    fn read_u32(&mut self, offset: usize) -> PyResult<u32> {
        let bytes = self.buf.as_mut_bytes();
        let end = offset
            .checked_add(4)
            .filter(|e| *e <= bytes.len())
            .ok_or_else(|| PyValueError::new_err("read out of bounds"))?;
        Ok(u32::from_le_bytes(bytes[offset..end].try_into().unwrap()))
    }
}

#[pymethods]
impl DeviceBuffer {
    /// The device address (put its little-endian u64 into a kernarg).
    fn address(&self) -> u64 {
        self.buf.address() as usize as u64
    }

    /// Copy the complete allocation to the host and decode little-endian u32s.
    fn read_u32s(&self) -> PyResult<Vec<u32>> {
        if self.buf.len() % 4 != 0 {
            return Err(PyValueError::new_err(
                "device buffer length is not a multiple of four",
            ));
        }
        let mut bytes = vec![0_u8; self.buf.len()];
        // SAFETY: callers use this after retained replay completion.
        unsafe { self.buf.copy_to_host(&mut bytes) }.map_err(to_py)?;
        Ok(bytes
            .chunks_exact(4)
            .map(|word| u32::from_le_bytes(word.try_into().unwrap()))
            .collect())
    }
}

#[pymethods]
impl Pm4Ib {
    /// Submit the retained IB and wait for completion.
    fn replay(&mut self) -> PyResult<()> {
        // SAFETY: the IB owns its kernargs; device pointers in them must stay valid.
        unsafe { self.ib.replay_and_wait() }.map_err(to_py)?;
        Ok(())
    }

    /// Overwrite the retained kernarg segment of dispatch `dispatch_index` (in
    /// `Gpu.build` record order) with `data` at `byte_offset`, in place. The
    /// next `replay()` observes the new values with no IB rebuild — the
    /// per-token update path for a retained decode graph. Safe to call between
    /// replays, which wait for wave retirement.
    #[pyo3(signature = (dispatch_index, data, byte_offset=0))]
    fn set_kernargs(
        &mut self,
        dispatch_index: usize,
        data: &[u8],
        byte_offset: usize,
    ) -> PyResult<()> {
        let buffer = self
            .kernargs
            .get_mut(dispatch_index)
            .ok_or_else(|| PyValueError::new_err("dispatch_index out of range"))?;
        let end = byte_offset
            .checked_add(data.len())
            .filter(|end| *end <= buffer.len())
            .ok_or_else(|| PyValueError::new_err("kernarg write exceeds segment"))?;
        buffer.as_mut_bytes()[byte_offset..end].copy_from_slice(data);
        Ok(())
    }
}

#[pymodule]
fn redline_dispatch(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyGraph>()?;
    m.add_class::<PyGraphExec>()?;
    m.add_class::<Gpu>()?;
    m.add_class::<Module>()?;
    m.add_class::<Buffer>()?;
    m.add_class::<DeviceBuffer>()?;
    m.add_class::<Pm4Ib>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
