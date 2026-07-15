<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# Redline on the ROCm/ROCm#6409 matrix

> **Hipfire-native follow-up:** the independent Rust rewrite in
> [`examples/hipfire-6409`](../hipfire-6409/README.md) removes hipEngine from
> the harness, uses tuned spill-free HIP kernels, adds HipGraph as a fourth
> contender, and fixes the earlier multi-workgroup test's block-ID assumption.
> Its final correctness-gated result is Redline 46/90 first places (51.11%),
> with 49/90 pairwise wins over Vulkan, 88/90 over HipGraph, and 89/90 over
> direct HIP. Treat that run as the stricter harness/codegen attribution result;
> the 212-row hipEngine run below remains the historical reproduction of the
> pinned issue suite.

[ROCm#6409](https://github.com/ROCm/ROCm/issues/6409) is not evidence that a
correct retained-PM4 HIP path does not exist. Redline now runs the issue's
pinned hipEngine matrix using ordinary hipcc/LLVM code objects, one retained
GFX12 PM4 IB, and in-IB GPU timestamps. HIP graph capture is used once to
recover the exact kernel and argument values; no HIP graph is replayed or timed.

The benchmark source is pinned at
[`f2c3ad6d74c86e3641ce09ff9fd759eaa6cd75e0`](https://github.com/shisa-ai/hipEngine/tree/f2c3ad6d74c86e3641ce09ff9fd759eaa6cd75e0/benchmarks/micro).
This gfx1201 rerun uses the retained issue sampling (`10` repetitions, `3`
warmups, `5` samples; dispatch `20/5`) in both `serial_latency` and
`independent_throughput` modes.

## Result

The ratio below is `Redline GPU time / Vulkan GPU time`; below `1.0` favors
Redline. “Wins” counts correctness-passing rows with a ratio below one.

| Family | Serial wins | Serial median | Independent wins | Independent median |
| --- | ---: | ---: | ---: | ---: |
| Geometry | 8/8 | 0.403x | 8/8 | 0.477x |
| Reduction variants | 24/24 | 0.413x | 24/24 | 0.389x |
| Memory/waitcnt | 8/8 | 0.927x | 5/8 | 0.965x |
| Packed dot | 0/8 | 1.060x | 0/8 | 1.239x |
| VOPD/VALU | 8/8 | 0.775x | 8/8 | 0.844x |
| Sampler | 12/12 | 0.426x | rejected HIP control | — |
| Two-stage reduction | 16/16 | 0.494x | 16/16 | 0.614x |
| Q4 selected-dual | 3/5 | 0.993x | 5/5 | 0.491x |
| Q6 X8 selected-down | 2/3 | 0.607x | 3/3 | 0.346x |
| Dense Q8_0 | 16/20 | 0.447x | 20/20 | 0.294x |

Across the 212 three-way, correctness-passing rows, Redline beats Vulkan in
186 and HIP in 157. The median ratios are 0.479x versus Vulkan and 0.834x versus
HIP. This directly invalidates the categorical theory that there is no real
Redline PM4 path: the same HIP kernels and buffers execute correctly through a
retained PM4 submission, and that submission wins most of the pinned matrix.

This is not an unconditional “Redline wins everything” result. Packed-dot is a
real remaining loss in both modes. Some serial production rows also remain
Vulkan wins. Those are kernel/codegen or workload-shape targets; they do not
erase the transport result.

The machine-readable audit is
[`summary.json`](../hipengine-6409/results/gfx1201/2026-07-12/summary.json).
The directory also contains every normalized artifact, raw Redline artifact,
environment record, and command log.

## Dispatch/grid row

For the one-block `gmb_noop_kernel` count sweep, Redline is a correct single
retained PM4 IB and decisively beats HIP graph GPU time:

| Mode | Count | HIP | Vulkan | Redline | Redline/HIP | Redline/Vulkan |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| serial | 1 | 17.800 us | 1.600 us | 3.240 us | 0.182x | 2.025x |
| serial | 50 | 3.678 us | 1.272 us | 1.666 us | 0.453x | 1.310x |
| serial | 200 | 3.458 us | 1.171 us | 1.643 us | 0.475x | 1.403x |
| serial | 941 | 3.402 us | 1.146 us | 1.643 us | 0.483x | 1.434x |
| independent | 1 | 20.960 us | 1.600 us | 2.640 us | 0.126x | 1.650x |
| independent | 50 | 9.086 us | 0.038 us | 0.110 us | 0.012x | 2.854x |
| independent | 200 | 8.138 us | 0.023 us | 0.070 us | 0.009x | 3.097x |
| independent | 941 | 7.864 us | 0.019 us | 0.061 us | 0.008x | 3.232x |

These results overturn the “PM4 cannot correctly launch even one HIP kernel”
premise and reproduce the known Redline-over-HipGraph advantage. They do not yet
beat Vulkan's tiny-kernel dispatch floor on this gfx1201 host.

The grid sweep exposed a separate correctness bug: with more than one workgroup,
the current gfx1201 `DISPATCH_DIRECT` encoding does not supply distinct hardware
block IDs to this HIP kernel. Grid 128/1024/8192 Redline timings are therefore
rejected, not reported as wins. Fixing that TGID/tunnel encoding is the next
dispatch-specific task. The artifact is
[`dispatch-matrix.json`](../hipengine-6409/results/gfx1201/2026-07-12/dispatch-matrix.json).

## What is actually being timed

Redline's profiled replay brackets the retained IB on the GPU:

- start: `COPY_DATA` GPU timestamp to fine-grained memory;
- body: ordinary hipcc kernels lowered to PM4 dispatches;
- serial dependency: compute-idle plus the required cache acquire;
- end: bottom-of-pipe `RELEASE_MEM` timestamp;
- submission: one AMD vendor AQL packet and one completion signal.

The GPU clock frequency is queried from the HSA agent. These timestamps replace
the earlier host-wall-only comparison and make the primary timing domain match
HIP events and Vulkan device timestamps.

`serial_latency` inserts a dependency boundary between logical operations.
`independent_throughput` uses disjoint outputs, removes cross-operation barriers,
and preserves internal barriers inside multi-stage operations. All timed output
slices are checked by the pinned runner's oracle.

## HIP-only code-object contract

The retained results do not use the experimental ACO-shaped or direct-SRD
assembly images in this repository. The code path is backend-agnostic:

1. hipcc builds the unchanged HIP source with the exact requested codegen flags;
2. a small manifest records standard AMDGPU kernarg offsets;
3. the unchanged launch closure is captured once to recover exact arguments;
4. Redline loads that ordinary offload bundle through public HSA;
5. the captured graph is destroyed and the retained PM4 IB is replayed.

That distinction matters. These rows measure a submission/runtime replacement
for HipGraph, not a compiler fork and not a Vulkan-ish kernel smuggled into the
Redline column.

## Reproduce

Build the C API and run the pinned matrix:

```bash
cargo build --release -p redline-capi
python3 examples/hipengine-6409/run_matrix.py \
  --hipengine-root /tmp/hipEngine-f2c \
  --out-dir examples/hipengine-6409/results/gfx1201/2026-07-12

cargo build --release -p redline-dispatch --example gmb_floor
python3 examples/hipengine-6409/dispatch_matrix.py \
  --hipengine-root /tmp/hipEngine-f2c \
  --environment examples/hipengine-6409/results/gfx1201/2026-07-12/environment.json \
  --out-dir examples/hipengine-6409/results/gfx1201/2026-07-12

python3 examples/hipengine-6409/summarize_results.py \
  examples/hipengine-6409/results/gfx1201/2026-07-12 \
  --out examples/hipengine-6409/results/gfx1201/2026-07-12/summary.json
```

This host needed shaderc 2025.2 for `GL_EXT_integer_dot_product`; the retained
environment and logs record that toolchain. `run_matrix.py` resumes from
completed artifacts, so interrupted runs do not discard earlier families.
