// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! Real-GPU retained-PM4 replay over the C ABI.
//!
//! This is the fast path (a single retained GFX12 PM4 indirect buffer, replayed
//! with one doorbell — `SingleQueuePm4Ib`) exposed for an inference engine to
//! drive with its *own* kernels and kernargs:
//!
//! 1. `rl_gpu_new(ordinal)` — bind a ROCr GPU (honours `ROCR_VISIBLE_DEVICES`).
//! 2. `rl_gpu_load_module(gpu, code, len)` — load a code object (HSACO bytes).
//! 3. `rl_pm4_builder_new(gpu)`, then `rl_pm4_dispatch(...)` per launch with the
//!    engine's kernarg segment bytes (which carry its device pointers), and
//!    `rl_pm4_wait_idle(...)` to serialize where the graph requires it.
//! 4. `rl_pm4_finalize(gpu, builder)` — lower to one retained PM4 IB.
//! 5. `rl_pm4_replay(ib)` — submit + wait; call it every token.
//!
//! The engine supplies each kernel's **full kernarg segment** (query the size
//! with `rl_module_kernarg_size`); Redline copies it verbatim, so hidden args
//! the engine already populated (grid dims, etc.) are preserved.
//!
//! Lifetime contract (C caller owns it): a module must outlive every IB built
//! from it; an IB must outlive its replays. Free with the matching `*_free`.

use std::ffi::{CStr, c_char};
use std::sync::Arc;

use redline_dispatch::aql::{
    Executable, Gfx12Pm4CommandBuffer, GpuDevice, GpuSelector, KernargBuffer, KernargPool, Runtime,
    SingleQueuePm4Ib, load_symbols,
};

use crate::{RL_ERR_COMPILE, RL_ERR_NULL, RL_ERR_RECORD, RL_ERR_REPLAY, RL_ERR_UTF8, RL_OK};

/// A GPU binding: ROCr runtime + selected device + kernarg pool.
pub struct RlGpu {
    device: GpuDevice,
    pool: KernargPool,
    _runtime: Runtime,
}

/// A loaded code object; kernels are looked up from it by symbol.
pub struct RlModule {
    executable: Executable,
}

/// A builder accumulating dispatches into one PM4 command buffer.
pub struct RlPm4Builder {
    cmd: Gfx12Pm4CommandBuffer,
    kernargs: Vec<KernargBuffer>,
    modules: Vec<Executable>,
    pool: KernargPool,
}

/// A finalized, retained PM4 indirect buffer, replayable end to end.
pub struct RlPm4Ib {
    ib: SingleQueuePm4Ib,
    _kernargs: Vec<KernargBuffer>,
    _modules: Vec<Executable>,
}

/// Bind ROCr GPU `device_ordinal` (of the `ROCR_VISIBLE_DEVICES` set). Returns
/// null on failure. Free with [`rl_gpu_free`].
#[unsafe(no_mangle)]
pub extern "C" fn rl_gpu_new(device_ordinal: i32) -> *mut RlGpu {
    let build = || -> Option<RlGpu> {
        let runtime = Runtime::initialize(load_symbols().ok()?).ok()?;
        let ordinal = usize::try_from(device_ordinal).ok()?;
        let device = runtime.select_gpu(GpuSelector::Ordinal(ordinal)).ok()?;
        let pool = KernargPool::discover(&device).ok()?;
        Some(RlGpu {
            device,
            pool,
            _runtime: runtime,
        })
    };
    match build() {
        Some(gpu) => Box::into_raw(Box::new(gpu)),
        None => std::ptr::null_mut(),
    }
}

/// # Safety
/// `gpu` must be a pointer from [`rl_gpu_new`], or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rl_gpu_free(gpu: *mut RlGpu) {
    if !gpu.is_null() {
        drop(unsafe { Box::from_raw(gpu) });
    }
}

