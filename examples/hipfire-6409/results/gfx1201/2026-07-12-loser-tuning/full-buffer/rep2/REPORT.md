# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 96/133 rows (72.18%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 96 | 36 | 1 | 0 | 72.18 | 133 |
| vulkan | 35 | 79 | 4 | 15 | 26.32 | 133 |
| hipgraph | 1 | 9 | 37 | 86 | 0.75 | 133 |
| hip | 1 | 9 | 91 | 32 | 0.75 | 133 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 21 | 23 | 1 | 0 | 45 |
| serial_latency | vulkan | 22 | 13 | 1 | 9 | 45 |
| serial_latency | hipgraph | 1 | 9 | 27 | 8 | 45 |
| serial_latency | hip | 1 | 0 | 16 | 28 | 45 |
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
| `serial_latency/dispatch-grid/count=50,grid=1;hip-wave=32` | 2 | vulkan (+35.22%) | 1.7200 | 1.2720 |
| `serial_latency/dispatch-grid/count=200,grid=1;hip-wave=32` | 2 | vulkan (+49.68%) | 1.6016 | 1.0700 |
| `serial_latency/dispatch-grid/count=941,grid=1;hip-wave=32` | 2 | vulkan (+49.73%) | 1.5712 | 1.0494 |
| `serial_latency/dispatch-grid/count=64,grid=128;hip-wave=32` | 2 | vulkan (+155.16%) | 2.9663 | 1.1625 |
| `serial_latency/dispatch-grid/count=64,grid=1024;hip-wave=32` | 2 | vulkan (+71.35%) | 3.7750 | 2.2031 |
| `serial_latency/reduction/variant=wave,k=8192,rows=1;hip-wave=32` | 2 | hip (+0.57%) | 134.0706 | 133.3124 |
| `serial_latency/reduction/variant=lds-tree,k=8192,rows=1;hip-wave=32` | 2 | hipgraph (+0.73%) | 134.2769 | 133.3092 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=4096,body=16;hip-wave=32` | 2 | vulkan (+23.17%) | 7.3594 | 5.9750 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+29.11%) | 8.6988 | 6.7375 |
| `serial_latency/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+21.79%) | 22.9138 | 18.8137 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=4096,body=16;hip-wave=32` | 2 | vulkan (+22.23%) | 7.5631 | 6.1875 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+63.88%) | 16.0888 | 9.8175 |
| `serial_latency/packed-dot/variant=q8-signed,n=4096,body=16;hip-wave=32` | 2 | vulkan (+2.23%) | 7.6350 | 7.4681 |
| `serial_latency/packed-dot/variant=q4-unsigned,n=4096,body=16;hip-wave=32` | 2 | vulkan (+3.02%) | 7.6375 | 7.4138 |
| `serial_latency/packed-dot/variant=q6-zero,n=4096,body=16;hip-wave=32` | 2 | vulkan (+0.77%) | 7.8487 | 7.7888 |
| `serial_latency/vopd/variant=independent-fma,n=32768,body=64;hip-wave=32` | 2 | vulkan (+48.10%) | 13.0312 | 8.7988 |
| `serial_latency/vopd/variant=dependent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+24.36%) | 11.0700 | 8.9012 |
| `serial_latency/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=32` | 2 | vulkan (+74.33%) | 27.7275 | 15.9050 |
| `serial_latency/vopd/variant=dequant-like,n=32768,body=64;hip-wave=32` | 2 | vulkan (+62.12%) | 23.2913 | 14.3663 |
| `serial_latency/sampler/argmax,vocab=32768,rows=1;hip-wave=32` | 2 | vulkan (+4.39%) | 42.6688 | 40.8756 |
| `serial_latency/sampler/argmax,vocab=32768,rows=4;hip-wave=32` | 2 | vulkan (+4.84%) | 43.0350 | 41.0469 |
| `serial_latency/sampler/argmax,vocab=131072,rows=1;hip-wave=32` | 3 | vulkan (+2.86%), hipgraph (+0.76%) | 161.8388 | 157.3319 |
| `serial_latency/sampler/argmax,vocab=131072,rows=4;hip-wave=32` | 2 | vulkan (+2.53%) | 162.4169 | 158.4019 |
| `serial_latency/two-stage-reduction/k=32768,rows=4,splits=8;hip-wave=32` | 2 | vulkan (+0.55%) | 10.2613 | 10.2050 |
| `independent_throughput/dispatch-grid/count=64,grid=1024;hip-wave=32` | 2 | vulkan (+58.15%) | 2.2556 | 1.4263 |
| `independent_throughput/dispatch-grid/count=64,grid=8192;hip-wave=32` | 2 | vulkan (+50.77%) | 8.2837 | 5.4944 |
| `independent_throughput/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+21.49%) | 9.4187 | 7.7525 |
| `independent_throughput/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+23.62%) | 5.1875 | 4.1963 |
| `independent_throughput/vopd/variant=independent-fma,n=32768,body=64;hip-wave=32` | 2 | vulkan (+8.66%) | 4.4050 | 4.0537 |
| `independent_throughput/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=32` | 2 | vulkan (+41.36%) | 10.5388 | 7.4550 |
| `independent_throughput/vopd/variant=dequant-like,n=32768,body=64;hip-wave=32` | 2 | vulkan (+21.02%) | 8.0375 | 6.6413 |
| `single_kernel_aggressive/dispatch-grid/count=64,grid=8192;hip-wave=32` | 2 | vulkan (+17.83%) | 10.8400 | 9.2000 |
| `single_kernel_aggressive/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+16.67%) | 10.3600 | 8.8800 |
| `single_kernel_aggressive/vopd/variant=independent-fma,n=32768,body=64;hip-wave=32` | 2 | vulkan (+14.51%) | 8.8400 | 7.7200 |
| `single_kernel_aggressive/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=32` | 2 | vulkan (+37.41%) | 15.2800 | 11.1200 |
| `single_kernel_aggressive/vopd/variant=dequant-like,n=32768,body=64;hip-wave=32` | 2 | vulkan (+32.43%) | 13.7200 | 10.3600 |
| `single_kernel_aggressive/sampler/argmax,vocab=131072,rows=1;hip-wave=32` | 2 | vulkan (+0.63%) | 96.0000 | 95.4000 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 98 | 35 | 0 | 73.68 | 0.8193 | 133 |
| RL / hipgraph | 131 | 2 | 0 | 98.50 | 0.2406 | 133 |
| RL / hip | 132 | 1 | 0 | 99.25 | 0.2667 | 133 |

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
