<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# Hipfire-native ROCm issue 6409 benchmark

This is a standalone Rust rewrite of the ROCm issue 6409 microbenchmark. It
uses Hipfire's Rust HIP bridge as the harness, Redline's retained GFX10/GFX11/
GFX12 PM4 paths, and a native Vulkan/RADV backend. It does not import hipEngine
and it does not use HIP graph capture to construct the Redline tape.

`cargo run --release --bin hipengine_exact` and
`cargo run --release --bin hipengine_exact_memory` are diagnostic exceptions.
They load HipEngine's already-built, hash-certified VOPD and memory/waitcnt
`.redline.co` artifacts without recompiling them, reconstruct their exact
launch ABI and deterministic fixtures in Rust, and compare HipEngine-compatible
replay against Hipfire's kernarg-reuse and safe ownership-handoff policies.
This leaves the normal benchmark independent while providing bytecode-identical
parity checks when a HipEngine result directory is available.

For comparison runs on an unlocked APU, use a long precondition; the default
short suite warmup is not sufficient to stabilize clocks:

```bash
cargo run --release --bin hipengine_exact -- \
  --preheat 1000 --warmups 1000 --samples 21 \
  --out results/gfx1201/hipengine-exact-vopd.json

cargo run --release --bin hipengine_exact_memory -- \
  --preheat 1000 --warmups 1000 --samples 21 \
  --out results/gfx1201/hipengine-exact-memory.json
```

The initial gfx1201 exact-artifact run is documented in
[`results/gfx1201/2026-07-13-hipengine-exact-vopd-REPORT.md`](results/gfx1201/2026-07-13-hipengine-exact-vopd-REPORT.md).
The memory and production-family source control, including the distinction
between matched HipEngine Vulkan and Hipfire's stronger custom Vulkan rows, is
documented in the
[exact-source report](results/gfx1201/2026-07-13-exact-source-gap-closure/REPORT.md).

## Default HipEngine-parity suite

The default `--matrix hipengine` profile now covers the pinned HipEngine
`f2c3ad6` benchmark row set 1:1: the same families, operations, shape/sweep
axes, repetition counts, and serial/independent modes. There are 112 core
configurations plus eight dispatch controls in each mode, or **120
configurations per mode and 240 rows total**.

| Family | Configurations per mode |
|---|---:|
| Dispatch/grid | 8 |
| Geometry | 8 |
| Reduction | 24 |
| Memory/waitcnt | 8 |
| Packed dot | 8 |
| VOPD | 8 |
| Sampler | 12 |
| Two-stage reduction | 16 |
| Selected-dual Q4 | 5 |
| Selected-down Q6 | 3 |
| Dense Q8 | 20 |
| **Total** | **120** |

This is row-coverage parity, not a requirement to inherit HipEngine's launch
or codegen choices. Every row is fired through Hipfire's existing optimized
path: native Rust orchestration, Radiowave-selected HIP wave/workgroup/source
variants, Redline retained PM4, and matched GLSL for Vulkan. The
production-shaped rows perform an actual BF16-to-Q8_1 stage and consume that
packed result in the dependent dot stage. HIP, HipGraph, and Redline execute
the identical selected HSACO within a row; Vulkan deliberately exercises
RADV/ACO codegen for the same operation.

The `radiowave` policy no longer owns a benchmark-local list of winning wave,
workgroup, or source-variant decisions. It selects them from Radiowave's typed
recipe catalog. Every result row records the applied recipe IDs and source
lowerings, so a result can be traced back to the policy that produced it.
Promoted evidence is architecture-specific: setting `HIPFIRE_BENCH_ARCH` to an
uncertified target produces the baseline plan and exposes the applicable
recipes as autoresearch candidates; it never silently imports gfx1201 winners.
Pass Hipfire's exported catalog with `--recipe-catalog PATH` or
`RADIOWAVE_RECIPE_CATALOG=PATH`; the built-in reference catalog remains the
default. This closes the loop from `ar radiowave` promotion back into the
benchmark consumer.

