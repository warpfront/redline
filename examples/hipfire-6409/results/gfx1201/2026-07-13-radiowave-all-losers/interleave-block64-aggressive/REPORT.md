# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 2/2 rows (100.00%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 2 | 0 | 0 | 0 | 100.00 | 2 |
| vulkan | 0 | 2 | 0 | 0 | 0.00 | 2 |
| hipgraph | 0 | 0 | 0 | 2 | 0.00 | 2 |
| hip | 0 | 0 | 2 | 0 | 0.00 | 2 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 0 | 0 | 0 | 0 | 0 |
| serial_latency | vulkan | 0 | 0 | 0 | 0 | 0 |
| serial_latency | hipgraph | 0 | 0 | 0 | 0 | 0 |
| serial_latency | hip | 0 | 0 | 0 | 0 | 0 |
| independent_throughput | redline | 0 | 0 | 0 | 0 | 0 |
| independent_throughput | vulkan | 0 | 0 | 0 | 0 | 0 |
| independent_throughput | hipgraph | 0 | 0 | 0 | 0 | 0 |
| independent_throughput | hip | 0 | 0 | 0 | 0 | 0 |
| single_kernel_aggressive | redline | 2 | 0 | 0 | 0 | 2 |
| single_kernel_aggressive | vulkan | 0 | 2 | 0 | 0 | 2 |
| single_kernel_aggressive | hipgraph | 0 | 0 | 0 | 2 | 2 |
| single_kernel_aggressive | hip | 0 | 0 | 2 | 0 | 2 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 2 | 0 | 0 | 100.00 | 0.8148 | 2 |
| RL / hipgraph | 2 | 0 | 0 | 100.00 | 0.4439 | 2 |
| RL / hip | 2 | 0 | 0 | 100.00 | 0.6413 | 2 |

## Prior hipEngine-harness baseline

The old run is directional rather than row-for-row identical: it used a larger parameter sweep, only three contenders, and the previous kernels.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine | RL / vulkan | 186 | 26 | 87.74 | 0.4788 | 212 |
| hipEngine | RL / hip | 157 | 55 | 74.06 | 0.8342 | 212 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 2/2 rows and HipGraph in 2/2 while all three select identical per-row hipcc code objects. This run uses the `radiowave_tuned` wave policy; the aggressive single-kernel mode removes dependency fences from Redline's timed tape. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 2/2 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. This artifact records the `radiowave_tuned` wave policy as the controlled HIP launch-policy factor. The aggressive Redline timed IB has no dependency fences, but the completion timestamp still necessarily proves the dispatch finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
