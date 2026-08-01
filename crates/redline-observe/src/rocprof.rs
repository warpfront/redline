// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! Embedded rocprofiler-sdk dispatch counter collection (ROCm 7.14).
//!
//! Registration-based lifecycle (no `rocprofiler_initialize` /
//! `rocprofiler_finalize`): in-process tools call
//! [`rocprofiler_force_configure`](RocProf::start_dispatch_counting) with a
//! `rocprofiler_configure`-compatible callback. The minimal per-dispatch path is:
//!
//! 1. `rocprofiler_create_context`
//! 2. `rocprofiler_configure_callback_dispatch_counting_service`
//! 3. `rocprofiler_start_context`
//!
//! Counter selection happens inside the dispatch callback via
//! `rocprofiler_iterate_agent_supported_counters` +
//! `rocprofiler_create_counter_config` (sample counter: `SQ_WAVES`).
//!
//! Header provenance (ROCm Core SDK 7.14 / rocprofiler-sdk 1.3.2):
//! - `/opt/rocm/core/include/rocprofiler-sdk/rocprofiler.h`
//! - `/opt/rocm/core/include/rocprofiler-sdk/registration.h`
//! - `/opt/rocm/core/include/rocprofiler-sdk/context.h`
//! - `/opt/rocm/core/include/rocprofiler-sdk/dispatch_counting_service.h`
//! - `/opt/rocm/core/include/rocprofiler-sdk/counter_config.h`
//! - `/opt/rocm/core/include/rocprofiler-sdk/counters.h`
//! - `/opt/rocm/core/include/rocprofiler-sdk/fwd.h`
//!
//! Load target: `librocprofiler-sdk.so.1`.

use std::collections::HashMap;
use std::ffi::{CStr, c_char, c_int, c_void};
use std::mem::size_of;
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, Mutex};

use libloading::Library;
use thiserror::Error;

// ---------------------------------------------------------------------------
// Documented env knobs (library strings; no header docs)
// ---------------------------------------------------------------------------

/// HSA queue interposition mode for dispatch interception.
///
/// Proven in `librocprofiler-sdk.so.1` runtime strings
/// (`ROCPROFILER_QUEUE_INTERPOSITION`, related
/// `hsa/queue_interposition.cpp`). Not declared in installed headers.
pub const ROCPROFILER_QUEUE_INTERPOSITION: &str = "ROCPROFILER_QUEUE_INTERPOSITION";

/// Optional absolute/soname override for the rocprofiler-sdk shared library.
/// When set, only this path is attempted (used by unit tests).
pub const REDLINE_ROCPROFILER_LIB: &str = "REDLINE_ROCPROFILER_LIB";

/// Default hardware counter collected per dispatch (sample path).
pub const DEFAULT_DISPATCH_COUNTER: &str = "SQ_WAVES";

const CANDIDATES: &[&str] = &[
    "librocprofiler-sdk.so.1",
    "/opt/rocm/core/lib/librocprofiler-sdk.so.1",
    "/opt/rocm/lib/librocprofiler-sdk.so.1",
];

// ---------------------------------------------------------------------------
// Status + ABI types (transcribed from fwd.h / registration.h / …)
// ---------------------------------------------------------------------------

/// `rocprofiler_status_t`.
pub type Status = i32;

pub const STATUS_SUCCESS: Status = 0;
pub const STATUS_ERROR: Status = 1;
pub const STATUS_ERROR_CONTEXT_NOT_FOUND: Status = 2;
pub const STATUS_ERROR_COUNTER_NOT_FOUND: Status = 8;
pub const STATUS_ERROR_SERVICE_ALREADY_CONFIGURED: Status = 15;
pub const STATUS_ERROR_CONFIGURATION_LOCKED: Status = 16;
pub const STATUS_ERROR_FINALIZED: Status = 21;
pub const STATUS_ERROR_HSA_NOT_LOADED: Status = 22;
pub const STATUS_ERROR_AGENT_DISPATCH_CONFLICT: Status = 31;
pub const STATUS_ERROR_NO_HARDWARE_COUNTERS: Status = 35;

