# Hipfire/Redline ROCm issue 6409 backend comparison

Correctness-gated result: **Redline wins 194/240 rows (80.83%)**.

## Role: secondary leverage A/B (not primary certification)

This tree is **dirty-tree / non-regression leverage evidence**, not the clean
primary Hipfire gfx1201 headline. Prefer
[`../2026-07-22-rocm7.14-retest/REPORT.md`](../2026-07-22-rocm7.14-retest/REPORT.md)
(**192/240**) for product claims.

### Arms retained here

| Artifact | `partition_policy` | `roctx` (as recorded) | Redline firsts (that arm) |
| --- | --- | --- | ---: |
| `baseline-default.json` | `none` | `null` (unavailable) | 193/240 |
| `partitioned-equal2.json` | `equal:2` | `sdk` | 194/240 |
| `summary.json` (aggregate used below) | — | — | **194/240 (80.83%)** |

### Provenance (do not rewrite JSON)

Both arms recorded `repository_dirty=true` and `hipfire_clone_dirty=true`.
`hipcc` is present on the leverage arms (ROCm 7.14.60850 / TheRock core-7.14).
amd-smi / broader observe coverage was **not** uniformly available across arms;
baseline `roctx` is explicitly `null`. Queue policy remained `auto` / 2
independent Redline queues.

Use this run only to show that the partitioned leverage configuration did not
regress the four-backend placement story under those capture conditions.

## Placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 194 | 28 | 0 | 18 | 80.83 | 240 |
| vulkan | 38 | 149 | 15 | 38 | 15.83 | 240 |
| hipgraph | 5 | 40 | 96 | 99 | 2.08 | 240 |
| hip | 3 | 23 | 129 | 85 | 1.25 | 240 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 90 | 14 | 0 | 16 | 120 |
| serial_latency | vulkan | 23 | 61 | 6 | 30 | 120 |
| serial_latency | hipgraph | 5 | 38 | 64 | 13 | 120 |
| serial_latency | hip | 2 | 7 | 50 | 61 | 120 |
| independent_throughput | redline | 104 | 14 | 0 | 2 | 120 |
| independent_throughput | vulkan | 15 | 88 | 9 | 8 | 120 |
| independent_throughput | hipgraph | 0 | 2 | 32 | 86 | 120 |
| independent_throughput | hip | 1 | 16 | 79 | 24 | 120 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+4.39%) | 18.6600 | 17.8760 |
| `serial_latency/memory-waitcnt/variant=strided4,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+2.46%) | 18.3120 | 17.8720 |
| `serial_latency/memory-waitcnt/variant=gather1,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+15.79%), hipgraph (+9.07%), hip (+8.07%) | 42.5520 | 36.7480 |
| `serial_latency/memory-waitcnt/variant=gather1,n=32768,body=64,wg=256;hip-wave=32` | 4 | vulkan (+22.68%), hipgraph (+15.16%), hip (+14.23%) | 44.8480 | 36.5560 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=64,wg=64;hip-wave=64` | 2 | vulkan (+1.00%) | 18.2640 | 18.0840 |
| `serial_latency/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+2.55%), hip (+1.16%), hipgraph (+1.14%) | 436.5560 | 425.7160 |
| `serial_latency/packed-dot/variant=q8_signed,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 4 | vulkan (+2.06%), hipgraph (+1.62%), hip (+1.57%) | 433.9520 | 425.1920 |
| `serial_latency/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | hipgraph (+3.52%), hip (+3.50%), vulkan (+2.94%) | 442.0640 | 427.0305 |
| `serial_latency/packed-dot/variant=q4_unsigned,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 4 | hip (+1.52%), vulkan (+0.80%), hipgraph (+0.51%) | 433.7480 | 427.2345 |
| `serial_latency/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | vulkan (+2.85%), hipgraph (+2.40%), hip (+2.34%) | 441.6440 | 429.4240 |
| `serial_latency/packed-dot/variant=q6_zero,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 4 | vulkan (+1.63%), hipgraph (+1.28%), hip (+1.22%) | 432.4000 | 425.4520 |
| `serial_latency/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=64;hip-wave=32` | 4 | hipgraph (+3.42%), hip (+3.38%), vulkan (+2.53%) | 441.7680 | 427.1385 |
| `serial_latency/packed-dot/variant=scalar_dequant,groups=16,n=32768,body=64,wg=256;hip-wave=32` | 4 | hip (+1.24%), vulkan (+0.39%), hipgraph (+0.22%) | 432.6720 | 427.3626 |
| `serial_latency/vopd/variant=independent_fma,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 4 | hipgraph (+22.65%), hip (+21.48%), vulkan (+10.95%) | 64.2560 | 52.3883 |
| `serial_latency/vopd/variant=independent_fma,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 2 | vulkan (+2.62%) | 58.9920 | 57.4880 |
| `serial_latency/vopd/variant=dependent_fma,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 4 | hipgraph (+19.36%), hip (+18.90%), vulkan (+8.77%) | 62.5520 | 52.4083 |
| `serial_latency/vopd/variant=dependent_fma,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 2 | vulkan (+2.06%) | 57.7640 | 56.6000 |
| `serial_latency/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 4 | vulkan (+39.58%), hipgraph (+27.97%), hip (+27.61%) | 155.4360 | 111.3560 |
| `serial_latency/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 4 | vulkan (+28.31%), hipgraph (+10.06%), hip (+9.88%) | 143.1120 | 111.5400 |
| `serial_latency/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 4 | hipgraph (+26.66%), hip (+26.29%), vulkan (+21.52%) | 120.2440 | 94.9325 |
| `serial_latency/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 4 | vulkan (+11.76%), hipgraph (+6.97%), hip (+6.76%) | 110.7880 | 99.1320 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=64` | 2 | vulkan (+41.83%) | 59.7440 | 42.1240 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+104.74%) | 92.9040 | 45.3760 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=64` | 2 | vulkan (+44.21%) | 64.0080 | 44.3840 |
| `serial_latency/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+126.61%) | 108.8520 | 48.0360 |
| `serial_latency/q6-x8-selected-down/operation=x8_selected_dp4a_dot_prequantized,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=64` | 2 | vulkan (+108.13%) | 23.1440 | 11.1200 |
| `serial_latency/q6-x8-selected-down/operation=x8_selected_dp4a_quantize_plus_dot,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+63.33%) | 22.3960 | 13.7120 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=4,row_tile=1,wg=32;hip-wave=32` | 2 | vulkan (+14.47%) | 17.9080 | 15.6440 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_dot_prequantized,in=2048,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+22.58%) | 22.8480 | 18.6400 |
| `serial_latency/dense-q8/operation=q8_0_dense_dp4a_quantize_plus_dot,in=2048,out=2048,rows=4,row_tile=4,wg=32;hip-wave=32` | 2 | vulkan (+11.69%) | 23.7680 | 21.2800 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=1024;hip-wave=32` | 2 | vulkan (+105.70%) | 0.6536 | 0.3177 |
| `independent_throughput/dispatch-grid/sweep=grid,count=941,grid=8192;hip-wave=32` | 2 | vulkan (+33.42%) | 2.5546 | 1.9147 |
| `independent_throughput/memory-waitcnt/variant=gather1,n=32768,body=64,wg=64;hip-wave=32` | 2 | vulkan (+41.02%) | 39.0960 | 27.7240 |
| `independent_throughput/memory-waitcnt/variant=gather1,n=32768,body=64,wg=256;hip-wave=32` | 2 | vulkan (+41.63%) | 39.1240 | 27.6240 |
| `independent_throughput/vopd/variant=independent_fma,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 2 | vulkan (+0.41%) | 58.1200 | 57.8840 |
| `independent_throughput/vopd/variant=dependent_fma,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 2 | vulkan (+0.18%) | 56.6440 | 56.5400 |
| `independent_throughput/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 4 | vulkan (+25.69%), hip (+18.11%), hipgraph (+13.02%) | 138.3920 | 110.1040 |
| `independent_throughput/vopd/variant=mixed_int_float,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 2 | vulkan (+24.87%) | 137.6320 | 110.2200 |
| `independent_throughput/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=64;hip-wave=64` | 4 | hip (+13.94%), hipgraph (+8.78%), vulkan (+7.21%) | 104.3520 | 91.5845 |
| `independent_throughput/vopd/variant=dequant_like,accums=4,n=65536,body=512,wg=256;hip-wave=64` | 2 | vulkan (+7.77%) | 104.3800 | 96.8520 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=64` | 2 | vulkan (+27.64%) | 48.0800 | 37.6680 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=64;hip-wave=32` | 2 | vulkan (+110.00%) | 87.8240 | 41.8200 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_dot_prequantized,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=64` | 2 | vulkan (+34.15%) | 58.1600 | 43.3560 |
| `independent_throughput/q4-selected-dual/operation=selected_dual_dp4a_quantize_plus_dot,x_rows=4,rows=32,experts=256,in=2048,out=512,wg=128;hip-wave=32` | 2 | vulkan (+133.70%) | 108.0160 | 46.2200 |
| `independent_throughput/q6-x8-selected-down/operation=x8_selected_dp4a_dot_prequantized,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=64` | 2 | vulkan (+46.70%) | 12.9680 | 8.8400 |
| `independent_throughput/q6-x8-selected-down/operation=x8_selected_dp4a_quantize_plus_dot,rows=8,experts=256,in=512,out=2048,wg=64;hip-wave=32` | 2 | vulkan (+34.80%) | 16.6880 | 12.3800 |

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 194 | 46 | 0 | 80.83 | 0.7111 | 240 |
| RL / hipgraph | 222 | 18 | 0 | 92.50 | 0.2505 | 240 |
| RL / hip | 222 | 18 | 0 | 92.50 | 0.2755 | 240 |

## Pinned hipEngine-harness comparison

**Quarantined / not current evidence.** The historical pasted HipEngine 192/212
table is not this Hipfire leverage run and is not the retained ROCm 7.14
HipEngine scorecard. See
[`../../../../hipengine-6409/results/gfx1201/2026-07-22-714-bench/REPORT.md`](../../../../hipengine-6409/results/gfx1201/2026-07-22-714-bench/REPORT.md)
for current HipEngine numbers.

## Harness verdict

Secondary leverage context only — not the primary clean retest. This is **not a Hipfire harness failure**: Redline beats direct HIP in 222/240 rows and HipGraph in 222/240 while all three select identical per-row hipcc code objects. This run uses the `hipengine_f2c` matrix, `radiowave_tuned` wave policy, and `default` scheduler profile. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 194/240 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. The `hipengine_f2c` profile matches HipEngine's row coverage while intentionally retaining Hipfire/Radiowave's tuned wave, workgroup, and source-variant choices. This artifact records the `radiowave_tuned` wave policy and `default` scheduler profile as controlled HIP compilation/launch factors. Every completion timestamp necessarily proves the measured work finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
