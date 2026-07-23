<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# Dispatch-floor #6409 demos

Small HIP / assembly kernels used while isolating the ROCm issue
[#6409](https://github.com/ROCm/ROCm/issues/6409) dispatch floor (noop, buffer,
direct-SRD, and hipGraph baseline sketches).

## Current product validation

These files are **methodology fixtures**, not the published scorecard.

Current retained product evidence is the **ROCm 7.14** set documented in the
[root README](../../README.md#current-results-rocm-714) and
[`docs/DISPATCH-FLOOR.md`](../../docs/DISPATCH-FLOOR.md):

- Hipfire gfx1201 primary **192/240** firsts —
  [`../hipfire-6409/results/gfx1201/2026-07-22-rocm7.14-retest/`](../hipfire-6409/results/gfx1201/2026-07-22-rocm7.14-retest/REPORT.md)
- HipEngine gfx1201/1151/1100 `2026-07-22-714-bench` summaries/reports under
  [`../hipengine-6409/results/`](../hipengine-6409/results/)
- Cross-RDNA + multiqueue controls under `../hipfire-6409/results/`
  (`2026-07-14-rdna-rocr-native`, `2026-07-14-redline-*`)

Historical ROCm 7.2 fence-spectrum and PM4 host-latency tables live only in
[`docs/DISPATCH-FLOOR.md`](../../docs/DISPATCH-FLOOR.md) and are labeled as
methodology, not current certification.

## Related harnesses

| Harness | Role |
| --- | --- |
| [`../hipfire-6409`](../hipfire-6409/README.md) | Standalone four-backend matrix (primary public Hipfire numbers) |
| [`../hipengine-6409`](../hipengine-6409/README.md) | Pinned HipEngine micro suite + Redline adapters |
| `cargo run --example dispatch_floor -p redline-dispatch` | Fence-policy microbench (`bench/floor_kernel.hip`) |

## Local sketches in this directory

| File | Notes |
| --- | --- |
| `hipgraph_baseline.hip` | Real `hipGraphLaunch` baseline sketch |
| `gmb_noop.hip` / `gmb_buffer.hip` | Tiny dispatch / buffer kernels |
| `gmb_direct_srd.s` / `gmb_aco_style.s` | Experimental assembly images (not used by retained product matrices) |
| `demo.py` | Small driver helper |

Prefer the Hipfire/HipEngine harness READMEs for exact reproduce commands that
match retained JSON.
