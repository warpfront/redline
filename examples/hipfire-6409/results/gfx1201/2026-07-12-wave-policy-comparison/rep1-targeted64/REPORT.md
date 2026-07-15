# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 92/133 rows (69.17%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 92 | 39 | 2 | 0 | 69.17 | 133 |
| vulkan | 39 | 76 | 5 | 13 | 29.32 | 133 |
| hipgraph | 1 | 8 | 38 | 86 | 0.75 | 133 |
| hip | 1 | 10 | 88 | 34 | 0.75 | 133 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 20 | 23 | 2 | 0 | 45 |
| serial_latency | vulkan | 23 | 13 | 1 | 8 | 45 |
| serial_latency | hipgraph | 1 | 8 | 31 | 5 | 45 |
| serial_latency | hip | 1 | 1 | 11 | 32 | 45 |
| independent_throughput | redline | 36 | 9 | 0 | 0 | 45 |
| independent_throughput | vulkan | 9 | 36 | 0 | 0 | 45 |
| independent_throughput | hipgraph | 0 | 0 | 2 | 43 | 45 |
| independent_throughput | hip | 0 | 0 | 43 | 2 | 45 |
| single_kernel_aggressive | redline | 36 | 7 | 0 | 0 | 43 |
| single_kernel_aggressive | vulkan | 7 | 27 | 4 | 5 | 43 |
| single_kernel_aggressive | hipgraph | 0 | 0 | 5 | 38 | 43 |
| single_kernel_aggressive | hip | 0 | 9 | 34 | 0 | 43 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/dispatch-grid/count=50,grid=1;hip-wave=32` | 2 | vulkan (+32.79%) | 1.7592 | 1.3248 |
| `serial_latency/dispatch-grid/count=200,grid=1;hip-wave=32` | 2 | vulkan (+46.48%) | 1.6332 | 1.1150 |
| `serial_latency/dispatch-grid/count=941,grid=1;hip-wave=32` | 2 | vulkan (+53.80%) | 1.6347 | 1.0629 |
| `serial_latency/dispatch-grid/count=64,grid=128;hip-wave=32` | 2 | vulkan (+154.72%) | 3.0169 | 1.1844 |
| `serial_latency/dispatch-grid/count=64,grid=1024;hip-wave=32` | 2 | vulkan (+70.59%) | 3.8500 | 2.2569 |
| `serial_latency/reduction/variant=wave,k=8192,rows=1;hip-wave=32` | 2 | hipgraph (+0.25%) | 138.9525 | 138.6052 |
| `serial_latency/reduction/variant=lds-tree,k=8192,rows=1;hip-wave=32` | 2 | hip (+0.60%) | 139.2406 | 138.4152 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=4096,body=16;hip-wave=32` | 2 | vulkan (+19.39%) | 7.3425 | 6.1500 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+26.53%) | 8.7700 | 6.9313 |
| `serial_latency/memory-waitcnt/variant=gather,n=4096,body=16;hip-wave=32` | 2 | vulkan (+12.95%) | 10.4675 | 9.2675 |
| `serial_latency/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+23.12%) | 23.9862 | 19.4812 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=4096,body=16;hip-wave=32` | 2 | vulkan (+20.72%) | 7.6750 | 6.3575 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+62.44%) | 16.5425 | 10.1837 |
| `serial_latency/packed-dot/variant=q8-signed,n=4096,body=16;hip-wave=32` | 2 | vulkan (+26.61%) | 9.7688 | 7.7156 |
| `serial_latency/packed-dot/variant=q4-unsigned,n=4096,body=16;hip-wave=32` | 2 | vulkan (+27.07%) | 9.7469 | 7.6706 |
| `serial_latency/packed-dot/variant=q6-zero,n=4096,body=16;hip-wave=32` | 2 | vulkan (+23.41%) | 9.8863 | 8.0106 |
| `serial_latency/vopd/variant=independent-fma,n=32768,body=64;hip-wave=32` | 2 | vulkan (+48.40%) | 13.4825 | 9.0850 |
| `serial_latency/vopd/variant=dependent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+23.70%) | 11.4162 | 9.2287 |
| `serial_latency/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=32` | 2 | vulkan (+73.80%) | 28.6750 | 16.4988 |
| `serial_latency/vopd/variant=dequant-like,n=32768,body=64;hip-wave=32` | 2 | vulkan (+61.37%) | 23.3425 | 14.4650 |
| `serial_latency/sampler/argmax,vocab=32768,rows=1;hip-wave=32` | 2 | vulkan (+4.39%) | 43.9625 | 42.1156 |
| `serial_latency/sampler/argmax,vocab=32768,rows=4;hip-wave=32` | 2 | vulkan (+4.89%) | 44.2756 | 42.2106 |
| `serial_latency/sampler/argmax,vocab=131072,rows=1;hip-wave=32` | 3 | vulkan (+2.84%), hipgraph (+0.71%) | 166.7063 | 162.1044 |
| `serial_latency/sampler/argmax,vocab=131072,rows=4;hip-wave=32` | 3 | vulkan (+3.23%), hip (+0.19%) | 168.9162 | 163.6344 |
| `serial_latency/two-stage-reduction/k=32768,rows=4,splits=8;hip-wave=32` | 2 | vulkan (+0.38%) | 10.6237 | 10.5838 |
| `independent_throughput/dispatch-grid/count=64,grid=1024;hip-wave=32` | 2 | vulkan (+55.41%) | 2.2719 | 1.4619 |
| `independent_throughput/dispatch-grid/count=64,grid=8192;hip-wave=32` | 2 | vulkan (+50.61%) | 8.2338 | 5.4669 |
| `independent_throughput/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+23.68%) | 9.5113 | 7.6900 |
| `independent_throughput/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+21.75%) | 5.2138 | 4.2825 |
| `independent_throughput/packed-dot/variant=q8-signed,n=4096,body=16;hip-wave=32` | 2 | vulkan (+2.25%) | 1.4481 | 1.4163 |
| `independent_throughput/packed-dot/variant=q4-unsigned,n=4096,body=16;hip-wave=32` | 2 | vulkan (+4.90%) | 1.4587 | 1.3906 |
| `independent_throughput/vopd/variant=independent-fma,n=32768,body=64;hip-wave=32` | 2 | vulkan (+8.72%) | 4.4088 | 4.0550 |
| `independent_throughput/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=32` | 2 | vulkan (+41.24%) | 10.5062 | 7.4387 |
| `independent_throughput/vopd/variant=dequant-like,n=32768,body=64;hip-wave=32` | 2 | vulkan (+22.04%) | 8.0975 | 6.6350 |
| `single_kernel_aggressive/dispatch-grid/count=64,grid=8192;hip-wave=32` | 2 | vulkan (+17.75%) | 10.8800 | 9.2400 |
| `single_kernel_aggressive/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+1.50%) | 13.5200 | 13.3200 |
| `single_kernel_aggressive/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+12.61%) | 10.0000 | 8.8800 |
| `single_kernel_aggressive/vopd/variant=independent-fma,n=32768,body=64;hip-wave=32` | 2 | vulkan (+12.95%) | 8.7200 | 7.7200 |
| `single_kernel_aggressive/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=32` | 2 | vulkan (+44.24%) | 16.0400 | 11.1200 |
| `single_kernel_aggressive/vopd/variant=dequant-like,n=32768,body=64;hip-wave=32` | 2 | vulkan (+31.15%) | 13.6400 | 10.4000 |
| `single_kernel_aggressive/sampler/argmax,vocab=131072,rows=1;hip-wave=32` | 2 | vulkan (+0.50%) | 96.2800 | 95.8000 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 94 | 39 | 0 | 70.68 | 0.8485 | 133 |
| RL / hipgraph | 131 | 2 | 0 | 98.50 | 0.2405 | 133 |
| RL / hip | 131 | 2 | 0 | 98.50 | 0.2693 | 133 |

## Prior hipEngine-harness baseline

The old run is directional rather than row-for-row identical: it used a larger parameter sweep, only three contenders, and the previous kernels.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine | RL / vulkan | 186 | 26 | 87.74 | 0.4788 | 212 |
| hipEngine | RL / hip | 157 | 55 | 74.06 | 0.8342 | 212 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 131/133 rows and HipGraph in 131/133 while all three select identical per-row hipcc code objects. This run uses the `targeted_wave64` wave policy; the aggressive single-kernel mode removes dependency fences from Redline's timed tape. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 94/133 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. This artifact records the `targeted_wave64` wave policy as the controlled HIP launch-policy factor. The aggressive Redline timed IB has no dependency fences, but the completion timestamp still necessarily proves the dispatch finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
