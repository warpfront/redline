# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 92/133 rows (69.17%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 92 | 40 | 1 | 0 | 69.17 | 133 |
| vulkan | 40 | 75 | 4 | 14 | 30.08 | 133 |
| hipgraph | 1 | 7 | 39 | 86 | 0.75 | 133 |
| hip | 0 | 11 | 89 | 33 | 0.00 | 133 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 21 | 23 | 1 | 0 | 45 |
| serial_latency | vulkan | 23 | 13 | 1 | 8 | 45 |
| serial_latency | hipgraph | 1 | 7 | 30 | 7 | 45 |
| serial_latency | hip | 0 | 2 | 13 | 30 | 45 |
| independent_throughput | redline | 35 | 10 | 0 | 0 | 45 |
| independent_throughput | vulkan | 10 | 35 | 0 | 0 | 45 |
| independent_throughput | hipgraph | 0 | 0 | 3 | 42 | 45 |
| independent_throughput | hip | 0 | 0 | 42 | 3 | 45 |
| single_kernel_aggressive | redline | 36 | 7 | 0 | 0 | 43 |
| single_kernel_aggressive | vulkan | 7 | 27 | 3 | 6 | 43 |
| single_kernel_aggressive | hipgraph | 0 | 0 | 6 | 37 | 43 |
| single_kernel_aggressive | hip | 0 | 9 | 34 | 0 | 43 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/dispatch-grid/count=50,grid=1;hip-wave=64` | 2 | vulkan (+34.23%) | 1.7632 | 1.3136 |
| `serial_latency/dispatch-grid/count=200,grid=1;hip-wave=64` | 2 | vulkan (+51.37%) | 1.6336 | 1.0792 |
| `serial_latency/dispatch-grid/count=941,grid=1;hip-wave=64` | 2 | vulkan (+51.86%) | 1.6236 | 1.0692 |
| `serial_latency/dispatch-grid/count=64,grid=128;hip-wave=64` | 2 | vulkan (+161.89%) | 3.0019 | 1.1462 |
| `serial_latency/dispatch-grid/count=64,grid=1024;hip-wave=64` | 2 | vulkan (+56.84%) | 3.5406 | 2.2575 |
| `serial_latency/reduction/variant=wave,k=8192,rows=1;hip-wave=32` | 2 | hipgraph (+2.15%) | 138.1369 | 135.2291 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=4096,body=16;hip-wave=64` | 2 | vulkan (+26.36%) | 7.7713 | 6.1500 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=16;hip-wave=64` | 2 | vulkan (+32.01%) | 9.1137 | 6.9037 |
| `serial_latency/memory-waitcnt/variant=gather,n=4096,body=16;hip-wave=64` | 2 | vulkan (+16.31%) | 10.7981 | 9.2837 |
| `serial_latency/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=64` | 2 | vulkan (+22.96%) | 23.1575 | 18.8337 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=4096,body=16;hip-wave=64` | 2 | vulkan (+17.88%) | 7.4837 | 6.3487 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=64` | 2 | vulkan (+53.69%) | 15.2750 | 9.9388 |
| `serial_latency/packed-dot/variant=q8-signed,n=4096,body=16;hip-wave=64` | 2 | vulkan (+33.86%) | 10.1125 | 7.5544 |
| `serial_latency/packed-dot/variant=q4-unsigned,n=4096,body=16;hip-wave=64` | 2 | vulkan (+32.45%) | 10.0875 | 7.6162 |
| `serial_latency/packed-dot/variant=q6-zero,n=4096,body=16;hip-wave=64` | 2 | vulkan (+32.67%) | 10.5787 | 7.9737 |
| `serial_latency/vopd/variant=independent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+45.07%) | 12.7625 | 8.7975 |
| `serial_latency/vopd/variant=dependent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+24.46%) | 11.3863 | 9.1487 |
| `serial_latency/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+56.23%) | 25.5675 | 16.3650 |
| `serial_latency/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+49.33%) | 22.0675 | 14.7775 |
| `serial_latency/sampler/argmax,vocab=32768,rows=1;hip-wave=64` | 2 | vulkan (+5.23%) | 44.0694 | 41.8781 |
| `serial_latency/sampler/argmax,vocab=32768,rows=4;hip-wave=64` | 2 | vulkan (+5.24%) | 44.4363 | 42.2250 |
| `serial_latency/sampler/argmax,vocab=131072,rows=1;hip-wave=64` | 3 | vulkan (+3.26%), hipgraph (+0.40%) | 166.4606 | 161.2031 |
| `serial_latency/sampler/argmax,vocab=131072,rows=4;hip-wave=64` | 2 | vulkan (+3.50%) | 167.8281 | 162.1556 |
| `serial_latency/two-stage-reduction/k=32768,rows=4,splits=8;hip-wave=64` | 2 | vulkan (+2.57%) | 10.7937 | 10.5237 |
| `independent_throughput/dispatch-grid/count=64,grid=1024;hip-wave=64` | 2 | vulkan (+67.89%) | 2.4344 | 1.4500 |
| `independent_throughput/dispatch-grid/count=64,grid=8192;hip-wave=64` | 2 | vulkan (+46.68%) | 8.1225 | 5.5375 |
| `independent_throughput/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=64` | 2 | vulkan (+21.74%) | 9.4362 | 7.7512 |
| `independent_throughput/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=64` | 2 | vulkan (+16.89%) | 5.0600 | 4.3288 |
| `independent_throughput/packed-dot/variant=q8-signed,n=4096,body=16;hip-wave=64` | 2 | vulkan (+68.41%) | 2.3388 | 1.3887 |
| `independent_throughput/packed-dot/variant=q4-unsigned,n=4096,body=16;hip-wave=64` | 2 | vulkan (+63.87%) | 2.3044 | 1.4062 |
| `independent_throughput/packed-dot/variant=q6-zero,n=4096,body=16;hip-wave=64` | 2 | vulkan (+55.98%) | 2.2237 | 1.4256 |
| `independent_throughput/vopd/variant=independent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+2.49%) | 4.1650 | 4.0637 |
| `independent_throughput/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+26.55%) | 9.4563 | 7.4725 |
| `independent_throughput/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+18.68%) | 7.8863 | 6.6450 |
| `single_kernel_aggressive/dispatch-grid/count=64,grid=8192;hip-wave=64` | 2 | vulkan (+8.62%) | 10.0800 | 9.2800 |
| `single_kernel_aggressive/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=64` | 2 | vulkan (+0.90%) | 13.4000 | 13.2800 |
| `single_kernel_aggressive/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=64` | 2 | vulkan (+7.11%) | 9.6400 | 9.0000 |
| `single_kernel_aggressive/vopd/variant=independent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+7.73%) | 8.3600 | 7.7600 |
| `single_kernel_aggressive/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+26.52%) | 14.1200 | 11.1600 |
| `single_kernel_aggressive/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+19.54%) | 12.4800 | 10.4400 |
| `single_kernel_aggressive/sampler/argmax,vocab=131072,rows=4;hip-wave=64` | 2 | vulkan (+0.46%) | 95.9600 | 95.5200 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 93 | 40 | 0 | 69.92 | 0.8480 | 133 |
| RL / hipgraph | 131 | 2 | 0 | 98.50 | 0.2632 | 133 |
| RL / hip | 133 | 0 | 0 | 100.00 | 0.3408 | 133 |

## Prior hipEngine-harness baseline

The old run is directional rather than row-for-row identical: it used a larger parameter sweep, only three contenders, and the previous kernels.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine | RL / vulkan | 186 | 26 | 87.74 | 0.4788 | 212 |
| hipEngine | RL / hip | 157 | 55 | 74.06 | 0.8342 | 212 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 133/133 rows and HipGraph in 131/133 while all three select identical per-row hipcc code objects. This run uses the `blanket_wave64` wave policy; the aggressive single-kernel mode removes dependency fences from Redline's timed tape. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 93/133 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. This artifact records the `blanket_wave64` wave policy as the controlled HIP launch-policy factor. The aggressive Redline timed IB has no dependency fences, but the completion timestamp still necessarily proves the dispatch finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
