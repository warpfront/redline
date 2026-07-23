<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# Using Redline

Redline provides lightning-fast kernel dispatch for ROCm by recording a fixed
kernel sequence once and replaying it as retained PM4 over public ROCr/HSA
queues. Choose the integration surface that matches who owns graph capture and
kernel loading in your application.

> **Current distribution status:** Redline is usable from a source checkout.
> `redline-dispatch` is not yet published on PyPI, the C SDK is not yet attached
> to GitHub Releases, and the Rust crates are not yet on crates.io. The commands
> below deliberately build the current checkout instead of referring to packages
> that do not exist yet.

## Choose your path

| You have | Use | Current source status |
| --- | --- | --- |
| A C or C++ engine that owns its kernels and buffers | [`redline-capi`](#cc-engine-api) | Real module load, graph or direct retained-PM4 build, kernarg patch, replay, and multiqueue APIs |
| A Python application | [`redline-dispatch` Python module](#python) | Graph authoring plus real retained-PM4 load/build/replay; direct Python PM4 build is currently GFX12-only |
| An existing application that already calls `hipGraph*` | [`redline-hipgraph`](#existing-hipgraph-application) | `LD_PRELOAD` interposer for supported captures; unsupported operations fall through to HIP |
| A Rust application or engine | [`redline-dispatch`](#rust) | Native graph, public-AQL, HIP multistream, and retained-PM4 APIs from a source/path dependency |

For engine integrations, start with the C ABI unless Rust is already part of
your process. The explicit API makes kernel identity, kernargs, dependencies,
and ownership visible; the preload path is for compatible applications that
cannot change their graph call sites.

## Prerequisites

The commands in this guide are verified on x86-64 Linux with the ROCm Core SDK
7.14 TheRock layout:

```bash
export PATH=/opt/rocm/core/bin:/opt/rocm/core/lib/llvm/bin:$PATH
export ROCM_PATH=/opt/rocm/core
export HIP_PATH=/opt/rocm/core
```

You need:

- ROCm Core SDK **7.14 or newer** and a visible AMD RDNA GPU;
- Rust 1.85 or newer for source builds;
- a C compiler for the ABI-only smoke, and `hipcc` for the real-GPU C smoke;
- Python 3.9 or newer plus `maturin` for the Python module.

`ROCR_VISIBLE_DEVICES` selects the physical devices visible to both ROCr and
HIP. After filtering, Redline device ordinal `0` means the first visible GPU:

```bash
ROCR_VISIBLE_DEVICES=0 your_command
```

### Architecture coverage

| Path | Coverage |
| --- | --- |
| Public ROCr/AQL replay | Architecture-neutral; exercised on gfx1010, gfx1030, gfx1100, gfx1151, and gfx1201 |
| C/Rust retained direct PM4 | Family-specific GFX10/GFX11 and GFX12 encoders; a device-family mismatch fails closed |
| Automatic independent-queue policy | Q2 on gfx1100 and gfx12, Q4 on other measured gfx11 devices, Q1 on unswept gfx10 |
| Python `Gpu.build()` | GFX12 direct-PM4 encoder today; use the C/Rust paths for architecture-dispatched GFX10/GFX11 replay |

Legacy direct PM4 accepts zero-scratch HSA kernels whose implicit inputs are the
kernarg pointer and optional private-segment buffer. Unsupported scratch,
queue, dispatch, or flat-scratch contracts are rejected rather than replayed
with guessed state.

## The shared execution contract

Every integration ultimately supplies the same information.

1. **Exact code object and symbol.** Load the HSACO bytes that contain the
   kernel. Symbols commonly carry the `.kd` suffix; use the name present in the
   code-object metadata.
2. **Launch geometry.** Redline APIs use global work-item counts for `grid` and
   local work-item counts for `block`.
3. **Packed kernarg segment.** Pack pointers and scalars at the offsets declared
   by the kernel metadata. Device pointers must remain GPU-accessible through
   replay. Query the expected segment size instead of assuming a host struct
   layout.
4. **Dependencies and memory access.** The graph API takes explicit resource
   reads/writes and rejects unordered hazards. The lower-level PM4 builder takes
   explicit dependency boundaries such as `rl_pm4_wait_rmw`.
5. **Object lifetimes.** Keep the GPU binding, loaded module, code object, and
   every referenced allocation alive through the last replay. Destroy the
   retained IB before the module and GPU.

A verified Radiowave manifest is optional. Supplying one binds the manifest to
the exact code-object bytes and allows Redline to use a narrower VMEM-only
consumer boundary where inspection proves it safe. Missing or ambiguous
certification retains the broader same-agent scalar/vector boundary.

The direct retained lifecycle is:

```text
select GPU -> load HSACO -> pack kernargs -> record dispatches and waits
           -> finalize once -> patch changed kernargs -> replay and wait
           -> free IB -> free module -> free GPU
```

`finalize` consumes its builder. `replay` waits for completion before returning,
so patching a retained kernarg after one replay and before the next is
race-free. Do not mutate kernargs while a replay is in flight.

## C/C++ engine API

Use the C ABI when an engine already owns HIP allocations, code objects, and
its fixed launch sequence.

### Build

```bash
cargo build --release -p redline-capi
```

Artifacts:

```text
crates/redline-capi/include/redline_dispatch.h
target/release/libredline_dispatch.so
target/release/libredline_dispatch.a
```

There is not yet a CMake package, pkg-config file, or install target. Point your
build at the header and one of the emitted libraries.

### Validate the ABI without a GPU launch

```bash
cc crates/redline-capi/examples/smoke.c \
  -I crates/redline-capi/include \
  -L target/release \
  -Wl,-rpath,"$PWD/target/release" \
  -lredline_dispatch -lpthread -ldl -lm \
  -o /tmp/redline-capi-smoke

/tmp/redline-capi-smoke
```

Expected output includes:

```text
abi=1 lanes=1
C-ABI smoke OK
```

This smoke validates graph recording, hazard compilation, plan fingerprinting,
and C ownership. Its `launch_mock` call does not submit GPU work.

### Retained GPU dispatch

The core low-level sequence is:

```c
RlGpu *gpu = rl_gpu_new(0);
RlModule *module = NULL;
int rc = rl_gpu_load_module(gpu, hsaco_bytes, hsaco_len, &module);

long kernarg_size = rl_module_kernarg_size(module, "my_kernel.kd");
/* Pack exactly kernarg_size bytes using the kernel ABI. */

RlPm4Builder *builder = rl_pm4_builder_new(gpu);
rc = rl_pm4_dispatch(builder, module, "my_kernel.kd",
                     grid_x, grid_y, grid_z,
                     block_x, block_y, block_z,
                     dynamic_group_bytes, kernarg, kernarg_len);

RlPm4Ib *ib = NULL;
rc = rl_pm4_finalize(gpu, builder, &ib); /* consumes builder */
rc = rl_pm4_replay(ib);                 /* submit + completion wait */

rl_pm4_ib_free(ib);
rl_module_free(module);
rl_gpu_free(gpu);
```

Check every return value against `RL_OK`; the header defines stable error
classes for null inputs, UTF-8, recording, compilation, replay, handles, and
certification. The complete correctness-gated example is
[`crates/redline-capi/examples/gpu_smoke.c`](../crates/redline-capi/examples/gpu_smoke.c).

To build and run that example with its included counter kernel:

```bash
export GPU_ARCH=gfx1201  # replace with the exact target reported by rocminfo

hipcc --genco --offload-arch="$GPU_ARCH" \
  bench/floor_kernel_ctr.hip -o /tmp/redline-counter.co

hipcc -x hip crates/redline-capi/examples/gpu_smoke.c \
  -I crates/redline-capi/include \
  -L target/release \
  -Wl,-rpath,"$PWD/target/release" \
  -lredline_dispatch -lpthread -ldl -lm \
  -o /tmp/redline-capi-gpu-smoke

ROCR_VISIBLE_DEVICES=0 \
  /tmp/redline-capi-gpu-smoke 256 /tmp/redline-counter.co
```

A successful run ends with:

```text
real-GPU C-ABI gate: counter = 256 / 256 certified=no [PASS]
```

Pass a matching Radiowave JSON manifest as the fourth argument to exercise
certified module loading.

### Build once, patch per token

After finalization, update only the scalars or pointers that changed:

```c
uint32_t position = next_position;
rc = rl_pm4_ib_set_kernargs(
    ib, dispatch_index, position_byte_offset,
    (const uint8_t *)&position, sizeof(position));
rc = rl_pm4_replay(ib);
```

The retained PM4 packet keeps the same kernarg address, so this does not rebuild
the IB. `rl_pm4_finalize_multi` and `rl_pm4_replay_multi` provide one retained IB
per independent public queue; memory used by separate lanes must be independent.
Use `rl_gpu_pm4_queue_count(..., RlQueueAuto, independent_width)` instead of
hard-coding a queue count.

The higher-level C graph path combines resource declarations with real replay:
record nodes with `rl_graph_kernel_ex`, instantiate with
`rl_graph_instantiate`, and launch with `rl_graphexec_launch`.

## Python

The Python module contains two layers:

- `Graph` / `GraphExec` for graph authoring, dependency validation, lane
  inspection, fingerprinting, and mock replay;
- `Gpu` / `Module` / `Pm4Ib` for real module load, allocation, retained PM4
  build, kernarg patching, and replay.

### Install the current checkout

```bash
python3 -m venv .venv-redline
. .venv-redline/bin/activate
python -m pip install --upgrade pip maturin
maturin develop --release --manifest-path crates/redline-py/Cargo.toml

python -c 'import redline_dispatch as rl; print(rl.Graph, rl.Gpu)'
```

To produce an installable wheel instead of installing into the active virtual
environment:

```bash
maturin build --release --strip \
  --manifest-path crates/redline-py/Cargo.toml
```

This creates an abi3 wheel for Python 3.9 or newer. It is a local artifact;
`pip install redline-dispatch` will not work until the package is published.

### Author and inspect a graph

```python
import redline_dispatch as rl

graph = rl.Graph(mode="latency")
activations = graph.buffer("activations", 4096)
project = graph.kernel(
    "project", (32, 1, 1), (64, 1, 1),
    accesses=[
        (activations, 0, 2048, False),
        (activations, 2048, 2048, True),
    ],
)
graph.kernel(
    "consume", (32, 1, 1), (64, 1, 1),
    accesses=[(activations, 2048, 2048, False)],
    deps=[project],
)

exec = graph.instantiate()
exec.launch_mock()
print(exec.lane_count, exec.fingerprint())
```

### Build and replay on a GFX12 GPU

```python
import redline_dispatch as rl

gpu = rl.Gpu(0)
module = gpu.load_module(open("counter.co", "rb").read(), None)
counter = gpu.alloc(4)
kernarg = counter.address().to_bytes(8, "little")

dispatches = [
    ("ctr_k.kd", (1, 1, 1), (1, 1, 1), 0, kernarg, True)
    for _ in range(256)
]
ib = gpu.build(module, dispatches)
ib.replay()
assert counter.read_u32(0) == 256
```

A dispatch tuple is:

```text
(symbol, grid, block, dynamic_group_bytes, packed_kernarg_bytes, serialize)
```

`serialize=True` inserts a safe RMW boundary before the next consumer. A
verified VMEM-only next consumer gets the narrower boundary; uncertified or
ambiguous code uses the generic fail-closed boundary.

Run the complete GPU example after compiling the counter code object as shown
in the C section:

```bash
ROCR_VISIBLE_DEVICES=0 \
  python crates/redline-py/examples/gpu_smoke.py \
  256 /tmp/redline-counter.co
```

For retained decode, call
`ib.set_kernargs(dispatch_index, bytes, byte_offset=...)` between replays. See
[`crates/redline-py/examples/decode_kernargs.py`](../crates/redline-py/examples/decode_kernargs.py).

## Existing hipGraph application

Build the non-Python interposer:

```bash
cargo build --release -p redline-hipgraph

LD_PRELOAD="$PWD/target/release/libredline_hipgraph.so" /usr/bin/true
```

Then preload the same artifact into the HIP graph process:

```bash
LD_PRELOAD="$PWD/target/release/libredline_hipgraph.so" your_hip_app
```

Supported module-loaded and static-fatbin graph captures can resolve to
Redline's retained PM4 path. Unsupported graph operations fall through to the
real HIP implementation; the preload library is not a complete HIP runtime
replacement.

The optional Python control module requires the `python` feature and a symlink
to the same backing shared object so capture and interposition share one set of
Rust statics. Follow the exact build and lifecycle instructions in
[`crates/redline-hipgraph/README.md`](../crates/redline-hipgraph/README.md).

## Rust

Until the crates are published, use a path dependency from a source checkout:

```toml
[dependencies]
redline-dispatch = { path = "../redline/crates/redline-dispatch" }
```

The graph API uses a familiar create → add nodes → instantiate → replay shape,
with explicit buffer accesses added so Redline can derive correct minimal
fences. Run the architecture-neutral migration example:

```bash
cargo run --release -p redline-dispatch --example hipgraph_migration
```

That example uses `MockBackend` to demonstrate graph construction and the
compiled replay plan. For a real public-ROCr retained replay using the counter
code object built above:

```bash
ROCR_VISIBLE_DEVICES=0 \
REDLINE_GRAPH_HSACO=/tmp/redline-counter.co \
REDLINE_GRAPH_SYMBOL=ctr_k \
  cargo run --release -p redline-dispatch --example graph_launch_smoke
```

A successful run prints:

```text
OBSERVED=2 EXPECTED=2
```

Use the native APIs when you need custom artifact catalogs, resource bindings,
HIP multistream fallback, public-AQL lowering, or direct access to the
per-generation PM4 encoders.

## Failure behavior and troubleshooting

Redline fails closed when it cannot prove a replay contract.

| Symptom | Check |
| --- | --- |
| `rl_gpu_new` returns null or `Gpu(0)` raises | Confirm ROCm ≥7.14, `ROCR_VISIBLE_DEVICES`, and the `/opt/rocm/core` environment |
| Module or symbol lookup fails | Compile for the exact GPU architecture and use the symbol spelling in HSACO metadata, commonly `name.kd` |
| Kernarg recording fails | Query `rl_module_kernarg_size` / `module.kernarg_size()` and pack the exact ABI offsets |
| PM4 compilation rejects the kernel | Check scratch, private-segment, implicit-user-SGPR, LDS, and device-family requirements |
| Output is stale or wrong | Declare graph resource accesses or insert the required low-level RMW dependency; keep device allocations live |
| Multiqueue stalls or aliases data | Use the automatic queue policy and ensure every lane has an independent memory footprint |
| Preload remains on HIP | The captured operation or kernel contract is unsupported; inspect with the interposer API rather than assuming PM4 replay |

C functions return `RL_OK` or a negative `RL_ERR_*` class. Rust APIs return typed
errors. Python converts those failures to exceptions. Do not suppress a
certification or compilation error and continue with a partially constructed
retained object.

## Distribution and publishing

### Available now

- A source-built C shared/static library and public header.
- A locally built abi3 Python wheel.
- Source-built hipGraph preload and Rust APIs.

These are sufficient for developers with repository access. External
developers need versioned artifacts and a public source/release location.

### Intended release contract

1. **PyPI:** publish `redline-dispatch` abi3 wheels through
   [`.github/workflows/publish-pypi.yml`](../.github/workflows/publish-pypi.yml)
   using PyPI trusted publishing or a scoped token. Validate the uploaded wheel
   on a ROCm host before calling the release complete.
2. **C SDK:** attach a versioned Linux x86-64 archive to the same GitHub Release:

   ```text
   redline-c-sdk-VERSION-linux-x86_64/
   ├── include/redline_dispatch.h
   ├── lib/libredline_dispatch.so
   ├── lib/libredline_dispatch.a
   ├── LICENSE
   ├── NOTICE
   └── SHA256SUMS
   ```

3. **Later, if demanded:** add CMake/pkg-config metadata and an apt/deb package.
   Publish Rust crates only after internal path dependencies have versioned
   crates.io metadata.

Do not document a PyPI install or C SDK download as available until those
artifacts have been published and independently installed on a clean ROCm host.
