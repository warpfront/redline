# Radiowave-certified VMEM boundary

This is the final counterbalanced comparison of Redline's generic same-agent
shader-cache boundary against the fail-closed Radiowave-certified VMEM path on
gfx1201. Both arms load the same ordinary hipcc code objects and use the same
Radiowave launch policy. Only Redline's dependency-cache policy changes.

## Result

Each aggregate cell is the median of 14 raw GPU samples: two complete
seven-sample replicates in A/B/B/A order (`same`, `vmem`, `vmem`, `same`). All
532 row-runs passed the CPU oracle for HIP, HipGraph, Redline, and Vulkan.

| Boundary and mode | RL first | Strict RL over Vulkan | Median RL/Vulkan |
|---|---:|---:|---:|
| Generic same-agent, serial | 32/45 | 32/45 | 0.8468x |
| Certified VMEM, serial | **39/45** | **39/45** | **0.7356x** |
| Certified VMEM, independent | **42/45** | **42/45** | **0.5108x** |
| Certified VMEM, aggressive | **40/43** | **40/43** | **0.7702x** |
| Generic same-agent, all modes | 115/133 | 115/133 | 0.7505x |
| **Certified VMEM, all modes** | **121/133** | **121/133** | **0.6993x** |

Across all 45 serial rows, certified VMEM is 10.2% faster than generic-safe at
the median and is faster in 43 rows. The one no-boundary count=1 row and one
0.16% large-Q6 reversal account for the two non-improvements. Across the 44
serial rows that actually emit a certified dependency edge, the median speedup
is 11.0%, with 43 faster and one 0.16% slower.

The original dispatch floor now crosses Vulkan without weakening correctness:

| 941-launch serial RMW | us/dispatch | Relative to Vulkan |
|---|---:|---:|
| Redline generic same-agent | 1.4760 | 1.4366x |
| Vulkan/RADV | 1.0400 | 1.0000x |
| **Redline certified VMEM** | **0.7549** | **0.7259x** |

The certified safe path is 48.9% faster than the generic-safe Redline control
and 27.4% faster than Vulkan on this chain.

## What changed

Radiowave's schema-2 inspection manifest classifies each consumer's mutable
resource reads. Scalar reads are allowed only while they are provably loads
from the live, immutable kernarg pointer. Buffer/global/flat VMEM reads are
tracked separately. A scalar-buffer load, an unknown scalar-memory form, or a
load after the kernarg SGPR pair is overwritten rejects VMEM-only
certification. The embedded manifest's wavefront and code-object SHA-256 must
also match at runtime.

Redline selects the boundary per consumer:

- `vmem_only`: `CS_PARTIAL_FLUSH` plus GLV/merged-GL1 invalidation
  (`GCR=0x00300`).
- `scalar_or_unknown`, missing, or stale evidence: `CS_PARTIAL_FLUSH` plus
  GLK/GLV/merged-GL1 invalidation (`GCR=0x00380`).

The producer-completion edge is identical in both arms, coherent L2/MALL is
retained, and ownership changes still use the broader system acquire. The
optimization removes only an unrelated scalar-cache invalidation when the
consumer proves that mutable resource data cannot be read there.

All 23 kernels in both emitted wave32 and wave64 manifests certify VMEM-only
resource reads after their output RMWs were expressed with Radiowave's buffer
helpers. Serial tapes also reuse one immutable kernarg block per kernel because
their arguments do not change. That keeps the permitted scalar kernarg
prologue hot and lets the stateful PM4 encoder elide redundant
`COMPUTE_USER_DATA` writes. Independent rows keep distinct kernargs because
each operation has a different output offset.

Every sample completes the HIP-to-PM4 system ownership acquire before the
GPU-timestamped retained tape. The timed serial tape therefore contains the
real dispatches and their real dependency edges, while reset and ownership
transfer remain outside timing for every backend.

## Remaining Vulkan wins

The certified aggregate leaves 12 Vulkan wins: six serial, three independent,
and three aggressive. They are confined to large gather/interleave and VOPD
shapes. The safe-boundary crossover itself is gone; the remaining losses also
appear in aggressive or independent modes and are kernel/codegen targets:

| Mode | Remaining Vulkan-winning shapes |
|---|---|
| Serial | large gather, large interleave, and all four VOPD variants |
| Independent | large gather, mixed VOPD, and dequant VOPD |
| Aggressive | large interleave, mixed VOPD, and dequant VOPD |

## Artifacts

- [`aggregate.json`](aggregate.json) contains the 14-sample aggregate, every
  paired boundary comparison, selected cache policy, and all 266 aggregate
  rows.
- [`ab1-same`](../ab1-same/results.json) and
  [`ab2-same`](../ab2-same/results.json) are the generic-safe controls.
- [`ab1-vmem`](../ab1-vmem/results.json) and
  [`ab2-vmem`](../ab2-vmem/results.json) are the certified runs.
- [`aggregate_radiowave_vmem.jq`](../../../../scripts/aggregate_radiowave_vmem.jq)
  is the aggregation program.

The benchmark now defaults to `--redline-rmw radiowave-vmem`.
