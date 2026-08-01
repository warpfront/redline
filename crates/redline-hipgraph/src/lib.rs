// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! A hipGraph-compatible C ABI backed by Redline's retained PM4 replay engine.
//!
//! The same shared object works as a link-time library and as an LD_PRELOAD
//! interposer. Graphs whose kernels come from `hipModuleLoadData*` or a
//! compiler-registered static fat binary use Redline. In interposer mode a
//! native HIP shadow graph remains the correctness fallback for unregistered
//! kernels, unsupported code objects, and devices outside Redline's PM4 families.

mod abi;
mod metadata;

pub use abi::{
    dim3, hipError_t, hipFunction_t, hipGraph_t, hipGraphExec_t, hipGraphNode_t,
    hipKernelNodeParams, hipModule_t, hipStream_t,
};

use std::collections::{BTreeMap, HashMap, HashSet};
use std::ffi::{CStr, c_char, c_void};
use std::fmt;
use std::ptr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, Mutex, MutexGuard, OnceLock};

use abi::*;
use metadata::{
    KernargLayout, bundle_debug_info, copy_code_object_image, kernarg_layout,
    select_bundle_code_object,
};
#[cfg(feature = "python")]
use pyo3::exceptions::{PyOverflowError, PyRuntimeError, PyValueError};
#[cfg(feature = "python")]
use pyo3::prelude::*;
use redline_dispatch::aql::{NodeDispatch, Pm4GraphReplay, lower_plan_to_pm4_ib};
use redline_dispatch::hipgraph::{Graph, GraphExec};
use redline_dispatch::{Dim3, KernelLaunch, NodeId};
use redline_rocr::{Executable, GpuDevice, GpuSelector, KernargPool, Runtime, load_symbols};

#[derive(Clone)]
struct ModuleRecord {
    executable: Executable,
    code: Arc<[u8]>,
    native: bool,
}

#[derive(Clone)]
struct FunctionRecord {
    executable: Executable,
    symbol: String,
    layout: KernargLayout,
    module: usize,
}

const HIP_FATBIN_MAGIC: i32 = 0x4849_5046;

/// ROCm's compiler-emitted `__fatBinC_Wrapper_t`.
#[repr(C)]
#[derive(Clone, Copy)]
struct HipFatBinCWrapper {
    magic: i32,
    version: i32,
    binary: *const c_void,
    unused: *const c_void,
}

/// Read the compiler wrapper by its C ABI offsets.
///
/// HIP emits this object in `.hipFatBinSegment`. Although current toolchains
/// align that section for pointers, use unaligned field reads so registration
/// does not depend on the ELF section alignment chosen by the producer.
///
/// # Safety
///
/// `data` must be null or point to a readable 24-byte HIP fatbin wrapper.
unsafe fn read_hip_fatbin_wrapper(data: *const c_void) -> Option<HipFatBinCWrapper> {
    if data.is_null() {
        return None;
    }
    let bytes = data.cast::<u8>();
    // SAFETY: the caller guarantees a complete wrapper; `read_unaligned`
    // preserves the C offsets without imposing Rust reference alignment.
    Some(HipFatBinCWrapper {
        magic: unsafe { ptr::read_unaligned(bytes.cast::<i32>()) },
        version: unsafe { ptr::read_unaligned(bytes.add(4).cast::<i32>()) },
        binary: unsafe { ptr::read_unaligned(bytes.add(8).cast::<*const c_void>()) },
        unused: unsafe { ptr::read_unaligned(bytes.add(16).cast::<*const c_void>()) },
    })
}

struct FatBinaryRecord {
    bundle: Arc<[u8]>,
    module: OnceLock<Option<ModuleRecord>>,
}

struct StaticFunctionRecord {
    fatbin: usize,
    symbol: String,
    resolved: OnceLock<Option<FunctionRecord>>,
}

#[derive(Clone)]
struct NodeMeta {
    executable: Executable,
    symbol: String,
    kernargs: Vec<u8>,
    grid: [u32; 3],
    block: [u16; 3],
    dyn_group: u32,
}

struct GraphState {
    graph: Graph,
    node_meta: BTreeMap<NodeId, NodeMeta>,
    node_handles: Vec<usize>,
    native_graph: usize,
    force_native: bool,
}

struct GraphHandle {
    state: Mutex<GraphState>,
}

#[derive(Clone, Copy)]
struct NodeHandle {
    owner: usize,
    node: Option<NodeId>,
    native_node: usize,
}

struct ExecState {
    exec: Option<GraphExec>,
    replay: Option<Pm4GraphReplay>,
    dirty: bool,
    node_meta: BTreeMap<NodeId, NodeMeta>,
    nodes: HashMap<usize, NodeId>,
    native_nodes: HashMap<usize, usize>,
    native_exec: usize,
    force_native: bool,
}

struct ExecHandle {
    state: Mutex<ExecState>,
}

#[derive(Clone, Copy)]
struct CaptureState {
    graph: usize,
    last_node: usize,
    invalid: bool,
    native_active: bool,
}

#[derive(Default)]
struct HandleSets {
    graphs: HashSet<usize>,
    nodes: HashSet<usize>,
    execs: HashSet<usize>,
    owned_modules: HashSet<usize>,
    owned_functions: HashSet<usize>,
}

#[derive(Default)]
struct Global {
    modules: Mutex<HashMap<usize, ModuleRecord>>,
    functions: Mutex<HashMap<usize, FunctionRecord>>,
    fatbins: Mutex<HashMap<usize, Arc<FatBinaryRecord>>>,
    static_functions: Mutex<HashMap<usize, Arc<StaticFunctionRecord>>>,
    handles: Mutex<HandleSets>,
    captures: Mutex<HashMap<usize, CaptureState>>,
}

struct RuntimeState {
    _runtime: Runtime,
    device: GpuDevice,
    pool: KernargPool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SymbolResolution {
    Handle,
    Next,
    Dlvsym,
    Missing,
}

impl fmt::Display for SymbolResolution {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Handle => "handle",
            Self::Next => "next",
            Self::Dlvsym => "dlvsym",
            Self::Missing => "false",
        })
    }
}

#[derive(Clone, Copy)]
struct ResolvedSymbol {
    address: usize,
    resolution: SymbolResolution,
}

static LIBAMDHIP64_HANDLE: LazyLock<usize> = LazyLock::new(open_libamdhip64);
static REAL_SYMBOLS: LazyLock<Mutex<HashMap<&'static [u8], ResolvedSymbol>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static GLOBAL: LazyLock<Global> = LazyLock::new(Global::default);
static RUNTIME: OnceLock<Result<RuntimeState, hipError_t>> = OnceLock::new();
static HG_DEBUG_ENABLED: LazyLock<bool> =
    LazyLock::new(|| std::env::var_os("REDLINE_HG_DEBUG").is_some());
static FATBIN_REGISTRATION_COUNT: AtomicU64 = AtomicU64::new(0);

fn global() -> &'static Global {
    &GLOBAL
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn hg_debug_enabled() -> bool {
    *HG_DEBUG_ENABLED
}

fn hg_debug(args: fmt::Arguments<'_>) {
    if hg_debug_enabled() {
        eprintln!("redline-hg: {args}");
    }
}

macro_rules! hgdbg {
    ($($arg:tt)*) => {
        if hg_debug_enabled() {
            hg_debug(format_args!($($arg)*));
        }
    };
}

fn runtime() -> Result<&'static RuntimeState, hipError_t> {
    match RUNTIME.get_or_init(|| {
        let symbols = load_symbols().map_err(|_| hipErrorNotInitialized)?;
        let runtime = Runtime::initialize(symbols).map_err(|_| hipErrorNotInitialized)?;
        // ROCr applies ROCR_VISIBLE_DEVICES before agent enumeration, so ordinal
        // zero is the first device in the caller's visible-device namespace.
        let device = runtime
            .select_gpu(GpuSelector::Ordinal(0))
            .map_err(|_| hipErrorNotInitialized)?;
        let pool = KernargPool::discover(&device).map_err(|_| hipErrorNotInitialized)?;
        Ok(RuntimeState {
            _runtime: runtime,
            device,
            pool,
        })
    }) {
        Ok(runtime) => Ok(runtime),
        Err(status) => Err(*status),
    }
}

const HIP_SYMBOL_VERSIONS: &[&[u8]] = &[
    b"hip_7.1\0",
    b"hip_7.0\0",
    b"hip_6.5\0",
    b"hip_6.2\0",
    b"hip_6.1\0",
    b"hip_5.3\0",
    b"hip_5.2\0",
    b"hip_4.5\0",
    b"hip_4.4\0",
    b"hip_4.3\0",
    b"hip_4.2\0",
];

/// Resolve a definition without introducing a link-time dependency on
/// libamdhip64. `name` must be NUL-terminated.
///
/// A specific libamdhip64 handle is required when Python/ctypes loaded HIP with
/// `RTLD_LOCAL`, which excludes the library from the `RTLD_NEXT` search scope.
unsafe fn real_symbol_with_resolution<T: Copy>(
    name: &'static [u8],
) -> (Option<T>, SymbolResolution) {
    debug_assert_eq!(name.last(), Some(&0));
    let resolved = unsafe { resolve_real_symbol(name) };
    if resolved.address == 0 {
        return (None, resolved.resolution);
    }

    debug_assert_eq!(std::mem::size_of::<T>(), std::mem::size_of::<*mut c_void>());
    let address = resolved.address as *mut c_void;
    (
        Some(unsafe { std::mem::transmute_copy(&address) }),
        resolved.resolution,
    )
}

unsafe fn real_symbol<T: Copy>(name: &'static [u8]) -> Option<T> {
    unsafe { real_symbol_with_resolution(name) }.0
}

fn open_libamdhip64() -> usize {
    let mut handle = unsafe {
        libc::dlopen(
            b"libamdhip64.so\0".as_ptr().cast(),
            libc::RTLD_NOW | libc::RTLD_NOLOAD,
        )
    };
    if handle.is_null() {
        handle = unsafe {
            libc::dlopen(
                b"libamdhip64.so.7\0".as_ptr().cast(),
                libc::RTLD_NOW | libc::RTLD_NOLOAD,
            )
        };
    }
    if handle.is_null() {
        handle = unsafe {
            libc::dlopen(
                b"libamdhip64.so\0".as_ptr().cast(),
                libc::RTLD_NOW | libc::RTLD_GLOBAL,
            )
        };
    }
    handle as usize
}

fn libamdhip64_handle() -> *mut c_void {
    *LIBAMDHIP64_HANDLE as *mut c_void
}

unsafe fn resolve_real_symbol(name: &'static [u8]) -> ResolvedSymbol {
    if let Some(resolved) = lock(&REAL_SYMBOLS).get(name).copied() {
        return resolved;
    }

    let resolved = unsafe { resolve_real_symbol_uncached(name) };
    *lock(&REAL_SYMBOLS).entry(name).or_insert(resolved)
}

