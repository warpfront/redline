# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 123/240 rows (51.25%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 123 | 85 | 19 | 13 | 51.25 | 240 |
| vulkan | 85 | 97 | 2 | 56 | 35.42 | 240 |
| hipgraph | 32 | 34 | 87 | 87 | 13.33 | 240 |
| hip | 0 | 24 | 132 | 84 | 0.00 | 240 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 48 | 47 | 17 | 8 | 120 |
| serial_latency | vulkan | 40 | 22 | 2 | 56 | 120 |
| serial_latency | hipgraph | 32 | 34 | 49 | 5 | 120 |
| serial_latency | hip | 0 | 17 | 52 | 51 | 120 |
| independent_throughput | redline | 75 | 38 | 2 | 5 | 120 |
| independent_throughput | vulkan | 45 | 75 | 0 | 0 | 120 |
| independent_throughput | hipgraph | 0 | 0 | 38 | 82 | 120 |
| independent_throughput | hip | 0 | 7 | 80 | 33 | 120 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/dispatch-grid/sweep=count,count=1,grid=1;hip-wave=32` | 3 | vulkan (+129.41%), hip (+9.20%) | 21.8400 | 9.5200 |
| `serial_latency/dispatch-grid/sweep=count,count=50,grid=1;hip-wave=32` | 2 | vulkan (+8.26%) | 3.2304 | 2.9840 |
| `serial_latency/dispatch-grid/sweep=count,count=200,grid=1;hip-wave=32` | 2 | vulkan (+8.09%) | 3.1692 | 2.9320 |
| `serial_latency/dispatch-grid/sweep=count,count=941,grid=1;hip-wave=32` | 2 | vulkan (+8.52%) | 3.1521 | 2.9047 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=1;hip-wave=32` | 2 | vulkan (+8.25%) | 3.1525 | 2.9121 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=128;hip-wave=32` | 2 | vulkan (+13.79%) | 3.4004 | 2.9882 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=1024;hip-wave=32` | 2 | vulkan (+9.83%) | 5.8861 | 5.3592 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=8192;hip-wave=32` | 2 | vulkan (+156.82%) | 59.4315 | 23.1414 |
| `serial_latency/geometry/k=512,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+5.41%), hip (+0.27%) | 66.6600 | 63.2358 |
| `serial_latency/geometry/k=512,rows=1,wg=256,body=32;hip-wave=32` | 3 | hipgraph (+8.68%), hip (+3.04%) | 15.1800 | 13.9679 |
| `serial_latency/geometry/k=512,rows=4,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+6.36%), hip (+0.63%) | 67.0080 | 62.9997 |
| `serial_latency/geometry/k=512,rows=4,wg=256,body=32;hip-wave=32` | 3 | hipgraph (+6.18%), hip (+1.69%) | 15.1800 | 14.2959 |
| `serial_latency/geometry/k=2048,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+2.62%), hip (+0.02%) | 141.2680 | 137.6635 |
| `serial_latency/geometry/k=2048,rows=1,wg=256,body=32;hip-wave=32` | 2 | hipgraph (+5.34%) | 67.2800 | 63.8678 |
| `serial_latency/geometry/k=2048,rows=4,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+2.71%), hip (+0.10%) | 141.2720 | 137.5475 |
| `serial_latency/geometry/k=2048,rows=4,wg=256,body=32;hip-wave=32` | 2 | hipgraph (+4.95%) | 67.4800 | 64.2998 |
| `serial_latency/reduction/variant=lds_tree,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+0.06%) | 136.9400 | 136.8514 |
| `serial_latency/reduction/variant=extra_barrier,k=512,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+0.01%) | 63.2040 | 63.1958 |
| `serial_latency/reduction/variant=extra_barrier,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+0.36%) | 137.7600 | 137.2635 |
| `serial_latency/reduction/variant=extra_barrier,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 2 | hipgraph (+0.19%) | 63.8400 | 63.7197 |
| `serial_latency/reduction/variant=wave_shuffle,k=512,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+6.47%), hip (+0.65%) | 67.0800 | 63.0037 |
| `serial_latency/reduction/variant=wave_shuffle,k=512,rows=1,wg=256,body=32;hip-wave=32` | 3 | hipgraph (+8.01%), hip (+3.68%) | 15.2040 | 14.0759 |
| `serial_latency/reduction/variant=wave_shuffle,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+2.68%), hip (+0.05%) | 141.2440 | 137.5515 |
| `serial_latency/reduction/variant=wave_shuffle,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 3 | hipgraph (+4.53%), hip (+0.02%) | 67.3160 | 64.3998 |
| `serial_latency/reduction/variant=multi_accum4,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+0.08%) | 142.5840 | 142.4635 |
| `serial_latency/reduction/variant=multi_accum8,k=512,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+0.43%) | 66.4880 | 66.2037 |
| `serial_latency/reduction/variant=multi_accum8,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+0.10%) | 149.2680 | 149.1235 |
| `serial_latency/reduction/variant=multi_accum8,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 2 | hipgraph (+0.01%) | 67.7120 | 67.7038 |
| `serial_latency/reduction/variant=multi_accum16,k=512,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+1.16%) | 73.7480 | 72.8997 |
| `serial_latency/reduction/variant=multi_accum16,k=512,rows=1,wg=256,body=32;hip-wave=32` | 2 | hipgraph (+0.46%) | 58.4080 | 58.1398 |
| `serial_latency/reduction/variant=multi_accum16,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+0.14%) | 150.1640 | 149.9515 |
| `serial_latency/reduction/variant=multi_accum16,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 2 | hipgraph (+0.61%) | 75.1640 | 74.7077 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+10.55%) | 86.3040 | 78.0680 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+11.40%) | 87.1280 | 78.2120 |
| `serial_latency/memory-waitcnt/variant=strided4,n=32768,body=64,wg=64;hip-wave=32` | 3 | vulkan (+10.43%), hipgraph (+1.35%) | 86.2120 | 78.0720 |
| `serial_latency/memory-waitcnt/variant=strided4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+11.19%) | 86.6400 | 77.9240 |
| `serial_latency/memory-waitcnt/variant=gather1,n=32768,body=64,wg=64;hip-wave=32` | 4 | hipgraph (+0.95%), vulkan (+0.93%), hip (+0.13%) | 363.4560 | 360.0226 |
| `serial_latency/memory-waitcnt/variant=gather1,n=32768,body=64,wg=256;hip-wave=32` | 3 | vulkan (+1.97%), hipgraph (+0.23%) | 365.4360 | 358.3600 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=64;hip-wave=32` | 3 | vulkan (+10.99%), hipgraph (+1.15%) | 86.8040 | 78.2080 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+9.85%) | 87.3720 | 79.5400 |
| `serial_latency/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+67.48%), hip (+0.08%), hipgraph (+0.00%) | 1498.4840 | 894.7240 |
| `serial_latency/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 4 | vulkan (+60.74%), hip (+0.93%), hipgraph (+0.57%) | 1437.0520 | 893.9960 |
| `serial_latency/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+68.69%), hipgraph (+1.41%), hip (+0.56%) | 1509.1560 | 894.6440 |
| `serial_latency/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 4 | vulkan (+61.55%), hipgraph (+1.27%), hip (+0.82%) | 1442.1000 | 892.6600 |
| `serial_latency/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 3 | vulkan (+58.39%), hipgraph (+0.45%) | 1495.3800 | 944.0840 |
| `serial_latency/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 4 | vulkan (+52.77%), hip (+0.80%), hipgraph (+0.15%) | 1438.9440 | 941.8880 |
| `serial_latency/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+69.51%), hipgraph (+1.60%), hip (+0.74%) | 1514.1160 | 893.2320 |
| `serial_latency/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 4 | vulkan (+63.12%), hip (+0.66%), hipgraph (+0.54%) | 1454.1640 | 891.4560 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=1,wg=64;hip-wave=32` | 2 | hipgraph (+0.70%) | 188.3000 | 186.9833 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=1,wg=64;hip-wave=32` | 3 | hipgraph (+0.29%), hip (+0.01%) | 1024.6200 | 1021.6241 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=1,wg=256;hip-wave=32` | 2 | hipgraph (+0.28%) | 72.9840 | 72.7797 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=1,wg=256;hip-wave=32` | 2 | hipgraph (+0.03%) | 292.1760 | 292.0749 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=4,wg=256;hip-wave=32` | 2 | hipgraph (+1.10%) | 329.1480 | 325.5588 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=8,wg=64;hip-wave=32` | 2 | hipgraph (+0.82%) | 189.3600 | 187.8233 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=8,wg=256;hip-wave=32` | 3 | vulkan (+32.71%), hipgraph (+2.26%) | 72.9920 | 55.0000 |
| `serial_latency/two-stage-reduction/k=32768,rows=1,splits=4,wg=128,body=16;hip-wave=32` | 2 | hipgraph (+1.79%) | 18.2360 | 17.9159 |
| `serial_latency/q4-selected-dual/operation=q8_1_quantize,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=32;hip-wave=32` | 2 | vulkan (+120.81%) | 5.0080 | 2.2680 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+8.29%) | 259.9080 | 240.0160 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+4.90%) | 261.0120 | 248.8080 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+17.24%) | 248.2120 | 211.7080 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+13.11%) | 251.3200 | 222.2000 |
| `serial_latency/q6-x8-selected-down/operation=q8_1_quantize,rows=8,experts=256,in=512,out=2048,wg=32;hip-wave=32` | 2 | vulkan (+124.55%) | 4.9400 | 2.2000 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=768,out=2048,rows=1,wg=32;hip-wave=32` | 2 | vulkan (+121.98%) | 4.9280 | 2.2200 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+4.53%) | 7.9400 | 7.5960 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=768,out=2048,rows=4,wg=32;hip-wave=32` | 2 | vulkan (+133.46%) | 4.9400 | 2.1160 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+7.22%) | 23.8840 | 22.2760 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+33.41%) | 35.4880 | 26.6000 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+9.49%) | 38.6200 | 35.2720 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=2048,out=2048,rows=1,wg=32;hip-wave=32` | 2 | vulkan (+129.71%) | 4.8240 | 2.1000 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=2048,out=2048,rows=4,wg=32;hip-wave=32` | 2 | vulkan (+127.17%) | 4.9160 | 2.1640 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+12.29%) | 86.4000 | 76.9440 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+2.76%) | 185.4480 | 180.4720 |
| `independent_throughput/dispatch-grid/sweep=count,count=1,grid=1;hip-wave=32` | 3 | vulkan (+124.89%), hip (+20.18%) | 20.9600 | 9.3200 |
| `independent_throughput/dispatch-grid/sweep=count,count=941,grid=1;hip-wave=32` | 2 | vulkan (+82.17%) | 0.1307 | 0.0718 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=1;hip-wave=32` | 2 | vulkan (+6.97%) | 0.1305 | 0.1220 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=128;hip-wave=32` | 2 | vulkan (+856.08%) | 2.8038 | 0.2933 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=1024;hip-wave=32` | 2 | vulkan (+162.34%) | 5.5012 | 2.0970 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=8192;hip-wave=32` | 2 | vulkan (+84.16%) | 29.5065 | 16.0219 |
| `independent_throughput/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+41.95%) | 86.8840 | 61.2080 |
| `independent_throughput/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+92.02%) | 95.0040 | 49.4760 |
| `independent_throughput/memory-waitcnt/variant=strided4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+91.68%) | 86.0400 | 44.8880 |
| `independent_throughput/memory-waitcnt/variant=strided4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+73.98%) | 93.3000 | 53.6280 |
| `independent_throughput/memory-waitcnt/variant=gather1,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+29.63%) | 359.9600 | 277.6880 |
| `independent_throughput/memory-waitcnt/variant=gather1,n=32768,body=64,wg=256;hip-wave=32` | 4 | vulkan (+46.21%), hip (+4.63%), hipgraph (+0.18%) | 394.4920 | 269.8040 |
| `independent_throughput/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+22.57%) | 90.5760 | 73.8960 |
| `independent_throughput/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+49.70%) | 94.3240 | 63.0080 |
| `independent_throughput/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+88.06%), hip (+11.22%), hipgraph (+6.28%) | 1831.8360 | 974.0920 |
| `independent_throughput/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+16.67%) | 1203.8840 | 1031.8920 |
| `independent_throughput/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 3 | vulkan (+75.41%), hip (+7.11%) | 1704.8200 | 971.9280 |
| `independent_throughput/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+19.15%) | 1204.9720 | 1011.3080 |
| `independent_throughput/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+67.62%), hip (+16.76%), hipgraph (+7.29%) | 1853.1920 | 1105.6040 |
| `independent_throughput/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+11.69%) | 1267.7400 | 1135.0120 |
| `independent_throughput/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+96.92%), hip (+15.81%), hipgraph (+5.13%) | 1835.2600 | 931.9760 |
| `independent_throughput/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+27.56%) | 1306.7720 | 1024.4680 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=2,wg=128,body=16;hip-wave=32` | 4 | vulkan (+25.70%), hip (+19.94%), hipgraph (+7.04%) | 54.6360 | 43.4640 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=4,wg=128,body=16;hip-wave=32` | 2 | vulkan (+24.92%) | 14.8760 | 11.9080 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=2,wg=256,body=16;hip-wave=32` | 2 | vulkan (+21.17%) | 13.9880 | 11.5440 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+64.36%) | 251.1080 | 152.7800 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+25.11%) | 254.1720 | 203.1600 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+51.49%) | 244.9760 | 161.7120 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+39.32%) | 247.3840 | 177.5600 |
| `independent_throughput/q6-x8-selected-down/operation=x8_selected_dp4a_dot_prequantized,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+55.30%) | 46.9680 | 30.2440 |
| `independent_throughput/q6-x8-selected-down/operation=x8_selected_dp4a_quantize_plus_dot,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+114.65%) | 58.0760 | 27.0560 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+4.71%) | 6.4080 | 6.1200 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+4.96%) | 9.5640 | 9.1120 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+14.58%) | 22.3880 | 19.5400 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+19.33%) | 24.6640 | 20.6680 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+18.19%) | 21.5760 | 18.2560 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+22.35%) | 28.5320 | 23.3200 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+26.32%) | 15.7440 | 12.4640 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+8.15%) | 22.7240 | 21.0120 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+26.81%) | 17.9560 | 14.1600 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+74.97%) | 41.2720 | 23.5880 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+72.27%) | 78.6160 | 45.6360 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+87.45%) | 83.5800 | 44.5880 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+19.50%) | 99.8360 | 83.5440 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+164.75%) | 185.8640 | 70.2040 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 154 | 86 | 0 | 64.17 | 0.8540 | 240 |
| RL / hipgraph | 191 | 49 | 0 | 79.58 | 0.5355 | 240 |
| RL / hip | 213 | 27 | 0 | 88.75 | 0.5253 | 240 |