/// `rocprofiler_counter_info_version_id_t` — `ROCPROFILER_COUNTER_INFO_VERSION_0`.
const COUNTER_INFO_VERSION_0: u32 = 1;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ContextId {
    pub handle: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AgentId {
    pub handle: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CounterId {
    pub handle: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CounterConfigId {
    pub handle: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct QueueId {
    pub handle: u64,
}

/// `rocprofiler_user_data_t` — union of `uint64_t` / `void*`.
#[repr(C)]
#[derive(Clone, Copy)]
pub union UserData {
    pub value: u64,
    pub ptr: *mut c_void,
}

// SAFETY: plain bytes; callbacks may race on the value bits only.
unsafe impl Send for UserData {}
unsafe impl Sync for UserData {}

impl Default for UserData {
    fn default() -> Self {
        Self { value: 0 }
    }
}

impl std::fmt::Debug for UserData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // SAFETY: reading the integer view of the union.
        unsafe {
            f.debug_struct("UserData")
                .field("value", &self.value)
                .finish()
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct AsyncCorrelationId {
    pub internal: u64,
    pub external: UserData,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Dim3 {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

/// `rocprofiler_kernel_dispatch_info_t` (128 bytes including reserved tail).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct KernelDispatchInfo {
    pub size: u64,
    pub agent_id: AgentId,
    pub queue_id: QueueId,
    pub kernel_id: u64,
    pub dispatch_id: u64,
    pub private_segment_size: u32,
    pub group_segment_size: u32,
    pub workgroup_size: Dim3,
    pub grid_size: Dim3,
    pub reserved_padding: [u8; 56],
}

impl Default for KernelDispatchInfo {
    fn default() -> Self {
        Self {
            size: 0,
            agent_id: AgentId::default(),
            queue_id: QueueId::default(),
            kernel_id: 0,
            dispatch_id: 0,
            private_segment_size: 0,
            group_segment_size: 0,
            workgroup_size: Dim3::default(),
            grid_size: Dim3::default(),
            reserved_padding: [0; 56],
        }
    }
}

/// `rocprofiler_dispatch_counting_service_data_t` (experimental).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct DispatchCountingServiceData {
    pub size: u64,
    pub correlation_id: AsyncCorrelationId,
    pub start_timestamp: u64,
    pub end_timestamp: u64,
    pub dispatch_info: KernelDispatchInfo,
}

/// `rocprofiler_counter_record_t` (experimental).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CounterRecord {
    pub id: u64,
    pub counter_value: f64,
    pub dispatch_id: u64,
    pub user_data: UserData,
    pub agent_id: AgentId,
}

/// `rocprofiler_counter_info_v0_t` (experimental) — bitfields packed as `flags`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct CounterInfoV0 {
    id: CounterId,
    name: *const c_char,
    description: *const c_char,
    block: *const c_char,
    expression: *const c_char,
    /// bit0 = is_constant, bit1 = is_derived
    flags: u8,
    _pad: [u8; 7],
}

/// `rocprofiler_client_id_t` (experimental).
#[repr(C)]
struct ClientId {
    size: usize,
    name: *const c_char,
    handle: u32,
}

/// `rocprofiler_tool_configure_result_t`.
#[repr(C)]
struct ToolConfigureResult {
    size: usize,
    initialize: Option<ToolInitializeFn>,
    finalize: Option<ToolFinalizeFn>,
    tool_data: *mut c_void,
}

// SAFETY: static tool_data is always null; fn pointers are Sync.
unsafe impl Send for ToolConfigureResult {}
unsafe impl Sync for ToolConfigureResult {}

type ToolInitializeFn =
    unsafe extern "C" fn(finalize_func: Option<ClientFinalizeFn>, tool_data: *mut c_void) -> c_int;
type ToolFinalizeFn = unsafe extern "C" fn(tool_data: *mut c_void);
type ClientFinalizeFn = unsafe extern "C" fn(client_id: ClientId);
type ConfigureFn = unsafe extern "C" fn(
    version: u32,
    runtime_version: *const c_char,
    priority: u32,
    client_id: *mut ClientId,
) -> *mut ToolConfigureResult;

type DispatchCountingServiceCb = unsafe extern "C" fn(
    dispatch_data: DispatchCountingServiceData,
    config: *mut CounterConfigId,
    user_data: *mut UserData,
    callback_data_args: *mut c_void,
);
type DispatchCountingRecordCb = unsafe extern "C" fn(
    dispatch_data: DispatchCountingServiceData,
    record_data: *mut CounterRecord,
    record_count: usize,
    user_data: UserData,
    callback_data_args: *mut c_void,
);
type AvailableCountersCb = unsafe extern "C" fn(
    agent_id: AgentId,
    counters: *mut CounterId,
    num_counters: usize,
    user_data: *mut c_void,
) -> Status;

type ForceConfigureFn = unsafe extern "C" fn(configure_func: Option<ConfigureFn>) -> Status;
type IsInitializedFn = unsafe extern "C" fn(status: *mut c_int) -> Status;
type CreateContextFn = unsafe extern "C" fn(context_id: *mut ContextId) -> Status;
type StartContextFn = unsafe extern "C" fn(context_id: ContextId) -> Status;
type StopContextFn = unsafe extern "C" fn(context_id: ContextId) -> Status;
type ContextIsValidFn = unsafe extern "C" fn(context_id: ContextId, status: *mut c_int) -> Status;
type ConfigureCallbackDispatchCountingServiceFn = unsafe extern "C" fn(
    context_id: ContextId,
    dispatch_callback: Option<DispatchCountingServiceCb>,
    dispatch_callback_args: *mut c_void,
    record_callback: Option<DispatchCountingRecordCb>,
    record_callback_args: *mut c_void,
) -> Status;
type CreateCounterConfigFn = unsafe extern "C" fn(
    agent_id: AgentId,
    counters_list: *mut CounterId,
    counters_count: usize,
    config_id: *mut CounterConfigId,
) -> Status;
type DestroyCounterConfigFn = unsafe extern "C" fn(config_id: CounterConfigId) -> Status;
type IterateAgentSupportedCountersFn = unsafe extern "C" fn(
    agent_id: AgentId,
    cb: Option<AvailableCountersCb>,
    user_data: *mut c_void,
) -> Status;
type QueryCounterInfoFn =
    unsafe extern "C" fn(counter_id: CounterId, version: u32, info: *mut c_void) -> Status;
type GetStatusStringFn = unsafe extern "C" fn(status: Status) -> *const c_char;
type GetVersionFn =
    unsafe extern "C" fn(major: *mut u32, minor: *mut u32, patch: *mut u32) -> Status;

const _: () = {
    assert!(size_of::<ContextId>() == 8);
    assert!(size_of::<AgentId>() == 8);
    assert!(size_of::<CounterId>() == 8);
    assert!(size_of::<CounterConfigId>() == 8);
    assert!(size_of::<UserData>() == 8);
    assert!(size_of::<AsyncCorrelationId>() == 16);
    assert!(size_of::<Dim3>() == 12);
    assert!(size_of::<KernelDispatchInfo>() == 128);
    assert!(size_of::<DispatchCountingServiceData>() == 168);
    assert!(size_of::<CounterRecord>() == 40);
    assert!(size_of::<CounterInfoV0>() == 48);
    assert!(size_of::<ClientId>() == 24);
    assert!(size_of::<ToolConfigureResult>() == 32);
};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum RocProfError {
    #[error(
        "librocprofiler-sdk.so.1 not found (requires ROCm >= 7.14); tried {candidates}: {detail}"
    )]
    LibraryNotFound { candidates: String, detail: String },

    #[error(
        "rocprofiler-sdk symbol `{symbol}` missing (requires ROCm >= 7.14 / rocprofiler-sdk 1.3+)"
    )]
    MissingSymbol { symbol: &'static str },

    #[error("rocprofiler-sdk already configured in this process (force_configure locked)")]
    ConfigurationLocked,

    /// Process-wide single-session constraint: at most one live
    /// [`DispatchCountSession`] may exist (rocprofiler tool registration is
    /// process-global).
    #[error(
        "rocprofiler-sdk dispatch counting session already active in this process \
         (process-wide single-registration constraint)"
    )]
    SessionAlreadyActive,

    #[error("rocprofiler-sdk context create/start failed: {status} ({message})")]
    ContextFailed { status: Status, message: String },

    #[error(
        "rocprofiler_configure_callback_dispatch_counting_service failed: {status} ({message})"
    )]
    CountingServiceFailed { status: Status, message: String },

    #[error("rocprofiler-sdk tool initialization failed: {0}")]
    ToolInitFailed(String),

    #[error("rocprofiler-sdk status {status}: {message}")]
    Api { status: Status, message: String },

    #[error("dispatch counting session is not active")]
    SessionInactive,

    /// First error recorded while configuring counters for a dispatch.
    #[error("dispatch counter configuration failed: {0}")]
    DispatchConfigFailed(String),

    #[error("rocprofiler_stop_context failed: {status} ({message})")]
    StopFailed { status: Status, message: String },
}

