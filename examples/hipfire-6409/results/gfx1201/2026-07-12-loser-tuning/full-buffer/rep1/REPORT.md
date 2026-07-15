# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 97/133 rows (72.93%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 97 | 36 | 0 | 0 | 72.93 | 133 |
| vulkan | 35 | 79 | 4 | 15 | 26.32 | 133 |
| hipgraph | 0 | 8 | 44 | 81 | 0.00 | 133 |
| hip | 1 | 10 | 85 | 37 | 0.75 | 133 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 22 | 23 | 0 | 0 | 45 |
| serial_latency | vulkan | 22 | 13 | 1 | 9 | 45 |
| serial_latency | hipgraph | 0 | 8 | 30 | 7 | 45 |
| serial_latency | hip | 1 | 1 | 14 | 29 | 45 |
| independent_throughput | redline | 38 | 7 | 0 | 0 | 45 |
| independent_throughput | vulkan | 7 | 38 | 0 | 0 | 45 |
| independent_throughput | hipgraph | 0 | 0 | 8 | 37 | 45 |
| independent_throughput | hip | 0 | 0 | 37 | 8 | 45 |
| single_kernel_aggressive | redline | 37 | 6 | 0 | 0 | 43 |
| single_kernel_aggressive | vulkan | 6 | 28 | 3 | 6 | 43 |
| single_kernel_aggressive | hipgraph | 0 | 0 | 6 | 37 | 43 |
| single_kernel_aggressive | hip | 0 | 9 | 34 | 0 | 43 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/dispatch-grid/count=50,grid=1;hip-wave=32` | 2 | vulkan (+35.70%) | 1.6848 | 1.2416 |
| `serial_latency/dispatch-grid/count=200,grid=1;hip-wave=32` | 2 | vulkan (+49.13%) | 1.5736 | 1.0552 |
| `serial_latency/dispatch-grid/count=941,grid=1;hip-wave=32` | 2 | vulkan (+55.47%) | 1.5491 | 0.9964 |
| `serial_latency/dispatch-grid/count=64,grid=128;hip-wave=32` | 2 | vulkan (+158.05%) | 2.9869 | 1.1575 |
| `serial_latency/dispatch-grid/count=64,grid=1024;hip-wave=32` | 2 | vulkan (+73.13%) | 3.7850 | 2.1862 |
| `serial_latency/reduction/variant=lds-tree,k=8192,rows=1;hip-wave=32` | 2 | hip (+0.40%) | 135.6469 | 135.1035 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=4096,body=16;hip-wave=32` | 2 | vulkan (+21.76%) | 7.3794 | 6.0606 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+29.03%) | 8.7725 | 6.7988 |
| `serial_latency/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+20.89%) | 22.4025 | 18.5312 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=4096,body=16;hip-wave=32` | 2 | vulkan (+20.42%) | 7.5269 | 6.2506 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+66.39%) | 16.2812 | 9.7850 |
| `serial_latency/packed-dot/variant=q8-signed,n=4096,body=16;hip-wave=32` | 2 | vulkan (+5.04%) | 7.8900 | 7.5113 |
| `serial_latency/packed-dot/variant=q4-unsigned,n=4096,body=16;hip-wave=32` | 2 | vulkan (+4.83%) | 7.8450 | 7.4837 |
| `serial_latency/packed-dot/variant=q6-zero,n=4096,body=16;hip-wave=32` | 2 | vulkan (+1.43%) | 7.9550 | 7.8431 |
| `serial_latency/vopd/variant=independent-fma,n=32768,body=64;hip-wave=32` | 2 | vulkan (+48.27%) | 13.1575 | 8.8737 |
| `serial_latency/vopd/variant=dependent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+23.96%) | 11.1762 | 9.0162 |
| `serial_latency/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=32` | 2 | vulkan (+74.02%) | 28.0287 | 16.1062 |
| `serial_latency/vopd/variant=dequant-like,n=32768,body=64;hip-wave=32` | 2 | vulkan (+62.15%) | 23.5725 | 14.5375 |
| `serial_latency/sampler/argmax,vocab=32768,rows=1;hip-wave=32` | 2 | vulkan (+4.52%) | 43.0831 | 41.2181 |
| `serial_latency/sampler/argmax,vocab=32768,rows=4;hip-wave=32` | 2 | vulkan (+3.99%) | 43.0525 | 41.4025 |
| `serial_latency/sampler/argmax,vocab=131072,rows=1;hip-wave=32` | 2 | vulkan (+2.85%) | 163.2369 | 158.7094 |
| `serial_latency/sampler/argmax,vocab=131072,rows=4;hip-wave=32` | 2 | vulkan (+2.56%) | 164.8144 | 160.7075 |
| `serial_latency/two-stage-reduction/k=32768,rows=4,splits=8;hip-wave=32` | 2 | vulkan (+1.48%) | 10.4437 | 10.2912 |
| `independent_throughput/dispatch-grid/count=64,grid=1024;hip-wave=32` | 2 | vulkan (+55.14%) | 2.2525 | 1.4519 |
| `independent_throughput/dispatch-grid/count=64,grid=8192;hip-wave=32` | 2 | vulkan (+50.82%) | 8.2925 | 5.4981 |
| `independent_throughput/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+22.08%) | 9.3775 | 7.6813 |
| `independent_throughput/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+20.75%) | 5.1787 | 4.2888 |
| `independent_throughput/vopd/variant=independent-fma,n=32768,body=64;hip-wave=32` | 2 | vulkan (+8.51%) | 4.3825 | 4.0388 |
| `independent_throughput/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=32` | 2 | vulkan (+41.08%) | 10.4875 | 7.4337 |
| `independent_throughput/vopd/variant=dequant-like,n=32768,body=64;hip-wave=32` | 2 | vulkan (+21.07%) | 8.0363 | 6.6375 |
| `single_kernel_aggressive/dispatch-grid/count=64,grid=8192;hip-wave=32` | 2 | vulkan (+8.23%) | 10.0000 | 9.2400 |
| `single_kernel_aggressive/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+17.94%) | 10.5200 | 8.9200 |
| `single_kernel_aggressive/vopd/variant=independent-fma,n=32768,body=64;hip-wave=32` | 2 | vulkan (+14.51%) | 8.8400 | 7.7200 |
| `single_kernel_aggressive/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=32` | 2 | vulkan (+39.78%) | 15.6000 | 11.1600 |
| `single_kernel_aggressive/vopd/variant=dequant-like,n=32768,body=64;hip-wave=32` | 2 | vulkan (+33.59%) | 13.8400 | 10.3600 |
| `single_kernel_aggressive/sampler/argmax,vocab=131072,rows=1;hip-wave=32` | 2 | vulkan (+1.63%) | 97.0400 | 95.4800 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 98 | 35 | 0 | 73.68 | 0.8199 | 133 |
| RL / hipgraph | 133 | 0 | 0 | 100.00 | 0.2412 | 133 |
| RL / hip | 132 | 1 | 0 | 99.25 | 0.2670 | 133 |

## Prior hipEngine-harness baseline

The old run is directional rather than row-for-row identical: it used a larger parameter sweep, only three contenders, and the previous kernels.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine | RL / vulkan | 186 | 26 | 87.74 | 0.4788 | 212 |
| hipEngine | RL / hip | 157 | 55 | 74.06 | 0.8342 | 212 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 132/133 rows and HipGraph in 133/133 while all three select identical per-row hipcc code objects. This run uses the `targeted_wave64` wave policy; the aggressive single-kernel mode removes dependency fences from Redline's timed tape. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 98/133 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. This artifact records the `targeted_wave64` wave policy as the controlled HIP launch-policy factor. The aggressive Redline timed IB has no dependency fences, but the completion timestamp still necessarily proves the dispatch finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
