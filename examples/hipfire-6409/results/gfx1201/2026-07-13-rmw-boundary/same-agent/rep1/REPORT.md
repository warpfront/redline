# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 27/45 rows (60.00%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 27 | 18 | 0 | 0 | 60.00 | 45 |
| vulkan | 15 | 18 | 1 | 11 | 33.33 | 45 |
| hipgraph | 1 | 8 | 30 | 6 | 2.22 | 45 |
| hip | 2 | 1 | 14 | 28 | 4.44 | 45 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 27 | 18 | 0 | 0 | 45 |
| serial_latency | vulkan | 15 | 18 | 1 | 11 | 45 |
| serial_latency | hipgraph | 1 | 8 | 30 | 6 | 45 |
| serial_latency | hip | 2 | 1 | 14 | 28 | 45 |
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
| `serial_latency/dispatch-grid/count=50,grid=1;hip-wave=32` | 2 | vulkan (+35.80%) | 1.7296 | 1.2736 |
| `serial_latency/dispatch-grid/count=200,grid=1;hip-wave=32` | 2 | vulkan (+46.43%) | 1.6128 | 1.1014 |
| `serial_latency/dispatch-grid/count=941,grid=1;hip-wave=32` | 2 | vulkan (+55.87%) | 1.5982 | 1.0253 |
| `serial_latency/dispatch-grid/count=64,grid=128;hip-wave=32` | 2 | vulkan (+152.41%) | 2.9737 | 1.1781 |
| `serial_latency/dispatch-grid/count=64,grid=1024;hip-wave=32` | 2 | vulkan (+37.36%) | 3.0631 | 2.2300 |
| `serial_latency/reduction/variant=wave,k=8192,rows=1;hip-wave=32` | 2 | hip (+2.02%) | 140.3900 | 137.6131 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=4096,body=16;hip-wave=32` | 2 | vulkan (+20.30%) | 7.5044 | 6.2381 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+28.60%) | 8.9763 | 6.9800 |
| `serial_latency/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+20.24%) | 23.0888 | 19.2025 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=4096,body=16;hip-wave=64` | 2 | vulkan (+11.33%) | 7.1769 | 6.4463 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=64` | 2 | vulkan (+26.37%) | 12.6925 | 10.0437 |
| `serial_latency/packed-dot/variant=q8-signed,n=4096,body=16;hip-wave=32` | 2 | vulkan (+0.10%) | 7.7544 | 7.7469 |
| `serial_latency/vopd/variant=independent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+20.34%) | 11.0650 | 9.1950 |
| `serial_latency/vopd/variant=dependent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+18.25%) | 11.0225 | 9.3212 |
| `serial_latency/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+43.92%) | 24.0512 | 16.7112 |
| `serial_latency/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+45.57%) | 21.9250 | 15.0612 |
| `serial_latency/sampler/argmax,vocab=131072,rows=1;hip-wave=32` | 2 | hipgraph (+0.73%) | 157.3975 | 156.2506 |
| `serial_latency/sampler/argmax,vocab=131072,rows=4;hip-wave=32` | 2 | hip (+0.76%) | 159.8963 | 158.6869 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 30 | 15 | 0 | 66.67 | 0.9287 | 45 |
| RL / hipgraph | 44 | 1 | 0 | 97.78 | 0.5546 | 45 |
| RL / hip | 43 | 2 | 0 | 95.56 | 0.5500 | 45 |

## Prior hipEngine-harness baseline

The old run is directional rather than row-for-row identical: it used a larger parameter sweep, only three contenders, and the previous kernels.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine | RL / vulkan | 186 | 26 | 87.74 | 0.4788 | 212 |
| hipEngine | RL / hip | 157 | 55 | 74.06 | 0.8342 | 212 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 43/45 rows and HipGraph in 44/45 while all three select identical per-row hipcc code objects. This run uses the `radiowave_tuned` wave policy; the aggressive single-kernel mode removes dependency fences from Redline's timed tape. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 30/45 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. This artifact records the `radiowave_tuned` wave policy as the controlled HIP launch-policy factor. The aggressive Redline timed IB has no dependency fences, but the completion timestamp still necessarily proves the dispatch finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
