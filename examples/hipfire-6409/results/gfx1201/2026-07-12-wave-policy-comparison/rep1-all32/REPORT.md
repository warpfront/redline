# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 78/133 rows (58.65%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 78 | 53 | 2 | 0 | 58.65 | 133 |
| vulkan | 51 | 62 | 7 | 13 | 38.35 | 133 |
| hipgraph | 3 | 7 | 46 | 77 | 2.26 | 133 |
| hip | 1 | 11 | 78 | 43 | 0.75 | 133 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 13 | 30 | 2 | 0 | 45 |
| serial_latency | vulkan | 28 | 7 | 1 | 9 | 45 |
| serial_latency | hipgraph | 3 | 6 | 30 | 6 | 45 |
| serial_latency | hip | 1 | 2 | 12 | 30 | 45 |
| independent_throughput | redline | 31 | 14 | 0 | 0 | 45 |
| independent_throughput | vulkan | 14 | 31 | 0 | 0 | 45 |
| independent_throughput | hipgraph | 0 | 0 | 12 | 33 | 45 |
| independent_throughput | hip | 0 | 0 | 33 | 12 | 45 |
| single_kernel_aggressive | redline | 34 | 9 | 0 | 0 | 43 |
| single_kernel_aggressive | vulkan | 9 | 24 | 6 | 4 | 43 |
| single_kernel_aggressive | hipgraph | 0 | 1 | 4 | 38 | 43 |
| single_kernel_aggressive | hip | 0 | 9 | 33 | 1 | 43 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/dispatch-grid/count=50,grid=1;hip-wave=32` | 2 | vulkan (+34.33%) | 1.7216 | 1.2816 |
| `serial_latency/dispatch-grid/count=200,grid=1;hip-wave=32` | 2 | vulkan (+47.24%) | 1.6058 | 1.0906 |
| `serial_latency/dispatch-grid/count=941,grid=1;hip-wave=32` | 2 | vulkan (+54.11%) | 1.5815 | 1.0262 |
| `serial_latency/dispatch-grid/count=64,grid=128;hip-wave=32` | 2 | vulkan (+152.80%) | 3.0225 | 1.1956 |
| `serial_latency/dispatch-grid/count=64,grid=1024;hip-wave=32` | 2 | vulkan (+71.53%) | 3.8862 | 2.2656 |
| `serial_latency/reduction/variant=wave,k=8192,rows=1;hip-wave=32` | 2 | hip (+0.47%) | 140.3063 | 139.6481 |
| `serial_latency/reduction/variant=lds-tree,k=8192,rows=1;hip-wave=32` | 2 | hipgraph (+0.91%) | 140.6406 | 139.3763 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=4096,body=16;hip-wave=32` | 2 | vulkan (+21.48%) | 7.5606 | 6.2237 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+28.30%) | 8.9775 | 6.9975 |
| `serial_latency/memory-waitcnt/variant=gather,n=4096,body=16;hip-wave=32` | 2 | vulkan (+11.65%) | 10.4931 | 9.3981 |
| `serial_latency/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+23.02%) | 24.1988 | 19.6713 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=4096,body=16;hip-wave=32` | 2 | vulkan (+20.78%) | 7.7656 | 6.4294 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+62.83%) | 16.7488 | 10.2863 |
| `serial_latency/packed-dot/variant=q8-signed,n=4096,body=16;hip-wave=32` | 2 | vulkan (+27.05%) | 9.8350 | 7.7412 |
| `serial_latency/packed-dot/variant=q4-unsigned,n=4096,body=16;hip-wave=32` | 2 | vulkan (+27.19%) | 9.8306 | 7.7294 |
| `serial_latency/packed-dot/variant=q6-zero,n=4096,body=16;hip-wave=32` | 2 | vulkan (+23.56%) | 10.0031 | 8.0956 |
| `serial_latency/vopd/variant=independent-fma,n=32768,body=64;hip-wave=32` | 2 | vulkan (+48.62%) | 13.6638 | 9.1937 |
| `serial_latency/vopd/variant=dependent-fma,n=32768,body=64;hip-wave=32` | 2 | vulkan (+45.19%) | 13.5538 | 9.3350 |
| `serial_latency/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=32` | 2 | vulkan (+80.45%) | 29.0162 | 16.0800 |
| `serial_latency/vopd/variant=dequant-like,n=32768,body=64;hip-wave=32` | 2 | vulkan (+61.73%) | 24.3663 | 15.0663 |
| `serial_latency/sampler/argmax,vocab=32768,rows=1;hip-wave=32` | 2 | vulkan (+4.45%) | 44.3794 | 42.4894 |
| `serial_latency/sampler/argmax,vocab=32768,rows=4;hip-wave=32` | 2 | vulkan (+4.70%) | 44.7081 | 42.7006 |
| `serial_latency/sampler/argmax,vocab=131072,rows=1;hip-wave=32` | 3 | vulkan (+2.81%), hipgraph (+1.90%) | 168.0719 | 163.4837 |
| `serial_latency/sampler/argmax,vocab=131072,rows=4;hip-wave=32` | 2 | vulkan (+3.76%) | 168.7212 | 162.6019 |
| `serial_latency/two-stage-reduction/k=32768,rows=4,splits=8;hip-wave=32` | 2 | vulkan (+0.09%) | 10.6175 | 10.6075 |
| `serial_latency/q4-selected-dual/m=512,k=768,tile=2;hip-wave=32` | 2 | vulkan (+21.76%) | 7.5113 | 6.1688 |
| `serial_latency/q4-selected-dual/m=2048,k=2048,tile=2;hip-wave=32` | 2 | vulkan (+5.15%) | 14.9175 | 14.1875 |
| `serial_latency/q4-selected-dual/m=4096,k=8192,tile=2;hip-wave=32` | 2 | vulkan (+47.78%) | 91.3300 | 61.8000 |
| `serial_latency/q6-x8-selected-down/m=512,k=2048,tile=8;hip-wave=32` | 2 | hipgraph (+0.92%) | 38.0175 | 37.6700 |
| `serial_latency/q6-x8-selected-down/m=2048,k=4096,tile=8;hip-wave=32` | 3 | hipgraph (+1.38%), hip (+0.11%) | 72.2275 | 71.2437 |
| `serial_latency/dense-q8/m=512,k=768,tile=4;hip-wave=32` | 2 | vulkan (+14.29%) | 9.7300 | 8.5137 |
| `serial_latency/dense-q8/m=2048,k=2048,tile=4;hip-wave=32` | 2 | vulkan (+0.13%) | 19.8150 | 19.7888 |
| `independent_throughput/dispatch-grid/count=64,grid=1024;hip-wave=32` | 2 | vulkan (+56.98%) | 2.2125 | 1.4094 |
| `independent_throughput/dispatch-grid/count=64,grid=8192;hip-wave=32` | 2 | vulkan (+50.87%) | 8.2894 | 5.4944 |
| `independent_throughput/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+23.09%) | 9.4637 | 7.6887 |
| `independent_throughput/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+19.98%) | 5.1650 | 4.3050 |
| `independent_throughput/packed-dot/variant=q8-signed,n=4096,body=16;hip-wave=32` | 2 | vulkan (+4.64%) | 1.5075 | 1.4406 |
| `independent_throughput/packed-dot/variant=q4-unsigned,n=4096,body=16;hip-wave=32` | 2 | vulkan (+0.27%) | 1.4087 | 1.4050 |
| `independent_throughput/vopd/variant=independent-fma,n=32768,body=64;hip-wave=32` | 2 | vulkan (+8.72%) | 4.3963 | 4.0438 |
| `independent_throughput/vopd/variant=dependent-fma,n=32768,body=64;hip-wave=32` | 2 | vulkan (+7.45%) | 4.3288 | 4.0287 |
| `independent_throughput/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=32` | 2 | vulkan (+46.19%) | 10.4925 | 7.1775 |
| `independent_throughput/vopd/variant=dequant-like,n=32768,body=64;hip-wave=32` | 2 | vulkan (+20.99%) | 8.0412 | 6.6463 |
| `independent_throughput/q4-selected-dual/m=2048,k=2048,tile=2;hip-wave=32` | 2 | vulkan (+2.52%) | 3.9100 | 3.8138 |
| `independent_throughput/q4-selected-dual/m=4096,k=8192,tile=2;hip-wave=32` | 2 | vulkan (+0.12%) | 31.3600 | 31.3225 |
| `independent_throughput/q6-x8-selected-down/m=4096,k=8192,tile=8;hip-wave=32` | 2 | vulkan (+14.72%) | 28.6475 | 24.9725 |
| `independent_throughput/dense-q8/m=2048,k=8192,tile=4;hip-wave=32` | 2 | vulkan (+2.65%) | 12.7000 | 12.3725 |
| `single_kernel_aggressive/dispatch-grid/count=64,grid=8192;hip-wave=32` | 2 | vulkan (+17.24%) | 10.8800 | 9.2800 |
| `single_kernel_aggressive/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+3.03%) | 13.6000 | 13.2000 |
| `single_kernel_aggressive/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+16.52%) | 10.4400 | 8.9600 |
| `single_kernel_aggressive/vopd/variant=independent-fma,n=32768,body=64;hip-wave=32` | 2 | vulkan (+6.74%) | 8.2400 | 7.7200 |
| `single_kernel_aggressive/vopd/variant=dependent-fma,n=32768,body=64;hip-wave=32` | 2 | vulkan (+11.28%) | 8.6800 | 7.8000 |
| `single_kernel_aggressive/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=32` | 2 | vulkan (+39.57%) | 15.5200 | 11.1200 |
| `single_kernel_aggressive/vopd/variant=dequant-like,n=32768,body=64;hip-wave=32` | 2 | vulkan (+33.20%) | 13.8000 | 10.3600 |
| `single_kernel_aggressive/sampler/argmax,vocab=131072,rows=4;hip-wave=32` | 2 | vulkan (+0.13%) | 95.4800 | 95.3600 |
| `single_kernel_aggressive/q4-selected-dual/m=4096,k=8192,tile=2;hip-wave=32` | 2 | vulkan (+18.06%) | 51.5200 | 43.6400 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 82 | 51 | 0 | 61.65 | 0.8881 | 133 |
| RL / hipgraph | 129 | 4 | 0 | 96.99 | 0.3067 | 133 |
| RL / hip | 131 | 2 | 0 | 98.50 | 0.3212 | 133 |

## Prior hipEngine-harness baseline

The old run is directional rather than row-for-row identical: it used a larger parameter sweep, only three contenders, and the previous kernels.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine | RL / vulkan | 186 | 26 | 87.74 | 0.4788 | 212 |
| hipEngine | RL / hip | 157 | 55 | 74.06 | 0.8342 | 212 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 131/133 rows and HipGraph in 129/133 while all three select identical per-row hipcc code objects. This run uses the `all_wave32` wave policy; the aggressive single-kernel mode removes dependency fences from Redline's timed tape. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 82/133 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. This artifact records the `all_wave32` wave policy as the controlled HIP launch-policy factor. The aggressive Redline timed IB has no dependency fences, but the completion timestamp still necessarily proves the dispatch finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
