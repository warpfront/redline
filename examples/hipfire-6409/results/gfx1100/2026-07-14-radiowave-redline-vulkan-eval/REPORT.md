# Hipfire/Redline ROCm issue 6409 backend comparison

Correctness-gated result: **Redline wins 156/240 rows (65.00%)**.

## Placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 156 | 84 | 0 | 0 | 65.00 | 240 |
| vulkan | 84 | 156 | 0 | 0 | 35.00 | 240 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 88 | 32 | 0 | 0 | 120 |
| serial_latency | vulkan | 32 | 88 | 0 | 0 | 120 |
| independent_throughput | redline | 68 | 52 | 0 | 0 | 120 |
| independent_throughput | vulkan | 52 | 68 | 0 | 0 | 120 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/dispatch-grid/sweep=count,count=1,grid=1;hip-wave=32` | 2 | vulkan (+0.84%) | 9.6000 | 9.5200 |
| `serial_latency/dispatch-grid/sweep=count,count=50,grid=1;hip-wave=32` | 2 | vulkan (+1.29%) | 2.2552 | 2.2264 |
| `serial_latency/dispatch-grid/sweep=grid,count=941,grid=8192;hip-wave=32` | 2 | vulkan (+4.81%) | 5.7810 | 5.5156 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+41.00%) | 26.2200 | 18.5960 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+10.12%) | 28.2920 | 25.6920 |
| `serial_latency/memory-waitcnt/variant=strided4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+37.43%) | 24.1760 | 17.5920 |
| `serial_latency/memory-waitcnt/variant=strided4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+6.90%) | 21.5720 | 20.1800 |
| `serial_latency/memory-waitcnt/variant=gather1,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+34.70%) | 77.3160 | 57.4000 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+61.32%) | 32.0320 | 19.8560 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+26.31%) | 32.1880 | 25.4840 |
| `serial_latency/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+0.44%) | 289.9240 | 288.6400 |
| `serial_latency/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+0.54%) | 290.0520 | 288.4880 |
| `serial_latency/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+0.49%) | 290.4960 | 289.0800 |
| `serial_latency/vopd/variant=independent_fma,accums=4,n=65536,body=512,wg=64;hip-wave=32` | 2 | vulkan (+14.01%) | 83.3200 | 73.0840 |
| `serial_latency/vopd/variant=dependent_fma,accums=4,n=65536,body=512,wg=64;hip-wave=32` | 2 | vulkan (+25.85%) | 89.5160 | 71.1280 |
| `serial_latency/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=64;hip-wave=32` | 2 | vulkan (+39.41%) | 150.4880 | 107.9440 |
| `serial_latency/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=64;hip-wave=32` | 2 | vulkan (+37.94%) | 137.9000 | 99.9720 |
| `serial_latency/sampler/top-k=1,vocab=32768,rows=1,wg=64;hip-wave=32` | 2 | vulkan (+7.80%) | 108.3160 | 100.4800 |
| `serial_latency/q4-selected-dual/operation=q8_1_quantize,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=32;hip-wave=32` | 2 | vulkan (+21.61%) | 1.6880 | 1.3880 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+51.36%) | 81.1760 | 53.6320 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+81.80%) | 87.0040 | 47.8560 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+31.61%) | 71.8920 | 54.6240 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+124.40%) | 95.2800 | 42.4600 |
| `serial_latency/q6-x8-selected-down/operation=q8_1_quantize,rows=8,experts=256,in=512,out=2048,wg=32;hip-wave=32` | 2 | vulkan (+7.51%) | 1.6600 | 1.5440 |
| `serial_latency/q6-x8-selected-down/operation=x8_selected_dp4a_dot_prequantized,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+218.71%) | 54.9960 | 17.2560 |
| `serial_latency/q6-x8-selected-down/operation=x8_selected_dp4a_quantize_plus_dot,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+157.77%) | 58.4520 | 22.6760 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=768,out=2048,rows=1,wg=32;hip-wave=32` | 2 | vulkan (+13.08%) | 1.8680 | 1.6520 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=768,out=2048,rows=4,wg=32;hip-wave=32` | 2 | vulkan (+19.08%) | 1.8720 | 1.5720 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+1.78%) | 15.0560 | 14.7920 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+12.54%) | 15.9760 | 14.1960 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=2048,out=2048,rows=1,wg=32;hip-wave=32` | 2 | vulkan (+12.03%) | 2.4960 | 2.2280 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+5.07%) | 52.9280 | 50.3720 |
| `independent_throughput/dispatch-grid/sweep=count,count=941,grid=1;hip-wave=32` | 2 | vulkan (+293.58%) | 0.2709 | 0.0688 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=1;hip-wave=32` | 2 | vulkan (+104.76%) | 0.1903 | 0.0929 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=128;hip-wave=32` | 2 | vulkan (+1065.18%) | 0.9118 | 0.0783 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=1024;hip-wave=32` | 2 | vulkan (+439.03%) | 3.1765 | 0.5893 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=8192;hip-wave=32` | 2 | vulkan (+82.38%) | 7.9146 | 4.3397 |
| `independent_throughput/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+5.94%) | 10.7080 | 10.1080 |
| `independent_throughput/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+6.09%) | 13.0400 | 12.2920 |
| `independent_throughput/memory-waitcnt/variant=strided4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+25.21%) | 13.6080 | 10.8680 |
| `independent_throughput/memory-waitcnt/variant=gather1,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+42.97%) | 82.6960 | 57.8400 |
| `independent_throughput/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+7.99%) | 20.4440 | 18.9320 |
| `independent_throughput/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+4.24%) | 296.1000 | 284.0480 |
| `independent_throughput/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+4.58%) | 296.9000 | 283.9080 |
| `independent_throughput/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+10.84%) | 265.1440 | 239.2040 |
| `independent_throughput/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+11.46%) | 303.6600 | 272.4440 |
| `independent_throughput/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+8.89%) | 260.3400 | 239.0960 |
| `independent_throughput/vopd/variant=independent_fma,accums=4,n=65536,body=512,wg=64;hip-wave=32` | 2 | vulkan (+29.51%) | 79.0600 | 61.0440 |
| `independent_throughput/vopd/variant=dependent_fma,accums=4,n=65536,body=512,wg=64;hip-wave=32` | 2 | vulkan (+39.73%) | 88.2400 | 63.1520 |
| `independent_throughput/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=64;hip-wave=32` | 2 | vulkan (+42.73%) | 142.8680 | 100.0960 |
| `independent_throughput/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=64;hip-wave=32` | 2 | vulkan (+40.15%) | 124.6680 | 88.9560 |
| `independent_throughput/two-stage-reduction/k=8192,rows=1,splits=2,wg=128,body=16;hip-wave=32` | 2 | vulkan (+21.69%) | 5.9480 | 4.8880 |
| `independent_throughput/two-stage-reduction/k=8192,rows=1,splits=4,wg=128,body=16;hip-wave=32` | 2 | vulkan (+18.61%) | 4.3840 | 3.6960 |
| `independent_throughput/two-stage-reduction/k=8192,rows=4,splits=2,wg=128,body=16;hip-wave=32` | 2 | vulkan (+29.84%) | 6.4400 | 4.9600 |
| `independent_throughput/two-stage-reduction/k=8192,rows=4,splits=4,wg=128,body=16;hip-wave=32` | 2 | vulkan (+77.41%) | 5.3080 | 2.9920 |
| `independent_throughput/two-stage-reduction/k=8192,rows=4,splits=2,wg=256,body=16;hip-wave=32` | 2 | vulkan (+2.56%) | 4.6440 | 4.5280 |
| `independent_throughput/two-stage-reduction/k=8192,rows=4,splits=4,wg=256,body=16;hip-wave=32` | 2 | vulkan (+63.11%) | 4.4040 | 2.7000 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=2,wg=128,body=16;hip-wave=32` | 2 | vulkan (+72.27%) | 16.7720 | 9.7360 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=4,wg=128,body=16;hip-wave=32` | 2 | vulkan (+49.03%) | 9.8240 | 6.5920 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=2,wg=256,body=16;hip-wave=32` | 2 | vulkan (+53.35%) | 9.7160 | 6.3360 |
| `independent_throughput/two-stage-reduction/k=32768,rows=1,splits=4,wg=256,body=16;hip-wave=32` | 2 | vulkan (+50.97%) | 6.1960 | 4.1040 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=2,wg=128,body=16;hip-wave=32` | 2 | vulkan (+62.44%) | 17.3160 | 10.6600 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=4,wg=128,body=16;hip-wave=32` | 2 | vulkan (+76.81%) | 10.7360 | 6.0720 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=2,wg=256,body=16;hip-wave=32` | 2 | vulkan (+57.24%) | 10.1200 | 6.4360 |
| `independent_throughput/two-stage-reduction/k=32768,rows=4,splits=4,wg=256,body=16;hip-wave=32` | 2 | vulkan (+45.48%) | 7.1400 | 4.9080 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+91.10%) | 78.3440 | 40.9960 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+136.20%) | 84.1520 | 35.6280 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+75.30%) | 81.3720 | 46.4200 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+174.65%) | 96.1720 | 35.0160 |
| `independent_throughput/q6-x8-selected-down/operation=x8_selected_dp4a_dot_prequantized,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+696.09%) | 77.3800 | 9.7200 |
| `independent_throughput/q6-x8-selected-down/operation=x8_selected_dp4a_quantize_plus_dot,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+345.95%) | 53.0680 | 11.9000 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+26.90%) | 3.1320 | 2.4680 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+31.23%) | 7.6640 | 5.8400 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+102.58%) | 11.0120 | 5.4360 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+21.53%) | 9.2560 | 7.6160 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+93.21%) | 18.9960 | 9.8320 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+10.33%) | 10.8560 | 9.8400 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+36.61%) | 21.0320 | 15.3960 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+6.56%) | 6.1040 | 5.7280 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=1,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+17.39%) | 12.8520 | 10.9480 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+6.21%) | 7.4600 | 7.0240 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=1,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+160.82%) | 24.5480 | 9.4120 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+19.13%) | 30.4360 | 25.5480 |
| `independent_throughput/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+71.84%) | 52.1560 | 30.3520 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 156 | 84 | 0 | 65.00 | 0.8040 | 240 |

## Pinned hipEngine-harness comparison

The `hipengine` profile covers the same benchmark row set as pinned HipEngine `f2c3ad6`: family, operation, shape/sweep axes, repetition count, and serial/independent mode. Those rows deliberately run through Hipfire's existing Radiowave-tuned launch policy; wave size, workgroup geometry, source variant, ABI, and machine code remain optimization variables rather than parity constraints. HipEngine has 212 three-way matched core rows because its HIP independent sampler path rejects 12 rows; its 16 dispatch controls are reported separately. Hipfire executes and correctness-gates all 240 rows across four backends.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine core | RL / vulkan | 192 | 20 | 90.57 | 0.4589 | 212 |
| hipEngine core | RL / hip | 158 | 54 | 74.53 | 0.8343 | 212 |
| hipEngine dispatch | RL / vulkan | 6 | 10 | 37.50 | 1.4331 | 16 |
| hipEngine dispatch | RL / hip | 16 | 0 | 100.00 | 0.2249 | 16 |

## Harness verdict

This tuning smoke intentionally measures only `redline` versus `vulkan`. It uses the `hipengine_f2c` matrix, `radiowave_tuned` wave policy, and `default` scheduler profile. Every ranked row passed both CPU oracles. Redline beats Vulkan in 156/240 rows; final promotion still requires a full four-backend certification run.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. The `hipengine_f2c` profile matches HipEngine's row coverage while intentionally retaining Hipfire/Radiowave's tuned wave, workgroup, and source-variant choices. This artifact records the `radiowave_tuned` wave policy and `default` scheduler profile as controlled HIP compilation/launch factors. Every completion timestamp necessarily proves the measured work finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
