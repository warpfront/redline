# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 2/12 rows (16.67%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 2 | 10 | 0 | 0 | 16.67 | 12 |
| vulkan | 10 | 2 | 0 | 0 | 83.33 | 12 |
| hipgraph | 0 | 0 | 4 | 8 | 0.00 | 12 |
| hip | 0 | 0 | 8 | 4 | 0.00 | 12 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 0 | 4 | 0 | 0 | 4 |
| serial_latency | vulkan | 4 | 0 | 0 | 0 | 4 |
| serial_latency | hipgraph | 0 | 0 | 4 | 0 | 4 |
| serial_latency | hip | 0 | 0 | 0 | 4 | 4 |
| independent_throughput | redline | 1 | 3 | 0 | 0 | 4 |
| independent_throughput | vulkan | 3 | 1 | 0 | 0 | 4 |
| independent_throughput | hipgraph | 0 | 0 | 0 | 4 | 4 |
| independent_throughput | hip | 0 | 0 | 4 | 0 | 4 |
| single_kernel_aggressive | redline | 1 | 3 | 0 | 0 | 4 |
| single_kernel_aggressive | vulkan | 3 | 1 | 0 | 0 | 4 |
| single_kernel_aggressive | hipgraph | 0 | 0 | 0 | 4 | 4 |
| single_kernel_aggressive | hip | 0 | 0 | 4 | 0 | 4 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/vopd/variant=independent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+41.47%) | 12.8350 | 9.0725 |
| `serial_latency/vopd/variant=dependent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+27.68%) | 11.7562 | 9.2075 |
| `serial_latency/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+62.48%) | 26.8238 | 16.5087 |
| `serial_latency/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+54.59%) | 22.9850 | 14.8688 |
| `independent_throughput/vopd/variant=independent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+14.51%) | 8.6312 | 7.5375 |
| `independent_throughput/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+34.98%) | 9.8875 | 7.3250 |
| `independent_throughput/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+29.87%) | 8.4575 | 6.5125 |
| `single_kernel_aggressive/vopd/variant=independent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+8.29%) | 8.8800 | 8.2000 |
| `single_kernel_aggressive/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+32.53%) | 15.4800 | 11.6800 |
| `single_kernel_aggressive/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+20.80%) | 13.2400 | 10.9600 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 2 | 10 | 0 | 16.67 | 1.2877 | 12 |
| RL / hipgraph | 12 | 0 | 0 | 100.00 | 0.2547 | 12 |
| RL / hip | 12 | 0 | 0 | 100.00 | 0.2984 | 12 |

## Prior hipEngine-harness baseline

The old run is directional rather than row-for-row identical: it used a larger parameter sweep, only three contenders, and the previous kernels.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine | RL / vulkan | 186 | 26 | 87.74 | 0.4788 | 212 |
| hipEngine | RL / hip | 157 | 55 | 74.06 | 0.8342 | 212 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 12/12 rows and HipGraph in 12/12 while all three select identical per-row hipcc code objects. This run uses the `radiowave_tuned` wave policy; the aggressive single-kernel mode removes dependency fences from Redline's timed tape. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 2/12 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. This artifact records the `radiowave_tuned` wave policy as the controlled HIP launch-policy factor. The aggressive Redline timed IB has no dependency fences, but the completion timestamp still necessarily proves the dispatch finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
