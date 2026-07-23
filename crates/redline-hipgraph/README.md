# redline-hipgraph

`redline-hipgraph` provides the HIP graph C ABI and an optional Python control
module from the same crate. The C exports always include `hipGraph*`,
`__hipRegister*`, and kernel-launch interposition. Enabling the `python` feature
adds the `redline_hipgraph` PyO3 module to that same shared object.

The Python functions and C interposer call the same implementations and use the
same Rust `GLOBAL` and `RUNTIME` statics. In a Python process, a capture started
through the module is therefore populated by `hipLaunchKernel` calls intercepted
from hipEngine kernel shared libraries.

## Non-Python interposer build

The default feature set does not include PyO3. Use this variant when preloading
into an arbitrary non-Python process:

```sh
PATH=/opt/rocm/core/bin:$PATH \
ROCM_PATH=/opt/rocm/core HIP_PATH=/opt/rocm/core \
  cargo build --release -p redline-hipgraph

LD_PRELOAD="$PWD/target/release/libredline_hipgraph.so" /usr/bin/true
```

This build has no undefined Python C-API symbols and is safe to load without a
Python runtime. It does not export `PyInit_redline_hipgraph`.

## Python/hipEngine build

Enable the opt-in `python` feature for hipEngine. It resolves PyO3 0.24 with the
`extension-module` and `abi3-py38` features:

```sh
PATH=/opt/rocm/core/bin:$PATH \
ROCM_PATH=/opt/rocm/core HIP_PATH=/opt/rocm/core \
  cargo build --release -p redline-hipgraph --features python

ln -sfn libredline_hipgraph.so target/release/redline_hipgraph.so
LD_PRELOAD="$PWD/target/release/libredline_hipgraph.so" \
PYTHONPATH="$PWD/target/release" \
  python3 -c 'import redline_hipgraph as r; print(r.available())'
```

Cargo names the cdylib `libredline_hipgraph.so`, while Python looks for
`redline_hipgraph.so`. Keep the Python name as a symlink, not a copy, and use
its same backing file for `LD_PRELOAD`. The dynamic loader then reuses the
already-preloaded object when Python imports it, preserving one capture and
registry state. Loading a copied shared object would create a second set of
Rust statics and break capture sharing.

PyO3's `extension-module` feature intentionally leaves Python C-API symbols for
the interpreter to provide instead of adding a `libpython` dependency. The
Python-enabled artifact must therefore be loaded only in a Python process; use
the default build for non-Python preload targets.

## Python API

- `available() -> bool`: reports whether ROCr can initialize and a visible GPU
  can be selected; initialization failures return `False` rather than raising.
- `capture_begin(stream: int) -> None`: begins capture on an integer-valued
  `hipStream_t`.
- `capture_end(stream: int) -> int`: ends capture, instantiates it, destroys the
  intermediate graph, and returns an opaque graph-exec handle.
- `launch(exec: int, stream: int) -> int`: launches the retained graph and
  returns its HIP status code (`0` is `hipSuccess`).
- `is_pm4(exec: int) -> bool`: reports whether the exec resolved to Redline's
  retained PM4 path rather than the native HIP fallback.
- `exec_destroy(exec: int) -> None`: destroys an exec returned by
  `capture_end`.

Call `exec_destroy` when the exec is no longer needed. Capture/lifecycle errors
raise Python exceptions whose messages include the HIP error name and numeric
status.
