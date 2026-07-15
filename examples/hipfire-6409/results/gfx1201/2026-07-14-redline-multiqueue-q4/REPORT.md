# Hipfire/Redline ROCm issue 6409 backend comparison

Correctness-gated result: **Redline wins 101/240 rows (42.08%)**.

## Placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 101 | 139 | 0 | 0 | 42.08 | 240 |
| vulkan | 139 | 101 | 0 | 0 | 57.92 | 240 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 86 | 34 | 0 | 0 | 120 |
| serial_latency | vulkan | 34 | 86 | 0 | 0 | 120 |
| independent_throughput | redline | 15 | 105 | 0 | 0 | 120 |
| independent_throughput | vulkan | 105 | 15 | 0 | 0 | 120 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+4.13%) | 22.7920 | 21.8880 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+1.57%) | 22.5120 | 22.1640 |
| `serial_latency/memory-waitcnt/variant=gather1,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+16.20%) | 80.8280 | 69.5600 |
| `serial_latency/memory-waitcnt/variant=gather1,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+21.09%) | 84.3200 | 69.6320 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=64;hip-wave=64` | 2 | vulkan (+12.18%) | 40.0080 | 35.6640 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=256;hip-wave=64` | 2 | vulkan (+11.98%) | 39.9920 | 35.7120 |
| `serial_latency/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+4.61%) | 441.1960 | 421.7680 |
| `serial_latency/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+4.14%) | 439.9400 | 422.4680 |
| `serial_latency/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+5.98%) | 446.8000 | 421.6080 |
| `serial_latency/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+3.93%) | 438.9760 | 422.3960 |
| `serial_latency/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+5.99%) | 446.9280 | 421.6520 |
| `serial_latency/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+3.84%) | 438.0640 | 421.8680 |
| `serial_latency/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+2.11%) | 434.3280 | 425.3720 |
| `serial_latency/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+3.05%) | 438.8520 | 425.8480 |
| `serial_latency/vopd/variant=independent_fma,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 2 | vulkan (+13.68%) | 124.9920 | 109.9520 |
| `serial_latency/vopd/variant=independent_fma,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 2 | vulkan (+3.57%) | 113.5760 | 109.6640 |
| `serial_latency/vopd/variant=dependent_fma,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 2 | vulkan (+11.11%) | 122.4160 | 110.1760 |
| `serial_latency/vopd/variant=dependent_fma,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 2 | vulkan (+1.69%) | 112.7560 | 110.8840 |
| `serial_latency/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 2 | vulkan (+44.92%) | 315.0680 | 217.4080 |
| `serial_latency/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 2 | vulkan (+33.93%) | 289.5560 | 216.1920 |
| `serial_latency/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 2 | vulkan (+30.65%) | 251.6760 | 192.6400 |
| `serial_latency/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 2 | vulkan (+17.85%) | 227.3120 | 192.8800 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=64` | 2 | vulkan (+54.57%) | 92.9280 | 60.1200 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+147.53%) | 160.2480 | 64.7400 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=64` | 2 | vulkan (+53.75%) | 112.7760 | 73.3480 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+148.96%) | 192.1600 | 77.1840 |
| `serial_latency/q6-x8-selected-down/operation=x8_selected_dp4a_dot_prequantized,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=64` | 2 | vulkan (+131.34%) | 35.8760 | 15.5080 |
| `serial_latency/q6-x8-selected-down/operation=x8_selected_dp4a_quantize_plus_dot,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+81.21%) | 35.9240 | 19.8240 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+14.54%) | 13.7040 | 11.9640 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+12.67%) | 10.2120 | 9.0640 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+19.78%) | 32.6240 | 27.2360 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+3.74%) | 33.9680 | 32.7440 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+52.76%) | 35.7040 | 23.3720 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+33.05%) | 37.4880 | 28.1760 |
| `independent_throughput/dispatch-grid/sweep=count,count=50,grid=1;hip-wave=32` | 2 | vulkan (+778.56%) | 12.4896 | 1.4216 |
| `independent_throughput/dispatch-grid/sweep=count,count=200,grid=1;hip-wave=32` | 2 | vulkan (+502.18%) | 2.7664 | 0.4594 |
| `independent_throughput/dispatch-grid/sweep=count,count=941,grid=1;hip-wave=32` | 2 | vulkan (+274.06%) | 0.5810 | 0.1553 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=1;hip-wave=32` | 2 | vulkan (+278.44%) | 0.5888 | 0.1556 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=128;hip-wave=32` | 2 | vulkan (+217.06%) | 0.7008 | 0.2210 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=1024;hip-wave=32` | 2 | vulkan (+114.74%) | 1.3430 | 0.6254 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=8192;hip-wave=32` | 2 | vulkan (+43.01%) | 5.4967 | 3.8436 |
| `independent_throughput/geometry/k=512,rows=1,wg=64,body=32;hip-wave=32` | 2 | vulkan (+122.38%) | 52.3120 | 23.5240 |
| `independent_throughput/geometry/k=512,rows=1,wg=256,body=32;hip-wave=32` | 2 | vulkan (+351.19%) | 56.1280 | 12.4400 |
| `independent_throughput/geometry/k=512,rows=4,wg=64,body=32;hip-wave=32` | 2 | vulkan (+123.72%) | 52.3320 | 23.3920 |
| `independent_throughput/geometry/k=512,rows=4,wg=256,body=32;hip-wave=32` | 2 | vulkan (+340.00%) | 54.7360 | 12.4400 |
| `independent_throughput/geometry/k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | vulkan (+52.49%) | 104.4280 | 68.4840 |
| `independent_throughput/geometry/k=2048,rows=1,wg=256,body=32;hip-wave=32` | 2 | vulkan (+121.57%) | 52.4680 | 23.6800 |
| `independent_throughput/geometry/k=2048,rows=4,wg=64,body=32;hip-wave=32` | 2 | vulkan (+53.62%) | 104.7640 | 68.1960 |
| `independent_throughput/geometry/k=2048,rows=4,wg=256,body=32;hip-wave=32` | 2 | vulkan (+119.76%) | 52.2760 | 23.7880 |
| `independent_throughput/reduction/variant=lds_tree,k=512,rows=1,wg=64,body=32;hip-wave=32` | 2 | vulkan (+123.16%) | 52.3080 | 23.4400 |
| `independent_throughput/reduction/variant=lds_tree,k=512,rows=1,wg=256,body=32;hip-wave=32` | 2 | vulkan (+347.58%) | 56.1800 | 12.5520 |
| `independent_throughput/reduction/variant=lds_tree,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | vulkan (+52.99%) | 104.9000 | 68.5680 |
| `independent_throughput/reduction/variant=lds_tree,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 2 | vulkan (+119.31%) | 52.3360 | 23.8640 |
| `independent_throughput/reduction/variant=extra_barrier,k=512,rows=1,wg=64,body=32;hip-wave=32` | 2 | vulkan (+121.94%) | 52.1480 | 23.4960 |
| `independent_throughput/reduction/variant=extra_barrier,k=512,rows=1,wg=256,body=32;hip-wave=32` | 2 | vulkan (+347.71%) | 56.3040 | 12.5760 |
| `independent_throughput/reduction/variant=extra_barrier,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | vulkan (+52.79%) | 104.6760 | 68.5080 |
| `independent_throughput/reduction/variant=extra_barrier,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 2 | vulkan (+120.33%) | 52.5000 | 23.8280 |
| `independent_throughput/reduction/variant=wave_shuffle,k=512,rows=1,wg=64,body=32;hip-wave=32` | 2 | vulkan (+121.49%) | 52.2440 | 23.5880 |
| `independent_throughput/reduction/variant=wave_shuffle,k=512,rows=1,wg=256,body=32;hip-wave=32` | 2 | vulkan (+354.10%) | 56.4720 | 12.4360 |
| `independent_throughput/reduction/variant=wave_shuffle,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | vulkan (+53.05%) | 104.7160 | 68.4200 |
| `independent_throughput/reduction/variant=wave_shuffle,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 2 | vulkan (+120.54%) | 52.3560 | 23.7400 |
| `independent_throughput/reduction/variant=multi_accum4,k=512,rows=1,wg=64,body=32;hip-wave=32` | 2 | vulkan (+118.23%) | 52.1040 | 23.8760 |
| `independent_throughput/reduction/variant=multi_accum4,k=512,rows=1,wg=256,body=32;hip-wave=32` | 2 | vulkan (+321.39%) | 55.8600 | 13.2560 |
| `independent_throughput/reduction/variant=multi_accum4,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | vulkan (+49.75%) | 104.5760 | 69.8360 |
| `independent_throughput/reduction/variant=multi_accum4,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 2 | vulkan (+117.33%) | 52.5680 | 24.1880 |
| `independent_throughput/reduction/variant=multi_accum8,k=512,rows=1,wg=64,body=32;hip-wave=32` | 2 | vulkan (+122.19%) | 52.5000 | 23.6280 |
| `independent_throughput/reduction/variant=multi_accum8,k=512,rows=1,wg=256,body=32;hip-wave=32` | 2 | vulkan (+314.04%) | 56.3760 | 13.6160 |
| `independent_throughput/reduction/variant=multi_accum8,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | vulkan (+52.92%) | 105.4000 | 68.9240 |
| `independent_throughput/reduction/variant=multi_accum8,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 2 | vulkan (+120.52%) | 52.6240 | 23.8640 |
| `independent_throughput/reduction/variant=multi_accum16,k=512,rows=1,wg=64,body=32;hip-wave=32` | 2 | vulkan (+106.44%) | 53.0480 | 25.6960 |
| `independent_throughput/reduction/variant=multi_accum16,k=512,rows=1,wg=256,body=32;hip-wave=32` | 2 | vulkan (+303.53%) | 64.8720 | 16.0760 |
| `independent_throughput/reduction/variant=multi_accum16,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | vulkan (+54.40%) | 105.5920 | 68.3880 |
| `independent_throughput/reduction/variant=multi_accum16,k=2048,rows=1,wg=256,body=32;hip-wave=32` | 2 | vulkan (+106.92%) | 53.3200 | 25.7680 |
| `independent_throughput/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+267.44%) | 57.6000 | 15.6760 |
| `independent_throughput/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+282.26%) | 61.1000 | 15.9840 |
| `independent_throughput/memory-waitcnt/variant=strided4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+337.84%) | 61.1920 | 13.9760 |
| `independent_throughput/memory-waitcnt/variant=strided4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+338.92%) | 61.1680 | 13.9360 |
| `independent_throughput/memory-waitcnt/variant=gather1,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+127.14%) | 121.4640 | 53.4760 |
| `independent_throughput/memory-waitcnt/variant=gather1,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+129.17%) | 122.6160 | 53.5040 |
| `independent_throughput/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=64;hip-wave=64` | 2 | vulkan (+102.09%) | 66.9400 | 33.1240 |
| `independent_throughput/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=256;hip-wave=64` | 2 | vulkan (+200.76%) | 99.6000 | 33.1160 |
| `independent_throughput/vopd/variant=independent_fma,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 2 | vulkan (+16.05%) | 123.2280 | 106.1880 |
| `independent_throughput/vopd/variant=independent_fma,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 2 | vulkan (+6.68%) | 120.6160 | 113.0600 |
| `independent_throughput/vopd/variant=dependent_fma,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 2 | vulkan (+8.67%) | 114.1960 | 105.0840 |
| `independent_throughput/vopd/variant=dependent_fma,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 2 | vulkan (+13.86%) | 121.2520 | 106.4880 |
| `independent_throughput/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 2 | vulkan (+32.99%) | 282.1400 | 212.1440 |
| `independent_throughput/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 2 | vulkan (+30.94%) | 279.0200 | 213.0880 |
| `independent_throughput/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 2 | vulkan (+18.69%) | 224.0120 | 188.7440 |
| `independent_throughput/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 2 | vulkan (+15.94%) | 220.7720 | 190.4120 |
| `independent_throughput/sampler/top-k=1,vocab=32768,rows=1,wg=64;hip-wave=32` | 2 | vulkan (+86.62%) | 99.5720 | 53.3560 |
| `independent_throughput/sampler/top-k=1,vocab=32768,rows=1,wg=256;hip-wave=32` | 2 | vulkan (+158.58%) | 52.8840 | 20.4520 |
| `independent_throughput/sampler/top-k=1,vocab=32768,rows=4,wg=64;hip-wave=32` | 2 | vulkan (+84.19%) | 99.2840 | 53.9040 |
| `independent_throughput/sampler/top-k=1,vocab=32768,rows=4,wg=256;hip-wave=32` | 2 | vulkan (+159.69%) | 52.8320 | 20.3440 |
| `independent_throughput/sampler/top-k=1,vocab=32768,rows=8,wg=64;hip-wave=32` | 2 | vulkan (+84.31%) | 99.3440 | 53.9000 |
| `independent_throughput/sampler/top-k=1,vocab=32768,rows=8,wg=256;hip-wave=32` | 2 | vulkan (+160.51%) | 52.9360 | 20.3200 |
| `independent_throughput/two-stage-reduction/k=8192,rows=1,splits=2,wg=128,body=16;hip-wave=32` | 2 | vulkan (+214.03%) | 60.0560 | 19.1240 |
| `independent_throughput/two-stage-reduction/k=8192,rows=1,splits=4,wg=128,body=16;hip-wave=32` | 2 | vulkan (+245.22%) | 55.1240 | 15.9680 |
| `independent_throughput/two-stage-reduction/k=8192,rows=1,splits=2,wg=256,body=16;hip-wave=32` | 2 | vulkan (+256.91%) | 56.3200 | 15.7800 |
| `independent_throughput/two-stage-reduction/k=8192,rows=1,splits=4,wg=256,body=16;hip-wave=32` | 2 | vulkan (+387.96%) | 68.5880 | 14.0560 |
| `independent_throughput/two-stage-reduction/k=8192,rows=4,splits=2,wg=128,body=16;hip-wave=32` | 2 | vulkan (+285.65%) | 70.9440 | 18.3960 |
| `independent_throughput/two-stage-reduction/k=8192,rows=4,splits=4,wg=128,body=16;hip-wave=32` | 2 | vulkan (+297.81%) | 61.0720 | 15.3520 |
| `independent_throughput/two-stage-reduction/k=8192,rows=4,splits=2,wg=256,body=16;hip-wave=32` | 2 | vulkan (+312.05%) | 62.0880 | 15.0680 |
| `independent_throughput/two-stage-reduction/k=8192,rows=4,splits=4,wg=256,body=16;hip-wave=32` | 2 | vulkan (+353.27%) | 61.7360 | 13.6200 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=2,wg=128,body=16;hip-wave=32` | 2 | vulkan (+99.49%) | 69.5040 | 34.8400 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=4,wg=128,body=16;hip-wave=32` | 2 | vulkan (+159.18%) | 59.7160 | 23.0400 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=2,wg=256,body=16;hip-wave=32` | 2 | vulkan (+162.89%) | 59.4240 | 22.6040 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=4,wg=256,body=16;hip-wave=32` | 2 | vulkan (+248.82%) | 62.1600 | 17.8200 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=2,wg=128,body=16;hip-wave=32` | 2 | vulkan (+190.65%) | 100.4600 | 34.5640 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=4,wg=128,body=16;hip-wave=32` | 2 | vulkan (+220.28%) | 73.6520 | 22.9960 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=2,wg=256,body=16;hip-wave=32` | 2 | vulkan (+166.58%) | 60.2800 | 22.6120 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=4,wg=256,body=16;hip-wave=32` | 2 | vulkan (+180.66%) | 50.4960 | 17.9920 |
| `independent_throughput/q4-selected-dual/operation=q8_1_quantize,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=32;hip-wave=32` | 2 | vulkan (+715.52%) | 60.3160 | 7.3960 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=64` | 2 | vulkan (+106.84%) | 120.3320 | 58.1760 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+178.34%) | 182.1680 | 65.4480 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=64` | 2 | vulkan (+89.61%) | 137.3520 | 72.4400 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+180.49%) | 214.1920 | 76.3640 |
| `independent_throughput/q6-x8-selected-down/operation=q8_1_quantize,rows=8,experts=256,in=512,out=2048,wg=32;hip-wave=32` | 2 | vulkan (+658.08%) | 62.2840 | 8.2160 |
| `independent_throughput/q6-x8-selected-down/operation=x8_selected_dp4a_dot_prequantized,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=64` | 2 | vulkan (+336.50%) | 67.9720 | 15.5720 |
| `independent_throughput/q6-x8-selected-down/operation=x8_selected_dp4a_quantize_plus_dot,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+233.69%) | 63.9880 | 19.1760 |
| `independent_throughput/dense-q8/operation=q8_1_quantize,in=768,out=2048,rows=1,wg=32;hip-wave=32` | 2 | vulkan (+568.05%) | 55.3680 | 8.2880 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+497.00%) | 62.8520 | 10.5280 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+312.69%) | 57.1000 | 13.8360 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+489.49%) | 62.8160 | 10.6560 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+278.21%) | 50.0000 | 13.2200 |
| `independent_throughput/dense-q8/operation=q8_1_quantize,in=768,out=2048,rows=4,wg=32;hip-wave=32` | 2 | vulkan (+652.59%) | 62.1640 | 8.2600 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+290.07%) | 60.1800 | 15.4280 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+144.46%) | 51.5520 | 21.0880 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+345.17%) | 60.1160 | 13.5040 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+216.36%) | 50.8200 | 16.0640 |
| `independent_throughput/dense-q8/operation=q8_1_quantize,in=2048,out=2048,rows=1,wg=32;hip-wave=32` | 2 | vulkan (+668.15%) | 62.3120 | 8.1120 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+388.30%) | 59.0840 | 12.1000 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+300.59%) | 60.2000 | 15.0280 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+260.98%) | 49.7280 | 13.7760 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+257.97%) | 61.3840 | 17.1480 |
| `independent_throughput/dense-q8/operation=q8_1_quantize,in=2048,out=2048,rows=4,wg=32;hip-wave=32` | 2 | vulkan (+576.31%) | 55.2680 | 8.1720 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+77.88%) | 49.9200 | 28.0640 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+59.14%) | 51.4480 | 32.3280 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+128.33%) | 55.1560 | 24.1560 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+118.27%) | 58.6000 | 26.8480 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 101 | 139 | 0 | 42.08 | 1.1317 | 240 |

