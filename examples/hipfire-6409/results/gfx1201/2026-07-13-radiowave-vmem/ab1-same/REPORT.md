# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 114/133 rows (85.71%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 114 | 19 | 0 | 0 | 85.71 | 133 |
| vulkan | 19 | 91 | 6 | 17 | 14.29 | 133 |
| hipgraph | 0 | 8 | 45 | 80 | 0.00 | 133 |
| hip | 0 | 15 | 82 | 36 | 0.00 | 133 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 32 | 13 | 0 | 0 | 45 |
| serial_latency | vulkan | 13 | 20 | 1 | 11 | 45 |
| serial_latency | hipgraph | 0 | 8 | 32 | 5 | 45 |
| serial_latency | hip | 0 | 4 | 12 | 29 | 45 |
| independent_throughput | redline | 41 | 4 | 0 | 0 | 45 |
| independent_throughput | vulkan | 4 | 41 | 0 | 0 | 45 |
| independent_throughput | hipgraph | 0 | 0 | 7 | 38 | 45 |
| independent_throughput | hip | 0 | 0 | 38 | 7 | 45 |
| single_kernel_aggressive | redline | 41 | 2 | 0 | 0 | 43 |
| single_kernel_aggressive | vulkan | 2 | 30 | 5 | 6 | 43 |
| single_kernel_aggressive | hipgraph | 0 | 0 | 6 | 37 | 43 |
| single_kernel_aggressive | hip | 0 | 11 | 32 | 0 | 43 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/dispatch-grid/count=50,grid=1;hip-wave=32` | 2 | vulkan (+26.79%) | 1.6280 | 1.2840 |
| `serial_latency/dispatch-grid/count=200,grid=1;hip-wave=32` | 2 | vulkan (+40.35%) | 1.5318 | 1.0914 |
| `serial_latency/dispatch-grid/count=941,grid=1;hip-wave=32` | 2 | vulkan (+47.02%) | 1.5085 | 1.0261 |
| `serial_latency/dispatch-grid/count=64,grid=128;hip-wave=32` | 2 | vulkan (+154.29%) | 2.9656 | 1.1663 |
| `serial_latency/dispatch-grid/count=64,grid=1024;hip-wave=32` | 2 | vulkan (+36.86%) | 3.0425 | 2.2231 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=4096,body=16;hip-wave=32` | 2 | vulkan (+0.21%) | 6.2588 | 6.2456 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+24.63%) | 8.7025 | 6.9825 |
| `serial_latency/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+20.00%) | 22.9462 | 19.1225 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=64` | 2 | vulkan (+25.12%) | 12.8712 | 10.2875 |
| `serial_latency/vopd/variant=independent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+18.14%) | 10.8750 | 9.2050 |
| `serial_latency/vopd/variant=dependent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+17.64%) | 10.9938 | 9.3450 |
| `serial_latency/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+43.48%) | 23.9425 | 16.6875 |
| `serial_latency/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+45.02%) | 21.8025 | 15.0337 |
| `independent_throughput/memory-waitcnt/variant=gather,n=4096,body=16;hip-wave=32` | 2 | vulkan (+6.25%) | 1.0406 | 0.9794 |
| `independent_throughput/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+22.46%) | 9.3637 | 7.6463 |
| `independent_throughput/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+19.39%) | 8.8812 | 7.4387 |
| `independent_throughput/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+15.97%) | 7.7713 | 6.7012 |
| `single_kernel_aggressive/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+19.13%) | 13.2000 | 11.0800 |
| `single_kernel_aggressive/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+20.08%) | 12.4400 | 10.3600 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 113 | 19 | 1 | 84.96 | 0.7360 | 133 |
| RL / hipgraph | 133 | 0 | 0 | 100.00 | 0.2567 | 133 |
| RL / hip | 133 | 0 | 0 | 100.00 | 0.3407 | 133 |

## Prior hipEngine-harness baseline

The old run is directional rather than row-for-row identical: it used a larger parameter sweep, only three contenders, and the previous kernels.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine | RL / vulkan | 186 | 26 | 87.74 | 0.4788 | 212 |
| hipEngine | RL / hip | 157 | 55 | 74.06 | 0.8342 | 212 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 133/133 rows and HipGraph in 133/133 while all three select identical per-row hipcc code objects. This run uses the `radiowave_tuned` wave policy; the aggressive single-kernel mode removes dependency fences from Redline's timed tape. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 113/133 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. This artifact records the `radiowave_tuned` wave policy as the controlled HIP launch-policy factor. The aggressive Redline timed IB has no dependency fences, but the completion timestamp still necessarily proves the dispatch finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