unsafe fn resolve_real_symbol_uncached(name: &'static [u8]) -> ResolvedSymbol {
    let handle = libamdhip64_handle();
    if !handle.is_null() {
        let address = unsafe { libc::dlsym(handle, name.as_ptr().cast()) };
        if !address.is_null() {
            return ResolvedSymbol {
                address: address as usize,
                resolution: SymbolResolution::Handle,
            };
        }
        for version in HIP_SYMBOL_VERSIONS {
            let address =
                unsafe { libc::dlvsym(handle, name.as_ptr().cast(), version.as_ptr().cast()) };
            if !address.is_null() {
                return ResolvedSymbol {
                    address: address as usize,
                    resolution: SymbolResolution::Dlvsym,
                };
            }
        }
    }

    let address = unsafe { libc::dlsym(libc::RTLD_NEXT, name.as_ptr().cast()) };
    if !address.is_null() {
        return ResolvedSymbol {
            address: address as usize,
            resolution: SymbolResolution::Next,
        };
    }
    for version in HIP_SYMBOL_VERSIONS {
        let address = unsafe {
            libc::dlvsym(
                libc::RTLD_NEXT,
                name.as_ptr().cast(),
                version.as_ptr().cast(),
            )
        };
        if !address.is_null() {
            return ResolvedSymbol {
                address: address as usize,
                resolution: SymbolResolution::Dlvsym,
            };
        }
    }

    ResolvedSymbol {
        address: 0,
        resolution: SymbolResolution::Missing,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __hipRegisterFatBinary(data: *const c_void) -> *mut *mut c_void {
    type Function = unsafe extern "C" fn(*const c_void) -> *mut *mut c_void;
    let call = FATBIN_REGISTRATION_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
    // SAFETY: HIP's registration ABI supplies a compiler-emitted wrapper.
    let wrapper = unsafe { read_hip_fatbin_wrapper(data) };
    let (real, resolution) =
        unsafe { real_symbol_with_resolution::<Function>(b"__hipRegisterFatBinary\0") };
    let modules = match real {
        Some(function) => unsafe { function(data) },
        None => ptr::null_mut(),
    };

    let mut captured = false;
    if let Some(wrapper) =
        wrapper.filter(|wrapper| wrapper.magic == HIP_FATBIN_MAGIC && !modules.is_null())
    {
        // SAFETY: a valid HIP wrapper's binary field points at its clang bundle.
        if let Some(bundle) = unsafe { copy_code_object_image(wrapper.binary) } {
            lock(&global().fatbins).insert(
                modules as usize,
                Arc::new(FatBinaryRecord {
                    bundle,
                    module: OnceLock::new(),
                }),
            );
            captured = true;
        }
    }

    let (magic, version, binary) = wrapper
        .map(|wrapper| (wrapper.magic as u32, wrapper.version, wrapper.binary))
        .unwrap_or((0, 0, ptr::null()));
    let bundles = lock(&global().fatbins).len();
    hgdbg!(
        "__hipRegisterFatBinary data={data:p} magic=0x{magic:08x} version={version} binary={binary:p} call={call} real_symbol={resolution} handle={modules:p} captured={captured} bundles={bundles}"
    );
    modules
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __hipRegisterFunction(
    modules: *mut *mut c_void,
    host_function: *const c_void,
    device_function: *mut c_char,
    device_name: *const c_char,
    thread_limit: u32,
    tid: *mut dim3,
    bid: *mut dim3,
    block_dim: *mut dim3,
    grid_dim: *mut dim3,
    wsize: *mut i32,
) {
    type Function = unsafe extern "C" fn(
        *mut *mut c_void,
        *const c_void,
        *mut c_char,
        *const c_char,
        u32,
        *mut dim3,
        *mut dim3,
        *mut dim3,
        *mut dim3,
        *mut i32,
    );
    let Some(function) = (unsafe { real_symbol::<Function>(b"__hipRegisterFunction\0") }) else {
        return;
    };
    unsafe {
        function(
            modules,
            host_function,
            device_function,
            device_name,
            thread_limit,
            tid,
            bid,
            block_dim,
            grid_dim,
            wsize,
        )
    };
    if modules.is_null() || host_function.is_null() || device_name.is_null() {
        return;
    }
    let Ok(symbol) = (unsafe { CStr::from_ptr(device_name) }).to_str() else {
        return;
    };
    lock(&global().static_functions).insert(
        host_function as usize,
        Arc::new(StaticFunctionRecord {
            fatbin: modules as usize,
            symbol: symbol.to_owned(),
            resolved: OnceLock::new(),
        }),
    );
    hgdbg!("__hipRegisterFunction host_function={host_function:p} device_name={symbol}");
}

fn is_graph(handle: hipGraph_t) -> bool {
    !handle.is_null() && lock(&global().handles).graphs.contains(&(handle as usize))
}

fn is_exec(handle: hipGraphExec_t) -> bool {
    !handle.is_null() && lock(&global().handles).execs.contains(&(handle as usize))
}

fn node_snapshot(handle: hipGraphNode_t) -> Option<NodeHandle> {
    if handle.is_null() || !lock(&global().handles).nodes.contains(&(handle as usize)) {
        return None;
    }
    Some(unsafe { *(handle.cast::<NodeHandle>()) })
}

fn graph_handle(handle: hipGraph_t) -> Option<&'static GraphHandle> {
    is_graph(handle).then(|| unsafe { &*handle.cast::<GraphHandle>() })
}

fn exec_handle(handle: hipGraphExec_t) -> Option<&'static ExecHandle> {
    is_exec(handle).then(|| unsafe { &*handle.cast::<ExecHandle>() })
}

fn allocate_graph(native_graph: usize) -> hipGraph_t {
    let handle = Box::into_raw(Box::new(GraphHandle {
        state: Mutex::new(GraphState {
            graph: Graph::new(),
            node_meta: BTreeMap::new(),
            node_handles: Vec::new(),
            native_graph,
            force_native: false,
        }),
    }))
    .cast::<c_void>();
    lock(&global().handles).graphs.insert(handle as usize);
    handle
}

fn allocate_node(
    graph_key: usize,
    state: &mut GraphState,
    node: Option<NodeId>,
    native_node: usize,
) -> hipGraphNode_t {
    let handle = Box::into_raw(Box::new(NodeHandle {
        owner: graph_key,
        node,
        native_node,
    }))
    .cast::<c_void>();
    lock(&global().handles).nodes.insert(handle as usize);
    state.node_handles.push(handle as usize);
    handle
}

fn allocate_exec(state: ExecState) -> hipGraphExec_t {
    let handle = Box::into_raw(Box::new(ExecHandle {
        state: Mutex::new(state),
    }))
    .cast::<c_void>();
    lock(&global().handles).execs.insert(handle as usize);
    handle
}

fn dependency_nodes(
    graph_key: usize,
    dependencies: *const hipGraphNode_t,
    count: usize,
) -> Result<Vec<NodeHandle>, hipError_t> {
    if count == 0 {
        return Ok(Vec::new());
    }
    if dependencies.is_null() {
        return Err(hipErrorInvalidValue);
    }
    let dependencies = unsafe { std::slice::from_raw_parts(dependencies, count) };
    dependencies
        .iter()
        .map(|&handle| {
            let node = node_snapshot(handle).ok_or(hipErrorInvalidHandle)?;
            if node.owner != graph_key {
                return Err(hipErrorInvalidHandle);
            }
            Ok(node)
        })
        .collect()
}

const MAX_KERNARG_BYTES: usize = 1 << 20;

#[derive(Clone, Copy)]
struct PackContext {
    grid_blocks: [u32; 3],
    block: [u32; 3],
    shared_mem: u32,
}

fn hidden_value(kind: &str, context: PackContext) -> u64 {
    match kind {
        "hidden_block_count_x" => u64::from(context.grid_blocks[0]),
        "hidden_block_count_y" => u64::from(context.grid_blocks[1]),
        "hidden_block_count_z" => u64::from(context.grid_blocks[2]),
        "hidden_group_size_x" => u64::from(context.block[0]),
        "hidden_group_size_y" => u64::from(context.block[1]),
        "hidden_group_size_z" => u64::from(context.block[2]),
        "hidden_dynamic_lds_size" => u64::from(context.shared_mem),
        "hidden_grid_dims" => {
            if context.grid_blocks[2] > 1 {
                3
            } else if context.grid_blocks[1] > 1 {
                2
            } else {
                1
            }
        }
        // Global offsets and runtime service pointers are zero for this
        // dedicated-queue graph replay. Grid/block sizes are in the packet.
        _ => 0,
    }
}

unsafe fn packed_extra(extra: *mut *mut c_void, segment_size: usize) -> Option<Vec<u8>> {
    if extra.is_null() {
        return None;
    }
    let mut buffer = ptr::null::<u8>();
    let mut size = None;
    // HIP's extra protocol is key/value pairs terminated by key 3.  Bound the
    // walk so malformed foreign input fails closed rather than running away.
    for pair in 0..16_usize {
        let key = unsafe { *extra.add(pair * 2) } as usize;
        match key {
            1 => buffer = unsafe { *extra.add(pair * 2 + 1) }.cast_const().cast(),
            2 => {
                let size_pointer = unsafe { *extra.add(pair * 2 + 1) }.cast::<usize>();
                if size_pointer.is_null() {
                    return None;
                }
                size = Some(unsafe { *size_pointer });
            }
            3 => break,
            _ => return None,
        }
    }
    let size = size?;
    if size > segment_size || segment_size > MAX_KERNARG_BYTES {
        return None;
    }
    if buffer.is_null() && size != 0 {
        return None;
    }
    let mut packed = vec![0_u8; segment_size];
    if size != 0 {
        unsafe { ptr::copy_nonoverlapping(buffer, packed.as_mut_ptr(), size) };
    }
    Some(packed)
}

unsafe fn pack_kernel_params(
    layout: &KernargLayout,
    kernel_params: *mut *mut c_void,
    extra: *mut *mut c_void,
    context: PackContext,
) -> Result<Vec<u8>, hipError_t> {
    if layout.segment_size > MAX_KERNARG_BYTES {
        return Err(hipErrorInvalidValue);
    }
    if let Some(packed) = unsafe { packed_extra(extra, layout.segment_size) } {
        return Ok(packed);
    }
    let mut packed = vec![0_u8; layout.segment_size];
    let explicit_count = layout
        .fields
        .iter()
        .filter(|field| !field.value_kind.starts_with("hidden_"))
        .count();
    if explicit_count != 0 && kernel_params.is_null() {
        return Err(hipErrorInvalidValue);
    }
    let mut explicit_index = 0_usize;
    for field in &layout.fields {
        let end = field
            .offset
            .checked_add(field.size)
            .ok_or(hipErrorInvalidValue)?;
        if end > packed.len() {
            return Err(hipErrorInvalidValue);
        }
        if field.value_kind.starts_with("hidden_") {
            let value = hidden_value(&field.value_kind, context).to_le_bytes();
            for (index, destination) in packed[field.offset..end].iter_mut().enumerate() {
                *destination = value.get(index).copied().unwrap_or(0);
            }
        } else {
            let source = unsafe { *kernel_params.add(explicit_index) }.cast::<u8>();
            if source.is_null() {
                return Err(hipErrorInvalidValue);
            }
            unsafe {
                ptr::copy_nonoverlapping(source, packed.as_mut_ptr().add(field.offset), field.size)
            };
            explicit_index += 1;
        }
    }
    Ok(packed)
}

fn global_grid(grid_blocks: dim3, block: dim3) -> Result<[u32; 3], hipError_t> {
    let blocks = [grid_blocks.x, grid_blocks.y, grid_blocks.z];
    let threads = [block.x, block.y, block.z];
    let mut grid = [0_u32; 3];
    for axis in 0..3 {
        if blocks[axis] == 0 || threads[axis] == 0 {
            return Err(hipErrorInvalidValue);
        }
        grid[axis] = blocks[axis]
            .checked_mul(threads[axis])
            .ok_or(hipErrorInvalidValue)?;
    }
    Ok(grid)
}

unsafe fn build_node_meta(
    function: &FunctionRecord,
    params: hipKernelNodeParams,
) -> Result<(KernelLaunch, NodeMeta), hipError_t> {
    let grid = global_grid(params.gridDim, params.blockDim)?;
    let block_u16 = [
        u16::try_from(params.blockDim.x).map_err(|_| hipErrorInvalidValue)?,
        u16::try_from(params.blockDim.y).map_err(|_| hipErrorInvalidValue)?,
        u16::try_from(params.blockDim.z).map_err(|_| hipErrorInvalidValue)?,
    ];
    let context = PackContext {
        grid_blocks: [params.gridDim.x, params.gridDim.y, params.gridDim.z],
        block: [params.blockDim.x, params.blockDim.y, params.blockDim.z],
        shared_mem: params.sharedMemBytes,
    };
    let kernargs = unsafe {
        pack_kernel_params(&function.layout, params.kernelParams, params.extra, context)?
    };
    let grid_dim = Dim3::new(grid[0], grid[1], grid[2]).map_err(|_| hipErrorInvalidValue)?;
    let block_dim = Dim3::new(params.blockDim.x, params.blockDim.y, params.blockDim.z)
        .map_err(|_| hipErrorInvalidValue)?;
    let launch = KernelLaunch::new(function.symbol.clone(), grid_dim, block_dim)
        .map_err(|_| hipErrorInvalidValue)?
        .with_dynamic_shared_bytes(params.sharedMemBytes);
    Ok((
        launch,
        NodeMeta {
            executable: function.executable.clone(),
            symbol: function.symbol.clone(),
            kernargs,
            grid,
            block: block_u16,
            dyn_group: params.sharedMemBytes,
        },
    ))
}

fn resolve_module_function(
    module: &ModuleRecord,
    requested: &str,
    module_key: usize,
) -> Option<FunctionRecord> {
    let initial = kernarg_layout(&module.code, requested, 0);
    let candidates = [
        initial.symbol,
        requested.to_owned(),
        format!("{requested}.kd"),
    ];
    let (symbol, kernel) = candidates.into_iter().find_map(|symbol| {
        module
            .executable
            .kernel(&symbol)
            .ok()
            .map(|kernel| (symbol, kernel))
    })?;
    let mut layout = kernarg_layout(
        &module.code,
        requested,
        kernel.metadata().kernarg_segment_size as usize,
    );
    layout.symbol.clone_from(&symbol);
    Some(FunctionRecord {
        executable: module.executable.clone(),
        symbol,
        layout,
        module: module_key,
    })
}

fn resolve_static_function(registration: &StaticFunctionRecord) -> Option<FunctionRecord> {
    let fatbin = lock(&global().fatbins).get(&registration.fatbin).cloned()?;
    let module = fatbin.module.get_or_init(|| {
        let runtime = runtime().ok()?;
        if hg_debug_enabled() {
            if let Some(info) = bundle_debug_info(&fatbin.bundle, runtime.device.name()) {
                hgdbg!(
                    "bundle magic={} version={} entries={} selected={}",
                    info.magic,
                    info.version,
                    info.entries,
                    info.selected.unwrap_or("none")
                );
            } else {
                hgdbg!("bundle magic=other version=0 entries=0 selected=none");
            }
        }
        let image = select_bundle_code_object(&fatbin.bundle, runtime.device.name())?;
        let code = Arc::<[u8]>::from(image);
        load_redline_code_object(code).ok()
    });
    resolve_module_function(module.as_ref()?, &registration.symbol, registration.fatbin)
}

fn function_record(function: hipFunction_t) -> Option<FunctionRecord> {
    let function_key = function as usize;
    if let Some(record) = lock(&global().functions).get(&function_key).cloned() {
        return Some(record);
    }
    let registration = lock(&global().static_functions)
        .get(&function_key)
        .cloned()?;
    registration
        .resolved
        .get_or_init(|| resolve_static_function(&registration))
        .clone()
}

unsafe fn native_graph_create(flags: u32) -> Result<usize, hipError_t> {
    type Function = unsafe extern "C" fn(*mut hipGraph_t, u32) -> hipError_t;
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphCreate\0") }) else {
        return Ok(0);
    };
    let mut graph = ptr::null_mut();
    let status = unsafe { function(&mut graph, flags) };
    if status == hipSuccess {
        Ok(graph as usize)
    } else {
        Err(status)
    }
}

