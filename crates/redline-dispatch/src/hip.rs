// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! HIP multi-stream implementation of [`crate::DispatchBackend`].
//!
//! This backend dynamically loads `libamdhip64` and maps each logical plan lane
//! to a nonblocking HIP stream. Dependencies within a lane rely on stream FIFO;
//! dependencies crossing lanes use timing-disabled HIP events. A separate
//! nonblocking coordinator stream waits on the terminal events, records a join
//! event, and synchronizes it before token-latency replay returns.
//!
//! # Process initialization contract
//!
//! HIP creates and caches its underlying HSA queue pool during runtime
//! initialization. Multi-stream users must set `GPU_MAX_HW_QUEUES` in the
//! process environment **before any HIP API call in the process**, then
//! construct this backend with a worker count no greater than that value.
//! [`HipMultiStreamBackend::load`] validates the existing environment before
//! loading `libamdhip64`; it deliberately never mutates process-global
//! environment state. [`HipBackendConfig::serial_fallback`] is the conservative
//! exception: it owns exactly one worker and does not require that variable.
//! The backend cannot detect whether another component initialized HIP earlier,
//! nor do multiple HIP streams prove that the runtime assigned distinct
//! hardware queues. Verify that separately with a queue-aware profiler.
//!
//! # Borrowed resources
//!
//! Kernel functions and device allocations are borrowed raw HIP handles. The
//! backend never unloads modules or frees device memory. The caller must keep a
//! registered function's module loaded and every bound allocation live until
//! the backend is dropped (or the binding/registration API eventually grows an
//! explicit removal operation) and all replays have completed. Token-latency
//! replay currently guarantees completion before returning. Throughput replay
//! is rejected rather than pretending to provide safe in-flight reuse.
//!
//! # Replay paths
//!
//! [`HipMultiStreamBackend::prepare_plan`] plus
//! [`HipMultiStreamBackend::replay_serialized_batch`] is the intended
//! measurement path: it packs arguments and lowers synchronization once, then
//! chains a batch of tokens on the GPU with one final host synchronization. The
//! [`DispatchBackend`] implementation remains a deliberately simple correctness
//! adapter. It allocates a logical signal for every dispatch and synchronizes
//! every replay, so it must not be used to characterize record/replay overhead.

use std::alloc::{Layout, alloc, dealloc, handle_alloc_error};
use std::collections::{BTreeSet, HashMap};
use std::env;
use std::ffi::{CStr, OsStr, c_char, c_int, c_uint, c_void};
use std::fmt;
use std::num::NonZeroUsize;
use std::ptr::{self, NonNull};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use libloading::Library;

use crate::batch_mem::BatchMemPlan;
use crate::ffi_batch_mem::{self, BatchMemSymbols, HIP_MEM_BATCH_FLAGS_NONE, HipMemLocation};
use crate::ffi_execution_ctx::{
    self, ExecutionCtxSymbols, HIP_DEV_RESOURCE_TYPE_SM, HipDevResource, HipDevResourceDesc,
    HipDevSmResourceGroupParams, HipExecutionCtx,
};
use crate::partition::{self, PartitionPolicy};
use crate::{
    Access, BeginReplay, CompiledPlan, DispatchBackend, DispatchRequest, EndReplay, KernelArg,
    LaneId, NodeId, ReplayBindingError, ReplayBindings, ReplayMode, ReplayToken, ResourceId,
};

type HipErrorCode = c_int;
type HipFunction = *mut c_void;
type HipStream = *mut c_void;
type HipEvent = *mut c_void;

type HipInit = unsafe extern "C" fn(c_uint) -> HipErrorCode;
type HipGetDeviceCount = unsafe extern "C" fn(*mut c_int) -> HipErrorCode;
type HipSetDevice = unsafe extern "C" fn(c_int) -> HipErrorCode;
type HipGetErrorString = unsafe extern "C" fn(HipErrorCode) -> *const c_char;
type HipStreamCreateWithFlags = unsafe extern "C" fn(*mut HipStream, c_uint) -> HipErrorCode;
type HipStreamDestroy = unsafe extern "C" fn(HipStream) -> HipErrorCode;
type HipStreamSynchronize = unsafe extern "C" fn(HipStream) -> HipErrorCode;
type HipStreamWaitEvent = unsafe extern "C" fn(HipStream, HipEvent, c_uint) -> HipErrorCode;
type HipEventCreateWithFlags = unsafe extern "C" fn(*mut HipEvent, c_uint) -> HipErrorCode;
type HipEventDestroy = unsafe extern "C" fn(HipEvent) -> HipErrorCode;
type HipEventRecord = unsafe extern "C" fn(HipEvent, HipStream) -> HipErrorCode;
type HipEventSynchronize = unsafe extern "C" fn(HipEvent) -> HipErrorCode;
type HipModuleLaunchKernel = unsafe extern "C" fn(
    HipFunction,
    c_uint,
    c_uint,
    c_uint,
    c_uint,
    c_uint,
    c_uint,
    c_uint,
    HipStream,
    *mut *mut c_void,
    *mut *mut c_void,
) -> HipErrorCode;

const HIP_SUCCESS: HipErrorCode = 0;
const HIP_STREAM_NON_BLOCKING: c_uint = 0x01;
const HIP_EVENT_DISABLE_TIMING: c_uint = 0x02;

static NEXT_HIP_BACKEND_ID: AtomicU64 = AtomicU64::new(1);

/// Queue policy selected when constructing a HIP backend.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HipBackendMode {
    /// Require a pre-existing `GPU_MAX_HW_QUEUES` value large enough for all
    /// requested worker streams.
    MultiStream,
    /// Own exactly one worker stream and leave `GPU_MAX_HW_QUEUES` entirely to
    /// the embedding process.
    SerialFallback,
}

/// Construction parameters for a HIP replay backend.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HipBackendConfig {
    device_ordinal: i32,
    worker_count: NonZeroUsize,
    mode: HipBackendMode,
    /// CU partition policy for green-context worker binding.
    ///
    /// [`PartitionPolicy::None`] keeps the legacy full-device stream path.
    /// Any other policy loads HIP 7.14 execution-context symbols and binds
    /// each worker stream to its own green context.
    partition_policy: PartitionPolicy,
}

impl HipBackendConfig {
    /// Configure the existing multi-stream path.
    ///
    /// [`HipMultiStreamBackend::load`] requires `GPU_MAX_HW_QUEUES` to have
    /// been set before HIP initialization and validates it against
    /// `worker_count`.
    pub fn new(device_ordinal: i32, worker_count: NonZeroUsize) -> Self {
        Self {
            device_ordinal,
            worker_count,
            mode: HipBackendMode::MultiStream,
            partition_policy: PartitionPolicy::None,
        }
    }

    /// Configure the conservative one-lane HIP fallback.
    ///
    /// This path neither requires, reads for validation, nor mutates
    /// `GPU_MAX_HW_QUEUES`. Its single worker stream preserves FIFO launch
    /// order and the prepared batch still joins once at the terminal boundary.
    pub fn serial_fallback(device_ordinal: i32) -> Self {
        Self {
            device_ordinal,
            worker_count: NonZeroUsize::MIN,
            mode: HipBackendMode::SerialFallback,
            partition_policy: PartitionPolicy::None,
        }
    }

    /// Bind worker streams to green-context CU partitions (ROCm >= 7.14).
    ///
    /// [`PartitionPolicy::None`] is a no-op relative to the legacy path.
    /// Non-`None` policies require a partition count that matches
    /// [`Self::worker_count`].
    pub fn partition_policy(mut self, policy: PartitionPolicy) -> Self {
        self.partition_policy = policy;
        self
    }

    pub fn device_ordinal(&self) -> i32 {
        self.device_ordinal
    }

    pub fn mode(&self) -> HipBackendMode {
        self.mode
    }

    /// Maximum number of logical worker lanes this backend accepts.
    ///
    /// In multi-stream mode this is also the minimum required numeric value of
    /// the pre-existing `GPU_MAX_HW_QUEUES` environment variable. Serial
    /// fallback always returns one and has no environment requirement. Neither
    /// mode proves that a stream receives a distinct hardware queue.
    pub fn worker_count(&self) -> NonZeroUsize {
        self.worker_count
    }

    /// CU partition policy applied at [`HipMultiStreamBackend::load`].
    pub fn get_partition_policy(&self) -> &PartitionPolicy {
        &self.partition_policy
    }
}

/// A device allocation borrowed by the backend.
#[derive(Clone, Copy, Debug)]
struct ResourceBinding {
    base: NonNull<c_void>,
    size: u64,
}

struct HipFns {
    _library: Library,
    get_error_string: HipGetErrorString,
    stream_create_with_flags: HipStreamCreateWithFlags,
    stream_destroy: HipStreamDestroy,
    stream_synchronize: HipStreamSynchronize,
    stream_wait_event: HipStreamWaitEvent,
    event_create_with_flags: HipEventCreateWithFlags,
    event_destroy: HipEventDestroy,
    event_record: HipEventRecord,
    event_synchronize: HipEventSynchronize,
    module_launch_kernel: HipModuleLaunchKernel,
}

impl HipFns {
    fn load(device_ordinal: i32) -> Result<Arc<Self>, HipBackendError> {
        let library = open_hip_library()?;

        // SAFETY: names and function signatures match hip_runtime_api.h. The
        // library is retained in HipFns for every copied function pointer.
        let (
            init,
            get_device_count,
            set_device,
            get_error_string,
            stream_create_with_flags,
            stream_destroy,
            stream_synchronize,
            stream_wait_event,
            event_create_with_flags,
            event_destroy,
            event_record,
            event_synchronize,
            module_launch_kernel,
        ) = unsafe {
            (
                load_symbol::<HipInit>(&library, b"hipInit\0")?,
                load_symbol::<HipGetDeviceCount>(&library, b"hipGetDeviceCount\0")?,
                load_symbol::<HipSetDevice>(&library, b"hipSetDevice\0")?,
                load_symbol::<HipGetErrorString>(&library, b"hipGetErrorString\0")?,
                load_symbol::<HipStreamCreateWithFlags>(&library, b"hipStreamCreateWithFlags\0")?,
                load_symbol::<HipStreamDestroy>(&library, b"hipStreamDestroy\0")?,
                load_symbol::<HipStreamSynchronize>(&library, b"hipStreamSynchronize\0")?,
                load_symbol::<HipStreamWaitEvent>(&library, b"hipStreamWaitEvent\0")?,
                load_symbol::<HipEventCreateWithFlags>(&library, b"hipEventCreateWithFlags\0")?,
                load_symbol::<HipEventDestroy>(&library, b"hipEventDestroy\0")?,
                load_symbol::<HipEventRecord>(&library, b"hipEventRecord\0")?,
                load_symbol::<HipEventSynchronize>(&library, b"hipEventSynchronize\0")?,
                load_symbol::<HipModuleLaunchKernel>(&library, b"hipModuleLaunchKernel\0")?,
            )
        };

        let fns = Arc::new(Self {
            _library: library,
            get_error_string,
            stream_create_with_flags,
            stream_destroy,
            stream_synchronize,
            stream_wait_event,
            event_create_with_flags,
            event_destroy,
            event_record,
            event_synchronize,
            module_launch_kernel,
        });

        // SAFETY: HIP accepts zero initialization flags and the count output
        // points to initialized writable memory.
        unsafe {
            fns.check("hipInit", init(0))?;
            let mut count = 0;
            fns.check("hipGetDeviceCount", get_device_count(&mut count))?;
            if device_ordinal < 0 || device_ordinal >= count {
                return Err(HipBackendError::InvalidDevice {
                    requested: device_ordinal,
                    available: count,
                });
            }
            fns.check("hipSetDevice", set_device(device_ordinal))?;
        }
        Ok(fns)
    }

