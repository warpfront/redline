# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 94/240 rows (39.17%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 94 | 105 | 22 | 19 | 39.17 | 240 |
| vulkan | 99 | 89 | 5 | 47 | 41.25 | 240 |
| hipgraph | 46 | 37 | 89 | 68 | 19.17 | 240 |
| hip | 1 | 9 | 124 | 106 | 0.42 | 240 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 28 | 53 | 20 | 19 | 120 |
| serial_latency | vulkan | 49 | 26 | 5 | 40 | 120 |
| serial_latency | hipgraph | 43 | 36 | 39 | 2 | 120 |
| serial_latency | hip | 0 | 5 | 56 | 59 | 120 |
| independent_throughput | redline | 66 | 52 | 2 | 0 | 120 |
| independent_throughput | vulkan | 50 | 63 | 0 | 7 | 120 |
| independent_throughput | hipgraph | 3 | 1 | 50 | 66 | 120 |
| independent_throughput | hip | 1 | 4 | 68 | 47 | 120 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/dispatch-grid/sweep=count,count=50,grid=1;hip-wave=32` | 2 | vulkan (+44.61%) | 3.0496 | 2.1088 |
| `serial_latency/dispatch-grid/sweep=count,count=200,grid=1;hip-wave=32` | 2 | vulkan (+37.07%) | 3.0522 | 2.2268 |
| `serial_latency/dispatch-grid/sweep=count,count=941,grid=1;hip-wave=32` | 2 | vulkan (+36.14%) | 3.0430 | 2.2352 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=1;hip-wave=32` | 2 | vulkan (+35.54%) | 3.0431 | 2.2452 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=128;hip-wave=32` | 2 | vulkan (+276.40%) | 8.4869 | 2.2548 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=1024;hip-wave=32` | 2 | vulkan (+214.51%) | 9.2274 | 2.9339 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=8192;hip-wave=32` | 2 | vulkan (+58.85%) | 12.8429 | 8.0849 |
| `serial_latency/geometry/k=512,rows=1,wg=64,body=32;hip-wave=32` | 4 | hipgraph (+39.63%), vulkan (+28.97%), hip (+9.02%) | 39.0000 | 27.9319 |
| `serial_latency/geometry/k=512,rows=1,wg=256,body=32;hip-wave=32` | 4 | vulkan (+24.08%), hipgraph (+15.36%), hip (+1.08%) | 11.9560 | 9.6360 |
| `serial_latency/geometry/k=512,rows=4,wg=256,body=32;hip-wave=32` | 4 | vulkan (+30.75%), hipgraph (+19.56%), hip (+2.74%) | 12.4680 | 9.5360 |
| `serial_latency/geometry/k=2048,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+7.98%), hip (+3.34%) | 79.4680 | 73.5918 |
| `serial_latency/geometry/k=2048,rows=1,wg=256,body=32;hip-wave=32` | 4 | hipgraph (+41.62%), vulkan (+24.89%), hip (+10.57%) | 39.5560 | 27.9319 |
| `serial_latency/geometry/k=2048,rows=4,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+8.02%), hip (+2.91%) | 79.5640 | 73.6598 |
| `serial_latency/geometry/k=2048,rows=4,wg=256,body=32;hip-wave=32` | 3 | hipgraph (+19.33%), vulkan (+5.82%) | 33.3320 | 27.9319 |
| `serial_latency/reduction/variant=lds_tree,k=512,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+21.45%), vulkan (+12.26%) | 33.9240 | 27.9319 |
| `serial_latency/reduction/variant=lds_tree,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+2.77%) | 75.2240 | 73.1997 |
| `serial_latency/reduction/variant=lds_tree,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 3 | hipgraph (+21.78%), vulkan (+6.86%) | 34.0160 | 27.9319 |
| `serial_latency/reduction/variant=extra_barrier,k=512,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+21.50%), vulkan (+12.31%) | 33.9360 | 27.9319 |
| `serial_latency/reduction/variant=extra_barrier,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+2.93%) | 75.3880 | 73.2398 |
| `serial_latency/reduction/variant=extra_barrier,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 3 | hipgraph (+21.80%), vulkan (+6.77%) | 34.0160 | 27.9278 |
| `serial_latency/reduction/variant=wave_shuffle,k=512,rows=1,wg=64,body=32;hip-wave=32` | 4 | hipgraph (+39.68%), vulkan (+28.95%), hip (+9.12%) | 39.0200 | 27.9359 |
| `serial_latency/reduction/variant=wave_shuffle,k=512,rows=1,wg=256,body=32;hip-wave=32` | 4 | vulkan (+24.18%), hipgraph (+15.42%), hip (+1.15%) | 11.9760 | 9.6440 |
| `serial_latency/reduction/variant=wave_shuffle,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+7.97%), hip (+2.94%) | 79.4640 | 73.5957 |
| `serial_latency/reduction/variant=wave_shuffle,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 4 | hipgraph (+41.52%), vulkan (+24.94%), hip (+10.62%) | 39.5120 | 27.9199 |
| `serial_latency/reduction/variant=multi_accum4,k=512,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+21.97%), vulkan (+12.12%) | 34.1560 | 28.0039 |
| `serial_latency/reduction/variant=multi_accum4,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+2.49%) | 83.5880 | 81.5557 |
| `serial_latency/reduction/variant=multi_accum4,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 3 | hipgraph (+18.03%), vulkan (+3.36%) | 34.2560 | 29.0239 |
| `serial_latency/reduction/variant=multi_accum8,k=512,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+18.57%), vulkan (+15.26%) | 34.2560 | 28.8919 |
| `serial_latency/reduction/variant=multi_accum8,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+2.33%) | 87.1400 | 85.1557 |
| `serial_latency/reduction/variant=multi_accum8,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 3 | hipgraph (+14.36%), vulkan (+8.72%) | 34.3640 | 30.0479 |
| `serial_latency/reduction/variant=multi_accum16,k=512,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+7.23%), vulkan (+3.64%) | 35.8400 | 33.4239 |
| `serial_latency/reduction/variant=multi_accum16,k=512,rows=1,wg=256,body=32;hip-wave=32` | 2 | vulkan (+1.23%) | 18.1760 | 17.9560 |
| `serial_latency/reduction/variant=multi_accum16,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+2.51%) | 87.0040 | 84.8717 |
| `serial_latency/reduction/variant=multi_accum16,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 3 | hipgraph (+6.30%), vulkan (+1.43%) | 37.1120 | 34.9119 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+69.14%), hipgraph (+17.35%), hip (+9.34%) | 29.9240 | 17.6920 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+69.43%) | 29.7520 | 17.5600 |
| `serial_latency/memory-waitcnt/variant=strided4,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+67.07%), hipgraph (+17.58%), hip (+8.26%) | 29.5840 | 17.7080 |
| `serial_latency/memory-waitcnt/variant=strided4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+68.73%) | 29.6560 | 17.5760 |
| `serial_latency/memory-waitcnt/variant=gather1,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+7.59%), hipgraph (+4.73%), hip (+0.29%) | 92.9880 | 86.4280 |
| `serial_latency/memory-waitcnt/variant=gather1,n=32768,body=64,wg=256;hip-wave=32` | 3 | vulkan (+12.00%), hipgraph (+1.15%) | 95.6200 | 85.3720 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+36.08%), hipgraph (+14.63%), hip (+6.66%) | 29.1480 | 21.4200 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+37.88%) | 29.7040 | 21.5440 |
| `serial_latency/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 2 | hipgraph (+0.27%) | 482.8400 | 481.5222 |
| `serial_latency/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | hipgraph (+0.25%) | 483.3080 | 482.1262 |
| `serial_latency/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 2 | hipgraph (+0.42%) | 484.3120 | 482.2822 |
| `serial_latency/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | hipgraph (+0.52%) | 486.1240 | 483.5942 |
| `serial_latency/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 2 | hipgraph (+0.52%) | 484.6040 | 482.0822 |
| `serial_latency/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | hipgraph (+0.29%) | 485.2160 | 483.8102 |
| `serial_latency/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 2 | hipgraph (+0.47%) | 482.8480 | 480.5782 |
| `serial_latency/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | hipgraph (+0.47%) | 482.7600 | 480.5183 |
| `serial_latency/vopd/variant=dependent_fma,accums=4,n=65536,body=512,wg=64;hip-wave=32` | 3 | hipgraph (+7.04%), hip (+1.47%) | 70.9320 | 66.2638 |
| `serial_latency/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=64;hip-wave=32` | 4 | hipgraph (+5.75%), hip (+2.55%), vulkan (+1.17%) | 125.5200 | 118.6916 |
| `serial_latency/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=256;hip-wave=32` | 2 | vulkan (+4.67%) | 129.8080 | 124.0200 |
| `serial_latency/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=64;hip-wave=32` | 4 | vulkan (+8.08%), hipgraph (+6.83%), hip (+3.15%) | 108.1760 | 100.0880 |
| `serial_latency/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=256;hip-wave=32` | 2 | vulkan (+10.82%) | 110.8880 | 100.0640 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=1,wg=64;hip-wave=32` | 2 | hipgraph (+2.07%) | 105.1960 | 103.0636 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=1,wg=64;hip-wave=32` | 2 | hipgraph (+0.24%) | 725.1440 | 723.3973 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=1,wg=256;hip-wave=32` | 3 | vulkan (+8.75%), hipgraph (+7.69%) | 36.6960 | 33.7440 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=1,wg=256;hip-wave=32` | 2 | hipgraph (+1.21%) | 197.7640 | 195.3993 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=4,wg=64;hip-wave=32` | 2 | hipgraph (+2.16%) | 106.1840 | 103.9357 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=4,wg=64;hip-wave=32` | 2 | hipgraph (+0.29%) | 733.3360 | 731.2013 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=4,wg=256;hip-wave=32` | 3 | vulkan (+9.20%), hipgraph (+6.16%) | 37.1320 | 34.0040 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=4,wg=256;hip-wave=32` | 2 | hipgraph (+1.27%) | 200.3400 | 197.8193 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=8,wg=64;hip-wave=32` | 2 | hipgraph (+2.13%) | 106.8960 | 104.6676 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=8,wg=64;hip-wave=32` | 2 | hipgraph (+0.03%) | 862.8800 | 862.5927 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=8,wg=256;hip-wave=32` | 3 | vulkan (+9.92%), hipgraph (+6.51%) | 37.4360 | 34.0560 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=8,wg=256;hip-wave=32` | 2 | hipgraph (+1.13%) | 239.5360 | 236.8591 |
| `serial_latency/two-stage-reduction/k=32768,rows=4,splits=4,wg=128,body=16;hip-wave=32` | 2 | hipgraph (+0.41%) | 15.7440 | 15.6799 |
| `serial_latency/q4-selected-dual/operation=q8_1_quantize,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=32;hip-wave=32` | 4 | vulkan (+393.52%), hipgraph (+69.43%), hip (+38.60%) | 9.7520 | 1.9760 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+22.78%) | 85.2360 | 69.4240 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+12.66%) | 93.3200 | 82.8320 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+50.91%) | 95.9720 | 63.5960 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+28.51%) | 104.0840 | 80.9920 |
| `serial_latency/q6-x8-selected-down/operation=q8_1_quantize,rows=8,experts=256,in=512,out=2048,wg=32;hip-wave=32` | 4 | vulkan (+378.30%), hipgraph (+71.60%), hip (+33.89%) | 8.9920 | 1.8800 |
| `serial_latency/q6-x8-selected-down/operation=x8_selected_dp4a_dot_prequantized,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+364.33%) | 108.2080 | 23.3040 |
| `serial_latency/q6-x8-selected-down/operation=x8_selected_dp4a_quantize_plus_dot,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+209.43%) | 113.7960 | 36.7760 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=768,out=2048,rows=1,wg=32;hip-wave=32` | 4 | vulkan (+262.97%), hipgraph (+34.70%), hip (+6.05%) | 6.9400 | 1.9120 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+135.32%) | 10.5800 | 4.4960 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+11.67%) | 17.4480 | 15.6240 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+100.78%) | 13.4520 | 6.7000 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 3 | vulkan (+32.72%), hipgraph (+9.22%) | 20.3800 | 15.3560 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=768,out=2048,rows=4,wg=32;hip-wave=32` | 4 | vulkan (+417.49%), hipgraph (+84.31%), hip (+45.30%) | 9.5840 | 1.8520 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+51.06%) | 12.5320 | 8.2960 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+13.03%) | 22.4800 | 19.8880 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+81.16%) | 16.9640 | 9.3640 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+53.91%) | 26.2880 | 17.0800 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=2048,out=2048,rows=1,wg=32;hip-wave=32` | 4 | vulkan (+424.18%), hipgraph (+85.17%), hip (+45.70%) | 9.5400 | 1.8200 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+25.79%) | 13.0720 | 10.3920 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+25.74%) | 22.5280 | 17.9160 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+40.62%) | 22.6560 | 16.1120 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+33.08%) | 31.8960 | 23.9680 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=2048,out=2048,rows=4,wg=32;hip-wave=32` | 4 | vulkan (+409.30%), hipgraph (+68.46%), hip (+38.05%) | 9.6360 | 1.8920 |
| `independent_throughput/dispatch-grid/sweep=count,count=941,grid=1;hip-wave=32` | 2 | vulkan (+151.03%) | 0.1654 | 0.0659 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=1;hip-wave=32` | 2 | vulkan (+47.74%) | 0.1694 | 0.1147 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=128;hip-wave=32` | 2 | vulkan (+3038.00%) | 3.7629 | 0.1199 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=1024;hip-wave=32` | 2 | vulkan (+743.79%) | 6.7267 | 0.7972 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=8192;hip-wave=32` | 2 | vulkan (+168.49%) | 16.5320 | 6.1573 |
| `independent_throughput/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+28.07%) | 17.6640 | 13.7920 |
| `independent_throughput/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+48.84%) | 18.2840 | 12.2840 |
| `independent_throughput/memory-waitcnt/variant=strided4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+16.64%) | 16.9040 | 14.4920 |
| `independent_throughput/memory-waitcnt/variant=strided4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+20.75%) | 16.9240 | 14.0160 |
| `independent_throughput/memory-waitcnt/variant=gather1,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+8.24%) | 77.1160 | 71.2440 |
| `independent_throughput/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+6.38%) | 17.8040 | 16.7360 |
| `independent_throughput/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 2 | hipgraph (+12.22%) | 360.1280 | 320.9229 |
| `independent_throughput/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 2 | hipgraph (+3.61%) | 438.0440 | 422.7665 |
| `independent_throughput/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 3 | vulkan (+9.03%), hip (+1.62%) | 431.5320 | 395.8040 |
| `independent_throughput/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 3 | hipgraph (+8.21%), hip (+1.24%) | 436.9360 | 403.7787 |
| `independent_throughput/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | hip (+0.95%) | 431.1400 | 427.0906 |
| `independent_throughput/two-stage-reduction/k=8192,rows=1,splits=4,wg=128,body=16;hip-wave=32` | 2 | vulkan (+26.53%) | 6.0280 | 4.7640 |
| `independent_throughput/two-stage-reduction/k=8192,rows=4,splits=4,wg=128,body=16;hip-wave=32` | 2 | vulkan (+72.64%) | 8.4800 | 4.9120 |
| `independent_throughput/two-stage-reduction/k=8192,rows=4,splits=4,wg=256,body=16;hip-wave=32` | 2 | vulkan (+51.33%) | 7.4880 | 4.9480 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=2,wg=128,body=16;hip-wave=32` | 2 | vulkan (+84.94%) | 19.1080 | 10.3320 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=4,wg=128,body=16;hip-wave=32` | 2 | vulkan (+27.72%) | 11.2600 | 8.8160 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=2,wg=256,body=16;hip-wave=32` | 2 | vulkan (+41.77%) | 11.3360 | 7.9960 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=4,wg=256,body=16;hip-wave=32` | 2 | vulkan (+50.42%) | 7.8040 | 5.1880 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=2,wg=128,body=16;hip-wave=32` | 2 | vulkan (+55.01%) | 19.6680 | 12.6880 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=4,wg=128,body=16;hip-wave=32` | 2 | vulkan (+90.29%) | 13.5640 | 7.1280 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=2,wg=256,body=16;hip-wave=32` | 2 | vulkan (+49.74%) | 11.7400 | 7.8400 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=4,wg=256,body=16;hip-wave=32` | 2 | vulkan (+16.02%) | 10.1680 | 8.7640 |
| `independent_throughput/q4-selected-dual/operation=q8_1_quantize,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=32;hip-wave=32` | 2 | vulkan (+92.23%) | 7.1280 | 3.7080 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+48.47%) | 80.1320 | 53.9720 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+44.19%) | 89.1280 | 61.8120 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+62.48%) | 94.2440 | 58.0040 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+63.94%) | 102.2360 | 62.3600 |
| `independent_throughput/q6-x8-selected-down/operation=q8_1_quantize,rows=8,experts=256,in=512,out=2048,wg=32;hip-wave=32` | 2 | vulkan (+94.46%) | 6.8760 | 3.5360 |
| `independent_throughput/q6-x8-selected-down/operation=x8_selected_dp4a_dot_prequantized,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+1717.75%) | 216.7480 | 11.9240 |
| `independent_throughput/q6-x8-selected-down/operation=x8_selected_dp4a_quantize_plus_dot,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+739.50%) | 115.7840 | 13.7920 |
| `independent_throughput/dense-q8/operation=q8_1_quantize,in=768,out=2048,rows=1,wg=32;hip-wave=32` | 2 | vulkan (+10.71%) | 4.1760 | 3.7720 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+189.77%) | 6.9080 | 2.3840 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+61.77%) | 12.9480 | 8.0040 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+33.42%) | 6.0680 | 4.5480 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+76.27%) | 14.2000 | 8.0560 |
| `independent_throughput/dense-q8/operation=q8_1_quantize,in=768,out=2048,rows=4,wg=32;hip-wave=32` | 2 | vulkan (+203.92%) | 6.8200 | 2.2440 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+57.54%) | 11.2800 | 7.1600 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+112.22%) | 19.0320 | 8.9680 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+18.69%) | 8.9160 | 7.5120 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+74.19%) | 18.3600 | 10.5400 |
| `independent_throughput/dense-q8/operation=q8_1_quantize,in=2048,out=2048,rows=1,wg=32;hip-wave=32` | 2 | vulkan (+75.85%) | 6.7880 | 3.8600 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+41.33%) | 7.0440 | 4.9840 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+49.05%) | 15.3640 | 10.3080 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+24.16%) | 8.5720 | 6.9040 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+192.31%) | 24.9520 | 8.5360 |
| `independent_throughput/dense-q8/operation=q8_1_quantize,in=2048,out=2048,rows=4,wg=32;hip-wave=32` | 2 | vulkan (+84.62%) | 6.8160 | 3.6920 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+10.06%) | 17.5560 | 15.9520 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+42.49%) | 26.0360 | 18.2720 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+66.19%) | 37.7440 | 22.7120 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 125 | 115 | 0 | 52.08 | 0.9718 | 240 |
| RL / hipgraph | 175 | 65 | 0 | 72.92 | 0.6323 | 240 |
| RL / hip | 214 | 26 | 0 | 89.17 | 0.5871 | 240 |

