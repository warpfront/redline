# Hipfire/Redline ROCm issue 6409 backend comparison

Correctness-gated result: **Redline wins 192/240 rows (80.00%)**.

## Provenance (capture context)

This is the **primary** retained Hipfire gfx1201 ROCm 7.14 four-backend matrix
(`2026-07-22-rocm7.14-retest`). Numbers below are correctness-gated from
`summary.json` / `results.json` and are not rewritten.

Recorded environment flags in `results.json` (left as captured):

- `repository_dirty=true`, `hipfire_clone_dirty=true`
- `hipcc` version string **empty** in this capture
- `redline_queue_policy=auto`, `redline_independent_queues=2`
- Device: HIP ordinal 0 / gfx1201; Vulkan `AMD Radeon RX 9070 XT (RADV GFX1201)`

Treat the headline as a correctness-gated measurement with that dirty-tree /
empty-hipcc context — not as a laundered clean CI stamp.

## Placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 192 | 30 | 0 | 18 | 80.00 | 240 |
| vulkan | 42 | 147 | 14 | 37 | 17.50 | 240 |
| hipgraph | 3 | 42 | 93 | 102 | 1.25 | 240 |
| hip | 3 | 21 | 133 | 83 | 1.25 | 240 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 89 | 15 | 0 | 16 | 120 |
| serial_latency | vulkan | 26 | 58 | 6 | 30 | 120 |
| serial_latency | hipgraph | 3 | 40 | 60 | 17 | 120 |
| serial_latency | hip | 2 | 7 | 54 | 57 | 120 |
| independent_throughput | redline | 103 | 15 | 0 | 2 | 120 |
| independent_throughput | vulkan | 16 | 89 | 8 | 7 | 120 |
| independent_throughput | hipgraph | 0 | 2 | 33 | 85 | 120 |
| independent_throughput | hip | 1 | 14 | 79 | 26 | 120 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+4.26%) | 18.6000 | 17.8400 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+2.99%) | 18.3440 | 17.8120 |
| `serial_latency/memory-waitcnt/variant=strided4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+0.92%) | 17.9680 | 17.8040 |
| `serial_latency/memory-waitcnt/variant=strided4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+0.07%) | 17.8040 | 17.7920 |
| `serial_latency/memory-waitcnt/variant=gather1,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+16.24%), hipgraph (+9.38%), hip (+8.44%) | 42.6920 | 36.7280 |
| `serial_latency/memory-waitcnt/variant=gather1,n=32768,body=64,wg=256;hip-wave=32` | 4 | vulkan (+22.42%), hipgraph (+15.24%), hip (+14.54%) | 44.8640 | 36.6480 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=64;hip-wave=64` | 2 | vulkan (+0.84%) | 18.2360 | 18.0840 |
| `serial_latency/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+3.91%), hipgraph (+3.28%), hip (+3.28%) | 443.3320 | 426.6600 |
| `serial_latency/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 4 | vulkan (+3.70%), hipgraph (+1.79%), hip (+1.77%) | 440.9280 | 425.1920 |
| `serial_latency/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+3.41%), hipgraph (+2.70%), hip (+2.67%) | 443.7200 | 429.1040 |
| `serial_latency/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 4 | vulkan (+0.98%), hipgraph (+0.67%), hip (+0.59%) | 434.7520 | 430.5160 |
| `serial_latency/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+3.28%), hipgraph (+2.73%), hip (+2.69%) | 443.5440 | 429.4520 |
| `serial_latency/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 4 | vulkan (+1.39%), hipgraph (+0.94%), hip (+0.90%) | 432.6920 | 426.7680 |
| `serial_latency/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | hipgraph (+3.16%), hip (+3.09%), vulkan (+2.68%) | 442.1760 | 428.6475 |
| `serial_latency/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 4 | hip (+0.96%), vulkan (+0.44%), hipgraph (+0.27%) | 433.1080 | 428.9876 |
| `serial_latency/vopd/variant=independent_fma,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 4 | hipgraph (+23.77%), hip (+19.36%), vulkan (+10.49%) | 65.3120 | 52.7684 |
| `serial_latency/vopd/variant=dependent_fma,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 4 | hip (+18.85%), hipgraph (+16.58%), vulkan (+11.81%) | 72.9160 | 61.3525 |
| `serial_latency/vopd/variant=dependent_fma,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 2 | vulkan (+0.56%) | 60.4480 | 60.1120 |
| `serial_latency/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 4 | vulkan (+38.71%), hip (+27.79%), hipgraph (+26.52%) | 159.1360 | 114.7240 |
| `serial_latency/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 4 | vulkan (+27.98%), hip (+9.88%), hipgraph (+9.59%) | 145.5080 | 113.6920 |
| `serial_latency/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 4 | hipgraph (+26.14%), hip (+25.16%), vulkan (+21.19%) | 120.7560 | 95.7328 |
| `serial_latency/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 4 | vulkan (+11.44%), hipgraph (+7.31%), hip (+6.98%) | 111.4680 | 100.0280 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=64` | 2 | vulkan (+41.71%) | 59.8680 | 42.2480 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+108.64%) | 92.2600 | 44.2200 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=64` | 2 | vulkan (+43.42%) | 64.2280 | 44.7840 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+131.20%) | 110.7280 | 47.8920 |
| `serial_latency/q6-x8-selected-down/operation=x8_selected_dp4a_dot_prequantized,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=64` | 2 | vulkan (+64.79%) | 18.3640 | 11.1440 |
| `serial_latency/q6-x8-selected-down/operation=x8_selected_dp4a_quantize_plus_dot,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+57.64%) | 21.7480 | 13.7960 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+14.28%) | 17.8960 | 15.6600 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+22.70%) | 22.7920 | 18.5760 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+11.83%) | 23.7800 | 21.2640 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=128;hip-wave=32` | 2 | vulkan (+19.89%) | 0.1401 | 0.1169 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=1024;hip-wave=32` | 2 | vulkan (+137.72%) | 0.7601 | 0.3197 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=8192;hip-wave=32` | 2 | vulkan (+58.55%) | 3.0445 | 1.9202 |
| `independent_throughput/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+2.70%) | 10.9680 | 10.6800 |
| `independent_throughput/memory-waitcnt/variant=strided4,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+0.89%) | 10.8760 | 10.7800 |
| `independent_throughput/memory-waitcnt/variant=gather1,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+39.87%) | 38.9720 | 27.8640 |
| `independent_throughput/memory-waitcnt/variant=gather1,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+42.50%) | 39.2280 | 27.5280 |
| `independent_throughput/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 4 | vulkan (+25.89%), hip (+19.42%), hipgraph (+13.36%) | 138.7440 | 110.2080 |
| `independent_throughput/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 2 | vulkan (+24.96%) | 137.9120 | 110.3640 |
| `independent_throughput/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 4 | hip (+15.50%), hipgraph (+9.66%), vulkan (+7.24%) | 104.7880 | 90.7247 |
| `independent_throughput/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 2 | vulkan (+7.14%) | 104.5920 | 97.6240 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=64` | 2 | vulkan (+17.45%) | 47.3480 | 40.3120 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+99.41%) | 83.9120 | 42.0800 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=64` | 2 | vulkan (+29.31%) | 56.3760 | 43.5960 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+114.79%) | 99.0080 | 46.0960 |
| `independent_throughput/q6-x8-selected-down/operation=x8_selected_dp4a_dot_prequantized,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=64` | 2 | vulkan (+74.91%) | 15.5040 | 8.8640 |
| `independent_throughput/q6-x8-selected-down/operation=x8_selected_dp4a_quantize_plus_dot,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+57.47%) | 19.3120 | 12.2640 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 192 | 48 | 0 | 80.00 | 0.6981 | 240 |
| RL / hipgraph | 222 | 18 | 0 | 92.50 | 0.2531 | 240 |
| RL / hip | 222 | 18 | 0 | 92.50 | 0.2734 | 240 |

