<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# Hipfire-native ROCm issue 6409 benchmark

Standalone Rust harness for the ROCm [#6409](https://github.com/ROCm/ROCm/issues/6409)
microbenchmark matrix. It compares **direct HIP**, **HipGraph**, **Redline**
(retained GFX10/11/12 PM4 via public ROCr), and **Vulkan/RADV** on the same
row set without importing hipEngine into the timed path.

HIP / HipGraph / Redline load the identical per-row HSACO selected by
Radiowave. Vulkan runs matched GLSL. Every ranked row must pass all selected
backend CPU oracles.

## Matrix

Default `--matrix hipengine`: **120 configurations × 2 modes = 240 rows**
(dispatch/grid, geometry, reduction, memory/waitcnt, packed dot, VOPD, sampler,
two-stage reduction, Q4 selected-dual, Q6 x8, dense Q8).

Legacy 45-shape / 133-row experiment: `--matrix legacy --include-aggressive`.

## Reproduce (supported commands)

```bash
export PATH=/opt/rocm/core/bin:/opt/rocm/core/lib/llvm/bin:$PATH
export ROCM_PATH=/opt/rocm/core HIP_PATH=/opt/rocm/core

cd examples/hipfire-6409
cargo build --release --bin hipfire-6409-bench

# Primary-style four-backend run (gfx1201 default; override with HIPFIRE_BENCH_ARCH)
./target/release/hipfire-6409-bench \
  --matrix hipengine \
  --wave-policy radiowave \
  --scheduler-profile default \
  --redline-rmw radiowave-vmem \
  --redline-queues auto \
  --warmups 3 --samples 7 \
  --out results/gfx1201/manual/results.json
```

Useful flags already supported by the binary:

| Flag | Meaning |
| --- | --- |
| `--backends all` / `redline,vulkan,...` | Backend subset |
| `--redline-queues auto\|1\|2\|4` | Independent IB queue policy (auto: Q4 gfx11/1151, Q2 gfx12, Q1 else) |
| `--redline-rmw radiowave-vmem\|same-agent` | RMW cache boundary |
| `--wave-policy radiowave` | Radiowave recipe catalog selection |
| `--scheduler-profile default` | Shared HIP/HipGraph/Redline object profile |
| `HIPFIRE_BENCH_ARCH=gfx1100` | Target ISA / recipe plan (use a per-arch Cargo `--target-dir`) |

Cross-arch build pattern:

```bash
for target in 0:gfx1100 1:gfx1151 2:gfx1030 3:gfx1010; do
  ordinal=${target%%:*}; arch=${target#*:}
  HIPFIRE_BENCH_ARCH=$arch \
    cargo build --release --target-dir target/$arch --bin hipfire-6409-bench
  ROCR_VISIBLE_DEVICES=$ordinal HIPFIRE_BENCH_ARCH=$arch \
    target/$arch/release/hipfire-6409-bench \
      --matrix hipengine --wave-policy radiowave \
      --scheduler-profile default --redline-rmw radiowave-vmem \
      --warmups 3 --samples 7 \
      --out results/$arch/manual/results.json
done
```

Optional bytecode-identical HipEngine artifact checks (not the default matrix):

```bash
cargo run --release --bin hipengine_exact -- \
  --preheat 1000 --warmups 1000 --samples 21 \
  --out results/gfx1201/hipengine-exact-vopd.json
```

## Retained result index

Only these trees are product-facing. Prefer their `REPORT.md` / `summary.json`
over any other directory still present on disk.

### ROCm 7.14 (current)

| Path | Role | Headline |
| --- | --- | --- |
| [`results/gfx1201/2026-07-22-rocm7.14-retest/`](results/gfx1201/2026-07-22-rocm7.14-retest/REPORT.md) | **Primary** four-backend | **192/240** Redline firsts (80.0%); RL > Vulkan 192/240 |
| [`results/gfx1201/2026-07-22-rocm714-leverage-certification/`](results/gfx1201/2026-07-22-rocm714-leverage-certification/REPORT.md) | Secondary leverage A/B | 194/240 (80.83%) — **dirty-tree non-regression, not clean cert** |

### Cross-RDNA native PM4

| Path | Headline |
| --- | --- |
| [`results/2026-07-14-rdna-rocr-native/`](results/2026-07-14-rdna-rocr-native/REPORT.md) | **537/960** firsts; RL > Vulkan **606/960**; per-arch runs under `results/gfx{1010,1030,1100,1151}/2026-07-14-rdna-rocr-native/` |

### Multiqueue controls

| Arch | Paths |
| --- | --- |
| gfx1100 | [`…/2026-07-14-redline-current-q1-independent/`](results/gfx1100/2026-07-14-redline-current-q1-independent/REPORT.md) (67/120 indep), [`…/2026-07-14-redline-multiqueue-q4/`](results/gfx1100/2026-07-14-redline-multiqueue-q4/REPORT.md) (189/240) |
| gfx1151 | [`…/q1-independent`](results/gfx1151/2026-07-14-redline-current-q1-independent/REPORT.md) (68/120), [`…/q4`](results/gfx1151/2026-07-14-redline-multiqueue-q4/REPORT.md) (206/240) |
| gfx1201 | [`q1-indep`](results/gfx1201/2026-07-14-redline-current-q1-independent/REPORT.md) (88/120), [`q2-indep`](results/gfx1201/2026-07-14-redline-multiqueue-q2-independent/REPORT.md) (99/120), [`q2 full`](results/gfx1201/2026-07-14-redline-multiqueue-q2/REPORT.md) (187/240), [`q4 negative`](results/gfx1201/2026-07-14-redline-multiqueue-q4/REPORT.md) |

## Interpretation guardrails

1. **Primary vs secondary.** Quote **192/240** from `2026-07-22-rocm7.14-retest`
   as the current Hipfire gfx1201 headline. The leverage tree's 194/240 is an
   A/B non-regression under dirty tree — do not present it as the clean primary.
2. **Provenance is in the JSON.** Retained records may set `repository_dirty` /
   `hipfire_clone_dirty`, leave `hipcc` empty, or store absolute capture paths.
   Docs label that; artifacts are not rewritten.
3. **Identical HSACO for HIP stack columns.** Vulkan-only wins can still be
   ACO/LLVM lowering, not proof that retained PM4 is missing.
4. **Pairwise vs four-way firsts.** First-place counts need all ranked backends
   present; two-backend multiqueue runs are RL-vs-Vulkan controls.
5. **Do not link archive candidates.** Older `2026-07-12*` / `2026-07-13*` tuning
   diaries and probe trees are not product evidence even if still on disk.

Kernels: [`kernels/hipfire_6409.hip`](kernels/hipfire_6409.hip).
Radiowave policy: [`../../crates/radiowave`](../../crates/radiowave/README.md).
