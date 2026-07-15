# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 121/133 rows (90.98%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 121 | 12 | 0 | 0 | 90.98 | 133 |
| vulkan | 12 | 99 | 6 | 16 | 9.02 | 133 |
| hipgraph | 0 | 10 | 38 | 85 | 0.00 | 133 |
| hip | 0 | 12 | 89 | 32 | 0.00 | 133 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 39 | 6 | 0 | 0 | 45 |
| serial_latency | vulkan | 6 | 27 | 1 | 11 | 45 |
| serial_latency | hipgraph | 0 | 10 | 30 | 5 | 45 |
| serial_latency | hip | 0 | 2 | 14 | 29 | 45 |
| independent_throughput | redline | 42 | 3 | 0 | 0 | 45 |
| independent_throughput | vulkan | 3 | 42 | 0 | 0 | 45 |
| independent_throughput | hipgraph | 0 | 0 | 3 | 42 | 45 |
| independent_throughput | hip | 0 | 0 | 42 | 3 | 45 |
| single_kernel_aggressive | redline | 40 | 3 | 0 | 0 | 43 |
| single_kernel_aggressive | vulkan | 3 | 30 | 5 | 5 | 43 |
| single_kernel_aggressive | hipgraph | 0 | 0 | 5 | 38 | 43 |
| single_kernel_aggressive | hip | 0 | 10 | 33 | 0 | 43 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+14.51%) | 22.3950 | 19.5575 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=64` | 2 | vulkan (+14.97%) | 11.7100 | 10.1850 |
| `serial_latency/vopd/variant=independent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+4.70%) | 9.5437 | 9.1150 |
| `serial_latency/vopd/variant=dependent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+3.21%) | 9.5625 | 9.2650 |
| `serial_latency/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+36.02%) | 22.4975 | 16.5400 |
| `serial_latency/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+36.35%) | 20.3588 | 14.9313 |
| `independent_throughput/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+21.93%) | 9.3975 | 7.7075 |
| `independent_throughput/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+19.96%) | 8.9462 | 7.4575 |
| `independent_throughput/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+17.01%) | 7.7812 | 6.6500 |
| `single_kernel_aggressive/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=64` | 2 | vulkan (+0.90%) | 9.0000 | 8.9200 |
| `single_kernel_aggressive/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+18.21%) | 13.2400 | 11.2000 |
| `single_kernel_aggressive/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+19.92%) | 12.5200 | 10.4400 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 121 | 12 | 0 | 90.98 | 0.6831 | 133 |
| RL / hipgraph | 133 | 0 | 0 | 100.00 | 0.2275 | 133 |
| RL / hip | 133 | 0 | 0 | 100.00 | 0.3145 | 133 |

## Prior hipEngine-harness baseline

The old run is directional rather than row-for-row identical: it used a larger parameter sweep, only three contenders, and the previous kernels.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine | RL / vulkan | 186 | 26 | 87.74 | 0.4788 | 212 |
| hipEngine | RL / hip | 157 | 55 | 74.06 | 0.8342 | 212 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 133/133 rows and HipGraph in 133/133 while all three select identical per-row hipcc code objects. This run uses the `radiowave_tuned` wave policy; the aggressive single-kernel mode removes dependency fences from Redline's timed tape. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 121/133 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. This artifact records the `radiowave_tuned` wave policy as the controlled HIP launch-policy factor. The aggressive Redline timed IB has no dependency fences, but the completion timestamp still necessarily proves the dispatch finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
