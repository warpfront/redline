# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 3/12 rows (25.00%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 3 | 9 | 0 | 0 | 25.00 | 12 |
| vulkan | 9 | 3 | 0 | 0 | 75.00 | 12 |
| hipgraph | 0 | 0 | 3 | 9 | 0.00 | 12 |
| hip | 0 | 0 | 9 | 3 | 0.00 | 12 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 0 | 4 | 0 | 0 | 4 |
| serial_latency | vulkan | 4 | 0 | 0 | 0 | 4 |
| serial_latency | hipgraph | 0 | 0 | 3 | 1 | 4 |
| serial_latency | hip | 0 | 0 | 1 | 3 | 4 |
| independent_throughput | redline | 1 | 3 | 0 | 0 | 4 |
| independent_throughput | vulkan | 3 | 1 | 0 | 0 | 4 |
| independent_throughput | hipgraph | 0 | 0 | 0 | 4 | 4 |
| independent_throughput | hip | 0 | 0 | 4 | 0 | 4 |
| single_kernel_aggressive | redline | 2 | 2 | 0 | 0 | 4 |
| single_kernel_aggressive | vulkan | 2 | 2 | 0 | 0 | 4 |
| single_kernel_aggressive | hipgraph | 0 | 0 | 0 | 4 | 4 |
| single_kernel_aggressive | hip | 0 | 0 | 4 | 0 | 4 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/vopd/variant=independent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+4.91%) | 9.3012 | 8.8663 |
| `serial_latency/vopd/variant=dependent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+3.52%) | 9.3387 | 9.0213 |
| `serial_latency/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+35.98%) | 21.9250 | 16.1238 |
| `serial_latency/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+19.51%) | 17.3750 | 14.5388 |
| `independent_throughput/vopd/variant=independent-fma,n=32768,body=64;hip-wave=64` | 2 | vulkan (+2.71%) | 7.5687 | 7.3688 |
| `independent_throughput/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+28.46%) | 9.8063 | 7.6338 |
| `independent_throughput/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+4.12%) | 7.3900 | 7.0975 |
| `single_kernel_aggressive/vopd/variant=mixed-int-float,n=32768,body=64;hip-wave=64` | 2 | vulkan (+23.18%) | 14.2400 | 11.5600 |
| `single_kernel_aggressive/vopd/variant=dequant-like,n=32768,body=64;hip-wave=64` | 2 | vulkan (+6.99%) | 11.6400 | 10.8800 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 3 | 9 | 0 | 25.00 | 1.0451 | 12 |
| RL / hipgraph | 12 | 0 | 0 | 100.00 | 0.2237 | 12 |
| RL / hip | 12 | 0 | 0 | 100.00 | 0.2675 | 12 |

## Prior hipEngine-harness baseline

The old run is directional rather than row-for-row identical: it used a larger parameter sweep, only three contenders, and the previous kernels.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine | RL / vulkan | 186 | 26 | 87.74 | 0.4788 | 212 |
| hipEngine | RL / hip | 157 | 55 | 74.06 | 0.8342 | 212 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 12/12 rows and HipGraph in 12/12 while all three select identical per-row hipcc code objects. This run uses the `radiowave_tuned` wave policy; the aggressive single-kernel mode removes dependency fences from Redline's timed tape. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 3/12 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. This artifact records the `radiowave_tuned` wave policy as the controlled HIP launch-policy factor. The aggressive Redline timed IB has no dependency fences, but the completion timestamp still necessarily proves the dispatch finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
