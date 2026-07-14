# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 99/133 rows (74.44%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 99 | 34 | 0 | 0 | 74.44 | 133 |
| vulkan | 34 | 80 | 4 | 15 | 25.56 | 133 |
| hipgraph | 0 | 7 | 40 | 86 | 0.00 | 133 |
| hip | 0 | 12 | 89 | 32 | 0.00 | 133 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 24 | 21 | 0 | 0 | 45 |
| serial_latency | vulkan | 21 | 14 | 1 | 9 | 45 |
| serial_latency | hipgraph | 0 | 7 | 30 | 8 | 45 |
| serial_latency | hip | 0 | 3 | 14 | 28 | 45 |
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
| `serial_latency/dispatch-grid/count=50,grid=1;hip-wave=32` | 2 | vulkan (+33.58%) | 1.7376 | 1.3008 |
| `serial_latency/dispatch-grid/count=200,grid=1;hip-wave=32` | 2 | vulkan (+45.90%) | 1.6072 | 1.1016 |
| `serial_latency/dispatch-grid/count=941,grid=1;hip-wave=32` | 2 | vulkan (+52.05%) | 1.6145 | 1.0618 |
| `serial_latency/dispatch-grid/count=64,grid=128;hip-wave=32` | 2 | vulkan (+152.74%) | 2.9681 | 1.1744 |
| `serial_latency/dispatch-grid/count=64,grid=1024;hip-wave=32` | 2 | vulkan (+69.59%) | 3.7988 | 2.2400 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=4096,body=16;hip-wave=32` | 2 | vulkan (+21.89%) | 7.4650 | 6.1244 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+29.09%) | 8.8313 | 6.8412 |
| `serial_latency/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+21.50%) | 23.4988 | 19.3400 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=4096,body=16;hip-wave=32` | 2 | vulkan (+20.95%) | 7.6519 | 6.3262 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+63.97%) | 16.4937 | 10.0587 |
| `serial_latency/packed-dot/variant=q8-signed,n=4096,body=16;hip-wave=32` | 2 | vulkan (+1.37%) | 7.7169 | 7.6125 |
| `serial_latency/packed-dot/variant=q4-unsigned,n=4096,body=16;hip-wave=32` | 2 | vulkan (+1.17%) | 7.6969 | 7.6081 |
| `serial_latency/packed-dot/variant=q6-zero,n=4096,body=16;hip-wave=32` | 2 | vulkan (+0.68%) | 8.0062 | 7.9519 |
| `serial_latency/vopd/variant=independent-fma,n=32768,body=64;hip-wave=32` | 2 | vulkan (+47.77%) | 13.3438 | 9.0300 |
| `serial_latency/vopd/variant=dependent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+23.87%) | 11.3112 | 9.1312 |
| `serial_latency/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=32` | 2 | vulkan (+74.07%) | 28.4600 | 16.3500 |
| `serial_latency/vopd/variant=dequant-like,n=32768,body=64;hip-wave=32` | 2 | vulkan (+62.01%) | 23.8962 | 14.7500 |
| `serial_latency/sampler/argmax,vocab=32768,rows=1;hip-wave=32` | 2 | vulkan (+4.42%) | 43.6106 | 41.7656 |
| `serial_latency/sampler/argmax,vocab=32768,rows=4;hip-wave=32` | 2 | vulkan (+4.26%) | 43.5412 | 41.7606 |
| `serial_latency/sampler/argmax,vocab=131072,rows=1;hip-wave=32` | 2 | vulkan (+2.92%) | 165.4569 | 160.7644 |
| `serial_latency/sampler/argmax,vocab=131072,rows=4;hip-wave=32` | 2 | vulkan (+3.32%) | 167.6825 | 162.2887 |
| `independent_throughput/dispatch-grid/count=64,grid=1024;hip-wave=32` | 2 | vulkan (+60.92%) | 2.2750 | 1.4138 |
| `independent_throughput/dispatch-grid/count=64,grid=8192;hip-wave=32` | 2 | vulkan (+50.80%) | 8.2819 | 5.4919 |
| `independent_throughput/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+22.64%) | 9.4187 | 7.6800 |
| `independent_throughput/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+20.62%) | 5.1850 | 4.2988 |
| `independent_throughput/vopd/variant=independent-fma,n=32768,body=64;hip-wave=32` | 2 | vulkan (+9.02%) | 4.4112 | 4.0462 |
| `independent_throughput/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=32` | 2 | vulkan (+41.23%) | 10.5538 | 7.4725 |
| `independent_throughput/vopd/variant=dequant-like,n=32768,body=64;hip-wave=32` | 2 | vulkan (+21.03%) | 8.0850 | 6.6800 |
| `single_kernel_aggressive/dispatch-grid/count=64,grid=8192;hip-wave=32` | 2 | vulkan (+8.23%) | 10.0000 | 9.2400 |
| `single_kernel_aggressive/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+16.22%) | 10.3200 | 8.8800 |
| `single_kernel_aggressive/vopd/variant=independent-fma,n=32768,body=64;hip-wave=32` | 2 | vulkan (+9.84%) | 8.4800 | 7.7200 |
| `single_kernel_aggressive/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=32` | 2 | vulkan (+42.29%) | 15.8800 | 11.1600 |
| `single_kernel_aggressive/vopd/variant=dequant-like,n=32768,body=64;hip-wave=32` | 2 | vulkan (+30.38%) | 13.5600 | 10.4000 |
| `single_kernel_aggressive/sampler/argmax,vocab=131072,rows=4;hip-wave=32` | 2 | vulkan (+0.38%) | 95.5200 | 95.1600 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 99 | 34 | 0 | 74.44 | 0.8161 | 133 |
| RL / hipgraph | 133 | 0 | 0 | 100.00 | 0.2403 | 133 |
| RL / hip | 133 | 0 | 0 | 100.00 | 0.2671 | 133 |

## Prior hipEngine-harness baseline

The old run is directional rather than row-for-row identical: it used a larger parameter sweep, only three contenders, and the previous kernels.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine | RL / vulkan | 186 | 26 | 87.74 | 0.4788 | 212 |
| hipEngine | RL / hip | 157 | 55 | 74.06 | 0.8342 | 212 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 133/133 rows and HipGraph in 133/133 while all three select identical per-row hipcc code objects. This run uses the `targeted_wave64` wave policy; the aggressive single-kernel mode removes dependency fences from Redline's timed tape. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 99/133 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. This artifact records the `targeted_wave64` wave policy as the controlled HIP launch-policy factor. The aggressive Redline timed IB has no dependency fences, but the completion timestamp still necessarily proves the dispatch finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
