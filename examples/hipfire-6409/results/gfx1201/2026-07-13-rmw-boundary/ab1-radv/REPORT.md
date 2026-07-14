# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 30/45 rows (66.67%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 30 | 15 | 0 | 0 | 66.67 | 45 |
| vulkan | 14 | 19 | 1 | 11 | 31.11 | 45 |
| hipgraph | 0 | 9 | 28 | 8 | 0.00 | 45 |
| hip | 1 | 2 | 16 | 26 | 2.22 | 45 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 30 | 15 | 0 | 0 | 45 |
| serial_latency | vulkan | 14 | 19 | 1 | 11 | 45 |
| serial_latency | hipgraph | 0 | 9 | 28 | 8 | 45 |
| serial_latency | hip | 1 | 2 | 16 | 26 | 45 |
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
| `serial_latency/dispatch-grid/count=50,grid=1;hip-wave=32` | 2 | vulkan (+35.59%) | 1.7280 | 1.2744 |
| `serial_latency/dispatch-grid/count=200,grid=1;hip-wave=32` | 2 | vulkan (+46.56%) | 1.6122 | 1.1000 |
| `serial_latency/dispatch-grid/count=941,grid=1;hip-wave=32` | 2 | vulkan (+55.53%) | 1.5943 | 1.0251 |
| `serial_latency/dispatch-grid/count=64,grid=128;hip-wave=32` | 2 | vulkan (+153.90%) | 2.9881 | 1.1769 |
| `serial_latency/dispatch-grid/count=64,grid=1024;hip-wave=32` | 2 | vulkan (+37.71%) | 3.0650 | 2.2256 |
| `serial_latency/reduction/variant=wave,k=8192,rows=1;hip-wave=32` | 2 | hip (+0.42%) | 140.3381 | 139.7507 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=4096,body=16;hip-wave=32` | 2 | vulkan (+20.28%) | 7.5088 | 6.2425 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+28.81%) | 8.9600 | 6.9562 |
| `serial_latency/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+20.69%) | 23.7375 | 19.6687 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=4096,body=16;hip-wave=64` | 2 | vulkan (+10.26%) | 7.1050 | 6.4437 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=64` | 2 | vulkan (+25.58%) | 12.8875 | 10.2625 |
| `serial_latency/vopd/variant=independent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+19.85%) | 11.0025 | 9.1800 |
| `serial_latency/vopd/variant=dependent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+18.67%) | 11.0813 | 9.3375 |
| `serial_latency/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+44.35%) | 24.0912 | 16.6900 |
| `serial_latency/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+45.50%) | 21.9000 | 15.0512 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 31 | 14 | 0 | 68.89 | 0.9316 | 45 |
| RL / hipgraph | 45 | 0 | 0 | 100.00 | 0.5540 | 45 |
| RL / hip | 44 | 1 | 0 | 97.78 | 0.5518 | 45 |

## Prior hipEngine-harness baseline

The old run is directional rather than row-for-row identical: it used a larger parameter sweep, only three contenders, and the previous kernels.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine | RL / vulkan | 186 | 26 | 87.74 | 0.4788 | 212 |
| hipEngine | RL / hip | 157 | 55 | 74.06 | 0.8342 | 212 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 44/45 rows and HipGraph in 45/45 while all three select identical per-row hipcc code objects. This run uses the `radiowave_tuned` wave policy; the aggressive single-kernel mode removes dependency fences from Redline's timed tape. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 31/45 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. This artifact records the `radiowave_tuned` wave policy as the controlled HIP launch-policy factor. The aggressive Redline timed IB has no dependency fences, but the completion timestamp still necessarily proves the dispatch finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