## Pinned hipEngine-harness comparison

**Quarantined / not current evidence.** Earlier drafts of this report pasted a
historical HipEngine three-backend table (212 core + 16 dispatch rows from an
older harness pin). That table is **not** this Hipfire run and must not be
quoted as ROCm 7.14 HipEngine product evidence.

Current HipEngine ROCm 7.14 core results live only under:

- [`../../../../hipengine-6409/results/gfx1201/2026-07-22-714-bench/REPORT.md`](../../../../hipengine-6409/results/gfx1201/2026-07-22-714-bench/REPORT.md) — RL > Vulkan **197/224**
- gfx1151 / gfx1100 sibling `2026-07-22-714-bench` reports

Hipfire remains the four-backend 240-row control in **this** directory
(192/240 firsts). Do not merge the two harness scorecards.

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 222/240 rows and HipGraph in 222/240 while all three select identical per-row hipcc code objects. This run uses the `hipengine_f2c` matrix, `radiowave_tuned` wave policy, and `default` scheduler profile. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 192/240 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. The `hipengine_f2c` profile matches HipEngine's row coverage while intentionally retaining Hipfire/Radiowave's tuned wave, workgroup, and source-variant choices. This artifact records the `radiowave_tuned` wave policy and `default` scheduler profile as controlled HIP compilation/launch factors. Every completion timestamp necessarily proves the measured work finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
