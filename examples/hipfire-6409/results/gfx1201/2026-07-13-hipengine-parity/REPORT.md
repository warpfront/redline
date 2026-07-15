# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 180/240 rows (75.00%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 180 | 37 | 5 | 18 | 75.00 | 240 |
| vulkan | 50 | 147 | 6 | 37 | 20.83 | 240 |
| hipgraph | 8 | 12 | 80 | 140 | 3.33 | 240 |
| hip | 2 | 44 | 149 | 45 | 0.83 | 240 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 85 | 19 | 0 | 16 | 120 |
| serial_latency | vulkan | 31 | 55 | 4 | 30 | 120 |
| serial_latency | hipgraph | 3 | 12 | 69 | 36 | 120 |
| serial_latency | hip | 1 | 34 | 47 | 38 | 120 |
| independent_throughput | redline | 95 | 18 | 5 | 2 | 120 |
| independent_throughput | vulkan | 19 | 92 | 2 | 7 | 120 |
| independent_throughput | hipgraph | 5 | 0 | 11 | 104 | 120 |
| independent_throughput | hip | 1 | 10 | 102 | 7 | 120 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/geometry/k=512,rows=1,wg=256,body=32;hip-wave=32` | 2 | vulkan (+12.10%) | 14.5960 | 13.0200 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+1.94%) | 23.5280 | 23.0800 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+2.41%) | 23.5960 | 23.0400 |
| `serial_latency/memory-waitcnt/variant=gather1,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+16.54%), hip (+10.09%), hipgraph (+9.89%) | 84.7920 | 72.7560 |
| `serial_latency/memory-waitcnt/variant=gather1,n=32768,body=64,wg=256;hip-wave=32` | 4 | vulkan (+21.71%), hip (+14.85%), hipgraph (+14.84%) | 88.3640 | 72.6040 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=64;hip-wave=64` | 2 | vulkan (+12.53%) | 41.9480 | 37.2760 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=256;hip-wave=64` | 2 | vulkan (+12.26%) | 41.8360 | 37.2680 |
| `serial_latency/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+6.05%), hip (+5.24%), hipgraph (+5.17%) | 447.2560 | 421.7440 |
| `serial_latency/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 4 | vulkan (+3.92%), hip (+3.38%), hipgraph (+3.32%) | 439.3760 | 422.8040 |
| `serial_latency/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+5.95%), hip (+5.13%), hipgraph (+5.07%) | 446.8200 | 421.7440 |
| `serial_latency/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 4 | vulkan (+3.93%), hip (+3.32%), hipgraph (+3.22%) | 439.0520 | 422.4600 |
| `serial_latency/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+6.21%), hip (+5.44%), hipgraph (+5.38%) | 448.0360 | 421.8520 |
| `serial_latency/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 4 | vulkan (+3.85%), hip (+3.18%), hipgraph (+3.11%) | 438.3760 | 422.1040 |
| `serial_latency/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+2.23%), hip (+2.17%), hipgraph (+2.11%) | 435.0440 | 425.5680 |
| `serial_latency/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 4 | hip (+3.19%), vulkan (+3.16%), hipgraph (+3.12%) | 439.6000 | 425.9930 |
| `serial_latency/vopd/variant=independent_fma,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 4 | hipgraph (+24.84%), hip (+24.19%), vulkan (+13.07%) | 129.2720 | 103.5523 |
| `serial_latency/vopd/variant=dependent_fma,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 4 | hipgraph (+20.56%), hip (+19.51%), vulkan (+12.11%) | 128.4400 | 106.5403 |
| `serial_latency/vopd/variant=dependent_fma,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 2 | vulkan (+1.66%) | 116.5240 | 114.6200 |
| `serial_latency/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 4 | vulkan (+45.81%), hipgraph (+31.95%), hip (+31.44%) | 331.0800 | 227.0600 |
| `serial_latency/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 4 | vulkan (+33.38%), hip (+12.67%), hipgraph (+12.38%) | 301.2280 | 225.8360 |
| `serial_latency/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 4 | hipgraph (+28.59%), vulkan (+26.17%), hip (+23.49%) | 254.2240 | 197.7044 |
| `serial_latency/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 4 | vulkan (+17.45%), hip (+3.02%), hipgraph (+2.69%) | 236.7960 | 201.6080 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=64` | 2 | vulkan (+52.97%) | 97.8760 | 63.9840 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+54.73%) | 107.0160 | 69.1640 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=64` | 2 | vulkan (+38.36%) | 83.7440 | 60.5240 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+164.25%) | 213.4520 | 80.7760 |
| `serial_latency/q6-x8-selected-down/operation=x8_selected_dp4a_dot_prequantized,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=64` | 2 | vulkan (+124.47%) | 36.6960 | 16.3480 |
| `serial_latency/q6-x8-selected-down/operation=x8_selected_dp4a_quantize_plus_dot,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+82.00%) | 38.1680 | 20.9720 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+0.08%) | 15.4480 | 15.4360 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+16.93%) | 15.1360 | 12.9440 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+14.27%) | 11.2720 | 9.8640 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+20.13%) | 35.7840 | 29.7880 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+1.42%) | 36.1960 | 35.6880 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+53.64%) | 38.1640 | 24.8400 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+34.29%) | 40.0400 | 29.8160 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=128;hip-wave=32` | 2 | vulkan (+18.05%) | 0.1560 | 0.1321 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=1024;hip-wave=32` | 2 | vulkan (+149.57%) | 0.8316 | 0.3332 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=8192;hip-wave=32` | 2 | vulkan (+80.24%) | 3.4621 | 1.9208 |
| `independent_throughput/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+16.14%) | 13.4720 | 11.6000 |
| `independent_throughput/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+10.09%) | 13.0960 | 11.8960 |
| `independent_throughput/memory-waitcnt/variant=strided4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+19.05%) | 13.9240 | 11.6960 |
| `independent_throughput/memory-waitcnt/variant=strided4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+4.62%) | 12.3240 | 11.7800 |
| `independent_throughput/memory-waitcnt/variant=gather1,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+42.36%) | 39.7400 | 27.9160 |
| `independent_throughput/memory-waitcnt/variant=gather1,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+39.15%) | 39.9080 | 28.6800 |
| `independent_throughput/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 3 | hipgraph (+4.00%), hip (+0.43%) | 426.6320 | 410.2131 |
| `independent_throughput/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 3 | hipgraph (+5.45%), hip (+0.99%) | 426.6920 | 404.6451 |
| `independent_throughput/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 3 | hipgraph (+5.86%), hip (+0.42%) | 426.5680 | 402.9651 |
| `independent_throughput/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 3 | hipgraph (+14.53%), hip (+3.06%) | 422.4880 | 368.8850 |
| `independent_throughput/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 3 | hipgraph (+5.12%), hip (+4.47%) | 422.2960 | 401.7172 |
| `independent_throughput/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 4 | vulkan (+29.70%), hip (+17.34%), hipgraph (+10.98%) | 141.0480 | 108.7520 |
| `independent_throughput/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 2 | vulkan (+30.03%) | 141.3520 | 108.7080 |
| `independent_throughput/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 4 | hip (+14.56%), vulkan (+12.20%), hipgraph (+5.11%) | 110.0600 | 96.0683 |
| `independent_throughput/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 2 | vulkan (+12.83%) | 110.1720 | 97.6440 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=64` | 2 | vulkan (+23.89%) | 50.7720 | 40.9800 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+121.05%) | 93.6120 | 42.3480 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=64` | 2 | vulkan (+37.25%) | 60.8360 | 44.3240 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+130.50%) | 108.5080 | 47.0760 |
| `independent_throughput/q6-x8-selected-down/operation=x8_selected_dp4a_dot_prequantized,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=64` | 2 | vulkan (+136.82%) | 23.0000 | 9.7120 |
| `independent_throughput/q6-x8-selected-down/operation=x8_selected_dp4a_quantize_plus_dot,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+77.43%) | 24.5920 | 13.8600 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+26.10%) | 23.1880 | 18.3880 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 185 | 55 | 0 | 77.08 | 0.7945 | 240 |
| RL / hipgraph | 217 | 23 | 0 | 90.42 | 0.2751 | 240 |
| RL / hip | 217 | 23 | 0 | 90.42 | 0.2849 | 240 |

## Pinned hipEngine-harness comparison

The `hipengine` profile covers the same benchmark row set as pinned HipEngine `f2c3ad6`: family, operation, shape/sweep axes, repetition count, and serial/independent mode. Those rows deliberately run through Hipfire's existing Radiowave-tuned launch policy; wave size, workgroup geometry, source variant, ABI, and machine code remain optimization variables rather than parity constraints. HipEngine has 212 three-way matched core rows because its HIP independent sampler path rejects 12 rows; its 16 dispatch controls are reported separately. Hipfire executes and correctness-gates all 240 rows across four backends.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine core | RL / vulkan | 192 | 20 | 90.57 | 0.4589 | 212 |
| hipEngine core | RL / hip | 158 | 54 | 74.53 | 0.8343 | 212 |
| hipEngine dispatch | RL / vulkan | 6 | 10 | 37.50 | 1.4331 | 16 |
| hipEngine dispatch | RL / hip | 16 | 0 | 100.00 | 0.2249 | 16 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 217/240 rows and HipGraph in 217/240 while all three select identical per-row hipcc code objects. This run uses the `hipengine_f2c` matrix, `radiowave_tuned` wave policy, and `default` scheduler profile. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 185/240 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. The `hipengine_f2c` profile matches HipEngine's row coverage while intentionally retaining Hipfire/Radiowave's tuned wave, workgroup, and source-variant choices. This artifact records the `radiowave_tuned` wave policy and `default` scheduler profile as controlled HIP compilation/launch factors. Every completion timestamp necessarily proves the measured work finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