unsafe fn native_graph_destroy(graph: usize) {
    type Function = unsafe extern "C" fn(hipGraph_t) -> hipError_t;
    if graph != 0 {
        if let Some(function) = unsafe { real_symbol::<Function>(b"hipGraphDestroy\0") } {
            let _ = unsafe { function(graph as hipGraph_t) };
        }
    }
}

unsafe fn native_exec_destroy(exec: usize) {
    type Function = unsafe extern "C" fn(hipGraphExec_t) -> hipError_t;
    if exec != 0 {
        if let Some(function) = unsafe { real_symbol::<Function>(b"hipGraphExecDestroy\0") } {
            let _ = unsafe { function(exec as hipGraphExec_t) };
        }
    }
}

unsafe fn native_kernel_node(
    state: &GraphState,
    dependencies: &[NodeHandle],
    params: *const hipKernelNodeParams,
) -> Result<usize, hipError_t> {
    if state.native_graph == 0 {
        return Ok(0);
    }
    type Function = unsafe extern "C" fn(
        *mut hipGraphNode_t,
        hipGraph_t,
        *const hipGraphNode_t,
        usize,
        *const hipKernelNodeParams,
    ) -> hipError_t;
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphAddKernelNode\0") }) else {
        return Ok(0);
    };
    let native_dependencies = dependencies
        .iter()
        .map(|dependency| dependency.native_node as hipGraphNode_t)
        .collect::<Vec<_>>();
    if native_dependencies
        .iter()
        .any(|dependency| dependency.is_null())
    {
        return Ok(0);
    }
    let mut node = ptr::null_mut();
    let status = unsafe {
        function(
            &mut node,
            state.native_graph as hipGraph_t,
            native_dependencies.as_ptr(),
            native_dependencies.len(),
            params,
        )
    };
    if status == hipSuccess {
        Ok(node as usize)
    } else {
        Err(status)
    }
}

