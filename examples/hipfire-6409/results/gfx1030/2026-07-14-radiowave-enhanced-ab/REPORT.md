# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 93/240 rows (38.75%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 93 | 106 | 21 | 20 | 38.75 | 240 |
| vulkan | 99 | 89 | 5 | 47 | 41.25 | 240 |
| hipgraph | 47 | 36 | 88 | 69 | 19.58 | 240 |
| hip | 1 | 9 | 126 | 104 | 0.42 | 240 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 27 | 54 | 20 | 19 | 120 |
| serial_latency | vulkan | 49 | 26 | 5 | 40 | 120 |
| serial_latency | hipgraph | 44 | 35 | 39 | 2 | 120 |
| serial_latency | hip | 0 | 5 | 56 | 59 | 120 |
| independent_throughput | redline | 66 | 52 | 1 | 1 | 120 |
| independent_throughput | vulkan | 50 | 63 | 0 | 7 | 120 |
| independent_throughput | hipgraph | 3 | 1 | 49 | 67 | 120 |
| independent_throughput | hip | 1 | 4 | 70 | 45 | 120 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/dispatch-grid/sweep=count,count=50,grid=1;hip-wave=32` | 2 | vulkan (+46.25%) | 3.0888 | 2.1120 |
| `serial_latency/dispatch-grid/sweep=count,count=200,grid=1;hip-wave=32` | 2 | vulkan (+36.79%) | 3.0490 | 2.2290 |
| `serial_latency/dispatch-grid/sweep=count,count=941,grid=1;hip-wave=32` | 2 | vulkan (+35.88%) | 3.0392 | 2.2368 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=1;hip-wave=32` | 2 | vulkan (+35.26%) | 3.0387 | 2.2465 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=128;hip-wave=32` | 2 | vulkan (+276.83%) | 8.5033 | 2.2565 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=1024;hip-wave=32` | 2 | vulkan (+214.04%) | 9.2262 | 2.9379 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=8192;hip-wave=32` | 2 | vulkan (+64.79%) | 13.3631 | 8.1092 |
| `serial_latency/geometry/k=512,rows=1,wg=64,body=32;hip-wave=32` | 4 | hipgraph (+39.30%), vulkan (+28.74%), hip (+8.83%) | 38.9160 | 27.9359 |
| `serial_latency/geometry/k=512,rows=1,wg=256,body=32;hip-wave=32` | 4 | vulkan (+24.64%), hipgraph (+16.07%), hip (+1.62%) | 12.0200 | 9.6440 |
| `serial_latency/geometry/k=512,rows=4,wg=256,body=32;hip-wave=32` | 4 | vulkan (+30.82%), hipgraph (+19.68%), hip (+2.97%) | 12.4800 | 9.5400 |
| `serial_latency/geometry/k=2048,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+7.96%), hip (+2.94%) | 79.4920 | 73.6318 |
| `serial_latency/geometry/k=2048,rows=1,wg=256,body=32;hip-wave=32` | 4 | hipgraph (+41.64%), vulkan (+24.81%), hip (+10.60%) | 39.5640 | 27.9319 |
| `serial_latency/geometry/k=2048,rows=4,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+8.02%), hip (+3.03%) | 79.6160 | 73.7039 |
| `serial_latency/geometry/k=2048,rows=4,wg=256,body=32;hip-wave=32` | 3 | hipgraph (+21.97%), vulkan (+8.08%) | 34.0640 | 27.9279 |
| `serial_latency/reduction/variant=lds_tree,k=512,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+21.45%), vulkan (+12.24%) | 33.9240 | 27.9320 |
| `serial_latency/reduction/variant=lds_tree,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+2.81%) | 75.2640 | 73.2038 |
| `serial_latency/reduction/variant=lds_tree,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 3 | hipgraph (+21.75%), vulkan (+6.76%) | 34.0080 | 27.9320 |
| `serial_latency/reduction/variant=extra_barrier,k=512,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+21.50%), vulkan (+12.28%) | 33.9480 | 27.9399 |
| `serial_latency/reduction/variant=extra_barrier,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+2.81%) | 75.3200 | 73.2639 |
| `serial_latency/reduction/variant=extra_barrier,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 3 | hipgraph (+21.81%), vulkan (+6.71%) | 34.0240 | 27.9319 |
| `serial_latency/reduction/variant=wave_shuffle,k=512,rows=1,wg=64,body=32;hip-wave=32` | 4 | hipgraph (+39.72%), vulkan (+28.99%), hip (+9.14%) | 39.0320 | 27.9359 |
| `serial_latency/reduction/variant=wave_shuffle,k=512,rows=1,wg=256,body=32;hip-wave=32` | 4 | vulkan (+23.93%), hipgraph (+15.06%), hip (+0.98%) | 11.9520 | 9.6440 |
| `serial_latency/reduction/variant=wave_shuffle,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+7.94%), hip (+2.94%) | 79.4600 | 73.6158 |
| `serial_latency/reduction/variant=wave_shuffle,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 4 | hipgraph (+41.52%), vulkan (+24.65%), hip (+10.56%) | 39.5280 | 27.9320 |
| `serial_latency/reduction/variant=multi_accum4,k=512,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+21.96%), vulkan (+12.06%) | 34.1640 | 28.0120 |
| `serial_latency/reduction/variant=multi_accum4,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+2.48%) | 83.7440 | 81.7198 |
| `serial_latency/reduction/variant=multi_accum4,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 3 | hipgraph (+17.82%), vulkan (+2.72%) | 34.2720 | 29.0879 |
| `serial_latency/reduction/variant=multi_accum8,k=512,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+18.44%), vulkan (+14.93%) | 34.2800 | 28.9439 |
| `serial_latency/reduction/variant=multi_accum8,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+2.43%) | 87.3960 | 85.3199 |
| `serial_latency/reduction/variant=multi_accum8,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 3 | hipgraph (+14.26%), vulkan (+8.18%) | 34.3880 | 30.0960 |
| `serial_latency/reduction/variant=multi_accum16,k=512,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+7.31%), vulkan (+3.54%) | 35.9160 | 33.4679 |
| `serial_latency/reduction/variant=multi_accum16,k=512,rows=1,wg=256,body=32;hip-wave=32` | 2 | vulkan (+1.38%) | 18.2200 | 17.9720 |
| `serial_latency/reduction/variant=multi_accum16,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+2.51%) | 87.2440 | 85.1078 |
| `serial_latency/reduction/variant=multi_accum16,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 3 | hipgraph (+6.51%), vulkan (+1.32%) | 37.2320 | 34.9559 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+69.29%), hipgraph (+17.59%), hip (+9.59%) | 29.9440 | 17.6880 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+69.50%) | 29.7640 | 17.5600 |
| `serial_latency/memory-waitcnt/variant=strided4,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+67.07%), hipgraph (+17.40%), hip (+8.29%) | 29.5840 | 17.7080 |
| `serial_latency/memory-waitcnt/variant=strided4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+68.42%) | 29.6560 | 17.6080 |
| `serial_latency/memory-waitcnt/variant=gather1,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+7.82%), hipgraph (+4.15%), hip (+0.30%) | 92.7360 | 86.0080 |
| `serial_latency/memory-waitcnt/variant=gather1,n=32768,body=64,wg=256;hip-wave=32` | 3 | vulkan (+12.03%), hipgraph (+0.91%) | 95.6000 | 85.3320 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+38.89%), hipgraph (+17.21%), hip (+8.99%) | 29.7720 | 21.4360 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+38.12%) | 29.7280 | 21.5240 |
| `serial_latency/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 2 | hipgraph (+0.43%) | 483.4640 | 481.4072 |
| `serial_latency/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | hipgraph (+0.41%) | 483.5160 | 481.5432 |
| `serial_latency/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 2 | hipgraph (+0.45%) | 484.5080 | 482.3431 |
| `serial_latency/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | hipgraph (+0.28%) | 485.3800 | 484.0031 |
| `serial_latency/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 2 | hipgraph (+0.46%) | 484.5240 | 482.3151 |
| `serial_latency/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | hipgraph (+0.32%) | 485.4680 | 483.9071 |
| `serial_latency/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 2 | hipgraph (+0.54%) | 483.0080 | 480.3992 |
| `serial_latency/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | hipgraph (+0.29%) | 482.2160 | 480.8191 |
| `serial_latency/vopd/variant=dependent_fma,accums=4,n=65536,body=512,wg=64;hip-wave=32` | 3 | hipgraph (+6.98%), hip (+1.46%) | 71.0440 | 66.4079 |
| `serial_latency/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=64;hip-wave=32` | 4 | hipgraph (+5.18%), hip (+2.07%), vulkan (+0.68%) | 125.1240 | 118.9638 |
| `serial_latency/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=256;hip-wave=32` | 2 | vulkan (+4.97%) | 130.4600 | 124.2840 |
| `serial_latency/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=64;hip-wave=32` | 4 | vulkan (+8.03%), hipgraph (+6.83%), hip (+3.12%) | 108.3680 | 100.3160 |
| `serial_latency/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=256;hip-wave=32` | 2 | vulkan (+11.04%) | 111.3640 | 100.2920 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=1,wg=64;hip-wave=32` | 2 | hipgraph (+1.85%) | 105.2240 | 103.3158 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=1,wg=64;hip-wave=32` | 2 | hipgraph (+0.29%) | 727.0040 | 724.9068 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=1,wg=256;hip-wave=32` | 3 | vulkan (+8.79%), hipgraph (+5.26%) | 36.7400 | 33.7720 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=1,wg=256;hip-wave=32` | 2 | hipgraph (+1.05%) | 198.2120 | 196.1517 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=4,wg=64;hip-wave=32` | 2 | hipgraph (+2.12%) | 106.4560 | 104.2438 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=4,wg=64;hip-wave=32` | 2 | hipgraph (+0.26%) | 735.9200 | 733.9948 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=4,wg=256;hip-wave=32` | 3 | vulkan (+9.10%), hipgraph (+6.27%) | 37.1560 | 34.0560 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=4,wg=256;hip-wave=32` | 2 | hipgraph (+1.25%) | 200.6920 | 198.2077 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=8,wg=64;hip-wave=32` | 2 | hipgraph (+2.66%) | 107.0680 | 104.2919 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=8,wg=64;hip-wave=32` | 2 | hipgraph (+0.41%) | 864.3600 | 860.7944 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=8,wg=256;hip-wave=32` | 3 | vulkan (+10.26%), hipgraph (+7.07%) | 37.5840 | 34.0880 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=8,wg=256;hip-wave=32` | 2 | hipgraph (+0.81%) | 239.1440 | 237.2196 |
| `serial_latency/two-stage-reduction/k=8192,rows=4,splits=4,wg=128,body=16;hip-wave=32` | 2 | hipgraph (+0.46%) | 10.4560 | 10.4080 |
| `serial_latency/two-stage-reduction/k=32768,rows=4,splits=4,wg=128,body=16;hip-wave=32` | 2 | hipgraph (+0.28%) | 15.7520 | 15.7080 |
| `serial_latency/q4-selected-dual/operation=q8_1_quantize,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=32;hip-wave=32` | 4 | vulkan (+390.91%), hipgraph (+68.98%), hip (+37.52%) | 9.7200 | 1.9800 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+22.43%) | 85.2040 | 69.5960 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+13.86%) | 94.9440 | 83.3840 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+51.01%) | 96.1240 | 63.6560 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+28.96%) | 105.0520 | 81.4600 |
| `serial_latency/q6-x8-selected-down/operation=q8_1_quantize,rows=8,experts=256,in=512,out=2048,wg=32;hip-wave=32` | 4 | vulkan (+377.49%), hipgraph (+71.29%), hip (+33.87%) | 8.9960 | 1.8840 |
| `serial_latency/q6-x8-selected-down/operation=x8_selected_dp4a_dot_prequantized,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+362.32%) | 108.7200 | 23.5160 |
| `serial_latency/q6-x8-selected-down/operation=x8_selected_dp4a_quantize_plus_dot,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+218.73%) | 117.2160 | 36.7760 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=768,out=2048,rows=1,wg=32;hip-wave=32` | 4 | vulkan (+265.61%), hipgraph (+34.45%), hip (+5.48%) | 6.9320 | 1.8960 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+135.17%) | 10.5640 | 4.4920 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+11.22%) | 17.4040 | 15.6480 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+84.50%) | 13.4760 | 7.3040 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 3 | vulkan (+32.28%), hipgraph (+8.77%) | 20.3400 | 15.3760 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=768,out=2048,rows=4,wg=32;hip-wave=32` | 4 | vulkan (+412.18%), hipgraph (+84.10%), hip (+45.36%) | 9.5880 | 1.8720 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+59.99%) | 13.3240 | 8.3280 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+11.43%) | 21.9240 | 19.6760 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+80.97%) | 17.0040 | 9.3960 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+54.33%) | 26.4280 | 17.1240 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=2048,out=2048,rows=1,wg=32;hip-wave=32` | 4 | vulkan (+415.95%), hipgraph (+85.73%), hip (+46.15%) | 9.5760 | 1.8560 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+27.54%) | 13.1160 | 10.2840 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+25.97%) | 22.5680 | 17.9160 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+38.76%) | 22.3520 | 16.1080 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+34.77%) | 32.4040 | 24.0440 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=2048,out=2048,rows=4,wg=32;hip-wave=32` | 4 | vulkan (+403.35%), hipgraph (+69.08%), hip (+38.12%) | 9.6240 | 1.9120 |
| `independent_throughput/dispatch-grid/sweep=count,count=941,grid=1;hip-wave=32` | 2 | vulkan (+170.69%) | 0.1660 | 0.0613 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=1;hip-wave=32` | 2 | vulkan (+45.24%) | 0.1700 | 0.1171 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=128;hip-wave=32` | 2 | vulkan (+2915.56%) | 3.5597 | 0.1180 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=1024;hip-wave=32` | 2 | vulkan (+730.89%) | 6.6380 | 0.7989 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=8192;hip-wave=32` | 2 | vulkan (+168.36%) | 16.5248 | 6.1577 |
| `independent_throughput/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+19.87%) | 17.5680 | 14.6560 |
| `independent_throughput/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+41.74%) | 18.1600 | 12.8120 |
| `independent_throughput/memory-waitcnt/variant=strided4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+15.85%) | 17.0440 | 14.7120 |
| `independent_throughput/memory-waitcnt/variant=strided4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+22.41%) | 16.8480 | 13.7640 |
| `independent_throughput/memory-waitcnt/variant=gather1,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+0.26%) | 77.0400 | 76.8440 |
| `independent_throughput/memory-waitcnt/variant=gather1,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+9.22%) | 77.2240 | 70.7040 |
| `independent_throughput/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+8.31%) | 17.8800 | 16.5080 |
| `independent_throughput/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 2 | hipgraph (+14.69%) | 370.9320 | 323.4235 |
| `independent_throughput/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 2 | hipgraph (+4.51%) | 438.6280 | 419.7193 |
| `independent_throughput/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 4 | vulkan (+9.42%), hip (+2.15%), hipgraph (+0.10%) | 432.2040 | 394.9840 |
| `independent_throughput/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 3 | hipgraph (+5.49%), hip (+0.80%) | 436.7920 | 414.0473 |
| `independent_throughput/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | hip (+2.60%) | 431.7640 | 420.8273 |
| `independent_throughput/two-stage-reduction/k=8192,rows=4,splits=4,wg=128,body=16;hip-wave=32` | 2 | vulkan (+70.32%) | 8.4480 | 4.9600 |
| `independent_throughput/two-stage-reduction/k=8192,rows=4,splits=4,wg=256,body=16;hip-wave=32` | 2 | vulkan (+56.21%) | 7.5920 | 4.8600 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=2,wg=128,body=16;hip-wave=32` | 2 | vulkan (+65.90%) | 19.1320 | 11.5320 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=4,wg=128,body=16;hip-wave=32` | 2 | vulkan (+25.41%) | 11.2520 | 8.9720 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=2,wg=256,body=16;hip-wave=32` | 2 | vulkan (+35.98%) | 11.3240 | 8.3280 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=4,wg=256,body=16;hip-wave=32` | 2 | vulkan (+40.43%) | 7.8640 | 5.6000 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=2,wg=128,body=16;hip-wave=32` | 2 | vulkan (+52.23%) | 19.5280 | 12.8280 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=4,wg=128,body=16;hip-wave=32` | 2 | vulkan (+82.41%) | 13.7320 | 7.5280 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=2,wg=256,body=16;hip-wave=32` | 2 | vulkan (+50.93%) | 11.6520 | 7.7200 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=4,wg=256,body=16;hip-wave=32` | 2 | vulkan (+17.98%) | 10.2080 | 8.6520 |
| `independent_throughput/q4-selected-dual/operation=q8_1_quantize,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=32;hip-wave=32` | 2 | vulkan (+101.64%) | 7.8640 | 3.9000 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+48.25%) | 79.8840 | 53.8840 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+43.55%) | 89.2960 | 62.2040 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+62.78%) | 94.3400 | 57.9560 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+63.04%) | 102.5400 | 62.8920 |
| `independent_throughput/q6-x8-selected-down/operation=q8_1_quantize,rows=8,experts=256,in=512,out=2048,wg=32;hip-wave=32` | 2 | vulkan (+91.19%) | 7.0280 | 3.6760 |
| `independent_throughput/q6-x8-selected-down/operation=x8_selected_dp4a_dot_prequantized,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+1596.96%) | 207.5040 | 12.2280 |
| `independent_throughput/q6-x8-selected-down/operation=x8_selected_dp4a_quantize_plus_dot,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+838.48%) | 115.2080 | 12.2760 |
| `independent_throughput/dense-q8/operation=q8_1_quantize,in=768,out=2048,rows=1,wg=32;hip-wave=32` | 2 | vulkan (+19.98%) | 4.2280 | 3.5240 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+191.33%) | 6.8520 | 2.3520 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+51.68%) | 12.4440 | 8.2040 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+45.45%) | 6.2720 | 4.3120 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+74.00%) | 14.0520 | 8.0760 |
| `independent_throughput/dense-q8/operation=q8_1_quantize,in=768,out=2048,rows=4,wg=32;hip-wave=32` | 2 | vulkan (+178.00%) | 6.7720 | 2.4360 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+54.51%) | 11.1120 | 7.1920 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+104.55%) | 19.4240 | 9.4960 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+15.22%) | 8.8400 | 7.6720 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+67.08%) | 18.2320 | 10.9120 |
| `independent_throughput/dense-q8/operation=q8_1_quantize,in=2048,out=2048,rows=1,wg=32;hip-wave=32` | 2 | vulkan (+58.67%) | 6.7720 | 4.2680 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+44.79%) | 7.2280 | 4.9920 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+47.99%) | 15.9240 | 10.7600 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+20.71%) | 8.6480 | 7.1640 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+176.79%) | 24.4680 | 8.8400 |
| `independent_throughput/dense-q8/operation=q8_1_quantize,in=2048,out=2048,rows=4,wg=32;hip-wave=32` | 2 | vulkan (+83.21%) | 6.7200 | 3.6680 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+10.75%) | 17.7200 | 16.0000 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+40.26%) | 26.4080 | 18.8280 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+74.39%) | 38.4000 | 22.0200 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 125 | 115 | 0 | 52.08 | 0.9732 | 240 |
| RL / hipgraph | 173 | 67 | 0 | 72.08 | 0.6334 | 240 |
| RL / hip | 214 | 26 | 0 | 89.17 | 0.5894 | 240 |

## Pinned hipEngine-harness comparison

The `hipengine` profile covers the same benchmark row set as pinned HipEngine `f2c3ad6`: family, operation, shape/sweep axes, repetition count, and serial/independent mode. Those rows deliberately run through Hipfire's existing Radiowave-tuned launch policy; wave size, workgroup geometry, source variant, ABI, and machine code remain optimization variables rather than parity constraints. HipEngine has 212 three-way matched core rows because its HIP independent sampler path rejects 12 rows; its 16 dispatch controls are reported separately. Hipfire executes and correctness-gates all 240 rows across four backends.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine core | RL / vulkan | 192 | 20 | 90.57 | 0.4589 | 212 |
| hipEngine core | RL / hip | 158 | 54 | 74.53 | 0.8343 | 212 |
| hipEngine dispatch | RL / vulkan | 6 | 10 | 37.50 | 1.4331 | 16 |
| hipEngine dispatch | RL / hip | 16 | 0 | 100.00 | 0.2249 | 16 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 214/240 rows and HipGraph in 173/240 while all three select identical per-row hipcc code objects. This run uses the `hipengine_f2c` matrix, `radiowave_tuned` wave policy, and `default` scheduler profile. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 125/240 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. The `hipengine_f2c` profile matches HipEngine's row coverage while intentionally retaining Hipfire/Radiowave's tuned wave, workgroup, and source-variant choices. This artifact records the `radiowave_tuned` wave policy and `default` scheduler profile as controlled HIP compilation/launch factors. Every completion timestamp necessarily proves the measured work finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
