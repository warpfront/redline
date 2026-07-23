#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Drop-in HipSequenceTimer backed by a retained Redline PM4 IB.

The production-shaped hipEngine runners load HIP shared objects from Python.
This timer captures their unchanged launch closures once, recovers the exact
kernel arguments from the HIP graph, then replays the ordinary hipcc HSACO via
Redline. The captured HIP graph is introspection only and is never timed.
"""

from __future__ import annotations

import ctypes
import json
import os
import time
from collections import namedtuple
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Callable


HipTimingSamples = namedtuple("HipTimingSamples", "gpu_sequence_us host_sequence_us")
RL_OK = 0
RL_ERR_COMPILE = -4
HIP_GRAPH_NODE_KERNEL = 0
RL_QUEUE_AUTO = 0


class Dim3(ctypes.Structure):
    _fields_ = [("x", ctypes.c_uint), ("y", ctypes.c_uint), ("z", ctypes.c_uint)]


class HipKernelNodeParams(ctypes.Structure):
    _fields_ = [
        ("blockDim", Dim3),
        ("extra", ctypes.POINTER(ctypes.c_void_p)),
        ("func", ctypes.c_void_p),
        ("gridDim", Dim3),
        ("kernelParams", ctypes.POINTER(ctypes.c_void_p)),
        ("sharedMemBytes", ctypes.c_uint),
    ]


@dataclass(frozen=True)
class _RetainedIb:
    """Single-queue or multi-queue retained PM4 handle."""

    ptr: int
    multi: bool

    def free(self, lib: ctypes.CDLL) -> None:
        handle = ctypes.c_void_p(self.ptr)
        if self.multi:
            lib.rl_pm4_multi_ib_free(handle)
        else:
            lib.rl_pm4_ib_free(handle)

    def replay_profiled(self, lib: ctypes.CDLL, out_gpu_us: ctypes.c_double) -> int:
        handle = ctypes.c_void_p(self.ptr)
        if self.multi:
            return int(lib.rl_pm4_replay_multi_profiled(handle, ctypes.byref(out_gpu_us)))
        return int(lib.rl_pm4_replay_profiled(handle, ctypes.byref(out_gpu_us)))


def _lane_plan(timing_mode: str, queue_cap: int, logical_iterations: int) -> list[int]:
    """Map each logical iteration to one independent lane index."""
    if queue_cap < 1 or logical_iterations < 1:
        raise ValueError("queue_cap and logical_iterations must be positive")
    if timing_mode == "serial_latency":
        return [0] * logical_iterations
    if timing_mode != "independent_throughput":
        raise ValueError("invalid timing mode")
    active = min(queue_cap, logical_iterations)
    return [rep % active for rep in range(logical_iterations)]


def _configure_hip(lib: ctypes.CDLL) -> None:
    lib.hipGraphGetNodes.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_void_p), ctypes.POINTER(ctypes.c_size_t)]
    lib.hipGraphGetNodes.restype = ctypes.c_int
    lib.hipGraphNodeGetDependencies.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_void_p), ctypes.POINTER(ctypes.c_size_t)]
    lib.hipGraphNodeGetDependencies.restype = ctypes.c_int
    lib.hipGraphNodeGetType.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_int)]
    lib.hipGraphNodeGetType.restype = ctypes.c_int
    lib.hipGraphKernelNodeGetParams.argtypes = [ctypes.c_void_p, ctypes.POINTER(HipKernelNodeParams)]
    lib.hipGraphKernelNodeGetParams.restype = ctypes.c_int
    lib.hipKernelNameRefByPtr.argtypes = [ctypes.c_void_p, ctypes.c_void_p]
    lib.hipKernelNameRefByPtr.restype = ctypes.c_char_p


def _configure_redline(lib: ctypes.CDLL) -> None:
    vp = ctypes.c_void_p
    lib.rl_gpu_new.argtypes = [ctypes.c_int32]
    lib.rl_gpu_new.restype = vp
    lib.rl_gpu_pm4_queue_count.argtypes = [vp, ctypes.c_int, ctypes.c_size_t]
    lib.rl_gpu_pm4_queue_count.restype = ctypes.c_size_t
    lib.rl_gpu_load_module_radiowave.argtypes = [
        vp,
        ctypes.POINTER(ctypes.c_uint8),
        ctypes.c_size_t,
        ctypes.POINTER(ctypes.c_uint8),
        ctypes.c_size_t,
        ctypes.POINTER(vp),
    ]
    lib.rl_gpu_load_module_radiowave.restype = ctypes.c_int32
    lib.rl_module_radiowave_certified.argtypes = [vp]
    lib.rl_module_radiowave_certified.restype = ctypes.c_bool
    lib.rl_pm4_builder_new.argtypes = [vp]
    lib.rl_pm4_builder_new.restype = vp
    lib.rl_pm4_builder_free.argtypes = [vp]
    lib.rl_pm4_dispatch.argtypes = [vp, vp, ctypes.c_char_p, *([ctypes.c_uint32] * 7), ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t]
    lib.rl_pm4_dispatch.restype = ctypes.c_int32
    lib.rl_pm4_wait_rmw.argtypes = [vp, vp, ctypes.c_char_p]
    lib.rl_pm4_wait_rmw.restype = ctypes.c_int32
    lib.rl_pm4_finalize_profiled.argtypes = [vp, vp, ctypes.POINTER(vp)]
    lib.rl_pm4_finalize_profiled.restype = ctypes.c_int32
    lib.rl_pm4_replay_profiled.argtypes = [vp, ctypes.POINTER(ctypes.c_double)]
    lib.rl_pm4_replay_profiled.restype = ctypes.c_int32
    lib.rl_pm4_ib_free.argtypes = [vp]
    lib.rl_pm4_finalize_multi_profiled.argtypes = [
        vp,
        ctypes.POINTER(vp),
        ctypes.c_size_t,
        ctypes.POINTER(vp),
    ]
    lib.rl_pm4_finalize_multi_profiled.restype = ctypes.c_int32
    lib.rl_pm4_replay_multi_profiled.argtypes = [vp, ctypes.POINTER(ctypes.c_double)]
    lib.rl_pm4_replay_multi_profiled.restype = ctypes.c_int32
    lib.rl_pm4_multi_ib_lane_count.argtypes = [vp]
    lib.rl_pm4_multi_ib_lane_count.restype = ctypes.c_size_t
    lib.rl_pm4_multi_ib_free.argtypes = [vp]


class _Context:
    def __init__(self) -> None:
        root = Path(__file__).resolve().parents[2]
        self.lib = ctypes.CDLL(str(root / "target/release/libredline_dispatch.so"))
        _configure_redline(self.lib)
        self.gpu = self.lib.rl_gpu_new(0)
        if not self.gpu:
            raise RuntimeError("rl_gpu_new failed")
        self.modules: dict[Path, int] = {}
        self.specs: dict[str, tuple[Path, Path, dict[str, Any]]] = {}

    def refresh_sidecars(self) -> None:
        paths: set[Path] = set()
        for line in Path("/proc/self/maps").read_text().splitlines():
            fields = line.split()
            if fields and fields[-1].startswith("/"):
                path = Path(fields[-1])
                sidecar = path.with_suffix(path.suffix + ".redline.manifest.json")
                if sidecar.exists():
                    paths.add(sidecar)
        for manifest in paths:
            bundle = manifest.with_name(manifest.name.removesuffix(".manifest.json") + ".co")
            # Wrapper names are <output>.redline.{co,manifest.json}; the first
            # expression above handles that exact stable pairing.
            if not bundle.exists():
                bundle = Path(str(manifest).replace(".manifest.json", ".co"))
            radiowave = manifest.with_name(
                manifest.name.removesuffix(".manifest.json") + ".radiowave.json"
            )
            if not radiowave.exists():
                raise RuntimeError(f"Radiowave manifest is missing for {bundle}")
            data = json.loads(manifest.read_text())
            for spec in data["kernels"]:
                for name in (spec["name"], spec["symbol"], spec["symbol"].removesuffix(".kd")):
                    self.specs[name] = (bundle, radiowave, spec)

    def resolve(self, name: str) -> tuple[int, dict[str, Any]]:
        if name not in self.specs:
            self.refresh_sidecars()
        if name not in self.specs:
            raise RuntimeError(f"kernel is absent from loaded Redline sidecars: {name}")
        bundle, radiowave, spec = self.specs[name]
        if bundle not in self.modules:
            raw = bundle.read_bytes()
            encoded_manifest = radiowave.read_bytes()
            storage = (ctypes.c_uint8 * len(raw)).from_buffer_copy(raw)
            manifest_storage = (ctypes.c_uint8 * len(encoded_manifest)).from_buffer_copy(
                encoded_manifest
            )
            module = ctypes.c_void_p()
            if (
                self.lib.rl_gpu_load_module_radiowave(
                    self.gpu,
                    storage,
                    len(raw),
                    manifest_storage,
                    len(encoded_manifest),
                    ctypes.byref(module),
                )
                != RL_OK
            ):
                raise RuntimeError(f"rl_gpu_load_module_radiowave failed for {bundle}")
            if not self.lib.rl_module_radiowave_certified(module):
                raise RuntimeError(f"Radiowave certification was not retained for {bundle}")
            self.modules[bundle] = int(module.value)
        return self.modules[bundle], spec


_CONTEXT: _Context | None = None


def _context() -> _Context:
    global _CONTEXT
    if _CONTEXT is None:
        _CONTEXT = _Context()
    return _CONTEXT


def _topological_nodes(runtime: Any, graph: int) -> list[int]:
    lib = runtime.library
    _configure_hip(lib)
    count = ctypes.c_size_t()
    runtime.check(lib.hipGraphGetNodes(ctypes.c_void_p(graph), None, ctypes.byref(count)))
    raw = (ctypes.c_void_p * count.value)()
    runtime.check(lib.hipGraphGetNodes(ctypes.c_void_p(graph), raw, ctypes.byref(count)))
    nodes = [int(raw[i]) for i in range(count.value)]
    ordered: list[int] = []
    emitted: set[int] = set()
    while len(ordered) != len(nodes):
        progress = False
        for node in nodes:
            if node in emitted:
                continue
            dep_count = ctypes.c_size_t()
            runtime.check(lib.hipGraphNodeGetDependencies(ctypes.c_void_p(node), None, ctypes.byref(dep_count)))
            deps_raw = (ctypes.c_void_p * dep_count.value)()
            if dep_count.value:
                runtime.check(lib.hipGraphNodeGetDependencies(ctypes.c_void_p(node), deps_raw, ctypes.byref(dep_count)))
            deps = {int(deps_raw[i]) for i in range(dep_count.value)}
            if deps <= emitted:
                emitted.add(node)
                ordered.append(node)
                progress = True
        if not progress:
            raise RuntimeError("captured HIP graph is cyclic")
    return ordered


def _copy_integer(kernarg: bytearray, arg: dict[str, Any], value: int) -> None:
    offset, size = int(arg["offset"]), int(arg["size"])
    kernarg[offset : offset + size] = int(value).to_bytes(size, "little")


class HipSequenceTimer:
    """API-compatible timer used by the three production-shaped runners."""

    def __init__(self, runtime: Any, timing_mode: str, independent_streams: int = 4):
        if timing_mode not in {"serial_latency", "independent_throughput"}:
            raise ValueError("invalid timing mode")
        self.runtime = runtime
        self.timing_mode = timing_mode
        self.capture_stream = runtime.stream_create(nonblocking=True)
        ctx = _context()
        if timing_mode == "serial_latency":
            self.resolved_lanes = 1
        else:
            resolved = int(
                ctx.lib.rl_gpu_pm4_queue_count(
                    ctx.gpu, RL_QUEUE_AUTO, max(1, int(independent_streams))
                )
            )
            self.resolved_lanes = max(1, resolved)
        # Launch APIs still receive capture-stream aliases; multi-queue is ROCr.
        self.workers = [self.capture_stream] * self.resolved_lanes
        self._ibs: dict[tuple[int, int], _RetainedIb] = {}
        self._preheated_ibs: set[int] = set()

    @property
    def stream_count(self) -> int:
        return len(self.workers)

    def __enter__(self) -> "HipSequenceTimer":
        return self

    def __exit__(self, exc_type, exc, traceback) -> None:
        self.close()

    def close(self) -> None:
        ctx = _context()
        for ib in self._ibs.values():
            ib.free(ctx.lib)
        self._ibs.clear()
        if self.capture_stream:
            self.runtime.stream_destroy(self.capture_stream)
            self.capture_stream = 0

    def _graph(self, logical_iterations: int, launch: Callable[[int, int], None]) -> _RetainedIb:
        key = (logical_iterations, id(launch))
        if key in self._ibs:
            return self._ibs[key]
        self.runtime.stream_begin_capture(self.capture_stream)
        for rep in range(logical_iterations):
            launch(rep, self.capture_stream)
        graph = self.runtime.stream_end_capture(self.capture_stream)
        nodes = _topological_nodes(self.runtime, graph)
        if not nodes or len(nodes) % logical_iterations:
            self.runtime.graph_destroy(graph)
            raise RuntimeError("captured graph does not have a fixed kernel count per iteration")
        per_iteration = len(nodes) // logical_iterations
        ctx = _context()
        lane_of = _lane_plan(self.timing_mode, self.resolved_lanes, logical_iterations)
        active_lanes = max(lane_of) + 1 if lane_of else 1

        builders: list[int] = []
        builders_owned = True
        try:
            for _ in range(active_lanes):
                raw_builder = ctx.lib.rl_pm4_builder_new(ctx.gpu)
                if not raw_builder:
                    raise RuntimeError("rl_pm4_builder_new failed")
                builders.append(int(raw_builder))

            for index, node in enumerate(nodes):
                node_type = ctypes.c_int()
                self.runtime.check(
                    self.runtime.library.hipGraphNodeGetType(
                        ctypes.c_void_p(node), ctypes.byref(node_type)
                    )
                )
                if node_type.value != HIP_GRAPH_NODE_KERNEL:
                    raise RuntimeError("capture contains a non-kernel node")
                params = HipKernelNodeParams()
                self.runtime.check(
                    self.runtime.library.hipGraphKernelNodeGetParams(
                        ctypes.c_void_p(node), ctypes.byref(params)
                    )
                )
                raw_name = self.runtime.library.hipKernelNameRefByPtr(
                    params.func, ctypes.c_void_p(self.capture_stream)
                )
                if not raw_name:
                    raise RuntimeError("hipKernelNameRefByPtr failed")
                module, spec = ctx.resolve(raw_name.decode())
                rep = index // per_iteration
                stage = index % per_iteration
                lane = lane_of[rep]
                builder = builders[lane]
                dependency_before = (
                    self.timing_mode == "serial_latency" and index > 0
                ) or (
                    self.timing_mode == "independent_throughput" and stage != 0
                )
                if dependency_before:
                    rc = ctx.lib.rl_pm4_wait_rmw(
                        ctypes.c_void_p(builder),
                        ctypes.c_void_p(module),
                        spec["symbol"].encode(),
                    )
                    if rc != RL_OK:
                        raise RuntimeError(f"rl_pm4_wait_rmw failed for {spec['symbol']}")
                kernarg = bytearray(int(spec["kernarg_size"]))
                explicit = 0
                for arg in spec["args"]:
                    kind = arg["value_kind"]
                    offset, size = int(arg["offset"]), int(arg["size"])
                    if not kind.startswith("hidden_"):
                        if not params.kernelParams:
                            raise RuntimeError("captured explicit kernel argument is null")
                        kernarg[offset : offset + size] = ctypes.string_at(
                            params.kernelParams[explicit], size
                        )
                        explicit += 1
                    elif kind == "hidden_block_count_x":
                        _copy_integer(kernarg, arg, params.gridDim.x)
                    elif kind == "hidden_block_count_y":
                        _copy_integer(kernarg, arg, params.gridDim.y)
                    elif kind == "hidden_block_count_z":
                        _copy_integer(kernarg, arg, params.gridDim.z)
                    elif kind == "hidden_group_size_x":
                        _copy_integer(kernarg, arg, params.blockDim.x)
                    elif kind == "hidden_group_size_y":
                        _copy_integer(kernarg, arg, params.blockDim.y)
                    elif kind == "hidden_group_size_z":
                        _copy_integer(kernarg, arg, params.blockDim.z)
                    elif kind == "hidden_remainder_x":
                        _copy_integer(kernarg, arg, params.blockDim.x)
                    elif kind == "hidden_remainder_y":
                        _copy_integer(kernarg, arg, params.blockDim.y)
                    elif kind == "hidden_remainder_z":
                        _copy_integer(kernarg, arg, params.blockDim.z)
                    elif kind == "hidden_grid_dims":
                        _copy_integer(
                            kernarg,
                            arg,
                            3
                            if params.gridDim.z > 1
                            else 2
                            if params.gridDim.y > 1
                            else 1,
                        )
                    elif kind == "hidden_dynamic_lds_size":
                        _copy_integer(kernarg, arg, params.sharedMemBytes)
                if os.environ.get("REDLINE_CAPTURE_TRACE"):
                    explicit_end = max(
                        (
                            int(arg["offset"]) + int(arg["size"])
                            for arg in spec["args"]
                            if not arg["value_kind"].startswith("hidden_")
                        ),
                        default=0,
                    )
                    print(
                        f"[redline capture] {raw_name.decode()} grid={params.gridDim.x},{params.gridDim.y},{params.gridDim.z} "
                        f"block={params.blockDim.x},{params.blockDim.y},{params.blockDim.z} args={kernarg[:explicit_end].hex()}",
                        file=os.sys.stderr,
                    )
                storage = (ctypes.c_uint8 * len(kernarg)).from_buffer_copy(kernarg)
                rc = ctx.lib.rl_pm4_dispatch(
                    ctypes.c_void_p(builder),
                    ctypes.c_void_p(module),
                    spec["symbol"].encode(),
                    params.gridDim.x * params.blockDim.x,
                    params.gridDim.y * params.blockDim.y,
                    params.gridDim.z * params.blockDim.z,
                    params.blockDim.x,
                    params.blockDim.y,
                    params.blockDim.z,
                    params.sharedMemBytes,
                    storage,
                    len(storage),
                )
                if rc != RL_OK:
                    raise RuntimeError(f"rl_pm4_dispatch failed for {spec['symbol']}")

            ib_ptr = ctypes.c_void_p()
            if active_lanes == 1:
                # Single finalize always consumes the builder pointer.
                builders_owned = False
                if (
                    ctx.lib.rl_pm4_finalize_profiled(
                        ctx.gpu, ctypes.c_void_p(builders[0]), ctypes.byref(ib_ptr)
                    )
                    != RL_OK
                ):
                    raise RuntimeError("rl_pm4_finalize_profiled failed")
                retained = _RetainedIb(int(ib_ptr.value), multi=False)
            else:
                builder_array = (ctypes.c_void_p * active_lanes)(
                    *[ctypes.c_void_p(b) for b in builders]
                )
                rc = ctx.lib.rl_pm4_finalize_multi_profiled(
                    ctx.gpu,
                    builder_array,
                    active_lanes,
                    ctypes.byref(ib_ptr),
                )
                # Multi consumes builders after validation succeeds, including
                # when PM4 compilation subsequently fails (RL_ERR_COMPILE).
                if rc == RL_OK or rc == RL_ERR_COMPILE:
                    builders_owned = False
                if rc != RL_OK:
                    raise RuntimeError("rl_pm4_finalize_multi_profiled failed")
                retained = _RetainedIb(int(ib_ptr.value), multi=True)
                got_lanes = int(ctx.lib.rl_pm4_multi_ib_lane_count(ib_ptr))
                if got_lanes != active_lanes:
                    retained.free(ctx.lib)
                    raise RuntimeError(
                        f"multi IB lane count {got_lanes} != active {active_lanes}"
                    )
        except Exception:
            if builders_owned:
                for builder in builders:
                    ctx.lib.rl_pm4_builder_free(ctypes.c_void_p(builder))
            self.runtime.graph_destroy(graph)
            raise

        self.runtime.graph_destroy(graph)
        self._ibs[key] = retained
        return retained

    def measure(self, logical_iterations: int, samples: int, launch: Callable[[int, int], None]) -> HipTimingSamples:
        ib = self._graph(logical_iterations, launch)
        # Production-slice runners size their deterministic CPU fixture from
        # `warmup`, so using a large logical warmup also creates hundreds of
        # expensive reference outputs. Keep saturation preconditioning an
        # adapter concern instead: replay the already-recorded measured tape
        # outside the returned sample set, without changing fixture cardinality.
        preheat = int(os.environ.get("REDLINE_PREHEAT_REPLAYS", "0"))
        if preheat < 0:
            raise ValueError("REDLINE_PREHEAT_REPLAYS must be non-negative")
        ctx = _context()
        if preheat and ib.ptr not in self._preheated_ibs:
            ignored = ctypes.c_double()
            for _ in range(preheat):
                if ib.replay_profiled(ctx.lib, ignored) != RL_OK:
                    raise RuntimeError("Redline preheat replay failed")
            self._preheated_ibs.add(ib.ptr)
        gpu_samples: list[float] = []
        host_samples: list[float] = []
        for _ in range(samples):
            elapsed = ctypes.c_double()
            start = time.perf_counter_ns()
            if ib.replay_profiled(ctx.lib, elapsed) != RL_OK:
                raise RuntimeError("rl_pm4_replay_profiled failed")
            host_samples.append((time.perf_counter_ns() - start) / 1000.0)
            gpu_samples.append(elapsed.value)
        return HipTimingSamples(gpu_samples, host_samples)

    def run_and_wait(self, logical_iterations: int, launch: Callable[[int, int], None]) -> None:
        elapsed = ctypes.c_double()
        ib = self._graph(logical_iterations, launch)
        if ib.replay_profiled(_context().lib, elapsed) != RL_OK:
            raise RuntimeError("Redline replay failed")