    fn message(&self, code: HipErrorCode) -> String {
        // SAFETY: HIP returns a borrowed static C string for a recognized code.
        let raw = unsafe { (self.get_error_string)(code) };
        if raw.is_null() {
            return "unknown HIP error".to_owned();
        }
        // SAFETY: checked non-null; HIP owns the NUL-terminated static string.
        unsafe { CStr::from_ptr(raw) }
            .to_string_lossy()
            .into_owned()
    }

    fn check(&self, operation: &'static str, code: HipErrorCode) -> Result<(), HipBackendError> {
        if code == HIP_SUCCESS {
            Ok(())
        } else {
            Err(HipBackendError::Hip {
                operation,
                code,
                message: self.message(code),
            })
        }
    }
}

fn open_hip_library() -> Result<Library, HipBackendError> {
    const CANDIDATES: &[&str] = &[
        "libamdhip64.so",
        "libamdhip64.so.7",
        "/opt/rocm/core/lib/libamdhip64.so",
        "/opt/rocm/lib/libamdhip64.so",
    ];
    let mut failures = Vec::new();
    for candidate in CANDIDATES {
        // SAFETY: loading the installed HIP runtime is the purpose of this
        // backend. HipFns retains the successful library handle.
        match unsafe { Library::new(candidate) } {
            Ok(library) => return Ok(library),
            Err(error) => failures.push(format!("{candidate}: {error}")),
        }
    }
    Err(HipBackendError::LibraryLoad {
        candidates: CANDIDATES.join(", "),
        detail: failures.join("; "),
    })
}

unsafe fn load_symbol<T: Copy>(
    library: &Library,
    name: &'static [u8],
) -> Result<T, HipBackendError> {
    // SAFETY: each caller supplies the ABI from hip_runtime_api.h and HipFns
    // retains the library after the function pointer is copied.
    let symbol = unsafe { library.get::<T>(name) }.map_err(|error| HipBackendError::Symbol {
        symbol: std::str::from_utf8(&name[..name.len() - 1]).unwrap_or("<invalid>"),
        detail: error.to_string(),
    })?;
    Ok(*symbol)
}

struct Stream {
    fns: Arc<HipFns>,
    raw: NonNull<c_void>,
}

impl Stream {
    fn create(fns: &Arc<HipFns>) -> Result<Self, HipBackendError> {
        let mut raw = ptr::null_mut();
        // SAFETY: `raw` is a valid output location and the flag is defined by
        // hip_runtime_api.h.
        unsafe {
            fns.check(
                "hipStreamCreateWithFlags",
                (fns.stream_create_with_flags)(&mut raw, HIP_STREAM_NON_BLOCKING),
            )?;
        }
        let raw = NonNull::new(raw).ok_or(HipBackendError::NullHandle {
            operation: "hipStreamCreateWithFlags",
        })?;
        Ok(Self {
            fns: Arc::clone(fns),
            raw,
        })
    }

    /// Create a nonblocking stream bound to a green / execution context.
    fn create_from_execution_ctx(
        fns: &Arc<HipFns>,
        symbols: &ExecutionCtxSymbols,
        ctx: HipExecutionCtx,
    ) -> Result<Self, HipBackendError> {
        let mut raw = ptr::null_mut();
        // SAFETY: `raw` is a valid output location; `ctx` is a live green
        // context for this device; flags/priority match hip_runtime_api.h.
        unsafe {
            fns.check(
                "hipExecutionCtxStreamCreate",
                (symbols.execution_ctx_stream_create)(&mut raw, ctx, HIP_STREAM_NON_BLOCKING, 0),
            )?;
        }
        let raw = NonNull::new(raw).ok_or(HipBackendError::NullHandle {
            operation: "hipExecutionCtxStreamCreate",
        })?;
        Ok(Self {
            fns: Arc::clone(fns),
            raw,
        })
    }

    fn wait(&self, event: &Event) -> Result<(), HipBackendError> {
        // SAFETY: both handles are live and belong to this HIP runtime/device.
        unsafe {
            self.fns.check(
                "hipStreamWaitEvent",
                (self.fns.stream_wait_event)(self.raw.as_ptr(), event.raw.as_ptr(), 0),
            )
        }
    }

    fn synchronize(&self) -> Result<(), HipBackendError> {
        // SAFETY: the stream handle remains live for the host-synchronous call.
        unsafe {
            self.fns.check(
                "hipStreamSynchronize",
                (self.fns.stream_synchronize)(self.raw.as_ptr()),
            )
        }
    }
}

impl Drop for Stream {
    fn drop(&mut self) {
        // SAFETY: this object uniquely owns the stream returned by HIP. The
        // blocking synchronize preserves the borrowed-function/allocation
        // lifetime contract even when replay exits early with an error. Drop
        // cannot report either cleanup error.
        unsafe {
            let _ = (self.fns.stream_synchronize)(self.raw.as_ptr());
            let _ = (self.fns.stream_destroy)(self.raw.as_ptr());
        }
    }
}

/// RAII owner for a green execution context created by this backend.
///
/// Streams bound to the context must be destroyed first. Rust drops struct
/// fields in **declaration** order, so `workers` is declared before
/// `execution_contexts` on [`HipMultiStreamBackend`] (streams drop first).
/// Fallible-path locals drop in **reverse** declaration order — see
/// [`create_partitioned_workers`].
struct OwnedExecutionCtx {
    symbols: Arc<ExecutionCtxSymbols>,
    raw: HipExecutionCtx,
}

impl OwnedExecutionCtx {
    fn raw(&self) -> HipExecutionCtx {
        self.raw
    }
}

impl Drop for OwnedExecutionCtx {
    fn drop(&mut self) {
        if self.raw.0.is_null() {
            return;
        }
        // SAFETY: this object uniquely owns the context returned by
        // `hipGreenCtxCreate`. Streams that referenced it have already been
        // dropped (field order). Drop cannot report the destroy error.
        unsafe {
            let _ = (self.symbols.execution_ctx_destroy)(self.raw);
        }
        self.raw.0 = ptr::null_mut();
    }
}

struct Event {
    fns: Arc<HipFns>,
    raw: NonNull<c_void>,
}

impl Event {
    fn create(fns: &Arc<HipFns>) -> Result<Self, HipBackendError> {
        let mut raw = ptr::null_mut();
        // SAFETY: `raw` is a valid output location and the flag is defined by
        // hip_runtime_api.h.
        unsafe {
            fns.check(
                "hipEventCreateWithFlags",
                (fns.event_create_with_flags)(&mut raw, HIP_EVENT_DISABLE_TIMING),
            )?;
        }
        let raw = NonNull::new(raw).ok_or(HipBackendError::NullHandle {
            operation: "hipEventCreateWithFlags",
        })?;
        Ok(Self {
            fns: Arc::clone(fns),
            raw,
        })
    }

    fn record(&self, stream: &Stream) -> Result<(), HipBackendError> {
        // SAFETY: both handles are live and belong to this runtime/device.
        unsafe {
            self.fns.check(
                "hipEventRecord",
                (self.fns.event_record)(self.raw.as_ptr(), stream.raw.as_ptr()),
            )
        }
    }

    fn synchronize(&self) -> Result<(), HipBackendError> {
        // SAFETY: the event handle remains live for the call.
        unsafe {
            self.fns.check(
                "hipEventSynchronize",
                (self.fns.event_synchronize)(self.raw.as_ptr()),
            )
        }
    }
}

impl Drop for Event {
    fn drop(&mut self) {
        // SAFETY: this object uniquely owns the event returned by HIP. HIP
        // permits destroying a recording event and defers underlying release.
        unsafe {
            let _ = (self.fns.event_destroy)(self.raw.as_ptr());
        }
    }
}

/// Completion record returned after a token-latency replay is fully joined.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HipCompletion {
    pub token: ReplayToken,
}

/// Opaque completion event produced by one HIP dispatch.
#[derive(Clone)]
pub struct HipSignal {
    event: Rc<Event>,
    token: ReplayToken,
    lane: LaneId,
}

impl fmt::Debug for HipSignal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HipSignal")
            .field("token", &self.token)
            .field("lane", &self.lane)
            .finish_non_exhaustive()
    }
}

struct ActiveReplay {
    token: ReplayToken,
    lane_count: usize,
    events: Vec<Rc<Event>>,
    submitted: bool,
}

struct PreparedDispatch {
    lane: LaneId,
    function: NonNull<c_void>,
    grid: [u32; 3],
    block: [u32; 3],
    dynamic_shared_bytes: u32,
    arguments: PackedArguments,
    wait_slots: Vec<usize>,
    record_slot: Option<usize>,
    lane_head: bool,
}

/// Synchronization shape produced by HIP plan preparation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreparedHipPlanStats {
    pub dispatches: usize,
    /// Unique source nodes whose completion is consumed by another lane.
    pub cross_lane_signals: usize,
    /// One tail signal for every nonempty worker lane.
    pub terminal_lane_tails: usize,
    /// Unique dispatch event slots after cross-live-out/tail coalescing.
    pub dispatch_event_slots: usize,
    /// Cross-lane dependency waits emitted by one token replay.
    pub cross_lane_waits_per_token: usize,
    /// Prior-token tail waits emitted at one token boundary.
    pub token_boundary_waits_per_boundary: usize,
}

/// Fully validated and argument-packed HIP replay plan.
///
/// A prepared plan is tied to the backend that created it because it contains
/// borrowed function handles and packed device pointers. Preparing performs no
/// kernel launch. The function modules and device allocations retain the same
/// lifetime requirements as [`HipMultiStreamBackend::register_kernel`] and
/// [`HipMultiStreamBackend::bind_resource`].
pub struct PreparedHipPlan {
    backend_id: u64,
    lane_count: usize,
    dispatches: Vec<PreparedDispatch>,
    terminal_slot_by_lane: Vec<Option<usize>>,
    event_slot_count: usize,
    stats: PreparedHipPlanStats,
    /// Optional batch prefetch/discard plan executed around serialized replay.
    batch_mem_plan: Option<BatchMemPlan>,
}

impl PreparedHipPlan {
    pub fn stats(&self) -> PreparedHipPlanStats {
        self.stats
    }

    /// Attach a validated batch memory plan for prefetch-before / discard-after
    /// semantics around [`HipMultiStreamBackend::replay_serialized_batch`].
    ///
    /// An empty plan ([`BatchMemPlan::is_noop`]) is treated as a no-op.
    pub fn with_batch_mem_plan(mut self, plan: BatchMemPlan) -> Self {
        self.batch_mem_plan = Some(plan);
        self
    }

    /// Optional batch-mem plan attached via [`Self::with_batch_mem_plan`].
    pub fn batch_mem_plan(&self) -> Option<&BatchMemPlan> {
        self.batch_mem_plan.as_ref()
    }

    /// Predict the command counts for a serialized GPU batch.
    ///
    /// The successful path always performs one host synchronization on the
    /// coordinator join event. A non-empty discard plan adds one host
    /// synchronization per used worker lane after discard is enqueued (each
    /// lane that may hold discard work is drained). Error recovery may perform
    /// extra stream drains and is intentionally excluded.
    pub fn batch_stats(&self, token_count: NonZeroUsize) -> Result<HipBatchStats, HipBackendError> {
        // Discard is drained on every plan-used lane after enqueue.
        let extra_host_syncs = if self
            .batch_mem_plan
            .as_ref()
            .is_some_and(|plan| plan.has_discard())
        {
            self.terminal_slot_by_lane
                .iter()
                .filter(|slot| slot.is_some())
                .count()
                .max(1)
        } else {
            0
        };
        calculate_batch_stats(
            self.stats,
            self.event_slot_count,
            token_count,
            extra_host_syncs,
        )
    }
}

/// Expected or completed command counts for a prepared HIP batch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HipBatchStats {
    pub token_count: usize,
    pub kernel_launches: usize,
    pub dispatch_event_records: usize,
    pub dependency_waits: usize,
    pub token_boundary_waits: usize,
    pub coordinator_waits: usize,
    pub coordinator_join_records: usize,
    pub host_synchronizations: usize,
}

