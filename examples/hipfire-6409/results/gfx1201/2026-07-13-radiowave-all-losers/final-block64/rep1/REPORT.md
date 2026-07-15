# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 110/133 rows (82.71%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 110 | 23 | 0 | 0 | 82.71 | 133 |
| vulkan | 21 | 89 | 6 | 17 | 15.79 | 133 |
| hipgraph | 1 | 8 | 40 | 84 | 0.75 | 133 |
| hip | 1 | 13 | 87 | 32 | 0.75 | 133 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 29 | 16 | 0 | 0 | 45 |
| serial_latency | vulkan | 14 | 19 | 1 | 11 | 45 |
| serial_latency | hipgraph | 1 | 8 | 32 | 4 | 45 |
| serial_latency | hip | 1 | 2 | 12 | 30 | 45 |
| independent_throughput | redline | 41 | 4 | 0 | 0 | 45 |
| independent_throughput | vulkan | 4 | 41 | 0 | 0 | 45 |
| independent_throughput | hipgraph | 0 | 0 | 2 | 43 | 45 |
| independent_throughput | hip | 0 | 0 | 43 | 2 | 45 |
| single_kernel_aggressive | redline | 40 | 3 | 0 | 0 | 43 |
| single_kernel_aggressive | vulkan | 3 | 29 | 5 | 6 | 43 |
| single_kernel_aggressive | hipgraph | 0 | 0 | 6 | 37 | 43 |
| single_kernel_aggressive | hip | 0 | 11 | 32 | 0 | 43 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/dispatch-grid/count=50,grid=1;hip-wave=32` | 2 | vulkan (+36.77%) | 1.7200 | 1.2576 |
| `serial_latency/dispatch-grid/count=200,grid=1;hip-wave=32` | 2 | vulkan (+46.98%) | 1.5994 | 1.0882 |
| `serial_latency/dispatch-grid/count=941,grid=1;hip-wave=32` | 2 | vulkan (+55.22%) | 1.5730 | 1.0134 |
| `serial_latency/dispatch-grid/count=64,grid=128;hip-wave=32` | 2 | vulkan (+155.22%) | 2.9638 | 1.1612 |
| `serial_latency/dispatch-grid/count=64,grid=1024;hip-wave=32` | 2 | vulkan (+38.22%) | 3.0444 | 2.2025 |
| `serial_latency/reduction/variant=wave,k=8192,rows=1;hip-wave=32` | 2 | hipgraph (+2.11%) | 139.2800 | 136.3959 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=4096,body=16;hip-wave=32` | 2 | vulkan (+19.66%) | 7.4181 | 6.1994 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=16;hip-wave=32` | 2 | vulkan (+28.79%) | 8.9087 | 6.9175 |
| `serial_latency/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+21.17%) | 23.7075 | 19.5663 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=4096,body=16;hip-wave=64` | 2 | vulkan (+10.94%) | 7.0987 | 6.3987 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=16;hip-wave=64` | 2 | vulkan (+24.80%) | 12.7012 | 10.1775 |
| `serial_latency/vopd/variant=independent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+21.82%) | 11.1150 | 9.1237 |
| `serial_latency/vopd/variant=dependent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+18.91%) | 10.9987 | 9.2500 |
| `serial_latency/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+44.73%) | 23.9475 | 16.5463 |
| `serial_latency/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+45.41%) | 21.7100 | 14.9300 |
| `serial_latency/sampler/argmax,vocab=131072,rows=4;hip-wave=32` | 2 | hip (+0.31%) | 158.0700 | 157.5884 |
| `independent_throughput/dispatch-grid/count=941,grid=1;hip-wave=32` | 2 | vulkan (+0.87%) | 0.0840 | 0.0833 |
| `independent_throughput/memory-waitcnt/variant=gather,n=32768,body=16;hip-wave=32` | 2 | vulkan (+25.79%) | 9.4025 | 7.4750 |
| `independent_throughput/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+20.12%) | 8.9262 | 7.4313 |
| `independent_throughput/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+16.61%) | 7.7838 | 6.6750 |
| `single_kernel_aggressive/vopd/variant=independent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+0.52%) | 7.7600 | 7.7200 |
| `single_kernel_aggressive/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+18.28%) | 13.2000 | 11.1600 |
| `single_kernel_aggressive/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+19.77%) | 12.3600 | 10.3200 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 112 | 21 | 0 | 84.21 | 0.7771 | 133 |
| RL / hipgraph | 132 | 1 | 0 | 99.25 | 0.2624 | 133 |
| RL / hip | 132 | 1 | 0 | 99.25 | 0.3462 | 133 |

## Prior hipEngine-harness baseline

The old run is directional rather than row-for-row identical: it used a larger parameter sweep, only three contenders, and the previous kernels.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine | RL / vulkan | 186 | 26 | 87.74 | 0.4788 | 212 |
| hipEngine | RL / hip | 157 | 55 | 74.06 | 0.8342 | 212 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 132/133 rows and HipGraph in 132/133 while all three select identical per-row hipcc code objects. This run uses the `radiowave_tuned` wave policy; the aggressive single-kernel mode removes dependency fences from Redline's timed tape. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 112/133 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. This artifact records the `radiowave_tuned` wave policy as the controlled HIP launch-policy factor. The aggressive Redline timed IB has no dependency fences, but the completion timestamp still necessarily proves the dispatch finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
