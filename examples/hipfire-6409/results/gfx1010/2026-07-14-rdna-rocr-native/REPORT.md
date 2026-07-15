# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 129/240 rows (53.75%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 129 | 81 | 18 | 12 | 53.75 | 240 |
| vulkan | 86 | 96 | 2 | 56 | 35.83 | 240 |
| hipgraph | 25 | 41 | 86 | 88 | 10.42 | 240 |
| hip | 0 | 22 | 134 | 84 | 0.00 | 240 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 54 | 44 | 17 | 5 | 120 |
| serial_latency | vulkan | 41 | 21 | 2 | 56 | 120 |
| serial_latency | hipgraph | 25 | 41 | 49 | 5 | 120 |
| serial_latency | hip | 0 | 14 | 52 | 54 | 120 |
| independent_throughput | redline | 75 | 37 | 1 | 7 | 120 |
| independent_throughput | vulkan | 45 | 75 | 0 | 0 | 120 |
| independent_throughput | hipgraph | 0 | 0 | 37 | 83 | 120 |
| independent_throughput | hip | 0 | 8 | 82 | 30 | 120 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/dispatch-grid/sweep=count,count=1,grid=1;hip-wave=32` | 3 | vulkan (+128.03%), hip (+11.00%) | 21.8000 | 9.5600 |
| `serial_latency/dispatch-grid/sweep=count,count=50,grid=1;hip-wave=32` | 2 | vulkan (+8.94%) | 3.2368 | 2.9712 |
| `serial_latency/dispatch-grid/sweep=count,count=200,grid=1;hip-wave=32` | 2 | vulkan (+6.64%) | 3.1260 | 2.9314 |
| `serial_latency/dispatch-grid/sweep=count,count=941,grid=1;hip-wave=32` | 2 | vulkan (+7.29%) | 3.1586 | 2.9439 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=1;hip-wave=32` | 2 | vulkan (+8.80%) | 3.1587 | 2.9032 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=128;hip-wave=32` | 2 | vulkan (+13.70%) | 3.3995 | 2.9899 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=1024;hip-wave=32` | 2 | vulkan (+10.50%) | 5.8849 | 5.3257 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=8192;hip-wave=32` | 2 | vulkan (+153.57%) | 59.3687 | 23.4129 |
| `serial_latency/geometry/k=512,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+5.50%), hip (+0.55%) | 66.7720 | 63.2917 |
| `serial_latency/geometry/k=512,rows=1,wg=256,body=32;hip-wave=32` | 3 | hipgraph (+6.96%), hip (+2.91%) | 15.1280 | 14.1439 |
| `serial_latency/geometry/k=512,rows=4,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+5.70%), hip (+0.08%) | 66.5680 | 62.9757 |
| `serial_latency/geometry/k=512,rows=4,wg=256,body=32;hip-wave=32` | 3 | hipgraph (+6.71%), hip (+1.50%) | 15.1440 | 14.1920 |
| `serial_latency/geometry/k=2048,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+2.65%), hip (+0.08%) | 141.2880 | 137.6434 |
| `serial_latency/geometry/k=2048,rows=1,wg=256,body=32;hip-wave=32` | 3 | hipgraph (+5.19%), hip (+0.17%) | 67.5040 | 64.1757 |
| `serial_latency/geometry/k=2048,rows=4,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+2.67%), hip (+0.16%) | 141.1520 | 137.4793 |
| `serial_latency/geometry/k=2048,rows=4,wg=256,body=32;hip-wave=32` | 2 | hipgraph (+4.70%) | 67.5360 | 64.5037 |
| `serial_latency/reduction/variant=lds_tree,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+0.50%) | 137.3360 | 136.6514 |
| `serial_latency/reduction/variant=extra_barrier,k=512,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+0.09%) | 62.9520 | 62.8957 |
| `serial_latency/reduction/variant=extra_barrier,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+0.65%) | 137.8520 | 136.9634 |
| `serial_latency/reduction/variant=wave_shuffle,k=512,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+6.10%), hip (+0.68%) | 66.9840 | 63.1317 |
| `serial_latency/reduction/variant=wave_shuffle,k=512,rows=1,wg=256,body=32;hip-wave=32` | 3 | hipgraph (+7.78%), hip (+3.07%) | 15.1800 | 14.0839 |
| `serial_latency/reduction/variant=wave_shuffle,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+2.88%), hip (+0.34%) | 141.2880 | 137.3354 |
| `serial_latency/reduction/variant=wave_shuffle,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 3 | hipgraph (+4.79%), hip (+0.45%) | 67.4280 | 64.3477 |
| `serial_latency/reduction/variant=multi_accum8,k=512,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+0.43%) | 66.6120 | 66.3237 |
| `serial_latency/reduction/variant=multi_accum8,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+0.42%) | 149.7240 | 149.1033 |
| `serial_latency/reduction/variant=multi_accum16,k=512,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+1.54%) | 74.1000 | 72.9757 |
| `serial_latency/reduction/variant=multi_accum16,k=512,rows=1,wg=256,body=32;hip-wave=32` | 2 | hipgraph (+1.31%) | 59.1920 | 58.4278 |
| `serial_latency/reduction/variant=multi_accum16,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+0.05%) | 150.0760 | 150.0033 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+12.63%), hipgraph (+1.57%), hip (+0.36%) | 88.0080 | 78.1360 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+10.43%) | 86.5960 | 78.4200 |
| `serial_latency/memory-waitcnt/variant=strided4,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+12.97%), hipgraph (+4.27%), hip (+0.31%) | 88.1120 | 77.9960 |
| `serial_latency/memory-waitcnt/variant=strided4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+11.63%) | 86.9880 | 77.9280 |
| `serial_latency/memory-waitcnt/variant=gather1,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+1.22%) | 362.0520 | 357.6840 |
| `serial_latency/memory-waitcnt/variant=gather1,n=32768,body=64,wg=256;hip-wave=32` | 4 | vulkan (+1.92%), hipgraph (+1.07%), hip (+0.53%) | 368.0720 | 361.1440 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=64;hip-wave=32` | 3 | vulkan (+11.06%), hipgraph (+2.83%) | 86.9440 | 78.2840 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+11.10%) | 87.7520 | 78.9840 |
| `serial_latency/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 3 | vulkan (+67.07%), hipgraph (+0.11%) | 1497.8960 | 896.5920 |
| `serial_latency/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 3 | vulkan (+59.37%), hipgraph (+0.61%) | 1434.0880 | 899.8720 |
| `serial_latency/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+68.67%), hip (+0.58%), hipgraph (+0.19%) | 1508.6640 | 894.4320 |
| `serial_latency/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+59.80%) | 1430.6320 | 895.2640 |
| `serial_latency/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 3 | vulkan (+59.56%), hipgraph (+0.38%) | 1505.9960 | 943.8440 |
| `serial_latency/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 3 | vulkan (+52.34%), hipgraph (+0.01%) | 1441.4760 | 946.2160 |
| `serial_latency/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+68.49%) | 1503.0560 | 892.0480 |
| `serial_latency/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 4 | vulkan (+62.75%), hipgraph (+0.73%), hip (+0.48%) | 1454.2080 | 893.5000 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=1,wg=256;hip-wave=32` | 2 | hipgraph (+0.19%) | 73.0320 | 72.8917 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=1,wg=256;hip-wave=32` | 2 | hipgraph (+0.09%) | 292.1480 | 291.8826 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=4,wg=64;hip-wave=32` | 2 | hipgraph (+0.07%) | 1170.1600 | 1169.3026 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=4,wg=256;hip-wave=32` | 2 | hipgraph (+0.56%) | 329.8880 | 328.0625 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=8,wg=256;hip-wave=32` | 2 | vulkan (+33.24%) | 73.2760 | 54.9960 |
| `serial_latency/two-stage-reduction/k=32768,rows=1,splits=4,wg=128,body=16;hip-wave=32` | 2 | hipgraph (+2.06%) | 18.2640 | 17.8959 |
| `serial_latency/q4-selected-dual/operation=q8_1_quantize,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=32;hip-wave=32` | 2 | vulkan (+120.25%) | 5.0040 | 2.2720 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+8.68%) | 258.6840 | 238.0160 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+3.47%) | 257.9840 | 249.3240 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+18.10%) | 248.8120 | 210.6760 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+12.86%) | 251.3720 | 222.7200 |
| `serial_latency/q6-x8-selected-down/operation=q8_1_quantize,rows=8,experts=256,in=512,out=2048,wg=32;hip-wave=32` | 2 | vulkan (+132.33%) | 4.9440 | 2.1280 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=768,out=2048,rows=1,wg=32;hip-wave=32` | 2 | vulkan (+120.47%) | 4.9120 | 2.2280 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+5.13%) | 7.9480 | 7.5600 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=768,out=2048,rows=4,wg=32;hip-wave=32` | 2 | vulkan (+132.89%) | 4.9280 | 2.1160 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+6.78%) | 23.8840 | 22.3680 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+34.08%) | 35.6160 | 26.5640 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+8.47%) | 38.5120 | 35.5040 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=2048,out=2048,rows=1,wg=32;hip-wave=32` | 2 | vulkan (+131.48%) | 4.8240 | 2.0840 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=2048,out=2048,rows=4,wg=32;hip-wave=32` | 2 | vulkan (+127.04%) | 4.9040 | 2.1600 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+12.73%) | 85.6280 | 75.9600 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+1.79%) | 89.3760 | 87.8080 |
| `independent_throughput/dispatch-grid/sweep=count,count=1,grid=1;hip-wave=32` | 3 | vulkan (+123.08%), hip (+19.72%) | 20.8800 | 9.3600 |
| `independent_throughput/dispatch-grid/sweep=count,count=941,grid=1;hip-wave=32` | 2 | vulkan (+67.56%) | 0.1306 | 0.0780 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=1;hip-wave=32` | 2 | vulkan (+3.79%) | 0.1305 | 0.1258 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=128;hip-wave=32` | 2 | vulkan (+851.05%) | 2.8206 | 0.2966 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=1024;hip-wave=32` | 2 | vulkan (+161.87%) | 5.5021 | 2.1011 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=8192;hip-wave=32` | 2 | vulkan (+89.85%) | 30.4048 | 16.0149 |
| `independent_throughput/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+66.68%) | 88.6080 | 53.1600 |
| `independent_throughput/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+100.16%) | 94.1960 | 47.0600 |
| `independent_throughput/memory-waitcnt/variant=strided4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+98.61%) | 85.1560 | 42.8760 |
| `independent_throughput/memory-waitcnt/variant=strided4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+73.68%) | 94.3720 | 54.3360 |
| `independent_throughput/memory-waitcnt/variant=gather1,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+32.30%) | 360.0200 | 272.1240 |
| `independent_throughput/memory-waitcnt/variant=gather1,n=32768,body=64,wg=256;hip-wave=32` | 4 | vulkan (+43.25%), hip (+5.75%), hipgraph (+1.01%) | 396.6240 | 276.8720 |
| `independent_throughput/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+31.49%) | 90.9160 | 69.1440 |
| `independent_throughput/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+49.74%) | 94.4400 | 63.0680 |
| `independent_throughput/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+97.14%), hip (+15.96%), hipgraph (+6.20%) | 1840.4240 | 933.5520 |
| `independent_throughput/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+16.97%) | 1205.9440 | 1030.9480 |
| `independent_throughput/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+87.70%), hip (+23.86%), hipgraph (+4.10%) | 1810.5720 | 964.6240 |
| `independent_throughput/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+18.57%) | 1192.0480 | 1005.3480 |
| `independent_throughput/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+62.51%), hip (+16.83%), hipgraph (+3.14%) | 1792.5800 | 1103.0320 |
| `independent_throughput/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+9.96%) | 1239.7320 | 1127.4480 |
| `independent_throughput/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+96.78%), hip (+18.04%), hipgraph (+5.49%) | 1845.3440 | 937.7600 |
| `independent_throughput/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+26.38%) | 1278.9800 | 1011.9960 |
| `independent_throughput/two-stage-reduction/k=8192,rows=1,splits=4,wg=128,body=16;hip-wave=32` | 2 | vulkan (+20.18%) | 6.8840 | 5.7280 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=2,wg=128,body=16;hip-wave=32` | 4 | vulkan (+24.67%), hip (+19.51%), hipgraph (+8.86%) | 54.3800 | 43.6200 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=2,wg=128,body=16;hip-wave=32` | 4 | vulkan (+20.15%), hip (+14.00%), hipgraph (+2.54%) | 50.7840 | 42.2680 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=4,wg=128,body=16;hip-wave=32` | 2 | vulkan (+1.10%) | 13.9840 | 13.8320 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=2,wg=256,body=16;hip-wave=32` | 2 | vulkan (+20.70%) | 13.9920 | 11.5920 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+64.08%) | 250.2080 | 152.4960 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+35.42%) | 254.1160 | 187.6440 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+49.70%) | 244.6960 | 163.4600 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+40.37%) | 247.1720 | 176.0920 |
| `independent_throughput/q6-x8-selected-down/operation=x8_selected_dp4a_dot_prequantized,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+58.74%) | 46.6320 | 29.3760 |
| `independent_throughput/q6-x8-selected-down/operation=x8_selected_dp4a_quantize_plus_dot,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+94.75%) | 58.8000 | 30.1920 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+12.13%) | 21.9600 | 19.5840 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+18.04%) | 24.8680 | 21.0680 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+21.59%) | 22.0960 | 18.1720 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+25.33%) | 29.0360 | 23.1680 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+30.43%) | 16.3240 | 12.5160 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+15.91%) | 25.8160 | 22.2720 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+16.55%) | 17.6040 | 15.1040 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+70.37%) | 41.2560 | 24.2160 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+74.94%) | 79.5120 | 45.4520 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+71.11%) | 81.2640 | 47.4920 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+31.09%) | 100.6960 | 76.8160 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+154.33%) | 187.4080 | 73.6880 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 154 | 86 | 0 | 64.17 | 0.8556 | 240 |
| RL / hipgraph | 198 | 42 | 0 | 82.50 | 0.5293 | 240 |
| RL / hip | 215 | 25 | 0 | 89.58 | 0.5207 | 240 |

## Pinned hipEngine-harness comparison

The `hipengine` profile covers the same benchmark row set as pinned HipEngine `f2c3ad6`: family, operation, shape/sweep axes, repetition count, and serial/independent mode. Those rows deliberately run through Hipfire's existing Radiowave-tuned launch policy; wave size, workgroup geometry, source variant, ABI, and machine code remain optimization variables rather than parity constraints. HipEngine has 212 three-way matched core rows because its HIP independent sampler path rejects 12 rows; its 16 dispatch controls are reported separately. Hipfire executes and correctness-gates all 240 rows across four backends.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine core | RL / vulkan | 192 | 20 | 90.57 | 0.4589 | 212 |
| hipEngine core | RL / hip | 158 | 54 | 74.53 | 0.8343 | 212 |
| hipEngine dispatch | RL / vulkan | 6 | 10 | 37.50 | 1.4331 | 16 |
| hipEngine dispatch | RL / hip | 16 | 0 | 100.00 | 0.2249 | 16 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 215/240 rows and HipGraph in 198/240 while all three select identical per-row hipcc code objects. This run uses the `hipengine_f2c` matrix, `radiowave_tuned` wave policy, and `default` scheduler profile. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 154/240 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. The `hipengine_f2c` profile matches HipEngine's row coverage while intentionally retaining Hipfire/Radiowave's tuned wave, workgroup, and source-variant choices. This artifact records the `radiowave_tuned` wave policy and `default` scheduler profile as controlled HIP compilation/launch factors. Every completion timestamp necessarily proves the measured work finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