## Pinned hipEngine-harness comparison

The `hipengine` profile covers the same benchmark row set as pinned HipEngine `f2c3ad6`: family, operation, shape/sweep axes, repetition count, and serial/independent mode. Those rows deliberately run through Hipfire's existing Radiowave-tuned launch policy; wave size, workgroup geometry, source variant, ABI, and machine code remain optimization variables rather than parity constraints. HipEngine has 212 three-way matched core rows because its HIP independent sampler path rejects 12 rows; its 16 dispatch controls are reported separately. Hipfire executes and correctness-gates all 240 rows across four backends.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine core | RL / vulkan | 192 | 20 | 90.57 | 0.4589 | 212 |
| hipEngine core | RL / hip | 158 | 54 | 74.53 | 0.8343 | 212 |
| hipEngine dispatch | RL / vulkan | 6 | 10 | 37.50 | 1.4331 | 16 |
| hipEngine dispatch | RL / hip | 16 | 0 | 100.00 | 0.2249 | 16 |

## Harness verdict

This tuning smoke intentionally measures only `redline` versus `vulkan`. It uses the `hipengine_f2c` matrix, `radiowave_tuned` wave policy, and `radiowave-recipe-or-default` scheduler profile. Every ranked row passed both CPU oracles. Redline beats Vulkan in 101/240 rows; final promotion still requires a full four-backend certification run.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. The `hipengine_f2c` profile matches HipEngine's row coverage while intentionally retaining Hipfire/Radiowave's tuned wave, workgroup, and source-variant choices. This artifact records the `radiowave_tuned` wave policy and `radiowave-recipe-or-default` scheduler profile as controlled HIP compilation/launch factors. Every completion timestamp necessarily proves the measured work finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
