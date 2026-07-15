# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 103/234 rows (44.02%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 103 | 89 | 26 | 16 | 44.02 | 234 |
| vulkan | 89 | 89 | 9 | 47 | 38.03 | 234 |
| hipgraph | 38 | 42 | 85 | 69 | 16.24 | 234 |
| hip | 4 | 14 | 114 | 102 | 1.71 | 234 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 34 | 45 | 23 | 15 | 117 |
| serial_latency | vulkan | 46 | 22 | 9 | 40 | 117 |
| serial_latency | hipgraph | 35 | 40 | 41 | 1 | 117 |
| serial_latency | hip | 2 | 10 | 44 | 61 | 117 |
| independent_throughput | redline | 69 | 44 | 3 | 1 | 117 |
| independent_throughput | vulkan | 43 | 67 | 0 | 7 | 117 |
| independent_throughput | hipgraph | 3 | 2 | 44 | 68 | 117 |
| independent_throughput | hip | 2 | 4 | 70 | 41 | 117 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/dispatch-grid/sweep=count,count=50,grid=1;hip-wave=32` | 2 | vulkan (+46.12%) | 3.0848 | 2.1112 |
| `serial_latency/dispatch-grid/sweep=count,count=200,grid=1;hip-wave=32` | 2 | vulkan (+37.12%) | 3.0460 | 2.2214 |
| `serial_latency/dispatch-grid/sweep=count,count=941,grid=1;hip-wave=32` | 2 | vulkan (+36.12%) | 3.0387 | 2.2324 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=1;hip-wave=32` | 2 | vulkan (+35.12%) | 3.0289 | 2.2416 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=128;hip-wave=32` | 4 | vulkan (+256.26%), hipgraph (+83.87%), hip (+52.93%) | 8.0205 | 2.2513 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=1024;hip-wave=32` | 3 | vulkan (+254.42%), hipgraph (+24.80%) | 8.5353 | 2.4082 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=8192;hip-wave=32` | 2 | vulkan (+105.16%) | 8.6049 | 4.1942 |
| `serial_latency/geometry/k=512,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+10.24%), vulkan (+2.05%) | 30.6680 | 27.8200 |
| `serial_latency/geometry/k=512,rows=1,wg=256,body=32;hip-wave=32` | 3 | vulkan (+20.22%), hipgraph (+12.79%) | 11.5360 | 9.5960 |
| `serial_latency/geometry/k=512,rows=4,wg=256,body=32;hip-wave=32` | 4 | vulkan (+25.51%), hipgraph (+15.93%), hip (+0.88%) | 11.9080 | 9.4880 |
| `serial_latency/geometry/k=2048,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+7.04%), hip (+2.44%) | 77.9880 | 72.8600 |
| `serial_latency/geometry/k=2048,rows=1,wg=256,body=32;hip-wave=32` | 4 | hipgraph (+34.31%), vulkan (+18.36%), hip (+13.01%) | 37.3600 | 27.8160 |
| `serial_latency/geometry/k=2048,rows=4,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+6.40%), hip (+2.36%) | 77.6480 | 72.9760 |
| `serial_latency/reduction/variant=lds_tree,k=512,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+14.74%), vulkan (+6.43%) | 31.9160 | 27.8160 |
| `serial_latency/reduction/variant=lds_tree,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+1.44%) | 73.8320 | 72.7841 |
| `serial_latency/reduction/variant=lds_tree,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 3 | hipgraph (+14.72%), vulkan (+0.55%) | 31.9000 | 27.8080 |
| `serial_latency/reduction/variant=extra_barrier,k=512,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+14.49%), vulkan (+6.13%) | 31.8520 | 27.8200 |
| `serial_latency/reduction/variant=extra_barrier,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+2.29%) | 74.0960 | 72.4361 |
| `serial_latency/reduction/variant=extra_barrier,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 2 | hipgraph (+9.04%) | 31.2800 | 28.6880 |
| `serial_latency/reduction/variant=wave_shuffle,k=512,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+18.72%), vulkan (+9.95%) | 33.0320 | 27.8240 |
| `serial_latency/reduction/variant=wave_shuffle,k=512,rows=1,wg=256,body=32;hip-wave=32` | 3 | vulkan (+19.28%), hipgraph (+11.88%) | 11.4560 | 9.6040 |
| `serial_latency/reduction/variant=wave_shuffle,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+5.95%), hip (+1.66%) | 77.3840 | 73.0400 |
| `serial_latency/reduction/variant=wave_shuffle,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 4 | hipgraph (+26.84%), vulkan (+11.73%), hip (+5.14%) | 35.2680 | 27.8041 |
| `serial_latency/reduction/variant=multi_accum4,k=512,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+10.26%), vulkan (+1.35%) | 30.7360 | 27.8760 |
| `serial_latency/reduction/variant=multi_accum4,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+0.80%) | 81.4680 | 80.8240 |
| `serial_latency/reduction/variant=multi_accum4,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 2 | hipgraph (+1.79%) | 30.1720 | 29.6401 |
| `serial_latency/reduction/variant=multi_accum8,k=512,rows=1,wg=64,body=32;hip-wave=32` | 4 | hipgraph (+11.95%), vulkan (+8.47%), hip (+4.85%) | 32.1600 | 28.7280 |
| `serial_latency/reduction/variant=multi_accum16,k=512,rows=1,wg=256,body=32;hip-wave=32` | 2 | vulkan (+0.04%) | 17.9080 | 17.9000 |
| `serial_latency/reduction/variant=multi_accum16,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+0.21%) | 84.3640 | 84.1881 |
| `serial_latency/reduction/variant=multi_accum16,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 2 | hipgraph (+3.53%) | 35.4720 | 34.2640 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+33.36%) | 23.6000 | 17.6960 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+47.78%) | 25.9320 | 17.5480 |
| `serial_latency/memory-waitcnt/variant=strided4,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+62.42%), hipgraph (+14.55%), hip (+5.01%) | 28.7800 | 17.7200 |
| `serial_latency/memory-waitcnt/variant=strided4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+63.69%) | 28.8360 | 17.6160 |
| `serial_latency/memory-waitcnt/variant=gather1,n=32768,body=64,wg=64;hip-wave=32` | 3 | vulkan (+6.30%), hipgraph (+3.65%) | 92.0480 | 86.5960 |
| `serial_latency/memory-waitcnt/variant=gather1,n=32768,body=64,wg=256;hip-wave=32` | 3 | vulkan (+11.31%), hipgraph (+0.36%) | 94.6880 | 85.0640 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=64;hip-wave=64` | 4 | vulkan (+42.93%), hip (+8.48%), hipgraph (+6.44%) | 29.5360 | 20.6640 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=256;hip-wave=64` | 2 | vulkan (+10.96%) | 22.9960 | 20.7240 |
| `serial_latency/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 3 | hip (+0.07%), hipgraph (+0.06%) | 482.6360 | 482.3123 |
| `serial_latency/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | hipgraph (+0.37%) | 483.0240 | 481.2323 |
| `serial_latency/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 3 | hipgraph (+0.61%), hip (+0.15%) | 485.5040 | 482.5643 |
| `serial_latency/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 3 | hipgraph (+0.52%), hip (+0.16%) | 484.2840 | 481.7642 |
| `serial_latency/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | hipgraph (+0.74%) | 486.9160 | 483.3602 |
| `serial_latency/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | hipgraph (+0.43%) | 482.4040 | 480.3202 |
| `serial_latency/vopd/variant=independent_fma,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 3 | hipgraph (+8.12%), hip (+2.98%) | 70.0880 | 64.8240 |
| `serial_latency/vopd/variant=dependent_fma,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 4 | hipgraph (+20.95%), vulkan (+14.91%), hip (+14.27%) | 83.4640 | 69.0080 |
| `serial_latency/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 4 | vulkan (+6.69%), hipgraph (+5.06%), hip (+2.15%) | 131.9880 | 123.7160 |
| `serial_latency/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 2 | vulkan (+4.05%) | 128.7400 | 123.7240 |
| `serial_latency/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 4 | vulkan (+5.75%), hipgraph (+5.18%), hip (+1.61%) | 105.5720 | 99.8360 |
| `serial_latency/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 2 | vulkan (+2.77%) | 102.5840 | 99.8200 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=1,wg=64;hip-wave=32` | 3 | hipgraph (+0.36%), hip (+0.02%) | 724.5560 | 721.9362 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=1,wg=256;hip-wave=32` | 2 | vulkan (+12.19%) | 36.7840 | 32.7880 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=1,wg=256;hip-wave=32` | 2 | hipgraph (+0.22%) | 197.8960 | 197.4520 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=4,wg=64;hip-wave=32` | 2 | hipgraph (+2.46%) | 106.2120 | 103.6600 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=4,wg=256;hip-wave=32` | 2 | hipgraph (+0.79%) | 198.5040 | 196.9521 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=8,wg=64;hip-wave=32` | 2 | hipgraph (+1.60%) | 105.9560 | 104.2840 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=8,wg=256;hip-wave=32` | 3 | vulkan (+4.95%), hipgraph (+0.06%) | 34.5720 | 32.9400 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=8,wg=256;hip-wave=32` | 2 | hipgraph (+0.51%) | 237.5720 | 236.3641 |
| `serial_latency/two-stage-reduction/k=32768,rows=4,splits=2,wg=128,body=16;hip-wave=32` | 2 | hipgraph (+13.34%) | 30.5880 | 26.9880 |
| `serial_latency/two-stage-reduction/k=32768,rows=4,splits=4,wg=128,body=16;hip-wave=32` | 2 | hipgraph (+18.96%) | 23.6880 | 19.9120 |
| `serial_latency/two-stage-reduction/k=32768,rows=4,splits=2,wg=256,body=16;hip-wave=32` | 3 | hip (+46.99%), hipgraph (+40.83%) | 28.5160 | 19.4000 |
| `serial_latency/two-stage-reduction/k=32768,rows=4,splits=4,wg=256,body=16;hip-wave=32` | 2 | hipgraph (+12.39%) | 20.2760 | 18.0400 |
| `serial_latency/q4-selected-dual/operation=q8_1_quantize,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=32;hip-wave=32` | 4 | vulkan (+604.73%), hipgraph (+173.72%), hip (+86.87%) | 22.6640 | 3.2160 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+13.64%) | 89.7920 | 79.0160 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+29.64%) | 103.8960 | 80.1400 |
| `serial_latency/q6-x8-selected-down/operation=q8_1_quantize,rows=8,experts=256,in=512,out=2048,wg=32;hip-wave=32` | 4 | vulkan (+410.56%), hipgraph (+82.65%), hip (+47.42%) | 9.4760 | 1.8560 |
| `serial_latency/q6-x8-selected-down/operation=x8_selected_dp4a_quantize_plus_dot,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+219.80%) | 113.7320 | 35.5640 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=768,out=2048,rows=1,wg=32;hip-wave=32` | 3 | vulkan (+228.42%), hipgraph (+21.41%) | 6.1480 | 1.8720 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+133.82%) | 10.2880 | 4.4000 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+10.11%) | 17.0760 | 15.5080 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+96.65%) | 13.1520 | 6.6880 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 3 | vulkan (+31.07%), hipgraph (+7.94%) | 19.8960 | 15.1800 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=768,out=2048,rows=4,wg=32;hip-wave=32` | 4 | vulkan (+408.32%), hipgraph (+81.63%), hip (+43.66%) | 9.2920 | 1.8280 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+58.13%) | 13.1120 | 8.2920 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+12.22%) | 22.1080 | 19.7000 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+80.03%) | 16.7640 | 9.3120 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+48.17%) | 25.2600 | 17.0480 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=2048,out=2048,rows=1,wg=32;hip-wave=32` | 4 | vulkan (+398.88%), hipgraph (+77.38%), hip (+41.10%) | 8.9400 | 1.7920 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+19.77%) | 12.2400 | 10.2200 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+24.18%) | 22.0400 | 17.7480 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+33.92%) | 21.5720 | 16.1080 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+30.44%) | 31.1080 | 23.8480 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=2048,out=2048,rows=4,wg=32;hip-wave=32` | 4 | vulkan (+396.82%), hipgraph (+67.02%), hip (+36.52%) | 9.3600 | 1.8840 |
| `independent_throughput/dispatch-grid/sweep=count,count=941,grid=1;hip-wave=32` | 2 | vulkan (+109.56%) | 0.1565 | 0.0747 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=1;hip-wave=32` | 2 | vulkan (+31.84%) | 0.1566 | 0.1188 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=128;hip-wave=32` | 4 | vulkan (+3835.75%), hipgraph (+112.97%), hip (+41.11%) | 4.8768 | 0.1239 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=1024;hip-wave=32` | 2 | vulkan (+1428.54%) | 3.6899 | 0.2414 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=8192;hip-wave=32` | 2 | vulkan (+371.51%) | 6.6098 | 1.4018 |
| `independent_throughput/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+11.01%) | 17.3400 | 15.6200 |
| `independent_throughput/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+3.49%) | 16.9560 | 16.3840 |
| `independent_throughput/memory-waitcnt/variant=strided4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+45.26%) | 18.8600 | 12.9840 |
| `independent_throughput/memory-waitcnt/variant=strided4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+41.97%) | 16.8720 | 11.8840 |
| `independent_throughput/memory-waitcnt/variant=gather1,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+4.73%) | 80.9160 | 77.2640 |
| `independent_throughput/memory-waitcnt/variant=gather1,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+2.23%) | 76.9320 | 75.2520 |
| `independent_throughput/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 3 | hipgraph (+11.28%), vulkan (+5.51%) | 358.3400 | 322.0240 |
| `independent_throughput/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 2 | hipgraph (+4.41%) | 439.1200 | 420.5680 |
| `independent_throughput/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | hip (+1.95%) | 431.5720 | 423.3040 |
| `independent_throughput/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 3 | hipgraph (+5.58%), hip (+1.13%) | 436.0560 | 413.0159 |
| `independent_throughput/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | hip (+1.00%) | 431.9840 | 427.7240 |
| `independent_throughput/two-stage-reduction/k=8192,rows=1,splits=2,wg=128,body=16;hip-wave=32` | 2 | vulkan (+15.22%) | 12.4440 | 10.8000 |
| `independent_throughput/two-stage-reduction/k=8192,rows=1,splits=4,wg=256,body=16;hip-wave=32` | 2 | vulkan (+27.27%) | 9.9120 | 7.7880 |
| `independent_throughput/two-stage-reduction/k=8192,rows=4,splits=4,wg=256,body=16;hip-wave=32` | 2 | vulkan (+46.97%) | 7.3840 | 5.0240 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=2,wg=128,body=16;hip-wave=32` | 2 | vulkan (+55.38%) | 18.6080 | 11.9760 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=4,wg=128,body=16;hip-wave=32` | 2 | vulkan (+19.18%) | 11.2840 | 9.4680 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=2,wg=256,body=16;hip-wave=32` | 2 | vulkan (+42.71%) | 11.2000 | 7.8480 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=2,wg=128,body=16;hip-wave=32` | 2 | vulkan (+67.17%) | 18.9840 | 11.3560 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=4,wg=128,body=16;hip-wave=32` | 2 | vulkan (+36.62%) | 13.2360 | 9.6880 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=2,wg=256,body=16;hip-wave=32` | 2 | vulkan (+23.69%) | 11.5920 | 9.3720 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=4,wg=256,body=16;hip-wave=32` | 2 | vulkan (+20.09%) | 10.1360 | 8.4400 |
| `independent_throughput/q4-selected-dual/operation=q8_1_quantize,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=32;hip-wave=32` | 2 | vulkan (+106.03%) | 7.6480 | 3.7120 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+86.67%) | 109.2520 | 58.5280 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+50.57%) | 98.0480 | 65.1200 |
| `independent_throughput/q6-x8-selected-down/operation=q8_1_quantize,rows=8,experts=256,in=512,out=2048,wg=32;hip-wave=32` | 2 | vulkan (+123.52%) | 6.9560 | 3.1120 |
| `independent_throughput/q6-x8-selected-down/operation=x8_selected_dp4a_quantize_plus_dot,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+716.96%) | 112.7400 | 13.8000 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+76.19%) | 6.8080 | 3.8640 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+50.83%) | 12.7720 | 8.4680 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+44.92%) | 5.8720 | 4.0520 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+82.87%) | 13.9640 | 7.6360 |
| `independent_throughput/dense-q8/operation=q8_1_quantize,in=768,out=2048,rows=4,wg=32;hip-wave=32` | 2 | vulkan (+44.49%) | 6.5600 | 4.5400 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+52.94%) | 10.8280 | 7.0800 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+64.15%) | 19.1600 | 11.6720 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+25.87%) | 9.0680 | 7.2040 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+56.41%) | 17.9560 | 11.4800 |
| `independent_throughput/dense-q8/operation=q8_1_quantize,in=2048,out=2048,rows=1,wg=32;hip-wave=32` | 2 | vulkan (+94.68%) | 17.8480 | 9.1680 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+85.03%) | 18.7320 | 10.1240 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 3 | vulkan (+71.51%), hip (+11.81%) | 38.3760 | 22.3760 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+127.91%) | 16.1360 | 7.0800 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+155.55%) | 24.1240 | 9.4400 |
| `independent_throughput/dense-q8/operation=q8_1_quantize,in=2048,out=2048,rows=4,wg=32;hip-wave=32` | 2 | vulkan (+51.28%) | 6.6320 | 4.3840 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+116.57%) | 46.9600 | 21.6840 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+75.63%) | 36.8200 | 20.9640 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 134 | 100 | 0 | 57.26 | 0.9339 | 234 |
| RL / hipgraph | 174 | 60 | 0 | 74.36 | 0.6868 | 234 |
| RL / hip | 205 | 29 | 0 | 87.61 | 0.6240 | 234 |

