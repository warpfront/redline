# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 9/12 rows (75.00%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 9 | 3 | 0 | 0 | 75.00 | 12 |
| vulkan | 3 | 7 | 1 | 1 | 25.00 | 12 |
| hipgraph | 0 | 1 | 3 | 8 | 0.00 | 12 |
| hip | 0 | 1 | 8 | 3 | 0.00 | 12 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 2 | 2 | 0 | 0 | 4 |
| serial_latency | vulkan | 2 | 1 | 0 | 1 | 4 |
| serial_latency | hipgraph | 0 | 1 | 3 | 0 | 4 |
| serial_latency | hip | 0 | 0 | 1 | 3 | 4 |
| independent_throughput | redline | 3 | 1 | 0 | 0 | 4 |
| independent_throughput | vulkan | 1 | 3 | 0 | 0 | 4 |
| independent_throughput | hipgraph | 0 | 0 | 0 | 4 | 4 |
| independent_throughput | hip | 0 | 0 | 4 | 0 | 4 |
| single_kernel_aggressive | redline | 4 | 0 | 0 | 0 | 4 |
| single_kernel_aggressive | vulkan | 0 | 3 | 1 | 0 | 4 |
| single_kernel_aggressive | hipgraph | 0 | 0 | 0 | 4 | 4 |
| single_kernel_aggressive | hip | 0 | 1 | 3 | 0 | 4 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/packed-dot/variant=q8-signed,n=4096,body=16;hip-wave=32` | 2 | vulkan (+1.90%) | 7.7594 | 7.6144 |
| `serial_latency/packed-dot/variant=q4-unsigned,n=4096,body=16;hip-wave=32` | 2 | vulkan (+1.31%) | 7.7363 | 7.6363 |
| `independent_throughput/packed-dot/variant=q8-signed,n=4096,body=16;hip-wave=32` | 2 | vulkan (+4.08%) | 1.9438 | 1.8675 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 9 | 3 | 0 | 75.00 | 0.8113 | 12 |
| RL / hipgraph | 12 | 0 | 0 | 100.00 | 0.4090 | 12 |
| RL / hip | 12 | 0 | 0 | 100.00 | 0.6284 | 12 |

## Prior hipEngine-harness baseline

The old run is directional rather than row-for-row identical: it used a larger parameter sweep, only three contenders, and the previous kernels.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine | RL / vulkan | 186 | 26 | 87.74 | 0.4788 | 212 |
| hipEngine | RL / hip | 157 | 55 | 74.06 | 0.8342 | 212 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 12/12 rows and HipGraph in 12/12 while all three select identical per-row hipcc code objects. This run uses the `targeted_wave64` wave policy; the aggressive single-kernel mode removes dependency fences from Redline's timed tape. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 9/12 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. This artifact records the `targeted_wave64` wave policy as the controlled HIP launch-policy factor. The aggressive Redline timed IB has no dependency fences, but the completion timestamp still necessarily proves the dispatch finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