Pinned HipEngine has 212 matched core rows because its HIP independent sampler
path rejects 12 rows, and it reports 16 dispatch rows separately. Hipfire can
execute and correctness-gate all 240 rows across all four backends. The former
45-shape/133-row experiment remains available as
`--matrix legacy --include-aggressive` for historical regression comparisons.

The first complete parity run uses three warmups and seven measured GPU samples
per backend and row. All **240/240 rows pass all four CPU oracles**. Redline
takes **180/240 four-way first places (75.00%)** and beats Vulkan pairwise in
**185/240 rows (77.08%)**.

| Mode | RL first | Strict RL > Vulkan | N |
|---|---:|---:|---:|
| Serial RMW latency | 85 | 85 | 120 |
| Independent throughput | 95 | 100 | 120 |
| **All comparable rows** | **180** | **185** | **240** |

Across the full grid, Redline also beats both direct HIP and HipGraph in
217/240 rows. See the
[parity report](results/gfx1201/2026-07-13-hipengine-parity/REPORT.md) and
[machine-readable artifact](results/gfx1201/2026-07-13-hipengine-parity/results.json).

## Native RDNA portability result

The same 240-row matrix now runs through architecture-matched retained PM4 on
gfx1010, gfx1030, gfx1100, and gfx1151. All **960/960 rows** pass all four
backend oracles. In aggregate, Redline takes **537/960 first places (55.94%)**,
finishes first or second in **862/960 (89.79%)**, beats Vulkan pairwise in
**606/960 (63.13%)**, and beats HipGraph in **837/960 (87.19%)**.

| Architecture | RL 1st | RL > Vulkan | RL > HipGraph | Correct rows |
|---|---:|---:|---:|---:|
| gfx1100 | 158/240 | 171/240 | 225/240 | 240/240 |
| gfx1151 | 156/240 | 156/240 | 239/240 | 240/240 |
| gfx1030 | 94/240 | 125/240 | 175/240 | 240/240 |
| gfx1010 | 129/240 | 154/240 | 198/240 | 240/240 |

These are architecture-safe baseline plans: no non-gfx1201 performance recipe
is promoted yet. The result certifies the ROCr-native retained command path and
keeps the remaining per-architecture kernel tuning visible. See the
[cross-architecture report](results/2026-07-14-rdna-rocr-native/REPORT.md) and
[machine-readable aggregate](results/2026-07-14-rdna-rocr-native/aggregate.json).

## Multi-queue retained PM4 result

Independent rows can now stripe disjoint operations across one retained PM4 IB
per public ROCr queue. Redline release-publishes every vendor packet before it
rings any doorbell, waits all lane completion signals under one finite timeout,
and measures the same earliest-start through latest-end GPU makespan used by
the Vulkan control. Serial RMW and one-kernel aggressive rows remain on the
existing single queue.

The causal Q1/Q4 comparison uses identical current Radiowave HSACO hashes,
three warmups, seven measured GPU samples, and all 120 independent rows. Every
selected Redline and Vulkan result passes its CPU oracle.

| Architecture | Q1 RL > Vulkan | Q4 RL > Vulkan | Median Q1/Q4 RL speedup | Losses converted | Regressions |
|---|---:|---:|---:|---:|---:|
| gfx1100 | 67/120 | **100/120** | **1.147x** | 34 | 1 |
| gfx1151 | 68/120 | **110/120** | **1.058x** | 42 | 0 |

The corresponding full serial-plus-independent Q4 certifications take
**189/240 rows on gfx1100 (78.75%)** and **206/240 on gfx1151 (85.83%)**, with
240/240 correctness-passing rows on each GPU. See the
[gfx1100 Q1 control](results/gfx1100/2026-07-14-redline-current-q1-independent/REPORT.md),
[gfx1100 Q4 certification](results/gfx1100/2026-07-14-redline-multiqueue-q4/REPORT.md),
[gfx1151 Q1 control](results/gfx1151/2026-07-14-redline-current-q1-independent/REPORT.md),
and [gfx1151 Q4 certification](results/gfx1151/2026-07-14-redline-multiqueue-q4/REPORT.md).

