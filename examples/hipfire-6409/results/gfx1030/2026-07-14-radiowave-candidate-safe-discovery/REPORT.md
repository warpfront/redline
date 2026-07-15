# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 107/240 rows (44.58%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 107 | 99 | 18 | 16 | 44.58 | 240 |
| vulkan | 92 | 88 | 12 | 48 | 38.33 | 240 |
| hipgraph | 39 | 42 | 89 | 70 | 16.25 | 240 |
| hip | 2 | 11 | 121 | 106 | 0.83 | 240 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 34 | 55 | 16 | 15 | 120 |
| serial_latency | vulkan | 50 | 19 | 11 | 40 | 120 |
| serial_latency | hipgraph | 36 | 38 | 42 | 4 | 120 |
| serial_latency | hip | 0 | 8 | 51 | 61 | 120 |
| independent_throughput | redline | 73 | 44 | 2 | 1 | 120 |
| independent_throughput | vulkan | 42 | 69 | 1 | 8 | 120 |
| independent_throughput | hipgraph | 3 | 4 | 47 | 66 | 120 |
| independent_throughput | hip | 2 | 3 | 70 | 45 | 120 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/dispatch-grid/sweep=count,count=50,grid=1;hip-wave=32` | 2 | vulkan (+46.06%) | 3.0848 | 2.1120 |
| `serial_latency/dispatch-grid/sweep=count,count=200,grid=1;hip-wave=32` | 2 | vulkan (+36.98%) | 3.0432 | 2.2216 |
| `serial_latency/dispatch-grid/sweep=count,count=941,grid=1;hip-wave=32` | 2 | vulkan (+35.91%) | 3.0343 | 2.2325 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=1;hip-wave=32` | 2 | vulkan (+33.73%) | 2.9961 | 2.2405 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=128;hip-wave=32` | 4 | vulkan (+256.02%), hipgraph (+83.69%), hip (+52.71%) | 8.0132 | 2.2508 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=1024;hip-wave=32` | 3 | vulkan (+254.29%), hipgraph (+22.58%) | 8.5343 | 2.4088 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=8192;hip-wave=32` | 2 | vulkan (+93.80%) | 8.1149 | 4.1873 |
| `serial_latency/geometry/k=512,rows=1,wg=64,body=32;hip-wave=32` | 4 | hipgraph (+24.01%), vulkan (+12.45%), hip (+1.42%) | 34.4960 | 27.8160 |
| `serial_latency/geometry/k=512,rows=1,wg=256,body=32;hip-wave=32` | 3 | vulkan (+20.22%), hipgraph (+12.66%) | 11.5360 | 9.5960 |
| `serial_latency/geometry/k=512,rows=4,wg=256,body=32;hip-wave=32` | 4 | vulkan (+25.72%), hipgraph (+16.08%), hip (+0.68%) | 11.9280 | 9.4880 |
| `serial_latency/geometry/k=2048,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+6.92%), hip (+2.71%) | 78.3120 | 73.2440 |
| `serial_latency/geometry/k=2048,rows=1,wg=256,body=32;hip-wave=32` | 4 | hipgraph (+34.26%), vulkan (+18.38%), hip (+12.63%) | 37.3560 | 27.8240 |
| `serial_latency/geometry/k=2048,rows=4,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+6.86%), hip (+2.53%) | 78.3240 | 73.2960 |
| `serial_latency/reduction/variant=lds_tree,k=512,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+16.84%), vulkan (+8.36%) | 32.5000 | 27.8160 |
| `serial_latency/reduction/variant=lds_tree,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+2.34%) | 74.0800 | 72.3840 |
| `serial_latency/reduction/variant=lds_tree,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 2 | hipgraph (+12.41%) | 31.2640 | 27.8120 |
| `serial_latency/reduction/variant=extra_barrier,k=512,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+12.11%), vulkan (+3.81%) | 31.1840 | 27.8160 |
| `serial_latency/reduction/variant=extra_barrier,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+1.67%) | 74.1080 | 72.8880 |
| `serial_latency/reduction/variant=extra_barrier,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 2 | hipgraph (+10.07%) | 30.6080 | 27.8080 |
| `serial_latency/reduction/variant=wave_shuffle,k=512,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+12.61%), vulkan (+5.37%) | 32.3280 | 28.7080 |
| `serial_latency/reduction/variant=wave_shuffle,k=512,rows=1,wg=256,body=32;hip-wave=32` | 3 | vulkan (+20.47%), hipgraph (+12.93%) | 11.5600 | 9.5960 |
| `serial_latency/reduction/variant=wave_shuffle,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+6.07%), hip (+2.33%) | 77.6880 | 73.2400 |
| `serial_latency/reduction/variant=wave_shuffle,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 4 | hipgraph (+24.31%), vulkan (+8.00%), hip (+7.68%) | 34.5680 | 27.8080 |
| `serial_latency/reduction/variant=multi_accum4,k=512,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+5.47%) | 29.4040 | 27.8800 |
| `serial_latency/reduction/variant=multi_accum4,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+1.61%) | 82.6200 | 81.3080 |
| `serial_latency/reduction/variant=multi_accum4,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 2 | hipgraph (+7.08%) | 30.9120 | 28.8680 |
| `serial_latency/reduction/variant=multi_accum16,k=512,rows=1,wg=256,body=32;hip-wave=32` | 2 | vulkan (+0.31%) | 17.9520 | 17.8960 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+31.41%) | 23.2600 | 17.7000 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+64.48%) | 28.8560 | 17.5440 |
| `serial_latency/memory-waitcnt/variant=strided4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+62.01%) | 28.7800 | 17.7640 |
| `serial_latency/memory-waitcnt/variant=strided4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+66.29%) | 29.2600 | 17.5960 |
| `serial_latency/memory-waitcnt/variant=gather1,n=32768,body=64,wg=64;hip-wave=32` | 3 | vulkan (+5.55%), hipgraph (+3.59%) | 91.8080 | 86.9840 |
| `serial_latency/memory-waitcnt/variant=gather1,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+10.94%) | 94.1800 | 84.8960 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=64;hip-wave=64` | 4 | vulkan (+58.24%), hipgraph (+32.60%), hip (+14.10%) | 33.8880 | 21.4160 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=256;hip-wave=64` | 2 | vulkan (+12.68%) | 23.4200 | 20.7840 |
| `serial_latency/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 3 | hipgraph (+0.37%), hip (+0.24%) | 484.1360 | 482.3720 |
| `serial_latency/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | hipgraph (+0.10%) | 484.4600 | 483.9679 |
| `serial_latency/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 2 | hipgraph (+0.65%) | 483.4440 | 480.3039 |
| `serial_latency/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | hipgraph (+0.35%) | 482.3960 | 480.7000 |
| `serial_latency/vopd/variant=independent_fma,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 3 | hipgraph (+10.49%), hip (+5.22%) | 71.8120 | 64.9960 |
| `serial_latency/vopd/variant=independent_fma,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 2 | vulkan (+1.51%) | 73.7880 | 72.6920 |
| `serial_latency/vopd/variant=dependent_fma,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 4 | hipgraph (+19.51%), vulkan (+13.72%), hip (+13.04%) | 82.7760 | 69.2640 |
| `serial_latency/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 4 | vulkan (+5.51%), hipgraph (+3.58%), hip (+1.67%) | 130.0720 | 123.2800 |
| `serial_latency/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 2 | vulkan (+8.27%) | 133.9800 | 123.7440 |
| `serial_latency/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 4 | vulkan (+7.45%), hipgraph (+6.88%), hip (+3.49%) | 107.5080 | 100.0560 |
| `serial_latency/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 2 | vulkan (+4.53%) | 104.0640 | 99.5520 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=1,wg=64;hip-wave=32` | 2 | hipgraph (+1.57%) | 104.5920 | 102.9760 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=4,wg=256;hip-wave=32` | 3 | vulkan (+8.02%), hipgraph (+6.87%) | 36.6920 | 33.9680 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=4,wg=256;hip-wave=32` | 2 | hipgraph (+0.77%) | 199.3200 | 197.8040 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=8,wg=64;hip-wave=32` | 2 | hipgraph (+2.62%) | 106.4360 | 103.7200 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=8,wg=64;hip-wave=32` | 2 | hipgraph (+0.28%) | 858.8240 | 856.4198 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=8,wg=256;hip-wave=32` | 3 | vulkan (+10.21%), hipgraph (+6.90%) | 37.3560 | 33.8960 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=8,wg=256;hip-wave=32` | 3 | hipgraph (+5.76%), hip (+4.29%) | 250.5880 | 236.9360 |
| `serial_latency/two-stage-reduction/k=8192,rows=1,splits=2,wg=128,body=16;hip-wave=32` | 2 | hipgraph (+11.60%) | 19.4720 | 17.4480 |
| `serial_latency/two-stage-reduction/k=8192,rows=1,splits=4,wg=128,body=16;hip-wave=32` | 2 | hipgraph (+6.57%) | 17.7880 | 16.6920 |
| `serial_latency/two-stage-reduction/k=8192,rows=1,splits=2,wg=256,body=16;hip-wave=32` | 2 | hipgraph (+5.93%) | 17.6520 | 16.6640 |
| `serial_latency/two-stage-reduction/k=8192,rows=1,splits=4,wg=256,body=16;hip-wave=32` | 2 | hipgraph (+3.31%) | 16.8400 | 16.3000 |
| `serial_latency/two-stage-reduction/k=8192,rows=4,splits=2,wg=128,body=16;hip-wave=32` | 2 | hipgraph (+12.87%) | 19.7840 | 17.5280 |
| `serial_latency/two-stage-reduction/k=8192,rows=4,splits=4,wg=128,body=16;hip-wave=32` | 2 | hipgraph (+10.37%) | 18.3960 | 16.6680 |
| `serial_latency/two-stage-reduction/k=32768,rows=4,splits=4,wg=128,body=16;hip-wave=32` | 2 | hipgraph (+19.37%) | 23.8400 | 19.9720 |
| `serial_latency/two-stage-reduction/k=32768,rows=4,splits=2,wg=256,body=16;hip-wave=32` | 2 | hipgraph (+15.55%) | 23.4840 | 20.3240 |
| `serial_latency/two-stage-reduction/k=32768,rows=4,splits=4,wg=256,body=16;hip-wave=32` | 2 | hipgraph (+11.43%) | 20.1600 | 18.0920 |
| `serial_latency/q4-selected-dual/operation=q8_1_quantize,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=32;hip-wave=32` | 4 | vulkan (+592.39%), hipgraph (+172.74%), hip (+87.04%) | 22.5720 | 3.2600 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+19.68%) | 83.5240 | 69.7920 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+13.78%) | 91.9320 | 80.7960 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+57.53%) | 100.1560 | 63.5800 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+5.70%) | 101.7840 | 96.2920 |
| `serial_latency/q6-x8-selected-down/operation=q8_1_quantize,rows=8,experts=256,in=512,out=2048,wg=32;hip-wave=32` | 4 | vulkan (+396.30%), hipgraph (+77.80%), hip (+39.98%) | 9.1320 | 1.8400 |
| `serial_latency/q6-x8-selected-down/operation=x8_selected_dp4a_dot_prequantized,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+336.34%) | 101.0040 | 23.1480 |
| `serial_latency/q6-x8-selected-down/operation=x8_selected_dp4a_quantize_plus_dot,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+200.24%) | 108.9760 | 36.2960 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=768,out=2048,rows=1,wg=32;hip-wave=32` | 4 | vulkan (+251.97%), hipgraph (+28.34%), hip (+1.19%) | 6.4480 | 1.8320 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+129.92%) | 9.9600 | 4.3320 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+12.37%) | 17.1160 | 15.2320 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+91.22%) | 12.8040 | 6.6960 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 3 | vulkan (+23.88%), hipgraph (+0.28%) | 18.4240 | 14.8720 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=768,out=2048,rows=4,wg=32;hip-wave=32` | 4 | vulkan (+396.90%), hipgraph (+76.60%), hip (+40.15%) | 8.9640 | 1.8040 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+57.61%) | 13.2960 | 8.4360 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+12.42%) | 22.1200 | 19.6760 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+48.73%) | 16.1040 | 10.8280 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+52.83%) | 25.9440 | 16.9760 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=2048,out=2048,rows=1,wg=32;hip-wave=32` | 4 | vulkan (+388.52%), hipgraph (+74.39%), hip (+38.05%) | 8.8520 | 1.8120 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+21.26%) | 12.4320 | 10.2520 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+20.89%) | 21.3440 | 17.6560 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+33.86%) | 21.5520 | 16.1000 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+31.47%) | 31.4120 | 23.8920 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=2048,out=2048,rows=4,wg=32;hip-wave=32` | 4 | vulkan (+370.23%), hipgraph (+59.76%), hip (+31.25%) | 8.9720 | 1.9080 |
| `independent_throughput/dispatch-grid/sweep=count,count=941,grid=1;hip-wave=32` | 2 | vulkan (+101.70%) | 0.1567 | 0.0777 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=1;hip-wave=32` | 2 | vulkan (+30.34%) | 0.1569 | 0.1203 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=128;hip-wave=32` | 4 | vulkan (+3805.01%), hipgraph (+112.85%), hip (+39.05%) | 4.8703 | 0.1247 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=1024;hip-wave=32` | 2 | vulkan (+1535.49%) | 3.9690 | 0.2427 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=8192;hip-wave=32` | 2 | vulkan (+361.58%) | 6.5512 | 1.4193 |
| `independent_throughput/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+31.30%) | 17.6200 | 13.4200 |
| `independent_throughput/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+19.86%) | 17.4760 | 14.5800 |
| `independent_throughput/memory-waitcnt/variant=strided4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+28.97%) | 17.1680 | 13.3120 |
| `independent_throughput/memory-waitcnt/variant=strided4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+32.28%) | 16.7520 | 12.6640 |
| `independent_throughput/memory-waitcnt/variant=gather1,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+3.22%) | 77.3360 | 74.9200 |
| `independent_throughput/memory-waitcnt/variant=gather1,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+1.40%) | 77.5680 | 76.4960 |
| `independent_throughput/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 3 | hipgraph (+11.58%), vulkan (+7.04%) | 359.7360 | 322.4038 |
| `independent_throughput/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 2 | hipgraph (+1.74%) | 438.1520 | 430.6718 |
| `independent_throughput/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | hip (+0.90%) | 430.9920 | 427.1318 |
| `independent_throughput/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 3 | hipgraph (+5.62%), hip (+0.98%) | 436.7400 | 413.4918 |
| `independent_throughput/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | hip (+1.30%) | 431.6480 | 426.1038 |
| `independent_throughput/two-stage-reduction/k=8192,rows=4,splits=4,wg=256,body=16;hip-wave=32` | 2 | vulkan (+15.65%) | 7.2120 | 6.2360 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=2,wg=128,body=16;hip-wave=32` | 2 | vulkan (+73.64%) | 18.3920 | 10.5920 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=4,wg=128,body=16;hip-wave=32` | 2 | vulkan (+27.44%) | 11.1640 | 8.7600 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=2,wg=128,body=16;hip-wave=32` | 2 | vulkan (+66.20%) | 23.7000 | 14.2600 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=4,wg=128,body=16;hip-wave=32` | 2 | vulkan (+21.94%) | 22.2520 | 18.2480 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=4,wg=256,body=16;hip-wave=32` | 2 | vulkan (+21.08%) | 9.9240 | 8.1960 |
| `independent_throughput/q4-selected-dual/operation=q8_1_quantize,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=32;hip-wave=32` | 2 | vulkan (+96.98%) | 6.7920 | 3.4480 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+48.87%) | 80.2920 | 53.9360 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+42.02%) | 84.2040 | 59.2920 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+68.30%) | 101.1360 | 60.0920 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+56.17%) | 101.4520 | 64.9640 |
| `independent_throughput/q6-x8-selected-down/operation=q8_1_quantize,rows=8,experts=256,in=512,out=2048,wg=32;hip-wave=32` | 2 | vulkan (+99.46%) | 17.8480 | 8.9480 |
| `independent_throughput/q6-x8-selected-down/operation=x8_selected_dp4a_dot_prequantized,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+2030.86%) | 257.4080 | 12.0800 |
| `independent_throughput/q6-x8-selected-down/operation=x8_selected_dp4a_quantize_plus_dot,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+679.61%) | 115.3200 | 14.7920 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+54.10%) | 13.1480 | 8.5320 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+183.29%) | 13.2920 | 4.6920 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+71.59%) | 31.9640 | 18.6280 |
| `independent_throughput/dense-q8/operation=q8_1_quantize,in=768,out=2048,rows=4,wg=32;hip-wave=32` | 2 | vulkan (+114.07%) | 17.8280 | 8.3280 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+55.79%) | 11.4600 | 7.3560 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+63.46%) | 19.3800 | 11.8560 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+27.23%) | 9.2320 | 7.2560 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+79.92%) | 18.1720 | 10.1000 |
| `independent_throughput/dense-q8/operation=q8_1_quantize,in=2048,out=2048,rows=1,wg=32;hip-wave=32` | 2 | vulkan (+75.00%) | 6.6360 | 3.7920 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+14.69%) | 7.3720 | 6.4280 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+44.67%) | 15.9200 | 11.0040 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+17.41%) | 8.5240 | 7.2600 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+32.01%) | 24.5600 | 18.6040 |
| `independent_throughput/dense-q8/operation=q8_1_quantize,in=2048,out=2048,rows=4,wg=32;hip-wave=32` | 2 | vulkan (+89.44%) | 17.8680 | 9.4320 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+20.51%) | 19.7160 | 16.3600 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+46.60%) | 47.7040 | 32.5400 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+72.03%) | 37.1720 | 21.6080 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 140 | 100 | 0 | 58.33 | 0.9344 | 240 |
| RL / hipgraph | 182 | 58 | 0 | 75.83 | 0.6752 | 240 |
| RL / hip | 215 | 25 | 0 | 89.58 | 0.6117 | 240 |

