# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 115/133 rows (86.47%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 115 | 18 | 0 | 0 | 86.47 | 133 |
| vulkan | 18 | 91 | 8 | 16 | 13.53 | 133 |
| hipgraph | 0 | 11 | 37 | 85 | 0.00 | 133 |
| hip | 0 | 13 | 88 | 32 | 0.00 | 133 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 32 | 13 | 0 | 0 | 45 |
| serial_latency | vulkan | 13 | 19 | 2 | 11 | 45 |
| serial_latency | hipgraph | 0 | 11 | 31 | 3 | 45 |
| serial_latency | hip | 0 | 2 | 12 | 31 | 45 |
| independent_throughput | redline | 42 | 3 | 0 | 0 | 45 |
| independent_throughput | vulkan | 3 | 42 | 0 | 0 | 45 |
| independent_throughput | hipgraph | 0 | 0 | 1 | 44 | 45 |
| independent_throughput | hip | 0 | 0 | 44 | 1 | 45 |
| single_kernel_aggressive | redline | 41 | 2 | 0 | 0 | 43 |
| single_kernel_aggressive | vulkan | 2 | 30 | 6 | 5 | 43 |
| single_kernel_aggressive | hipgraph | 0 | 0 | 5 | 38 | 43 |
| single_kernel_aggressive | hip | 0 | 11 | 32 | 0 | 43 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/dispatch-grid/count=50,grid=1;hip-wave=32` | 2 | vulkan (+23.68%) | 1.6296 | 1.3176 |
| `serial_latency/dispatch-grid/count=200,grid=1;hip-wave=32` | 2 | vulkan (+39.38%) | 1.5374 | 1.1030 |
| `serial_latency/dispatch-grid/count=941,grid=1;hip-wave=32` | 2 | vulkan (+41.95%) | 1.5335 | 1.0803 |
| `serial_latency/dispatch-grid/count=64,grid=128;hip-wave=32` | 2 | vulkan (+146.52%) | 2.9644 | 1.2025 |
| `serial_latency/dispatch-grid/count=64,grid=1024;hip-wave=32` | 2 | vulkan (+32.84%) | 3.0463 | 2.2931 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=4096,body=16;hip-wave=32` | 2 | vulkan (+1.75%) | 6.3113 | 6.2025 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+22.89%) | 8.5162 | 6.9300 |
| `serial_latency/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+19.83%) | 23.4812 | 19.5963 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=64` | 2 | vulkan (+20.27%) | 12.2988 | 10.2263 |
| `serial_latency/vopd/variant=independent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+18.26%) | 10.8150 | 9.1450 |
| `serial_latency/vopd/variant=dependent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+16.88%) | 10.8525 | 9.2850 |
| `serial_latency/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+43.87%) | 23.9025 | 16.6137 |
| `serial_latency/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+43.79%) | 21.5738 | 15.0038 |
| `independent_throughput/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+20.92%) | 9.3925 | 7.7675 |
| `independent_throughput/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+19.92%) | 8.8887 | 7.4125 |
| `independent_throughput/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+16.87%) | 7.7838 | 6.6600 |
| `single_kernel_aggressive/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+19.42%) | 13.2800 | 11.1200 |
| `single_kernel_aggressive/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+21.71%) | 12.5600 | 10.3200 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 114 | 18 | 1 | 85.71 | 0.7401 | 133 |
| RL / hipgraph | 133 | 0 | 0 | 100.00 | 0.2558 | 133 |
| RL / hip | 133 | 0 | 0 | 100.00 | 0.3333 | 133 |

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