/// Load a code object (HSACO bytes). Writes the module pointer to `out`.
///
/// # Safety
/// `gpu`/`out` valid or null; `code` valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rl_gpu_load_module(
    gpu: *const RlGpu,
    code: *const u8,
    len: usize,
    out: *mut *mut RlModule,
) -> i32 {
    let Some(gpu) = (unsafe { gpu.as_ref() }) else {
        return RL_ERR_NULL;
    };
    if code.is_null() || out.is_null() {
        return RL_ERR_NULL;
    }
    let bytes: Arc<[u8]> = unsafe { std::slice::from_raw_parts(code, len) }.into();
    match Executable::load(&gpu.device, bytes) {
        Ok(executable) => {
            unsafe { *out = Box::into_raw(Box::new(RlModule { executable })) };
            RL_OK
        }
        Err(_) => RL_ERR_COMPILE,
    }
}

/// # Safety
/// `module` from [`rl_gpu_load_module`], or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rl_module_free(module: *mut RlModule) {
    if !module.is_null() {
        drop(unsafe { Box::from_raw(module) });
    }
}

/// The kernarg segment size (bytes) the engine must supply for `symbol`, or -1.
///
/// # Safety
/// `module`/`symbol` valid or null; `symbol` NUL-terminated.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rl_module_kernarg_size(module: *const RlModule, symbol: *const c_char) -> i64 {
    let Some(module) = (unsafe { module.as_ref() }) else {
        return -1;
    };
    if symbol.is_null() {
        return -1;
    }
    let Ok(sym) = (unsafe { CStr::from_ptr(symbol) }).to_str() else {
        return -1;
    };
    match module.executable.kernel(sym) {
        Ok(kernel) => i64::from(kernel.metadata().kernarg_segment_size),
        Err(_) => -1,
    }
}

/// New PM4 builder bound to `gpu`. Free with [`rl_pm4_builder_free`] (or hand it
/// to [`rl_pm4_finalize`], which consumes it). Returns null on failure.
///
/// # Safety
/// `gpu` valid or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rl_pm4_builder_new(gpu: *const RlGpu) -> *mut RlPm4Builder {
    let Some(gpu) = (unsafe { gpu.as_ref() }) else {
        return std::ptr::null_mut();
    };
    Box::into_raw(Box::new(RlPm4Builder {
        cmd: Gfx12Pm4CommandBuffer::new(),
        kernargs: Vec::new(),
        modules: Vec::new(),
        pool: gpu.pool.clone(),
    }))
}

/// Record one dispatch: `symbol` from `module`, `grid`/`block` in **workitems**
/// (grid = gridDim × blockDim), `dynamic_group_bytes`, and `kernarg`/`kernarg_len`
/// = the engine's kernarg segment (zero-padded / truncated to the kernel's size).
///
/// # Safety
/// Pointers valid or null; `symbol` NUL-terminated; `kernarg` valid for
/// `kernarg_len`.
#[unsafe(no_mangle)]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn rl_pm4_dispatch(
    builder: *mut RlPm4Builder,
    module: *const RlModule,
    symbol: *const c_char,
    grid_x: u32,
    grid_y: u32,
    grid_z: u32,
    block_x: u32,
    block_y: u32,
    block_z: u32,
    dynamic_group_bytes: u32,
    kernarg: *const u8,
    kernarg_len: usize,
) -> i32 {
    let Some(builder) = (unsafe { builder.as_mut() }) else {
        return RL_ERR_NULL;
    };
    let Some(module) = (unsafe { module.as_ref() }) else {
        return RL_ERR_NULL;
    };
    if symbol.is_null() {
        return RL_ERR_NULL;
    }
    let Ok(sym) = (unsafe { CStr::from_ptr(symbol) }).to_str() else {
        return RL_ERR_UTF8;
    };
    let Ok(kernel) = module.executable.kernel(sym) else {
        return RL_ERR_RECORD;
    };
    let Ok(block) = u16::try_from(block_x)
        .and_then(|x| Ok([x, u16::try_from(block_y)?, u16::try_from(block_z)?]))
    else {
        return RL_ERR_RECORD;
    };
    let Ok(geometry) = redline_dispatch::aql::LaunchGeometry::new([grid_x, grid_y, grid_z], block)
    else {
        return RL_ERR_RECORD;
    };
    let Ok(mut karg) = builder.pool.allocate_for(kernel.metadata()) else {
        return RL_ERR_RECORD;
    };
    {
        let dst = karg.as_mut_bytes();
        dst.fill(0);
        if !kernarg.is_null() && kernarg_len > 0 {
            let src = unsafe { std::slice::from_raw_parts(kernarg, kernarg_len) };
            let n = src.len().min(dst.len());
            dst[..n].copy_from_slice(&src[..n]);
        }
    }
    if builder
        .cmd
        .dispatch(&kernel, geometry, dynamic_group_bytes, karg.address())
        .is_err()
    {
        return RL_ERR_RECORD;
    }
    builder.kernargs.push(karg);
    builder.modules.push(module.executable.clone()); // keep the code object loaded
    RL_OK
}

