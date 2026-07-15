# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 110/133 rows (82.71%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 110 | 23 | 0 | 0 | 82.71 | 133 |
| vulkan | 21 | 89 | 6 | 17 | 15.79 | 133 |
| hipgraph | 0 | 8 | 46 | 79 | 0.00 | 133 |
| hip | 2 | 13 | 81 | 37 | 1.50 | 133 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 29 | 16 | 0 | 0 | 45 |
| serial_latency | vulkan | 14 | 19 | 1 | 11 | 45 |
| serial_latency | hipgraph | 0 | 8 | 29 | 8 | 45 |
| serial_latency | hip | 2 | 2 | 15 | 26 | 45 |
| independent_throughput | redline | 41 | 4 | 0 | 0 | 45 |
| independent_throughput | vulkan | 4 | 41 | 0 | 0 | 45 |
| independent_throughput | hipgraph | 0 | 0 | 11 | 34 | 45 |
| independent_throughput | hip | 0 | 0 | 34 | 11 | 45 |
| single_kernel_aggressive | redline | 40 | 3 | 0 | 0 | 43 |
| single_kernel_aggressive | vulkan | 3 | 29 | 5 | 6 | 43 |
| single_kernel_aggressive | hipgraph | 0 | 0 | 6 | 37 | 43 |
| single_kernel_aggressive | hip | 0 | 11 | 32 | 0 | 43 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/dispatch-grid/count=50,grid=1;hip-wave=32` | 2 | vulkan (+34.81%) | 1.7288 | 1.2824 |
| `serial_latency/dispatch-grid/count=200,grid=1;hip-wave=32` | 2 | vulkan (+47.36%) | 1.6098 | 1.0924 |
| `serial_latency/dispatch-grid/count=941,grid=1;hip-wave=32` | 2 | vulkan (+53.78%) | 1.5800 | 1.0274 |
| `serial_latency/dispatch-grid/count=64,grid=128;hip-wave=32` | 2 | vulkan (+155.47%) | 2.9794 | 1.1663 |
| `serial_latency/dispatch-grid/count=64,grid=1024;hip-wave=32` | 2 | vulkan (+37.50%) | 3.0594 | 2.2250 |
| `serial_latency/reduction/variant=lds-tree,k=8192,rows=1;hip-wave=32` | 2 | hip (+0.48%) | 141.3581 | 140.6887 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=4096,body=16;hip-wave=32` | 2 | vulkan (+19.58%) | 7.4975 | 6.2700 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+28.78%) | 8.9938 | 6.9837 |
| `serial_latency/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+21.54%) | 24.0350 | 19.7750 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=4096,body=16;hip-wave=64` | 2 | vulkan (+10.23%) | 7.1206 | 6.4600 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=64` | 2 | vulkan (+25.10%) | 12.9337 | 10.3387 |
| `serial_latency/vopd/variant=independent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+37.88%) | 12.7613 | 9.2550 |
| `serial_latency/vopd/variant=dependent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+23.32%) | 11.5663 | 9.3788 |
| `serial_latency/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+55.20%) | 26.0562 | 16.7888 |
| `serial_latency/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+48.90%) | 22.5775 | 15.1625 |
| `serial_latency/sampler/argmax,vocab=131072,rows=4;hip-wave=32` | 2 | hip (+0.54%) | 160.2931 | 159.4262 |
| `independent_throughput/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+21.98%) | 9.3712 | 7.6825 |
| `independent_throughput/vopd/variant=independent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+0.03%) | 4.0400 | 4.0388 |
| `independent_throughput/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+26.21%) | 9.3912 | 7.4413 |
| `independent_throughput/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+22.65%) | 7.8325 | 6.3863 |
| `single_kernel_aggressive/vopd/variant=independent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+9.33%) | 8.4400 | 7.7200 |
| `single_kernel_aggressive/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+24.73%) | 13.9200 | 11.1600 |
| `single_kernel_aggressive/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+24.03%) | 12.8000 | 10.3200 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 112 | 21 | 0 | 84.21 | 0.7766 | 133 |
| RL / hipgraph | 133 | 0 | 0 | 100.00 | 0.2602 | 133 |
| RL / hip | 131 | 2 | 0 | 98.50 | 0.3483 | 133 |

## Prior hipEngine-harness baseline

The old run is directional rather than row-for-row identical: it used a larger parameter sweep, only three contenders, and the previous kernels.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine | RL / vulkan | 186 | 26 | 87.74 | 0.4788 | 212 |
| hipEngine | RL / hip | 157 | 55 | 74.06 | 0.8342 | 212 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 131/133 rows and HipGraph in 133/133 while all three select identical per-row hipcc code objects. This run uses the `radiowave_tuned` wave policy; the aggressive single-kernel mode removes dependency fences from Redline's timed tape. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 112/133 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. This artifact records the `radiowave_tuned` wave policy as the controlled HIP launch-policy factor. The aggressive Redline timed IB has no dependency fences, but the completion timestamp still necessarily proves the dispatch finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
