# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 94/133 rows (70.68%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 94 | 38 | 1 | 0 | 70.68 | 133 |
| vulkan | 38 | 77 | 4 | 14 | 28.57 | 133 |
| hipgraph | 1 | 8 | 38 | 86 | 0.75 | 133 |
| hip | 0 | 10 | 90 | 33 | 0.00 | 133 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 22 | 22 | 1 | 0 | 45 |
| serial_latency | vulkan | 22 | 14 | 1 | 8 | 45 |
| serial_latency | hipgraph | 1 | 8 | 26 | 10 | 45 |
| serial_latency | hip | 0 | 1 | 17 | 27 | 45 |
| independent_throughput | redline | 36 | 9 | 0 | 0 | 45 |
| independent_throughput | vulkan | 9 | 36 | 0 | 0 | 45 |
| independent_throughput | hipgraph | 0 | 0 | 6 | 39 | 45 |
| independent_throughput | hip | 0 | 0 | 39 | 6 | 45 |
| single_kernel_aggressive | redline | 36 | 7 | 0 | 0 | 43 |
| single_kernel_aggressive | vulkan | 7 | 27 | 3 | 6 | 43 |
| single_kernel_aggressive | hipgraph | 0 | 0 | 6 | 37 | 43 |
| single_kernel_aggressive | hip | 0 | 9 | 34 | 0 | 43 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/dispatch-grid/count=50,grid=1;hip-wave=32` | 2 | vulkan (+34.55%) | 1.7448 | 1.2968 |
| `serial_latency/dispatch-grid/count=200,grid=1;hip-wave=32` | 2 | vulkan (+49.11%) | 1.6250 | 1.0898 |
| `serial_latency/dispatch-grid/count=941,grid=1;hip-wave=32` | 2 | vulkan (+53.24%) | 1.6117 | 1.0518 |
| `serial_latency/dispatch-grid/count=64,grid=128;hip-wave=32` | 2 | vulkan (+152.49%) | 2.9731 | 1.1775 |
| `serial_latency/dispatch-grid/count=64,grid=1024;hip-wave=32` | 2 | vulkan (+69.48%) | 3.8069 | 2.2462 |
| `serial_latency/reduction/variant=wave,k=8192,rows=1;hip-wave=32` | 2 | hipgraph (+2.16%) | 137.4150 | 134.5087 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=4096,body=16;hip-wave=32` | 2 | vulkan (+20.04%) | 7.3381 | 6.1131 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+29.69%) | 8.8287 | 6.8075 |
| `serial_latency/memory-waitcnt/variant=gather,n=4096,body=16;hip-wave=32` | 2 | vulkan (+11.29%) | 10.3619 | 9.3106 |
| `serial_latency/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+23.48%) | 23.1500 | 18.7475 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=4096,body=16;hip-wave=32` | 2 | vulkan (+19.45%) | 7.5488 | 6.3194 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+62.55%) | 16.0637 | 9.8825 |
| `serial_latency/packed-dot/variant=q8-signed,n=4096,body=16;hip-wave=32` | 2 | vulkan (+27.36%) | 9.5081 | 7.4656 |
| `serial_latency/packed-dot/variant=q4-unsigned,n=4096,body=16;hip-wave=32` | 2 | vulkan (+27.83%) | 9.6869 | 7.5781 |
| `serial_latency/packed-dot/variant=q6-zero,n=4096,body=16;hip-wave=32` | 2 | vulkan (+23.49%) | 9.8287 | 7.9594 |
| `serial_latency/vopd/variant=independent-fma,n=32768,body=64;hip-wave=32` | 2 | vulkan (+47.97%) | 13.3187 | 9.0013 |
| `serial_latency/vopd/variant=dependent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+24.11%) | 11.3250 | 9.1250 |
| `serial_latency/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=32` | 2 | vulkan (+80.33%) | 28.4000 | 15.7487 |
| `serial_latency/vopd/variant=dequant-like,n=32768,body=64;hip-wave=32` | 2 | vulkan (+62.63%) | 23.8788 | 14.6825 |
| `serial_latency/sampler/argmax,vocab=32768,rows=1;hip-wave=32` | 2 | vulkan (+4.42%) | 43.5213 | 41.6775 |
| `serial_latency/sampler/argmax,vocab=32768,rows=4;hip-wave=32` | 2 | vulkan (+4.85%) | 43.9688 | 41.9331 |
| `serial_latency/sampler/argmax,vocab=131072,rows=1;hip-wave=32` | 3 | vulkan (+2.89%), hipgraph (+2.02%) | 165.1656 | 160.5238 |
| `serial_latency/sampler/argmax,vocab=131072,rows=4;hip-wave=32` | 2 | vulkan (+3.38%) | 167.1056 | 161.6419 |
| `independent_throughput/dispatch-grid/count=64,grid=1024;hip-wave=32` | 2 | vulkan (+62.93%) | 2.2912 | 1.4062 |
| `independent_throughput/dispatch-grid/count=64,grid=8192;hip-wave=32` | 2 | vulkan (+51.36%) | 8.3219 | 5.4981 |
| `independent_throughput/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+22.53%) | 9.5038 | 7.7562 |
| `independent_throughput/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+25.33%) | 5.1775 | 4.1312 |
| `independent_throughput/packed-dot/variant=q8-signed,n=4096,body=16;hip-wave=32` | 2 | vulkan (+3.45%) | 1.4238 | 1.3762 |
| `independent_throughput/packed-dot/variant=q4-unsigned,n=4096,body=16;hip-wave=32` | 2 | vulkan (+5.39%) | 1.4669 | 1.3919 |
| `independent_throughput/vopd/variant=independent-fma,n=32768,body=64;hip-wave=32` | 2 | vulkan (+9.14%) | 4.4200 | 4.0500 |
| `independent_throughput/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=32` | 2 | vulkan (+41.47%) | 10.5288 | 7.4425 |
| `independent_throughput/vopd/variant=dequant-like,n=32768,body=64;hip-wave=32` | 2 | vulkan (+21.40%) | 8.0825 | 6.6575 |
| `single_kernel_aggressive/dispatch-grid/count=64,grid=8192;hip-wave=32` | 2 | vulkan (+17.75%) | 10.8800 | 9.2400 |
| `single_kernel_aggressive/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+3.00%) | 13.7200 | 13.3200 |
| `single_kernel_aggressive/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+12.56%) | 10.0400 | 8.9200 |
| `single_kernel_aggressive/vopd/variant=independent-fma,n=32768,body=64;hip-wave=32` | 2 | vulkan (+13.92%) | 8.8400 | 7.7600 |
| `single_kernel_aggressive/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=32` | 2 | vulkan (+38.93%) | 15.5600 | 11.2000 |
| `single_kernel_aggressive/vopd/variant=dequant-like,n=32768,body=64;hip-wave=32` | 2 | vulkan (+31.92%) | 13.7200 | 10.4000 |
| `single_kernel_aggressive/sampler/argmax,vocab=131072,rows=4;hip-wave=32` | 2 | vulkan (+0.34%) | 95.8000 | 95.4800 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 95 | 38 | 0 | 71.43 | 0.8485 | 133 |
| RL / hipgraph | 131 | 2 | 0 | 98.50 | 0.2408 | 133 |
| RL / hip | 133 | 0 | 0 | 100.00 | 0.2643 | 133 |

## Prior hipEngine-harness baseline

The old run is directional rather than row-for-row identical: it used a larger parameter sweep, only three contenders, and the previous kernels.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine | RL / vulkan | 186 | 26 | 87.74 | 0.4788 | 212 |
| hipEngine | RL / hip | 157 | 55 | 74.06 | 0.8342 | 212 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 133/133 rows and HipGraph in 131/133 while all three select identical per-row hipcc code objects. This run uses the `targeted_wave64` wave policy; the aggressive single-kernel mode removes dependency fences from Redline's timed tape. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 95/133 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. This artifact records the `targeted_wave64` wave policy as the controlled HIP launch-policy factor. The aggressive Redline timed IB has no dependency fences, but the completion timestamp still necessarily proves the dispatch finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
