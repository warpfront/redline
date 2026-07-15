# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 120/133 rows (90.23%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 120 | 13 | 0 | 0 | 90.23 | 133 |
| vulkan | 13 | 98 | 6 | 16 | 9.77 | 133 |
| hipgraph | 0 | 5 | 40 | 88 | 0.00 | 133 |
| hip | 0 | 17 | 87 | 29 | 0.00 | 133 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 39 | 6 | 0 | 0 | 45 |
| serial_latency | vulkan | 6 | 28 | 0 | 11 | 45 |
| serial_latency | hipgraph | 0 | 5 | 32 | 8 | 45 |
| serial_latency | hip | 0 | 6 | 13 | 26 | 45 |
| independent_throughput | redline | 42 | 3 | 0 | 0 | 45 |
| independent_throughput | vulkan | 3 | 42 | 0 | 0 | 45 |
| independent_throughput | hipgraph | 0 | 0 | 3 | 42 | 45 |
| independent_throughput | hip | 0 | 0 | 42 | 3 | 45 |
| single_kernel_aggressive | redline | 39 | 4 | 0 | 0 | 43 |
| single_kernel_aggressive | vulkan | 4 | 28 | 6 | 5 | 43 |
| single_kernel_aggressive | hipgraph | 0 | 0 | 5 | 38 | 43 |
| single_kernel_aggressive | hip | 0 | 11 | 32 | 0 | 43 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+14.29%) | 22.1388 | 19.3700 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=64` | 2 | vulkan (+8.08%) | 10.8825 | 10.0687 |
| `serial_latency/vopd/variant=independent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+6.26%) | 9.2538 | 8.7088 |
| `serial_latency/vopd/variant=dependent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+3.14%) | 9.4537 | 9.1662 |
| `serial_latency/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+35.97%) | 22.2437 | 16.3588 |
| `serial_latency/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+36.48%) | 19.5237 | 14.3050 |
| `independent_throughput/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+21.95%) | 9.4313 | 7.7337 |
| `independent_throughput/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+20.08%) | 8.9688 | 7.4688 |
| `independent_throughput/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+16.99%) | 7.8075 | 6.6738 |
| `single_kernel_aggressive/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+0.60%) | 13.3200 | 13.2400 |
| `single_kernel_aggressive/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=64` | 2 | vulkan (+2.27%) | 9.0000 | 8.8000 |
| `single_kernel_aggressive/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+16.49%) | 13.0000 | 11.1600 |
| `single_kernel_aggressive/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+20.38%) | 12.5200 | 10.4000 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 120 | 13 | 0 | 90.23 | 0.6957 | 133 |
| RL / hipgraph | 133 | 0 | 0 | 100.00 | 0.2260 | 133 |
| RL / hip | 133 | 0 | 0 | 100.00 | 0.3147 | 133 |

## Prior hipEngine-harness baseline

The old run is directional rather than row-for-row identical: it used a larger parameter sweep, only three contenders, and the previous kernels.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine | RL / vulkan | 186 | 26 | 87.74 | 0.4788 | 212 |
| hipEngine | RL / hip | 157 | 55 | 74.06 | 0.8342 | 212 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 133/133 rows and HipGraph in 133/133 while all three select identical per-row hipcc code objects. This run uses the `radiowave_tuned` wave policy; the aggressive single-kernel mode removes dependency fences from Redline's timed tape. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 120/133 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. This artifact records the `radiowave_tuned` wave policy as the controlled HIP launch-policy factor. The aggressive Redline timed IB has no dependency fences, but the completion timestamp still necessarily proves the dispatch finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
