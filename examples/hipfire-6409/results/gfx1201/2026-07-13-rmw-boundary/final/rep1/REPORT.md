# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 115/133 rows (86.47%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 115 | 18 | 0 | 0 | 86.47 | 133 |
| vulkan | 18 | 92 | 6 | 17 | 13.53 | 133 |
| hipgraph | 0 | 10 | 38 | 85 | 0.00 | 133 |
| hip | 0 | 13 | 89 | 31 | 0.00 | 133 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 32 | 13 | 0 | 0 | 45 |
| serial_latency | vulkan | 13 | 20 | 1 | 11 | 45 |
| serial_latency | hipgraph | 0 | 10 | 29 | 6 | 45 |
| serial_latency | hip | 0 | 2 | 15 | 28 | 45 |
| independent_throughput | redline | 42 | 3 | 0 | 0 | 45 |
| independent_throughput | vulkan | 3 | 42 | 0 | 0 | 45 |
| independent_throughput | hipgraph | 0 | 0 | 3 | 42 | 45 |
| independent_throughput | hip | 0 | 0 | 42 | 3 | 45 |
| single_kernel_aggressive | redline | 41 | 2 | 0 | 0 | 43 |
| single_kernel_aggressive | vulkan | 2 | 30 | 5 | 6 | 43 |
| single_kernel_aggressive | hipgraph | 0 | 0 | 6 | 37 | 43 |
| single_kernel_aggressive | hip | 0 | 11 | 32 | 0 | 43 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/dispatch-grid/count=50,grid=1;hip-wave=32` | 2 | vulkan (+25.05%) | 1.6136 | 1.2904 |
| `serial_latency/dispatch-grid/count=200,grid=1;hip-wave=32` | 2 | vulkan (+41.64%) | 1.5226 | 1.0750 |
| `serial_latency/dispatch-grid/count=941,grid=1;hip-wave=32` | 2 | vulkan (+49.51%) | 1.5244 | 1.0196 |
| `serial_latency/dispatch-grid/count=64,grid=128;hip-wave=32` | 2 | vulkan (+153.45%) | 2.9606 | 1.1681 |
| `serial_latency/dispatch-grid/count=64,grid=1024;hip-wave=32` | 2 | vulkan (+37.35%) | 3.0431 | 2.2156 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=4096,body=16;hip-wave=32` | 2 | vulkan (+1.36%) | 6.3075 | 6.2231 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+24.15%) | 8.6163 | 6.9400 |
| `serial_latency/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+19.40%) | 23.3762 | 19.5775 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=64` | 2 | vulkan (+25.24%) | 12.8187 | 10.2350 |
| `serial_latency/vopd/variant=independent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+19.52%) | 10.9125 | 9.1300 |
| `serial_latency/vopd/variant=dependent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+17.01%) | 10.8675 | 9.2875 |
| `serial_latency/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+43.76%) | 23.8863 | 16.6150 |
| `serial_latency/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+45.07%) | 21.7175 | 14.9700 |
| `independent_throughput/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+21.93%) | 9.3763 | 7.6900 |
| `independent_throughput/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+19.87%) | 8.8912 | 7.4175 |
| `independent_throughput/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+16.88%) | 7.7650 | 6.6437 |
| `single_kernel_aggressive/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+18.05%) | 13.0800 | 11.0800 |
| `single_kernel_aggressive/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+18.08%) | 12.2800 | 10.4000 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 114 | 18 | 1 | 85.71 | 0.7383 | 133 |
| RL / hipgraph | 133 | 0 | 0 | 100.00 | 0.2606 | 133 |
| RL / hip | 133 | 0 | 0 | 100.00 | 0.3465 | 133 |

## Prior hipEngine-harness baseline

The old run is directional rather than row-for-row identical: it used a larger parameter sweep, only three contenders, and the previous kernels.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine | RL / vulkan | 186 | 26 | 87.74 | 0.4788 | 212 |
| hipEngine | RL / hip | 157 | 55 | 74.06 | 0.8342 | 212 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 133/133 rows and HipGraph in 133/133 while all three select identical per-row hipcc code objects. This run uses the `radiowave_tuned` wave policy; the aggressive single-kernel mode removes dependency fences from Redline's timed tape. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 114/133 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. This artifact records the `radiowave_tuned` wave policy as the controlled HIP launch-policy factor. The aggressive Redline timed IB has no dependency fences, but the completion timestamp still necessarily proves the dispatch finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
