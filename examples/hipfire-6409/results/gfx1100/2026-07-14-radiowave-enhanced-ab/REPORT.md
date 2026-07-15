# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 157/240 rows (65.42%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 157 | 69 | 3 | 11 | 65.42 | 240 |
| vulkan | 63 | 102 | 10 | 65 | 26.25 | 240 |
| hipgraph | 7 | 45 | 80 | 108 | 2.92 | 240 |
| hip | 13 | 24 | 147 | 56 | 5.42 | 240 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 85 | 30 | 2 | 3 | 120 |
| serial_latency | vulkan | 21 | 31 | 5 | 63 | 120 |
| serial_latency | hipgraph | 5 | 41 | 57 | 17 | 120 |
| serial_latency | hip | 9 | 18 | 56 | 37 | 120 |
| independent_throughput | redline | 72 | 39 | 1 | 8 | 120 |
| independent_throughput | vulkan | 42 | 71 | 5 | 2 | 120 |
| independent_throughput | hipgraph | 2 | 4 | 23 | 91 | 120 |
| independent_throughput | hip | 4 | 6 | 91 | 19 | 120 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/dispatch-grid/sweep=count,count=1,grid=1;hip-wave=32` | 2 | vulkan (+0.84%) | 9.5600 | 9.4800 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=8192;hip-wave=32` | 2 | vulkan (+3.33%) | 5.8147 | 5.6275 |
| `serial_latency/reduction/variant=lds_tree,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hip (+0.86%) | 70.2520 | 69.6521 |
| `serial_latency/reduction/variant=extra_barrier,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hip (+0.72%) | 70.2560 | 69.7561 |
| `serial_latency/reduction/variant=multi_accum4,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hip (+0.89%) | 72.8560 | 72.2121 |
| `serial_latency/reduction/variant=multi_accum8,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hip (+1.24%) | 77.0440 | 76.1040 |
| `serial_latency/reduction/variant=multi_accum16,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hip (+1.20%) | 76.4720 | 75.5680 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=64;hip-wave=32` | 3 | vulkan (+36.18%), hip (+3.60%) | 23.3840 | 17.1720 |
| `serial_latency/memory-waitcnt/variant=strided4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+3.00%) | 13.1640 | 12.7800 |
| `serial_latency/memory-waitcnt/variant=strided4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+7.28%) | 14.3320 | 13.3600 |
| `serial_latency/memory-waitcnt/variant=gather1,n=32768,body=64,wg=64;hip-wave=32` | 4 | hip (+26.89%), vulkan (+17.01%), hipgraph (+13.81%) | 48.5160 | 38.2360 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+20.20%) | 19.7760 | 16.4520 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+27.24%) | 22.4800 | 17.6680 |
| `serial_latency/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+0.48%) | 290.1760 | 288.8000 |
| `serial_latency/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+0.39%) | 289.9120 | 288.7920 |
| `serial_latency/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+0.55%) | 290.3600 | 288.7720 |
| `serial_latency/vopd/variant=independent_fma,accums=4,n=65536,body=512,wg=64;hip-wave=32` | 4 | hip (+61.67%), hipgraph (+36.06%), vulkan (+11.12%) | 82.4320 | 50.9880 |
| `serial_latency/vopd/variant=dependent_fma,accums=4,n=65536,body=512,wg=256;hip-wave=32` | 2 | vulkan (+2.26%) | 61.4600 | 60.1040 |
| `serial_latency/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=64;hip-wave=32` | 4 | hipgraph (+47.81%), hip (+37.87%), vulkan (+37.24%) | 157.6080 | 106.6320 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=1,wg=64;hip-wave=32` | 2 | hipgraph (+0.06%) | 702.0880 | 701.6405 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=1,wg=256;hip-wave=32` | 2 | hip (+10.51%) | 29.5680 | 26.7560 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=1,wg=256;hip-wave=32` | 2 | hipgraph (+0.34%) | 201.2160 | 200.5281 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=4,wg=256;hip-wave=32` | 2 | hip (+10.40%) | 29.9720 | 27.1480 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=8,wg=256;hip-wave=32` | 3 | hipgraph (+1.08%), hip (+1.04%) | 29.0840 | 28.7720 |
| `serial_latency/two-stage-reduction/k=32768,rows=4,splits=2,wg=128,body=16;hip-wave=32` | 2 | hipgraph (+2.58%) | 23.6760 | 23.0800 |
| `serial_latency/q4-selected-dual/operation=q8_1_quantize,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=32;hip-wave=32` | 2 | vulkan (+4.94%) | 1.7840 | 1.7000 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+1.98%) | 59.0960 | 57.9480 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+26.05%) | 58.8000 | 46.6480 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+84.21%) | 67.1720 | 36.4640 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+112.15%) | 94.9160 | 44.7400 |
| `serial_latency/q6-x8-selected-down/operation=q8_1_quantize,rows=8,experts=256,in=512,out=2048,wg=32;hip-wave=32` | 2 | vulkan (+9.88%) | 1.8680 | 1.7000 |
| `serial_latency/q6-x8-selected-down/operation=x8_selected_dp4a_dot_prequantized,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+220.99%) | 59.4480 | 18.5200 |
| `serial_latency/q6-x8-selected-down/operation=x8_selected_dp4a_quantize_plus_dot,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+138.82%) | 61.4720 | 25.7400 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+23.10%) | 8.4200 | 6.8400 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=2048,out=2048,rows=4,wg=32;hip-wave=32` | 2 | vulkan (+9.22%) | 1.8480 | 1.6920 |
| `independent_throughput/dispatch-grid/sweep=count,count=941,grid=1;hip-wave=32` | 2 | vulkan (+215.65%) | 0.1963 | 0.0622 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=1;hip-wave=32` | 2 | vulkan (+93.82%) | 0.1945 | 0.1004 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=128;hip-wave=32` | 2 | vulkan (+991.45%) | 0.9604 | 0.0880 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=1024;hip-wave=32` | 2 | vulkan (+352.61%) | 2.6709 | 0.5901 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=8192;hip-wave=32` | 2 | vulkan (+92.84%) | 8.3793 | 4.3453 |
| `independent_throughput/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+14.46%) | 11.8400 | 10.3440 |
| `independent_throughput/memory-waitcnt/variant=strided4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+8.29%) | 11.4960 | 10.6160 |
| `independent_throughput/memory-waitcnt/variant=strided4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+5.45%) | 10.1360 | 9.6120 |
| `independent_throughput/memory-waitcnt/variant=gather1,n=32768,body=64,wg=64;hip-wave=32` | 4 | hip (+33.31%), vulkan (+25.10%), hipgraph (+22.16%) | 71.1920 | 53.4040 |
| `independent_throughput/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+19.75%), hipgraph (+16.64%), hip (+14.60%) | 296.1880 | 247.3320 |
| `independent_throughput/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 4 | hip (+3.43%), hipgraph (+2.00%), vulkan (+0.71%) | 265.0400 | 256.2403 |
| `independent_throughput/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | hipgraph (+16.89%), hip (+16.48%), vulkan (+5.63%) | 296.8240 | 253.9403 |
| `independent_throughput/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 4 | vulkan (+10.90%), hip (+2.86%), hipgraph (+2.18%) | 264.7600 | 238.7400 |
| `independent_throughput/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | hipgraph (+17.62%), hip (+17.30%), vulkan (+11.18%) | 300.9000 | 255.8163 |
| `independent_throughput/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 4 | vulkan (+11.97%), hip (+2.96%), hipgraph (+1.60%) | 263.3040 | 235.1640 |
| `independent_throughput/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 3 | hip (+9.54%), hipgraph (+1.56%) | 250.2840 | 228.4923 |
| `independent_throughput/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | hip (+1.15%) | 246.1400 | 243.3523 |
| `independent_throughput/vopd/variant=independent_fma,accums=4,n=65536,body=512,wg=64;hip-wave=32` | 2 | vulkan (+21.33%) | 78.7040 | 64.8680 |
| `independent_throughput/vopd/variant=dependent_fma,accums=4,n=65536,body=512,wg=64;hip-wave=32` | 2 | vulkan (+1.63%) | 53.4640 | 52.6040 |
| `independent_throughput/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=64;hip-wave=32` | 4 | vulkan (+36.78%), hip (+24.91%), hipgraph (+2.22%) | 140.4600 | 102.6880 |
| `independent_throughput/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=64;hip-wave=32` | 2 | vulkan (+8.58%) | 83.4320 | 76.8360 |
| `independent_throughput/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=256;hip-wave=32` | 2 | vulkan (+15.19%) | 89.1440 | 77.3920 |
| `independent_throughput/two-stage-reduction/k=8192,rows=1,splits=2,wg=128,body=16;hip-wave=32` | 2 | vulkan (+25.42%) | 6.0200 | 4.8000 |
| `independent_throughput/two-stage-reduction/k=8192,rows=1,splits=4,wg=128,body=16;hip-wave=32` | 2 | vulkan (+34.09%) | 4.7360 | 3.5320 |
| `independent_throughput/two-stage-reduction/k=8192,rows=4,splits=2,wg=128,body=16;hip-wave=32` | 2 | vulkan (+22.56%) | 7.1280 | 5.8160 |
| `independent_throughput/two-stage-reduction/k=8192,rows=4,splits=4,wg=128,body=16;hip-wave=32` | 2 | vulkan (+62.25%) | 5.8280 | 3.5920 |
| `independent_throughput/two-stage-reduction/k=8192,rows=4,splits=4,wg=256,body=16;hip-wave=32` | 2 | vulkan (+26.57%) | 5.0120 | 3.9600 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=2,wg=128,body=16;hip-wave=32` | 2 | vulkan (+45.57%) | 20.0480 | 13.7720 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=4,wg=128,body=16;hip-wave=32` | 2 | vulkan (+22.50%) | 11.0200 | 8.9960 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=2,wg=256,body=16;hip-wave=32` | 2 | vulkan (+34.98%) | 10.7280 | 7.9480 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=4,wg=256,body=16;hip-wave=32` | 2 | vulkan (+37.28%) | 6.7760 | 4.9360 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=2,wg=128,body=16;hip-wave=32` | 2 | vulkan (+27.73%) | 18.5920 | 14.5560 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=4,wg=128,body=16;hip-wave=32` | 2 | vulkan (+41.04%) | 11.3000 | 8.0120 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=2,wg=256,body=16;hip-wave=32` | 2 | vulkan (+35.38%) | 10.8680 | 8.0280 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=4,wg=256,body=16;hip-wave=32` | 2 | vulkan (+17.61%) | 7.4000 | 6.2920 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+28.75%) | 55.3320 | 42.9760 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+74.42%) | 57.1120 | 32.7440 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+117.81%) | 67.1040 | 30.8080 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+140.73%) | 101.6280 | 42.2160 |
| `independent_throughput/q6-x8-selected-down/operation=x8_selected_dp4a_dot_prequantized,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+757.73%) | 83.2000 | 9.7000 |
| `independent_throughput/q6-x8-selected-down/operation=x8_selected_dp4a_quantize_plus_dot,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+420.14%) | 53.3040 | 10.2480 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+16.02%) | 7.4160 | 6.3920 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+65.53%) | 9.5280 | 5.7560 |
| `independent_throughput/dense-q8/operation=q8_1_quantize,in=768,out=2048,rows=4,wg=32;hip-wave=32` | 2 | vulkan (+7.09%) | 2.1760 | 2.0320 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+45.00%) | 13.5840 | 9.3680 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+43.46%) | 14.2200 | 9.9120 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+28.41%) | 15.8760 | 12.3640 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+103.94%) | 47.8040 | 23.4400 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 170 | 70 | 0 | 70.83 | 0.7723 | 240 |
| RL / hipgraph | 224 | 16 | 0 | 93.33 | 0.4149 | 240 |
| RL / hip | 218 | 22 | 0 | 90.83 | 0.4633 | 240 |

