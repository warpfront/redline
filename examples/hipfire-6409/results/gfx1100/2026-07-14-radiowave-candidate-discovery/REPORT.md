# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 160/234 rows (68.38%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 160 | 56 | 10 | 8 | 68.38 | 234 |
| vulkan | 52 | 111 | 10 | 61 | 22.22 | 234 |
| hipgraph | 8 | 26 | 87 | 113 | 3.42 | 234 |
| hip | 14 | 41 | 127 | 52 | 5.98 | 234 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 84 | 28 | 2 | 3 | 117 |
| serial_latency | vulkan | 16 | 36 | 8 | 57 | 117 |
| serial_latency | hipgraph | 7 | 22 | 69 | 19 | 117 |
| serial_latency | hip | 10 | 31 | 38 | 38 | 117 |
| independent_throughput | redline | 76 | 28 | 8 | 5 | 117 |
| independent_throughput | vulkan | 36 | 75 | 2 | 4 | 117 |
| independent_throughput | hipgraph | 1 | 4 | 18 | 94 | 117 |
| independent_throughput | hip | 4 | 10 | 89 | 14 | 117 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/dispatch-grid/sweep=count,count=1,grid=1;hip-wave=32` | 2 | vulkan (+2.94%) | 9.8000 | 9.5200 |
| `serial_latency/dispatch-grid/sweep=count,count=50,grid=1;hip-wave=32` | 2 | vulkan (+5.19%) | 2.3168 | 2.2024 |
| `serial_latency/reduction/variant=lds_tree,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hip (+1.12%) | 70.0200 | 69.2477 |
| `serial_latency/reduction/variant=extra_barrier,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hip (+0.81%) | 69.9520 | 69.3917 |
| `serial_latency/reduction/variant=multi_accum4,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hip (+1.07%) | 72.6800 | 71.9116 |
| `serial_latency/reduction/variant=multi_accum8,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hip (+0.91%) | 77.3360 | 76.6397 |
| `serial_latency/reduction/variant=multi_accum16,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hip (+1.98%) | 77.0920 | 75.5957 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=64;hip-wave=32` | 3 | vulkan (+43.59%), hip (+18.43%) | 26.5240 | 18.4720 |
| `serial_latency/memory-waitcnt/variant=strided4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+2.33%) | 13.5200 | 13.2120 |
| `serial_latency/memory-waitcnt/variant=strided4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+26.24%) | 18.7800 | 14.8760 |
| `serial_latency/memory-waitcnt/variant=gather1,n=32768,body=64,wg=64;hip-wave=32` | 4 | hip (+45.99%), hipgraph (+24.36%), vulkan (+23.64%) | 66.0960 | 45.2758 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=64;hip-wave=64` | 2 | vulkan (+10.31%) | 22.6800 | 20.5600 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=256;hip-wave=64` | 2 | vulkan (+0.26%) | 19.8800 | 19.8280 |
| `serial_latency/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+0.38%) | 290.2200 | 289.1120 |
| `serial_latency/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+0.37%) | 289.8360 | 288.7640 |
| `serial_latency/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+0.62%) | 290.5160 | 288.7280 |
| `serial_latency/vopd/variant=independent_fma,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 3 | hip (+49.81%), hipgraph (+31.33%) | 89.4280 | 59.6958 |
| `serial_latency/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 4 | hip (+69.85%), hipgraph (+55.54%), vulkan (+34.93%) | 193.4160 | 113.8715 |
| `serial_latency/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 2 | hipgraph (+14.95%) | 113.0160 | 98.3196 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=1,wg=64;hip-wave=32` | 4 | hipgraph (+14.55%), hip (+11.65%), vulkan (+7.63%) | 131.6840 | 114.9595 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=1,wg=64;hip-wave=32` | 2 | hipgraph (+0.27%) | 703.5360 | 701.6287 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=1,wg=256;hip-wave=32` | 2 | hip (+11.33%) | 29.9400 | 26.8919 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=1,wg=256;hip-wave=32` | 2 | hipgraph (+0.70%) | 202.3920 | 200.9871 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=4,wg=64;hip-wave=32` | 2 | hipgraph (+1.74%) | 118.0200 | 116.0034 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=4,wg=64;hip-wave=32` | 2 | hipgraph (+0.19%) | 708.4520 | 707.1248 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=4,wg=256;hip-wave=32` | 2 | hip (+9.26%) | 30.1520 | 27.5959 |
| `serial_latency/sampler/top-k=8,vocab=32768,rows=4,wg=256;hip-wave=32` | 2 | hipgraph (+0.97%) | 204.8840 | 202.9111 |
| `serial_latency/q4-selected-dual/operation=q8_1_quantize,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=32;hip-wave=32` | 2 | vulkan (+19.55%) | 2.1280 | 1.7800 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+29.08%) | 58.5040 | 45.3240 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+119.92%) | 103.7600 | 47.1800 |
| `serial_latency/q6-x8-selected-down/operation=q8_1_quantize,rows=8,experts=256,in=512,out=2048,wg=32;hip-wave=32` | 2 | vulkan (+9.67%) | 1.8600 | 1.6960 |
| `serial_latency/q6-x8-selected-down/operation=x8_selected_dp4a_quantize_plus_dot,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+152.49%) | 54.5480 | 21.6040 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=2048,out=2048,rows=4,wg=32;hip-wave=32` | 2 | vulkan (+8.94%) | 1.8520 | 1.7000 |
| `independent_throughput/dispatch-grid/sweep=count,count=941,grid=1;hip-wave=32` | 2 | vulkan (+239.50%) | 0.2247 | 0.0662 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=1;hip-wave=32` | 2 | vulkan (+103.40%) | 0.2263 | 0.1112 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=128;hip-wave=32` | 2 | vulkan (+350.88%) | 0.5108 | 0.1133 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=1024;hip-wave=32` | 2 | vulkan (+264.00%) | 0.8303 | 0.2281 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=8192;hip-wave=32` | 2 | vulkan (+130.64%) | 3.3678 | 1.4602 |
| `independent_throughput/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+8.83%) | 14.2000 | 13.0480 |
| `independent_throughput/memory-waitcnt/variant=strided4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+45.70%) | 15.9920 | 10.9760 |
| `independent_throughput/memory-waitcnt/variant=gather1,n=32768,body=64,wg=64;hip-wave=32` | 3 | vulkan (+14.66%), hip (+8.60%) | 88.1720 | 76.9000 |
| `independent_throughput/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+23.72%), hipgraph (+15.86%), hip (+11.16%) | 295.7400 | 239.0480 |
| `independent_throughput/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 3 | hip (+4.68%), hipgraph (+1.30%) | 263.6240 | 251.8388 |
| `independent_throughput/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | hipgraph (+16.56%), hip (+14.74%), vulkan (+5.13%) | 295.7000 | 253.6788 |
| `independent_throughput/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 3 | hip (+4.74%), hipgraph (+1.39%) | 264.7560 | 252.7708 |
| `independent_throughput/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+17.85%), hipgraph (+16.28%), hip (+15.18%) | 301.2120 | 255.5800 |
| `independent_throughput/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 3 | vulkan (+7.82%), hip (+1.74%) | 255.9760 | 237.4200 |
| `independent_throughput/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 2 | hip (+5.18%) | 247.6680 | 235.4669 |
| `independent_throughput/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | hip (+1.28%) | 245.6920 | 242.5869 |
| `independent_throughput/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 3 | vulkan (+21.07%), hip (+7.98%) | 155.6880 | 128.5960 |
| `independent_throughput/two-stage-reduction/k=8192,rows=1,splits=2,wg=128,body=16;hip-wave=32` | 2 | vulkan (+2.94%) | 5.7400 | 5.5760 |
| `independent_throughput/two-stage-reduction/k=8192,rows=1,splits=4,wg=128,body=16;hip-wave=32` | 2 | vulkan (+8.78%) | 5.0560 | 4.6480 |
| `independent_throughput/two-stage-reduction/k=8192,rows=1,splits=2,wg=256,body=16;hip-wave=32` | 2 | vulkan (+48.70%) | 5.4840 | 3.6880 |
| `independent_throughput/two-stage-reduction/k=8192,rows=4,splits=2,wg=128,body=16;hip-wave=32` | 2 | vulkan (+26.55%) | 9.3600 | 7.3960 |
| `independent_throughput/two-stage-reduction/k=8192,rows=4,splits=4,wg=128,body=16;hip-wave=32` | 2 | vulkan (+15.88%) | 8.4640 | 7.3040 |
| `independent_throughput/two-stage-reduction/k=8192,rows=4,splits=4,wg=256,body=16;hip-wave=32` | 2 | vulkan (+40.09%) | 6.5280 | 4.6600 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=2,wg=128,body=16;hip-wave=32` | 4 | vulkan (+128.81%), hip (+34.16%), hipgraph (+7.77%) | 40.8480 | 17.8520 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=4,wg=128,body=16;hip-wave=32` | 3 | vulkan (+79.19%), hip (+3.07%) | 21.5240 | 12.0120 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=2,wg=256,body=16;hip-wave=32` | 3 | vulkan (+69.40%), hip (+1.86%) | 21.6560 | 12.7840 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=4,wg=256,body=16;hip-wave=32` | 2 | vulkan (+30.87%) | 13.0920 | 10.0040 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=2,wg=128,body=16;hip-wave=32` | 4 | vulkan (+125.00%), hip (+34.09%), hipgraph (+4.93%) | 41.4000 | 18.4000 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=4,wg=128,body=16;hip-wave=32` | 3 | vulkan (+74.56%), hip (+1.18%) | 21.6800 | 12.4200 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=2,wg=256,body=16;hip-wave=32` | 2 | vulkan (+76.67%) | 21.7800 | 12.3280 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=4,wg=256,body=16;hip-wave=32` | 2 | vulkan (+33.80%) | 13.4280 | 10.0360 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+117.80%) | 60.7920 | 27.9120 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+155.46%) | 111.1040 | 43.4920 |
| `independent_throughput/q6-x8-selected-down/operation=x8_selected_dp4a_quantize_plus_dot,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+349.02%) | 51.1160 | 11.3840 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+11.39%) | 7.1560 | 6.4240 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+58.78%) | 11.3240 | 7.1320 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+3.98%) | 12.8480 | 12.3560 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+11.46%) | 10.1920 | 9.1440 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+6.15%) | 15.1160 | 14.2400 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+88.87%) | 25.1720 | 13.3280 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+93.42%) | 59.2400 | 30.6280 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 178 | 56 | 0 | 76.07 | 0.7477 | 234 |
| RL / hipgraph | 217 | 17 | 0 | 92.74 | 0.4607 | 234 |
| RL / hip | 207 | 27 | 0 | 88.46 | 0.5259 | 234 |