## Pinned hipEngine-harness comparison

The `hipengine` profile covers the same benchmark row set as pinned HipEngine `f2c3ad6`: family, operation, shape/sweep axes, repetition count, and serial/independent mode. Those rows deliberately run through Hipfire's existing Radiowave-tuned launch policy; wave size, workgroup geometry, source variant, ABI, and machine code remain optimization variables rather than parity constraints. HipEngine has 212 three-way matched core rows because its HIP independent sampler path rejects 12 rows; its 16 dispatch controls are reported separately. Hipfire executes and correctness-gates all 240 rows across four backends.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine core | RL / vulkan | 192 | 20 | 90.57 | 0.4589 | 212 |
| hipEngine core | RL / hip | 158 | 54 | 74.53 | 0.8343 | 212 |
| hipEngine dispatch | RL / vulkan | 6 | 10 | 37.50 | 1.4331 | 16 |
| hipEngine dispatch | RL / hip | 16 | 0 | 100.00 | 0.2249 | 16 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 215/240 rows and HipGraph in 182/240 while all three select identical per-row hipcc code objects. This run uses the `hipengine_f2c` matrix, `radiowave_tuned` wave policy, and `default` scheduler profile. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 140/240 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. The `hipengine_f2c` profile matches HipEngine's row coverage while intentionally retaining Hipfire/Radiowave's tuned wave, workgroup, and source-variant choices. This artifact records the `radiowave_tuned` wave policy and `default` scheduler profile as controlled HIP compilation/launch factors. Every completion timestamp necessarily proves the measured work finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
