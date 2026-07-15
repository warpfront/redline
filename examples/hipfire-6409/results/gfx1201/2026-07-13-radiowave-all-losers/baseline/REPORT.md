# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 97/133 rows (72.93%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 97 | 34 | 2 | 0 | 72.93 | 133 |
| vulkan | 35 | 79 | 4 | 15 | 26.32 | 133 |
| hipgraph | 1 | 8 | 39 | 85 | 0.75 | 133 |
| hip | 0 | 12 | 88 | 33 | 0.00 | 133 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 22 | 21 | 2 | 0 | 45 |
| serial_latency | vulkan | 22 | 13 | 1 | 9 | 45 |
| serial_latency | hipgraph | 1 | 8 | 29 | 7 | 45 |
| serial_latency | hip | 0 | 3 | 13 | 29 | 45 |
| independent_throughput | redline | 38 | 7 | 0 | 0 | 45 |
| independent_throughput | vulkan | 7 | 38 | 0 | 0 | 45 |
| independent_throughput | hipgraph | 0 | 0 | 4 | 41 | 45 |
| independent_throughput | hip | 0 | 0 | 41 | 4 | 45 |
| single_kernel_aggressive | redline | 37 | 6 | 0 | 0 | 43 |
| single_kernel_aggressive | vulkan | 6 | 28 | 3 | 6 | 43 |
| single_kernel_aggressive | hipgraph | 0 | 0 | 6 | 37 | 43 |
| single_kernel_aggressive | hip | 0 | 9 | 34 | 0 | 43 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/dispatch-grid/count=50,grid=1;hip-wave=32` | 2 | vulkan (+35.44%) | 1.6968 | 1.2528 |
| `serial_latency/dispatch-grid/count=200,grid=1;hip-wave=32` | 2 | vulkan (+48.59%) | 1.5828 | 1.0652 |
| `serial_latency/dispatch-grid/count=941,grid=1;hip-wave=32` | 2 | vulkan (+58.07%) | 1.5852 | 1.0029 |
| `serial_latency/dispatch-grid/count=64,grid=128;hip-wave=32` | 2 | vulkan (+155.55%) | 2.9931 | 1.1712 |
| `serial_latency/dispatch-grid/count=64,grid=1024;hip-wave=32` | 2 | vulkan (+72.28%) | 3.8181 | 2.2163 |
| `serial_latency/reduction/variant=wave,k=8192,rows=1;hip-wave=32` | 2 | hipgraph (+2.11%) | 136.8600 | 134.0262 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=4096,body=16;hip-wave=32` | 2 | vulkan (+21.22%) | 7.4006 | 6.1050 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+28.85%) | 8.7988 | 6.8288 |
| `serial_latency/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+20.95%) | 23.1913 | 19.1737 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=4096,body=16;hip-wave=32` | 2 | vulkan (+20.74%) | 7.6006 | 6.2950 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+64.52%) | 16.4563 | 10.0025 |
| `serial_latency/packed-dot/variant=q8-signed,n=4096,body=16;hip-wave=32` | 2 | vulkan (+4.44%) | 7.9238 | 7.5869 |
| `serial_latency/packed-dot/variant=q4-unsigned,n=4096,body=16;hip-wave=32` | 2 | vulkan (+4.47%) | 7.8931 | 7.5556 |
| `serial_latency/packed-dot/variant=q6-zero,n=4096,body=16;hip-wave=32` | 2 | vulkan (+0.52%) | 7.9400 | 7.8987 |
| `serial_latency/vopd/variant=independent-fma,n=32768,body=64;hip-wave=32` | 2 | vulkan (+48.48%) | 13.2575 | 8.9288 |
| `serial_latency/vopd/variant=dependent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+24.16%) | 11.2875 | 9.0913 |
| `serial_latency/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=32` | 2 | vulkan (+74.49%) | 28.3050 | 16.2213 |
| `serial_latency/vopd/variant=dequant-like,n=32768,body=64;hip-wave=32` | 2 | vulkan (+62.98%) | 23.1938 | 14.2312 |
| `serial_latency/sampler/argmax,vocab=32768,rows=1;hip-wave=32` | 2 | vulkan (+4.40%) | 43.4050 | 41.5763 |
| `serial_latency/sampler/argmax,vocab=32768,rows=4;hip-wave=32` | 2 | vulkan (+4.40%) | 43.7088 | 41.8675 |
| `serial_latency/sampler/argmax,vocab=131072,rows=1;hip-wave=32` | 3 | vulkan (+2.84%), hipgraph (+0.26%) | 164.5806 | 160.0319 |
| `serial_latency/sampler/argmax,vocab=131072,rows=4;hip-wave=32` | 3 | vulkan (+4.03%), hip (+0.22%) | 166.8294 | 160.3675 |
| `serial_latency/two-stage-reduction/k=32768,rows=4,splits=8;hip-wave=32` | 2 | vulkan (+0.66%) | 10.4575 | 10.3887 |
| `independent_throughput/dispatch-grid/count=64,grid=1024;hip-wave=32` | 2 | vulkan (+63.33%) | 2.2713 | 1.3906 |
| `independent_throughput/dispatch-grid/count=64,grid=8192;hip-wave=32` | 2 | vulkan (+51.87%) | 8.3131 | 5.4737 |
| `independent_throughput/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+20.71%) | 9.3763 | 7.7675 |
| `independent_throughput/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+22.30%) | 5.1963 | 4.2488 |
| `independent_throughput/vopd/variant=independent-fma,n=32768,body=64;hip-wave=32` | 2 | vulkan (+9.34%) | 4.4037 | 4.0275 |
| `independent_throughput/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=32` | 2 | vulkan (+41.05%) | 10.4887 | 7.4363 |
| `independent_throughput/vopd/variant=dequant-like,n=32768,body=64;hip-wave=32` | 2 | vulkan (+21.01%) | 8.0425 | 6.6463 |
| `single_kernel_aggressive/dispatch-grid/count=64,grid=8192;hip-wave=32` | 2 | vulkan (+6.90%) | 9.9200 | 9.2800 |
| `single_kernel_aggressive/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+13.39%) | 10.1600 | 8.9600 |
| `single_kernel_aggressive/vopd/variant=independent-fma,n=32768,body=64;hip-wave=32` | 2 | vulkan (+14.51%) | 8.8400 | 7.7200 |
| `single_kernel_aggressive/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=32` | 2 | vulkan (+37.28%) | 15.3200 | 11.1600 |
| `single_kernel_aggressive/vopd/variant=dequant-like,n=32768,body=64;hip-wave=32` | 2 | vulkan (+32.56%) | 13.6800 | 10.3200 |
| `single_kernel_aggressive/sampler/argmax,vocab=131072,rows=1;hip-wave=32` | 2 | vulkan (+1.22%) | 96.3600 | 95.2000 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 98 | 35 | 0 | 73.68 | 0.8056 | 133 |
| RL / hipgraph | 131 | 2 | 0 | 98.50 | 0.2414 | 133 |
| RL / hip | 132 | 1 | 0 | 99.25 | 0.2673 | 133 |

## Prior hipEngine-harness baseline

The old run is directional rather than row-for-row identical: it used a larger parameter sweep, only three contenders, and the previous kernels.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine | RL / vulkan | 186 | 26 | 87.74 | 0.4788 | 212 |
| hipEngine | RL / hip | 157 | 55 | 74.06 | 0.8342 | 212 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 132/133 rows and HipGraph in 131/133 while all three select identical per-row hipcc code objects. This run uses the `targeted_wave64` wave policy; the aggressive single-kernel mode removes dependency fences from Redline's timed tape. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 98/133 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. This artifact records the `targeted_wave64` wave policy as the controlled HIP launch-policy factor. The aggressive Redline timed IB has no dependency fences, but the completion timestamp still necessarily proves the dispatch finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