// ---------------------------------------------------------------------------
// Resolved symbols
// ---------------------------------------------------------------------------

struct Symbols {
    force_configure: ForceConfigureFn,
    #[allow(dead_code)]
    is_initialized: IsInitializedFn,
    create_context: CreateContextFn,
    start_context: StartContextFn,
    stop_context: StopContextFn,
    context_is_valid: ContextIsValidFn,
    configure_callback_dispatch_counting_service: ConfigureCallbackDispatchCountingServiceFn,
    create_counter_config: CreateCounterConfigFn,
    destroy_counter_config: DestroyCounterConfigFn,
    iterate_agent_supported_counters: IterateAgentSupportedCountersFn,
    query_counter_info: QueryCounterInfoFn,
    get_status_string: GetStatusStringFn,
    #[allow(dead_code)]
    get_version: GetVersionFn,
}

impl Symbols {
    unsafe fn load(library: &Library) -> Result<Self, RocProfError> {
        // SAFETY: each symbol is resolved from librocprofiler-sdk and typed to
        // the public header ABI transcribed above.
        unsafe {
            Ok(Self {
                force_configure: load_fn(library, b"rocprofiler_force_configure\0")?,
                is_initialized: load_fn(library, b"rocprofiler_is_initialized\0")?,
                create_context: load_fn(library, b"rocprofiler_create_context\0")?,
                start_context: load_fn(library, b"rocprofiler_start_context\0")?,
                stop_context: load_fn(library, b"rocprofiler_stop_context\0")?,
                context_is_valid: load_fn(library, b"rocprofiler_context_is_valid\0")?,
                configure_callback_dispatch_counting_service: load_fn(
                    library,
                    b"rocprofiler_configure_callback_dispatch_counting_service\0",
                )?,
                create_counter_config: load_fn(library, b"rocprofiler_create_counter_config\0")?,
                destroy_counter_config: load_fn(library, b"rocprofiler_destroy_counter_config\0")?,
                iterate_agent_supported_counters: load_fn(
                    library,
                    b"rocprofiler_iterate_agent_supported_counters\0",
                )?,
                query_counter_info: load_fn(library, b"rocprofiler_query_counter_info\0")?,
                get_status_string: load_fn(library, b"rocprofiler_get_status_string\0")?,
                get_version: load_fn(library, b"rocprofiler_get_version\0")?,
            })
        }
    }

    fn status_message(&self, status: Status) -> String {
        // SAFETY: get_status_string returns a library-owned C string or null.
        unsafe {
            let p = (self.get_status_string)(status);
            if p.is_null() {
                format!("status {status}")
            } else {
                CStr::from_ptr(p).to_string_lossy().into_owned()
            }
        }
    }
}

unsafe fn load_fn<T: Copy>(library: &Library, name: &'static [u8]) -> Result<T, RocProfError> {
    // SAFETY: `name` is a static NUL-terminated symbol; cast matches header ABI.
    unsafe {
        let sym = library
            .get::<*mut c_void>(name)
            .map_err(|_| RocProfError::MissingSymbol {
                symbol: cstr_label(name),
            })?;
        Ok(std::mem::transmute_copy(&*sym))
    }
}