## Pinned hipEngine-harness comparison

The `hipengine` profile covers the same benchmark row set as pinned HipEngine `f2c3ad6`: family, operation, shape/sweep axes, repetition count, and serial/independent mode. Those rows deliberately run through Hipfire's existing Radiowave-tuned launch policy; wave size, workgroup geometry, source variant, ABI, and machine code remain optimization variables rather than parity constraints. HipEngine has 212 three-way matched core rows because its HIP independent sampler path rejects 12 rows; its 16 dispatch controls are reported separately. Hipfire executes and correctness-gates all 240 rows across four backends.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine core | RL / vulkan | 192 | 20 | 90.57 | 0.4589 | 212 |
| hipEngine core | RL / hip | 158 | 54 | 74.53 | 0.8343 | 212 |
| hipEngine dispatch | RL / vulkan | 6 | 10 | 37.50 | 1.4331 | 16 |
| hipEngine dispatch | RL / hip | 16 | 0 | 100.00 | 0.2249 | 16 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 213/240 rows and HipGraph in 191/240 while all three select identical per-row hipcc code objects. This run uses the `hipengine_f2c` matrix, `radiowave_tuned` wave policy, and `default` scheduler profile. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 154/240 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. The `hipengine_f2c` profile matches HipEngine's row coverage while intentionally retaining Hipfire/Radiowave's tuned wave, workgroup, and source-variant choices. This artifact records the `radiowave_tuned` wave policy and `default` scheduler profile as controlled HIP compilation/launch factors. Every completion timestamp necessarily proves the measured work finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