The local gfx1201 reproduction has a different queue-count optimum. With the
same Radiowave HSACO hashes in every run, Q1 wins **88/120** independent rows,
Q2 wins **99/120**, and Q4 wins only **15/120**. Q2 gives a **1.028x** median
Redline speedup over Q1, converts 11 losses with no regressions, and takes
**187/240 rows (77.92%)** in the full serial-plus-independent certification.
Q4 is a useful negative control: its median independent Redline time is
**6.525x** Q2, so four-way queue fan-out is not a portable default for gfx12.
All rows in all four runs pass their CPU oracle. See the
[gfx1201 Q1 control](results/gfx1201/2026-07-14-redline-current-q1-independent/REPORT.md),
[gfx1201 Q2 causal run](results/gfx1201/2026-07-14-redline-multiqueue-q2-independent/REPORT.md),
[gfx1201 Q2 certification](results/gfx1201/2026-07-14-redline-multiqueue-q2/REPORT.md),
and [gfx1201 Q4 negative control](results/gfx1201/2026-07-14-redline-multiqueue-q4/REPORT.md).

The harness therefore defaults `--redline-queues auto`: gfx11 and gfx1151
resolve to Q4, gfx12 resolves to Q2, and gfx10 or an unknown future family
fails closed to Q1 until a queue sweep is certified. `--redline-queues 1|2|4`
remains an explicit diagnostic override. Every result artifact records both
the requested policy and the resolved queue count.

The post-default Redline/Vulkan tuning validation exercises all 240 rows with
three warmups and seven measured GPU samples. Auto resolves to Q4 on both
devices, and every result passes its CPU oracle.

| Architecture | RL first | Strict RL > Vulkan | Ties | Correct rows |
|---|---:|---:|---:|---:|
| gfx1100 | 196/240 (81.67%) | 194/240 (80.83%) | 2 | 240/240 |
| gfx1151 | 203/240 (84.58%) | 203/240 (84.58%) | 0 | 240/240 |

See the latest [gfx1100 Auto/Q4 validation](results/gfx1100/2026-07-14-auto-q4-validation/REPORT.md)
and [gfx1151 Auto/Q4 validation](results/gfx1151/2026-07-14-auto-q4-validation/REPORT.md).
These are the deliberate two-backend tuning matrix, not a replacement for the
four-backend portability certification above.

The next shared kernel target is `q6-x8-selected-down`: its prequantized and
quantize-plus-dot rows lose in both timing modes on both gfx1100 and gfx1151,
with the independent prequantized row still 355.13% and 165.49% behind Vulkan,
respectively. That consistent decode-shaped deficit is a stronger Radiowave
code-generation target than the architecture-specific near-ties. The separate
high-grid dispatch losses remain the next retained-queue scheduling target.

## Legacy Radiowave and minimal-boundary result

The fail-closed Radiowave/Redline path raises Redline to **121/133 four-way
first places (90.98%)** and **121/133 strict wins over Vulkan (90.98%)**. Safe
serial RMW replay takes **39/45** rows, while the aggressive one-dispatch mode
takes **40/43**. The aggregate is the median of 14 raw GPU samples from two
counterbalanced replicates per row; all 266 row-runs passed all four backend
CPU oracles.

| Mode | RL first | Strict RL>Vulkan | Median RL/Vulkan |
|---|---:|---:|---:|
| Serial RMW latency | **39/45** | **39/45** | 0.7365x |
| Independent throughput | **42/45** | **42/45** | 0.5258x |
| Single-kernel aggressive | **40/43** | **40/43** | 0.7792x |
| **All modes** | **121/133** | **121/133** | **0.6923x** |

The matched generic same-agent boundary takes 115/133 overall and 32/45
serial rows. The certified path is 10.2% faster at the median across all 45
serial rows and adds seven serial wins. On the original 941-launch RMW floor,
Redline is **0.7608 us/dispatch** versus Vulkan at **1.0329 us**. The earlier
generic-safe Redline control remains 1.4760 us.