## Pinned hipEngine-harness comparison

The `hipengine` profile covers the same benchmark row set as pinned HipEngine `f2c3ad6`: family, operation, shape/sweep axes, repetition count, and serial/independent mode. Those rows deliberately run through Hipfire's existing Radiowave-tuned launch policy; wave size, workgroup geometry, source variant, ABI, and machine code remain optimization variables rather than parity constraints. HipEngine has 212 three-way matched core rows because its HIP independent sampler path rejects 12 rows; its 16 dispatch controls are reported separately. Hipfire executes and correctness-gates all 240 rows across four backends.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine core | RL / vulkan | 192 | 20 | 90.57 | 0.4589 | 212 |
| hipEngine core | RL / hip | 158 | 54 | 74.53 | 0.8343 | 212 |
| hipEngine dispatch | RL / vulkan | 6 | 10 | 37.50 | 1.4331 | 16 |
| hipEngine dispatch | RL / hip | 16 | 0 | 100.00 | 0.2249 | 16 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 214/240 rows and HipGraph in 175/240 while all three select identical per-row hipcc code objects. This run uses the `hipengine_f2c` matrix, `radiowave_tuned` wave policy, and `default` scheduler profile. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 125/240 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. The `hipengine_f2c` profile matches HipEngine's row coverage while intentionally retaining Hipfire/Radiowave's tuned wave, workgroup, and source-variant choices. This artifact records the `radiowave_tuned` wave policy and `default` scheduler profile as controlled HIP compilation/launch factors. Every completion timestamp necessarily proves the measured work finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
