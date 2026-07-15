# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 107/133 rows (80.45%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 107 | 26 | 0 | 0 | 80.45 | 133 |
| vulkan | 23 | 88 | 5 | 17 | 17.29 | 133 |
| hipgraph | 1 | 9 | 36 | 87 | 0.75 | 133 |
| hip | 2 | 10 | 92 | 29 | 1.50 | 133 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 26 | 19 | 0 | 0 | 45 |
| serial_latency | vulkan | 16 | 17 | 1 | 11 | 45 |
| serial_latency | hipgraph | 1 | 9 | 27 | 8 | 45 |
| serial_latency | hip | 2 | 0 | 17 | 26 | 45 |
| independent_throughput | redline | 41 | 4 | 0 | 0 | 45 |
| independent_throughput | vulkan | 4 | 41 | 0 | 0 | 45 |
| independent_throughput | hipgraph | 0 | 0 | 3 | 42 | 45 |
| independent_throughput | hip | 0 | 0 | 42 | 3 | 45 |
| single_kernel_aggressive | redline | 40 | 3 | 0 | 0 | 43 |
| single_kernel_aggressive | vulkan | 3 | 30 | 4 | 6 | 43 |
| single_kernel_aggressive | hipgraph | 0 | 0 | 6 | 37 | 43 |
| single_kernel_aggressive | hip | 0 | 10 | 33 | 0 | 43 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/dispatch-grid/count=50,grid=1;hip-wave=32` | 2 | vulkan (+38.14%) | 1.6776 | 1.2144 |
| `serial_latency/dispatch-grid/count=200,grid=1;hip-wave=32` | 2 | vulkan (+48.97%) | 1.5630 | 1.0492 |
| `serial_latency/dispatch-grid/count=941,grid=1;hip-wave=32` | 2 | vulkan (+56.46%) | 1.5393 | 0.9838 |
| `serial_latency/dispatch-grid/count=64,grid=128;hip-wave=32` | 2 | vulkan (+161.21%) | 2.9419 | 1.1262 |
| `serial_latency/dispatch-grid/count=64,grid=1024;hip-wave=32` | 2 | vulkan (+40.63%) | 3.0050 | 2.1369 |
| `serial_latency/reduction/variant=wave,k=8192,rows=1;hip-wave=32` | 2 | hipgraph (+0.32%) | 133.6831 | 133.2623 |
| `serial_latency/reduction/variant=lds-tree,k=8192,rows=1;hip-wave=32` | 2 | hip (+0.43%) | 133.6850 | 133.1110 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=4096,body=16;hip-wave=32` | 2 | vulkan (+23.14%) | 7.3662 | 5.9819 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+28.84%) | 8.6438 | 6.7088 |
| `serial_latency/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+21.29%) | 22.7375 | 18.7462 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=4096,body=16;hip-wave=64` | 2 | vulkan (+12.45%) | 6.9306 | 6.1631 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=64` | 2 | vulkan (+25.52%) | 12.2550 | 9.7637 |
| `serial_latency/packed-dot/variant=q8-signed,n=4096,body=16;hip-wave=32` | 2 | vulkan (+0.32%) | 7.4450 | 7.4212 |
| `serial_latency/packed-dot/variant=q4-unsigned,n=4096,body=16;hip-wave=32` | 2 | vulkan (+0.23%) | 7.4269 | 7.4100 |
| `serial_latency/vopd/variant=independent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+18.39%) | 10.3750 | 8.7637 |
| `serial_latency/vopd/variant=dependent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+19.13%) | 10.4600 | 8.7800 |
| `serial_latency/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+46.40%) | 22.4363 | 15.3250 |
| `serial_latency/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+46.18%) | 20.3888 | 13.9475 |
| `serial_latency/sampler/argmax,vocab=131072,rows=4;hip-wave=32` | 2 | hip (+0.22%) | 152.4963 | 152.1549 |
| `independent_throughput/dispatch-grid/count=941,grid=1;hip-wave=32` | 2 | vulkan (+1.84%) | 0.0870 | 0.0854 |
| `independent_throughput/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+22.47%) | 9.4213 | 7.6925 |
| `independent_throughput/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+19.84%) | 8.9475 | 7.4662 |
| `independent_throughput/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+16.97%) | 7.8137 | 6.6800 |
| `single_kernel_aggressive/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=64` | 2 | vulkan (+0.45%) | 8.9600 | 8.9200 |
| `single_kernel_aggressive/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+18.64%) | 13.2400 | 11.1600 |
| `single_kernel_aggressive/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+23.94%) | 12.8400 | 10.3600 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 110 | 23 | 0 | 82.71 | 0.7740 | 133 |
| RL / hipgraph | 132 | 1 | 0 | 99.25 | 0.2602 | 133 |
| RL / hip | 131 | 2 | 0 | 98.50 | 0.3395 | 133 |

## Prior hipEngine-harness baseline

The old run is directional rather than row-for-row identical: it used a larger parameter sweep, only three contenders, and the previous kernels.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine | RL / vulkan | 186 | 26 | 87.74 | 0.4788 | 212 |
| hipEngine | RL / hip | 157 | 55 | 74.06 | 0.8342 | 212 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 131/133 rows and HipGraph in 132/133 while all three select identical per-row hipcc code objects. This run uses the `radiowave_tuned` wave policy; the aggressive single-kernel mode removes dependency fences from Redline's timed tape. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 110/133 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. This artifact records the `radiowave_tuned` wave policy as the controlled HIP launch-policy factor. The aggressive Redline timed IB has no dependency fences, but the completion timestamp still necessarily proves the dispatch finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