The kernel changes are ordinary HIP source transformations selected by
Radiowave: a 32-thread
dispatch-floor workgroup, aligned B128 packed-dot/interleave loads, buffer
loads and stores where the 32-bit-offset contract is proven, mode-specific
interleave geometry, and per-variant VOPD wave/unroll policy. Radiowave now
also inspects every emitted consumer: mutable resource reads proven to be VMEM
use `CS_PARTIAL_FLUSH` plus vector/merged-L1 invalidation (`GCR=0x00300`);
scalar or ambiguous consumers fail closed to scalar/vector/merged-L1
invalidation (`0x00380`). The serial tape reuses one immutable kernarg block
when arguments do not change, keeping its allowed scalar prologue hot and
letting stateful PM4 elide redundant user-data writes. All 23 emitted kernels
in the original boundary run certified the VMEM path. The current 26-kernel
wave objects add a spill-free wave64 dequant chunk plus two isolated scheduler
experiments. Grouping 16 exact integer terms improves the selected dequant row
by 10.05-10.67% across the three modes, although it does not yet flip Vulkan.
See the
[current certification report](results/gfx1201/2026-07-13-dequant-chunk16-final/comparison/REPORT.md),
[machine-readable aggregate](results/gfx1201/2026-07-13-dequant-chunk16-final/comparison/aggregate.json),
and [scheduler-profile experiment](results/gfx1201/2026-07-13-scheduler-profiles/REPORT.md).

## Scheduler-profile result

Radiowave now builds five separately identified wave32/wave64 code-object
pairs: upstream default, max ILP, iterative ILP, maximum memory-clause, and a
pipelined max-ILP profile. The harness selects one with
`--scheduler-profile`; HIP, HipGraph, and Redline all use that same object, and
the profile plus both object hashes are recorded in every result artifact.

This is useful compiler control, but the first sweep did **not** justify a new
default. Max-ILP reduces static wave64 instructions from 509 to 445 for mixed
VOPD and from 651 to 587 for chunk-16 dequant. Its repeat measurements do not
produce a consistent normalized win over upstream scheduling. Iterative ILP
raises pressure to 29/39 VGPR on those kernels and also fails the timing gate.
The B32 interleave candidate successfully forces four consecutive buffer loads
and one `s_clause`, but is 3.27% slower than the selected B128 load at the
large aggressive shape. The paired mixed hash saves one wait and one delay
instruction but has not repeated a stable timing win. All remain explicit
experiments; the certified 121/133 selection stays on `default`, B128
interleave, and the original mixed hash. A fresh 133-row default regression
reproduces **121/133 firsts**, 12 second-place finishes, and no third/fourth
places; see its
[machine-readable result](results/gfx1201/2026-07-13-scheduler-profiles/default-regression/results.json).

The older sections below preserve the progression and controlled baselines.

## Historical first low-margin pass

The closest stable Vulkan losses were traced to hipcc's 64-bit global address
construction.  Ordinary HIP buffer builtins plus a four-request gather window
raise the targeted-wave64 result from 94/133 to **98/133** aggregate firsts:
independent packed Q8/Q4 and aggressive gather now beat Vulkan.  All 133 rows
pass correctness, and the accepted W32/W64 kernels have no private memory or
spills.  See the [tuning report](results/gfx1201/2026-07-12-loser-tuning/REPORT.md)
and [aggregate artifact](results/gfx1201/2026-07-12-loser-tuning/full-buffer/aggregate.json).

## Historical controlled wave-policy result

A two-pass counterbalanced comparison holds Redline submission semantics
constant while selecting all-wave32, targeted-wave64, or blanket-wave64 HIP.
Each aggregate row is the median of 14 raw GPU samples, and all six runs passed
all 133 four-way CPU oracles.

| Policy | RL first | Strict RL>Vulkan | Median RL/Vulkan |
|---|---:|---:|---:|
| all wave32 | 82/133 | 84 | 0.8825x |
| targeted wave64 | **94/133** | **94** | **0.8485x** |
| blanket wave64 | 92/133 | 93 | 0.8533x |

Targeted wave64 is fixed to Q4 selected x2, Q6 x8, dense Q8 x4, and
dependent-FMA VOPD. It is the best tested placement policy. Blanket wave64 is
slightly better for aggregate independent throughput because additional VOPD,
four-row sampler, and coalesced-memory shapes benefit, but packed dot and
two-stage reduction clearly prefer wave32. See the
[controlled comparison](results/gfx1201/2026-07-12-wave-policy-comparison/REPORT.md)
and [aggregate artifact](results/gfx1201/2026-07-12-wave-policy-comparison/aggregate.json).

