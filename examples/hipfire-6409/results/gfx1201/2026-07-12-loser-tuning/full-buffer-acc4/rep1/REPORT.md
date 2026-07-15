# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 97/133 rows (72.93%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 97 | 34 | 2 | 0 | 72.93 | 133 |
| vulkan | 34 | 80 | 4 | 15 | 25.56 | 133 |
| hipgraph | 1 | 8 | 42 | 82 | 0.75 | 133 |
| hip | 1 | 11 | 85 | 36 | 0.75 | 133 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 22 | 21 | 2 | 0 | 45 |
| serial_latency | vulkan | 21 | 14 | 1 | 9 | 45 |
| serial_latency | hipgraph | 1 | 8 | 30 | 6 | 45 |
| serial_latency | hip | 1 | 2 | 12 | 30 | 45 |
| independent_throughput | redline | 38 | 7 | 0 | 0 | 45 |
| independent_throughput | vulkan | 7 | 38 | 0 | 0 | 45 |
| independent_throughput | hipgraph | 0 | 0 | 6 | 39 | 45 |
| independent_throughput | hip | 0 | 0 | 39 | 6 | 45 |
| single_kernel_aggressive | redline | 37 | 6 | 0 | 0 | 43 |
| single_kernel_aggressive | vulkan | 6 | 28 | 3 | 6 | 43 |
| single_kernel_aggressive | hipgraph | 0 | 0 | 6 | 37 | 43 |
| single_kernel_aggressive | hip | 0 | 9 | 34 | 0 | 43 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/dispatch-grid/count=50,grid=1;hip-wave=32` | 2 | vulkan (+34.84%) | 1.7000 | 1.2608 |
| `serial_latency/dispatch-grid/count=200,grid=1;hip-wave=32` | 2 | vulkan (+48.01%) | 1.5896 | 1.0740 |
| `serial_latency/dispatch-grid/count=941,grid=1;hip-wave=32` | 2 | vulkan (+55.06%) | 1.5665 | 1.0103 |
| `serial_latency/dispatch-grid/count=64,grid=128;hip-wave=32` | 2 | vulkan (+153.94%) | 3.0044 | 1.1831 |
| `serial_latency/dispatch-grid/count=64,grid=1024;hip-wave=32` | 2 | vulkan (+71.87%) | 3.8456 | 2.2375 |
| `serial_latency/reduction/variant=wave,k=8192,rows=1;hip-wave=32` | 2 | hipgraph (+0.33%) | 138.0844 | 137.6369 |
| `serial_latency/reduction/variant=lds-tree,k=8192,rows=1;hip-wave=32` | 2 | hip (+0.47%) | 138.2756 | 137.6294 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=4096,body=16;hip-wave=32` | 2 | vulkan (+22.29%) | 7.5237 | 6.1525 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+29.31%) | 8.8562 | 6.8487 |
| `serial_latency/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+21.12%) | 23.4525 | 19.3625 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=4096,body=16;hip-wave=32` | 2 | vulkan (+20.45%) | 7.6400 | 6.3431 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+63.64%) | 16.5212 | 10.0962 |
| `serial_latency/packed-dot/variant=q8-signed,n=4096,body=16;hip-wave=32` | 2 | vulkan (+2.20%) | 7.8256 | 7.6569 |
| `serial_latency/packed-dot/variant=q4-unsigned,n=4096,body=16;hip-wave=32` | 2 | vulkan (+2.46%) | 7.8075 | 7.6200 |
| `serial_latency/packed-dot/variant=q6-zero,n=4096,body=16;hip-wave=32` | 2 | vulkan (+1.32%) | 8.0725 | 7.9675 |
| `serial_latency/vopd/variant=independent-fma,n=32768,body=64;hip-wave=32` | 2 | vulkan (+48.83%) | 13.4413 | 9.0312 |
| `serial_latency/vopd/variant=dependent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+23.16%) | 11.2875 | 9.1650 |
| `serial_latency/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=32` | 2 | vulkan (+74.32%) | 28.5487 | 16.3775 |
| `serial_latency/vopd/variant=dequant-like,n=32768,body=64;hip-wave=32` | 2 | vulkan (+61.89%) | 23.9763 | 14.8100 |
| `serial_latency/sampler/argmax,vocab=32768,rows=1;hip-wave=32` | 2 | vulkan (+4.35%) | 43.7206 | 41.8994 |
| `serial_latency/sampler/argmax,vocab=32768,rows=4;hip-wave=32` | 2 | vulkan (+4.16%) | 43.5862 | 41.8462 |
| `serial_latency/sampler/argmax,vocab=131072,rows=1;hip-wave=32` | 3 | vulkan (+2.88%), hipgraph (+1.87%) | 165.7544 | 161.1188 |
| `serial_latency/sampler/argmax,vocab=131072,rows=4;hip-wave=32` | 3 | vulkan (+4.12%), hip (+0.17%) | 168.0006 | 161.3475 |
| `independent_throughput/dispatch-grid/count=64,grid=1024;hip-wave=32` | 2 | vulkan (+61.71%) | 2.2619 | 1.3987 |
| `independent_throughput/dispatch-grid/count=64,grid=8192;hip-wave=32` | 2 | vulkan (+50.54%) | 8.2788 | 5.4994 |
| `independent_throughput/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+22.53%) | 9.4162 | 7.6850 |
| `independent_throughput/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+21.59%) | 5.1875 | 4.2663 |
| `independent_throughput/vopd/variant=independent-fma,n=32768,body=64;hip-wave=32` | 2 | vulkan (+8.53%) | 4.4062 | 4.0600 |
| `independent_throughput/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=32` | 2 | vulkan (+41.11%) | 10.5425 | 7.4713 |
| `independent_throughput/vopd/variant=dequant-like,n=32768,body=64;hip-wave=32` | 2 | vulkan (+20.83%) | 8.0700 | 6.6787 |
| `single_kernel_aggressive/dispatch-grid/count=64,grid=8192;hip-wave=32` | 2 | vulkan (+16.45%) | 10.7600 | 9.2400 |
| `single_kernel_aggressive/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+12.11%) | 10.0000 | 8.9200 |
| `single_kernel_aggressive/vopd/variant=independent-fma,n=32768,body=64;hip-wave=32` | 2 | vulkan (+12.95%) | 8.7200 | 7.7200 |
| `single_kernel_aggressive/vopd/variant=dependent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+1.03%) | 7.8400 | 7.7600 |
| `single_kernel_aggressive/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=32` | 2 | vulkan (+38.13%) | 15.3600 | 11.1200 |
| `single_kernel_aggressive/vopd/variant=dequant-like,n=32768,body=64;hip-wave=32` | 2 | vulkan (+31.27%) | 13.6000 | 10.3600 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 99 | 34 | 0 | 74.44 | 0.8204 | 133 |
| RL / hipgraph | 131 | 2 | 0 | 98.50 | 0.2397 | 133 |
| RL / hip | 131 | 2 | 0 | 98.50 | 0.2654 | 133 |

## Prior hipEngine-harness baseline

The old run is directional rather than row-for-row identical: it used a larger parameter sweep, only three contenders, and the previous kernels.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine | RL / vulkan | 186 | 26 | 87.74 | 0.4788 | 212 |
| hipEngine | RL / hip | 157 | 55 | 74.06 | 0.8342 | 212 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 131/133 rows and HipGraph in 131/133 while all three select identical per-row hipcc code objects. This run uses the `targeted_wave64` wave policy; the aggressive single-kernel mode removes dependency fences from Redline's timed tape. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 99/133 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. This artifact records the `targeted_wave64` wave policy as the controlled HIP launch-policy factor. The aggressive Redline timed IB has no dependency fences, but the completion timestamp still necessarily proves the dispatch finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