## Pinned hipEngine-harness comparison

The `hipengine` profile covers the same benchmark row set as pinned HipEngine `f2c3ad6`: family, operation, shape/sweep axes, repetition count, and serial/independent mode. Those rows deliberately run through Hipfire's existing Radiowave-tuned launch policy; wave size, workgroup geometry, source variant, ABI, and machine code remain optimization variables rather than parity constraints. HipEngine has 212 three-way matched core rows because its HIP independent sampler path rejects 12 rows; its 16 dispatch controls are reported separately. Hipfire executes and correctness-gates all 240 rows across four backends.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine core | RL / vulkan | 192 | 20 | 90.57 | 0.4589 | 212 |
| hipEngine core | RL / hip | 158 | 54 | 74.53 | 0.8343 | 212 |
| hipEngine dispatch | RL / vulkan | 6 | 10 | 37.50 | 1.4331 | 16 |
| hipEngine dispatch | RL / hip | 16 | 0 | 100.00 | 0.2249 | 16 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 207/234 rows and HipGraph in 217/234 while all three select identical per-row hipcc code objects. This run uses the `hipengine_f2c` matrix, `radiowave_tuned` wave policy, and `default` scheduler profile. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 178/234 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. The `hipengine_f2c` profile matches HipEngine's row coverage while intentionally retaining Hipfire/Radiowave's tuned wave, workgroup, and source-variant choices. This artifact records the `radiowave_tuned` wave policy and `default` scheduler profile as controlled HIP compilation/launch factors. Every completion timestamp necessarily proves the measured work finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