## Pre-comparison blanket-wave64 run

The final correctness-gated run used three warmups and seven measured GPU
samples for every backend/row. All 133 four-way rows passed their CPU oracle.
The run adds an aggressive single-kernel mode and selects ordinary hipcc
wave64 code for every family where Vulkan beat Redline in the preceding run.

| Backend | 1st | 2nd | 3rd | 4th | Wins | Win rate |
|---|---:|---:|---:|---:|---:|---:|
| Redline | 91 | 40 | 2 | 0 | 91/133 | **68.42%** |
| Vulkan/RADV | 41 | 74 | 4 | 14 | 41/133 | 30.83% |
| HipGraph | 1 | 8 | 35 | 89 | 1/133 | 0.75% |
| direct HIP | 0 | 11 | 92 | 30 | 0/133 | 0.00% |

Redline's mode split is the useful crossover signal:

| Mode | RL 1st | RL 2nd | RL 3rd | RL 4th | RL/Vulkan wins | RL/Vulkan median |
|---|---:|---:|---:|---:|---:|---:|
| Serial RMW latency | 21 | 22 | 2 | 0 | 22/45 | 1.0299x |
| Independent throughput | 34 | 11 | 0 | 0 | 34/45 | 0.6091x |
| Single-kernel aggressive | 36 | 7 | 0 | 0 | 35/43 + 1 tie | 0.8381x |

Across all modes, Redline beats Vulkan strictly in 91/133 rows (68.42%),
HipGraph in 131/133 (98.50%), and direct HIP in 132/133 (99.25%). The complete
four-way placements, every loss and its exact margin, raw samples, environment,
commits, submission policy, and both code-object hashes are in the
[final report](results/gfx1201/2026-07-12-wave64-aggressive/REPORT.md) and
[machine-readable artifact](results/gfx1201/2026-07-12-wave64-aggressive/results.json).

### What the new levers changed

On the original serial-plus-independent 90-row matrix, Redline moves from
46 to 55 first places and from 49 to 56 strict pairwise wins over Vulkan.
Nine former Vulkan losses flip to Redline, including all three serial Q4
selected shapes, two serial dense-Q8 shapes, large independent Q6 and dense-Q8,
independent Q4, and dependent-FMA VOPD. Two noisy/negative crossings move the
other way: the 941-dispatch independent no-op and independent packed Q6.

The aggressive single-kernel result is the direct answer to the submission
question: Redline takes 36/43 first places (83.72%). Its timed PM4 IB contains
one dispatch and **no entry acquire or dependency fence**. Because each sample
first zeroes the output through HIP, a separate retained acquire-only IB proves
HIP-to-PM4 ownership outside the GPU timestamp window. The end timestamp still
performs the unavoidable completion operation needed to measure and validate
the kernel. The seven Vulkan wins are confined to two large-memory rows, three
VOPD rows, and two 131072-vocabulary sampler rows; their median margin is 6.70%
(0.25% to 28.32%).

## Skill issue or HIP issue?

The controlled answer is: **not a Hipfire harness issue; the residual gap is
localized.**

For each row, HIP, HipGraph, and Redline load the exact same ordinary
LLVM/hipcc offload bundle produced through Radiowave. Redline beating direct
HIP on 132/133 rows and HipGraph on 131/133 rules out the harness as the cause
of the crossover. It also proves that retained PM4 can correctly dispatch
wave32 and wave64 kernels, including one kernel, long chains, multi-workgroup
kernels, and two-stage dependencies, from ordinary HIP code objects.

The 42 non-winning rows divide cleanly:

- Vulkan wins 41. Its median advantage on those rows is 26.29%; the range is
  0.25% to 162.80%. The aggressive losses identify the remaining kernel-side
  ACO-versus-LLVM lowering or scheduling surface without an inter-dispatch
  fence confound. A submission engine cannot rewrite the shared HIP ISA after
  loading it.