/// Completion returned after the final coordinator event has completed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HipBatchCompletion {
    pub first_token: ReplayToken,
    pub last_token: ReplayToken,
    pub stats: HipBatchStats,
}

/// Real HIP backend that submits plan lanes to nonblocking streams.
///
/// The backend supports [`ReplayMode::TokenLatency`] only. It owns streams and
/// events, but borrows registered `hipFunction_t` handles and device pointers.
/// When a non-`None` partition policy is configured it also owns per-worker
/// green execution contexts (destroyed after their streams).
pub struct HipMultiStreamBackend {
    backend_id: u64,
    mode: HipBackendMode,
    fns: Arc<HipFns>,
    /// Lazily loaded when a non-empty batch-mem plan is first executed.
    batch_mem_symbols: Option<Arc<BatchMemSymbols>>,
    /// Device ordinal selected at load (needed for prefetch location).
    device_ordinal: i32,
    coordinator: Stream,
    // DROP ORDER (struct fields): Rust drops fields in *declaration* order.
    // Streams must die before their green contexts, so `workers` is declared
    // before `execution_contexts`. Do not "fix" this back.
    /// Worker streams; destroyed before green contexts (declaration drop order).
    workers: Vec<Stream>,
    /// Green contexts; must outlive their streams and drop only after them.
    execution_contexts: Vec<OwnedExecutionCtx>,
    kernels: HashMap<String, NonNull<c_void>>,
    resources: HashMap<ResourceId, ResourceBinding>,
    available_events: Vec<Rc<Event>>,
    active: Option<ActiveReplay>,
    batch_active: Option<ReplayToken>,
    poisoned: Option<String>,
    configured_hw_queues: usize,
}

impl HipMultiStreamBackend {
    /// Load HIP and create the coordinator and worker streams.
    ///
    /// Multi-stream configurations require `GPU_MAX_HW_QUEUES` to already
    /// exist, parse as a positive integer, and be at least
    /// `config.worker_count()`. A serial-fallback configuration owns exactly
    /// one worker and does not require or mutate that environment variable.
    /// The caller must arrange any multi-stream setting before HIP
    /// initialization. This function never calls `std::env::set_var`.
    ///
    /// When `config.partition_policy` is not [`PartitionPolicy::None`], the
    /// backend loads HIP 7.14 execution-context symbols, validates the
    /// partition against the device SM/CU resource, creates one green context
    /// per worker, and binds each worker stream with
    /// `hipExecutionCtxStreamCreate`. Missing 7.14 symbols hard-fail.
    pub fn load(config: HipBackendConfig) -> Result<Self, HipBackendError> {
        let configured_hw_queues = match config.mode() {
            HipBackendMode::MultiStream => {
                validate_backend_config(&config, env::var_os("GPU_MAX_HW_QUEUES").as_deref())?
            }
            HipBackendMode::SerialFallback => validate_backend_config(&config, None)?,
        };
        let backend_id = NEXT_HIP_BACKEND_ID.fetch_add(1, Ordering::Relaxed);
        assert!(
            backend_id != u64::MAX,
            "HIP backend identity space exhausted"
        );
        let device_ordinal = config.device_ordinal();
        let fns = HipFns::load(device_ordinal)?;
        let coordinator = Stream::create(&fns)?;

        let (workers, execution_contexts) =
            if matches!(config.get_partition_policy(), PartitionPolicy::None) {
                // Byte-identical legacy path: plain nonblocking worker streams.
                let workers = (0..config.worker_count().get())
                    .map(|_| Stream::create(&fns))
                    .collect::<Result<Vec<_>, _>>()?;
                (workers, Vec::new())
            } else {
                create_partitioned_workers(&fns, device_ordinal, &config)?
            };

        Ok(Self {
            backend_id,
            mode: config.mode(),
            fns,
            batch_mem_symbols: None,
            device_ordinal,
            coordinator,
            workers,
            execution_contexts,
            kernels: HashMap::new(),
            resources: HashMap::new(),
            available_events: Vec::new(),
            active: None,
            batch_active: None,
            poisoned: None,
            configured_hw_queues,
        })
    }

    pub fn worker_count(&self) -> usize {
        self.workers.len()
    }

    pub fn mode(&self) -> HipBackendMode {
        self.mode
    }

    /// Number of green execution contexts owned by this backend.
    ///
    /// Zero when `partition_policy` was [`PartitionPolicy::None`] at load.
    pub fn green_context_count(&self) -> usize {
        self.execution_contexts.len()
    }

    /// Effective hardware-queue limit used for validation.
    ///
    /// Serial fallback reports one without consulting `GPU_MAX_HW_QUEUES`.
    pub fn configured_hw_queue_limit(&self) -> usize {
        self.configured_hw_queues
    }

    /// Reason this backend rejected further submissions after a failed recovery
    /// drain, if any.
    pub fn poisoned_reason(&self) -> Option<&str> {
        self.poisoned.as_deref()
    }

    /// Validate and lower host-resolvable plan state without submitting GPU work.
    ///
    /// Preparation resolves every kernel key, validates every logical resource
    /// against its physical binding, packs all kernel arguments, and coalesces
    /// cross-lane synchronization to lane frontiers. It cannot validate device
    /// launch limits or the borrowed function's actual ABI. HIP may report such
    /// an error during batch submission after earlier dispatches were already
    /// queued or executed; that failure is therefore partially side-effecting
    /// even though the backend subsequently attempts to drain all owned streams.
    pub fn prepare_plan(&self, plan: &CompiledPlan) -> Result<PreparedHipPlan, HipBackendError> {
        self.prepare_plan_internal(plan, None)
    }

    /// Prepare a plan whose dynamic scalar slots come from shared replay
    /// bindings.
    ///
    /// Resource arguments continue to use this backend's validated borrowed
    /// bindings. Scalar values are copied into the prepared argument storage,
    /// so later mutation of `bindings` does not alter an existing plan.
    pub fn prepare_plan_with_bindings(
        &self,
        plan: &CompiledPlan,
        bindings: &ReplayBindings,
    ) -> Result<PreparedHipPlan, HipBackendError> {
        self.prepare_plan_internal(plan, Some(bindings))
    }

    fn prepare_plan_internal(
        &self,
        plan: &CompiledPlan,
        bindings: Option<&ReplayBindings>,
    ) -> Result<PreparedHipPlan, HipBackendError> {
        self.require_inactive()?;
        if plan.replay_mode() != ReplayMode::TokenLatency {
            return Err(HipBackendError::UnsupportedReplayMode(plan.replay_mode()));
        }
        if plan.lane_count().get() > self.workers.len() {
            return Err(HipBackendError::TooManyLanes {
                requested: plan.lane_count().get(),
                available: self.workers.len(),
            });
        }
        self.validate_plan_bindings(plan)?;
        let lowered = lower_plan(plan)?;
        let mut dispatches = Vec::with_capacity(plan.dispatches().len());
        for (index, dispatch) in plan.dispatches().iter().enumerate() {
            self.validate_accesses(dispatch.accesses())?;
            let function = self
                .kernels
                .get(dispatch.launch().kernel())
                .copied()
                .ok_or_else(|| HipBackendError::KernelNotRegistered {
                    key: dispatch.launch().kernel().to_owned(),
                })?;
            let arguments = PackedArguments::new_with_bindings(
                dispatch.launch().arguments(),
                &self.resources,
                bindings,
            )?;
            let grid = dispatch.launch().grid();
            let block = dispatch.launch().block();
            dispatches.push(PreparedDispatch {
                lane: dispatch.lane(),
                function,
                grid: [grid.x, grid.y, grid.z],
                block: [block.x, block.y, block.z],
                dynamic_shared_bytes: dispatch.launch().dynamic_shared_bytes(),
                arguments,
                wait_slots: lowered.wait_slots_by_dispatch[index].clone(),
                record_slot: lowered.record_slot_by_dispatch[index],
                lane_head: lowered.lane_head_by_dispatch[index],
            });
        }
        Ok(PreparedHipPlan {
            backend_id: self.backend_id,
            lane_count: plan.lane_count().get(),
            dispatches,
            terminal_slot_by_lane: lowered.terminal_slot_by_lane,
            event_slot_count: lowered.event_slot_count,
            stats: lowered.stats,
            batch_mem_plan: None,
        })
    }

    /// Submit `token_count` copies of a prepared plan and synchronize once.
    ///
    /// Each token owns a separate event bank. At token boundaries, the first
    /// dispatch on each nonempty lane waits on the previous token's other-lane
    /// tails; its own prior tail is ordered by stream FIFO. The host waits only
    /// on the final coordinator join event. This serializes tokens without a
    /// host round-trip between them while preserving intra-token lane overlap.
    ///
    /// When a non-empty [`BatchMemPlan`] is attached via
    /// [`PreparedHipPlan::with_batch_mem_plan`], fixture ranges are prefetched
    /// only on lanes the plan actually uses, and the coordinator waits every
    /// enqueued prefetch event before the first dispatch. Discard is enqueued
    /// after the final coordinator join wait and drained on **every** used
    /// worker lane (plus any discard-only lane that carried a discard enqueue).
    pub fn replay_serialized_batch(
        &mut self,
        prepared: &mut PreparedHipPlan,
        first_token: ReplayToken,
        token_count: NonZeroUsize,
    ) -> Result<HipBatchCompletion, HipBackendError> {
        self.require_inactive()?;
        if prepared.backend_id != self.backend_id {
            return Err(HipBackendError::PreparedPlanBackendMismatch);
        }
        if prepared.lane_count > self.workers.len() {
            return Err(HipBackendError::TooManyLanes {
                requested: prepared.lane_count,
                available: self.workers.len(),
            });
        }
        let stats = prepared.batch_stats(token_count)?;
        let last_offset = u64::try_from(token_count.get() - 1)
            .map_err(|_| HipBackendError::ReplayTokenRangeOverflow)?;
        let last_token = ReplayToken(
            first_token
                .0
                .checked_add(last_offset)
                .ok_or(HipBackendError::ReplayTokenRangeOverflow)?,
        );
        let dispatch_events = checked_product(prepared.event_slot_count, token_count.get())?;
        let total_events = dispatch_events
            .checked_add(1)
            .ok_or(HipBackendError::BatchSizeOverflow)?;
        let mut events = self.acquire_events(total_events)?;
        let joined = events.pop().expect("one coordinator event was requested");

        // Clone the optional plan so the submit closure can mutably borrow
        // `prepared.dispatches` without overlapping the plan borrow.
        let batch_mem = prepared
            .batch_mem_plan
            .as_ref()
            .filter(|plan| !plan.is_noop())
            .cloned();
        if batch_mem.is_some() {
            self.ensure_batch_mem_symbols()?;
        }

        self.batch_active = Some(first_token);
        let mut submitted = false;
        let submit_result = (|| {
            if let Some(plan) = batch_mem.as_ref() {
                submitted = true;
                self.enqueue_prefetch_batch(plan, &prepared.terminal_slot_by_lane)?;
            }

            for token_index in 0..token_count.get() {
                let current_bank = token_index * prepared.event_slot_count;
                for dispatch in &mut prepared.dispatches {
                    let lane = dispatch.lane.0;
                    let stream = &self.workers[lane];

                    if token_index != 0 && dispatch.lane_head {
                        let prior_bank = (token_index - 1) * prepared.event_slot_count;
                        for (source_lane, terminal_slot) in
                            prepared.terminal_slot_by_lane.iter().enumerate()
                        {
                            if source_lane == lane {
                                continue;
                            }
                            if let Some(slot) = terminal_slot {
                                submitted = true;
                                stream.wait(events[prior_bank + slot].as_ref())?;
                            }
                        }
                    }

                    for slot in &dispatch.wait_slots {
                        submitted = true;
                        stream.wait(events[current_bank + slot].as_ref())?;
                    }

                    // SAFETY: prepare_plan resolved the borrowed registered
                    // handle, packed every ABI argument into stable aligned
                    // storage, and tied this plan to the creating backend. The
                    // caller's unsafe registration contract supplies validity.
                    submitted = true;
                    unsafe {
                        self.fns.check(
                            "hipModuleLaunchKernel",
                            (self.fns.module_launch_kernel)(
                                dispatch.function.as_ptr(),
                                dispatch.grid[0],
                                dispatch.grid[1],
                                dispatch.grid[2],
                                dispatch.block[0],
                                dispatch.block[1],
                                dispatch.block[2],
                                dispatch.dynamic_shared_bytes,
                                stream.raw.as_ptr(),
                                dispatch.arguments.params_mut_ptr(),
                                ptr::null_mut(),
                            ),
                        )?;
                    }
                    if let Some(slot) = dispatch.record_slot {
                        submitted = true;
                        events[current_bank + slot].record(stream)?;
                    }
                }
            }

            let final_bank = (token_count.get() - 1) * prepared.event_slot_count;
            for slot in prepared.terminal_slot_by_lane.iter().flatten() {
                submitted = true;
                self.coordinator.wait(events[final_bank + slot].as_ref())?;
            }
            submitted = true;
            joined.record(&self.coordinator)?;
            joined.synchronize()?;

            if let Some(plan) = batch_mem.as_ref() {
                self.enqueue_discard_batch(plan, &prepared.terminal_slot_by_lane)?;
            }
            Ok(())
        })();

        self.batch_active = None;
        match submit_result {
            Ok(()) => {
                self.available_events.extend(events);
                self.available_events.push(joined);
                Ok(HipBatchCompletion {
                    first_token,
                    last_token,
                    stats,
                })
            }
            Err(cause) => Err(self.recover_batch_failure(cause, submitted, events, joined)),
        }
    }