## Pinned hipEngine-harness comparison

The `hipengine` profile covers the same benchmark row set as pinned HipEngine `f2c3ad6`: family, operation, shape/sweep axes, repetition count, and serial/independent mode. Those rows deliberately run through Hipfire's existing Radiowave-tuned launch policy; wave size, workgroup geometry, source variant, ABI, and machine code remain optimization variables rather than parity constraints. HipEngine has 212 three-way matched core rows because its HIP independent sampler path rejects 12 rows; its 16 dispatch controls are reported separately. Hipfire executes and correctness-gates all 240 rows across four backends.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine core | RL / vulkan | 192 | 20 | 90.57 | 0.4589 | 212 |
| hipEngine core | RL / hip | 158 | 54 | 74.53 | 0.8343 | 212 |
| hipEngine dispatch | RL / vulkan | 6 | 10 | 37.50 | 1.4331 | 16 |
| hipEngine dispatch | RL / hip | 16 | 0 | 100.00 | 0.2249 | 16 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 218/240 rows and HipGraph in 224/240 while all three select identical per-row hipcc code objects. This run uses the `hipengine_f2c` matrix, `radiowave_tuned` wave policy, and `default` scheduler profile. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 170/240 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. The `hipengine_f2c` profile matches HipEngine's row coverage while intentionally retaining Hipfire/Radiowave's tuned wave, workgroup, and source-variant choices. This artifact records the `radiowave_tuned` wave policy and `default` scheduler profile as controlled HIP compilation/launch factors. Every completion timestamp necessarily proves the measured work finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