- HipGraph wins one serial wave reduction by 0.36%. The two third-place Redline
  rows are large serial samplers, where direct HIP or HipGraph also edge it by
  less than 1.8%. These are small crossover rows, not a transport failure.
- Serial no-op and RMW rows exposed a separate Redline issue at this stage: the
  safe dependency edge paired compute completion with unnecessary global L2
  writeback/invalidation. The current same-agent path retains coherent L2/MALL
  and invalidates only scalar/vector shader caches. Compute completion itself
  remains required for a true non-atomic RMW chain.

The prior hipEngine-harness run reported Redline/Vulkan wins of 186/212
(87.74%, median 0.4788x) and Redline/HIP wins of 157/212 (74.06%, median
0.8342x). Those are not row-for-row comparable: the old suite had a larger
parameter sweep, different kernels, and only three ranked backends. The new
control is stricter and shows both facts simultaneously: hipEngine was not
required for Redline's dispatch win, and Vulkan retains genuine codegen plus
minimal-dependency advantages on specific shapes.

## What is being compared

- `hip`: Hipfire's dlopen HIP bridge, direct `hipModuleLaunchKernel`, HIP-event
  GPU time.
- `hipgraph`: the same Hipfire bridge and the same selected HSACO, captured once and
  replayed under HIP-event GPU time.
- `redline`: public ROCr loads the same selected HSACO; serial rows replay one
  retained PM4 IB, while independent rows may stripe across up to four
  lane-local retained IBs with GPU-written start/end timestamps.
- `vulkan`: matched GLSL algorithms compiled by shaderc for RADV, device-local
  buffers, timestamp queries, and up to four compute queues.

`serial_latency` is a true output read-modify-write chain. HIP uses stream
order, Vulkan inserts compute-write to compute-read/write barriers, and
Redline inserts its architecture-matched safe RMW boundary. Every Redline sample completes a
system ownership acquire after the HIP reset and before the timestamped tape.
`independent_throughput` writes
disjoint output slices and removes cross-operation barriers; HIP and Vulkan use
up to four lanes and Redline uses up to four retained lane-local IBs. Two-stage
reductions retain their internal dependency within each lane in both modes.
`single_kernel_aggressive` executes
exactly one dispatch, excludes two-stage rows, and moves Redline's ownership
acquire outside timing so the timed tape has no dependency fence.

The legacy profile's 45 shapes per mode cover dispatch/grid, FMA geometry, wave and LDS
reductions, coalesced/gather/interleaved memory, packed dot, VOPD patterns,
argmax sampling, two-stage reduction, Q4 selected dual-row reuse, Q6 eight-row
reuse, and dense Q8 four-row reuse.

## HIP kernels

The HIP source is [kernels/hipfire_6409.hip](kernels/hipfire_6409.hip). The
inference-shaped kernels use `sudot4` where the target exposes DOT8 and an
equivalent scalar packed-dot fallback on gfx10, plus wave-aware shuffles, small
LDS wave partials, and row reuse (`Q4 x2`, `Q6 x8`, `Q8 x4`). The build compiles through
the [`radiowave`](../../crates/radiowave/README.md) crate instead of invoking
hipcc directly. Radiowave supplies the accepted buffer-resource load/store API,
emits wave32 and wave64 code objects for every typed scheduler profile from the
same source, and writes an inspection manifest beside each object in Cargo's
build output. Its tuned policy selects wave size, scheduler profile, workgroup
shape, and a reviewed source variant per family and timing mode. The selected
default-profile wave64 metadata reports:

| Kernel | Wave | VGPR | SGPR | LDS | Private segment | Spills |
|---|---:|---:|---:|---:|---:|---:|
| `q4_selected_dual` | 64 | 12 | 23 | 0 | 0 | 0 |
| `q6_x8` | 64 | 21 | 41 | 0 | 0 | 0 |
| `dense_q8` | 64 | 14 | 28 | 0 | 0 | 0 |
| `memory_interleave4_block64` | 64 | 12 | 14 | 0 | 0 | 0 |
| `vopd_independent` | 64 | 38 | 8 | 0 | 0 | 0 |
| `vopd_dependent` | 64 | 10 | 8 | 0 | 0 | 0 |
| `vopd_mixed` | 64 | 14 | 10 | 0 | 0 | 0 |
| `vopd_dequant` | 64 | 13 | 13 | 0 | 0 | 0 |
| `vopd_dequant_chunk16` | 64 | 37 | 9 | 0 | 0 | 0 |

