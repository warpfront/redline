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
| serial_latency | hipgraph | 0 | 8 | 29 | 8 | 45 |
| serial_latency | hip | 0 | 4 | 15 | 26 | 45 |
| independent_throughput | redline | 42 | 3 | 0 | 0 | 45 |
| independent_throughput | vulkan | 3 | 42 | 0 | 0 | 45 |
| independent_throughput | hipgraph | 0 | 0 | 10 | 35 | 45 |
| independent_throughput | hip | 0 | 0 | 35 | 10 | 45 |
| single_kernel_aggressive | redline | 40 | 3 | 0 | 0 | 43 |
| single_kernel_aggressive | vulkan | 3 | 29 | 5 | 6 | 43 |
| single_kernel_aggressive | hipgraph | 0 | 0 | 6 | 37 | 43 |
| single_kernel_aggressive | hip | 0 | 11 | 32 | 0 | 43 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/dispatch-grid/count=50,grid=1;hip-wave=32` | 2 | vulkan (+28.08%) | 1.6384 | 1.2792 |
| `serial_latency/dispatch-grid/count=200,grid=1;hip-wave=32` | 2 | vulkan (+38.53%) | 1.5332 | 1.1068 |
| `serial_latency/dispatch-grid/count=941,grid=1;hip-wave=32` | 2 | vulkan (+46.22%) | 1.5422 | 1.0547 |
| `serial_latency/dispatch-grid/count=64,grid=128;hip-wave=32` | 2 | vulkan (+155.01%) | 2.9581 | 1.1600 |
| `serial_latency/dispatch-grid/count=64,grid=1024;hip-wave=32` | 2 | vulkan (+35.06%) | 3.0288 | 2.2425 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=4096,body=16;hip-wave=32` | 2 | vulkan (+2.63%) | 6.2969 | 6.1356 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+24.15%) | 8.5862 | 6.9162 |
| `serial_latency/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+20.36%) | 22.6975 | 18.8588 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=64` | 2 | vulkan (+19.88%) | 12.1713 | 10.1525 |
| `serial_latency/vopd/variant=independent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+16.23%) | 10.5337 | 9.0625 |
| `serial_latency/vopd/variant=dependent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+17.49%) | 10.6575 | 9.0712 |
| `serial_latency/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+45.90%) | 22.9887 | 15.7562 |
| `serial_latency/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+46.19%) | 20.8600 | 14.2688 |
| `independent_throughput/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+21.75%) | 9.4113 | 7.7300 |
| `independent_throughput/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+19.80%) | 8.9400 | 7.4625 |
| `independent_throughput/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+16.98%) | 7.8113 | 6.6775 |
| `single_kernel_aggressive/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=64` | 2 | vulkan (+0.90%) | 9.0000 | 8.9200 |
| `single_kernel_aggressive/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+20.86%) | 13.4400 | 11.1200 |
| `single_kernel_aggressive/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+18.46%) | 12.3200 | 10.4000 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 114 | 19 | 0 | 85.71 | 0.7385 | 133 |
| RL / hipgraph | 133 | 0 | 0 | 100.00 | 0.2551 | 133 |
| RL / hip | 133 | 0 | 0 | 100.00 | 0.3520 | 133 |

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
