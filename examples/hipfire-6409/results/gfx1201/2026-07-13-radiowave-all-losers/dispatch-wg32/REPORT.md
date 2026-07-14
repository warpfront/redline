# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 16/21 rows (76.19%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 16 | 5 | 0 | 0 | 76.19 | 21 |
| vulkan | 5 | 16 | 0 | 0 | 23.81 | 21 |
| hipgraph | 0 | 0 | 5 | 16 | 0.00 | 21 |
| hip | 0 | 0 | 16 | 5 | 0.00 | 21 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 2 | 5 | 0 | 0 | 7 |
| serial_latency | vulkan | 5 | 2 | 0 | 0 | 7 |
| serial_latency | hipgraph | 0 | 0 | 5 | 2 | 7 |
| serial_latency | hip | 0 | 0 | 2 | 5 | 7 |
| independent_throughput | redline | 7 | 0 | 0 | 0 | 7 |
| independent_throughput | vulkan | 0 | 7 | 0 | 0 | 7 |
| independent_throughput | hipgraph | 0 | 0 | 0 | 7 | 7 |
| independent_throughput | hip | 0 | 0 | 7 | 0 | 7 |
| single_kernel_aggressive | redline | 7 | 0 | 0 | 0 | 7 |
| single_kernel_aggressive | vulkan | 0 | 7 | 0 | 0 | 7 |
| single_kernel_aggressive | hipgraph | 0 | 0 | 0 | 7 | 7 |
| single_kernel_aggressive | hip | 0 | 0 | 7 | 0 | 7 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/dispatch-grid/count=50,grid=1;hip-wave=32` | 2 | vulkan (+34.60%) | 1.7240 | 1.2808 |
| `serial_latency/dispatch-grid/count=200,grid=1;hip-wave=32` | 2 | vulkan (+46.31%) | 1.6080 | 1.0990 |
| `serial_latency/dispatch-grid/count=941,grid=1;hip-wave=32` | 2 | vulkan (+56.43%) | 1.5899 | 1.0164 |
| `serial_latency/dispatch-grid/count=64,grid=128;hip-wave=32` | 2 | vulkan (+152.84%) | 2.9787 | 1.1781 |
| `serial_latency/dispatch-grid/count=64,grid=1024;hip-wave=32` | 2 | vulkan (+37.57%) | 3.0575 | 2.2225 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 16 | 5 | 0 | 76.19 | 0.5928 | 21 |
| RL / hipgraph | 21 | 0 | 0 | 100.00 | 0.1921 | 21 |
| RL / hip | 21 | 0 | 0 | 100.00 | 0.3297 | 21 |

## Prior hipEngine-harness baseline

The old run is directional rather than row-for-row identical: it used a larger parameter sweep, only three contenders, and the previous kernels.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine | RL / vulkan | 186 | 26 | 87.74 | 0.4788 | 212 |
| hipEngine | RL / hip | 157 | 55 | 74.06 | 0.8342 | 212 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 21/21 rows and HipGraph in 21/21 while all three select identical per-row hipcc code objects. This run uses the `radiowave_tuned` wave policy; the aggressive single-kernel mode removes dependency fences from Redline's timed tape. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 16/21 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. This artifact records the `radiowave_tuned` wave policy as the controlled HIP launch-policy factor. The aggressive Redline timed IB has no dependency fences, but the completion timestamp still necessarily proves the dispatch finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
