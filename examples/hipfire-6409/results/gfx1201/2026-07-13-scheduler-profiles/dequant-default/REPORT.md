# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 0/3 rows (0.00%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 0 | 3 | 0 | 0 | 0.00 | 3 |
| vulkan | 3 | 0 | 0 | 0 | 100.00 | 3 |
| hipgraph | 0 | 0 | 0 | 3 | 0.00 | 3 |
| hip | 0 | 0 | 3 | 0 | 0.00 | 3 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 0 | 1 | 0 | 0 | 1 |
| serial_latency | vulkan | 1 | 0 | 0 | 0 | 1 |
| serial_latency | hipgraph | 0 | 0 | 0 | 1 | 1 |
| serial_latency | hip | 0 | 0 | 1 | 0 | 1 |
| independent_throughput | redline | 0 | 1 | 0 | 0 | 1 |
| independent_throughput | vulkan | 1 | 0 | 0 | 0 | 1 |
| independent_throughput | hipgraph | 0 | 0 | 0 | 1 | 1 |
| independent_throughput | hip | 0 | 0 | 1 | 0 | 1 |
| single_kernel_aggressive | redline | 0 | 1 | 0 | 0 | 1 |
| single_kernel_aggressive | vulkan | 1 | 0 | 0 | 0 | 1 |
| single_kernel_aggressive | hipgraph | 0 | 0 | 0 | 1 | 1 |
| single_kernel_aggressive | hip | 0 | 0 | 1 | 0 | 1 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+21.27%) | 17.9731 | 14.8212 |
| `independent_throughput/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+11.89%) | 15.1756 | 13.5631 |
| `single_kernel_aggressive/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+4.88%) | 20.6400 | 19.6800 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 0 | 3 | 0 | 0.00 | 1.1189 | 3 |
| RL / hipgraph | 3 | 0 | 0 | 100.00 | 0.3054 | 3 |
| RL / hip | 3 | 0 | 0 | 100.00 | 0.3379 | 3 |

## Prior hipEngine-harness baseline

The old run is directional rather than row-for-row identical: it used a larger parameter sweep, only three contenders, and the previous kernels.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine | RL / vulkan | 186 | 26 | 87.74 | 0.4788 | 212 |
| hipEngine | RL / hip | 157 | 55 | 74.06 | 0.8342 | 212 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 3/3 rows and HipGraph in 3/3 while all three select identical per-row hipcc code objects. This run uses the `radiowave_tuned` wave policy; the aggressive single-kernel mode removes dependency fences from Redline's timed tape. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 0/3 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. This artifact records the `radiowave_tuned` wave policy as the controlled HIP launch-policy factor. The aggressive Redline timed IB has no dependency fences, but the completion timestamp still necessarily proves the dispatch finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
