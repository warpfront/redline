# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 138/234 rows (58.97%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 138 | 74 | 12 | 10 | 58.97 | 234 |
| vulkan | 79 | 95 | 3 | 57 | 33.76 | 234 |
| hipgraph | 14 | 45 | 84 | 91 | 5.98 | 234 |
| hip | 3 | 20 | 135 | 76 | 1.28 | 234 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 62 | 40 | 11 | 4 | 117 |
| serial_latency | vulkan | 38 | 19 | 3 | 57 | 117 |
| serial_latency | hipgraph | 14 | 45 | 45 | 13 | 117 |
| serial_latency | hip | 3 | 13 | 58 | 43 | 117 |
| independent_throughput | redline | 76 | 34 | 1 | 6 | 117 |
| independent_throughput | vulkan | 41 | 76 | 0 | 0 | 117 |
| independent_throughput | hipgraph | 0 | 0 | 39 | 78 | 117 |
| independent_throughput | hip | 0 | 7 | 77 | 33 | 117 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/dispatch-grid/sweep=count,count=1,grid=1;hip-wave=32` | 3 | vulkan (+128.03%), hip (+10.55%) | 21.8000 | 9.5600 |
| `serial_latency/dispatch-grid/sweep=count,count=50,grid=1;hip-wave=32` | 2 | vulkan (+6.37%) | 3.2320 | 3.0384 |
| `serial_latency/dispatch-grid/sweep=count,count=200,grid=1;hip-wave=32` | 2 | vulkan (+6.71%) | 3.1488 | 2.9508 |
| `serial_latency/dispatch-grid/sweep=count,count=941,grid=1;hip-wave=32` | 2 | vulkan (+6.77%) | 3.1108 | 2.9134 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=1;hip-wave=32` | 2 | vulkan (+7.09%) | 3.1146 | 2.9084 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=128;hip-wave=32` | 2 | vulkan (+5.76%) | 3.1369 | 2.9660 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=1024;hip-wave=32` | 2 | vulkan (+5.90%) | 3.5145 | 3.3186 |
| `serial_latency/geometry/k=512,rows=1,wg=256,body=32;hip-wave=32` | 2 | hip (+1.33%) | 14.6440 | 14.4519 |
| `serial_latency/geometry/k=512,rows=4,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+2.90%) | 65.0880 | 63.2519 |
| `serial_latency/geometry/k=512,rows=4,wg=256,body=32;hip-wave=32` | 3 | hipgraph (+4.88%), hip (+0.52%) | 14.7160 | 14.0319 |
| `serial_latency/geometry/k=2048,rows=1,wg=64,body=32;hip-wave=32` | 3 | hip (+2.58%), hipgraph (+1.33%) | 144.4920 | 140.8557 |
| `serial_latency/geometry/k=2048,rows=1,wg=256,body=32;hip-wave=32` | 2 | hipgraph (+0.41%) | 68.9120 | 68.6278 |
| `serial_latency/geometry/k=2048,rows=4,wg=256,body=32;hip-wave=32` | 2 | hipgraph (+3.26%) | 66.4720 | 64.3758 |
| `serial_latency/reduction/variant=lds_tree,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+0.15%) | 136.7400 | 136.5397 |
| `serial_latency/reduction/variant=wave_shuffle,k=512,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+5.56%), hip (+0.07%) | 66.3200 | 62.8279 |
| `serial_latency/reduction/variant=wave_shuffle,k=512,rows=1,wg=256,body=32;hip-wave=32` | 3 | hipgraph (+7.72%), hip (+2.45%) | 14.9040 | 13.8360 |
| `serial_latency/reduction/variant=wave_shuffle,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+1.03%) | 138.9680 | 137.5518 |
| `serial_latency/reduction/variant=wave_shuffle,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 3 | hipgraph (+5.88%), hip (+0.41%) | 67.9760 | 64.2039 |
| `serial_latency/reduction/variant=multi_accum4,k=512,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+1.57%) | 65.2840 | 64.2758 |
| `serial_latency/reduction/variant=multi_accum8,k=512,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+2.08%) | 67.7000 | 66.3198 |
| `serial_latency/reduction/variant=multi_accum8,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 2 | hipgraph (+0.12%) | 67.8520 | 67.7719 |
| `serial_latency/reduction/variant=multi_accum16,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 2 | hipgraph (+0.27%) | 74.7440 | 74.5438 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+12.58%) | 87.6280 | 77.8360 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+10.47%) | 86.2960 | 78.1160 |
| `serial_latency/memory-waitcnt/variant=strided4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+11.74%) | 86.9280 | 77.7920 |
| `serial_latency/memory-waitcnt/variant=strided4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+11.71%) | 87.0800 | 77.9520 |
| `serial_latency/memory-waitcnt/variant=gather1,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+1.86%), hipgraph (+1.60%), hip (+0.60%) | 364.5880 | 357.9320 |
| `serial_latency/memory-waitcnt/variant=gather1,n=32768,body=64,wg=256;hip-wave=32` | 3 | vulkan (+2.05%), hipgraph (+0.64%) | 365.6720 | 358.3360 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=64;hip-wave=64` | 2 | vulkan (+11.31%) | 86.8200 | 78.0000 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=256;hip-wave=64` | 2 | vulkan (+11.61%) | 88.6600 | 79.4400 |
| `serial_latency/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+68.22%), hipgraph (+0.76%), hip (+0.66%) | 1505.7880 | 895.1560 |
| `serial_latency/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 4 | vulkan (+60.32%), hip (+0.48%), hipgraph (+0.03%) | 1437.8680 | 896.8560 |
| `serial_latency/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 3 | vulkan (+67.60%), hipgraph (+0.18%) | 1499.2320 | 894.5560 |
| `serial_latency/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+59.22%) | 1422.5440 | 893.4400 |
| `serial_latency/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+58.52%) | 1497.6600 | 944.7800 |
| `serial_latency/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 4 | vulkan (+53.03%), hip (+0.73%), hipgraph (+0.17%) | 1450.2080 | 947.6720 |
| `serial_latency/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 3 | vulkan (+68.45%), hipgraph (+0.43%) | 1501.1800 | 891.1920 |
| `serial_latency/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 3 | vulkan (+63.17%), hip (+0.94%) | 1452.7560 | 890.3440 |
| `serial_latency/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 2 | vulkan (+5.28%) | 334.0480 | 317.3040 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=1,wg=64;hip-wave=32` | 3 | hip (+0.06%), hipgraph (+0.04%) | 1025.9120 | 1025.3095 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=4,wg=256;hip-wave=32` | 2 | hipgraph (+0.48%) | 73.2000 | 72.8478 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=8,wg=256;hip-wave=32` | 2 | vulkan (+30.23%) | 71.0040 | 54.5200 |
| `serial_latency/q4-selected-dual/operation=q8_1_quantize,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=32;hip-wave=32` | 2 | vulkan (+45.79%) | 4.8520 | 3.3280 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+5.95%) | 261.1440 | 246.4760 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+11.93%) | 251.0560 | 224.2920 |
| `serial_latency/q6-x8-selected-down/operation=q8_1_quantize,rows=8,experts=256,in=512,out=2048,wg=32;hip-wave=32` | 2 | vulkan (+134.10%) | 4.8880 | 2.0880 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=768,out=2048,rows=1,wg=32;hip-wave=32` | 2 | vulkan (+118.76%) | 4.8040 | 2.1960 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+13.24%) | 8.5520 | 7.5520 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=768,out=2048,rows=4,wg=32;hip-wave=32` | 2 | vulkan (+160.29%) | 8.6000 | 3.3040 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+22.02%) | 28.2880 | 23.1840 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+23.78%) | 33.8960 | 27.3840 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=2048,out=2048,rows=1,wg=32;hip-wave=32` | 2 | vulkan (+128.74%) | 4.7120 | 2.0600 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=2048,out=2048,rows=4,wg=32;hip-wave=32` | 2 | vulkan (+165.24%) | 9.4000 | 3.5440 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+11.06%) | 86.5880 | 77.9640 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+2.36%) | 91.2080 | 89.1080 |
| `independent_throughput/dispatch-grid/sweep=count,count=1,grid=1;hip-wave=32` | 3 | vulkan (+122.46%), hip (+22.38%) | 21.0000 | 9.4400 |
| `independent_throughput/dispatch-grid/sweep=count,count=941,grid=1;hip-wave=32` | 2 | vulkan (+62.48%) | 0.1303 | 0.0802 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=128;hip-wave=32` | 2 | vulkan (+205.96%) | 0.8425 | 0.2754 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=1024;hip-wave=32` | 2 | vulkan (+1046.96%) | 6.5590 | 0.5719 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=8192;hip-wave=32` | 2 | vulkan (+87.15%) | 6.6791 | 3.5689 |
| `independent_throughput/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+81.69%) | 85.6200 | 47.1240 |
| `independent_throughput/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+59.69%) | 94.5040 | 59.1800 |
| `independent_throughput/memory-waitcnt/variant=strided4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+96.51%) | 83.7520 | 42.6200 |
| `independent_throughput/memory-waitcnt/variant=strided4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+70.22%) | 93.3640 | 54.8480 |
| `independent_throughput/memory-waitcnt/variant=gather1,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+22.91%) | 360.0120 | 292.9120 |
| `independent_throughput/memory-waitcnt/variant=gather1,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+22.02%) | 397.0960 | 325.4360 |
| `independent_throughput/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=64;hip-wave=64` | 2 | vulkan (+33.21%) | 81.7480 | 61.3680 |
| `independent_throughput/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=256;hip-wave=64` | 2 | vulkan (+18.38%) | 79.5040 | 67.1600 |
| `independent_throughput/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+96.96%), hip (+18.00%), hipgraph (+5.12%) | 1797.1880 | 912.4480 |
| `independent_throughput/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+19.62%) | 1214.9960 | 1015.7480 |
| `independent_throughput/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+72.65%), hip (+18.01%), hipgraph (+1.54%) | 1756.4800 | 1017.3520 |
| `independent_throughput/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+17.89%) | 1234.6920 | 1047.3200 |
| `independent_throughput/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+66.93%), hip (+20.90%), hipgraph (+1.68%) | 1809.1120 | 1083.7520 |
| `independent_throughput/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+7.15%) | 1251.4600 | 1167.9440 |
| `independent_throughput/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+88.57%), hip (+15.58%), hipgraph (+4.05%) | 1811.0560 | 960.4240 |
| `independent_throughput/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+25.82%) | 1257.7120 | 999.6400 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=2,wg=128,body=16;hip-wave=32` | 4 | vulkan (+26.64%), hip (+26.60%), hipgraph (+13.59%) | 56.3400 | 44.4880 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=2,wg=128,body=16;hip-wave=32` | 4 | vulkan (+19.15%), hip (+9.96%), hipgraph (+4.88%) | 50.8520 | 42.6800 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=2,wg=256,body=16;hip-wave=32` | 2 | vulkan (+16.33%) | 13.9360 | 11.9800 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=4,wg=256,body=16;hip-wave=32` | 2 | vulkan (+1.07%) | 9.4200 | 9.3200 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+38.70%) | 255.0240 | 183.8640 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+37.08%) | 247.5040 | 180.5600 |
| `independent_throughput/q6-x8-selected-down/operation=x8_selected_dp4a_quantize_plus_dot,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+86.67%) | 58.0840 | 31.1160 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+6.89%) | 7.0760 | 6.6200 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+34.34%) | 5.8840 | 4.3800 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+13.19%) | 22.2360 | 19.6440 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+4.69%) | 25.2840 | 24.1520 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+31.21%) | 21.8440 | 16.6480 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+47.84%) | 28.1080 | 19.0120 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+10.83%) | 24.5640 | 22.1640 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+22.21%) | 17.9600 | 14.6960 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+71.91%) | 40.7760 | 23.7200 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+68.90%) | 81.6880 | 48.3640 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+44.25%) | 82.5840 | 57.2520 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+71.83%) | 99.2960 | 57.7880 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+161.01%) | 192.5280 | 73.7640 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 155 | 79 | 0 | 66.24 | 0.8320 | 234 |
| RL / hipgraph | 205 | 29 | 0 | 87.61 | 0.5811 | 234 |
| RL / hip | 214 | 20 | 0 | 91.45 | 0.5656 | 234 |

