# Minimal same-agent RMW boundary

Redline now uses the same minimal dependency semantics as RADV for a coherent
same-agent compute buffer: retire prior compute, then invalidate only scalar and
vector shader read caches. It no longer writes back and invalidates coherent
L2/MALL at every retained-PM4 dependency edge.

## Final 133-row result

This aggregate is the median of 14 raw GPU samples from two complete runs. Each
run used three warmups and seven measured samples; all 133 four-backend CPU
oracles passed.

| Backend | 1st | 2nd | 3rd | 4th | Win rate |
|---|---:|---:|---:|---:|---:|
| Redline | **114** | 19 | 0 | 0 | **85.71%** |
| Vulkan/RADV | 19 | 91 | 6 | 17 | 14.29% |
| HipGraph | 0 | 10 | 38 | 85 | 0.00% |
| direct HIP | 0 | 13 | 89 | 31 | 0.00% |

Redline also beats Vulkan strictly in **114/133 rows (85.71%)**.

| Mode | RL first | Strict RL>Vulkan | Losses | Median RL/Vulkan |
|---|---:|---:|---:|---:|
| Serial RMW latency | **32/45** | **32/45** | 13 | 0.8530x |
| Independent throughput | **42/45** | **42/45** | 3 | 0.5155x |
| Single-kernel aggressive | **40/43** | **40/43** | 3 | 0.7730x |
| **All modes** | **114/133** | **114/133** | **19** | **0.7327x** |

The previous full Radiowave aggregate was 110 first places and 112 strict
Vulkan wins. The new full run is therefore +4 firsts and +2 pairwise wins, but
the counterbalanced boundary-only comparison below is the correct attribution
of the fence change because it controls short-run noise.

## Controlled boundary A/B

The A/B/B/A comparison holds HIP code objects, Vulkan shaders, row order,
submission policy, warmups, and sample count constant. Only the Redline cache
action changes.

| Redline RMW boundary | RL first | Strict RL>Vulkan | Median RL/Vulkan |
|---|---:|---:|---:|
| Historical global L2 WB+INV | 29/45 | 31/45 | 0.9350x |
| **Same-agent shader caches** | **32/45** | **32/45** | **0.8519x** |

The same-agent boundary is faster on 43/45 serial rows, equal on one N=1 row,
and 0.33% slower on one long dense-Q8 row. Its median Redline time is **4.27%
lower**. The small interleave row flips from 10.88% behind Vulkan to 7.36%
ahead.

Cache-sensitive families gain much more because their working set stays in
coherent L2 between dependent dispatches:

| Family | Median same-agent/global time |
|---|---:|
| sampler | 0.7661x |
| Q6 x8 selected | 0.8156x |
| packed dot | 0.8319x |
| two-stage reduction | 0.8834x |
| Q4 selected dual | 0.9267x |
| memory/waitcnt | 0.9573x |

## Safety boundary

Both halves remain necessary for a true non-atomic RMW chain. The focused
941-dispatch control measured:

| Boundary | µs/dispatch | Correct |
|---|---:|:---:|
| global L2 WB+INV | 1.6884 | yes |
| same-agent scalar/vector INV | **1.5796** | yes |
| compute wait only | 0.5191 | **no** |
| cache acquire only | 0.3009 | **no** |
| no boundary | 0.1084 | **no** |

The promoted core helper emits `CS_PARTIAL_FLUSH` (`EVENT_WRITE 0x407`) followed
by `ACQUIRE_MEM` with `GCR_CNTL=0x180` (scalar and vector shader cache
invalidation). This matches Mesa RADV 25.2.8's compute-write to
compute-read/write coherent-buffer path. External ownership changes and
non-coherent resources still require the broader system/global policy.

The fence-free aggressive number is therefore not a reachable target for a
strict RMW chain: it is fast precisely because producer and consumer waves may
overlap, which makes this kernel race. The achievable target is to match
Vulkan's minimal correct boundary and then win through retained submission and
better HIP codegen. Redline now matches those boundary semantics.

## Remaining Vulkan wins

The 13 serial losses now split into five tiny-dispatch chains, four VOPD rows,
large coalesced/gather/interleave, and small coalesced (only 1.79% behind).
Since Redline and Vulkan now execute the same completion and shader-cache
semantics on these coherent buffers, the residual is kernel ISA, dispatch
geometry/initiator behavior, or unavoidable serialized kernel duration—not an
extra safety scope that can simply be deleted.

## Artifacts

- [Final aggregate](aggregate.json)
- [Final replicate 1](rep1/results.json)
- [Final replicate 2](rep2/results.json)
- [Counterbalanced boundary aggregate](../comparison/aggregate.json)
- [A/B global control](../ab1-radv/results.json)
- [A/B same-agent arm](../ab1-same/results.json)
- [B/A same-agent arm](../ab2-same/results.json)
- [B/A global control](../ab2-radv/results.json)
