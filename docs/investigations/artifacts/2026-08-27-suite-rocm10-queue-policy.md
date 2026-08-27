<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# The internal suite was flattering redline by 2.6x on gfx1201

**Status: internal. NOT posted upstream.**

`examples/hipfire-6409` compares redline, Vulkan, hipGraph and plain HIP. Its HIP
backend hardcoded `lanes = 4.min(logical_iterations)` for
`TimingMode::IndependentThroughput` (`src/hip_backend.rs:411-415`), with no
`GPU_MAX_HW_QUEUES` anywhere in the suite. Four lanes is pessimal on gfx1201
under ROCm 10.0, where the measured optimum is 2 and the cliff past it is steep.

So the HIP and hipGraph arms were running past the part's queue-width cliff while
redline was not, and every independent-throughput ratio the suite reported was
inflated.

## Measured, gfx1201, ROCm 10.0, `independent_throughput/dispatch-grid`

us per operation, `correctness=true mismatches=0` on every backend in every run.

| count | policy | hip | hipgraph | redline | vulkan |
| ----: | --- | ---: | ---: | ---: | ---: |
| 50 | legacy (4) | 9.3288 | 6.6952 | 0.1528 | 0.7508 |
| 50 | auto (2) | 4.4992 | 4.4876 | 0.1520 | 0.7464 |
| 50 | explicit 2 | 4.4788 | 4.3604 | 0.1512 | 0.7516 |
| 200 | legacy (4) | 7.7433 | 7.8939 | 0.0784 | 0.2642 |
| 200 | auto (2) | 3.0080 | 3.5045 | 0.0780 | 0.2620 |
| 200 | explicit 2 | 3.2798 | 3.4978 | 0.0782 | 0.2638 |
| 941 | legacy (4) | 7.4739 | 8.1304 | 0.0591 | 0.1086 |
| 941 | auto (2) | 2.8727 | 3.2682 | 0.1173 | 0.1090 |
| 941 | explicit 2 | 2.7713 | 3.2914 | 0.0589 | 0.1082 |

Fixing the width makes HIP **2.6x faster** at count=941 (7.4739 -> 2.8727) and
hipGraph **2.5x faster** (8.1304 -> 3.2682). `auto` and explicit `2` agree
throughout, which confirms the per-device resolution picks 2 on gfx1201.

Effect on the headline, count=941: redline versus HIP falls from 126x to **48.6x**,
and versus hipGraph from 138x to **55.3x**. Versus Vulkan it is **1.84x**, which
the queue policy does not change since the Vulkan backend has its own queue setup.

Serial-latency rows are unaffected by design: that mode uses one lane, so a chain
is the honest shape there. Confirmed by running the same A/B over
`serial_latency/dispatch-grid`, where legacy and auto agree within noise.

One honest wart: at count=941 the `auto` run reported redline at 0.1173 against
0.0591 and 0.0589 in the other two runs — a 2x outlier for redline in a single
run, not a policy effect. It is left in rather than dropped, and it means
single-run suite numbers at this scale should not be quoted to two decimals.

## Provenance now travels with results

The suite could not previously say which ROCm it ran against, which is how an
invalid 7.14-vs-10.0 A/B slipped through earlier this session. It now prints and
records, via `src/rocm_provenance.rs`:

```
ROCm provenance hip_runtime_version_raw=71526333 \
  libamdhip64="/opt/rocm/core-10.0/lib/libamdhip64.so.7.15.26333-0000000" \
  libhsa="/opt/rocm/core-10.0/lib/libhsa-runtime64.so.1.21.0" mixed=false
```

`mixed=false` is the load-bearing field: it asserts no two ROCm trees were mapped
into the process at once, which is the exact failure that hung a GPU earlier.

## Defaults and reproducibility

`HipQueuePolicy::Legacy` remains the default, so existing `results/` numbers stay
reproducible. The tuned path is opt-in via `--hip-queues auto|1..16` or
`HIPFIRE_HIP_QUEUES`. Changing the default would silently invalidate every stored
comparison.

## Operational notes

- `HIPFIRE_BENCH_ARCH` is consumed at **build** time (`build.rs:121`) and defaults
  to `gfx1201`. Running a gfx1201-built binary on gfx1151 fails with
  `hipModuleLoadData: device kernel image is invalid`; rebuild per arch.
- The suite needs `glslc` (shaderc) at build time. shaderc 2026.1 works; 2023.8
  lacks `GL_EXT_integer_dot_product` and fails on `kernels/hipfire_6409.comp`.
- Do not set both `ROCR_VISIBLE_DEVICES` and `HIP_VISIBLE_DEVICES`; together they
  compose to zero visible devices and report "no ROCm-capable device is detected".
- On gfx1151 the redline backend still fails to load its own code object
  (`hsa_executable_load_agent_code_object` -> `HSA_STATUS_ERROR_INCOMPATIBLE_ARGUMENTS`)
  after an arch-matched rebuild. Unresolved; the gfx1201 run above is unaffected.
