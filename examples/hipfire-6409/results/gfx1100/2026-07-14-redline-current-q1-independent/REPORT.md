# Hipfire/Redline ROCm issue 6409 backend comparison

Correctness-gated result: **Redline wins 67/120 rows (55.83%)**.

## Placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 67 | 53 | 0 | 0 | 55.83 | 120 |
| vulkan | 53 | 67 | 0 | 0 | 44.17 | 120 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| independent_throughput | redline | 67 | 53 | 0 | 0 | 120 |
| independent_throughput | vulkan | 53 | 67 | 0 | 0 | 120 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `independent_throughput/dispatch-grid/sweep=count,count=1,grid=1;hip-wave=32` | 2 | vulkan (+0.42%) | 9.5600 | 9.5200 |
| `independent_throughput/dispatch-grid/sweep=count,count=200,grid=1;hip-wave=32` | 2 | vulkan (+64.79%) | 0.3398 | 0.2062 |
| `independent_throughput/dispatch-grid/sweep=count,count=941,grid=1;hip-wave=32` | 2 | vulkan (+188.80%) | 0.1919 | 0.0664 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=1;hip-wave=32` | 2 | vulkan (+83.31%) | 0.1751 | 0.0955 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=128;hip-wave=32` | 2 | vulkan (+836.83%) | 0.6411 | 0.0684 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=1024;hip-wave=32` | 2 | vulkan (+384.75%) | 1.1257 | 0.2322 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=8192;hip-wave=32` | 2 | vulkan (+217.24%) | 4.4918 | 1.4159 |
| `independent_throughput/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+7.13%) | 13.8920 | 12.9680 |
| `independent_throughput/memory-waitcnt/variant=strided4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+16.37%) | 13.2520 | 11.3880 |
| `independent_throughput/memory-waitcnt/variant=gather1,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+35.36%) | 79.5680 | 58.7840 |
| `independent_throughput/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+12.77%) | 20.2040 | 17.9160 |
| `independent_throughput/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+12.57%) | 295.2960 | 262.3200 |
| `independent_throughput/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+11.31%) | 296.2960 | 266.1960 |
| `independent_throughput/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+3.52%) | 264.5560 | 255.5520 |
| `independent_throughput/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+8.94%) | 301.1320 | 276.4160 |
| `independent_throughput/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+8.54%) | 259.4640 | 239.0520 |
| `independent_throughput/vopd/variant=independent_fma,accums=4,n=65536,body=512,wg=64;hip-wave=32` | 2 | vulkan (+24.57%) | 80.7480 | 64.8200 |
| `independent_throughput/vopd/variant=dependent_fma,accums=4,n=65536,body=512,wg=64;hip-wave=32` | 2 | vulkan (+26.72%) | 78.0040 | 61.5560 |
| `independent_throughput/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=64;hip-wave=32` | 2 | vulkan (+40.05%) | 138.6720 | 99.0160 |
| `independent_throughput/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=64;hip-wave=32` | 2 | vulkan (+43.73%) | 127.4560 | 88.6760 |
| `independent_throughput/two-stage-reduction/k=8192,rows=1,splits=2,wg=128,body=16;hip-wave=32` | 2 | vulkan (+21.25%) | 5.9800 | 4.9320 |
| `independent_throughput/two-stage-reduction/k=8192,rows=1,splits=4,wg=128,body=16;hip-wave=32` | 2 | vulkan (+23.05%) | 4.3560 | 3.5400 |
| `independent_throughput/two-stage-reduction/k=8192,rows=4,splits=2,wg=128,body=16;hip-wave=32` | 2 | vulkan (+16.35%) | 6.3760 | 5.4800 |
| `independent_throughput/two-stage-reduction/k=8192,rows=4,splits=4,wg=128,body=16;hip-wave=32` | 2 | vulkan (+24.81%) | 5.2720 | 4.2240 |
| `independent_throughput/two-stage-reduction/k=8192,rows=4,splits=2,wg=256,body=16;hip-wave=32` | 2 | vulkan (+6.24%) | 4.6320 | 4.3600 |
| `independent_throughput/two-stage-reduction/k=8192,rows=4,splits=4,wg=256,body=16;hip-wave=32` | 2 | vulkan (+37.59%) | 4.3920 | 3.1920 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=2,wg=128,body=16;hip-wave=32` | 2 | vulkan (+72.92%) | 16.8560 | 9.7480 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=4,wg=128,body=16;hip-wave=32` | 2 | vulkan (+44.68%) | 9.8440 | 6.8040 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=2,wg=256,body=16;hip-wave=32` | 2 | vulkan (+50.77%) | 9.7520 | 6.4680 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=4,wg=256,body=16;hip-wave=32` | 2 | vulkan (+68.14%) | 6.2280 | 3.7040 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=2,wg=128,body=16;hip-wave=32` | 2 | vulkan (+60.94%) | 17.1560 | 10.6600 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=4,wg=128,body=16;hip-wave=32` | 2 | vulkan (+79.74%) | 10.6840 | 5.9440 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=2,wg=256,body=16;hip-wave=32` | 2 | vulkan (+55.26%) | 10.1600 | 6.5440 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=4,wg=256,body=16;hip-wave=32` | 2 | vulkan (+43.05%) | 7.2440 | 5.0640 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+82.90%) | 73.5480 | 40.2120 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+157.83%) | 83.8160 | 32.5080 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+91.54%) | 77.1160 | 40.2600 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+174.45%) | 95.5520 | 34.8160 |
| `independent_throughput/q6-x8-selected-down/operation=x8_selected_dp4a_dot_prequantized,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+761.96%) | 80.1280 | 9.2960 |
| `independent_throughput/q6-x8-selected-down/operation=x8_selected_dp4a_quantize_plus_dot,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+348.08%) | 52.3000 | 11.6720 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+31.69%) | 7.5800 | 5.7560 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+94.62%) | 10.8520 | 5.5760 |
| `independent_throughput/dense-q8/operation=q8_1_quantize,in=768,out=2048,rows=4,wg=32;hip-wave=32` | 2 | vulkan (+15.18%) | 2.1240 | 1.8440 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+26.75%) | 9.0600 | 7.1480 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+109.30%) | 18.9880 | 9.0720 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+12.16%) | 11.4400 | 10.2000 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+37.17%) | 21.2400 | 15.4840 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+22.83%) | 6.5640 | 5.3440 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+13.86%) | 12.9120 | 11.3400 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+12.58%) | 7.9480 | 7.0600 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+172.31%) | 24.9000 | 9.1440 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+32.81%) | 30.3760 | 22.8720 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+153.46%) | 78.3000 | 30.8920 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 67 | 53 | 0 | 55.83 | 0.8981 | 120 |

## Harness verdict

This tuning smoke intentionally measures only `redline` versus `vulkan`. It uses the `hipengine_f2c` matrix, `radiowave_tuned` wave policy, and `radiowave-recipe-or-default` scheduler profile. Every ranked row passed both CPU oracles. Redline beats Vulkan in 67/120 rows; final promotion still requires a full four-backend certification run.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. The `hipengine_f2c` profile matches HipEngine's row coverage while intentionally retaining Hipfire/Radiowave's tuned wave, workgroup, and source-variant choices. This artifact records the `radiowave_tuned` wave policy and `radiowave-recipe-or-default` scheduler profile as controlled HIP compilation/launch factors. Every completion timestamp necessarily proves the measured work finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