/// Insert a dependency boundary after the dispatches recorded so far: wait for
/// the writer to retire, then invalidate scalar/vector read caches so the next
/// dispatch sees its L2-committed output. Required for a non-atomic read-modify-
/// write chain (decode); still the minimal gfx12 fence (L2/MALL stays coherent).
///
/// # Safety
/// `builder` valid or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rl_pm4_wait_idle(builder: *mut RlPm4Builder) {
    if let Some(builder) = unsafe { builder.as_mut() } {
        builder.cmd.wait_compute_idle();
        builder.cmd.acquire_inter_node_gfx12();
    }
}

/// # Safety
/// `builder` from [`rl_pm4_builder_new`] and NOT already finalized, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rl_pm4_builder_free(builder: *mut RlPm4Builder) {
    if !builder.is_null() {
        drop(unsafe { Box::from_raw(builder) });
    }
}

/// Lower the builder into a retained PM4 IB. **Consumes `builder`** (do not use
/// or free it after this). Writes the IB pointer to `out`.
///
/// # Safety
/// `gpu`/`out` valid or null; `builder` a live, un-finalized builder pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rl_pm4_finalize(
    gpu: *const RlGpu,
    builder: *mut RlPm4Builder,
    out: *mut *mut RlPm4Ib,
) -> i32 {
    let Some(gpu) = (unsafe { gpu.as_ref() }) else {
        return RL_ERR_NULL;
    };
    if builder.is_null() || out.is_null() {
        return RL_ERR_NULL;
    }
    let builder = unsafe { Box::from_raw(builder) };
    match SingleQueuePm4Ib::create(&gpu.device, &gpu.pool, &builder.cmd) {
        Ok(ib) => {
            let boxed = Box::new(RlPm4Ib {
                ib,
                _kernargs: builder.kernargs,
                _modules: builder.modules,
            });
            unsafe { *out = Box::into_raw(boxed) };
            RL_OK
        }
        Err(_) => RL_ERR_COMPILE,
    }
}

/// Replay the retained IB once (submit + wait for completion). Returns `RL_OK`.
///
/// # Safety
/// `ib` from [`rl_pm4_finalize`], or null; every device pointer encoded in the
/// engine's kernargs must remain valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rl_pm4_replay(ib: *mut RlPm4Ib) -> i32 {
    let Some(ib) = (unsafe { ib.as_mut() }) else {
        return RL_ERR_NULL;
    };
    // SAFETY: the retained IB owns its kernargs; the caller upholds pointee validity.
    match unsafe { ib.ib.replay_and_wait() } {
        Ok(()) => RL_OK,
        Err(_) => RL_ERR_REPLAY,
    }
}

/// # Safety
/// `ib` from [`rl_pm4_finalize`], or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rl_pm4_ib_free(ib: *mut RlPm4Ib) {
    if !ib.is_null() {
        drop(unsafe { Box::from_raw(ib) });
    }
}
