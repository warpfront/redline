# Certified dequant chunk16 result

Radiowave now selects `vopd_dequant_chunk16` for the wave64 dequant-like row.
The kernel groups 16 exact integer dequant contributions before updating the
two float accumulators, giving LLVM a larger scheduling window without spills
or a non-VMEM mutable read.

Two complete seven-sample replicates passed every CPU oracle. Redline retains
**121/133** strict wins over Vulkan and four-way first places:

| Mode | Rows | Redline first | Strict Redline>Vulkan | Median Redline/Vulkan |
|---|---:|---:|---:|---:|
| Safe serial | 45 | 39 | 39 | 0.7365x |
| Independent | 45 | 42 | 42 | 0.5258x |
| Aggressive | 43 | 40 | 40 | 0.7792x |
| **Overall** | **133** | **121** | **121** | **0.6923x** |

| Dequant mode | Previous Redline | Chunk16 Redline | Improvement | Vulkan | Remaining gap |
|---|---:|---:|---:|---:|---:|
| Safe serial | 19.9406 us | 17.8125 us | **10.67%** | 14.9244 us | +19.35% |
| Independent | 7.8075 us | 7.0125 us | **10.18%** | 6.6738 us | +5.08% |
| Aggressive | 12.5400 us | 11.2800 us | **10.05%** | 10.4400 us | +8.05% |

The original 941-dispatch floor remains a Redline win at 0.7608 us/dispatch
versus Vulkan at 1.0329 us. The machine-readable result is
[`aggregate.json`](aggregate.json); raw complete runs are in [`../rep1`](../rep1/)
and [`../rep2`](../rep2/).
