# Staged gather pipeline experiment

This experiment tested whether explicitly overlapping the next gather-index
loads with the current value loads closes the remaining large-gather gap.
Both candidates used ordinary HIP compiled by Radiowave and retained the
`vmem_only`, spill-free Redline certification. Neither candidate is promoted.

## Emitted ISA

| Kernel | VGPR | SGPR | Buffer loads | Waits | Static instructions |
|---|---:|---:|---:|---:|---:|
| Accepted gather | 8 | 21 | 11 | 21 | 150 |
| Four-index pipeline | 14 | 21 | 19 | 28 | 190 |
| Two-index pipeline | 10 | 19 | 11 | 20 | 148 |

The four-index version duplicated requests and increased both register
pressure and waits. The two-index version produced the intended overlap with
almost the same static footprint as the accepted kernel, so it was advanced
to a frozen-binary A/B/B/A comparison (42 samples per candidate, backend, and
row).

## Frozen comparison

| Large gather mode | Pipeline-2 Redline | Accepted Redline | Change | Vulkan | Pipeline-2 gap |
|---|---:|---:|---:|---:|---:|
| Safe serial | 21.9588 us | 22.0119 us | -0.24% | 19.3238 us | +13.64% |
| Independent | 17.2531 us | 17.3819 us | -0.74% | 12.9156 us | +33.58% |
| Aggressive | 14.7400 us | 14.9400 us | -1.34% | 16.2800 us | -9.46% |

All backend outputs passed their CPU oracle. The changes are too small to
separate from run-order and clock variation, no Vulkan loss flips, and the
large serial/independent gaps remain. The staged gather candidates were
therefore removed from the selected source. Raw probe and frozen-run artifacts
remain in this directory.

## Conclusion

LLVM can preserve a source-level gather window, but the remaining gap is not
caused by the absence of that overlap alone. A useful next gather experiment
needs to change request count or address/data locality rather than add another
software staging layer around the same two dependent VMEM requests.
