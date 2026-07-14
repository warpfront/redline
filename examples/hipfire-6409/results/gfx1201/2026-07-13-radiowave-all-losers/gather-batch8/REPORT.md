# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 3/6 rows (50.00%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 3 | 3 | 0 | 0 | 50.00 | 6 |
| vulkan | 3 | 1 | 1 | 1 | 50.00 | 6 |
| hipgraph | 0 | 1 | 1 | 4 | 0.00 | 6 |
| hip | 0 | 1 | 4 | 1 | 0.00 | 6 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 1 | 1 | 0 | 0 | 2 |
| serial_latency | vulkan | 1 | 0 | 0 | 1 | 2 |
| serial_latency | hipgraph | 0 | 1 | 1 | 0 | 2 |
| serial_latency | hip | 0 | 0 | 1 | 1 | 2 |
| independent_throughput | redline | 0 | 2 | 0 | 0 | 2 |
| independent_throughput | vulkan | 2 | 0 | 0 | 0 | 2 |
| independent_throughput | hipgraph | 0 | 0 | 0 | 2 | 2 |
| independent_throughput | hip | 0 | 0 | 2 | 0 | 2 |
| single_kernel_aggressive | redline | 2 | 0 | 0 | 0 | 2 |
| single_kernel_aggressive | vulkan | 0 | 1 | 1 | 0 | 2 |
| single_kernel_aggressive | hipgraph | 0 | 0 | 0 | 2 | 2 |
| single_kernel_aggressive | hip | 0 | 1 | 1 | 0 | 2 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+20.73%) | 23.2175 | 19.2312 |
| `independent_throughput/memory-waitcnt/variant=gather,n=4096,body=16;hip-wave=32` | 2 | vulkan (+70.57%) | 2.2644 | 1.3275 |
| `independent_throughput/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+33.43%) | 20.0050 | 14.9925 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 3 | 3 | 0 | 50.00 | 1.0467 | 6 |
| RL / hipgraph | 6 | 0 | 0 | 100.00 | 0.3909 | 6 |
| RL / hip | 6 | 0 | 0 | 100.00 | 0.4970 | 6 |

## Prior hipEngine-harness baseline

The old run is directional rather than row-for-row identical: it used a larger parameter sweep, only three contenders, and the previous kernels.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine | RL / vulkan | 186 | 26 | 87.74 | 0.4788 | 212 |
| hipEngine | RL / hip | 157 | 55 | 74.06 | 0.8342 | 212 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 6/6 rows and HipGraph in 6/6 while all three select identical per-row hipcc code objects. This run uses the `radiowave_tuned` wave policy; the aggressive single-kernel mode removes dependency fences from Redline's timed tape. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 3/6 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. This artifact records the `radiowave_tuned` wave policy as the controlled HIP launch-policy factor. The aggressive Redline timed IB has no dependency fences, but the completion timestamp still necessarily proves the dispatch finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
