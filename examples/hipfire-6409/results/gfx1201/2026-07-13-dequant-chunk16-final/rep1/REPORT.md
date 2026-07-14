# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 120/133 rows (90.23%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 120 | 13 | 0 | 0 | 90.23 | 133 |
| vulkan | 13 | 98 | 7 | 15 | 9.77 | 133 |
| hipgraph | 0 | 7 | 41 | 85 | 0.00 | 133 |
| hip | 0 | 15 | 85 | 33 | 0.00 | 133 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 39 | 6 | 0 | 0 | 45 |
| serial_latency | vulkan | 6 | 28 | 0 | 11 | 45 |
| serial_latency | hipgraph | 0 | 7 | 32 | 6 | 45 |
| serial_latency | hip | 0 | 4 | 13 | 28 | 45 |
| independent_throughput | redline | 41 | 4 | 0 | 0 | 45 |
| independent_throughput | vulkan | 4 | 41 | 0 | 0 | 45 |
| independent_throughput | hipgraph | 0 | 0 | 5 | 40 | 45 |
| independent_throughput | hip | 0 | 0 | 40 | 5 | 45 |
| single_kernel_aggressive | redline | 40 | 3 | 0 | 0 | 43 |
| single_kernel_aggressive | vulkan | 3 | 29 | 7 | 4 | 43 |
| single_kernel_aggressive | hipgraph | 0 | 0 | 4 | 39 | 43 |
| single_kernel_aggressive | hip | 0 | 11 | 32 | 0 | 43 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+14.94%) | 22.5712 | 19.6375 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=64` | 2 | vulkan (+13.89%) | 11.6263 | 10.2088 |
| `serial_latency/vopd/variant=independent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+4.57%) | 9.5550 | 9.1375 |
| `serial_latency/vopd/variant=dependent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+3.43%) | 9.6025 | 9.2837 |
| `serial_latency/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+35.82%) | 22.5500 | 16.6025 |
| `serial_latency/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+19.35%) | 17.8875 | 14.9875 |
| `independent_throughput/memory-waitcnt/variant=gather,n=4096,body=16;hip-wave=32` | 2 | vulkan (+4.18%) | 1.0444 | 1.0025 |
| `independent_throughput/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+21.54%) | 9.4213 | 7.7512 |
| `independent_throughput/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+19.77%) | 8.9525 | 7.4750 |
| `independent_throughput/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+5.15%) | 7.0125 | 6.6688 |
| `single_kernel_aggressive/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=64` | 2 | vulkan (+0.45%) | 8.9600 | 8.9200 |
| `single_kernel_aggressive/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+21.58%) | 13.5200 | 11.1200 |
| `single_kernel_aggressive/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+3.83%) | 10.8400 | 10.4400 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 120 | 13 | 0 | 90.23 | 0.6988 | 133 |
| RL / hipgraph | 133 | 0 | 0 | 100.00 | 0.2311 | 133 |
| RL / hip | 133 | 0 | 0 | 100.00 | 0.2978 | 133 |

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
