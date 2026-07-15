# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 125/240 rows (52.08%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 125 | 90 | 11 | 14 | 52.08 | 240 |
| vulkan | 88 | 94 | 2 | 56 | 36.67 | 240 |
| hipgraph | 27 | 38 | 89 | 86 | 11.25 | 240 |
| hip | 0 | 18 | 138 | 84 | 0.00 | 240 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 53 | 50 | 9 | 8 | 120 |
| serial_latency | vulkan | 40 | 22 | 2 | 56 | 120 |
| serial_latency | hipgraph | 27 | 38 | 50 | 5 | 120 |
| serial_latency | hip | 0 | 10 | 59 | 51 | 120 |
| independent_throughput | redline | 72 | 40 | 2 | 6 | 120 |
| independent_throughput | vulkan | 48 | 72 | 0 | 0 | 120 |
| independent_throughput | hipgraph | 0 | 0 | 39 | 81 | 120 |
| independent_throughput | hip | 0 | 8 | 79 | 33 | 120 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/dispatch-grid/sweep=count,count=1,grid=1;hip-wave=32` | 3 | vulkan (+128.99%), hip (+11.00%) | 21.8000 | 9.5200 |
| `serial_latency/dispatch-grid/sweep=count,count=50,grid=1;hip-wave=32` | 2 | vulkan (+7.28%) | 3.2288 | 3.0096 |
| `serial_latency/dispatch-grid/sweep=count,count=200,grid=1;hip-wave=32` | 2 | vulkan (+8.33%) | 3.1692 | 2.9256 |
| `serial_latency/dispatch-grid/sweep=count,count=941,grid=1;hip-wave=32` | 2 | vulkan (+8.02%) | 3.1515 | 2.9176 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=1;hip-wave=32` | 2 | vulkan (+8.59%) | 3.1519 | 2.9026 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=128;hip-wave=32` | 2 | vulkan (+13.87%) | 3.4040 | 2.9895 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=1024;hip-wave=32` | 2 | vulkan (+10.11%) | 5.8863 | 5.3457 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=8192;hip-wave=32` | 2 | vulkan (+155.09%) | 59.4562 | 23.3083 |
| `serial_latency/geometry/k=512,rows=1,wg=64,body=32;hip-wave=32` | 3 | hipgraph (+6.06%), hip (+0.46%) | 66.8000 | 62.9838 |
| `serial_latency/geometry/k=512,rows=1,wg=256,body=32;hip-wave=32` | 3 | hipgraph (+8.66%), hip (+3.68%) | 15.2120 | 13.9999 |
| `serial_latency/geometry/k=512,rows=4,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+5.28%) | 66.2360 | 62.9157 |
| `serial_latency/geometry/k=512,rows=4,wg=256,body=32;hip-wave=32` | 3 | hipgraph (+7.00%), hip (+2.37%) | 15.2240 | 14.2279 |
| `serial_latency/geometry/k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+3.01%) | 141.6200 | 137.4835 |
| `serial_latency/geometry/k=2048,rows=1,wg=256,body=32;hip-wave=32` | 2 | hipgraph (+4.96%) | 67.3960 | 64.2117 |
| `serial_latency/geometry/k=2048,rows=4,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+2.48%) | 141.1000 | 137.6874 |
| `serial_latency/geometry/k=2048,rows=4,wg=256,body=32;hip-wave=32` | 2 | hipgraph (+4.55%) | 67.5320 | 64.5958 |
| `serial_latency/reduction/variant=lds_tree,k=512,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+0.56%) | 62.9360 | 62.5838 |
| `serial_latency/reduction/variant=lds_tree,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+0.01%) | 136.8760 | 136.8635 |
| `serial_latency/reduction/variant=extra_barrier,k=512,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+0.41%) | 63.2640 | 63.0078 |
| `serial_latency/reduction/variant=extra_barrier,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+0.06%) | 137.0680 | 136.9795 |
| `serial_latency/reduction/variant=wave_shuffle,k=512,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+5.61%) | 66.5760 | 63.0397 |
| `serial_latency/reduction/variant=wave_shuffle,k=512,rows=1,wg=256,body=32;hip-wave=32` | 3 | hipgraph (+8.30%), hip (+2.90%) | 15.1840 | 14.0199 |
| `serial_latency/reduction/variant=wave_shuffle,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+2.36%) | 141.0760 | 137.8275 |
| `serial_latency/reduction/variant=wave_shuffle,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 2 | hipgraph (+5.32%) | 67.4360 | 64.0317 |
| `serial_latency/reduction/variant=multi_accum4,k=512,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+0.04%) | 64.5160 | 64.4876 |
| `serial_latency/reduction/variant=multi_accum4,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 2 | hipgraph (+0.38%) | 66.1440 | 65.8917 |
| `serial_latency/reduction/variant=multi_accum8,k=512,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+0.74%) | 66.6800 | 66.1878 |
| `serial_latency/reduction/variant=multi_accum16,k=512,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+1.63%) | 74.0360 | 72.8518 |
| `serial_latency/reduction/variant=multi_accum16,k=512,rows=1,wg=256,body=32;hip-wave=32` | 2 | hipgraph (+0.53%) | 58.7880 | 58.4758 |
| `serial_latency/reduction/variant=multi_accum16,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hipgraph (+0.21%) | 150.3080 | 149.9955 |
| `serial_latency/reduction/variant=multi_accum16,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 2 | hipgraph (+0.98%) | 75.4600 | 74.7277 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=64;hip-wave=32` | 3 | vulkan (+11.41%), hipgraph (+0.26%) | 86.7840 | 77.8960 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+12.76%) | 88.1480 | 78.1760 |
| `serial_latency/memory-waitcnt/variant=strided4,n=32768,body=64,wg=64;hip-wave=32` | 3 | vulkan (+10.84%), hipgraph (+1.08%) | 86.1720 | 77.7480 |
| `serial_latency/memory-waitcnt/variant=strided4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+11.78%) | 87.0120 | 77.8440 |
| `serial_latency/memory-waitcnt/variant=gather1,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+1.40%), hipgraph (+0.67%), hip (+0.16%) | 363.0160 | 358.0120 |
| `serial_latency/memory-waitcnt/variant=gather1,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+1.69%) | 365.4160 | 359.3360 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=64;hip-wave=32` | 3 | vulkan (+12.37%), hipgraph (+2.55%) | 87.8040 | 78.1400 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+9.46%) | 87.1680 | 79.6320 |
| `serial_latency/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+68.89%), hipgraph (+1.50%), hip (+0.86%) | 1509.3680 | 893.6960 |
| `serial_latency/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 4 | vulkan (+60.18%), hipgraph (+0.46%), hip (+0.16%) | 1433.2720 | 894.8080 |
| `serial_latency/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+68.77%), hip (+0.33%), hipgraph (+0.31%) | 1506.2680 | 892.5160 |
| `serial_latency/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 4 | vulkan (+60.90%), hipgraph (+1.22%), hip (+0.03%) | 1438.6520 | 894.1520 |
| `serial_latency/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+59.92%), hip (+0.60%), hipgraph (+0.29%) | 1506.3800 | 941.9720 |
| `serial_latency/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 4 | vulkan (+52.89%), hip (+1.08%), hipgraph (+0.31%) | 1443.3840 | 944.0880 |
| `serial_latency/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+69.58%), hip (+0.46%), hipgraph (+0.37%) | 1507.5640 | 888.9960 |
| `serial_latency/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 3 | vulkan (+62.41%), hipgraph (+0.27%) | 1446.9280 | 890.9240 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=1,wg=64;hip-wave=32` | 2 | hipgraph (+0.32%) | 1024.8760 | 1021.6280 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=1,wg=256;hip-wave=32` | 2 | hipgraph (+0.41%) | 73.1560 | 72.8557 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=4,wg=64;hip-wave=32` | 2 | hipgraph (+0.05%) | 1170.1240 | 1169.5834 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=8,wg=256;hip-wave=32` | 2 | vulkan (+32.84%) | 73.1280 | 55.0480 |
| `serial_latency/two-stage-reduction/k=32768,rows=1,splits=4,wg=128,body=16;hip-wave=32` | 2 | hipgraph (+2.21%) | 18.3040 | 17.9079 |
| `serial_latency/q4-selected-dual/operation=q8_1_quantize,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=32;hip-wave=32` | 2 | vulkan (+120.99%) | 5.0120 | 2.2680 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+7.96%) | 260.7320 | 241.5040 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+4.75%) | 261.0440 | 249.2160 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+17.36%) | 248.2120 | 211.5040 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+12.97%) | 251.3080 | 222.4640 |
| `serial_latency/q6-x8-selected-down/operation=q8_1_quantize,rows=8,experts=256,in=512,out=2048,wg=32;hip-wave=32` | 2 | vulkan (+130.22%) | 4.9360 | 2.1440 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=768,out=2048,rows=1,wg=32;hip-wave=32` | 2 | vulkan (+122.78%) | 4.9280 | 2.2120 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+4.70%) | 7.9280 | 7.5720 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=768,out=2048,rows=4,wg=32;hip-wave=32` | 2 | vulkan (+132.83%) | 4.9360 | 2.1200 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+4.01%) | 23.3600 | 22.4600 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+33.50%) | 35.6080 | 26.6720 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+9.95%) | 38.8120 | 35.3000 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=2048,out=2048,rows=1,wg=32;hip-wave=32` | 2 | vulkan (+129.28%) | 4.8240 | 2.1040 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=2048,out=2048,rows=4,wg=32;hip-wave=32` | 2 | vulkan (+127.37%) | 4.8840 | 2.1480 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+7.63%) | 82.3920 | 76.5520 |
| `independent_throughput/dispatch-grid/sweep=count,count=1,grid=1;hip-wave=32` | 3 | vulkan (+124.46%), hip (+19.96%) | 20.9200 | 9.3200 |
| `independent_throughput/dispatch-grid/sweep=count,count=941,grid=1;hip-wave=32` | 2 | vulkan (+67.85%) | 0.1307 | 0.0779 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=1;hip-wave=32` | 2 | vulkan (+6.04%) | 0.1305 | 0.1231 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=128;hip-wave=32` | 2 | vulkan (+862.22%) | 2.8239 | 0.2935 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=1024;hip-wave=32` | 2 | vulkan (+163.78%) | 5.5194 | 2.0924 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=8192;hip-wave=32` | 2 | vulkan (+91.95%) | 30.7437 | 16.0163 |
| `independent_throughput/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+42.54%) | 87.5400 | 61.4160 |
| `independent_throughput/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+75.50%) | 94.0600 | 53.5960 |
| `independent_throughput/memory-waitcnt/variant=strided4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+88.98%) | 86.1600 | 45.5920 |
| `independent_throughput/memory-waitcnt/variant=strided4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+89.76%) | 92.2240 | 48.6000 |
| `independent_throughput/memory-waitcnt/variant=gather1,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+31.06%) | 360.9040 | 275.3800 |
| `independent_throughput/memory-waitcnt/variant=gather1,n=32768,body=64,wg=256;hip-wave=32` | 4 | vulkan (+40.86%), hip (+4.48%), hipgraph (+0.24%) | 398.7480 | 283.0880 |
| `independent_throughput/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+34.74%) | 90.8880 | 67.4520 |
| `independent_throughput/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+60.40%) | 94.3800 | 58.8400 |
| `independent_throughput/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+94.05%), hip (+16.84%), hipgraph (+5.09%) | 1819.7080 | 937.7520 |
| `independent_throughput/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+17.49%) | 1216.5520 | 1035.4080 |
| `independent_throughput/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+83.59%), hip (+9.34%), hipgraph (+1.01%) | 1745.9920 | 951.0160 |
| `independent_throughput/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+23.74%) | 1218.5000 | 984.7480 |
| `independent_throughput/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+63.63%), hip (+17.94%), hipgraph (+7.28%) | 1861.8160 | 1137.8480 |
| `independent_throughput/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+8.21%) | 1236.8680 | 1142.9960 |
| `independent_throughput/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+92.57%), hip (+17.28%), hipgraph (+6.93%) | 1866.7520 | 969.3680 |
| `independent_throughput/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+30.58%) | 1332.1880 | 1020.2440 |
| `independent_throughput/two-stage-reduction/k=8192,rows=1,splits=4,wg=128,body=16;hip-wave=32` | 2 | vulkan (+6.95%) | 6.8920 | 6.4440 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=2,wg=128,body=16;hip-wave=32` | 4 | vulkan (+19.98%), hip (+16.99%), hipgraph (+4.55%) | 52.5320 | 43.7840 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=2,wg=128,body=16;hip-wave=32` | 3 | vulkan (+12.53%), hip (+4.36%) | 47.4040 | 42.1240 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=4,wg=128,body=16;hip-wave=32` | 2 | vulkan (+0.60%) | 14.0640 | 13.9800 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=2,wg=256,body=16;hip-wave=32` | 2 | vulkan (+21.68%) | 13.9440 | 11.4600 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+65.14%) | 251.1600 | 152.0920 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+29.70%) | 253.9480 | 195.8000 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+51.60%) | 244.3480 | 161.1840 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+41.17%) | 247.0840 | 175.0200 |
| `independent_throughput/q6-x8-selected-down/operation=x8_selected_dp4a_dot_prequantized,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+70.69%) | 46.6040 | 27.3040 |
| `independent_throughput/q6-x8-selected-down/operation=x8_selected_dp4a_quantize_plus_dot,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+95.42%) | 57.3920 | 29.3680 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+22.66%) | 7.1440 | 5.8240 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+0.61%) | 5.9480 | 5.9120 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+2.18%) | 9.5480 | 9.3440 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+14.71%) | 22.0800 | 19.2480 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+21.74%) | 24.7920 | 20.3640 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+15.40%) | 21.5760 | 18.6960 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+23.07%) | 28.3560 | 23.0400 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+21.73%) | 15.4840 | 12.7200 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+14.02%) | 24.4920 | 21.4800 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+20.04%) | 17.8280 | 14.8520 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+53.63%) | 40.7480 | 26.5240 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+81.57%) | 79.8240 | 43.9640 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+82.19%) | 82.2120 | 45.1240 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+32.81%) | 99.1480 | 74.6560 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+171.54%) | 190.7160 | 70.2360 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 152 | 88 | 0 | 63.33 | 0.8525 | 240 |
| RL / hipgraph | 195 | 45 | 0 | 81.25 | 0.5253 | 240 |
| RL / hip | 219 | 21 | 0 | 91.25 | 0.5183 | 240 |

## Pinned hipEngine-harness comparison

The `hipengine` profile covers the same benchmark row set as pinned HipEngine `f2c3ad6`: family, operation, shape/sweep axes, repetition count, and serial/independent mode. Those rows deliberately run through Hipfire's existing Radiowave-tuned launch policy; wave size, workgroup geometry, source variant, ABI, and machine code remain optimization variables rather than parity constraints. HipEngine has 212 three-way matched core rows because its HIP independent sampler path rejects 12 rows; its 16 dispatch controls are reported separately. Hipfire executes and correctness-gates all 240 rows across four backends.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine core | RL / vulkan | 192 | 20 | 90.57 | 0.4589 | 212 |
| hipEngine core | RL / hip | 158 | 54 | 74.53 | 0.8343 | 212 |
| hipEngine dispatch | RL / vulkan | 6 | 10 | 37.50 | 1.4331 | 16 |
| hipEngine dispatch | RL / hip | 16 | 0 | 100.00 | 0.2249 | 16 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 219/240 rows and HipGraph in 195/240 while all three select identical per-row hipcc code objects. This run uses the `hipengine_f2c` matrix, `radiowave_tuned` wave policy, and `default` scheduler profile. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 152/240 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. The `hipengine_f2c` profile matches HipEngine's row coverage while intentionally retaining Hipfire/Radiowave's tuned wave, workgroup, and source-variant choices. This artifact records the `radiowave_tuned` wave policy and `default` scheduler profile as controlled HIP compilation/launch factors. Every completion timestamp necessarily proves the measured work finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