    fn ensure_batch_mem_symbols(&mut self) -> Result<(), HipBackendError> {
        if self.batch_mem_symbols.is_some() {
            return Ok(());
        }
        let symbols = ffi_batch_mem::load_batch_mem_symbols().map_err(map_batch_mem_load_error)?;
        self.batch_mem_symbols = Some(symbols);
        Ok(())
    }

    fn plan_used_lanes(&self, terminal_slot_by_lane: &[Option<usize>]) -> Vec<usize> {
        terminal_slot_by_lane
            .iter()
            .enumerate()
            .filter_map(|(lane, slot)| slot.map(|_| lane))
            .filter(|&lane| lane < self.workers.len())
            .collect()
    }

    /// Prefetch only on plan-used lanes; coordinator waits every prefetch event
    /// before dispatch begins so unused-backend workers stay idle and used
    /// lanes cannot race ahead of migration.
    fn enqueue_prefetch_batch(
        &self,
        plan: &BatchMemPlan,
        terminal_slot_by_lane: &[Option<usize>],
    ) -> Result<(), HipBackendError> {
        if plan.prefetch().is_empty() {
            return Ok(());
        }
        let symbols = self
            .batch_mem_symbols
            .as_ref()
            .expect("batch-mem symbols loaded before enqueue");
        let lanes = self.plan_used_lanes(terminal_slot_by_lane);
        if lanes.is_empty() {
            return Ok(());
        }
        let mut ptrs: Vec<*mut c_void> = plan
            .prefetch()
            .iter()
            .map(|range| range.device_ptr() as usize as *mut c_void)
            .collect();
        let mut sizes: Vec<usize> = plan.prefetch().iter().map(|range| range.size()).collect();
        let mut locs = vec![HipMemLocation::device(self.device_ordinal)];
        let mut loc_idxs = vec![0usize; plan.prefetch().len()];
        let mut prefetch_events = Vec::with_capacity(lanes.len());
        for &lane in &lanes {
            let worker = &self.workers[lane];
            // SAFETY: pointers/sizes describe the caller-validated fixture ranges;
            // stream is a live plan-used worker; flags are required zero.
            unsafe {
                self.fns.check(
                    "hipMemPrefetchBatchAsync",
                    (symbols.mem_prefetch_batch_async)(
                        ptrs.as_mut_ptr(),
                        sizes.as_mut_ptr(),
                        ptrs.len(),
                        locs.as_mut_ptr(),
                        loc_idxs.as_mut_ptr(),
                        locs.len(),
                        HIP_MEM_BATCH_FLAGS_NONE,
                        worker.raw.as_ptr(),
                    ),
                )?;
            }
            let event = Event::create(&self.fns)?;
            event.record(worker)?;
            prefetch_events.push(event);
        }
        // Coordinator gates dispatch start on every enqueued prefetch.
        for event in &prefetch_events {
            self.coordinator.wait(event)?;
        }
        // Make the coordinator fence visible to each used worker before kernels.
        let gate = Event::create(&self.fns)?;
        gate.record(&self.coordinator)?;
        for &lane in &lanes {
            self.workers[lane].wait(&gate)?;
        }
        Ok(())
    }

    /// Enqueue discard after the joined fence, then host-drain **every**
    /// plan-used lane so discard cannot race with backend teardown/reuse.
    fn enqueue_discard_batch(
        &self,
        plan: &BatchMemPlan,
        terminal_slot_by_lane: &[Option<usize>],
    ) -> Result<(), HipBackendError> {
        if plan.discard().is_empty() {
            return Ok(());
        }
        let symbols = self
            .batch_mem_symbols
            .as_ref()
            .expect("batch-mem symbols loaded before enqueue");
        let mut lanes = self.plan_used_lanes(terminal_slot_by_lane);
        if lanes.is_empty() {
            // Degenerate empty plan lane set: fall back to worker 0 only.
            lanes.push(0);
        }
        let mut ptrs: Vec<*mut c_void> = plan
            .discard()
            .iter()
            .map(|range| range.device_ptr() as usize as *mut c_void)
            .collect();
        let mut sizes: Vec<usize> = plan.discard().iter().map(|range| range.size()).collect();
        // One discard enqueue is enough (ranges are global); place it on the
        // first used lane. Completion is still waited on every used lane.
        let stream = self.workers[lanes[0]].raw.as_ptr();
        // SAFETY: pointers/sizes describe the caller-validated fixture ranges;
        // stream is a live worker; flags are required zero. Called only after
        // joined.synchronize so prior worker work has completed.
        unsafe {
            self.fns.check(
                "hipMemDiscardBatchAsync",
                (symbols.mem_discard_batch_async)(
                    ptrs.as_mut_ptr(),
                    sizes.as_mut_ptr(),
                    ptrs.len(),
                    HIP_MEM_BATCH_FLAGS_NONE,
                    stream,
                ),
            )?;
        }
        // Finding: discard waits all (used) lanes' completion, not only lane 0.
        for &lane in &lanes {
            self.workers[lane].synchronize()?;
        }
        Ok(())
    }

    /// Register a borrowed `hipFunction_t` under a plan kernel key.
    ///
    /// # Safety
    ///
    /// `function` must be a non-null `hipFunction_t` for the selected device.
    /// Its owning HIP module must remain loaded until this backend is dropped
    /// and all replay work has completed. The function's ABI must exactly match
    /// the [`KernelArg`] sequence recorded for every launch using `key`.
    pub unsafe fn register_kernel(
        &mut self,
        key: impl Into<String>,
        function: *mut c_void,
    ) -> Result<(), HipBackendError> {
        self.require_inactive()?;
        let key = key.into();
        if key.trim().is_empty() {
            return Err(HipBackendError::EmptyKernelKey);
        }
        let function = NonNull::new(function).ok_or(HipBackendError::NullKernelFunction)?;
        if self.kernels.contains_key(&key) {
            return Err(HipBackendError::KernelAlreadyRegistered(key));
        }
        self.kernels.insert(key, function);
        Ok(())
    }

    /// Bind one logical plan resource to a borrowed device allocation.
    ///
    /// # Safety
    ///
    /// `device_base` must be a non-null HIP device pointer valid on the selected
    /// device, and `size` must truthfully describe its accessible byte extent.
    /// The allocation must remain live, and distinct logical resources must not
    /// alias, until this backend is dropped and all replay work has completed.
    pub unsafe fn bind_resource(
        &mut self,
        resource: ResourceId,
        device_base: *mut c_void,
        size: u64,
    ) -> Result<(), HipBackendError> {
        self.require_inactive()?;
        let base = NonNull::new(device_base).ok_or(HipBackendError::NullDevicePointer(resource))?;
        if size == 0 {
            return Err(HipBackendError::EmptyDeviceBinding(resource));
        }
        if self.resources.contains_key(&resource) {
            return Err(HipBackendError::ResourceAlreadyBound(resource));
        }
        let size_as_usize = usize::try_from(size)
            .map_err(|_| HipBackendError::DeviceBindingAddressOverflow { resource, size })?;
        let start = base.as_ptr() as usize;
        let end = start
            .checked_add(size_as_usize)
            .ok_or(HipBackendError::DeviceBindingAddressOverflow { resource, size })?;
        for (other_resource, other) in &self.resources {
            let other_start = other.base.as_ptr() as usize;
            let other_size = usize::try_from(other.size).map_err(|_| {
                HipBackendError::DeviceBindingAddressOverflow {
                    resource: *other_resource,
                    size: other.size,
                }
            })?;
            let other_end = other_start.checked_add(other_size).ok_or(
                HipBackendError::DeviceBindingAddressOverflow {
                    resource: *other_resource,
                    size: other.size,
                },
            )?;
            if start < other_end && other_start < end {
                return Err(HipBackendError::AliasingResourceBinding {
                    resource,
                    other: *other_resource,
                });
            }
        }
        self.resources
            .insert(resource, ResourceBinding { base, size });
        Ok(())
    }

    /// Check that every resource declared by `plan` has a sufficiently large
    /// physical binding before replay begins.
    pub fn validate_plan_bindings(&self, plan: &CompiledPlan) -> Result<(), HipBackendError> {
        validate_bindings(&self.resources, plan)
    }

    fn require_inactive(&self) -> Result<(), HipBackendError> {
        if let Some(detail) = &self.poisoned {
            return Err(HipBackendError::BackendPoisoned {
                detail: detail.clone(),
            });
        }
        if let Some(active) = &self.active {
            Err(HipBackendError::ReplayAlreadyActive(active.token))
        } else if let Some(token) = self.batch_active {
            Err(HipBackendError::ReplayAlreadyActive(token))
        } else {
            Ok(())
        }
    }

    fn acquire_events(&mut self, count: usize) -> Result<Vec<Rc<Event>>, HipBackendError> {
        let mut acquired = Vec::with_capacity(count);
        for _ in 0..count {
            let event = match self.available_events.pop() {
                Some(event) => event,
                None => match Event::create(&self.fns) {
                    Ok(event) => Rc::new(event),
                    Err(error) => {
                        self.available_events.extend(acquired);
                        return Err(error);
                    }
                },
            };
            acquired.push(event);
        }
        Ok(acquired)
    }