fn cstr_label(name: &'static [u8]) -> &'static str {
    // Strip trailing NUL for error messages.
    let end = name.iter().position(|&b| b == 0).unwrap_or(name.len());
    std::str::from_utf8(&name[..end]).unwrap_or("<invalid>")
}

// ---------------------------------------------------------------------------
// Process-global tool registration state (callbacks cannot capture)
// ---------------------------------------------------------------------------

struct SessionInner {
    records: Mutex<Vec<DispatchCount>>,
    /// Cached `rocprofiler_counter_config_id_t` per agent handle.
    config_cache: Mutex<HashMap<u64, CounterConfigId>>,
    counter_name: String,
    context: AtomicU64,
    active: AtomicBool,
    /// First agent/counter/config failure seen in the dispatch callback.
    /// Silent zero-count success is forbidden — surface via query/finish.
    callback_error: Mutex<Option<String>>,
}

/// One completed kernel-dispatch counter sample.
#[derive(Clone, Debug)]
pub struct DispatchCount {
    pub dispatch_id: u64,
    pub kernel_id: u64,
    pub agent_id: u64,
    pub correlation_id: u64,
    pub start_timestamp: u64,
    pub end_timestamp: u64,
    pub counters: Vec<CounterSample>,
}

/// Single counter value within a [`DispatchCount`].
#[derive(Clone, Debug)]
pub struct CounterSample {
    /// Encoded `rocprofiler_counter_instance_id_t`.
    pub instance_id: u64,
    pub value: f64,
    pub dispatch_id: u64,
    pub agent_id: u64,
}

struct ToolGlobals {
    /// Retains the rocprofiler-sdk mapping for the process tool lifetime so
    /// callback fn pointers stay valid even if the creating [`RocProf`] drops.
    library: Arc<Library>,
    symbols: Symbols,
    session: Arc<SessionInner>,
    /// Set by `tool_init` on failure so `start_dispatch_counting` can surface it.
    init_error: Mutex<Option<String>>,
    client_name: *const c_char,
}

// SAFETY: ToolGlobals is only touched under TOOL_SLOT / from rocprofiler threads
// after install; client_name points at static CStr; Arc<Library> is Send+Sync.
unsafe impl Send for ToolGlobals {}
unsafe impl Sync for ToolGlobals {}

static TOOL_SLOT: Mutex<Option<Arc<ToolGlobals>>> = Mutex::new(None);
/// Serializes the full configure → context-create → start sequence so two
/// threads cannot interleave registration while `TOOL_SLOT` is briefly released
/// for re-entrant `tool_globals()` lookups from rocprofiler callbacks.
static START_GATE: Mutex<()> = Mutex::new(());
/// `rocprofiler_force_configure` succeeded at least once in this process.
static FORCE_CONFIGURED: AtomicBool = AtomicBool::new(false);
/// True while a [`DispatchCountSession`] is live (process-wide single session).
static SESSION_LIVE: AtomicBool = AtomicBool::new(false);
static CONFIGURE_RESULT: LazyLock<ToolConfigureResult> = LazyLock::new(|| ToolConfigureResult {
    size: size_of::<ToolConfigureResult>(),
    initialize: Some(tool_init),
    finalize: Some(tool_fini),
    tool_data: ptr::null_mut(),
});

fn tool_globals() -> Option<Arc<ToolGlobals>> {
    TOOL_SLOT.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

fn record_callback_error(session: &SessionInner, message: String) {
    let mut slot = session
        .callback_error
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if slot.is_none() {
        *slot = Some(message);
    }
}

// ---------------------------------------------------------------------------
// C callbacks
// ---------------------------------------------------------------------------

unsafe extern "C" fn available_counters_cb(
    _agent_id: AgentId,
    counters: *mut CounterId,
    num_counters: usize,
    user_data: *mut c_void,
) -> Status {
    if user_data.is_null() || counters.is_null() {
        return STATUS_ERROR;
    }
    // SAFETY: user_data is &mut Vec<CounterId> from iterate call below.
    let out = unsafe { &mut *(user_data as *mut Vec<CounterId>) };
    // SAFETY: counters points at num_counters ids owned by rocprofiler for this CB.
    let slice = unsafe { std::slice::from_raw_parts(counters, num_counters) };
    out.extend_from_slice(slice);
    STATUS_SUCCESS
}

unsafe extern "C" fn dispatch_callback(
    dispatch_data: DispatchCountingServiceData,
    config: *mut CounterConfigId,
    _user_data: *mut UserData,
    _callback_data_args: *mut c_void,
) {
    if config.is_null() {
        return;
    }
    let Some(g) = tool_globals() else {
        return;
    };
    let agent = dispatch_data.dispatch_info.agent_id;

    {
        let cache = g
            .session
            .config_cache
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if let Some(cached) = cache.get(&agent.handle) {
            // SAFETY: config is an out-param from rocprofiler for this dispatch.
            unsafe {
                *config = *cached;
            }
            return;
        }
    }

    let mut gpu_counters: Vec<CounterId> = Vec::new();
    let iter_st = unsafe {
        (g.symbols.iterate_agent_supported_counters)(
            agent,
            Some(available_counters_cb),
            (&raw mut gpu_counters) as *mut c_void,
        )
    };
    if iter_st != STATUS_SUCCESS {
        record_callback_error(
            &g.session,
            format!(
                "iterate_agent_supported_counters agent={}: {} ({})",
                agent.handle,
                iter_st,
                g.symbols.status_message(iter_st)
            ),
        );
        return;
    }

    let want = g.session.counter_name.as_str();
    let mut collect: Vec<CounterId> = Vec::new();
    for counter in gpu_counters {
        let mut info = CounterInfoV0 {
            id: CounterId { handle: 0 },
            name: ptr::null(),
            description: ptr::null(),
            block: ptr::null(),
            expression: ptr::null(),
            flags: 0,
            _pad: [0; 7],
        };
        let qst = unsafe {
            (g.symbols.query_counter_info)(
                counter,
                COUNTER_INFO_VERSION_0,
                (&raw mut info) as *mut c_void,
            )
        };
        if qst != STATUS_SUCCESS || info.name.is_null() {
            continue;
        }
        // SAFETY: name is a rocprofiler-owned NUL-terminated string for this query.
        let name = unsafe { CStr::from_ptr(info.name) };
        if name.to_string_lossy() == want {
            collect.push(counter);
        }
    }
    if collect.is_empty() {
        record_callback_error(
            &g.session,
            format!("counter `{want}` not found for agent {}", agent.handle),
        );
        return;
    }

    let mut profile = CounterConfigId { handle: 0 };
    let cst = unsafe {
        (g.symbols.create_counter_config)(agent, collect.as_mut_ptr(), collect.len(), &mut profile)
    };
    if cst != STATUS_SUCCESS {
        record_callback_error(
            &g.session,
            format!(
                "create_counter_config agent={}: {} ({})",
                agent.handle,
                cst,
                g.symbols.status_message(cst)
            ),
        );
        return;
    }

    {
        let mut cache = g
            .session
            .config_cache
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        cache.insert(agent.handle, profile);
    }
    // SAFETY: out-param for this dispatch.
    unsafe {
        *config = profile;
    }
}

unsafe extern "C" fn record_callback(
    dispatch_data: DispatchCountingServiceData,
    record_data: *mut CounterRecord,
    record_count: usize,
    _user_data: UserData,
    _callback_data_args: *mut c_void,
) {
    let Some(g) = tool_globals() else {
        return;
    };
    if !g.session.active.load(Ordering::Acquire) {
        return;
    }
    let counters = if record_data.is_null() || record_count == 0 {
        Vec::new()
    } else {
        // SAFETY: rocprofiler provides record_count records for this callback.
        let slice = unsafe { std::slice::from_raw_parts(record_data, record_count) };
        slice
            .iter()
            .map(|r| CounterSample {
                instance_id: r.id,
                value: r.counter_value,
                dispatch_id: r.dispatch_id,
                agent_id: r.agent_id.handle,
            })
            .collect()
    };
    let entry = DispatchCount {
        dispatch_id: dispatch_data.dispatch_info.dispatch_id,
        kernel_id: dispatch_data.dispatch_info.kernel_id,
        agent_id: dispatch_data.dispatch_info.agent_id.handle,
        correlation_id: dispatch_data.correlation_id.internal,
        start_timestamp: dispatch_data.start_timestamp,
        end_timestamp: dispatch_data.end_timestamp,
        counters,
    };
    g.session
        .records
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push(entry);
}

unsafe extern "C" fn tool_init(
    _finalize_func: Option<ClientFinalizeFn>,
    _tool_data: *mut c_void,
) -> c_int {
    let Some(g) = tool_globals() else {
        return -1;
    };

    let mut ctx = ContextId { handle: 0 };
    let st = unsafe { (g.symbols.create_context)(&mut ctx) };
    if st != STATUS_SUCCESS {
        let msg = g.symbols.status_message(st);
        *g.init_error.lock().unwrap_or_else(|e| e.into_inner()) =
            Some(format!("create_context: {st} ({msg})"));
        return -1;
    }

    let mut valid: c_int = 0;
    let vst = unsafe { (g.symbols.context_is_valid)(ctx, &mut valid) };
    // Header sample: nonzero valid flag means invalid context.
    if vst != STATUS_SUCCESS || valid != 0 {
        *g.init_error.lock().unwrap_or_else(|e| e.into_inner()) = Some(format!(
            "context_is_valid rejected ctx (status={vst}, flag={valid})"
        ));
        return -1;
    }

    let cst = unsafe {
        (g.symbols.configure_callback_dispatch_counting_service)(
            ctx,
            Some(dispatch_callback),
            ptr::null_mut(),
            Some(record_callback),
            ptr::null_mut(),
        )
    };
    if cst != STATUS_SUCCESS {
        let msg = g.symbols.status_message(cst);
        *g.init_error.lock().unwrap_or_else(|e| e.into_inner()) = Some(format!(
            "configure_callback_dispatch_counting_service: {cst} ({msg})"
        ));
        return -1;
    }

    let sst = unsafe { (g.symbols.start_context)(ctx) };
    if sst != STATUS_SUCCESS {
        let msg = g.symbols.status_message(sst);
        *g.init_error.lock().unwrap_or_else(|e| e.into_inner()) =
            Some(format!("start_context: {sst} ({msg})"));
        return -1;
    }

    g.session.context.store(ctx.handle, Ordering::Release);
    g.session.active.store(true, Ordering::Release);
    0
}

unsafe extern "C" fn tool_fini(_tool_data: *mut c_void) {
    if let Some(g) = tool_globals() {
        let handle = g.session.context.load(Ordering::Acquire);
        if handle != 0 {
            let ctx = ContextId { handle };
            unsafe {
                let _ = (g.symbols.stop_context)(ctx);
            }
        }
        g.session.active.store(false, Ordering::Release);
    }
}

unsafe extern "C" fn redline_rocprofiler_configure(
    _version: u32,
    _runtime_version: *const c_char,
    _priority: u32,
    client_id: *mut ClientId,
) -> *mut ToolConfigureResult {
    if let Some(g) = tool_globals() {
        if !client_id.is_null() {
            // SAFETY: client_id is provided by rocprofiler during configure.
            unsafe {
                (*client_id).name = g.client_name;
            }
        }
    }
    // Stable address for the process lifetime of LazyLock.
    &*CONFIGURE_RESULT as *const ToolConfigureResult as *mut ToolConfigureResult
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Dynamically loaded `librocprofiler-sdk.so.1` entry points.
///
/// # Process-wide single-registration constraint
///
/// rocprofiler-sdk tool registration (`rocprofiler_force_configure`) is
/// process-global and irreversible for the life of the process. At most one
/// [`DispatchCountSession`] may be live at a time. Starting a second session
/// while another is active returns [`RocProfError::SessionAlreadyActive`].
/// After the active session is dropped (which stops the context), a later
/// start may re-`start_context` on the already-registered tool.
///
/// The shared library mapping is held in an [`Arc`] shared with any live
/// session / tool globals so dropping `RocProf` cannot unload symbols while
/// callbacks or sessions still hold function pointers.
pub struct RocProf {
    library: Arc<Library>,
    /// Kept so session Drop can stop the context without re-resolving.
    stop_context: StopContextFn,
    start_context: StartContextFn,
    force_configure: ForceConfigureFn,
    get_status_string: GetStatusStringFn,
    /// Full symbol table retained for the first configure path.
    symbols: Symbols,
}

impl std::fmt::Debug for RocProf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RocProf").finish_non_exhaustive()
    }
}

impl RocProf {
    /// Load `librocprofiler-sdk.so.1` and resolve the registration + counting API.
    pub fn load() -> Result<Self, RocProfError> {
        let (library, _tried, _failures) = open_library()?;
        let symbols = unsafe { Symbols::load(&library) }?;

        Ok(Self {
            stop_context: symbols.stop_context,
            start_context: symbols.start_context,
            force_configure: symbols.force_configure,
            get_status_string: symbols.get_status_string,
            symbols,
            library,
        })
    }

    /// Start per-dispatch hardware-counter collection via the callback service.
    ///
    /// Uses `rocprofiler_force_configure` for in-process registration (the
    /// library has no initialize/finalize symbols). The first call in a process
    /// performs full tool registration; after the session is dropped, a later
    /// call may re-`start_context` on the existing tool context when
    /// force_configure is already locked.
    ///
    /// # Errors
    /// - [`RocProfError::SessionAlreadyActive`] if another session is live
    /// - tool / context / configure failures as named variants
    ///
    /// Default counter: [`DEFAULT_DISPATCH_COUNTER`] (`SQ_WAVES`).
    pub fn start_dispatch_counting(&self) -> Result<DispatchCountSession, RocProfError> {
        self.start_dispatch_counting_with_counter(DEFAULT_DISPATCH_COUNTER)
    }

    /// Like [`Self::start_dispatch_counting`] but selects `counter_name`.
    pub fn start_dispatch_counting_with_counter(
        &self,
        counter_name: &str,
    ) -> Result<DispatchCountSession, RocProfError> {
        // Hold the start gate across the entire registration / restart path so
        // concurrent callers cannot interleave mid-configure even though
        // TOOL_SLOT is released around force_configure (callbacks re-lock it).
        let _start_gate = START_GATE.lock().unwrap_or_else(|e| e.into_inner());

        // Process-wide single-session gate (re-check under START_GATE).
        let mut slot = TOOL_SLOT.lock().unwrap_or_else(|e| e.into_inner());
        if SESSION_LIVE.load(Ordering::Acquire) {
            return Err(RocProfError::SessionAlreadyActive);
        }

        let session_inner = Arc::new(SessionInner {
            records: Mutex::new(Vec::new()),
            config_cache: Mutex::new(HashMap::new()),
            counter_name: counter_name.to_owned(),
            context: AtomicU64::new(0),
            active: AtomicBool::new(false),
            callback_error: Mutex::new(None),
        });

        if FORCE_CONFIGURED.load(Ordering::Acquire) {
            // Re-use process registration: require an existing context handle.
            let existing = slot.as_ref().ok_or(RocProfError::ConfigurationLocked)?;
            let handle = existing.session.context.load(Ordering::Acquire);
            if handle == 0 {
                return Err(RocProfError::ConfigurationLocked);
            }
            let client_name = existing.client_name;
            let library = Arc::clone(&existing.library);
            session_inner.context.store(handle, Ordering::Release);
            *slot = Some(Arc::new(ToolGlobals {
                library: Arc::clone(&library),
                symbols: clone_symbols(&self.symbols),
                session: Arc::clone(&session_inner),
                init_error: Mutex::new(None),
                client_name,
            }));
            // Release TOOL_SLOT before start_context; START_GATE still held.
            drop(slot);

            let ctx = ContextId { handle };
            let st = unsafe { (self.start_context)(ctx) };
            if st != STATUS_SUCCESS {
                // Failed to go live — clear any partial install.
                SESSION_LIVE.store(false, Ordering::Release);
                return Err(RocProfError::ContextFailed {
                    status: st,
                    message: self.status_message(st),
                });
            }
            session_inner.active.store(true, Ordering::Release);
            SESSION_LIVE.store(true, Ordering::Release);
            return Ok(DispatchCountSession {
                inner: session_inner,
                library,
                stop_context: self.stop_context,
                destroy_counter_config: self.symbols.destroy_counter_config,
                get_status_string: self.get_status_string,
                stopped: false,
            });
        }

        // First registration in this process.
        let globals = Arc::new(ToolGlobals {
            library: Arc::clone(&self.library),
            symbols: clone_symbols(&self.symbols),
            session: Arc::clone(&session_inner),
            init_error: Mutex::new(None),
            client_name: c"redline-observe".as_ptr(),
        });
        *slot = Some(Arc::clone(&globals));
        // Release TOOL_SLOT so force_configure → tool_init can re-lock via
        // tool_globals(); START_GATE keeps concurrent starts out.
        drop(slot);

        let st = unsafe { (self.force_configure)(Some(redline_rocprofiler_configure)) };
        if st == STATUS_ERROR_CONFIGURATION_LOCKED {
            let _ = TOOL_SLOT.lock().map(|mut s| s.take());
            return Err(RocProfError::ConfigurationLocked);
        }
        if st != STATUS_SUCCESS {
            let err = globals
                .init_error
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clone();
            let _ = TOOL_SLOT.lock().map(|mut s| s.take());
            return Err(RocProfError::Api {
                status: st,
                message: err.unwrap_or_else(|| self.status_message(st)),
            });
        }

        FORCE_CONFIGURED.store(true, Ordering::Release);

        if let Some(err) = globals
            .init_error
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
        {
            return Err(RocProfError::ToolInitFailed(err));
        }
        if session_inner.context.load(Ordering::Acquire) == 0 {
            return Err(RocProfError::ToolInitFailed(
                "tool_init did not publish a context id".into(),
            ));
        }

        SESSION_LIVE.store(true, Ordering::Release);
        Ok(DispatchCountSession {
            inner: session_inner,
            library: Arc::clone(&self.library),
            stop_context: self.stop_context,
            destroy_counter_config: self.symbols.destroy_counter_config,
            get_status_string: self.get_status_string,
            stopped: false,
        })
    }

    fn status_message(&self, status: Status) -> String {
        // SAFETY: library-owned C string or null.
        unsafe {
            let p = (self.get_status_string)(status);
            if p.is_null() {
                format!("status {status}")
            } else {
                CStr::from_ptr(p).to_string_lossy().into_owned()
            }
        }
    }
}

fn clone_symbols(s: &Symbols) -> Symbols {
    Symbols {
        force_configure: s.force_configure,
        is_initialized: s.is_initialized,
        create_context: s.create_context,
        start_context: s.start_context,
        stop_context: s.stop_context,
        context_is_valid: s.context_is_valid,
        configure_callback_dispatch_counting_service: s
            .configure_callback_dispatch_counting_service,
        create_counter_config: s.create_counter_config,
        destroy_counter_config: s.destroy_counter_config,
        iterate_agent_supported_counters: s.iterate_agent_supported_counters,
        query_counter_info: s.query_counter_info,
        get_status_string: s.get_status_string,
        get_version: s.get_version,
    }
}

/// Active dispatch-counting session. Stops the rocprofiler context on [`Drop`].
///
/// Holds an [`Arc<Library>`] so the rocprofiler-sdk mapping cannot be unloaded
/// while this session (or its stop path) still needs resolved symbols.
///
/// Process-wide: only one session may exist at a time (see [`RocProf`]).
pub struct DispatchCountSession {
    inner: Arc<SessionInner>,
    /// Pins librocprofiler-sdk for the session lifetime.
    #[allow(dead_code)]
    library: Arc<Library>,
    stop_context: StopContextFn,
    destroy_counter_config: DestroyCounterConfigFn,
    get_status_string: GetStatusStringFn,
    stopped: bool,
}

impl DispatchCountSession {
    /// Snapshot of per-dispatch counter records collected so far.
    ///
    /// # Errors
    /// Returns [`RocProfError::DispatchConfigFailed`] when the dispatch
    /// callback recorded a counter/agent/config failure (avoids silent zeros).
    pub fn query(&self) -> Result<Vec<DispatchCount>, RocProfError> {
        self.ensure_no_callback_error()?;
        Ok(self
            .inner
            .records
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone())
    }

    /// Number of completed dispatch records.
    ///
    /// # Errors
    /// Same as [`Self::query`] when a callback configuration error was recorded.
    pub fn len(&self) -> Result<usize, RocProfError> {
        self.ensure_no_callback_error()?;
        Ok(self
            .inner
            .records
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .len())
    }

    pub fn is_empty(&self) -> Result<bool, RocProfError> {
        Ok(self.len()? == 0)
    }

    /// Stop collection and return records (idempotent stop).
    ///
    /// Surfaces stop-context failures and any dispatch-callback configuration
    /// error after stopping.
    pub fn finish(&mut self) -> Result<Vec<DispatchCount>, RocProfError> {
        self.stop()?;
        self.query()
    }

    /// Stop collection without dropping (idempotent).
    ///
    /// Destroys cached counter configs after the context is stopped.
    pub fn stop(&mut self) -> Result<(), RocProfError> {
        if self.stopped {
            return self.ensure_no_callback_error();
        }
        let handle = self.inner.context.load(Ordering::Acquire);
        let mut stop_err: Option<RocProfError> = None;
        if handle != 0 {
            let ctx = ContextId { handle };
            let st = unsafe { (self.stop_context)(ctx) };
            if st != STATUS_SUCCESS {
                stop_err = Some(RocProfError::StopFailed {
                    status: st,
                    message: self.status_message(st),
                });
            }
        }
        // Destroy cached counter configs after context stop (resource lifecycle).
        {
            let mut cache = self
                .inner
                .config_cache
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            for (_agent, cfg) in cache.drain() {
                if cfg.handle != 0 {
                    let _ = unsafe { (self.destroy_counter_config)(cfg) };
                }
            }
        }
        self.inner.active.store(false, Ordering::Release);
        self.stopped = true;
        SESSION_LIVE.store(false, Ordering::Release);

        if let Some(err) = stop_err {
            return Err(err);
        }
        self.ensure_no_callback_error()
    }

    /// Context handle published by tool init (0 if inactive).
    pub fn context_handle(&self) -> u64 {
        self.inner.context.load(Ordering::Acquire)
    }

    fn ensure_no_callback_error(&self) -> Result<(), RocProfError> {
        if let Some(msg) = self
            .inner
            .callback_error
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
        {
            return Err(RocProfError::DispatchConfigFailed(msg));
        }
        Ok(())
    }

    fn status_message(&self, status: Status) -> String {
        // SAFETY: library-owned C string or null.
        unsafe {
            let p = (self.get_status_string)(status);
            if p.is_null() {
                format!("status {status}")
            } else {
                CStr::from_ptr(p).to_string_lossy().into_owned()
            }
        }
    }
}

impl Drop for DispatchCountSession {
    fn drop(&mut self) {
        // Best-effort stop on drop; prefer `finish`/`stop` to observe errors.
        let _ = self.stop();
    }
}

impl std::fmt::Debug for DispatchCountSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DispatchCountSession")
            .field("context", &self.context_handle())
            .field(
                "records",
                &self
                    .inner
                    .records
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .len(),
            )
            .field("stopped", &self.stopped)
            .finish()
    }
}

