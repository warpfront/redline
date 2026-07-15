# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 92/133 rows (69.17%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 92 | 41 | 0 | 0 | 69.17 | 133 |
| vulkan | 40 | 75 | 4 | 14 | 30.08 | 133 |
| hipgraph | 1 | 7 | 42 | 83 | 0.75 | 133 |
| hip | 0 | 10 | 87 | 36 | 0.00 | 133 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 21 | 24 | 0 | 0 | 45 |
| serial_latency | vulkan | 23 | 13 | 1 | 8 | 45 |
| serial_latency | hipgraph | 1 | 7 | 28 | 9 | 45 |
| serial_latency | hip | 0 | 1 | 16 | 28 | 45 |
| independent_throughput | redline | 35 | 10 | 0 | 0 | 45 |
| independent_throughput | vulkan | 10 | 35 | 0 | 0 | 45 |
| independent_throughput | hipgraph | 0 | 0 | 8 | 37 | 45 |
| independent_throughput | hip | 0 | 0 | 37 | 8 | 45 |
| single_kernel_aggressive | redline | 36 | 7 | 0 | 0 | 43 |
| single_kernel_aggressive | vulkan | 7 | 27 | 3 | 6 | 43 |
| single_kernel_aggressive | hipgraph | 0 | 0 | 6 | 37 | 43 |
| single_kernel_aggressive | hip | 0 | 9 | 34 | 0 | 43 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/dispatch-grid/count=50,grid=1;hip-wave=64` | 2 | vulkan (+32.89%) | 1.7488 | 1.3160 |
| `serial_latency/dispatch-grid/count=200,grid=1;hip-wave=64` | 2 | vulkan (+48.80%) | 1.6344 | 1.0984 |
| `serial_latency/dispatch-grid/count=941,grid=1;hip-wave=64` | 2 | vulkan (+55.21%) | 1.6199 | 1.0437 |
| `serial_latency/dispatch-grid/count=64,grid=128;hip-wave=64` | 2 | vulkan (+156.88%) | 3.0006 | 1.1681 |
| `serial_latency/dispatch-grid/count=64,grid=1024;hip-wave=64` | 2 | vulkan (+59.19%) | 3.5281 | 2.2163 |
| `serial_latency/reduction/variant=wave,k=8192,rows=1;hip-wave=32` | 2 | hipgraph (+2.10%) | 137.3644 | 134.5386 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=4096,body=16;hip-wave=64` | 2 | vulkan (+25.88%) | 7.7069 | 6.1225 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=16;hip-wave=64` | 2 | vulkan (+32.94%) | 9.0862 | 6.8350 |
| `serial_latency/memory-waitcnt/variant=gather,n=4096,body=16;hip-wave=64` | 2 | vulkan (+16.78%) | 10.7394 | 9.1962 |
| `serial_latency/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=64` | 2 | vulkan (+22.38%) | 23.6737 | 19.3450 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=4096,body=16;hip-wave=64` | 2 | vulkan (+18.98%) | 7.5144 | 6.3156 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=64` | 2 | vulkan (+55.56%) | 15.6625 | 10.0687 |
| `serial_latency/packed-dot/variant=q8-signed,n=4096,body=16;hip-wave=64` | 2 | vulkan (+33.04%) | 10.0994 | 7.5912 |
| `serial_latency/packed-dot/variant=q4-unsigned,n=4096,body=16;hip-wave=64` | 2 | vulkan (+32.66%) | 10.0900 | 7.6056 |
| `serial_latency/packed-dot/variant=q6-zero,n=4096,body=16;hip-wave=64` | 2 | vulkan (+33.02%) | 10.5381 | 7.9225 |
| `serial_latency/vopd/variant=independent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+42.07%) | 12.7650 | 8.9850 |
| `serial_latency/vopd/variant=dependent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+24.18%) | 11.3225 | 9.1175 |
| `serial_latency/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+56.07%) | 25.4725 | 16.3212 |
| `serial_latency/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+49.47%) | 22.0138 | 14.7275 |
| `serial_latency/sampler/argmax,vocab=32768,rows=1;hip-wave=64` | 2 | vulkan (+5.39%) | 43.9038 | 41.6587 |
| `serial_latency/sampler/argmax,vocab=32768,rows=4;hip-wave=64` | 2 | vulkan (+5.36%) | 44.1044 | 41.8600 |
| `serial_latency/sampler/argmax,vocab=131072,rows=1;hip-wave=64` | 2 | vulkan (+3.21%) | 165.6194 | 160.4750 |
| `serial_latency/sampler/argmax,vocab=131072,rows=4;hip-wave=64` | 2 | vulkan (+3.49%) | 166.9506 | 161.3212 |
| `serial_latency/two-stage-reduction/k=32768,rows=4,splits=8;hip-wave=64` | 2 | vulkan (+3.46%) | 10.8275 | 10.4650 |
| `independent_throughput/dispatch-grid/count=64,grid=1024;hip-wave=64` | 2 | vulkan (+64.22%) | 2.3606 | 1.4375 |
| `independent_throughput/dispatch-grid/count=64,grid=8192;hip-wave=64` | 2 | vulkan (+47.33%) | 8.1306 | 5.5187 |
| `independent_throughput/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=64` | 2 | vulkan (+24.00%) | 9.4475 | 7.6188 |
| `independent_throughput/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=64` | 2 | vulkan (+21.91%) | 5.0637 | 4.1537 |
| `independent_throughput/packed-dot/variant=q8-signed,n=4096,body=16;hip-wave=64` | 2 | vulkan (+73.18%) | 2.3250 | 1.3425 |
| `independent_throughput/packed-dot/variant=q4-unsigned,n=4096,body=16;hip-wave=64` | 2 | vulkan (+67.50%) | 2.3125 | 1.3806 |
| `independent_throughput/packed-dot/variant=q6-zero,n=4096,body=16;hip-wave=64` | 2 | vulkan (+57.77%) | 2.2412 | 1.4206 |
| `independent_throughput/vopd/variant=independent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+2.76%) | 4.1900 | 4.0775 |
| `independent_throughput/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+27.37%) | 9.5000 | 7.4588 |
| `independent_throughput/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+18.32%) | 7.8950 | 6.6725 |
| `single_kernel_aggressive/dispatch-grid/count=64,grid=8192;hip-wave=64` | 2 | vulkan (+9.48%) | 10.1600 | 9.2800 |
| `single_kernel_aggressive/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=64` | 2 | vulkan (+1.81%) | 13.5200 | 13.2800 |
| `single_kernel_aggressive/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=64` | 2 | vulkan (+6.67%) | 9.6000 | 9.0000 |
| `single_kernel_aggressive/vopd/variant=independent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+6.70%) | 8.2800 | 7.7600 |
| `single_kernel_aggressive/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+27.60%) | 14.2400 | 11.1600 |
| `single_kernel_aggressive/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+20.77%) | 12.5600 | 10.4000 |
| `single_kernel_aggressive/sampler/argmax,vocab=131072,rows=4;hip-wave=64` | 2 | vulkan (+0.42%) | 96.3200 | 95.9200 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 93 | 40 | 0 | 69.92 | 0.8527 | 133 |
| RL / hipgraph | 132 | 1 | 0 | 99.25 | 0.2553 | 133 |
| RL / hip | 133 | 0 | 0 | 100.00 | 0.3432 | 133 |

## Prior hipEngine-harness baseline

The old run is directional rather than row-for-row identical: it used a larger parameter sweep, only three contenders, and the previous kernels.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine | RL / vulkan | 186 | 26 | 87.74 | 0.4788 | 212 |
| hipEngine | RL / hip | 157 | 55 | 74.06 | 0.8342 | 212 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 133/133 rows and HipGraph in 132/133 while all three select identical per-row hipcc code objects. This run uses the `blanket_wave64` wave policy; the aggressive single-kernel mode removes dependency fences from Redline's timed tape. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 93/133 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. This artifact records the `blanket_wave64` wave policy as the controlled HIP launch-policy factor. The aggressive Redline timed IB has no dependency fences, but the completion timestamp still necessarily proves the dispatch finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
