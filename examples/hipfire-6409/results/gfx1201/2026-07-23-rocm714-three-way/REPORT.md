# Hipfire/Redline ROCm issue 6409 backend comparison

Correctness-gated result: **Redline wins 187/240 rows (77.92%)**.

## Placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 187 | 35 | 18 | 0 | 77.92 | 240 |
| hip | 4 | 60 | 176 | 0 | 1.67 | 240 |
| vulkan | 49 | 145 | 46 | 0 | 20.42 | 240 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 86 | 18 | 16 | 0 | 120 |
| serial_latency | hip | 3 | 43 | 74 | 0 | 120 |
| serial_latency | vulkan | 31 | 59 | 30 | 0 | 120 |
| independent_throughput | redline | 101 | 17 | 2 | 0 | 120 |
| independent_throughput | hip | 1 | 17 | 102 | 0 | 120 |
| independent_throughput | vulkan | 18 | 86 | 16 | 0 | 120 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=1024;hip-wave=32` | 2 | vulkan (+2.49%) | 0.8387 | 0.8184 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+2.06%) | 18.2680 | 17.9000 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+2.50%) | 18.3600 | 17.9120 |
| `serial_latency/memory-waitcnt/variant=strided4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+3.69%) | 18.5360 | 17.8760 |
| `serial_latency/memory-waitcnt/variant=strided4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+0.02%) | 17.8880 | 17.8840 |
| `serial_latency/memory-waitcnt/variant=gather1,n=32768,body=64,wg=64;hip-wave=32` | 3 | vulkan (+16.63%), hip (+8.43%) | 42.7480 | 36.6520 |
| `serial_latency/memory-waitcnt/variant=gather1,n=32768,body=64,wg=256;hip-wave=32` | 3 | vulkan (+21.70%), hip (+13.62%) | 44.4560 | 36.5280 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=64;hip-wave=64` | 2 | vulkan (+1.51%) | 18.3080 | 18.0360 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=256;hip-wave=64` | 2 | vulkan (+1.08%) | 18.2880 | 18.0920 |
| `serial_latency/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 3 | vulkan (+3.03%), hip (+2.43%) | 442.0480 | 429.0280 |
| `serial_latency/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 3 | vulkan (+2.03%), hip (+1.58%) | 433.9600 | 425.3120 |
| `serial_latency/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 3 | vulkan (+4.17%), hip (+2.45%) | 442.2360 | 424.5360 |
| `serial_latency/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 3 | vulkan (+0.92%), hip (+0.52%) | 433.9000 | 429.9360 |
| `serial_latency/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 3 | vulkan (+4.05%), hip (+3.39%) | 441.6240 | 424.4160 |
| `serial_latency/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 3 | vulkan (+1.62%), hip (+0.10%) | 432.1680 | 425.2920 |
| `serial_latency/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 3 | vulkan (+2.55%), hip (+2.21%) | 441.3440 | 430.3520 |
| `serial_latency/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 3 | vulkan (+1.43%), hip (+1.16%) | 432.3680 | 426.2600 |
| `serial_latency/vopd/variant=independent_fma,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 3 | hip (+21.31%), vulkan (+11.31%) | 62.5280 | 51.5443 |
| `serial_latency/vopd/variant=independent_fma,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 2 | vulkan (+2.82%) | 58.1040 | 56.5120 |
| `serial_latency/vopd/variant=dependent_fma,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 3 | hip (+19.02%), vulkan (+9.97%) | 62.5720 | 52.5724 |
| `serial_latency/vopd/variant=dependent_fma,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 2 | vulkan (+1.58%) | 57.7800 | 56.8840 |
| `serial_latency/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 3 | vulkan (+39.52%), hip (+27.52%) | 155.4200 | 111.4000 |
| `serial_latency/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 3 | vulkan (+28.26%), hip (+9.77%) | 143.1400 | 111.6000 |
| `serial_latency/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 3 | hip (+26.06%), vulkan (+21.29%) | 120.0800 | 95.2566 |
| `serial_latency/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 3 | vulkan (+11.72%), hip (+6.55%) | 110.7280 | 99.1120 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=64` | 2 | vulkan (+34.34%) | 59.0360 | 43.9440 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+99.74%) | 92.8640 | 46.4920 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=64` | 2 | vulkan (+44.17%) | 63.8480 | 44.2880 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+134.86%) | 111.3320 | 47.4040 |
| `serial_latency/q6-x8-selected-down/operation=x8_selected_dp4a_dot_prequantized,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=64` | 2 | vulkan (+104.71%) | 22.7560 | 11.1160 |
| `serial_latency/q6-x8-selected-down/operation=x8_selected_dp4a_quantize_plus_dot,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+64.10%) | 22.6720 | 13.8160 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+14.43%) | 17.9520 | 15.6880 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+22.57%) | 22.8280 | 18.6240 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+11.28%) | 23.7520 | 21.3440 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=128;hip-wave=32` | 2 | vulkan (+44.90%) | 0.1709 | 0.1180 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=1024;hip-wave=32` | 2 | vulkan (+133.41%) | 0.7519 | 0.3221 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=8192;hip-wave=32` | 2 | vulkan (+59.71%) | 3.0405 | 1.9038 |
| `independent_throughput/memory-waitcnt/variant=gather1,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+40.32%) | 38.7000 | 27.5800 |
| `independent_throughput/memory-waitcnt/variant=gather1,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+40.24%) | 39.0080 | 27.8160 |
| `independent_throughput/vopd/variant=independent_fma,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 2 | vulkan (+0.67%) | 57.3720 | 56.9920 |
| `independent_throughput/vopd/variant=independent_fma,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 2 | vulkan (+0.61%) | 56.9360 | 56.5920 |
| `independent_throughput/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 3 | vulkan (+25.83%), hip (+17.88%) | 138.1960 | 109.8280 |
| `independent_throughput/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 2 | vulkan (+24.90%) | 138.2520 | 110.6920 |
| `independent_throughput/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 3 | hip (+13.90%), vulkan (+6.85%) | 104.4480 | 91.7046 |
| `independent_throughput/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 2 | vulkan (+6.96%) | 104.4880 | 97.6920 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=64` | 2 | vulkan (+16.32%) | 47.1640 | 40.5480 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+102.55%) | 84.9480 | 41.9400 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=64` | 2 | vulkan (+24.02%) | 56.5360 | 45.5880 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+114.10%) | 98.9640 | 46.2240 |
| `independent_throughput/q6-x8-selected-down/operation=x8_selected_dp4a_dot_prequantized,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=64` | 2 | vulkan (+232.63%) | 30.1360 | 9.0600 |
| `independent_throughput/q6-x8-selected-down/operation=x8_selected_dp4a_quantize_plus_dot,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+170.19%) | 33.8600 | 12.5320 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+9.59%) | 18.1400 | 16.5520 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+39.69%) | 24.3680 | 17.4440 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / hip | 222 | 18 | 0 | 92.50 | 0.2838 | 240 |
| RL / vulkan | 187 | 53 | 0 | 77.92 | 0.7579 | 240 |

