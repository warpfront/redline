# Controlled wave-policy comparison

This comparison isolates HIP wave selection while holding the HIP source and
algorithms, problem shapes, Redline submission policy, backend order, warmups,
samples, correctness gates, and GPU constant. Every run completed 133/133
four-way correctness-valid rows.

The policies were fixed before measurement:

- `all_wave32`: every HIP-family kernel uses the wave32 code object.
- `targeted_wave64`: only `q4_selected_dual`, `q6_x8`, `dense_q8`, and
  `vopd_dependent` use wave64.
- `blanket_wave64`: every family with any Vulkan-over-Redline row in the
  preceding run uses wave64.

Two counterbalanced passes were run under Hipfire's GPU mutex:
`all32 -> targeted64 -> blanket64`, then
`blanket64 -> targeted64 -> all32`. Each policy/row result below is the median
of its 14 raw GPU samples across the two seven-sample replicates.

## Aggregate result

| Policy | RL 1st | Strict RL>VK | RL<VK | Median RL/VK |
|---|---:|---:|---:|---:|
| all wave32 | 82/133 | 84 | 49 | 0.8825x |
| targeted wave64 | **94/133** | **94** | **39** | **0.8485x** |
| blanket wave64 | 92/133 | 93 | 40 | 0.8533x |

The replicate-level first-place counts were 78/80 for all32, 92/94 for
targeted64, and 92/92 for blanket64. Targeted therefore wins the placement
comparison without relying on one favorable run order.

## By timing mode

| Policy | Mode | RL 1st | Strict RL>VK | Median RL/VK |
|---|---|---:|---:|---:|
| all32 | serial RMW | 17/45 | 19 | 1.0483x |
| targeted64 | serial RMW | **22/45** | **22** | **1.0011x** |
| blanket64 | serial RMW | 21/45 | 22 | 1.0311x |
| all32 | independent | 31/45 | 31 | 0.6712x |
| targeted64 | independent | **36/45** | **36** | 0.6328x |
| blanket64 | independent | 35/45 | 35 | **0.6038x** |
| all32 | aggressive N=1 | 34/43 | 34 | 0.8481x |
| targeted64 | aggressive N=1 | **36/43** | **36** | **0.8421x** |
| blanket64 | aggressive N=1 | **36/43** | **36** | 0.8533x |

Targeted64 is the best placement policy and is essentially at Vulkan parity on
the median serial row. Blanket64 has the best independent-throughput ratio,
which means some of its additional wave64 choices improve throughput without
creating more first places.

## What wave64 actually helps

Against all32, targeted wave64 changes 33 mode/shape rows. Their Redline
geometric-mean time improves by 27.5%. Median family improvements are:

| Targeted wave64 family | Serial | Independent | Aggressive N=1 |
|---|---:|---:|---:|
| Q4 selected x2 | 25.9% faster | 31.8% faster | 21.2% faster |
| Q6 x8 | 38.0% faster | 36.4% faster | 31.4% faster |
| dense Q8 x4 | 30.6% faster | 27.3% faster | 25.1% faster |
| dependent-FMA VOPD | 15.5% faster | 17.7% faster | 11.8% faster |

These kernels put productive work on all 64 lanes and advance through K by the
wave width. Wave64 directly halves each lane's inner-loop work for the quant
row-reuse kernels.

## Where blanket wave64 helps and hurts

Targeted and blanket differ on 73 mode/shape rows. Blanket is 1.15% faster by
geometric mean across those rows, but targeted earns two more first places.
The important effects are not uniform:

- Packed Q8/Q4/Q6 is decisively wave32: targeted is 38.5% faster in independent
  throughput, 3.9% faster serially, and 5.1% faster at N=1. The wave64 version
  loses the independent packed-Q6 first place.
- Non-dependent VOPD is wave64-favorable: blanket is about 5-12% faster across
  all modes, although those gains do not cross Vulkan.
- Four-row sampler throughput is wave64-favorable by 53-57%; one-row sampler
  throughput is about 3% better at wave32. Serial and N=1 sampler results are
  effectively unchanged.
- Coalesced-memory independent throughput is 18-20% faster at wave64, while
  its serial and N=1 paths favor wave32 by roughly 2-5%. Gather and interleave
  are mixed and generally smaller.
- Two-stage reduction is consistently 1.4-2.8% faster at wave32.
- Dispatch wave size is noise-level and produces no placement benefit.

Only two aggregate placements differ between targeted and blanket. Targeted
recovers independent packed Q6; a wave32 reduction row also crosses by less
than 0.4% despite using wave32 under both policies, which is ordinary residual
measurement noise. Blanket gains no unique first-place row.

## Verdict

Blanket wave64 was unnecessary for maximizing wins. The precommitted targeted
policy is the best tested placement policy: 94/133 firsts, 94 strict wins over
Vulkan, and the best overall median RL/Vulkan ratio. Blanket wave64 remains
useful for absolute independent throughput in VOPD, four-row sampler, and
coalesced-memory shapes, so a future shape-aware policy can retain those gains
while keeping packed dot and two-stage reduction on wave32.

The machine-readable aggregate is [`aggregate.json`](aggregate.json). Raw
replicates and their generated reports are in the six `rep1-*` and `rep2-*`
subdirectories. The aggregation is reproducible with:

```bash
jq -s -f scripts/aggregate_wave_policy.jq \
  results/gfx1201/2026-07-12-wave-policy-comparison/*/results.json
```