unsafe fn add_kernel_node_internal(
    graph: hipGraph_t,
    dependencies: *const hipGraphNode_t,
    dependency_count: usize,
    params: *const hipKernelNodeParams,
    shadow_native: bool,
) -> Result<hipGraphNode_t, hipError_t> {
    if params.is_null() {
        return Err(hipErrorInvalidValue);
    }
    let graph_key = graph as usize;
    let graph_handle = graph_handle(graph).ok_or(hipErrorInvalidHandle)?;
    let dependencies = dependency_nodes(graph_key, dependencies, dependency_count)?;
    let params_value = unsafe { *params };
    let function = function_record(params_value.func);
    let function_resolved = function.is_some();
    let mut state = lock(&graph_handle.state);
    let native_node = if shadow_native {
        unsafe { native_kernel_node(&state, &dependencies, params) }.unwrap_or(0)
    } else {
        0
    };

    let own_dependencies = dependencies
        .iter()
        .map(|dependency| dependency.node)
        .collect::<Option<Vec<_>>>();
    let own_result = match (function, own_dependencies) {
        (Some(function), Some(own_dependencies)) => {
            match unsafe { build_node_meta(&function, params_value) } {
                Ok((launch, meta)) => {
                    let result = if own_dependencies.is_empty() {
                        state.graph.kernel(launch, [])
                    } else {
                        state.graph.kernel_after(launch, [], own_dependencies)
                    };
                    result.map(|node| (node, meta)).ok()
                }
                Err(_) => None,
            }
        }
        _ => None,
    };
    let pm4_appended = own_result.is_some();

    let result = match own_result {
        Some((node, meta)) => {
            state.node_meta.insert(node, meta);
            Ok(allocate_node(
                graph_key,
                &mut state,
                Some(node),
                native_node,
            ))
        }
        None if native_node != 0 => {
            state.force_native = true;
            Ok(allocate_node(graph_key, &mut state, None, native_node))
        }
        None => Err(hipErrorNotSupported),
    };
    if !shadow_native {
        hgdbg!(
            "append_capture_kernel function={:p} function_record={} pm4_node={} appended_nodes={}",
            params_value.func,
            if function_resolved {
                "resolved"
            } else {
                "none"
            },
            if pm4_appended { "appended" } else { "none" },
            state.node_meta.len()
        );
    }
    result
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphCreate(graph: *mut hipGraph_t, flags: u32) -> hipError_t {
    if graph.is_null() {
        return hipErrorInvalidValue;
    }
    if flags != 0 {
        type Function = unsafe extern "C" fn(*mut hipGraph_t, u32) -> hipError_t;
        return match unsafe { real_symbol::<Function>(b"hipGraphCreate\0") } {
            Some(function) => unsafe { function(graph, flags) },
            None => hipErrorInvalidValue,
        };
    }
    let native = unsafe { native_graph_create(flags) }.unwrap_or(0);
    unsafe { *graph = allocate_graph(native) };
    hipSuccess
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphAddKernelNode(
    node: *mut hipGraphNode_t,
    graph: hipGraph_t,
    dependencies: *const hipGraphNode_t,
    dependency_count: usize,
    params: *const hipKernelNodeParams,
) -> hipError_t {
    if node.is_null() {
        return hipErrorInvalidValue;
    }
    if !is_graph(graph) {
        type Function = unsafe extern "C" fn(
            *mut hipGraphNode_t,
            hipGraph_t,
            *const hipGraphNode_t,
            usize,
            *const hipKernelNodeParams,
        ) -> hipError_t;
        return match unsafe { real_symbol::<Function>(b"hipGraphAddKernelNode\0") } {
            Some(function) => unsafe {
                function(node, graph, dependencies, dependency_count, params)
            },
            None => hipErrorInvalidHandle,
        };
    }
    match unsafe { add_kernel_node_internal(graph, dependencies, dependency_count, params, true) } {
        Ok(result) => {
            unsafe { *node = result };
            hipSuccess
        }
        Err(status) => status,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphAddDependencies(
    graph: hipGraph_t,
    from: *const hipGraphNode_t,
    to: *const hipGraphNode_t,
    count: usize,
) -> hipError_t {
    if !is_graph(graph) {
        type Function = unsafe extern "C" fn(
            hipGraph_t,
            *const hipGraphNode_t,
            *const hipGraphNode_t,
            usize,
        ) -> hipError_t;
        return match unsafe { real_symbol::<Function>(b"hipGraphAddDependencies\0") } {
            Some(function) => unsafe { function(graph, from, to, count) },
            None => hipErrorInvalidHandle,
        };
    }
    if count != 0 && (from.is_null() || to.is_null()) {
        return hipErrorInvalidValue;
    }
    let graph_key = graph as usize;
    let graph_handle = graph_handle(graph).expect("owned graph checked");
    let from_slice = if count == 0 {
        &[][..]
    } else {
        unsafe { std::slice::from_raw_parts(from, count) }
    };
    let to_slice = if count == 0 {
        &[][..]
    } else {
        unsafe { std::slice::from_raw_parts(to, count) }
    };
    let mut pairs = Vec::with_capacity(count);
    for (&from, &to) in from_slice.iter().zip(to_slice) {
        let Some(from) = node_snapshot(from) else {
            return hipErrorInvalidHandle;
        };
        let Some(to) = node_snapshot(to) else {
            return hipErrorInvalidHandle;
        };
        if from.owner != graph_key || to.owner != graph_key {
            return hipErrorInvalidHandle;
        }
        pairs.push((from, to));
    }

    let mut state = lock(&graph_handle.state);
    let mut native_ok = false;
    if state.native_graph != 0
        && pairs
            .iter()
            .all(|(from, to)| from.native_node != 0 && to.native_node != 0)
    {
        type Function = unsafe extern "C" fn(
            hipGraph_t,
            *const hipGraphNode_t,
            *const hipGraphNode_t,
            usize,
        ) -> hipError_t;
        if let Some(function) = unsafe { real_symbol::<Function>(b"hipGraphAddDependencies\0") } {
            let native_from = pairs
                .iter()
                .map(|(from, _)| from.native_node as hipGraphNode_t)
                .collect::<Vec<_>>();
            let native_to = pairs
                .iter()
                .map(|(_, to)| to.native_node as hipGraphNode_t)
                .collect::<Vec<_>>();
            native_ok = unsafe {
                function(
                    state.native_graph as hipGraph_t,
                    native_from.as_ptr(),
                    native_to.as_ptr(),
                    count,
                )
            } == hipSuccess;
        }
    }

    let mut own_ok = true;
    for (from, to) in &pairs {
        match (from.node, to.node) {
            (Some(from), Some(to)) => {
                if state.graph.recorder().depends_on(to, from).is_err() {
                    own_ok = false;
                    break;
                }
            }
            _ => own_ok = false,
        }
    }
    if own_ok {
        hipSuccess
    } else if native_ok {
        state.force_native = true;
        hipSuccess
    } else {
        hipErrorNotSupported
    }
}

fn build_pm4_replay(
    plan: &GraphExec,
    node_meta: &BTreeMap<NodeId, NodeMeta>,
) -> Result<Pm4GraphReplay, hipError_t> {
    let runtime = runtime()?;
    let kernels = node_meta
        .iter()
        .map(|(&node, meta)| {
            meta.executable
                .kernel(&meta.symbol)
                .map(|kernel| (node, kernel))
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| hipErrorInvalidHandle)?;
    lower_plan_to_pm4_ib(&runtime.device, &runtime.pool, plan.plan(), |node| {
        let meta = node_meta.get(&node)?;
        let kernel = kernels
            .binary_search_by_key(&node, |(kernel_node, _)| *kernel_node)
            .ok()
            .and_then(|index| kernels.get(index).map(|(_, kernel)| kernel))?;
        Some(NodeDispatch {
            kernel,
            kernargs: &meta.kernargs,
            grid: meta.grid,
            block: meta.block,
            dyn_group: meta.dyn_group,
        })
    })
    .map_err(|_| hipErrorNotSupported)
}

unsafe fn native_instantiate(
    native_graph: usize,
    error_node: *mut hipGraphNode_t,
    log_buffer: *mut c_char,
    buffer_size: usize,
) -> Result<usize, hipError_t> {
    if native_graph == 0 {
        return Ok(0);
    }
    type Function = unsafe extern "C" fn(
        *mut hipGraphExec_t,
        hipGraph_t,
        *mut hipGraphNode_t,
        *mut c_char,
        usize,
    ) -> hipError_t;
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphInstantiate\0") }) else {
        return Ok(0);
    };
    let mut exec = ptr::null_mut();
    let status = unsafe {
        function(
            &mut exec,
            native_graph as hipGraph_t,
            error_node,
            log_buffer,
            buffer_size,
        )
    };
    if status == hipSuccess {
        Ok(exec as usize)
    } else {
        Err(status)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphInstantiate(
    output: *mut hipGraphExec_t,
    graph: hipGraph_t,
    error_node: *mut hipGraphNode_t,
    log_buffer: *mut c_char,
    buffer_size: usize,
) -> hipError_t {
    if output.is_null() {
        return hipErrorInvalidValue;
    }
    if !is_graph(graph) {
        hgdbg!(
            "hipGraphInstantiate graph={graph:p} build_pm4_replay=skipped force_native=n/a native_exec=n/a"
        );
        type Function = unsafe extern "C" fn(
            *mut hipGraphExec_t,
            hipGraph_t,
            *mut hipGraphNode_t,
            *mut c_char,
            usize,
        ) -> hipError_t;
        return match unsafe { real_symbol::<Function>(b"hipGraphInstantiate\0") } {
            Some(function) => unsafe {
                function(output, graph, error_node, log_buffer, buffer_size)
            },
            None => hipErrorInvalidHandle,
        };
    }
    if !error_node.is_null() {
        unsafe { *error_node = ptr::null_mut() };
    }
    let graph_handle = graph_handle(graph).expect("owned graph checked");
    let state = lock(&graph_handle.state);
    let native_exec =
        unsafe { native_instantiate(state.native_graph, ptr::null_mut(), log_buffer, buffer_size) }
            .unwrap_or(0);
    let own_exec = state.graph.instantiate().ok();
    if own_exec.is_none() && native_exec == 0 {
        hgdbg!(
            "hipGraphInstantiate graph={graph:p} build_pm4_replay=skipped force_native={} native_exec=false",
            state.force_native
        );
        return hipErrorNotSupported;
    }
    if state.force_native && native_exec == 0 {
        hgdbg!(
            "hipGraphInstantiate graph={graph:p} build_pm4_replay=skipped force_native=true native_exec=false"
        );
        return hipErrorNotSupported;
    }
    let mut force_native = state.force_native;
    let mut pm4_build = "skipped";
    let replay = if force_native {
        None
    } else if let Some(plan) = own_exec.as_ref() {
        match build_pm4_replay(plan, &state.node_meta) {
            Ok(replay) => {
                pm4_build = "ok";
                Some(replay)
            }
            Err(_) if native_exec != 0 => {
                pm4_build = "failed";
                force_native = true;
                None
            }
            Err(status) => {
                hgdbg!(
                    "hipGraphInstantiate graph={graph:p} build_pm4_replay=failed force_native={force_native} native_exec=false"
                );
                return status;
            }
        }
    } else {
        None
    };
    hgdbg!(
        "hipGraphInstantiate graph={graph:p} build_pm4_replay={pm4_build} force_native={force_native} native_exec={}",
        native_exec != 0
    );
    let mut nodes = HashMap::new();
    let mut native_nodes = HashMap::new();
    for &node_key in &state.node_handles {
        if let Some(node) = node_snapshot(node_key as hipGraphNode_t) {
            if let Some(id) = node.node {
                nodes.insert(node_key, id);
            }
            if node.native_node != 0 {
                native_nodes.insert(node_key, node.native_node);
            }
        }
    }
    let exec = allocate_exec(ExecState {
        exec: own_exec,
        replay,
        dirty: false,
        node_meta: state.node_meta.clone(),
        nodes,
        native_nodes,
        native_exec,
        force_native,
    });
    unsafe { *output = exec };
    hipSuccess
}

unsafe fn native_launch(exec: usize, stream: hipStream_t) -> hipError_t {
    if exec == 0 {
        return hipErrorNotSupported;
    }
    type Function = unsafe extern "C" fn(hipGraphExec_t, hipStream_t) -> hipError_t;
    match unsafe { real_symbol::<Function>(b"hipGraphLaunch\0") } {
        Some(function) => unsafe { function(exec as hipGraphExec_t, stream) },
        None => hipErrorNotSupported,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphLaunch(exec: hipGraphExec_t, stream: hipStream_t) -> hipError_t {
    if !is_exec(exec) {
        hgdbg!("hipGraphLaunch exec={exec:p} stream={stream:p} branch=native_launch(unowned exec)");
        type Function = unsafe extern "C" fn(hipGraphExec_t, hipStream_t) -> hipError_t;
        return match unsafe { real_symbol::<Function>(b"hipGraphLaunch\0") } {
            Some(function) => unsafe { function(exec, stream) },
            None => hipErrorInvalidHandle,
        };
    }
    let exec_handle = exec_handle(exec).expect("owned exec checked");
    let mut state = lock(&exec_handle.state);
    if state.force_native {
        hgdbg!("hipGraphLaunch exec={exec:p} stream={stream:p} branch=native_launch(force_native)");
        return unsafe { native_launch(state.native_exec, stream) };
    }
    let Some(plan) = state.exec.as_ref() else {
        hgdbg!(
            "hipGraphLaunch exec={exec:p} stream={stream:p} branch=native_launch(no redline plan)"
        );
        return unsafe { native_launch(state.native_exec, stream) };
    };
    if state.dirty {
        match build_pm4_replay(plan, &state.node_meta) {
            Ok(replay) => {
                state.replay = Some(replay);
                state.dirty = false;
            }
            Err(_) if state.native_exec != 0 => {
                state.force_native = true;
                hgdbg!(
                    "hipGraphLaunch exec={exec:p} stream={stream:p} branch=native_launch(pm4 rebuild failed)"
                );
                return unsafe { native_launch(state.native_exec, stream) };
            }
            Err(status) => {
                hgdbg!(
                    "hipGraphLaunch exec={exec:p} stream={stream:p} branch=pm4 replay result=rebuild_failed"
                );
                return status;
            }
        }
    }
    let native_exec = state.native_exec;
    let Some(replay) = state.replay.as_mut() else {
        return if native_exec != 0 {
            hgdbg!(
                "hipGraphLaunch exec={exec:p} stream={stream:p} branch=native_launch(no pm4 replay)"
            );
            unsafe { native_launch(native_exec, stream) }
        } else {
            hgdbg!(
                "hipGraphLaunch exec={exec:p} stream={stream:p} branch=native_launch(no pm4 replay) native_exec=false"
            );
            hipErrorNotSupported
        };
    };
    // SAFETY: the graph exec retains code objects and packed pointer values;
    // HIP callers keep pointees live until graph launch completion. This MVP
    // intentionally waits on Redline's dedicated HSA queue before returning.
    match unsafe { replay.replay_and_wait() } {
        Ok(()) => {
            hgdbg!("hipGraphLaunch exec={exec:p} stream={stream:p} branch=pm4 replay");
            hipSuccess
        }
        Err(_) if native_exec != 0 => {
            hgdbg!(
                "hipGraphLaunch exec={exec:p} stream={stream:p} branch=native_launch(pm4 replay failed)"
            );
            unsafe { native_launch(native_exec, stream) }
        }
        Err(_) => {
            hgdbg!(
                "hipGraphLaunch exec={exec:p} stream={stream:p} branch=pm4 replay result=launch_failure"
            );
            hipErrorLaunchFailure
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphExecKernelNodeSetParams(
    exec: hipGraphExec_t,
    node: hipGraphNode_t,
    params: *const hipKernelNodeParams,
) -> hipError_t {
    if params.is_null() {
        return hipErrorInvalidValue;
    }
    if !is_exec(exec) {
        type Function = unsafe extern "C" fn(
            hipGraphExec_t,
            hipGraphNode_t,
            *const hipKernelNodeParams,
        ) -> hipError_t;
        return match unsafe { real_symbol::<Function>(b"hipGraphExecKernelNodeSetParams\0") } {
            Some(function) => unsafe { function(exec, node, params) },
            None => hipErrorInvalidHandle,
        };
    }
    let exec_handle = exec_handle(exec).expect("owned exec checked");
    let mut state = lock(&exec_handle.state);
    let node_key = node as usize;
    let native_status = match (
        state.native_exec,
        state.native_nodes.get(&node_key).copied(),
        unsafe {
            real_symbol::<
                unsafe extern "C" fn(
                    hipGraphExec_t,
                    hipGraphNode_t,
                    *const hipKernelNodeParams,
                ) -> hipError_t,
            >(b"hipGraphExecKernelNodeSetParams\0")
        },
    ) {
        (exec, Some(native_node), Some(function)) if exec != 0 => unsafe {
            function(
                exec as hipGraphExec_t,
                native_node as hipGraphNode_t,
                params,
            )
        },
        _ => hipErrorNotSupported,
    };
    let Some(&node_id) = state.nodes.get(&node_key) else {
        if native_status == hipSuccess {
            state.force_native = true;
            return hipSuccess;
        }
        return hipErrorInvalidHandle;
    };
    let params_value = unsafe { *params };
    let Some(function) = function_record(params_value.func) else {
        if native_status == hipSuccess {
            state.force_native = true;
            return hipSuccess;
        }
        return hipErrorNotSupported;
    };
    match unsafe { build_node_meta(&function, params_value) } {
        Ok((_, meta)) => {
            state.node_meta.insert(node_id, meta);
            state.dirty = true;
            hipSuccess
        }
        Err(_) if native_status == hipSuccess => {
            state.force_native = true;
            hipSuccess
        }
        Err(status) => status,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphExecUpdate(
    exec: hipGraphExec_t,
    graph: hipGraph_t,
    error_node: *mut hipGraphNode_t,
    update_result: *mut i32,
) -> hipError_t {
    if !is_exec(exec) {
        type Function = unsafe extern "C" fn(
            hipGraphExec_t,
            hipGraph_t,
            *mut hipGraphNode_t,
            *mut i32,
        ) -> hipError_t;
        return match unsafe { real_symbol::<Function>(b"hipGraphExecUpdate\0") } {
            Some(function) => unsafe { function(exec, graph, error_node, update_result) },
            None => hipErrorInvalidHandle,
        };
    }
    if !error_node.is_null() {
        unsafe { *error_node = ptr::null_mut() };
    }
    if !update_result.is_null() {
        unsafe { *update_result = hipGraphExecUpdateErrorNotSupported };
    }
    let Some(graph_handle) = graph_handle(graph) else {
        return hipErrorGraphExecUpdateFailure;
    };
    let exec_handle = exec_handle(exec).expect("owned exec checked");
    let graph_state = lock(&graph_handle.state);
    let mut exec_state = lock(&exec_handle.state);
    if graph_state.native_graph == 0 || exec_state.native_exec == 0 {
        return hipErrorGraphExecUpdateFailure;
    }
    type Function = unsafe extern "C" fn(
        hipGraphExec_t,
        hipGraph_t,
        *mut hipGraphNode_t,
        *mut i32,
    ) -> hipError_t;
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphExecUpdate\0") }) else {
        return hipErrorGraphExecUpdateFailure;
    };
    // HIP's hipGraphExecUpdate requires a valid hipGraphNode_t* out-param; passing
    // null here made the real runtime reject the call with hipErrorInvalidValue(1),
    // which callers (e.g. llama.cpp ggml-cuda.cu) treat as fatal rather than as a
    // recoverable update failure. Supply a local slot instead. The node handle is a
    // NATIVE node, so it is deliberately not propagated back to the caller's
    // error_node (already set to null above) to avoid handing out a foreign handle.
    let mut native_error_node: hipGraphNode_t = ptr::null_mut();
    let status = unsafe {
        function(
            exec_state.native_exec as hipGraphExec_t,
            graph_state.native_graph as hipGraph_t,
            &mut native_error_node,
            update_result,
        )
    };

    // ---- PM4 graph-exec UPDATE -------------------------------------------
    // Callers such as llama.cpp re-capture and update every iteration because
    // data pointers / shapes / strides advance while the topology stays fixed
    // (ggml-cuda.cu node_properties tracks exactly: node, src data ptrs, ne, nb).
    // The PM4 IB references kernarg buffers by ADDRESS, so those changes can be
    // applied by rewriting kernarg bytes in place -- no re-encode needed.
    //
    // Anything the PM4 stream bakes in (kernel identity, grid, block, dyn_group)
    // CANNOT be patched, so if any of those moved we fall back to native launch
    // rather than replaying a stale IB. Without this check the replay silently
    // produces wrong results (measured: byte-identical at 128 tokens, divergent
    // at 256, because attention geometry grows with the KV cache).
    // LOAD-BEARING ORDERING: this runs only after the native update returned
    // hipSuccess. HIP validates that the new graph is topologically identical to
    // the instantiated one, so a shape change cannot reach the kernarg patch
    // below. Do not hoist this above the native call.
    if status == hipSuccess && exec_state.replay.is_some() {
        let mut patchable = true;
        if let Some(plan) = exec_state.exec.as_ref() {
            let dispatches = plan.plan().dispatches();
            // geometry/kernel identity must be unchanged for every dispatch
            for planned in dispatches {
                let node = planned.node();
                match (graph_state.node_meta.get(&node), exec_state.node_meta.get(&node)) {
                    (Some(new_meta), Some(old_meta)) => {
                        if new_meta.grid != old_meta.grid
                            || new_meta.block != old_meta.block
                            || new_meta.dyn_group != old_meta.dyn_group
                            || new_meta.symbol != old_meta.symbol
                        {
                            patchable = false;
                            break;
                        }
                    }
                    _ => {
                        patchable = false;
                        break;
                    }
                }
            }
            if patchable {
                let order: Vec<NodeId> = dispatches.iter().map(|d| d.node()).collect();
                let new_args: Vec<Option<Vec<u8>>> = order
                    .iter()
                    .map(|node| graph_state.node_meta.get(node).map(|m| m.kernargs.clone()))
                    .collect();
                if let Some(replay) = exec_state.replay.as_mut() {
                    if replay.dispatch_count() == new_args.len() {
                        if let Err(error) = replay.update_kernargs(|index| {
                            new_args.get(index).and_then(|slot| slot.as_deref())
                        }) {
                            hgdbg!("hipGraphExecUpdate pm4_kernarg_patch=failed {error:?}");
                            exec_state.force_native = true;
                            return status;
                        }
                        // adopt the new kernargs as this exec's current state
                        for node in &order {
                            if let (Some(new_meta), Some(old_meta)) = (
                                graph_state.node_meta.get(node),
                                exec_state.node_meta.get_mut(node),
                            ) {
                                old_meta.kernargs.clone_from(&new_meta.kernargs);
                            }
                        }
                        hgdbg!(
                            "hipGraphExecUpdate pm4_kernarg_patch=ok dispatches={}",
                            new_args.len()
                        );
                        // replay stays valid -> do NOT pin to native
                        return status;
                    }
                }
            } else {
                hgdbg!("hipGraphExecUpdate pm4_kernarg_patch=skipped reason=geometry_changed");
            }
        }
    }

    if status == hipSuccess {
        // UPPER-BOUND PROBE ONLY (REDLINE_FORCE_REPLAY=1): normally a successful
        // native update pins this exec to native launches, which is why llama.cpp
        // (which updates every token) never exercises the PM4 replay. Skipping the
        // pin keeps the retained replay live so its SPEED can be measured -- the
        // replay then runs with capture-time pointers, so OUTPUT IS INCORRECT.
        // This exists to size the prize for implementing PM4 re-encode on update.
        if std::env::var_os("REDLINE_FORCE_REPLAY").is_none() {
            exec_state.force_native = true;
        } else {
            // This probe replays with capture-time pointers: results are WRONG by
            // construction. It exists to size the prize for PM4 re-encode on update.
            // It must never run silently -- a fast wrong number is worse than a slow
            // right one, and benchmark knobs outlive the person who set them.
            static WARNED: std::sync::Once = std::sync::Once::new();
            WARNED.call_once(|| {
                eprintln!(
                    "redline: REDLINE_FORCE_REPLAY=1 is set -- retained replay is kept \
live across graph-exec updates using CAPTURE-TIME kernargs. Output is INCORRECT. \
This is a speed upper-bound probe only; do not use it for correctness or for \
published measurements."
                );
            });
        }
    }
    status
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphExecDestroy(exec: hipGraphExec_t) -> hipError_t {
    if exec.is_null() {
        return hipSuccess;
    }
    if !is_exec(exec) {
        type Function = unsafe extern "C" fn(hipGraphExec_t) -> hipError_t;
        return match unsafe { real_symbol::<Function>(b"hipGraphExecDestroy\0") } {
            Some(function) => unsafe { function(exec) },
            None => hipErrorInvalidHandle,
        };
    }
    lock(&global().handles).execs.remove(&(exec as usize));
    let handle = unsafe { Box::from_raw(exec.cast::<ExecHandle>()) };
    let state = handle
        .state
        .into_inner()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    unsafe { native_exec_destroy(state.native_exec) };
    hipSuccess
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphDestroy(graph: hipGraph_t) -> hipError_t {
    if graph.is_null() {
        return hipSuccess;
    }
    if !is_graph(graph) {
        type Function = unsafe extern "C" fn(hipGraph_t) -> hipError_t;
        return match unsafe { real_symbol::<Function>(b"hipGraphDestroy\0") } {
            Some(function) => unsafe { function(graph) },
            None => hipErrorInvalidHandle,
        };
    }
    lock(&global().handles).graphs.remove(&(graph as usize));
    let handle = unsafe { Box::from_raw(graph.cast::<GraphHandle>()) };
    let state = handle
        .state
        .into_inner()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    {
        let mut handles = lock(&global().handles);
        for node in &state.node_handles {
            handles.nodes.remove(node);
        }
    }
    for node in state.node_handles {
        drop(unsafe { Box::from_raw((node as hipGraphNode_t).cast::<NodeHandle>()) });
    }
    unsafe { native_graph_destroy(state.native_graph) };
    hipSuccess
}

fn load_redline_code_object(code: Arc<[u8]>) -> Result<ModuleRecord, hipError_t> {
    let runtime = runtime()?;
    let executable =
        Executable::load(&runtime.device, code.clone()).map_err(|_| hipErrorInvalidImage)?;
    Ok(ModuleRecord {
        executable,
        code,
        native: false,
    })
}

unsafe fn load_redline_module(image: *const c_void) -> Result<ModuleRecord, hipError_t> {
    let code = unsafe { copy_code_object_image(image) }.ok_or(hipErrorInvalidImage)?;
    load_redline_code_object(code)
}

fn owned_token(set: fn(&mut HandleSets) -> &mut HashSet<usize>) -> usize {
    let pointer = Box::into_raw(Box::new(0_u8)) as usize;
    set(&mut lock(&global().handles)).insert(pointer);
    pointer
}

unsafe fn finish_module_load(
    output: *mut hipModule_t,
    image: *const c_void,
    native_status: Option<(hipError_t, hipModule_t)>,
) -> hipError_t {
    if output.is_null() || image.is_null() {
        return hipErrorInvalidValue;
    }
    if let Some((status, _)) = native_status {
        if status != hipSuccess {
            return status;
        }
    }
    let mut module = match unsafe { load_redline_module(image) } {
        Ok(module) => module,
        Err(status) => {
            return if let Some((_, native)) = native_status {
                unsafe { *output = native };
                hipSuccess
            } else {
                status
            };
        }
    };
    let handle = if let Some((_, native)) = native_status {
        module.native = true;
        native as usize
    } else {
        owned_token(|sets| &mut sets.owned_modules)
    };
    lock(&global().modules).insert(handle, module);
    unsafe { *output = handle as hipModule_t };
    hipSuccess
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipModuleLoadData(
    output: *mut hipModule_t,
    image: *const c_void,
) -> hipError_t {
    if output.is_null() || image.is_null() {
        return hipErrorInvalidValue;
    }
    type Function = unsafe extern "C" fn(*mut hipModule_t, *const c_void) -> hipError_t;
    let native = if let Some(function) = unsafe { real_symbol::<Function>(b"hipModuleLoadData\0") }
    {
        let mut module = ptr::null_mut();
        Some((unsafe { function(&mut module, image) }, module))
    } else {
        None
    };
    unsafe { finish_module_load(output, image, native) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipModuleLoadDataEx(
    output: *mut hipModule_t,
    image: *const c_void,
    option_count: u32,
    options: *mut i32,
    option_values: *mut *mut c_void,
) -> hipError_t {
    if output.is_null() || image.is_null() {
        return hipErrorInvalidValue;
    }
    type Function = unsafe extern "C" fn(
        *mut hipModule_t,
        *const c_void,
        u32,
        *mut i32,
        *mut *mut c_void,
    ) -> hipError_t;
    let native =
        if let Some(function) = unsafe { real_symbol::<Function>(b"hipModuleLoadDataEx\0") } {
            let mut module = ptr::null_mut();
            Some((
                unsafe { function(&mut module, image, option_count, options, option_values) },
                module,
            ))
        } else {
            None
        };
    unsafe { finish_module_load(output, image, native) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipModuleUnload(module: hipModule_t) -> hipError_t {
    if module.is_null() {
        return hipErrorInvalidValue;
    }
    let module_key = module as usize;
    let record = lock(&global().modules).get(&module_key).cloned();
    let Some(record) = record else {
        type Function = unsafe extern "C" fn(hipModule_t) -> hipError_t;
        return match unsafe { real_symbol::<Function>(b"hipModuleUnload\0") } {
            Some(function) => unsafe { function(module) },
            None => hipErrorInvalidHandle,
        };
    };
    if record.native {
        type Function = unsafe extern "C" fn(hipModule_t) -> hipError_t;
        let Some(function) = (unsafe { real_symbol::<Function>(b"hipModuleUnload\0") }) else {
            return hipErrorNotSupported;
        };
        let status = unsafe { function(module) };
        if status != hipSuccess {
            return status;
        }
    }
    lock(&global().modules).remove(&module_key);
    let function_keys = {
        let functions = lock(&global().functions);
        functions
            .iter()
            .filter_map(|(&key, function)| (function.module == module_key).then_some(key))
            .collect::<Vec<_>>()
    };
    {
        let mut functions = lock(&global().functions);
        for key in &function_keys {
            functions.remove(key);
        }
    }
    let mut handles = lock(&global().handles);
    for key in function_keys {
        if handles.owned_functions.remove(&key) {
            drop(unsafe { Box::from_raw(key as *mut u8) });
        }
    }
    if handles.owned_modules.remove(&module_key) {
        drop(unsafe { Box::from_raw(module_key as *mut u8) });
    }
    hipSuccess
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipModuleGetFunction(
    output: *mut hipFunction_t,
    module: hipModule_t,
    name: *const c_char,
) -> hipError_t {
    if output.is_null() || module.is_null() || name.is_null() {
        return hipErrorInvalidValue;
    }
    let Ok(requested) = (unsafe { CStr::from_ptr(name) }).to_str() else {
        return hipErrorInvalidValue;
    };
    let record = lock(&global().modules).get(&(module as usize)).cloned();
    let Some(module_record) = record else {
        type Function =
            unsafe extern "C" fn(*mut hipFunction_t, hipModule_t, *const c_char) -> hipError_t;
        return match unsafe { real_symbol::<Function>(b"hipModuleGetFunction\0") } {
            Some(function) => unsafe { function(output, module, name) },
            None => hipErrorInvalidHandle,
        };
    };

    type RealFunction =
        unsafe extern "C" fn(*mut hipFunction_t, hipModule_t, *const c_char) -> hipError_t;
    let native = if module_record.native {
        let Some(function) = (unsafe { real_symbol::<RealFunction>(b"hipModuleGetFunction\0") })
        else {
            return hipErrorNotSupported;
        };
        let mut result = ptr::null_mut();
        let status = unsafe { function(&mut result, module, name) };
        if status != hipSuccess {
            return status;
        }
        Some(result)
    } else {
        None
    };

    let Some(function_record) = resolve_module_function(&module_record, requested, module as usize)
    else {
        return hipErrorInvalidHandle;
    };
    let handle = if let Some(native) = native {
        native as usize
    } else {
        owned_token(|sets| &mut sets.owned_functions)
    };
    lock(&global().functions).insert(handle, function_record);
    unsafe { *output = handle as hipFunction_t };
    hipSuccess
}

unsafe fn append_capture_kernel(
    capture: CaptureState,
    function: hipFunction_t,
    grid: dim3,
    block: dim3,
    kernel_params: *mut *mut c_void,
    extra: *mut *mut c_void,
    shared_mem: u32,
) -> Result<hipGraphNode_t, hipError_t> {
    let params = hipKernelNodeParams {
        blockDim: block,
        extra,
        func: function,
        gridDim: grid,
        kernelParams: kernel_params,
        sharedMemBytes: shared_mem,
    };
    let dependency = capture.last_node as hipGraphNode_t;
    let (dependencies, count) = if dependency.is_null() {
        (ptr::null(), 0)
    } else {
        (&dependency as *const hipGraphNode_t, 1)
    };
    unsafe {
        add_kernel_node_internal(
            capture.graph as hipGraph_t,
            dependencies,
            count,
            &params,
            false,
        )
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipStreamBeginCapture(stream: hipStream_t, mode: i32) -> hipError_t {
    let stream_key = stream as usize;
    if lock(&global().captures).contains_key(&stream_key) {
        return hipErrorIllegalState;
    }
    type Function = unsafe extern "C" fn(hipStream_t, i32) -> hipError_t;
    let (function, resolution) =
        unsafe { real_symbol_with_resolution::<Function>(b"hipStreamBeginCapture\0") };
    let Some(function) = function else {
        hgdbg!(
            "hipStreamBeginCapture stream={stream:p} real_symbol={resolution} native_status={} redline_capture=none native_shadow=false",
            hipErrorNotSupported
        );
        return hipErrorNotSupported;
    };
    let native_status = unsafe { function(stream, mode) };
    if native_status != hipSuccess {
        hgdbg!(
            "hipStreamBeginCapture stream={stream:p} real_symbol={resolution} native_status={native_status} redline_capture=none native_shadow=false"
        );
        return native_status;
    }

    let graph = allocate_graph(0);
    lock(&global().captures).insert(
        stream_key,
        CaptureState {
            graph: graph as usize,
            last_node: 0,
            invalid: false,
            native_active: true,
        },
    );
    hgdbg!(
        "hipStreamBeginCapture stream={stream:p} graph={graph:p} real_symbol={resolution} native_status={native_status} redline_capture=created native_shadow=true"
    );
    hipSuccess
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipLaunchKernel(
    function_address: *const c_void,
    grid: dim3,
    block: dim3,
    kernel_params: *mut *mut c_void,
    shared_mem: usize,
    stream: hipStream_t,
) -> hipError_t {
    type Function = unsafe extern "C" fn(
        *const c_void,
        dim3,
        dim3,
        *mut *mut c_void,
        usize,
        hipStream_t,
    ) -> hipError_t;
    let capture = lock(&global().captures).get(&(stream as usize)).copied();
    let Some(capture) = capture else {
        return match unsafe { real_symbol::<Function>(b"hipLaunchKernel\0") } {
            Some(function) => unsafe {
                function(
                    function_address,
                    grid,
                    block,
                    kernel_params,
                    shared_mem,
                    stream,
                )
            },
            None => hipErrorNotSupported,
        };
    };
    let native_status = if capture.native_active {
        match unsafe { real_symbol::<Function>(b"hipLaunchKernel\0") } {
            Some(function) => unsafe {
                function(
                    function_address,
                    grid,
                    block,
                    kernel_params,
                    shared_mem,
                    stream,
                )
            },
            None => hipErrorNotSupported,
        }
    } else {
        hipErrorNotSupported
    };
    if capture.native_active && native_status != hipSuccess {
        if let Some(capture) = lock(&global().captures).get_mut(&(stream as usize)) {
            capture.invalid = true;
        }
        return native_status;
    }
    let own = u32::try_from(shared_mem)
        .map_err(|_| hipErrorInvalidValue)
        .and_then(|shared_mem| unsafe {
            append_capture_kernel(
                capture,
                function_address.cast_mut(),
                grid,
                block,
                kernel_params,
                ptr::null_mut(),
                shared_mem,
            )
        });
    match own {
        Ok(node) => {
            if let Some(capture) = lock(&global().captures).get_mut(&(stream as usize)) {
                capture.last_node = node as usize;
            }
            hipSuccess
        }
        Err(_) if capture.native_active => {
            if let Some(capture) = lock(&global().captures).get_mut(&(stream as usize)) {
                capture.invalid = true;
            }
            hipSuccess
        }
        Err(status) => status,
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn hipModuleLaunchKernel(
    function: hipFunction_t,
    grid_x: u32,
    grid_y: u32,
    grid_z: u32,
    block_x: u32,
    block_y: u32,
    block_z: u32,
    shared_mem: u32,
    stream: hipStream_t,
    kernel_params: *mut *mut c_void,
    extra: *mut *mut c_void,
) -> hipError_t {
    type Function = unsafe extern "C" fn(
        hipFunction_t,
        u32,
        u32,
        u32,
        u32,
        u32,
        u32,
        u32,
        hipStream_t,
        *mut *mut c_void,
        *mut *mut c_void,
    ) -> hipError_t;
    let capture = lock(&global().captures).get(&(stream as usize)).copied();
    let Some(capture) = capture else {
        return match unsafe { real_symbol::<Function>(b"hipModuleLaunchKernel\0") } {
            Some(real) => unsafe {
                real(
                    function,
                    grid_x,
                    grid_y,
                    grid_z,
                    block_x,
                    block_y,
                    block_z,
                    shared_mem,
                    stream,
                    kernel_params,
                    extra,
                )
            },
            None => hipErrorNotSupported,
        };
    };
    let native_status = if capture.native_active {
        match unsafe { real_symbol::<Function>(b"hipModuleLaunchKernel\0") } {
            Some(real) => unsafe {
                real(
                    function,
                    grid_x,
                    grid_y,
                    grid_z,
                    block_x,
                    block_y,
                    block_z,
                    shared_mem,
                    stream,
                    kernel_params,
                    extra,
                )
            },
            None => hipErrorNotSupported,
        }
    } else {
        hipErrorNotSupported
    };
    if capture.native_active && native_status != hipSuccess {
        if let Some(capture) = lock(&global().captures).get_mut(&(stream as usize)) {
            capture.invalid = true;
        }
        return native_status;
    }
    let own = unsafe {
        append_capture_kernel(
            capture,
            function,
            dim3::new(grid_x, grid_y, grid_z),
            dim3::new(block_x, block_y, block_z),
            kernel_params,
            extra,
            shared_mem,
        )
    };
    match own {
        Ok(node) => {
            if let Some(capture) = lock(&global().captures).get_mut(&(stream as usize)) {
                capture.last_node = node as usize;
            }
            hipSuccess
        }
        Err(_) if capture.native_active => {
            if let Some(capture) = lock(&global().captures).get_mut(&(stream as usize)) {
                capture.invalid = true;
            }
            hipSuccess
        }
        Err(status) => status,
    }
}

unsafe fn captured_native_node_count(graph: hipGraph_t) -> Option<usize> {
    if graph.is_null() {
        return Some(0);
    }
    type Function = unsafe extern "C" fn(hipGraph_t, *mut hipGraphNode_t, *mut usize) -> hipError_t;
    let function = unsafe { real_symbol::<Function>(b"hipGraphGetNodes\0") }?;
    let mut count = 0_usize;
    (unsafe { function(graph, ptr::null_mut(), &mut count) } == hipSuccess).then_some(count)
}

unsafe fn reconcile_captured_native_nodes(state: &mut GraphState) -> bool {
    if state.native_graph == 0 {
        return false;
    }
    type GetNodes = unsafe extern "C" fn(hipGraph_t, *mut hipGraphNode_t, *mut usize) -> hipError_t;
    type GetType = unsafe extern "C" fn(hipGraphNode_t, *mut i32) -> hipError_t;
    let Some(get_nodes) = (unsafe { real_symbol::<GetNodes>(b"hipGraphGetNodes\0") }) else {
        return false;
    };
    let Some(get_type) = (unsafe { real_symbol::<GetType>(b"hipGraphNodeGetType\0") }) else {
        return false;
    };
    let mut count = 0_usize;
    if unsafe {
        get_nodes(
            state.native_graph as hipGraph_t,
            ptr::null_mut(),
            &mut count,
        )
    } != hipSuccess
        || count != state.node_handles.len()
    {
        return false;
    }
    let mut native_nodes = vec![ptr::null_mut(); count];
    if count != 0
        && unsafe {
            get_nodes(
                state.native_graph as hipGraph_t,
                native_nodes.as_mut_ptr(),
                &mut count,
            )
        } != hipSuccess
    {
        return false;
    }
    for &native_node in &native_nodes {
        let mut node_type = -1_i32;
        if native_node.is_null()
            || unsafe { get_type(native_node, &mut node_type) } != hipSuccess
            || node_type != 0
        {
            return false;
        }
    }
    for (&redline_node, &native_node) in state.node_handles.iter().zip(&native_nodes) {
        if !lock(&global().handles).nodes.contains(&redline_node) {
            return false;
        }
        unsafe { (*(redline_node as *mut NodeHandle)).native_node = native_node as usize };
    }
    true
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipStreamEndCapture(
    stream: hipStream_t,
    output: *mut hipGraph_t,
) -> hipError_t {
    if output.is_null() {
        return hipErrorInvalidValue;
    }
    let Some(capture) = lock(&global().captures).remove(&(stream as usize)) else {
        type Function = unsafe extern "C" fn(hipStream_t, *mut hipGraph_t) -> hipError_t;
        let (real, resolution) =
            unsafe { real_symbol_with_resolution::<Function>(b"hipStreamEndCapture\0") };
        let status = match real {
            Some(function) => unsafe { function(stream, output) },
            None => hipErrorStreamCaptureUnmatched,
        };
        hgdbg!(
            "hipStreamEndCapture stream={stream:p} real_symbol={resolution} native_status={status} redline_capture=none"
        );
        return status;
    };
    let mut native_graph = ptr::null_mut();
    let mut native_status = hipErrorNotSupported;
    let mut native_resolution = SymbolResolution::Missing;
    if capture.native_active {
        type Function = unsafe extern "C" fn(hipStream_t, *mut hipGraph_t) -> hipError_t;
        let (function, resolution) =
            unsafe { real_symbol_with_resolution::<Function>(b"hipStreamEndCapture\0") };
        let Some(function) = function else {
            hgdbg!(
                "hipStreamEndCapture stream={stream:p} real_symbol={resolution} native_status={native_status} native_graph={native_graph:p}"
            );
            let _ = unsafe { hipGraphDestroy(capture.graph as hipGraph_t) };
            return native_status;
        };
        native_resolution = resolution;
        native_status = unsafe { function(stream, &mut native_graph) };
        if native_status != hipSuccess {
            hgdbg!(
                "hipStreamEndCapture stream={stream:p} real_symbol={resolution} native_status={native_status} native_graph={native_graph:p}"
            );
            let _ = unsafe { hipGraphDestroy(capture.graph as hipGraph_t) };
            return native_status;
        }
    }
    let graph = capture.graph as hipGraph_t;
    let Some(handle) = graph_handle(graph) else {
        if !native_graph.is_null() {
            unsafe { native_graph_destroy(native_graph as usize) };
        }
        return hipErrorUnknown;
    };
    let (redline_nodes, reconcile, topology_matches, force_native) = {
        let mut state = lock(&handle.state);
        state.native_graph = native_graph as usize;
        let reconcile = if capture.native_active && !capture.invalid {
            Some(unsafe { reconcile_captured_native_nodes(&mut state) })
        } else {
            None
        };
        let topology_matches =
            !capture.native_active || (!capture.invalid && reconcile == Some(true));
        state.force_native |= capture.invalid || !topology_matches;
        (
            state.node_meta.len(),
            reconcile,
            topology_matches,
            state.force_native,
        )
    };
    let native_nodes = if hg_debug_enabled() {
        unsafe { captured_native_node_count(native_graph) }
    } else {
        None
    };
    hgdbg!(
        "hipStreamEndCapture stream={stream:p} real_symbol={native_resolution} native_status={native_status} native_graph={native_graph:p} native_nodes={native_nodes:?} redline_nodes={redline_nodes} reconcile={reconcile:?} invalid={} topology_matches={topology_matches} force_native={force_native}",
        capture.invalid
    );
    if capture.invalid && native_graph.is_null() {
        let _ = unsafe { hipGraphDestroy(graph) };
        return hipErrorStreamCaptureInvalidated;
    }
    unsafe { *output = graph };
    hipSuccess
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipStreamIsCapturing(stream: hipStream_t, status: *mut i32) -> hipError_t {
    if status.is_null() {
        return hipErrorInvalidValue;
    }
    if lock(&global().captures).contains_key(&(stream as usize)) {
        unsafe { *status = hipStreamCaptureStatusActive };
        return hipSuccess;
    }
    type Function = unsafe extern "C" fn(hipStream_t, *mut i32) -> hipError_t;
    match unsafe { real_symbol::<Function>(b"hipStreamIsCapturing\0") } {
        Some(function) => unsafe { function(stream, status) },
        None => {
            unsafe { *status = hipStreamCaptureStatusNone };
            hipSuccess
        }
    }
}

unsafe fn translate_native_dependencies(
    graph_key: usize,
    dependencies: *const hipGraphNode_t,
    count: usize,
) -> Result<Vec<hipGraphNode_t>, hipError_t> {
    dependency_nodes(graph_key, dependencies, count)?
        .into_iter()
        .map(|node| {
            if node.native_node == 0 {
                Err(hipErrorNotSupported)
            } else {
                Ok(node.native_node as hipGraphNode_t)
            }
        })
        .collect()
}

fn finish_native_only_node(
    graph: hipGraph_t,
    native_node: hipGraphNode_t,
    output: *mut hipGraphNode_t,
) -> hipError_t {
    let Some(handle) = graph_handle(graph) else {
        return hipErrorInvalidHandle;
    };
    let mut state = lock(&handle.state);
    state.force_native = true;
    let node = allocate_node(graph as usize, &mut state, None, native_node as usize);
    unsafe { *output = node };
    hipSuccess
}

macro_rules! unsupported_pointer_node {
    ($name:ident, $symbol:literal) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(
            output: *mut hipGraphNode_t,
            graph: hipGraph_t,
            dependencies: *const hipGraphNode_t,
            count: usize,
            params: *const c_void,
        ) -> hipError_t {
            type Function = unsafe extern "C" fn(
                *mut hipGraphNode_t,
                hipGraph_t,
                *const hipGraphNode_t,
                usize,
                *const c_void,
            ) -> hipError_t;
            if !is_graph(graph) {
                return match unsafe { real_symbol::<Function>($symbol) } {
                    Some(function) => unsafe {
                        function(output, graph, dependencies, count, params)
                    },
                    None => hipErrorNotSupported,
                };
            }
            if output.is_null() {
                return hipErrorInvalidValue;
            }
            let Some(handle) = graph_handle(graph) else {
                return hipErrorInvalidHandle;
            };
            let native_graph = lock(&handle.state).native_graph;
            if native_graph == 0 {
                return hipErrorNotSupported;
            }
            let native_dependencies =
                match unsafe { translate_native_dependencies(graph as usize, dependencies, count) }
                {
                    Ok(dependencies) => dependencies,
                    Err(status) => return status,
                };
            let Some(function) = (unsafe { real_symbol::<Function>($symbol) }) else {
                return hipErrorNotSupported;
            };
            let mut native_node = ptr::null_mut();
            let status = unsafe {
                function(
                    &mut native_node,
                    native_graph as hipGraph_t,
                    native_dependencies.as_ptr(),
                    count,
                    params,
                )
            };
            if status == hipSuccess {
                finish_native_only_node(graph, native_node, output)
            } else {
                status
            }
        }
    };
}

unsupported_pointer_node!(hipGraphAddMemcpyNode, b"hipGraphAddMemcpyNode\0");
unsupported_pointer_node!(hipGraphAddMemsetNode, b"hipGraphAddMemsetNode\0");
unsupported_pointer_node!(hipGraphAddHostNode, b"hipGraphAddHostNode\0");

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hipGraphAddChildGraphNode(
    output: *mut hipGraphNode_t,
    graph: hipGraph_t,
    dependencies: *const hipGraphNode_t,
    count: usize,
    child: hipGraph_t,
) -> hipError_t {
    type Function = unsafe extern "C" fn(
        *mut hipGraphNode_t,
        hipGraph_t,
        *const hipGraphNode_t,
        usize,
        hipGraph_t,
    ) -> hipError_t;
    if !is_graph(graph) {
        return match unsafe { real_symbol::<Function>(b"hipGraphAddChildGraphNode\0") } {
            Some(function) => unsafe { function(output, graph, dependencies, count, child) },
            None => hipErrorNotSupported,
        };
    }
    if output.is_null() {
        return hipErrorInvalidValue;
    }
    let Some(parent_handle) = graph_handle(graph) else {
        return hipErrorInvalidHandle;
    };
    let Some(child_handle) = graph_handle(child) else {
        return hipErrorNotSupported;
    };
    let parent_native = lock(&parent_handle.state).native_graph;
    let child_native = lock(&child_handle.state).native_graph;
    if parent_native == 0 || child_native == 0 {
        return hipErrorNotSupported;
    }
    let native_dependencies =
        match unsafe { translate_native_dependencies(graph as usize, dependencies, count) } {
            Ok(dependencies) => dependencies,
            Err(status) => return status,
        };
    let Some(function) = (unsafe { real_symbol::<Function>(b"hipGraphAddChildGraphNode\0") })
    else {
        return hipErrorNotSupported;
    };
    let mut native_node = ptr::null_mut();
    let status = unsafe {
        function(
            &mut native_node,
            parent_native as hipGraph_t,
            native_dependencies.as_ptr(),
            count,
            child_native as hipGraph_t,
        )
    };
    if status == hipSuccess {
        finish_native_only_node(graph, native_node, output)
    } else {
        status
    }
}

#[cfg(feature = "python")]
#[allow(non_upper_case_globals)]
fn hip_error_name(status: hipError_t) -> &'static str {
    match status {
        hipSuccess => "hipSuccess",
        hipErrorInvalidValue => "hipErrorInvalidValue",
        hipErrorNotInitialized => "hipErrorNotInitialized",
        hipErrorInvalidImage => "hipErrorInvalidImage",
        hipErrorInvalidHandle => "hipErrorInvalidHandle",
        hipErrorIllegalState => "hipErrorIllegalState",
        hipErrorLaunchFailure => "hipErrorLaunchFailure",
        hipErrorNotSupported => "hipErrorNotSupported",
        hipErrorStreamCaptureInvalidated => "hipErrorStreamCaptureInvalidated",
        hipErrorStreamCaptureUnmatched => "hipErrorStreamCaptureUnmatched",
        hipErrorUnknown => "hipErrorUnknown",
        _ => "unrecognized hipError_t",
    }
}

#[cfg(feature = "python")]
fn py_hip_error(operation: &str, status: hipError_t) -> PyErr {
    PyRuntimeError::new_err(format!(
        "{operation} failed: {} ({status})",
        hip_error_name(status)
    ))
}

#[cfg(feature = "python")]
fn py_status(operation: &str, status: hipError_t) -> PyResult<()> {
    if status == hipSuccess {
        Ok(())
    } else {
        Err(py_hip_error(operation, status))
    }
}

#[cfg(feature = "python")]
fn py_pointer(value: u64, kind: &str) -> PyResult<*mut c_void> {
    usize::try_from(value)
        .map(|value| value as *mut c_void)
        .map_err(|_| PyOverflowError::new_err(format!("{kind} does not fit in a pointer")))
}

#[cfg(feature = "python")]
fn py_owned_exec(value: u64) -> PyResult<hipGraphExec_t> {
    let exec = py_pointer(value, "exec handle")?;
    if is_exec(exec) {
        Ok(exec)
    } else {
        Err(PyValueError::new_err(format!(
            "unknown redline graph exec handle 0x{value:x}"
        )))
    }
}

/// Return whether Redline can initialize ROCr and select a visible GPU.
#[cfg(feature = "python")]
#[pyfunction]
fn available() -> bool {
    std::panic::catch_unwind(|| runtime().is_ok()).unwrap_or(false)
}

/// Begin a Redline graph capture on the integer-valued HIP stream handle.
#[cfg(feature = "python")]
#[pyfunction]
fn capture_begin(stream: u64) -> PyResult<()> {
    let stream = py_pointer(stream, "stream")?;
    // HIP's global capture mode is zero. Calling the exported implementation
    // keeps Python and intercepted launches on the same global capture map.
    py_status("capture_begin", unsafe { hipStreamBeginCapture(stream, 0) })
}

/// End capture, instantiate the graph, and return its opaque exec handle.
#[cfg(feature = "python")]
#[pyfunction]
fn capture_end(stream: u64) -> PyResult<u64> {
    let stream = py_pointer(stream, "stream")?;
    let mut graph = ptr::null_mut();
    let end_status = unsafe { hipStreamEndCapture(stream, &mut graph) };
    py_status("capture_end", end_status)?;
    if graph.is_null() {
        return Err(PyRuntimeError::new_err(
            "capture_end succeeded without returning a graph",
        ));
    }

    let mut exec = ptr::null_mut();
    let instantiate_status =
        unsafe { hipGraphInstantiate(&mut exec, graph, ptr::null_mut(), ptr::null_mut(), 0) };
    let _ = unsafe { hipGraphDestroy(graph) };
    py_status("capture_end graph instantiation", instantiate_status)?;
    if exec.is_null() {
        return Err(PyRuntimeError::new_err(
            "graph instantiation succeeded without returning an exec handle",
        ));
    }
    Ok(exec as usize as u64)
}

/// Launch an opaque graph exec and return the HIP status code.
#[cfg(feature = "python")]
#[pyfunction]
fn launch(exec: u64, stream: u64) -> PyResult<i32> {
    let exec = py_owned_exec(exec)?;
    let stream = py_pointer(stream, "stream")?;
    Ok(unsafe { hipGraphLaunch(exec, stream) })
}

/// Destroy an opaque graph exec returned by [`capture_end`].
#[cfg(feature = "python")]
#[pyfunction]
fn exec_destroy(exec: u64) -> PyResult<()> {
    let exec = py_owned_exec(exec)?;
    py_status("exec_destroy", unsafe { hipGraphExecDestroy(exec) })
}

/// Return true when an exec currently owns a retained Redline PM4 replay.
#[cfg(feature = "python")]
#[pyfunction]
fn is_pm4(exec: u64) -> bool {
    let Ok(exec) = py_pointer(exec, "exec handle") else {
        return false;
    };
    let Some(handle) = exec_handle(exec) else {
        return false;
    };
    let state = lock(&handle.state);
    !state.force_native && state.replay.is_some()
}

#[cfg(feature = "python")]
#[pymodule]
fn redline_hipgraph(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(available, m)?)?;
    m.add_function(wrap_pyfunction!(capture_begin, m)?)?;
    m.add_function(wrap_pyfunction!(capture_end, m)?)?;
    m.add_function(wrap_pyfunction!(launch, m)?)?;
    m.add_function(wrap_pyfunction!(exec_destroy, m)?)?;
    m.add_function(wrap_pyfunction!(is_pm4, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fatbin_wrapper_matches_hip_compiler_abi() {
        assert_eq!(HIP_FATBIN_MAGIC, 0x4849_5046);
        assert_eq!(std::mem::size_of::<HipFatBinCWrapper>(), 24);
        assert_eq!(std::mem::offset_of!(HipFatBinCWrapper, magic), 0);
        assert_eq!(std::mem::offset_of!(HipFatBinCWrapper, version), 4);
        assert_eq!(std::mem::offset_of!(HipFatBinCWrapper, binary), 8);
        assert_eq!(std::mem::offset_of!(HipFatBinCWrapper, unused), 16);
    }

    #[test]
    fn fatbin_wrapper_reads_compiler_bytes_by_offset() {
        let binary = 0x1234_5678_usize as *const c_void;
        #[repr(align(8))]
        struct Aligned([u8; 25]);
        let mut storage = Aligned([0_u8; 25]);
        let bytes = &mut storage.0[1..];
        bytes[0..4].copy_from_slice(&[0x46, 0x50, 0x49, 0x48]);
        bytes[4..8].copy_from_slice(&1_i32.to_le_bytes());
        bytes[8..16].copy_from_slice(&(binary as usize).to_ne_bytes());
        assert_ne!(
            bytes.as_ptr() as usize % std::mem::align_of::<HipFatBinCWrapper>(),
            0
        );

        // SAFETY: `bytes` contains a complete compiler wrapper fixture.
        let wrapper = unsafe { read_hip_fatbin_wrapper(bytes.as_ptr().cast()) }.expect("wrapper");
        assert_eq!(wrapper.magic, HIP_FATBIN_MAGIC);
        assert_eq!(wrapper.version, 1);
        assert_eq!(wrapper.binary, binary);
        assert!(wrapper.unused.is_null());
    }

    #[test]
    fn versioned_lookup_includes_native_capture_abi() {
        assert!(HIP_SYMBOL_VERSIONS.contains(&b"hip_4.3\0".as_slice()));
    }

    #[test]
    fn locally_loaded_hip_symbol_is_resolved() {
        let handle = unsafe {
            libc::dlopen(
                b"libamdhip64.so\0".as_ptr().cast(),
                libc::RTLD_NOW | libc::RTLD_LOCAL,
            )
        };
        assert!(!handle.is_null(), "libamdhip64.so must be loader-visible");

        type Function = unsafe extern "C" fn(hipStream_t, i32) -> hipError_t;
        let (symbol, resolution) =
            unsafe { real_symbol_with_resolution::<Function>(b"hipStreamBeginCapture\0") };
        assert!(
            symbol.is_some(),
            "an RTLD_LOCAL libamdhip64 symbol must resolve"
        );
        assert_eq!(resolution, SymbolResolution::Handle);
    }
}