    fn drain_all_streams(&self) -> Result<(), String> {
        let mut failures = Vec::new();
        for (lane, worker) in self.workers.iter().enumerate() {
            if let Err(error) = worker.synchronize() {
                failures.push(format!("worker lane {lane}: {error}"));
            }
        }
        if let Err(error) = self.coordinator.synchronize() {
            failures.push(format!("coordinator: {error}"));
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures.join("; "))
        }
    }

    fn recover_batch_failure(
        &mut self,
        cause: HipBackendError,
        submitted: bool,
        events: Vec<Rc<Event>>,
        joined: Rc<Event>,
    ) -> HipBackendError {
        if !submitted {
            discard_failed_event_generation(events, joined);
            return cause;
        }
        let drain_result = self.drain_all_streams();
        // A failed record/wait/launch can leave an event generation in a state
        // that is not safe to re-record. Keep the handles alive through the
        // recovery drain, then destroy rather than pooling them.
        discard_failed_event_generation(events, joined);
        match drain_result {
            Ok(()) => cause,
            Err(drain_error) => {
                let cause = cause.to_string();
                self.poisoned = Some(format!(
                    "batch submission failed ({cause}); recovery drain failed ({drain_error})"
                ));
                HipBackendError::BatchSubmissionPoisoned { cause, drain_error }
            }
        }
    }

    fn recover_adapter_failure(&mut self, cause: HipBackendError) -> HipBackendError {
        let Some(active) = self.active.take() else {
            return cause;
        };
        let drain_result = active.submitted.then(|| self.drain_all_streams());
        // Signals from a failed adapter replay may denote a failed event record.
        // Destroy them after the drain rather than returning them to the pool.
        drop(active);
        match drain_result {
            None | Some(Ok(())) => cause,
            Some(Err(drain_error)) => {
                let cause = cause.to_string();
                self.poisoned = Some(format!(
                    "adapter submission failed ({cause}); recovery drain failed ({drain_error})"
                ));
                HipBackendError::AdapterSubmissionPoisoned { cause, drain_error }
            }
        }
    }

    fn validate_accesses(&self, accesses: &[Access]) -> Result<(), HipBackendError> {
        for access in accesses {
            let region = access.region();
            let Some(binding) = self.resources.get(&region.resource()) else {
                return Err(HipBackendError::ResourceNotBound(region.resource()));
            };
            if region.end() > binding.size {
                return Err(HipBackendError::BindingTooSmall {
                    resource: region.resource(),
                    required: region.end(),
                    bound: binding.size,
                });
            }
        }
        Ok(())
    }
}

fn discard_failed_event_generation<T>(events: Vec<T>, joined: T) {
    drop(events);
    drop(joined);
}

struct LoweredPlan {
    wait_slots_by_dispatch: Vec<Vec<usize>>,
    record_slot_by_dispatch: Vec<Option<usize>>,
    lane_head_by_dispatch: Vec<bool>,
    terminal_slot_by_lane: Vec<Option<usize>>,
    event_slot_count: usize,
    stats: PreparedHipPlanStats,
}