fn open_library() -> Result<(Arc<Library>, String, String), RocProfError> {
    if let Ok(path) = std::env::var(REDLINE_ROCPROFILER_LIB) {
        // SAFETY: loading a user-specified path for tests / override.
        return match unsafe { Library::new(&path) } {
            Ok(lib) => Ok((Arc::new(lib), path.clone(), String::new())),
            Err(err) => Err(RocProfError::LibraryNotFound {
                candidates: path,
                detail: err.to_string(),
            }),
        };
    }

    let mut failures = Vec::new();
    for candidate in CANDIDATES {
        // SAFETY: loading the installed rocprofiler-sdk is the purpose of this module.
        match unsafe { Library::new(candidate) } {
            Ok(lib) => {
                return Ok((Arc::new(lib), CANDIDATES.join(", "), failures.join("; ")));
            }
            Err(err) => failures.push(format!("{candidate}: {err}")),
        }
    }
    Err(RocProfError::LibraryNotFound {
        candidates: CANDIDATES.join(", "),
        detail: failures.join("; "),
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_matches_measured_sdk_1_3_2() {
        assert_eq!(size_of::<ContextId>(), 8);
        assert_eq!(size_of::<DispatchCountingServiceData>(), 168);
        assert_eq!(size_of::<KernelDispatchInfo>(), 128);
        assert_eq!(size_of::<CounterRecord>(), 40);
        assert_eq!(size_of::<CounterInfoV0>(), 48);
        assert_eq!(size_of::<ToolConfigureResult>(), 32);
        assert_eq!(size_of::<ClientId>(), 24);
    }

    #[test]
    fn queue_interposition_env_name() {
        assert_eq!(
            ROCPROFILER_QUEUE_INTERPOSITION,
            "ROCPROFILER_QUEUE_INTERPOSITION"
        );
    }

    #[test]
    fn load_missing_library_is_named_error() {
        // SAFETY: test-only env override; restored below.
        unsafe {
            std::env::set_var(
                REDLINE_ROCPROFILER_LIB,
                "/nonexistent/librocprofiler-sdk.so.1",
            );
        }
        let err = RocProf::load().expect_err("must fail on missing lib");
        // SAFETY: clear override so other tests see real candidates.
        unsafe {
            std::env::remove_var(REDLINE_ROCPROFILER_LIB);
        }
        match err {
            RocProfError::LibraryNotFound { ref candidates, .. } => {
                assert!(
                    candidates.contains("nonexistent") || candidates.contains("librocprofiler"),
                    "candidates={candidates}"
                );
                let msg = err.to_string();
                assert!(
                    msg.contains("requires ROCm >= 7.14"),
                    "error must name 7.14 requirement: {msg}"
                );
            }
            other => panic!("expected LibraryNotFound, got {other:?}"),
        }
    }

    #[test]
    fn missing_symbol_error_names_714() {
        let err = RocProfError::MissingSymbol {
            symbol: "rocprofiler_force_configure",
        };
        let msg = err.to_string();
        assert!(msg.contains("requires ROCm >= 7.14"), "{msg}");
        assert!(msg.contains("rocprofiler_force_configure"), "{msg}");
    }
}
