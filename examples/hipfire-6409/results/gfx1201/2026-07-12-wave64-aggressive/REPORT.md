# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 91/133 rows (68.42%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 91 | 40 | 2 | 0 | 68.42 | 133 |
| vulkan | 41 | 74 | 4 | 14 | 30.83 | 133 |
| hipgraph | 1 | 8 | 35 | 89 | 0.75 | 133 |
| hip | 0 | 11 | 92 | 30 | 0.00 | 133 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 21 | 22 | 2 | 0 | 45 |
| serial_latency | vulkan | 23 | 13 | 1 | 8 | 45 |
| serial_latency | hipgraph | 1 | 8 | 28 | 8 | 45 |
| serial_latency | hip | 0 | 2 | 14 | 29 | 45 |
| independent_throughput | redline | 34 | 11 | 0 | 0 | 45 |
| independent_throughput | vulkan | 11 | 34 | 0 | 0 | 45 |
| independent_throughput | hipgraph | 0 | 0 | 1 | 44 | 45 |
| independent_throughput | hip | 0 | 0 | 44 | 1 | 45 |
| single_kernel_aggressive | redline | 36 | 7 | 0 | 0 | 43 |
| single_kernel_aggressive | vulkan | 7 | 27 | 3 | 6 | 43 |
| single_kernel_aggressive | hipgraph | 0 | 0 | 6 | 37 | 43 |
| single_kernel_aggressive | hip | 0 | 9 | 34 | 0 | 43 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/dispatch-grid/count=50,grid=1;hip-wave=64` | 2 | vulkan (+35.74%) | 1.6864 | 1.2424 |
| `serial_latency/dispatch-grid/count=200,grid=1;hip-wave=64` | 2 | vulkan (+48.87%) | 1.5750 | 1.0580 |
| `serial_latency/dispatch-grid/count=941,grid=1;hip-wave=64` | 2 | vulkan (+57.92%) | 1.5742 | 0.9969 |
| `serial_latency/dispatch-grid/count=64,grid=128;hip-wave=64` | 2 | vulkan (+162.80%) | 2.9581 | 1.1256 |
| `serial_latency/dispatch-grid/count=64,grid=1024;hip-wave=64` | 2 | vulkan (+58.34%) | 3.4062 | 2.1513 |
| `serial_latency/reduction/variant=wave,k=8192,rows=1;hip-wave=32` | 2 | hipgraph (+0.36%) | 136.1056 | 135.6132 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=4096,body=16;hip-wave=64` | 2 | vulkan (+26.29%) | 7.6712 | 6.0744 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=16;hip-wave=64` | 2 | vulkan (+32.40%) | 9.0062 | 6.8025 |
| `serial_latency/memory-waitcnt/variant=gather,n=4096,body=16;hip-wave=64` | 2 | vulkan (+16.35%) | 10.6394 | 9.1444 |
| `serial_latency/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=64` | 2 | vulkan (+23.90%) | 23.0212 | 18.5813 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=4096,body=16;hip-wave=64` | 2 | vulkan (+18.96%) | 7.4475 | 6.2606 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=64` | 2 | vulkan (+55.72%) | 15.5425 | 9.9812 |
| `serial_latency/packed-dot/variant=q8-signed,n=4096,body=16;hip-wave=64` | 2 | vulkan (+33.68%) | 10.1237 | 7.5731 |
| `serial_latency/packed-dot/variant=q4-unsigned,n=4096,body=16;hip-wave=64` | 2 | vulkan (+33.90%) | 10.0975 | 7.5412 |
| `serial_latency/packed-dot/variant=q6-zero,n=4096,body=16;hip-wave=64` | 2 | vulkan (+32.80%) | 10.4406 | 7.8619 |
| `serial_latency/vopd/variant=independent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+41.65%) | 12.6050 | 8.8987 |
| `serial_latency/vopd/variant=dependent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+24.05%) | 11.2188 | 9.0437 |
| `serial_latency/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+61.33%) | 25.2638 | 15.6600 |
| `serial_latency/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+49.87%) | 21.8100 | 14.5525 |
| `serial_latency/sampler/argmax,vocab=32768,rows=1;hip-wave=64` | 2 | vulkan (+5.31%) | 43.5619 | 41.3663 |
| `serial_latency/sampler/argmax,vocab=32768,rows=4;hip-wave=64` | 2 | vulkan (+5.48%) | 43.7431 | 41.4706 |
| `serial_latency/sampler/argmax,vocab=131072,rows=1;hip-wave=64` | 3 | vulkan (+3.31%), hip (+1.77%) | 164.4775 | 159.2075 |
| `serial_latency/sampler/argmax,vocab=131072,rows=4;hip-wave=64` | 3 | vulkan (+2.99%), hipgraph (+0.69%) | 165.6213 | 160.8194 |
| `serial_latency/two-stage-reduction/k=32768,rows=4,splits=8;hip-wave=64` | 2 | vulkan (+3.98%) | 10.7462 | 10.3350 |
| `independent_throughput/dispatch-grid/count=941,grid=1;hip-wave=64` | 2 | vulkan (+2.82%) | 0.0868 | 0.0845 |
| `independent_throughput/dispatch-grid/count=64,grid=1024;hip-wave=64` | 2 | vulkan (+103.17%) | 2.4013 | 1.1819 |
| `independent_throughput/dispatch-grid/count=64,grid=8192;hip-wave=64` | 2 | vulkan (+46.67%) | 8.1062 | 5.5269 |
| `independent_throughput/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=64` | 2 | vulkan (+22.02%) | 9.3863 | 7.6925 |
| `independent_throughput/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=64` | 2 | vulkan (+20.13%) | 5.0275 | 4.1850 |
| `independent_throughput/packed-dot/variant=q8-signed,n=4096,body=16;hip-wave=64` | 2 | vulkan (+72.09%) | 2.3275 | 1.3525 |
| `independent_throughput/packed-dot/variant=q4-unsigned,n=4096,body=16;hip-wave=64` | 2 | vulkan (+67.73%) | 2.2969 | 1.3694 |
| `independent_throughput/packed-dot/variant=q6-zero,n=4096,body=16;hip-wave=64` | 2 | vulkan (+59.09%) | 2.2044 | 1.3856 |
| `independent_throughput/vopd/variant=independent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+2.13%) | 4.1325 | 4.0462 |
| `independent_throughput/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+26.87%) | 9.4362 | 7.4375 |
| `independent_throughput/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+18.22%) | 7.8438 | 6.6350 |
| `single_kernel_aggressive/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=64` | 2 | vulkan (+1.20%) | 13.5200 | 13.3600 |
| `single_kernel_aggressive/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=64` | 2 | vulkan (+9.87%) | 9.8000 | 8.9200 |
| `single_kernel_aggressive/vopd/variant=independent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+6.70%) | 8.2800 | 7.7600 |
| `single_kernel_aggressive/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+28.32%) | 14.3200 | 11.1600 |
| `single_kernel_aggressive/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+23.55%) | 12.8000 | 10.3600 |
| `single_kernel_aggressive/sampler/argmax,vocab=131072,rows=1;hip-wave=64` | 2 | vulkan (+1.52%) | 96.0400 | 94.6000 |
| `single_kernel_aggressive/sampler/argmax,vocab=131072,rows=4;hip-wave=64` | 2 | vulkan (+0.25%) | 95.2800 | 95.0400 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 91 | 41 | 1 | 68.42 | 0.8493 | 133 |
| RL / hipgraph | 131 | 2 | 0 | 98.50 | 0.2649 | 133 |
| RL / hip | 132 | 1 | 0 | 99.25 | 0.3358 | 133 |

## Prior hipEngine-harness baseline

The old run is directional rather than row-for-row identical: it used a larger parameter sweep, only three contenders, and the previous kernels.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine | RL / vulkan | 186 | 26 | 87.74 | 0.4788 | 212 |
| hipEngine | RL / hip | 157 | 55 | 74.06 | 0.8342 | 212 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 132/133 rows and HipGraph in 131/133 while all three select identical per-row hipcc code objects. Prior Vulkan-winning families use explicitly wave64 HIP; the aggressive single-kernel mode removes dependency fences from Redline's timed tape. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 91/133 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. Wave64 is applied as a controlled HIP launch-policy change only to families where Vulkan previously beat Redline. The aggressive Redline timed IB has no dependency fences, but the completion timestamp still necessarily proves the dispatch finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