fn lower_plan(plan: &CompiledPlan) -> Result<LoweredPlan, HipBackendError> {
    let dispatch_count = plan.dispatches().len();
    let lane_count = plan.lane_count().get();
    let mut dispatch_index_by_node = vec![None; dispatch_count];
    let mut lane_by_node = vec![None; dispatch_count];
    let mut lane_position_by_node = vec![None; dispatch_count];
    let mut next_lane_position = vec![0_usize; lane_count];
    let mut first_dispatch_by_lane = vec![None; lane_count];
    let mut last_node_by_lane = vec![None; lane_count];

    for (dispatch_index, dispatch) in plan.dispatches().iter().enumerate() {
        let node_index = dispatch.node().index() as usize;
        let lane = dispatch.lane().0;
        if node_index >= dispatch_count || lane >= lane_count {
            return Err(HipBackendError::InvalidCompiledPlan {
                detail: format!(
                    "node {} on lane {lane} exceeds plan dimensions ({dispatch_count}, {lane_count})",
                    dispatch.node().index()
                ),
            });
        }
        if dispatch_index_by_node[node_index]
            .replace(dispatch_index)
            .is_some()
        {
            return Err(HipBackendError::InvalidCompiledPlan {
                detail: format!("duplicate node index {}", dispatch.node().index()),
            });
        }
        let lane_position = next_lane_position[lane];
        next_lane_position[lane] += 1;
        lane_by_node[node_index] = Some(lane);
        lane_position_by_node[node_index] = Some(lane_position);
        first_dispatch_by_lane[lane].get_or_insert(dispatch_index);
        last_node_by_lane[lane] = Some(dispatch.node());
    }

    let mut completion_frontiers: Vec<Option<Vec<Option<usize>>>> = vec![None; dispatch_count];
    let mut lane_frontiers = vec![vec![None; lane_count]; lane_count];
    let mut wait_nodes_by_dispatch = vec![Vec::new(); dispatch_count];
    let mut cross_signal_nodes = BTreeSet::new();

    for (dispatch_index, dispatch) in plan.dispatches().iter().enumerate() {
        let node_index = dispatch.node().index() as usize;
        let lane = lane_by_node[node_index].expect("validated node lane above");
        let mut known = lane_frontiers[lane].clone();
        let mut latest_dependency_by_lane = vec![None; lane_count];

        for dependency in dispatch.dependencies() {
            let dependency_index = dependency.index() as usize;
            let source_lane = lane_by_node
                .get(dependency_index)
                .and_then(|lane| *lane)
                .ok_or_else(|| HipBackendError::InvalidCompiledPlan {
                    detail: format!("unknown dependency node {}", dependency.index()),
                })?;
            if source_lane == lane {
                continue;
            }
            let dependency_position = lane_position_by_node[dependency_index]
                .expect("validated dependency position above");
            let replace = latest_dependency_by_lane[source_lane]
                .map(|current: NodeId| {
                    let current_position = lane_position_by_node[current.index() as usize]
                        .expect("validated dependency position above");
                    dependency_position > current_position
                })
                .unwrap_or(true);
            if replace {
                latest_dependency_by_lane[source_lane] = Some(*dependency);
            }
        }

        for (source_lane, dependency) in latest_dependency_by_lane.into_iter().enumerate() {
            let Some(dependency) = dependency else {
                continue;
            };
            let dependency_index = dependency.index() as usize;
            let dependency_position = lane_position_by_node[dependency_index]
                .expect("validated dependency position above");
            if known[source_lane]
                .map(|position| position < dependency_position)
                .unwrap_or(true)
            {
                wait_nodes_by_dispatch[dispatch_index].push(dependency);
                cross_signal_nodes.insert(dependency);
            }
            let dependency_frontiers =
                completion_frontiers[dependency_index]
                    .as_ref()
                    .ok_or_else(|| HipBackendError::InvalidCompiledPlan {
                        detail: format!(
                            "dependency node {} appears after its consumer",
                            dependency.index()
                        ),
                    })?;
            for (known_position, dependency_position) in known.iter_mut().zip(dependency_frontiers)
            {
                if dependency_position > known_position {
                    *known_position = *dependency_position;
                }
            }
        }

        known[lane] = lane_position_by_node[node_index];
        lane_frontiers[lane] = known.clone();
        completion_frontiers[node_index] = Some(known);
    }

    let terminal_nodes = last_node_by_lane
        .iter()
        .flatten()
        .copied()
        .collect::<BTreeSet<_>>();
    let mut event_nodes = cross_signal_nodes.clone();
    event_nodes.extend(&terminal_nodes);
    let mut slot_by_node = vec![None; dispatch_count];
    let mut event_slot_count = 0;
    for dispatch in plan.dispatches() {
        if event_nodes.contains(&dispatch.node()) {
            slot_by_node[dispatch.node().index() as usize] = Some(event_slot_count);
            event_slot_count += 1;
        }
    }

    let wait_slots_by_dispatch = wait_nodes_by_dispatch
        .into_iter()
        .map(|wait_nodes| {
            wait_nodes
                .into_iter()
                .map(|node| {
                    slot_by_node[node.index() as usize]
                        .expect("every cross-lane wait source received an event slot")
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let record_slot_by_dispatch = plan
        .dispatches()
        .iter()
        .map(|dispatch| slot_by_node[dispatch.node().index() as usize])
        .collect::<Vec<_>>();
    let lane_head_by_dispatch = (0..dispatch_count)
        .map(|dispatch_index| first_dispatch_by_lane.contains(&Some(dispatch_index)))
        .collect::<Vec<_>>();
    let terminal_slot_by_lane = last_node_by_lane
        .iter()
        .map(|node| node.and_then(|node| slot_by_node[node.index() as usize]))
        .collect::<Vec<_>>();
    let cross_lane_waits_per_token = wait_slots_by_dispatch
        .iter()
        .try_fold(0_usize, |total, waits| total.checked_add(waits.len()))
        .ok_or(HipBackendError::BatchSizeOverflow)?;
    let nonempty_lanes = terminal_nodes.len();
    let token_boundary_waits_per_boundary = nonempty_lanes
        .checked_mul(nonempty_lanes.saturating_sub(1))
        .ok_or(HipBackendError::BatchSizeOverflow)?;

    Ok(LoweredPlan {
        wait_slots_by_dispatch,
        record_slot_by_dispatch,
        lane_head_by_dispatch,
        terminal_slot_by_lane,
        event_slot_count,
        stats: PreparedHipPlanStats {
            dispatches: dispatch_count,
            cross_lane_signals: cross_signal_nodes.len(),
            terminal_lane_tails: terminal_nodes.len(),
            dispatch_event_slots: event_slot_count,
            cross_lane_waits_per_token,
            token_boundary_waits_per_boundary,
        },
    })
}

fn checked_product(left: usize, right: usize) -> Result<usize, HipBackendError> {
    left.checked_mul(right)
        .ok_or(HipBackendError::BatchSizeOverflow)
}

fn calculate_batch_stats(
    plan: PreparedHipPlanStats,
    event_slot_count: usize,
    token_count: NonZeroUsize,
    extra_host_syncs: usize,
) -> Result<HipBatchStats, HipBackendError> {
    let count = token_count.get();
    let boundaries = count - 1;
    // Base: joined.synchronize(). Extra: one synchronize per used lane after discard.
    let host_synchronizations = 1 + extra_host_syncs;
    Ok(HipBatchStats {
        token_count: count,
        kernel_launches: checked_product(plan.dispatches, count)?,
        dispatch_event_records: checked_product(event_slot_count, count)?,
        dependency_waits: checked_product(plan.cross_lane_waits_per_token, count)?,
        token_boundary_waits: checked_product(plan.token_boundary_waits_per_boundary, boundaries)?,
        coordinator_waits: plan.terminal_lane_tails,
        coordinator_join_records: 1,
        host_synchronizations,
    })
}

impl DispatchBackend for HipMultiStreamBackend {
    type Signal = HipSignal;
    type Completion = HipCompletion;
    type Error = HipBackendError;

    fn begin_replay(&mut self, replay: BeginReplay) -> Result<(), Self::Error> {
        self.require_inactive()?;
        if let ReplayMode::Throughput { .. } = replay.mode {
            return Err(HipBackendError::UnsupportedReplayMode(replay.mode));
        }
        if replay.lane_count > self.workers.len() {
            return Err(HipBackendError::TooManyLanes {
                requested: replay.lane_count,
                available: self.workers.len(),
            });
        }
        self.active = Some(ActiveReplay {
            token: replay.token,
            lane_count: replay.lane_count,
            events: Vec::new(),
            submitted: false,
        });
        Ok(())
    }

    fn dispatch(
        &mut self,
        request: DispatchRequest<'_, Self::Signal>,
    ) -> Result<Self::Signal, Self::Error> {
        let result = (|| {
            let active = self
                .active
                .as_ref()
                .ok_or(HipBackendError::ReplayNotActive(request.token))?;
            if active.token != request.token {
                return Err(HipBackendError::ReplayNotActive(request.token));
            }
            if request.lane.0 >= active.lane_count {
                return Err(HipBackendError::InvalidLane {
                    lane: request.lane,
                    lane_count: active.lane_count,
                });
            }
            self.validate_accesses(request.accesses)?;
            for dependency in request.dependency_signals {
                if dependency.token != request.token {
                    return Err(HipBackendError::ForeignDependencySignal {
                        replay: request.token,
                        signal: dependency.token,
                    });
                }
            }

            let function = self
                .kernels
                .get(request.launch.kernel())
                .copied()
                .ok_or_else(|| HipBackendError::KernelNotRegistered {
                    key: request.launch.kernel().to_owned(),
                })?;
            let mut packed = PackedArguments::new(request.launch.arguments(), &self.resources)?;
            let event = match self.available_events.pop() {
                Some(event) => event,
                None => Rc::new(Event::create(&self.fns)?),
            };
            self.active
                .as_mut()
                .expect("active replay was validated above")
                .submitted = true;
            let stream = &self.workers[request.lane.0];
            for dependency in request.dependency_signals {
                if dependency.lane != request.lane {
                    stream.wait(&dependency.event)?;
                }
            }

            let grid = request.launch.grid();
            let block = request.launch.block();
            // SAFETY: the registered handle contract guarantees the function
            // and ABI. This call owns naturally aligned argument storage until
            // HIP has copied the kernargs, and the stream remains live.
            unsafe {
                self.fns.check(
                    "hipModuleLaunchKernel",
                    (self.fns.module_launch_kernel)(
                        function.as_ptr(),
                        grid.x,
                        grid.y,
                        grid.z,
                        block.x,
                        block.y,
                        block.z,
                        request.launch.dynamic_shared_bytes(),
                        stream.raw.as_ptr(),
                        packed.params_mut_ptr(),
                        ptr::null_mut(),
                    ),
                )?;
            }

            event.record(stream)?;
            self.active
                .as_mut()
                .expect("active replay was validated above")
                .events
                .push(Rc::clone(&event));
            Ok(HipSignal {
                event,
                token: request.token,
                lane: request.lane,
            })
        })();
        result.map_err(|cause| self.recover_adapter_failure(cause))
    }

    fn end_replay(
        &mut self,
        replay: EndReplay<'_, Self::Signal>,
    ) -> Result<Self::Completion, Self::Error> {
        let result = (|| {
            let active = self
                .active
                .as_ref()
                .ok_or(HipBackendError::ReplayNotActive(replay.token))?;
            if active.token != replay.token {
                return Err(HipBackendError::ReplayNotActive(replay.token));
            }
            if replay.mode != ReplayMode::TokenLatency {
                return Err(HipBackendError::UnsupportedReplayMode(replay.mode));
            }
            for terminal in replay.terminal_signals {
                if terminal.token != replay.token {
                    return Err(HipBackendError::ForeignDependencySignal {
                        replay: replay.token,
                        signal: terminal.token,
                    });
                }
            }

            let joined = Event::create(&self.fns)?;
            self.active
                .as_mut()
                .expect("active replay was validated above")
                .submitted = true;
            for terminal in replay.terminal_signals {
                self.coordinator.wait(&terminal.event)?;
            }
            joined.record(&self.coordinator)?;
            joined.synchronize()?;
            let active = self
                .active
                .take()
                .expect("active replay was validated above");
            self.available_events.extend(active.events);
            Ok(HipCompletion {
                token: replay.token,
            })
        })();
        result.map_err(|cause| self.recover_adapter_failure(cause))
    }
}

fn validate_backend_config(
    config: &HipBackendConfig,
    hardware_queue_environment: Option<&OsStr>,
) -> Result<usize, HipBackendError> {
    if config.mode() == HipBackendMode::SerialFallback {
        debug_assert_eq!(config.worker_count(), NonZeroUsize::MIN);
        return Ok(1);
    }

    validate_hw_queue_environment(config.worker_count(), hardware_queue_environment)
}

/// Create one green context + bound stream per validated CU partition.
///
/// Device CU count comes from `hipDeviceGetDevResource` with the SM resource
/// type (`HipDevSmResource::sm_count`). Equal partitions use
/// `hipDevSmResourceSplitByCount`; explicit unequal slices use
/// `hipDevSmResourceSplit` with per-group SM counts.
fn create_partitioned_workers(
    fns: &Arc<HipFns>,
    device_ordinal: i32,
    config: &HipBackendConfig,
) -> Result<(Vec<Stream>, Vec<OwnedExecutionCtx>), HipBackendError> {
    let symbols =
        ffi_execution_ctx::load_execution_ctx_symbols().map_err(map_execution_ctx_load_error)?;

    let mut device_resource = HipDevResource::default();
    // SAFETY: `device_resource` is a valid writable hipDevResource; type is SM.
    unsafe {
        fns.check(
            "hipDeviceGetDevResource",
            (symbols.device_get_dev_resource)(
                device_ordinal,
                &mut device_resource,
                HIP_DEV_RESOURCE_TYPE_SM,
            ),
        )?;
    }
    // SAFETY: hipDeviceGetDevResource with SM type fills the sm union arm.
    let device_cu_count = unsafe { device_resource.payload.sm.sm_count };

    let partitions = partition::validate(config.get_partition_policy(), device_cu_count)
        .map_err(HipBackendError::InvalidPartition)?;

    let worker_count = config.worker_count().get();
    if partitions.len() != worker_count {
        return Err(HipBackendError::PartitionWorkerMismatch {
            partitions: partitions.len(),
            workers: worker_count,
        });
    }

    let resources = match config.get_partition_policy() {
        PartitionPolicy::None => unreachable!("caller filters PartitionPolicy::None"),
        PartitionPolicy::Equal(_) => {
            let min_count = partitions[0].cu_count;
            let mut results = vec![HipDevResource::default(); worker_count];
            let mut nb_groups =
                c_uint::try_from(worker_count).map_err(|_| HipBackendError::InvalidConfig {
                    detail: format!("worker count {worker_count} exceeds u32"),
                })?;
            let mut remainder = HipDevResource::default();
            // SAFETY: result buffer has nb_groups slots; input is the device SM
            // resource; remainder is a valid out-parameter.
            unsafe {
                fns.check(
                    "hipDevSmResourceSplitByCount",
                    (symbols.dev_sm_resource_split_by_count)(
                        results.as_mut_ptr(),
                        &mut nb_groups,
                        &device_resource,
                        &mut remainder,
                        0,
                        min_count,
                    ),
                )?;
            }
            if usize::try_from(nb_groups).unwrap_or(usize::MAX) != worker_count {
                return Err(HipBackendError::InvalidConfig {
                    detail: format!(
                        "hipDevSmResourceSplitByCount returned {nb_groups} groups, expected {worker_count}"
                    ),
                });
            }
            results
        }
        PartitionPolicy::Explicit(_) => {
            let mut results = vec![HipDevResource::default(); worker_count];
            let mut remainder = HipDevResource::default();
            let mut group_params: Vec<HipDevSmResourceGroupParams> = partitions
                .iter()
                .map(|part| HipDevSmResourceGroupParams {
                    sm_count: part.cu_count,
                    coscheduled_sm_count: 0,
                    preferred_coscheduled_sm_count: 0,
                    flags: ffi_execution_ctx::HIP_DEV_SM_RESOURCE_GROUP_DEFAULT,
                    reserved: [0; 12],
                })
                .collect();
            let nb_groups =
                c_uint::try_from(worker_count).map_err(|_| HipBackendError::InvalidConfig {
                    detail: format!("worker count {worker_count} exceeds u32"),
                })?;
            // SAFETY: result/group_params lengths match nb_groups; input is the
            // device SM resource; remainder is a valid out-parameter.
            unsafe {
                fns.check(
                    "hipDevSmResourceSplit",
                    (symbols.dev_sm_resource_split)(
                        results.as_mut_ptr(),
                        nb_groups,
                        &device_resource,
                        &mut remainder,
                        0,
                        group_params.as_mut_ptr(),
                    ),
                )?;
            }
            results
        }
    };

    // DROP ORDER (locals): Rust drops locals in *reverse* declaration order.
    // On a mid-loop failure, streams must die before contexts, so `contexts`
    // is declared before `workers` (workers drop first). Opposite of the
    // HipMultiStreamBackend struct-field rule above — do not unify them.
    let mut contexts = Vec::with_capacity(worker_count);
    let mut workers = Vec::with_capacity(worker_count);
    for resource in resources {
        let ctx = green_ctx_from_resource(fns, &symbols, device_ordinal, resource)?;
        let stream = Stream::create_from_execution_ctx(fns, &symbols, ctx.raw())?;
        contexts.push(ctx);
        workers.push(stream);
    }
    Ok((workers, contexts))
}

fn green_ctx_from_resource(
    fns: &Arc<HipFns>,
    symbols: &Arc<ExecutionCtxSymbols>,
    device_ordinal: i32,
    mut resource: HipDevResource,
) -> Result<OwnedExecutionCtx, HipBackendError> {
    let mut desc = HipDevResourceDesc(ptr::null_mut());
    // SAFETY: desc/resource are valid out/in locations for one SM resource.
    unsafe {
        fns.check(
            "hipDevResourceGenerateDesc",
            (symbols.dev_resource_generate_desc)(&mut desc, &mut resource, 1),
        )?;
    }
    if desc.0.is_null() {
        return Err(HipBackendError::NullHandle {
            operation: "hipDevResourceGenerateDesc",
        });
    }
    let mut ctx = HipExecutionCtx(ptr::null_mut());
    // SAFETY: desc was just generated; device_ordinal is the loaded device.
    unsafe {
        fns.check(
            "hipGreenCtxCreate",
            (symbols.green_ctx_create)(&mut ctx, desc, device_ordinal, 0),
        )?;
    }
    if ctx.0.is_null() {
        return Err(HipBackendError::NullHandle {
            operation: "hipGreenCtxCreate",
        });
    }
    Ok(OwnedExecutionCtx {
        symbols: Arc::clone(symbols),
        raw: ctx,
    })
}

fn map_execution_ctx_load_error(error: ffi_execution_ctx::LoadError) -> HipBackendError {
    match error {
        ffi_execution_ctx::LoadError::Library { candidates, detail } => {
            HipBackendError::LibraryLoad { candidates, detail }
        }
        ffi_execution_ctx::LoadError::Symbol(missing) => HipBackendError::Symbol {
            symbol: missing.0,
            detail: missing.to_string(),
        },
    }
}

fn map_batch_mem_load_error(error: ffi_batch_mem::LoadError) -> HipBackendError {
    match error {
        ffi_batch_mem::LoadError::Library { candidates, detail } => {
            HipBackendError::LibraryLoad { candidates, detail }
        }
        ffi_batch_mem::LoadError::Symbol(missing) => HipBackendError::Symbol {
            symbol: missing.0,
            detail: missing.to_string(),
        },
    }
}

fn validate_hw_queue_environment(
    required: NonZeroUsize,
    raw: Option<&OsStr>,
) -> Result<usize, HipBackendError> {
    let raw = raw.ok_or(HipBackendError::MissingHardwareQueueEnvironment { required })?;
    let value = raw
        .to_str()
        .ok_or_else(|| HipBackendError::InvalidHardwareQueueEnvironment {
            value: raw.to_string_lossy().into_owned(),
        })?
        .parse::<usize>()
        .map_err(|_| HipBackendError::InvalidHardwareQueueEnvironment {
            value: raw.to_string_lossy().into_owned(),
        })?;
    if value == 0 {
        return Err(HipBackendError::InvalidHardwareQueueEnvironment {
            value: raw.to_string_lossy().into_owned(),
        });
    }
    if value < required.get() {
        return Err(HipBackendError::InsufficientHardwareQueueEnvironment {
            configured: value,
            required,
        });
    }
    Ok(value)
}

/// Owns one naturally aligned argument value.
struct AlignedArgument {
    pointer: NonNull<u8>,
    layout: Layout,
}

impl AlignedArgument {
    fn new(bytes: &[u8], argument_index: usize) -> Result<Self, HipBackendError> {
        if bytes.is_empty() {
            return Err(HipBackendError::EmptyScalarArgument { argument_index });
        }
        let alignment = bytes
            .len()
            .checked_next_power_of_two()
            .ok_or(HipBackendError::ArgumentLayout { argument_index })?
            .max(std::mem::align_of::<usize>());
        let layout = Layout::from_size_align(bytes.len(), alignment)
            .map_err(|_| HipBackendError::ArgumentLayout { argument_index })?;
        // SAFETY: layout is nonzero and valid. Allocation failure follows the
        // standard infallible Rust allocation policy.
        let raw = unsafe { alloc(layout) };
        let pointer = NonNull::new(raw).unwrap_or_else(|| handle_alloc_error(layout));
        // SAFETY: destination covers bytes.len() initialized writable bytes;
        // source is a live slice and the allocations cannot overlap.
        unsafe {
            ptr::copy_nonoverlapping(bytes.as_ptr(), pointer.as_ptr(), bytes.len());
        }
        Ok(Self { pointer, layout })
    }

    fn as_mut_void(&mut self) -> *mut c_void {
        self.pointer.as_ptr().cast()
    }

    #[cfg(test)]
    fn bytes(&self) -> &[u8] {
        // SAFETY: the allocation remains live and exactly `len` bytes were
        // initialized in new().
        unsafe { std::slice::from_raw_parts(self.pointer.as_ptr(), self.layout.size()) }
    }
}

impl Drop for AlignedArgument {
    fn drop(&mut self) {
        // SAFETY: pointer was allocated with this exact layout and is uniquely
        // owned by this object.
        unsafe { dealloc(self.pointer.as_ptr(), self.layout) };
    }
}

struct PackedArguments {
    storage: Vec<AlignedArgument>,
    params: Vec<*mut c_void>,
}

impl PackedArguments {
    fn new(
        arguments: &[KernelArg],
        bindings: &HashMap<ResourceId, ResourceBinding>,
    ) -> Result<Self, HipBackendError> {
        Self::new_with_bindings(arguments, bindings, None)
    }

    fn new_with_bindings(
        arguments: &[KernelArg],
        bindings: &HashMap<ResourceId, ResourceBinding>,
        replay_bindings: Option<&ReplayBindings>,
    ) -> Result<Self, HipBackendError> {
        let mut storage = Vec::with_capacity(arguments.len());
        for (argument_index, argument) in arguments.iter().enumerate() {
            let bytes: Vec<u8> = match argument {
                KernelArg::Scalar(bytes) => bytes.to_vec(),
                KernelArg::ScalarSlot { slot, size } => {
                    let Some(replay_bindings) = replay_bindings else {
                        return Err(HipBackendError::DynamicScalarNotBound { slot: *slot });
                    };
                    let bytes = replay_bindings.scalar(*slot).ok_or_else(|| {
                        HipBackendError::ReplayBindings(ReplayBindingError::ScalarNotBound {
                            slot: *slot,
                        })
                    })?;
                    if bytes.len() != *size as usize {
                        return Err(HipBackendError::ReplayBindings(
                            ReplayBindingError::ScalarSize {
                                slot: *slot,
                                expected: *size,
                                actual: bytes.len(),
                            },
                        ));
                    }
                    bytes.to_vec()
                }
                KernelArg::Resource {
                    resource,
                    byte_offset,
                } => {
                    let binding = bindings
                        .get(resource)
                        .ok_or(HipBackendError::ResourceNotBound(*resource))?;
                    if *byte_offset >= binding.size {
                        return Err(HipBackendError::ResourceArgumentOutOfBounds {
                            resource: *resource,
                            offset: *byte_offset,
                            bound: binding.size,
                        });
                    }
                    let offset = usize::try_from(*byte_offset).map_err(|_| {
                        HipBackendError::DeviceAddressOverflow {
                            resource: *resource,
                            offset: *byte_offset,
                        }
                    })?;
                    let address = (binding.base.as_ptr() as usize).checked_add(offset).ok_or(
                        HipBackendError::DeviceAddressOverflow {
                            resource: *resource,
                            offset: *byte_offset,
                        },
                    )?;
                    address.to_ne_bytes().to_vec()
                }
            };
            storage.push(AlignedArgument::new(&bytes, argument_index)?);
        }
        let params = storage
            .iter_mut()
            .map(AlignedArgument::as_mut_void)
            .collect();
        Ok(Self { storage, params })
    }

    fn params_mut_ptr(&mut self) -> *mut *mut c_void {
        if self.params.is_empty() {
            ptr::null_mut()
        } else {
            self.params.as_mut_ptr()
        }
    }
}

impl Drop for PackedArguments {
    fn drop(&mut self) {
        // Keep the ownership relationship explicit: `params` borrows pointers
        // into `storage` and must never outlive it.
        self.params.clear();
        self.storage.clear();
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HipBackendError {
    #[error("GPU_MAX_HW_QUEUES must be set to at least {required} before any HIP initialization")]
    MissingHardwareQueueEnvironment { required: NonZeroUsize },
    #[error("GPU_MAX_HW_QUEUES must be a positive integer, got {value:?}")]
    InvalidHardwareQueueEnvironment { value: String },
    #[error(
        "GPU_MAX_HW_QUEUES={configured} is smaller than the requested {required} worker queues"
    )]
    InsufficientHardwareQueueEnvironment {
        configured: usize,
        required: NonZeroUsize,
    },
    #[error("failed to load HIP runtime from {candidates}: {detail}")]
    LibraryLoad { candidates: String, detail: String },
    #[error("failed to load HIP symbol {symbol}: {detail}")]
    Symbol {
        symbol: &'static str,
        detail: String,
    },
    #[error("{operation} returned HIP error {code}: {message}")]
    Hip {
        operation: &'static str,
        code: HipErrorCode,
        message: String,
    },
    #[error("{operation} succeeded but returned a null handle")]
    NullHandle { operation: &'static str },
    #[error("device {requested} is outside the available range 0..{available}")]
    InvalidDevice { requested: i32, available: i32 },
    #[error("replay {0:?} is already active")]
    ReplayAlreadyActive(ReplayToken),
    #[error("replay {0:?} is not active")]
    ReplayNotActive(ReplayToken),
    #[error("HIP backend is poisoned after an unrecoverable submission failure: {detail}")]
    BackendPoisoned { detail: String },
    #[error("prepared HIP plan belongs to another backend")]
    PreparedPlanBackendMismatch,
    #[error("compiled plan is internally inconsistent: {detail}")]
    InvalidCompiledPlan { detail: String },
    #[error("batch command or event count overflows usize")]
    BatchSizeOverflow,
    #[error("replay token range overflows u64")]
    ReplayTokenRangeOverflow,
    #[error("batch submission failed ({cause}) and recovery drain failed ({drain_error})")]
    BatchSubmissionPoisoned { cause: String, drain_error: String },
    #[error("adapter submission failed ({cause}) and recovery drain failed ({drain_error})")]
    AdapterSubmissionPoisoned { cause: String, drain_error: String },
    #[error("HIP multi-stream backend does not yet support {0:?}")]
    UnsupportedReplayMode(ReplayMode),
    #[error("plan requests {requested} lanes but this backend owns only {available} workers")]
    TooManyLanes { requested: usize, available: usize },
    #[error("lane {lane:?} is outside the active lane count {lane_count}")]
    InvalidLane { lane: LaneId, lane_count: usize },
    #[error("kernel key is empty")]
    EmptyKernelKey,
    #[error("registered hipFunction_t is null")]
    NullKernelFunction,
    #[error("kernel {0:?} is already registered")]
    KernelAlreadyRegistered(String),
    #[error("kernel {key:?} is not registered")]
    KernelNotRegistered { key: String },
    #[error("dynamic scalar slot {slot:?} requires ReplayBindings before HIP preparation")]
    DynamicScalarNotBound { slot: crate::ScalarSlotId },
    #[error("invalid shared replay bindings: {0}")]
    ReplayBindings(#[source] ReplayBindingError),
    #[error("device pointer for {0:?} is null")]
    NullDevicePointer(ResourceId),
    #[error("device binding for {0:?} is empty")]
    EmptyDeviceBinding(ResourceId),
    #[error("resource {0:?} is already bound")]
    ResourceAlreadyBound(ResourceId),
    #[error("device binding range for {resource:?} with size {size} overflows")]
    DeviceBindingAddressOverflow { resource: ResourceId, size: u64 },
    #[error("device binding for {resource:?} aliases distinct logical resource {other:?}")]
    AliasingResourceBinding {
        resource: ResourceId,
        other: ResourceId,
    },
    #[error("resource {0:?} has no device binding")]
    ResourceNotBound(ResourceId),
    #[error("resource {resource:?} needs {required} bytes but binding has {bound}")]
    BindingTooSmall {
        resource: ResourceId,
        required: u64,
        bound: u64,
    },
    #[error("resource argument {resource:?}+{offset} exceeds its {bound}-byte binding")]
    ResourceArgumentOutOfBounds {
        resource: ResourceId,
        offset: u64,
        bound: u64,
    },
    #[error("device address for {resource:?}+{offset} overflows this host address space")]
    DeviceAddressOverflow { resource: ResourceId, offset: u64 },
    #[error("scalar kernel argument {argument_index} is empty")]
    EmptyScalarArgument { argument_index: usize },
    #[error("kernel argument {argument_index} has an unsupported size/alignment")]
    ArgumentLayout { argument_index: usize },
    #[error("dependency signal belongs to {signal:?}, not active replay {replay:?}")]
    ForeignDependencySignal {
        replay: ReplayToken,
        signal: ReplayToken,
    },
    #[error("invalid HIP backend partition policy: {0}")]
    InvalidPartition(#[source] partition::PartitionError),
    #[error("partition policy produced {partitions} slices but backend worker_count is {workers}")]
    PartitionWorkerMismatch { partitions: usize, workers: usize },
    #[error("invalid HIP backend configuration: {detail}")]
    InvalidConfig { detail: String },
}

fn validate_bindings(
    resources: &HashMap<ResourceId, ResourceBinding>,
    plan: &CompiledPlan,
) -> Result<(), HipBackendError> {
    for resource in plan.resources() {
        let binding = resources
            .get(&resource.id())
            .ok_or(HipBackendError::ResourceNotBound(resource.id()))?;
        if binding.size < resource.size() {
            return Err(HipBackendError::BindingTooSmall {
                resource: resource.id(),
                required: resource.size(),
                bound: binding.size,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Dim3, KernelLaunch, Recorder};
    use std::cell::Cell;

    fn empty_launch(name: &str) -> KernelLaunch {
        KernelLaunch::new(name, Dim3::x(1).unwrap(), Dim3::x(1).unwrap()).unwrap()
    }

    #[test]
    fn serial_fallback_config_ignores_missing_hardware_queue_environment() {
        let config = HipBackendConfig::serial_fallback(3);
        assert_eq!(config.device_ordinal(), 3);
        assert_eq!(config.mode(), HipBackendMode::SerialFallback);
        assert_eq!(config.worker_count(), NonZeroUsize::MIN);
        assert_eq!(validate_backend_config(&config, None).unwrap(), 1);

        // A serial fallback does not inherit validation failures from an
        // embedding process's unrelated queue setting either.
        assert_eq!(
            validate_backend_config(&config, Some(OsStr::new("not-a-number"))).unwrap(),
            1
        );
    }

    #[test]
    fn multi_stream_config_preserves_hardware_queue_environment_contract() {
        let required = NonZeroUsize::new(2).unwrap();
        let config = HipBackendConfig::new(0, required);
        assert_eq!(config.mode(), HipBackendMode::MultiStream);
        assert!(matches!(
            validate_backend_config(&config, None),
            Err(HipBackendError::MissingHardwareQueueEnvironment {
                required: missing
            }) if missing == required
        ));
        assert_eq!(
            validate_backend_config(&config, Some(OsStr::new("4"))).unwrap(),
            4
        );
    }

    #[test]
    fn serial_fallback_batch_has_one_terminal_host_sync() {
        let mut recorder = Recorder::new();
        for index in 0..3 {
            recorder
                .dispatch(empty_launch(&format!("serial-{index}")), [])
                .unwrap();
        }
        let plan = recorder
            .compile(crate::CompileOptions::lanes(1, ReplayMode::TokenLatency).unwrap())
            .unwrap();
        let lowered = lower_plan(&plan).unwrap();
        assert_eq!(lowered.stats.cross_lane_signals, 0);
        assert_eq!(lowered.stats.terminal_lane_tails, 1);
        assert_eq!(lowered.stats.dispatch_event_slots, 1);

        let batch = calculate_batch_stats(
            lowered.stats,
            lowered.event_slot_count,
            NonZeroUsize::new(5).unwrap(),
            0,
        )
        .unwrap();
        assert_eq!(batch.kernel_launches, 15);
        assert_eq!(batch.dispatch_event_records, 5);
        assert_eq!(batch.dependency_waits, 0);
        assert_eq!(batch.token_boundary_waits, 0);
        assert_eq!(batch.coordinator_waits, 1);
        assert_eq!(batch.coordinator_join_records, 1);
        assert_eq!(batch.host_synchronizations, 1);
    }

    #[test]
    fn frontier_lowering_coalesces_two_lane_root_child_phases() {
        let mut recorder = Recorder::new();
        let roots = (0..4)
            .map(|index| {
                recorder
                    .dispatch(empty_launch(&format!("root-{index}")), [])
                    .unwrap()
            })
            .collect::<Vec<_>>();
        let children = (0..4)
            .map(|index| {
                recorder
                    .dispatch(empty_launch(&format!("child-{index}")), [])
                    .unwrap()
            })
            .collect::<Vec<_>>();

        // The roots and children alternate lanes. Each child lane consumes the
        // other lane's root tail twice; FIFO frontier propagation removes the
        // second wait on each lane.
        recorder.depends_on(children[0], roots[3]).unwrap();
        recorder.depends_on(children[1], roots[2]).unwrap();
        recorder.depends_on(children[2], roots[3]).unwrap();
        recorder.depends_on(children[3], roots[2]).unwrap();
        let plan = recorder
            .compile(crate::CompileOptions::lanes(2, ReplayMode::TokenLatency).unwrap())
            .unwrap();
        assert_eq!(
            plan.dispatches()
                .iter()
                .map(|dispatch| dispatch.lane().0)
                .collect::<Vec<_>>(),
            vec![0, 1, 0, 1, 0, 1, 0, 1]
        );

        let lowered = lower_plan(&plan).unwrap();
        assert_eq!(lowered.stats.dispatches, 8);
        assert_eq!(lowered.stats.cross_lane_signals, 2);
        assert_eq!(lowered.stats.terminal_lane_tails, 2);
        assert_eq!(lowered.stats.dispatch_event_slots, 4);
        assert_eq!(lowered.stats.cross_lane_waits_per_token, 2);
        assert_eq!(lowered.stats.token_boundary_waits_per_boundary, 2);
        assert_eq!(
            lowered
                .record_slot_by_dispatch
                .iter()
                .map(Option::is_some)
                .collect::<Vec<_>>(),
            vec![false, false, true, true, false, false, true, true]
        );
        assert_eq!(
            lowered
                .wait_slots_by_dispatch
                .iter()
                .map(Vec::len)
                .collect::<Vec<_>>(),
            vec![0, 0, 0, 0, 1, 1, 0, 0]
        );

        let batch = calculate_batch_stats(
            lowered.stats,
            lowered.event_slot_count,
            NonZeroUsize::new(3).unwrap(),
            0,
        )
        .unwrap();
        assert_eq!(batch.kernel_launches, 24);
        assert_eq!(batch.dispatch_event_records, 12);
        assert_eq!(batch.dependency_waits, 6);
        assert_eq!(batch.token_boundary_waits, 4);
        assert_eq!(batch.coordinator_waits, 2);
        assert_eq!(batch.coordinator_join_records, 1);
        assert_eq!(batch.host_synchronizations, 1);
    }

    #[test]
    fn batch_shape_rejects_count_overflow_before_submission() {
        let shape = PreparedHipPlanStats {
            dispatches: usize::MAX,
            cross_lane_signals: 0,
            terminal_lane_tails: 1,
            dispatch_event_slots: 1,
            cross_lane_waits_per_token: 0,
            token_boundary_waits_per_boundary: 0,
        };
        assert!(matches!(
            calculate_batch_stats(shape, 1, NonZeroUsize::new(2).unwrap(), 0),
            Err(HipBackendError::BatchSizeOverflow)
        ));
    }

    #[test]
    fn failed_event_generation_is_discarded_instead_of_recycled() {
        struct DropCounter(Rc<Cell<usize>>);

        impl Drop for DropCounter {
            fn drop(&mut self) {
                self.0.set(self.0.get() + 1);
            }
        }

        let drops = Rc::new(Cell::new(0));
        let events = vec![
            DropCounter(Rc::clone(&drops)),
            DropCounter(Rc::clone(&drops)),
        ];
        let joined = DropCounter(Rc::clone(&drops));
        discard_failed_event_generation(events, joined);
        assert_eq!(drops.get(), 3);
    }

    #[test]
    fn live_runtime_load_smoke_when_enabled() {
        if env::var_os("REDLINE_TEST_HIP").is_none() {
            return;
        }
        let workers = env::var("GPU_MAX_HW_QUEUES")
            .unwrap()
            .parse::<usize>()
            .unwrap()
            .min(2);
        let mut backend = HipMultiStreamBackend::load(HipBackendConfig::new(
            0,
            NonZeroUsize::new(workers).unwrap(),
        ))
        .unwrap();
        assert_eq!(backend.worker_count(), workers);
        assert!(backend.configured_hw_queue_limit() >= workers);
        assert_eq!(backend.poisoned_reason(), None);

        let mut recorder = Recorder::new();
        recorder
            .dispatch(empty_launch("unregistered-kernel"), [])
            .unwrap();
        let plan = recorder
            .compile(crate::CompileOptions::lanes(1, ReplayMode::TokenLatency).unwrap())
            .unwrap();
        assert!(matches!(
            backend.prepare_plan(&plan),
            Err(HipBackendError::KernelNotRegistered { .. })
        ));

        let dispatch = &plan.dispatches()[0];
        backend
            .begin_replay(BeginReplay {
                token: ReplayToken(7),
                mode: ReplayMode::TokenLatency,
                lane_count: 1,
            })
            .unwrap();
        assert!(matches!(
            backend.dispatch(DispatchRequest {
                token: ReplayToken(7),
                node: dispatch.node(),
                lane: dispatch.lane(),
                launch: dispatch.launch(),
                accesses: dispatch.accesses(),
                dependency_signals: &[],
            }),
            Err(HipBackendError::KernelNotRegistered { .. })
        ));
        assert!(backend.active.is_none());
        assert_eq!(backend.poisoned_reason(), None);

        backend
            .begin_replay(BeginReplay {
                token: ReplayToken(8),
                mode: ReplayMode::TokenLatency,
                lane_count: 1,
            })
            .unwrap();
        let wrong_end_mode = ReplayMode::throughput(2).unwrap();
        assert!(matches!(
            backend.end_replay(EndReplay {
                token: ReplayToken(8),
                mode: wrong_end_mode,
                terminal_signals: &[],
            }),
            Err(HipBackendError::UnsupportedReplayMode(mode)) if mode == wrong_end_mode
        ));
        assert!(backend.active.is_none());
        assert_eq!(backend.poisoned_reason(), None);

        let throughput = ReplayMode::throughput(2).unwrap();
        assert!(matches!(
            backend.begin_replay(BeginReplay {
                token: ReplayToken(0),
                mode: throughput,
                lane_count: workers,
            }),
            Err(HipBackendError::UnsupportedReplayMode(mode)) if mode == throughput
        ));
    }

    #[test]
    fn packs_scalar_bytes_without_reordering() {
        let arguments = [
            KernelArg::scalar(17_u32.to_ne_bytes().to_vec()),
            KernelArg::scalar((-2.5_f32).to_ne_bytes().to_vec()),
        ];
        let packed = PackedArguments::new(&arguments, &HashMap::new()).unwrap();
        assert_eq!(packed.storage[0].bytes(), 17_u32.to_ne_bytes());
        assert_eq!(packed.storage[1].bytes(), (-2.5_f32).to_ne_bytes());
        assert_eq!((packed.params[0] as usize) % 4, 0);
        assert_eq!((packed.params[1] as usize) % 4, 0);
    }

    #[test]
    fn packs_dynamic_scalar_from_shared_replay_bindings() {
        let slot = crate::ScalarSlotId::new(7);
        let arguments = [KernelArg::scalar_slot(slot, 4).unwrap()];
        let mut replay_bindings = ReplayBindings::new();
        replay_bindings.bind_scalar(slot, 29_u32.to_ne_bytes().to_vec());

        let packed =
            PackedArguments::new_with_bindings(&arguments, &HashMap::new(), Some(&replay_bindings))
                .unwrap();
        assert_eq!(packed.storage[0].bytes(), 29_u32.to_ne_bytes());
        assert_eq!((packed.params[0] as usize) % 4, 0);

        let missing = PackedArguments::new(&arguments, &HashMap::new())
            .err()
            .unwrap();
        assert!(matches!(
            missing,
            HipBackendError::DynamicScalarNotBound { slot: missing } if missing == slot
        ));
    }

    #[test]
    fn dynamic_scalar_binding_size_is_checked() {
        let slot = crate::ScalarSlotId::new(9);
        let arguments = [KernelArg::scalar_slot(slot, 4).unwrap()];
        let mut replay_bindings = ReplayBindings::new();
        replay_bindings.bind_scalar(slot, vec![1_u8, 2]);

        let error =
            PackedArguments::new_with_bindings(&arguments, &HashMap::new(), Some(&replay_bindings))
                .err()
                .unwrap();
        assert!(matches!(
            error,
            HipBackendError::ReplayBindings(ReplayBindingError::ScalarSize {
                slot: bound,
                expected: 4,
                actual: 2,
            }) if bound == slot
        ));
    }

    #[test]
    fn packs_resource_as_base_plus_offset_device_pointer() {
        let mut recorder = Recorder::new();
        let resource = recorder.resource("buffer", 64).unwrap();
        let arguments = [KernelArg::resource(resource, 24)];
        let bindings = HashMap::from([(
            resource,
            ResourceBinding {
                base: NonNull::new(0x1000_usize as *mut c_void).unwrap(),
                size: 64,
            },
        )]);
        let packed = PackedArguments::new(&arguments, &bindings).unwrap();
        let mut bytes = [0_u8; std::mem::size_of::<usize>()];
        bytes.copy_from_slice(packed.storage[0].bytes());
        assert_eq!(usize::from_ne_bytes(bytes), 0x1018);
    }

    #[test]
    fn resource_packing_checks_physical_binding_bounds() {
        let mut recorder = Recorder::new();
        let resource = recorder.resource("buffer", 64).unwrap();
        let arguments = [KernelArg::resource(resource, 32)];
        let bindings = HashMap::from([(
            resource,
            ResourceBinding {
                base: NonNull::new(0x1000_usize as *mut c_void).unwrap(),
                size: 16,
            },
        )]);
        assert!(matches!(
            PackedArguments::new(&arguments, &bindings),
            Err(HipBackendError::ResourceArgumentOutOfBounds { .. })
        ));
    }

    #[test]
    fn plan_binding_validation_uses_declared_resource_extent() {
        let mut recorder = Recorder::new();
        let resource = recorder.resource("buffer", 64).unwrap();
        let region = recorder.region(resource, 0, 64).unwrap();
        recorder
            .dispatch(
                KernelLaunch::new("kernel", Dim3::x(1).unwrap(), Dim3::x(1).unwrap()).unwrap(),
                [crate::Access::read(region)],
            )
            .unwrap();
        let plan = recorder
            .compile(crate::CompileOptions::lanes(1, ReplayMode::TokenLatency).unwrap())
            .unwrap();

        let resources = HashMap::from([(
            resource,
            ResourceBinding {
                base: NonNull::new(0x1000_usize as *mut c_void).unwrap(),
                size: 32,
            },
        )]);
        assert!(matches!(
            validate_bindings(&resources, &plan),
            Err(HipBackendError::BindingTooSmall {
                required: 64,
                bound: 32,
                ..
            })
        ));
    }
}
