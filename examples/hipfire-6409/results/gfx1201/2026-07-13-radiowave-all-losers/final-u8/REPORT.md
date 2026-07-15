# Radiowave all-loser tuning report

This is the correctness-gated aggregate of two complete gfx1201 runs of the
Hipfire-native ROCm issue 6409 matrix. Each aggregate row is the median of 14
raw GPU samples (two replicates, three warmups and seven measured samples per
replicate). All 133 rows passed the four backend CPU oracle in both replicates.

## Result

| Backend | 1st | 2nd | 3rd | 4th | Wins | Win rate |
|---|---:|---:|---:|---:|---:|---:|
| Redline | **110** | 23 | 0 | 0 | **110/133** | **82.71%** |
| Vulkan/RADV | 21 | 89 | 6 | 17 | 21/133 | 15.79% |
| HipGraph | 1 | 8 | 38 | 86 | 1/133 | 0.75% |
| direct HIP | 1 | 13 | 89 | 30 | 1/133 | 0.75% |

Redline beats Vulkan strictly in **112/133 rows (84.21%)**. The difference
between 112 pairwise wins and 110 first places is two large serial samplers:
Redline beats Vulkan, but HipGraph wins the one-row case by 0.51% and direct
HIP wins the four-row case by 0.14%.

| Mode | RL first | Strict RL>Vulkan | Losses | Median RL/Vulkan |
|---|---:|---:|---:|---:|
| Serial RMW latency | 29/45 | 31/45 | 14 | 0.9461x |
| Independent throughput | 41/45 | 41/45 | 4 | 0.5202x |
| Single-kernel aggressive | **40/43** | **40/43** | 3 | 0.7687x |
| **All modes** | **110/133** | **112/133** | **21** | **0.7687x** |

The fresh targeted64/Radiowave baseline was 97/133 first places and 98/133
strict wins over Vulkan. This pass therefore adds **13 first places** and
**14 pairwise Vulkan wins** without changing the algorithms, using Vulkan ISA,
or bypassing the ordinary HIP code-object path.

## Accepted HIP tuning

- `dispatch_tiny` uses a 32-thread workgroup under the Radiowave policy. Only
  lane zero is live, so this preserves the work and oracle while removing seven
  deliberately idle wave32 waves. Redline now wins every aggressive dispatch
  row, including `grid=8192`.
- Packed Q8/Q4/Q6/scalar dot loads each eight-word tile as two aligned B128
  buffer requests. All **12/12 packed-dot rows** now beat Vulkan.
- Sampler and two-stage reduction use the reviewed 32-bit-offset buffer
  load/store path. Every former sampler loss and the large serial two-stage
  loss flip to Redline.
- Interleave uses B128 input tiles, wave64, buffer output for independent
  throughput, and a single 64-thread wave for aggressive latency. The focused
  21-sample large aggressive run is a Redline win; the full aggregate is a
  0.45% Vulkan crossover.
- VOPD variants use wave64, buffer output, a strength-reduced bias recurrence,
  and source-shaped unroll factors: 8 for independent FMA, 2 for dependent
  FMA, 8 for mixed integer/float, and 2 for dequant. Independent FMA now beats
  Vulkan in both independent and aggressive modes.
- `radiowave_tuned` is now the harness default. It selects the wave, workgroup,
  and kernel variant per family and timing mode while Vulkan retains its native
  shader and geometry.

The selected code objects have zero private-segment bytes and zero SGPR/VGPR
spills. The final wave64 VOPD kernels use 38, 10, 14, and 13 VGPRs respectively.
The final build uses upstream ROCm 7.2 hipcc with no experimental LLVM scheduler
flags.

## What remains

The three aggressive losses isolate the remaining kernel-side surface because
the timed Redline tape contains one dispatch and no dependency fence:

| Aggressive row | RL slower than Vulkan | Classification |
|---|---:|---|
| interleave4, `n=32768` | 0.45% | measurement crossover; focused 21-sample run favors Redline |
| mixed-int-float VOPD | 18.88% | substantive LLVM/ACO instruction scheduling gap |
| dequant-like VOPD | 19.31% | substantive bit-extract/conversion scheduling gap |

Independent throughput has four losses: a 0.12% 941-dispatch tie, large gather
at 22.40%, mixed VOPD at 19.89%, and dequant VOPD at 17.01%. These are the next
clean codegen targets.

Serial latency has 14 losses. Five are dispatch-only chains (37% to 157%),
which directly expose Redline's current safe RMW dependency boundary versus
Vulkan's cheaper barrier. The coalesced, gather, interleave, and VOPD rows mix
that fixed dependency cost with remaining kernel codegen differences. They
must not be “fixed” by removing a real read-after-write dependency.

## Rejected variants

The experiment directory preserves the negative evidence. Blanket buffer
conversion regressed dispatch and coalesced access; B128 coalesced loads harmed
overlap; an eight-request gather window harmed independent throughput; VOPD
block128, two-output tiling, explicit bit extraction, reduced-register loops,
and broad scheduler/ILP flags all lost or emitted identical relevant ISA. The
accepted source is the best correctness-gated combination from those sweeps,
not a pile-up of every attempted optimization.

## Artifacts

- [Machine-readable aggregate](aggregate.json)
- [Replicate 1](rep1/results.json) and [report](rep1/REPORT.md)
- [Replicate 2](rep2/results.json) and [report](rep2/REPORT.md)
- [Fresh pre-tuning baseline](../baseline/results.json)
- [Focused packed B128 run](../packed-b128/REPORT.md)
- [Focused dispatch workgroup run](../dispatch-wg32/REPORT.md)
- [Focused aggressive interleave run](../interleave-block64-aggressive/REPORT.md)

HIP, HipGraph, and Redline load the same per-row Radiowave/hipcc code object.
Vulkan runs the matched GLSL algorithm through RADV/ACO. Thus the aggressive
results separate retained submission from compiler codegen without pretending
that the two compiler stacks emit identical machine code.