## Pinned hipEngine-harness comparison

The `hipengine` profile covers the same benchmark row set as pinned HipEngine `f2c3ad6`: family, operation, shape/sweep axes, repetition count, and serial/independent mode. Those rows deliberately run through Hipfire's existing Radiowave-tuned launch policy; wave size, workgroup geometry, source variant, ABI, and machine code remain optimization variables rather than parity constraints. HipEngine has 212 three-way matched core rows because its HIP independent sampler path rejects 12 rows; its 16 dispatch controls are reported separately. Hipfire executes and correctness-gates all 240 rows across four backends.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine core | RL / vulkan | 192 | 20 | 90.57 | 0.4589 | 212 |
| hipEngine core | RL / hip | 158 | 54 | 74.53 | 0.8343 | 212 |
| hipEngine dispatch | RL / vulkan | 6 | 10 | 37.50 | 1.4331 | 16 |
| hipEngine dispatch | RL / hip | 16 | 0 | 100.00 | 0.2249 | 16 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 214/234 rows and HipGraph in 205/234 while all three select identical per-row hipcc code objects. This run uses the `hipengine_f2c` matrix, `radiowave_tuned` wave policy, and `default` scheduler profile. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 155/234 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. The `hipengine_f2c` profile matches HipEngine's row coverage while intentionally retaining Hipfire/Radiowave's tuned wave, workgroup, and source-variant choices. This artifact records the `radiowave_tuned` wave policy and `default` scheduler profile as controlled HIP compilation/launch factors. Every completion timestamp necessarily proves the measured work finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
