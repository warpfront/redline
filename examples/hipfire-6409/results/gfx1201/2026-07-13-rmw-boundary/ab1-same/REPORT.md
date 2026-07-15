# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 32/45 rows (71.11%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 32 | 13 | 0 | 0 | 71.11 | 45 |
| vulkan | 13 | 20 | 1 | 11 | 28.89 | 45 |
| hipgraph | 0 | 8 | 32 | 5 | 0.00 | 45 |
| hip | 0 | 4 | 12 | 29 | 0.00 | 45 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 32 | 13 | 0 | 0 | 45 |
| serial_latency | vulkan | 13 | 20 | 1 | 11 | 45 |
| serial_latency | hipgraph | 0 | 8 | 32 | 5 | 45 |
| serial_latency | hip | 0 | 4 | 12 | 29 | 45 |
| independent_throughput | redline | 0 | 0 | 0 | 0 | 0 |
| independent_throughput | vulkan | 0 | 0 | 0 | 0 | 0 |
| independent_throughput | hipgraph | 0 | 0 | 0 | 0 | 0 |
| independent_throughput | hip | 0 | 0 | 0 | 0 | 0 |
| single_kernel_aggressive | redline | 0 | 0 | 0 | 0 | 0 |
| single_kernel_aggressive | vulkan | 0 | 0 | 0 | 0 | 0 |
| single_kernel_aggressive | hipgraph | 0 | 0 | 0 | 0 | 0 |
| single_kernel_aggressive | hip | 0 | 0 | 0 | 0 | 0 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/dispatch-grid/count=50,grid=1;hip-wave=32` | 2 | vulkan (+26.03%) | 1.6384 | 1.3000 |
| `serial_latency/dispatch-grid/count=200,grid=1;hip-wave=32` | 2 | vulkan (+39.20%) | 1.5362 | 1.1036 |
| `serial_latency/dispatch-grid/count=941,grid=1;hip-wave=32` | 2 | vulkan (+48.43%) | 1.5392 | 1.0370 |
| `serial_latency/dispatch-grid/count=64,grid=128;hip-wave=32` | 2 | vulkan (+151.41%) | 2.9619 | 1.1781 |
| `serial_latency/dispatch-grid/count=64,grid=1024;hip-wave=32` | 2 | vulkan (+34.90%) | 3.0394 | 2.2531 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=4096,body=16;hip-wave=32` | 2 | vulkan (+1.39%) | 6.2956 | 6.2094 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+21.27%) | 8.4388 | 6.9588 |
| `serial_latency/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+20.41%) | 23.5550 | 19.5625 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=64` | 2 | vulkan (+19.77%) | 12.2700 | 10.2450 |
| `serial_latency/vopd/variant=independent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+19.31%) | 10.9062 | 9.1412 |
| `serial_latency/vopd/variant=dependent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+17.03%) | 10.8063 | 9.2338 |
| `serial_latency/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+44.04%) | 23.8463 | 16.5550 |
| `serial_latency/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+44.74%) | 21.6163 | 14.9350 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 32 | 13 | 0 | 71.11 | 0.8513 | 45 |
| RL / hipgraph | 45 | 0 | 0 | 100.00 | 0.5472 | 45 |
| RL / hip | 45 | 0 | 0 | 100.00 | 0.5465 | 45 |

## Prior hipEngine-harness baseline

The old run is directional rather than row-for-row identical: it used a larger parameter sweep, only three contenders, and the previous kernels.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine | RL / vulkan | 186 | 26 | 87.74 | 0.4788 | 212 |
| hipEngine | RL / hip | 157 | 55 | 74.06 | 0.8342 | 212 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 45/45 rows and HipGraph in 45/45 while all three select identical per-row hipcc code objects. This run uses the `radiowave_tuned` wave policy; the aggressive single-kernel mode removes dependency fences from Redline's timed tape. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 32/45 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. This artifact records the `radiowave_tuned` wave policy as the controlled HIP launch-policy factor. The aggressive Redline timed IB has no dependency fences, but the completion timestamp still necessarily proves the dispatch finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