## Pinned hipEngine-harness comparison

The `hipengine` profile covers the same benchmark row set as pinned HipEngine `f2c3ad6`: family, operation, shape/sweep axes, repetition count, and serial/independent mode. Those rows deliberately run through Hipfire's existing Radiowave-tuned launch policy; wave size, workgroup geometry, source variant, ABI, and machine code remain optimization variables rather than parity constraints. HipEngine has 212 three-way matched core rows because its HIP independent sampler path rejects 12 rows; its 16 dispatch controls are reported separately. Hipfire executes and correctness-gates all 240 rows across four backends.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine core | RL / vulkan | 192 | 20 | 90.57 | 0.4589 | 212 |
| hipEngine core | RL / hip | 158 | 54 | 74.53 | 0.8343 | 212 |
| hipEngine dispatch | RL / vulkan | 6 | 10 | 37.50 | 1.4331 | 16 |
| hipEngine dispatch | RL / hip | 16 | 0 | 100.00 | 0.2249 | 16 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 205/234 rows and HipGraph in 174/234 while all three select identical per-row hipcc code objects. This run uses the `hipengine_f2c` matrix, `radiowave_tuned` wave policy, and `default` scheduler profile. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 134/234 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. The `hipengine_f2c` profile matches HipEngine's row coverage while intentionally retaining Hipfire/Radiowave's tuned wave, workgroup, and source-variant choices. This artifact records the `radiowave_tuned` wave policy and `default` scheduler profile as controlled HIP compilation/launch factors. Every completion timestamp necessarily proves the measured work finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
