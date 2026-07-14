# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 80/133 rows (60.15%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 80 | 51 | 2 | 0 | 60.15 | 133 |
| vulkan | 51 | 62 | 7 | 13 | 38.35 | 133 |
| hipgraph | 2 | 9 | 41 | 81 | 1.50 | 133 |
| hip | 0 | 11 | 83 | 39 | 0.00 | 133 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 16 | 27 | 2 | 0 | 45 |
| serial_latency | vulkan | 27 | 8 | 1 | 9 | 45 |
| serial_latency | hipgraph | 2 | 8 | 30 | 5 | 45 |
| serial_latency | hip | 0 | 2 | 12 | 31 | 45 |
| independent_throughput | redline | 31 | 14 | 0 | 0 | 45 |
| independent_throughput | vulkan | 14 | 31 | 0 | 0 | 45 |
| independent_throughput | hipgraph | 0 | 0 | 7 | 38 | 45 |
| independent_throughput | hip | 0 | 0 | 38 | 7 | 45 |
| single_kernel_aggressive | redline | 33 | 10 | 0 | 0 | 43 |
| single_kernel_aggressive | vulkan | 10 | 23 | 6 | 4 | 43 |
| single_kernel_aggressive | hipgraph | 0 | 1 | 4 | 38 | 43 |
| single_kernel_aggressive | hip | 0 | 9 | 33 | 1 | 43 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/dispatch-grid/count=50,grid=1;hip-wave=32` | 2 | vulkan (+34.25%) | 1.7248 | 1.2848 |
| `serial_latency/dispatch-grid/count=200,grid=1;hip-wave=32` | 2 | vulkan (+48.68%) | 1.6010 | 1.0768 |
| `serial_latency/dispatch-grid/count=941,grid=1;hip-wave=32` | 2 | vulkan (+49.12%) | 1.5716 | 1.0539 |
| `serial_latency/dispatch-grid/count=64,grid=128;hip-wave=32` | 2 | vulkan (+149.37%) | 2.9644 | 1.1887 |
| `serial_latency/dispatch-grid/count=64,grid=1024;hip-wave=32` | 2 | vulkan (+70.36%) | 3.7831 | 2.2206 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=4096,body=16;hip-wave=32` | 2 | vulkan (+22.30%) | 7.4831 | 6.1188 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+28.76%) | 8.8475 | 6.8712 |
| `serial_latency/memory-waitcnt/variant=gather,n=4096,body=16;hip-wave=32` | 2 | vulkan (+11.24%) | 10.3537 | 9.3075 |
| `serial_latency/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+22.30%) | 23.5875 | 19.2862 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=4096,body=16;hip-wave=32` | 2 | vulkan (+21.31%) | 7.6863 | 6.3362 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+63.16%) | 16.4100 | 10.0575 |
| `serial_latency/packed-dot/variant=q8-signed,n=4096,body=16;hip-wave=32` | 2 | vulkan (+26.93%) | 9.6456 | 7.5994 |
| `serial_latency/packed-dot/variant=q4-unsigned,n=4096,body=16;hip-wave=32` | 2 | vulkan (+25.96%) | 9.6163 | 7.6344 |
| `serial_latency/packed-dot/variant=q6-zero,n=4096,body=16;hip-wave=32` | 2 | vulkan (+24.02%) | 9.6544 | 7.7844 |
| `serial_latency/vopd/variant=independent-fma,n=32768,body=64;hip-wave=32` | 2 | vulkan (+48.27%) | 13.3625 | 9.0125 |
| `serial_latency/vopd/variant=dependent-fma,n=32768,body=64;hip-wave=32` | 2 | vulkan (+45.54%) | 13.0712 | 8.9812 |
| `serial_latency/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=32` | 2 | vulkan (+74.50%) | 28.4237 | 16.2888 |
| `serial_latency/vopd/variant=dequant-like,n=32768,body=64;hip-wave=32` | 2 | vulkan (+61.91%) | 23.7913 | 14.6937 |
| `serial_latency/sampler/argmax,vocab=32768,rows=1;hip-wave=32` | 2 | vulkan (+4.50%) | 43.5444 | 41.6713 |
| `serial_latency/sampler/argmax,vocab=32768,rows=4;hip-wave=32` | 2 | vulkan (+5.03%) | 43.7525 | 41.6569 |
| `serial_latency/sampler/argmax,vocab=131072,rows=1;hip-wave=32` | 3 | vulkan (+2.81%), hipgraph (+0.68%) | 165.0769 | 160.5669 |
| `serial_latency/sampler/argmax,vocab=131072,rows=4;hip-wave=32` | 3 | vulkan (+2.88%), hip (+0.04%) | 167.1287 | 162.4475 |
| `serial_latency/q4-selected-dual/m=512,k=768,tile=2;hip-wave=32` | 2 | vulkan (+22.88%) | 7.4112 | 6.0312 |
| `serial_latency/q4-selected-dual/m=2048,k=2048,tile=2;hip-wave=32` | 2 | vulkan (+4.93%) | 14.6525 | 13.9637 |
| `serial_latency/q4-selected-dual/m=4096,k=8192,tile=2;hip-wave=32` | 2 | vulkan (+46.18%) | 88.8750 | 60.7975 |
| `serial_latency/q6-x8-selected-down/m=512,k=2048,tile=8;hip-wave=32` | 2 | hipgraph (+0.76%) | 37.1213 | 36.8414 |
| `serial_latency/q6-x8-selected-down/m=2048,k=4096,tile=8;hip-wave=32` | 2 | hipgraph (+1.15%) | 70.8113 | 70.0078 |
| `serial_latency/dense-q8/m=512,k=768,tile=4;hip-wave=32` | 2 | vulkan (+14.82%) | 9.5612 | 8.3275 |
| `serial_latency/dense-q8/m=2048,k=2048,tile=4;hip-wave=32` | 2 | vulkan (+0.47%) | 19.3025 | 19.2112 |
| `independent_throughput/dispatch-grid/count=64,grid=1024;hip-wave=32` | 2 | vulkan (+59.15%) | 2.2669 | 1.4244 |
| `independent_throughput/dispatch-grid/count=64,grid=8192;hip-wave=32` | 2 | vulkan (+50.63%) | 8.3112 | 5.5175 |
| `independent_throughput/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+22.35%) | 9.5237 | 7.7838 |
| `independent_throughput/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+21.24%) | 5.2237 | 4.3087 |
| `independent_throughput/packed-dot/variant=q8-signed,n=4096,body=16;hip-wave=32` | 2 | vulkan (+33.19%) | 1.5200 | 1.1413 |
| `independent_throughput/packed-dot/variant=q4-unsigned,n=4096,body=16;hip-wave=32` | 2 | vulkan (+5.11%) | 1.4912 | 1.4187 |
| `independent_throughput/vopd/variant=independent-fma,n=32768,body=64;hip-wave=32` | 2 | vulkan (+8.94%) | 4.4500 | 4.0850 |
| `independent_throughput/vopd/variant=dependent-fma,n=32768,body=64;hip-wave=32` | 2 | vulkan (+7.70%) | 4.3513 | 4.0400 |
| `independent_throughput/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=32` | 2 | vulkan (+40.96%) | 10.5775 | 7.5038 |
| `independent_throughput/vopd/variant=dequant-like,n=32768,body=64;hip-wave=32` | 2 | vulkan (+21.08%) | 8.0988 | 6.6887 |
| `independent_throughput/q4-selected-dual/m=2048,k=2048,tile=2;hip-wave=32` | 2 | vulkan (+13.50%) | 3.9000 | 3.4362 |
| `independent_throughput/q4-selected-dual/m=4096,k=8192,tile=2;hip-wave=32` | 2 | vulkan (+0.33%) | 31.4175 | 31.3150 |
| `independent_throughput/q6-x8-selected-down/m=4096,k=8192,tile=8;hip-wave=32` | 2 | vulkan (+14.71%) | 28.8225 | 25.1275 |
| `independent_throughput/dense-q8/m=2048,k=8192,tile=4;hip-wave=32` | 2 | vulkan (+2.57%) | 12.7700 | 12.4500 |
| `single_kernel_aggressive/dispatch-grid/count=64,grid=8192;hip-wave=32` | 2 | vulkan (+16.02%) | 10.7200 | 9.2400 |
| `single_kernel_aggressive/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+1.50%) | 13.5200 | 13.3200 |
| `single_kernel_aggressive/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+13.96%) | 10.1200 | 8.8800 |
| `single_kernel_aggressive/vopd/variant=independent-fma,n=32768,body=64;hip-wave=32` | 2 | vulkan (+10.82%) | 8.6000 | 7.7600 |
| `single_kernel_aggressive/vopd/variant=dependent-fma,n=32768,body=64;hip-wave=32` | 2 | vulkan (+10.71%) | 8.6800 | 7.8400 |
| `single_kernel_aggressive/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=32` | 2 | vulkan (+40.86%) | 15.7200 | 11.1600 |
| `single_kernel_aggressive/vopd/variant=dequant-like,n=32768,body=64;hip-wave=32` | 2 | vulkan (+33.33%) | 13.9200 | 10.4400 |
| `single_kernel_aggressive/sampler/argmax,vocab=131072,rows=1;hip-wave=32` | 2 | vulkan (+0.42%) | 96.1600 | 95.7600 |
| `single_kernel_aggressive/sampler/argmax,vocab=131072,rows=4;hip-wave=32` | 2 | vulkan (+0.54%) | 96.2800 | 95.7600 |
| `single_kernel_aggressive/q4-selected-dual/m=4096,k=8192,tile=2;hip-wave=32` | 2 | vulkan (+18.68%) | 51.8400 | 43.6800 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 82 | 51 | 0 | 61.65 | 0.8945 | 133 |
| RL / hipgraph | 130 | 3 | 0 | 97.74 | 0.3073 | 133 |
| RL / hip | 132 | 1 | 0 | 99.25 | 0.3278 | 133 |

## Prior hipEngine-harness baseline

The old run is directional rather than row-for-row identical: it used a larger parameter sweep, only three contenders, and the previous kernels.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine | RL / vulkan | 186 | 26 | 87.74 | 0.4788 | 212 |
| hipEngine | RL / hip | 157 | 55 | 74.06 | 0.8342 | 212 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 132/133 rows and HipGraph in 130/133 while all three select identical per-row hipcc code objects. This run uses the `all_wave32` wave policy; the aggressive single-kernel mode removes dependency fences from Redline's timed tape. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 82/133 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. This artifact records the `all_wave32` wave policy as the controlled HIP launch-policy factor. The aggressive Redline timed IB has no dependency fences, but the completion timestamp still necessarily proves the dispatch finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
