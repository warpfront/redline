# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 131/240 rows (54.58%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 131 | 82 | 19 | 8 | 54.58 | 240 |
| vulkan | 86 | 94 | 3 | 57 | 35.83 | 240 |
| hipgraph | 19 | 42 | 85 | 94 | 7.92 | 240 |
| hip | 4 | 22 | 133 | 81 | 1.67 | 240 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 56 | 44 | 18 | 2 | 120 |
| serial_latency | vulkan | 41 | 20 | 3 | 56 | 120 |
| serial_latency | hipgraph | 19 | 42 | 43 | 16 | 120 |
| serial_latency | hip | 4 | 14 | 56 | 46 | 120 |
| independent_throughput | redline | 75 | 38 | 1 | 6 | 120 |
| independent_throughput | vulkan | 45 | 74 | 0 | 1 | 120 |
| independent_throughput | hipgraph | 0 | 0 | 42 | 78 | 120 |
| independent_throughput | hip | 0 | 8 | 77 | 35 | 120 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/dispatch-grid/sweep=count,count=1,grid=1;hip-wave=32` | 3 | vulkan (+128.99%), hip (+11.00%) | 21.8000 | 9.5200 |
| `serial_latency/dispatch-grid/sweep=count,count=50,grid=1;hip-wave=32` | 2 | vulkan (+4.84%) | 3.1888 | 3.0416 |
| `serial_latency/dispatch-grid/sweep=count,count=200,grid=1;hip-wave=32` | 2 | vulkan (+7.56%) | 3.1718 | 2.9488 |
| `serial_latency/dispatch-grid/sweep=count,count=941,grid=1;hip-wave=32` | 2 | vulkan (+7.00%) | 3.1160 | 2.9121 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=1;hip-wave=32` | 2 | vulkan (+7.27%) | 3.1143 | 2.9032 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=128;hip-wave=32` | 2 | vulkan (+5.65%) | 3.1365 | 2.9687 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=1024;hip-wave=32` | 2 | vulkan (+5.94%) | 3.5083 | 3.3115 |
| `serial_latency/geometry/k=512,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+1.63%) | 69.4560 | 68.3438 |
| `serial_latency/geometry/k=512,rows=1,wg=256,body=32;hip-wave=32` | 3 | hipgraph (+6.73%), hip (+2.02%) | 14.7800 | 13.8480 |
| `serial_latency/geometry/k=512,rows=4,wg=256,body=32;hip-wave=32` | 3 | hipgraph (+5.24%), hip (+1.26%) | 14.7920 | 14.0560 |
| `serial_latency/geometry/k=2048,rows=4,wg=64,body=32;hip-wave=32` | 3 | hip (+1.43%), hipgraph (+0.79%) | 142.8880 | 140.8676 |
| `serial_latency/geometry/k=2048,rows=4,wg=256,body=32;hip-wave=32` | 3 | hipgraph (+6.37%), hip (+0.31%) | 68.1560 | 64.0719 |
| `serial_latency/reduction/variant=lds_tree,k=512,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+0.00%) | 62.6360 | 62.6358 |
| `serial_latency/reduction/variant=extra_barrier,k=512,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+0.43%) | 63.0080 | 62.7359 |
| `serial_latency/reduction/variant=extra_barrier,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+5.24%) | 137.1720 | 130.3436 |
| `serial_latency/reduction/variant=wave_shuffle,k=512,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+5.06%), hip (+0.50%) | 66.2240 | 63.0317 |
| `serial_latency/reduction/variant=wave_shuffle,k=512,rows=1,wg=256,body=32;hip-wave=32` | 3 | hipgraph (+6.19%), hip (+2.33%) | 14.7560 | 13.8960 |
| `serial_latency/reduction/variant=wave_shuffle,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 3 | hip (+6.94%), hipgraph (+2.65%) | 141.0480 | 131.8997 |
| `serial_latency/reduction/variant=wave_shuffle,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 2 | hipgraph (+3.49%) | 66.4560 | 64.2158 |
| `serial_latency/reduction/variant=multi_accum4,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+0.38%) | 142.7960 | 142.2516 |
| `serial_latency/reduction/variant=multi_accum8,k=512,rows=1,wg=256,body=32;hip-wave=32` | 4 | hip (+39.97%), vulkan (+22.88%), hipgraph (+11.79%) | 30.0320 | 21.4559 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+10.94%) | 86.4400 | 77.9160 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+11.33%) | 87.0280 | 78.1720 |
| `serial_latency/memory-waitcnt/variant=strided4,n=32768,body=64,wg=64;hip-wave=32` | 3 | vulkan (+9.72%), hipgraph (+0.46%) | 85.2760 | 77.7200 |
| `serial_latency/memory-waitcnt/variant=strided4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+11.19%) | 86.7120 | 77.9840 |
| `serial_latency/memory-waitcnt/variant=gather1,n=32768,body=64,wg=64;hip-wave=32` | 3 | hipgraph (+1.03%), vulkan (+0.60%) | 362.2560 | 358.5671 |
| `serial_latency/memory-waitcnt/variant=gather1,n=32768,body=64,wg=256;hip-wave=32` | 3 | vulkan (+3.23%), hip (+1.31%) | 371.1000 | 359.4760 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=64;hip-wave=64` | 2 | vulkan (+11.92%) | 87.4120 | 78.1040 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=256;hip-wave=64` | 2 | vulkan (+11.12%) | 87.9320 | 79.1360 |
| `serial_latency/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+67.84%) | 1501.2280 | 894.4560 |
| `serial_latency/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 3 | vulkan (+58.88%), hipgraph (+0.71%) | 1429.4400 | 899.6920 |
| `serial_latency/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 3 | vulkan (+67.92%), hipgraph (+0.17%) | 1502.0680 | 894.5360 |
| `serial_latency/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 3 | vulkan (+60.32%), hipgraph (+0.32%) | 1433.0840 | 893.8960 |
| `serial_latency/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 3 | vulkan (+59.15%), hipgraph (+0.36%) | 1500.5800 | 942.8920 |
| `serial_latency/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+52.34%) | 1440.3840 | 945.5080 |
| `serial_latency/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+68.88%), hipgraph (+0.39%), hip (+0.33%) | 1504.4480 | 890.8440 |
| `serial_latency/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+61.97%) | 1444.7640 | 892.0000 |
| `serial_latency/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 2 | vulkan (+4.11%) | 330.5360 | 317.4960 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=1,wg=64;hip-wave=32` | 2 | hipgraph (+0.58%) | 187.6080 | 186.5194 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=1,wg=64;hip-wave=32` | 2 | hipgraph (+0.22%) | 1022.3120 | 1020.0811 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=4,wg=64;hip-wave=32` | 3 | hipgraph (+4.23%), hip (+3.86%) | 1220.8130 | 1171.2847 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=4,wg=256;hip-wave=32` | 2 | hipgraph (+17.13%) | 73.3800 | 62.6478 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=8,wg=256;hip-wave=32` | 2 | vulkan (+29.54%) | 71.2560 | 55.0080 |
| `serial_latency/two-stage-reduction/k=8192,rows=1,splits=4,wg=256,body=16;hip-wave=32` | 2 | hip (+15.53%) | 16.6680 | 14.4280 |
| `serial_latency/two-stage-reduction/k=8192,rows=4,splits=2,wg=128,body=16;hip-wave=32` | 3 | hipgraph (+44.18%), hip (+22.25%) | 22.4400 | 15.5639 |
| `serial_latency/two-stage-reduction/k=32768,rows=1,splits=4,wg=128,body=16;hip-wave=32` | 2 | hipgraph (+4.16%) | 28.5760 | 27.4360 |
| `serial_latency/two-stage-reduction/k=32768,rows=4,splits=2,wg=128,body=16;hip-wave=32` | 3 | hipgraph (+37.68%), hip (+19.67%) | 57.2040 | 41.5479 |
| `serial_latency/q4-selected-dual/operation=q8_1_quantize,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=32;hip-wave=32` | 2 | vulkan (+146.42%) | 5.5000 | 2.2320 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+6.46%) | 254.5400 | 239.1000 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+4.37%) | 261.9000 | 250.9280 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+17.90%) | 249.3440 | 211.4800 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+7.20%) | 254.5000 | 237.4040 |
| `serial_latency/q6-x8-selected-down/operation=q8_1_quantize,rows=8,experts=256,in=512,out=2048,wg=32;hip-wave=32` | 2 | vulkan (+164.87%) | 8.8680 | 3.3480 |
| `serial_latency/q6-x8-selected-down/operation=x8_selected_dp4a_quantize_plus_dot,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+0.38%) | 82.9280 | 82.6160 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=768,out=2048,rows=1,wg=32;hip-wave=32` | 2 | vulkan (+120.15%) | 4.7640 | 2.1640 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+65.60%) | 12.3800 | 7.4760 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=768,out=2048,rows=4,wg=32;hip-wave=32` | 2 | vulkan (+129.81%) | 4.7800 | 2.0800 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+22.00%) | 28.3680 | 23.2520 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+12.38%) | 29.9960 | 26.6920 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+7.38%) | 38.0560 | 35.4400 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=2048,out=2048,rows=1,wg=32;hip-wave=32` | 2 | vulkan (+120.74%) | 4.5560 | 2.0640 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=2048,out=2048,rows=4,wg=32;hip-wave=32` | 2 | vulkan (+155.78%) | 5.4840 | 2.1440 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+12.78%) | 87.1080 | 77.2360 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+3.65%) | 91.8680 | 88.6320 |
| `independent_throughput/dispatch-grid/sweep=count,count=1,grid=1;hip-wave=32` | 2 | vulkan (+13.07%) | 24.9200 | 22.0400 |
| `independent_throughput/dispatch-grid/sweep=count,count=941,grid=1;hip-wave=32` | 2 | vulkan (+241.27%) | 0.2468 | 0.0723 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=1;hip-wave=32` | 2 | vulkan (+93.98%) | 0.2451 | 0.1264 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=128;hip-wave=32` | 2 | vulkan (+41.85%) | 0.3931 | 0.2771 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=1024;hip-wave=32` | 2 | vulkan (+364.41%) | 2.7339 | 0.5887 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=8192;hip-wave=32` | 2 | vulkan (+205.27%) | 10.8100 | 3.5411 |
| `independent_throughput/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+70.53%) | 87.3720 | 51.2360 |
| `independent_throughput/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+48.30%) | 91.8400 | 61.9280 |
| `independent_throughput/memory-waitcnt/variant=strided4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+91.53%) | 86.6880 | 45.2600 |
| `independent_throughput/memory-waitcnt/variant=strided4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+87.02%) | 95.3360 | 50.9760 |
| `independent_throughput/memory-waitcnt/variant=gather1,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+17.31%) | 360.1960 | 307.0560 |
| `independent_throughput/memory-waitcnt/variant=gather1,n=32768,body=64,wg=256;hip-wave=32` | 4 | vulkan (+34.27%), hip (+5.56%), hipgraph (+0.61%) | 395.2960 | 294.4040 |
| `independent_throughput/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=64;hip-wave=64` | 2 | vulkan (+10.43%) | 77.4800 | 70.1600 |
| `independent_throughput/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=256;hip-wave=64` | 2 | vulkan (+22.43%) | 79.8120 | 65.1880 |
| `independent_throughput/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+92.84%), hip (+25.55%), hipgraph (+4.83%) | 1817.8320 | 942.6600 |
| `independent_throughput/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+20.64%) | 1247.6320 | 1034.1640 |
| `independent_throughput/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+91.01%), hip (+17.81%), hipgraph (+4.92%) | 1854.3600 | 970.8120 |
| `independent_throughput/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+14.71%) | 1168.4800 | 1018.6400 |
| `independent_throughput/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+73.14%), hip (+23.09%), hipgraph (+7.47%) | 1872.6000 | 1081.5400 |
| `independent_throughput/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+14.21%) | 1306.6360 | 1144.0480 |
| `independent_throughput/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+96.18%), hip (+22.10%), hipgraph (+3.83%) | 1837.1840 | 936.4840 |
| `independent_throughput/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+30.65%) | 1284.6400 | 983.2520 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=2,wg=128,body=16;hip-wave=32` | 4 | vulkan (+22.42%), hip (+17.51%), hipgraph (+8.48%) | 54.4120 | 44.4480 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=2,wg=128,body=16;hip-wave=32` | 3 | vulkan (+10.15%), hip (+2.74%) | 45.8000 | 41.5800 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=2,wg=256,body=16;hip-wave=32` | 2 | vulkan (+22.00%) | 14.0200 | 11.4920 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=4,wg=256,body=16;hip-wave=32` | 2 | vulkan (+1.43%) | 9.3600 | 9.2280 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+66.90%) | 250.9200 | 150.3400 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+34.10%) | 253.6560 | 189.1480 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+42.66%) | 244.9560 | 171.7040 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+34.56%) | 247.3760 | 183.8440 |
| `independent_throughput/q6-x8-selected-down/operation=x8_selected_dp4a_dot_prequantized,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+110.67%) | 47.5520 | 22.5720 |
| `independent_throughput/q6-x8-selected-down/operation=x8_selected_dp4a_quantize_plus_dot,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+103.18%) | 58.5240 | 28.8040 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+25.04%) | 5.8920 | 4.7120 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+3.96%) | 9.4520 | 9.0920 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+20.80%) | 22.9960 | 19.0360 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+13.92%) | 26.2600 | 23.0520 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+35.36%) | 21.3760 | 15.7920 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+40.88%) | 28.9360 | 20.5400 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+17.40%) | 23.4760 | 19.9960 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+16.58%) | 17.1840 | 14.7400 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+58.68%) | 40.5400 | 25.5480 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+73.00%) | 82.0040 | 47.4000 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+44.16%) | 82.4040 | 57.1600 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+61.10%) | 97.2320 | 60.3560 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+149.26%) | 188.0120 | 75.4280 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 152 | 88 | 0 | 63.33 | 0.8426 | 240 |
| RL / hipgraph | 206 | 34 | 0 | 85.83 | 0.5465 | 240 |
| RL / hip | 218 | 22 | 0 | 90.83 | 0.5313 | 240 |

## Pinned hipEngine-harness comparison

The `hipengine` profile covers the same benchmark row set as pinned HipEngine `f2c3ad6`: family, operation, shape/sweep axes, repetition count, and serial/independent mode. Those rows deliberately run through Hipfire's existing Radiowave-tuned launch policy; wave size, workgroup geometry, source variant, ABI, and machine code remain optimization variables rather than parity constraints. HipEngine has 212 three-way matched core rows because its HIP independent sampler path rejects 12 rows; its 16 dispatch controls are reported separately. Hipfire executes and correctness-gates all 240 rows across four backends.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine core | RL / vulkan | 192 | 20 | 90.57 | 0.4589 | 212 |
| hipEngine core | RL / hip | 158 | 54 | 74.53 | 0.8343 | 212 |
| hipEngine dispatch | RL / vulkan | 6 | 10 | 37.50 | 1.4331 | 16 |
| hipEngine dispatch | RL / hip | 16 | 0 | 100.00 | 0.2249 | 16 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 218/240 rows and HipGraph in 206/240 while all three select identical per-row hipcc code objects. This run uses the `hipengine_f2c` matrix, `radiowave_tuned` wave policy, and `default` scheduler profile. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 152/240 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. The `hipengine_f2c` profile matches HipEngine's row coverage while intentionally retaining Hipfire/Radiowave's tuned wave, workgroup, and source-variant choices. This artifact records the `radiowave_tuned` wave policy and `default` scheduler profile as controlled HIP compilation/launch factors. Every completion timestamp necessarily proves the measured work finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
