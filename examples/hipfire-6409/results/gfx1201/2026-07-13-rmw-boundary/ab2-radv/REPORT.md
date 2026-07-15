# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 29/45 rows (64.44%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 29 | 16 | 0 | 0 | 64.44 | 45 |
| vulkan | 14 | 19 | 1 | 11 | 31.11 | 45 |
| hipgraph | 1 | 8 | 31 | 5 | 2.22 | 45 |
| hip | 1 | 2 | 13 | 29 | 2.22 | 45 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 29 | 16 | 0 | 0 | 45 |
| serial_latency | vulkan | 14 | 19 | 1 | 11 | 45 |
| serial_latency | hipgraph | 1 | 8 | 31 | 5 | 45 |
| serial_latency | hip | 1 | 2 | 13 | 29 | 45 |
| independent_throughput | redline | 0 | 0 | 0 | 0 | 0 |
| independent_throughput | vulkan | 0 | 0 | 0 | 0 | 0 |
| independent_throughput | hipgraph | 0 | 0 | 0 | 0 | 0 |
| independent_throughput | hip | 0 | 0 | 0 | 0 | 0 |
| single_kernel_aggressive | redline | 0 | 0 | 0 | 0 | 0 |
| single_kernel_aggressive | vulkan | 0 | 0 | 0 | 0 | 0 |
| single_kernel_aggressive | hipgraph | 0 | 0 | 0 | 0 | 0 |
| single_kernel_aggressive | hip | 0 | 0 | 0 | 0 | 0 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/dispatch-grid/count=50,grid=1;hip-wave=32` | 2 | vulkan (+34.97%) | 1.7416 | 1.2904 |
| `serial_latency/dispatch-grid/count=200,grid=1;hip-wave=32` | 2 | vulkan (+47.27%) | 1.6132 | 1.0954 |
| `serial_latency/dispatch-grid/count=941,grid=1;hip-wave=32` | 2 | vulkan (+53.73%) | 1.5962 | 1.0383 |
| `serial_latency/dispatch-grid/count=64,grid=128;hip-wave=32` | 2 | vulkan (+156.17%) | 2.9700 | 1.1594 |
| `serial_latency/dispatch-grid/count=64,grid=1024;hip-wave=32` | 2 | vulkan (+36.08%) | 3.0337 | 2.2294 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=4096,body=16;hip-wave=32` | 2 | vulkan (+21.45%) | 7.5062 | 6.1806 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+28.70%) | 8.8950 | 6.9112 |
| `serial_latency/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+20.11%) | 23.4175 | 19.4975 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=4096,body=16;hip-wave=64` | 2 | vulkan (+11.46%) | 7.1131 | 6.3819 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=64` | 2 | vulkan (+24.72%) | 12.7225 | 10.2012 |
| `serial_latency/vopd/variant=independent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+20.07%) | 10.9313 | 9.1037 |
| `serial_latency/vopd/variant=dependent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+18.98%) | 10.9625 | 9.2137 |
| `serial_latency/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+49.46%) | 23.8087 | 15.9300 |
| `serial_latency/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+47.05%) | 21.0525 | 14.3163 |
| `serial_latency/sampler/argmax,vocab=131072,rows=1;hip-wave=32` | 2 | hipgraph (+2.05%) | 155.7969 | 152.6652 |
| `serial_latency/sampler/argmax,vocab=131072,rows=4;hip-wave=32` | 2 | hip (+0.53%) | 158.0762 | 157.2408 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 31 | 14 | 0 | 68.89 | 0.9348 | 45 |
| RL / hipgraph | 44 | 1 | 0 | 97.78 | 0.5500 | 45 |
| RL / hip | 44 | 1 | 0 | 97.78 | 0.5498 | 45 |

## Prior hipEngine-harness baseline

The old run is directional rather than row-for-row identical: it used a larger parameter sweep, only three contenders, and the previous kernels.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine | RL / vulkan | 186 | 26 | 87.74 | 0.4788 | 212 |
| hipEngine | RL / hip | 157 | 55 | 74.06 | 0.8342 | 212 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 44/45 rows and HipGraph in 44/45 while all three select identical per-row hipcc code objects. This run uses the `radiowave_tuned` wave policy; the aggressive single-kernel mode removes dependency fences from Redline's timed tape. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 31/45 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. This artifact records the `radiowave_tuned` wave policy as the controlled HIP launch-policy factor. The aggressive Redline timed IB has no dependency fences, but the completion timestamp still necessarily proves the dispatch finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