## Pinned hipEngine-harness comparison

The `hipengine` profile covers the same benchmark row set as pinned HipEngine `f2c3ad6`: family, operation, shape/sweep axes, repetition count, and serial/independent mode. Those rows deliberately run through Hipfire's existing Radiowave-tuned launch policy; wave size, workgroup geometry, source variant, ABI, and machine code remain optimization variables rather than parity constraints. HipEngine has 212 three-way matched core rows because its HIP independent sampler path rejects 12 rows; its 16 dispatch controls are reported separately. Hipfire executes and correctness-gates all 240 rows across four backends.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine core | RL / vulkan | 197 | 27 | 87.95 | 0.4798 | 224 |
| hipEngine core | RL / hip | 160 | 64 | 71.43 | 0.8555 | 224 |
| hipEngine dispatch | RL / vulkan | 0 | 0 | 0.00 | 0.0000 | 0 |
| hipEngine dispatch | RL / hip | 0 | 0 | 0.00 | 0.0000 | 0 |

## Harness verdict

This tuning smoke intentionally measures only `redline` versus `hip` versus `vulkan`. It uses the `hipengine_f2c` matrix, `radiowave_tuned` wave policy, and `default` scheduler profile. Every ranked row passed both CPU oracles. Redline beats Vulkan in 187/240 rows; final promotion still requires a full four-backend certification run.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. The `hipengine_f2c` profile matches HipEngine's row coverage while intentionally retaining Hipfire/Radiowave's tuned wave, workgroup, and source-variant choices. This artifact records the `radiowave_tuned` wave policy and `default` scheduler profile as controlled HIP compilation/launch factors. Every completion timestamp necessarily proves the measured work finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
