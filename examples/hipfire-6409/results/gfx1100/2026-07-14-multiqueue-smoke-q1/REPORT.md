# Hipfire/Redline ROCm issue 6409 backend comparison

Correctness-gated result: **Redline wins 1/8 rows (12.50%)**.

## Placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 1 | 7 | 0 | 0 | 12.50 | 8 |
| vulkan | 7 | 1 | 0 | 0 | 87.50 | 8 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| independent_throughput | redline | 1 | 7 | 0 | 0 | 8 |
| independent_throughput | vulkan | 7 | 1 | 0 | 0 | 8 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `independent_throughput/dispatch-grid/sweep=count,count=1,grid=1;hip-wave=32` | 2 | vulkan (+2.93%) | 9.8400 | 9.5600 |
| `independent_throughput/dispatch-grid/sweep=count,count=200,grid=1;hip-wave=32` | 2 | vulkan (+51.56%) | 0.3410 | 0.2250 |
| `independent_throughput/dispatch-grid/sweep=count,count=941,grid=1;hip-wave=32` | 2 | vulkan (+207.83%) | 0.1989 | 0.0646 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=1;hip-wave=32` | 2 | vulkan (+89.22%) | 0.1745 | 0.0922 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=128;hip-wave=32` | 2 | vulkan (+840.94%) | 0.6448 | 0.0685 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=1024;hip-wave=32` | 2 | vulkan (+382.52%) | 1.1332 | 0.2349 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=8192;hip-wave=32` | 2 | vulkan (+205.83%) | 4.6817 | 1.5308 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 1 | 7 | 0 | 12.50 | 2.4752 | 8 |

## Harness verdict

This tuning smoke intentionally measures only `redline` versus `vulkan`. It uses the `hipengine_f2c` matrix, `radiowave_tuned` wave policy, and `radiowave-recipe-or-default` scheduler profile. Every ranked row passed both CPU oracles. Redline beats Vulkan in 1/8 rows; final promotion still requires a full four-backend certification run.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. The `hipengine_f2c` profile matches HipEngine's row coverage while intentionally retaining Hipfire/Radiowave's tuned wave, workgroup, and source-variant choices. This artifact records the `radiowave_tuned` wave policy and `radiowave-recipe-or-default` scheduler profile as controlled HIP compilation/launch factors. Every completion timestamp necessarily proves the measured work finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
