# Hipfire/Redline ROCm issue 6409 backend comparison

Correctness-gated result: **Redline wins 187/240 rows (77.92%)**.

## Placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 187 | 53 | 0 | 0 | 77.92 | 240 |
| vulkan | 53 | 187 | 0 | 0 | 22.08 | 240 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 87 | 33 | 0 | 0 | 120 |
| serial_latency | vulkan | 33 | 87 | 0 | 0 | 120 |
| independent_throughput | redline | 100 | 20 | 0 | 0 | 120 |
| independent_throughput | vulkan | 20 | 100 | 0 | 0 | 120 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+1.54%) | 22.3480 | 22.0080 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+1.20%) | 22.3000 | 22.0360 |
| `serial_latency/memory-waitcnt/variant=gather1,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+16.94%) | 79.2120 | 67.7360 |
| `serial_latency/memory-waitcnt/variant=gather1,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+22.04%) | 82.1000 | 67.2720 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=64;hip-wave=64` | 2 | vulkan (+12.19%) | 39.8760 | 35.5440 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=256;hip-wave=64` | 2 | vulkan (+9.42%) | 38.9680 | 35.6120 |
| `serial_latency/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+5.94%) | 446.7440 | 421.6960 |
| `serial_latency/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+3.96%) | 439.1040 | 422.3920 |
| `serial_latency/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+6.03%) | 447.1120 | 421.6840 |
| `serial_latency/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+3.82%) | 438.9480 | 422.7800 |
| `serial_latency/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+5.96%) | 447.0800 | 421.9480 |
| `serial_latency/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+3.78%) | 438.1440 | 422.1920 |
| `serial_latency/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+2.28%) | 435.1200 | 425.4320 |
| `serial_latency/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+2.98%) | 438.6560 | 425.9640 |
| `serial_latency/vopd/variant=independent_fma,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 2 | vulkan (+9.44%) | 120.1240 | 109.7600 |
| `serial_latency/vopd/variant=independent_fma,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 2 | vulkan (+3.03%) | 113.6640 | 110.3160 |
| `serial_latency/vopd/variant=dependent_fma,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 2 | vulkan (+11.66%) | 122.3840 | 109.6080 |
| `serial_latency/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 2 | vulkan (+45.21%) | 314.6360 | 216.6760 |
| `serial_latency/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 2 | vulkan (+33.25%) | 289.2720 | 217.0960 |
| `serial_latency/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 2 | vulkan (+29.71%) | 248.8840 | 191.8840 |
| `serial_latency/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 2 | vulkan (+17.20%) | 226.0560 | 192.8880 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=64` | 2 | vulkan (+57.54%) | 93.7480 | 59.5080 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+148.42%) | 161.7120 | 65.0960 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=64` | 2 | vulkan (+53.02%) | 111.4080 | 72.8080 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+149.87%) | 191.0920 | 76.4760 |
| `serial_latency/q6-x8-selected-down/operation=x8_selected_dp4a_dot_prequantized,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=64` | 2 | vulkan (+127.07%) | 35.1680 | 15.4880 |
| `serial_latency/q6-x8-selected-down/operation=x8_selected_dp4a_quantize_plus_dot,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+79.63%) | 35.6600 | 19.8520 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+14.73%) | 13.6760 | 11.9200 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+12.48%) | 10.1640 | 9.0360 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+18.89%) | 31.7480 | 26.7040 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+4.07%) | 33.4240 | 32.1160 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+51.89%) | 35.1360 | 23.1320 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+32.27%) | 36.8120 | 27.8320 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=1024;hip-wave=32` | 2 | vulkan (+65.62%) | 1.0331 | 0.6238 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=8192;hip-wave=32` | 2 | vulkan (+42.13%) | 5.3800 | 3.7851 |
| `independent_throughput/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+1.36%) | 16.1440 | 15.9280 |
| `independent_throughput/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+0.62%) | 16.1920 | 16.0920 |
| `independent_throughput/memory-waitcnt/variant=gather1,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+48.37%) | 77.8160 | 52.4480 |
| `independent_throughput/memory-waitcnt/variant=gather1,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+46.03%) | 77.8160 | 53.2880 |
| `independent_throughput/vopd/variant=independent_fma,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 2 | vulkan (+3.86%) | 110.1280 | 106.0320 |
| `independent_throughput/vopd/variant=independent_fma,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 2 | vulkan (+1.11%) | 107.0200 | 105.8400 |
| `independent_throughput/vopd/variant=dependent_fma,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 2 | vulkan (+3.56%) | 108.6960 | 104.9640 |
| `independent_throughput/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 2 | vulkan (+30.25%) | 275.6400 | 211.6200 |
| `independent_throughput/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 2 | vulkan (+30.00%) | 275.0840 | 211.6040 |
| `independent_throughput/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 2 | vulkan (+15.23%) | 216.8640 | 188.2000 |
| `independent_throughput/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 2 | vulkan (+18.92%) | 222.9960 | 187.5200 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=64` | 2 | vulkan (+32.69%) | 81.0320 | 61.0680 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+137.72%) | 148.1200 | 62.3080 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=64` | 2 | vulkan (+39.24%) | 102.7680 | 73.8080 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+142.98%) | 175.8600 | 72.3760 |
| `independent_throughput/q6-x8-selected-down/operation=x8_selected_dp4a_dot_prequantized,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=64` | 2 | vulkan (+113.46%) | 33.5640 | 15.7240 |
| `independent_throughput/q6-x8-selected-down/operation=x8_selected_dp4a_quantize_plus_dot,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+115.15%) | 40.9640 | 19.0400 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+0.40%) | 27.0080 | 26.9000 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 187 | 53 | 0 | 77.92 | 0.7544 | 240 |

## Pinned hipEngine-harness comparison

The `hipengine` profile covers the same benchmark row set as pinned HipEngine `f2c3ad6`: family, operation, shape/sweep axes, repetition count, and serial/independent mode. Those rows deliberately run through Hipfire's existing Radiowave-tuned launch policy; wave size, workgroup geometry, source variant, ABI, and machine code remain optimization variables rather than parity constraints. HipEngine has 212 three-way matched core rows because its HIP independent sampler path rejects 12 rows; its 16 dispatch controls are reported separately. Hipfire executes and correctness-gates all 240 rows across four backends.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine core | RL / vulkan | 192 | 20 | 90.57 | 0.4589 | 212 |
| hipEngine core | RL / hip | 158 | 54 | 74.53 | 0.8343 | 212 |
| hipEngine dispatch | RL / vulkan | 6 | 10 | 37.50 | 1.4331 | 16 |
| hipEngine dispatch | RL / hip | 16 | 0 | 100.00 | 0.2249 | 16 |

## Harness verdict

This tuning smoke intentionally measures only `redline` versus `vulkan`. It uses the `hipengine_f2c` matrix, `radiowave_tuned` wave policy, and `radiowave-recipe-or-default` scheduler profile. Every ranked row passed both CPU oracles. Redline beats Vulkan in 187/240 rows; final promotion still requires a full four-backend certification run.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. The `hipengine_f2c` profile matches HipEngine's row coverage while intentionally retaining Hipfire/Radiowave's tuned wave, workgroup, and source-variant choices. This artifact records the `radiowave_tuned` wave policy and `radiowave-recipe-or-default` scheduler profile as controlled HIP compilation/launch factors. Every completion timestamp necessarily proves the measured work finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