These are tuned, spill-free HIP kernels, not Vulkan ISA launched through
Redline and not a compiler fork. “Optimal” here means the benchmark implements
the intended wave/tile/data-reuse strategy without resource-path pathologies;
it is not a mathematical claim that no future kernel can be faster.

## Reproduce

The harness is pinned to the clean `redline` branch clone at Hipfire commit
`455ffb9dfd6a5712889b504737f88fbbe87d3efe`:

```bash
git clone --branch redline --single-branch --no-tags \
  https://github.com/Kaden-Schutt/hipfire.git engines/hipfire

cd examples/hipfire-6409
cargo build --release

source ../../engines/hipfire/scripts/gpu-lock.sh
gpu_acquire hipfire-6409
./target/release/hipfire-6409-bench \
  --matrix hipengine \
  --wave-policy radiowave \
  --scheduler-profile default \
  --redline-rmw radiowave-vmem \
  --warmups 3 --samples 7 \
  --out results/gfx1201/manual-radiowave/results.json
gpu_release
```

During architecture tuning, omit the already-established HIP and HipGraph
controls and compare only the two relevant paths:

```bash
./target/release/hipfire-6409-bench \
  --matrix hipengine \
  --backends redline,vulkan \
  --wave-policy radiowave \
  --recipe-mode candidates \
  --recipe-allow hipfire.cache.geometry_fma_vmem \
  --recipe-allow hipfire.cache.reduction_wave_vmem \
  --warmups 3 --samples 7 \
  --out results/gfx1100/cache-candidate/results.json
```

This is a tuning smoke, not the final result table. Once recipes are promoted,
rerun with `--backends all` to certify the complete Redline/Vulkan/HipGraph/HIP
matrix and lower the accepted architecture evidence into the catalog.

The build defaults to gfx1201 and honors `HIPFIRE_BENCH_ARCH`, `HIPCC`, and
`GLSLC`. Cargo calls Radiowave's library API; only Radiowave owns the hipcc
command line. `radiowave-vmem` is the default; `same-agent` remains the
fail-closed control and `radv-global` the historical system-wide control. The
upstream `default` scheduler profile is selected unless a certified per-kernel
recipe chooses another profile or `--scheduler-profile` explicitly overrides
the whole run. Each
`hipfire_6409_wave*.radiowave.json` manifest records the source, injected
header, selected scheduler profile, exact compiler command, compiler and output
hashes, kernel resource metadata, scalar/VMEM cache-footprint classification,
memory-operation counts, waits, delay instructions, clauses, and maximum VMEM
run. This host used
ROCm 7.2 hipcc, Mesa/RADV 25.2.8, and shaderc 2025 for
`GL_EXT_integer_dot_product`.

Keep a distinct Cargo target directory for each architecture so its embedded
HSACO cannot be reused by another build. This is the exact hipx four-device
layout used for the RDNA portability run:

```bash
for target in 0:gfx1100 1:gfx1151 2:gfx1030 3:gfx1010; do
  ordinal=${target%%:*}
  arch=${target#*:}
  HIPFIRE_BENCH_ARCH=$arch \
    cargo build --release --target-dir target/$arch --bin hipfire-6409-bench
  ROCR_VISIBLE_DEVICES=$ordinal HIPFIRE_BENCH_ARCH=$arch \
    target/$arch/release/hipfire-6409-bench \
      --matrix hipengine \
      --wave-policy radiowave \
      --scheduler-profile default \
      --redline-rmw radiowave-vmem \
      --warmups 3 --samples 7 \
      --out results/$arch/2026-07-14-rdna-rocr-native/results.json
done
```

The Vulkan backend enumerates AMD devices by PCI identity and selects the same
physical adapter that ROCr selected, rather than assuming Vulkan and HSA use
the same ordinal.
