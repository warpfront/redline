# Wave64 mixed/dequant scheduling experiment

This experiment attacked the remaining Vulkan VOPD-family wins with ordinary
wave64 HIP source transformations. Rejected variants were removed. The
16-iteration dequant chunk is promoted because its gain over the accepted HIP
kernel survives a frozen-binary A/B/B/A comparison and the complete 133-row
correctness gate.

## Candidate sweep

- Pairwise affine composition for `mixed-int-float` reduced the static body,
  but raised VGPR use and slowed the measured kernel. It was rejected.
- Wave32 VOPD formation had already been tested separately: LLVM emitted
  `v_dual_*`, but the wave32 kernels lost to the accepted wave64 placement.
- Static dequant chunks exposed enough hash-chain ILP to reduce runtime. A
  16-step chunk was best.
- Two 8-step static phases retained the 37-VGPR footprint without improving
  the schedule. A dynamic 8-step chunk reduced the footprint to 23 VGPRs but
  paid for variable bit extraction. A branch-selected phase used 27 VGPRs and
  176 waits and also lost.
- Compact and whole-loop integer-accumulation forms reached 8-9 VGPRs, but
  exposed recurrence and loop latency. Carrying the integer sum across static
  16-step blocks used 34 VGPRs but was still slower than resetting per block.

The promoted `vopd_dequant_chunk16` emits wave64 with 37 VGPRs, 9 SGPRs, no
private segment or spills, one buffer load, one buffer store, and a
`vmem_only` mutable-read footprint. It sums 16 exact integer dequant terms
before performing two float conversions/FMAs, while interleaving the two hash
chains in the enlarged LLVM scheduling window.

## Frozen A/B/B/A result

The candidate and accepted selector were frozen into separate executables.
Each number below is the median of 42 raw Redline samples from two 21-sample
runs per executable.

| Mode | Chunk16 | Accepted | Redline improvement | Same-run Vulkan | Chunk16 gap |
|---|---:|---:|---:|---:|---:|
| Safe serial | 17.5631 us | 20.5213 us | **14.41%** | 14.5188 us | +20.97% |
| Independent | 7.3106 us | 8.1331 us | **10.11%** | 6.4456 us | +13.42% |
| Aggressive | 11.5600 us | 12.7200 us | **9.12%** | 10.9400 us | +5.67% |

The kernel-side improvement is real, but it does not yet flip a Vulkan row.

## Complete-matrix certification

Two complete seven-sample replicates pass all four backend CPU oracles for all
133 rows (266 row-runs and 1,064 validated backend outputs). The result remains
**121/133 strict Redline wins over Vulkan (90.98%)**:

| Mode | Redline wins | Median Redline/Vulkan |
|---|---:|---:|
| Safe serial | 39/45 | 0.7365x |
| Independent | 42/45 | 0.5258x |
| Aggressive | 40/43 | 0.7792x |
| **Overall** | **121/133** | **0.6923x** |

Against the previous certified aggregate, the selected dequant row improves
10.67% in safe serial replay (19.9406 to 17.8125 us), 10.18% in independent
throughput (7.8075 to 7.0125 us), and 10.05% in aggressive single-dispatch
timing (12.5400 to 11.2800 us). The remaining same-run Vulkan gaps are 19.35%,
5.08%, and 8.05%, respectively.

See the [frozen comparison](chunk16-frozen-ab/comparison.json), the
[complete aggregate](../2026-07-13-dequant-chunk16-final/comparison/aggregate.json),
and the [final certification report](../2026-07-13-dequant-chunk16-final/comparison/REPORT.md).
