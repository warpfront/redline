# Hipfire/Redline ROCm issue 6409 backend comparison

Correctness-gated result: **Redline wins 128/164 rows (78.05%)**.

## Placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 128 | 32 | 4 | 0 | 78.05 | 164 |
| hip | 8 | 53 | 103 | 0 | 4.88 | 164 |
| vulkan | 28 | 79 | 57 | 0 | 17.07 | 164 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 90 | 26 | 4 | 0 | 120 |
| serial_latency | hip | 8 | 53 | 59 | 0 | 120 |
| serial_latency | vulkan | 22 | 41 | 57 | 0 | 120 |
| independent_throughput | redline | 38 | 6 | 0 | 0 | 44 |
| independent_throughput | hip | 0 | 0 | 44 | 0 | 44 |
| independent_throughput | vulkan | 6 | 38 | 0 | 0 | 44 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/dispatch-grid/sweep=count,count=50,grid=1;hip-wave=32` | 2 | vulkan (+0.25%) | 2.2760 | 2.2704 |
| `serial_latency/geometry/k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hip (+1.25%) | 63.3640 | 62.5799 |
| `serial_latency/geometry/k=2048,rows=4,wg=64,body=32;hip-wave=32` | 2 | hip (+1.30%) | 63.5800 | 62.7639 |
| `serial_latency/reduction/variant=lds_tree,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hip (+1.13%) | 67.7640 | 67.0079 |
| `serial_latency/reduction/variant=extra_barrier,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hip (+0.98%) | 67.7320 | 67.0759 |
| `serial_latency/reduction/variant=wave_shuffle,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hip (+1.22%) | 63.2800 | 62.5199 |
| `serial_latency/reduction/variant=multi_accum4,k=2048,rows=1,wg=64,body=32;hip-wave=32` | 2 | hip (+8.06%) | 80.3080 | 74.3198 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+4.02%) | 21.9600 | 21.1120 |
| `serial_latency/memory-waitcnt/variant=strided4,n=32768,body=64,wg=64;hip-wave=32` | 3 | vulkan (+62.92%), hip (+2.59%) | 24.2160 | 14.8640 |
| `serial_latency/memory-waitcnt/variant=gather1,n=32768,body=64,wg=64;hip-wave=32` | 2 | hip (+6.75%) | 64.7520 | 60.6599 |
| `serial_latency/memory-waitcnt/variant=gather1,n=32768,body=64,wg=256;hip-wave=32` | 3 | vulkan (+61.68%), hip (+13.29%) | 86.3040 | 53.3800 |
| `serial_latency/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+0.31%) | 290.1680 | 289.2720 |
| `serial_latency/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+0.38%) | 289.8280 | 288.7400 |
| `serial_latency/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+0.40%) | 290.1800 | 289.0240 |
| `serial_latency/vopd/variant=independent_fma,accums=4,n=65536,body=512,wg=256;hip-wave=32` | 2 | vulkan (+51.20%) | 84.8280 | 56.1040 |
| `serial_latency/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=64;hip-wave=32` | 3 | vulkan (+41.70%), hip (+18.60%) | 148.2920 | 104.6520 |
| `serial_latency/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=64;hip-wave=32` | 2 | hip (+1.59%) | 96.8920 | 95.3758 |
| `serial_latency/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=256;hip-wave=32` | 3 | vulkan (+41.48%), hip (+7.55%) | 142.6040 | 100.7960 |
| `serial_latency/q4-selected-dual/operation=q8_1_quantize,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=32;hip-wave=32` | 2 | vulkan (+19.95%) | 1.8280 | 1.5240 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+84.71%) | 78.7920 | 42.6560 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+36.36%) | 61.0280 | 44.7560 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+61.93%) | 67.4000 | 41.6240 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+154.08%) | 95.1680 | 37.4560 |
| `serial_latency/q6-x8-selected-down/operation=q8_1_quantize,rows=8,experts=256,in=512,out=2048,wg=32;hip-wave=32` | 2 | vulkan (+3.56%) | 1.8600 | 1.7960 |
| `serial_latency/q6-x8-selected-down/operation=x8_selected_dp4a_dot_prequantized,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+143.99%) | 60.9000 | 24.9600 |
| `serial_latency/q6-x8-selected-down/operation=x8_selected_dp4a_quantize_plus_dot,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+119.97%) | 62.2440 | 28.2960 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=768,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+11.07%) | 12.6000 | 11.3440 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=2048,out=2048,rows=1,wg=32;hip-wave=32` | 2 | vulkan (+0.44%) | 1.8080 | 1.8000 |
| `serial_latency/dense-q8/operation=q8_1_quantize,in=2048,out=2048,rows=4,wg=32;hip-wave=32` | 2 | vulkan (+4.79%) | 2.0120 | 1.9200 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+4.00%) | 32.4840 | 31.2360 |
| `independent_throughput/dispatch-grid/sweep=count,count=941,grid=1;hip-wave=32` | 2 | vulkan (+193.14%) | 0.1926 | 0.0657 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=1;hip-wave=32` | 2 | vulkan (+64.72%) | 0.1765 | 0.1071 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=128;hip-wave=32` | 2 | vulkan (+543.97%) | 0.5291 | 0.0822 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=1024;hip-wave=32` | 2 | vulkan (+320.68%) | 1.0368 | 0.2465 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=8192;hip-wave=32` | 2 | vulkan (+82.63%) | 2.8329 | 1.5511 |
| `independent_throughput/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+13.27%) | 12.0560 | 10.6440 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / hip | 152 | 12 | 0 | 92.68 | 0.5462 | 164 |
| RL / vulkan | 136 | 28 | 0 | 82.93 | 0.6581 | 164 |

## Pinned hipEngine-harness comparison

The `hipengine` profile covers the same benchmark row set as pinned HipEngine `f2c3ad6`: family, operation, shape/sweep axes, repetition count, and serial/independent mode. Those rows deliberately run through Hipfire's existing Radiowave-tuned launch policy; wave size, workgroup geometry, source variant, ABI, and machine code remain optimization variables rather than parity constraints. HipEngine has 212 three-way matched core rows because its HIP independent sampler path rejects 12 rows; its 16 dispatch controls are reported separately. Hipfire executes and correctness-gates all 240 rows across four backends.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine core | RL / vulkan | 197 | 27 | 87.95 | 0.4798 | 224 |
| hipEngine core | RL / hip | 160 | 64 | 71.43 | 0.8555 | 224 |
| hipEngine dispatch | RL / vulkan | 0 | 0 | 0.00 | 0.0000 | 0 |
| hipEngine dispatch | RL / hip | 0 | 0 | 0.00 | 0.0000 | 0 |

## Harness verdict

This tuning smoke intentionally measures only `redline` versus `hip` versus `vulkan`. It uses the `hipengine_f2c` matrix, `radiowave_tuned` wave policy, and `default` scheduler profile. Every ranked row passed both CPU oracles. Redline beats Vulkan in 136/164 rows; final promotion still requires a full four-backend certification run.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. The `hipengine_f2c` profile matches HipEngine's row coverage while intentionally retaining Hipfire/Radiowave's tuned wave, workgroup, and source-variant choices. This artifact records the `radiowave_tuned` wave policy and `default` scheduler profile as controlled HIP compilation/launch factors. Every completion timestamp necessarily proves the measured work finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
