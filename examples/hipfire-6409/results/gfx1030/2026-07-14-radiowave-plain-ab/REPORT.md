# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 92/240 rows (38.33%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 92 | 102 | 22 | 24 | 38.33 | 240 |
| vulkan | 100 | 88 | 5 | 47 | 41.67 | 240 |
| hipgraph | 47 | 36 | 90 | 67 | 19.58 | 240 |
| hip | 1 | 14 | 123 | 102 | 0.42 | 240 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 27 | 54 | 20 | 19 | 120 |
| serial_latency | vulkan | 49 | 26 | 5 | 40 | 120 |
| serial_latency | hipgraph | 44 | 35 | 38 | 3 | 120 |
| serial_latency | hip | 0 | 5 | 57 | 58 | 120 |
| independent_throughput | redline | 65 | 48 | 2 | 5 | 120 |
| independent_throughput | vulkan | 51 | 62 | 0 | 7 | 120 |
| independent_throughput | hipgraph | 3 | 1 | 52 | 64 | 120 |
| independent_throughput | hip | 1 | 9 | 66 | 44 | 120 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/dispatch-grid/sweep=count,count=50,grid=1;hip-wave=32` | 2 | vulkan (+46.29%) | 3.0872 | 2.1104 |
| `serial_latency/dispatch-grid/sweep=count,count=200,grid=1;hip-wave=32` | 2 | vulkan (+36.86%) | 3.0466 | 2.2260 |
| `serial_latency/dispatch-grid/sweep=count,count=941,grid=1;hip-wave=32` | 2 | vulkan (+35.88%) | 3.0372 | 2.2353 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=1;hip-wave=32` | 2 | vulkan (+35.33%) | 3.0373 | 2.2443 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=128;hip-wave=32` | 2 | vulkan (+277.02%) | 8.5003 | 2.2546 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=1024;hip-wave=32` | 2 | vulkan (+214.40%) | 9.2254 | 2.9343 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=8192;hip-wave=32` | 2 | vulkan (+58.99%) | 12.8540 | 8.0846 |
| `serial_latency/geometry/k=512,rows=1,wg=64,body=32;hip-wave=32` | 4 | hipgraph (+39.71%), vulkan (+29.08%), hip (+9.14%) | 39.0280 | 27.9359 |
| `serial_latency/geometry/k=512,rows=1,wg=256,body=32;hip-wave=32` | 4 | vulkan (+23.91%), hipgraph (+15.44%), hip (+1.01%) | 11.9600 | 9.6520 |
| `serial_latency/geometry/k=512,rows=4,wg=256,body=32;hip-wave=32` | 4 | vulkan (+30.57%), hipgraph (+19.45%), hip (+2.57%) | 12.4560 | 9.5400 |
| `serial_latency/geometry/k=2048,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+7.92%), hip (+2.93%) | 79.4560 | 73.6277 |
| `serial_latency/geometry/k=2048,rows=1,wg=256,body=32;hip-wave=32` | 4 | hipgraph (+39.12%), vulkan (+22.56%), hip (+8.64%) | 38.8480 | 27.9239 |
| `serial_latency/geometry/k=2048,rows=4,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+7.98%), hip (+3.08%) | 79.5560 | 73.6758 |
| `serial_latency/geometry/k=2048,rows=4,wg=256,body=32;hip-wave=32` | 3 | hipgraph (+19.30%), vulkan (+5.70%) | 33.3240 | 27.9319 |
| `serial_latency/reduction/variant=lds_tree,k=512,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+21.45%), vulkan (+12.24%) | 33.9280 | 27.9359 |
| `serial_latency/reduction/variant=lds_tree,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+2.82%) | 75.2760 | 73.2118 |
| `serial_latency/reduction/variant=lds_tree,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 3 | hipgraph (+21.74%), vulkan (+6.76%) | 34.0040 | 27.9319 |
| `serial_latency/reduction/variant=extra_barrier,k=512,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+21.51%), vulkan (+12.25%) | 33.9400 | 27.9319 |
| `serial_latency/reduction/variant=extra_barrier,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+2.74%) | 75.2760 | 73.2678 |
| `serial_latency/reduction/variant=extra_barrier,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 3 | hipgraph (+21.81%), vulkan (+6.69%) | 34.0240 | 27.9320 |
| `serial_latency/reduction/variant=wave_shuffle,k=512,rows=1,wg=64,body=32;hip-wave=32` | 4 | hipgraph (+39.77%), vulkan (+29.17%), hip (+9.14%) | 39.0400 | 27.9320 |
| `serial_latency/reduction/variant=wave_shuffle,k=512,rows=1,wg=256,body=32;hip-wave=32` | 4 | vulkan (+24.13%), hipgraph (+15.38%), hip (+1.05%) | 11.9760 | 9.6480 |
| `serial_latency/reduction/variant=wave_shuffle,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+7.95%), hip (+2.92%) | 79.4760 | 73.6198 |
| `serial_latency/reduction/variant=wave_shuffle,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 4 | hipgraph (+41.67%), vulkan (+24.85%), hip (+10.64%) | 39.5720 | 27.9319 |
| `serial_latency/reduction/variant=multi_accum4,k=512,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+21.96%), vulkan (+12.04%) | 34.1680 | 28.0159 |
| `serial_latency/reduction/variant=multi_accum4,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+2.46%) | 83.7320 | 81.7238 |
| `serial_latency/reduction/variant=multi_accum4,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 3 | hipgraph (+17.87%), vulkan (+2.77%) | 34.2760 | 29.0799 |
| `serial_latency/reduction/variant=multi_accum8,k=512,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+18.38%), vulkan (+14.92%) | 34.2640 | 28.9439 |
| `serial_latency/reduction/variant=multi_accum8,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+2.40%) | 87.3680 | 85.3197 |
| `serial_latency/reduction/variant=multi_accum8,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 3 | hipgraph (+14.22%), vulkan (+8.09%) | 34.3760 | 30.0959 |
| `serial_latency/reduction/variant=multi_accum16,k=512,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+7.29%), vulkan (+3.53%) | 35.9120 | 33.4719 |
| `serial_latency/reduction/variant=multi_accum16,k=512,rows=1,wg=256,body=32;hip-wave=32` | 2 | vulkan (+1.15%) | 18.2240 | 18.0160 |
| `serial_latency/reduction/variant=multi_accum16,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+2.49%) | 87.2160 | 85.0957 |
| `serial_latency/reduction/variant=multi_accum16,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 3 | hipgraph (+6.48%), vulkan (+1.34%) | 37.2280 | 34.9639 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+68.87%), hipgraph (+17.50%), hip (+9.54%) | 29.9440 | 17.7320 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+69.34%) | 29.7640 | 17.5760 |
| `serial_latency/memory-waitcnt/variant=strided4,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+66.64%), hipgraph (+17.52%), hip (+8.21%) | 29.5680 | 17.7440 |
| `serial_latency/memory-waitcnt/variant=strided4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+68.39%) | 29.6560 | 17.6120 |
| `serial_latency/memory-waitcnt/variant=gather1,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+7.93%), hipgraph (+5.01%), hip (+0.71%) | 93.1440 | 86.3000 |
| `serial_latency/memory-waitcnt/variant=gather1,n=32768,body=64,wg=256;hip-wave=32` | 3 | vulkan (+12.51%), hipgraph (+1.41%) | 95.9320 | 85.2640 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+38.97%), hipgraph (+17.07%), hip (+8.88%) | 29.7720 | 21.4240 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+37.83%) | 29.7160 | 21.5600 |
| `serial_latency/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 2 | hipgraph (+0.22%) | 482.5960 | 481.5306 |
| `serial_latency/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | hipgraph (+0.35%) | 483.5320 | 481.8346 |
| `serial_latency/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 2 | hipgraph (+0.47%) | 484.3280 | 482.0586 |
| `serial_latency/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | hipgraph (+0.43%) | 485.9480 | 483.8507 |
| `serial_latency/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 2 | hipgraph (+0.49%) | 484.2440 | 481.8786 |
| `serial_latency/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | hipgraph (+0.08%) | 484.2080 | 483.8346 |
| `serial_latency/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 2 | hipgraph (+0.43%) | 482.2640 | 480.1786 |
| `serial_latency/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | hipgraph (+0.44%) | 482.5600 | 480.4227 |
| `serial_latency/vopd/variant=dependent_fma,accums=4,n=65536,body=512,wg=64;hip-wave=32` | 3 | hipgraph (+7.01%), hip (+1.44%) | 70.9360 | 66.2878 |
| `serial_latency/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=64;hip-wave=32` | 4 | hipgraph (+5.76%), hip (+2.57%), vulkan (+1.21%) | 125.5600 | 118.7237 |
| `serial_latency/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=256;hip-wave=32` | 2 | vulkan (+4.94%) | 130.1920 | 124.0680 |
| `serial_latency/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=64;hip-wave=32` | 4 | vulkan (+8.05%), hipgraph (+6.83%), hip (+3.14%) | 108.1720 | 100.1160 |
| `serial_latency/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=256;hip-wave=32` | 2 | vulkan (+11.00%) | 111.0920 | 100.0800 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=1,wg=64;hip-wave=32` | 2 | hipgraph (+2.14%) | 105.1560 | 102.9517 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=1,wg=64;hip-wave=32` | 2 | hipgraph (+0.45%) | 726.1200 | 722.8380 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=1,wg=256;hip-wave=32` | 3 | vulkan (+8.73%), hipgraph (+5.09%) | 36.7000 | 33.7520 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=1,wg=256;hip-wave=32` | 2 | hipgraph (+1.22%) | 197.8640 | 195.4874 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=4,wg=64;hip-wave=32` | 2 | hipgraph (+1.94%) | 106.4080 | 104.3797 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=4,wg=64;hip-wave=32` | 2 | hipgraph (+0.09%) | 735.2000 | 734.5180 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=4,wg=256;hip-wave=32` | 3 | vulkan (+9.34%), hipgraph (+6.45%) | 37.1880 | 34.0120 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=4,wg=256;hip-wave=32` | 2 | hipgraph (+1.25%) | 200.1720 | 197.7074 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=8,wg=64;hip-wave=32` | 2 | hipgraph (+2.33%) | 106.7960 | 104.3638 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=8,wg=64;hip-wave=32` | 2 | hipgraph (+0.30%) | 863.1520 | 860.5736 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=8,wg=256;hip-wave=32` | 3 | vulkan (+10.19%), hipgraph (+6.77%) | 37.5320 | 34.0600 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=8,wg=256;hip-wave=32` | 2 | hipgraph (+0.93%) | 239.1400 | 236.9273 |
| `serial_latency/two-stage-reduction/k=8192,rows=4,splits=4,wg=128,body=16;hip-wave=32` | 2 | hipgraph (+0.27%) | 10.4520 | 10.4240 |
| `serial_latency/two-stage-reduction/k=32768,rows=4,splits=4,wg=128,body=16;hip-wave=32` | 2 | hipgraph (+0.20%) | 15.7200 | 15.6880 |
| `serial_latency/q4-selected-dual/operation=q8_1_quantize,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=32;hip-wave=32` | 4 | vulkan (+393.71%), hipgraph (+69.26%), hip (+38.30%) | 9.7360 | 1.9720 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+21.75%) | 84.6280 | 69.5120 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+13.28%) | 95.0920 | 83.9440 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+51.19%) | 96.1360 | 63.5880 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+29.17%) | 105.0480 | 81.3280 |
| `serial_latency/q6-x8-selected-down/operation=q8_1_quantize,rows=8,experts=256,in=512,out=2048,wg=32;hip-wave=32` | 4 | vulkan (+419.70%), hipgraph (+85.27%), hip (+44.90%) | 9.7080 | 1.8680 |
| `serial_latency/q6-x8-selected-down/operation=x8_selected_dp4a_dot_prequantized,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+367.87%) | 109.2000 | 23.3400 |
| `serial_latency/q6-x8-selected-down/operation=x8_selected_dp4a_quantize_plus_dot,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+241.46%) | 122.6400 | 35.9160 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=768,out=2048,rows=1,wg=32;hip-wave=32` | 4 | vulkan (+269.36%), hipgraph (+34.99%), hip (+6.18%) | 6.9440 | 1.8800 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+139.46%) | 10.5840 | 4.4200 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+10.99%) | 17.4120 | 15.6880 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+101.02%) | 13.4440 | 6.6880 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 3 | vulkan (+33.13%), hipgraph (+8.91%) | 20.3320 | 15.2720 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=768,out=2048,rows=4,wg=32;hip-wave=32` | 4 | vulkan (+417.75%), hipgraph (+84.00%), hip (+45.32%) | 9.5680 | 1.8480 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+57.88%) | 13.1040 | 8.3000 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+13.25%) | 22.5240 | 19.8880 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+81.05%) | 16.8960 | 9.3320 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+53.32%) | 26.2240 | 17.1040 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=2048,out=2048,rows=1,wg=32;hip-wave=32` | 4 | vulkan (+419.21%), hipgraph (+85.06%), hip (+45.89%) | 9.5120 | 1.8320 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+26.41%) | 13.0960 | 10.3600 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+26.58%) | 22.5720 | 17.8320 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+39.31%) | 22.3840 | 16.0680 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+35.90%) | 32.5680 | 23.9640 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=2048,out=2048,rows=4,wg=32;hip-wave=32` | 4 | vulkan (+408.69%), hipgraph (+67.67%), hip (+37.83%) | 9.6040 | 1.8880 |
| `independent_throughput/dispatch-grid/sweep=count,count=941,grid=1;hip-wave=32` | 2 | vulkan (+134.26%) | 0.1648 | 0.0704 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=1;hip-wave=32` | 2 | vulkan (+49.94%) | 0.1688 | 0.1126 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=128;hip-wave=32` | 2 | vulkan (+3196.90%) | 3.7545 | 0.1139 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=1024;hip-wave=32` | 2 | vulkan (+730.04%) | 6.6435 | 0.8004 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=8192;hip-wave=32` | 2 | vulkan (+168.19%) | 16.5169 | 6.1586 |
| `independent_throughput/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+20.44%) | 17.8160 | 14.7920 |
| `independent_throughput/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+19.43%) | 17.1600 | 14.3680 |
| `independent_throughput/memory-waitcnt/variant=strided4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+12.78%) | 17.2200 | 15.2680 |
| `independent_throughput/memory-waitcnt/variant=strided4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+17.95%) | 16.8520 | 14.2880 |
| `independent_throughput/memory-waitcnt/variant=gather1,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+1.39%) | 77.6080 | 76.5440 |
| `independent_throughput/memory-waitcnt/variant=gather1,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+9.09%) | 77.5840 | 71.1160 |
| `independent_throughput/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+5.97%) | 17.8920 | 16.8840 |
| `independent_throughput/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 2 | hipgraph (+11.20%) | 358.2400 | 322.1552 |
| `independent_throughput/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 2 | hipgraph (+3.50%) | 438.0120 | 423.2110 |
| `independent_throughput/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 3 | vulkan (+9.65%), hip (+1.36%) | 431.0400 | 393.0920 |
| `independent_throughput/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 3 | hipgraph (+5.91%), hip (+0.40%) | 436.3040 | 411.9430 |
| `independent_throughput/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | hip (+1.56%) | 431.7200 | 425.0950 |
| `independent_throughput/two-stage-reduction/k=8192,rows=1,splits=4,wg=128,body=16;hip-wave=32` | 2 | vulkan (+18.89%) | 5.9160 | 4.9760 |
| `independent_throughput/two-stage-reduction/k=8192,rows=4,splits=4,wg=128,body=16;hip-wave=32` | 2 | vulkan (+61.23%) | 8.4160 | 5.2200 |
| `independent_throughput/two-stage-reduction/k=8192,rows=4,splits=4,wg=256,body=16;hip-wave=32` | 2 | vulkan (+37.45%) | 7.5760 | 5.5120 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=2,wg=128,body=16;hip-wave=32` | 2 | vulkan (+64.19%) | 19.0920 | 11.6280 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=4,wg=128,body=16;hip-wave=32` | 2 | vulkan (+21.44%) | 11.2160 | 9.2360 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=2,wg=256,body=16;hip-wave=32` | 2 | vulkan (+39.71%) | 11.3720 | 8.1400 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=4,wg=256,body=16;hip-wave=32` | 2 | vulkan (+51.93%) | 7.8760 | 5.1840 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=2,wg=128,body=16;hip-wave=32` | 2 | vulkan (+52.00%) | 19.4680 | 12.8080 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=4,wg=128,body=16;hip-wave=32` | 2 | vulkan (+106.66%) | 13.7800 | 6.6680 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=2,wg=256,body=16;hip-wave=32` | 2 | vulkan (+39.40%) | 11.6480 | 8.3560 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=4,wg=256,body=16;hip-wave=32` | 2 | vulkan (+15.55%) | 10.2560 | 8.8760 |
| `independent_throughput/q4-selected-dual/operation=q8_1_quantize,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=32;hip-wave=32` | 4 | vulkan (+292.62%), hip (+32.15%), hipgraph (+21.14%) | 16.8040 | 4.2800 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+48.61%) | 79.8160 | 53.7080 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+43.53%) | 88.9840 | 61.9960 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+62.46%) | 94.3320 | 58.0640 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+65.06%) | 102.4280 | 62.0560 |
| `independent_throughput/q6-x8-selected-down/operation=q8_1_quantize,rows=8,experts=256,in=512,out=2048,wg=32;hip-wave=32` | 4 | vulkan (+318.07%), hip (+21.49%), hipgraph (+8.89%) | 14.9000 | 3.5640 |
| `independent_throughput/q6-x8-selected-down/operation=x8_selected_dp4a_dot_prequantized,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+1667.11%) | 211.0640 | 11.9440 |
| `independent_throughput/q6-x8-selected-down/operation=x8_selected_dp4a_quantize_plus_dot,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+709.44%) | 114.5200 | 14.1480 |
| `independent_throughput/dense-q8/operation=q8_1_quantize,in=768,out=2048,rows=1,wg=32;hip-wave=32` | 2 | vulkan (+57.55%) | 5.7600 | 3.6560 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+171.38%) | 6.9800 | 2.5720 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+70.64%) | 13.2760 | 7.7800 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+37.47%) | 6.2520 | 4.5480 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+80.69%) | 14.4840 | 8.0160 |
| `independent_throughput/dense-q8/operation=q8_1_quantize,in=768,out=2048,rows=4,wg=32;hip-wave=32` | 4 | vulkan (+513.93%), hip (+20.80%), hipgraph (+9.42%) | 14.6360 | 2.3840 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+54.00%) | 11.0080 | 7.1480 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+114.86%) | 19.7160 | 9.1760 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+14.33%) | 8.8720 | 7.7600 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+67.07%) | 18.1440 | 10.8600 |
| `independent_throughput/dense-q8/operation=q8_1_quantize,in=2048,out=2048,rows=1,wg=32;hip-wave=32` | 4 | vulkan (+248.18%), hip (+20.25%), hipgraph (+9.41%) | 14.5120 | 4.1680 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+56.41%) | 7.3200 | 4.6800 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+45.33%) | 15.7360 | 10.8280 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+19.60%) | 8.6160 | 7.2040 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+193.93%) | 24.9720 | 8.4960 |
| `independent_throughput/dense-q8/operation=q8_1_quantize,in=2048,out=2048,rows=4,wg=32;hip-wave=32` | 4 | vulkan (+357.27%), hip (+30.19%), hipgraph (+19.46%) | 16.3520 | 3.5760 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+10.54%) | 17.7080 | 16.0200 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+44.06%) | 26.1960 | 18.1840 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+78.93%) | 37.7320 | 21.0880 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 124 | 116 | 0 | 51.67 | 0.9703 | 240 |
| RL / hipgraph | 169 | 71 | 0 | 70.42 | 0.6743 | 240 |
| RL / hip | 209 | 31 | 0 | 87.08 | 0.6019 | 240 |

## Pinned hipEngine-harness comparison

The `hipengine` profile covers the same benchmark row set as pinned HipEngine `f2c3ad6`: family, operation, shape/sweep axes, repetition count, and serial/independent mode. Those rows deliberately run through Hipfire's existing Radiowave-tuned launch policy; wave size, workgroup geometry, source variant, ABI, and machine code remain optimization variables rather than parity constraints. HipEngine has 212 three-way matched core rows because its HIP independent sampler path rejects 12 rows; its 16 dispatch controls are reported separately. Hipfire executes and correctness-gates all 240 rows across four backends.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine core | RL / vulkan | 192 | 20 | 90.57 | 0.4589 | 212 |
| hipEngine core | RL / hip | 158 | 54 | 74.53 | 0.8343 | 212 |
| hipEngine dispatch | RL / vulkan | 6 | 10 | 37.50 | 1.4331 | 16 |
| hipEngine dispatch | RL / hip | 16 | 0 | 100.00 | 0.2249 | 16 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 209/240 rows and HipGraph in 169/240 while all three select identical per-row hipcc code objects. This run uses the `hipengine_f2c` matrix, `radiowave_tuned` wave policy, and `default` scheduler profile. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 124/240 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. The `hipengine_f2c` profile matches HipEngine's row coverage while intentionally retaining Hipfire/Radiowave's tuned wave, workgroup, and source-variant choices. This artifact records the `radiowave_tuned` wave policy and `default` scheduler profile as controlled HIP compilation/launch factors. Every completion timestamp necessarily proves the measured work finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
