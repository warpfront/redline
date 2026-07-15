# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 4/4 rows (100.00%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 4 | 0 | 0 | 0 | 100.00 | 4 |
| vulkan | 0 | 4 | 0 | 0 | 0.00 | 4 |
| hipgraph | 0 | 0 | 3 | 1 | 0.00 | 4 |
| hip | 0 | 0 | 1 | 3 | 0.00 | 4 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 2 | 0 | 0 | 0 | 2 |
| serial_latency | vulkan | 0 | 2 | 0 | 0 | 2 |
| serial_latency | hipgraph | 0 | 0 | 1 | 1 | 2 |
| serial_latency | hip | 0 | 0 | 1 | 1 | 2 |
| independent_throughput | redline | 2 | 0 | 0 | 0 | 2 |
| independent_throughput | vulkan | 0 | 2 | 0 | 0 | 2 |
| independent_throughput | hipgraph | 0 | 0 | 2 | 0 | 2 |
| independent_throughput | hip | 0 | 0 | 0 | 2 | 2 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|

## Pairwise Redline results

| Comparison | Wins | Losses | Ties | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|---:|
| RL / vulkan | 4 | 0 | 0 | 100.00 | 0.9164 | 4 |
| RL / hipgraph | 4 | 0 | 0 | 100.00 | 0.8408 | 4 |
| RL / hip | 4 | 0 | 0 | 100.00 | 0.8327 | 4 |

## Pinned hipEngine-harness comparison

The `hipengine` profile covers the same benchmark row set as pinned HipEngine `f2c3ad6`: family, operation, shape/sweep axes, repetition count, and serial/independent mode. Those rows deliberately run through Hipfire's existing Radiowave-tuned launch policy; wave size, workgroup geometry, source variant, ABI, and machine code remain optimization variables rather than parity constraints. HipEngine has 212 three-way matched core rows because its HIP independent sampler path rejects 12 rows; its 16 dispatch controls are reported separately. Hipfire executes and correctness-gates all 240 rows across four backends.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine core | RL / vulkan | 192 | 20 | 90.57 | 0.4589 | 212 |
| hipEngine core | RL / hip | 158 | 54 | 74.53 | 0.8343 | 212 |
| hipEngine dispatch | RL / vulkan | 6 | 10 | 37.50 | 1.4331 | 16 |
| hipEngine dispatch | RL / hip | 16 | 0 | 100.00 | 0.2249 | 16 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 4/4 rows and HipGraph in 4/4 while all three select identical per-row hipcc code objects. This run uses the `hipengine_f2c` matrix, `radiowave_tuned` wave policy, and `default` scheduler profile. Remaining Vulkan-only wins therefore isolate compiler/lowering, kernel scheduling, or unavoidable completion/timestamp costs rather than a missing Redline dispatch path. Redline beats Vulkan pairwise in 4/4 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical selected hipcc code object for each row; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. The `hipengine_f2c` profile matches HipEngine's row coverage while intentionally retaining Hipfire/Radiowave's tuned wave, workgroup, and source-variant choices. This artifact records the `radiowave_tuned` wave policy and `default` scheduler profile as controlled HIP compilation/launch factors. Every completion timestamp necessarily proves the measured work finished. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
