# mqv2 WMMA microbench baseline — 2026-09-01

## Provenance

- ROCm: 10.0, hipcc 7.15.26333-0000000 at /opt/rocm/core-10.0/bin/hipcc, `LD_LIBRARY_PATH=/opt/rocm/core-10.0/lib`
- hipRuntimeGetVersion raw 71526333 (approx 71500000) on gfx1201; same tree for gfx1151/gfx1100 via hipx
- Device mapping: gfx1201 (Navi48 RX9070, local), gfx1151 (device 1 on hipx), gfx1100 (device 0 on hipx)
- HSACO sha256 (default profile):
  - gfx1201: `e4b82895465ca821c049becc7fdb4d36c97d1e8a269efacd6578574ec13fc36f` (641K, iterative_ilp placeholder empty due to clang RAGreedy segfault)
  - gfx1151: `641K` (after fix for duplicate decode_tile umbrella; iterative_ilp empty)
  - gfx1100: `~600K` (same umbrella fix)
- Code objects: `mqv2_<arch>_<profile>.hsaco` embedded via `include_bytes!`; empty slice skipped by driver. Only `default` profile exercised (`--scheduler-profile default`); `iterative_ilp` for gfx1201 crashes clang-23 (emit same as hipx).
- Build: `HIPCC=/opt/rocm/core-10.0/bin/hipcc HIPFIRE_BENCH_ARCH=<arch> cargo build --release` in `examples/hipfire-mqv2`

## Config

- Default matrix: `spec.rs` shapes `smoke` (n16 k512, proj_m 64 each) x 4 families, `prefill` (n128 k2048 and n512 k4096 with family-specific total_m as per contract)
- Row matrix per arch: `kernels::descriptors(arch)` x shapes x modes {serial, independent} x profile {default}
  - gfx1201: 4 kernels (qkv BT8 bits 2,3,5,6) -> smoke 8 rows, prefill 16 rows
  - gfx1151/gfx1100: 47 kernels (31 BT +16 MW) -> smoke 94 rows, prefill 188 rows (expected)
- Timing: `--warmups 2 --samples 5 --iterations 4` (defaults); display GPU kept short with `--iterations 2 --samples 2` for initial smoke
- Backends: `hip` (single stream or 4-way fan-out with join), `hipgraph` (captured graph replay), `redline` (retained PM4 IB, one per row, dependency boundaries for serial, multi-queue for independent via QueuePolicy::Auto)

## Per-arch tables (median us, GFLOP/s, gate, bit-identity)

GFLOP/s = 2·N·K·total_m / us / 1000.

### gfx1201 — prefill, 16 rows, all pass, all bit-identical (3/3 backends)

| key | hip us | hipgraph us | redline us | hip GF | hg GF | rl GF | gate hip/hg/rl | identical |
|---|---|---|---|---|---|---|---|---|
| serial qkv mq2 n128 k2048 m2048+512+512 | 78.5 | 74.8 | 56.1 | 20525 | 21544 | 28725 | pass/pass/pass | yes |
| indep qkv mq2 n128 k2048 | 69.3 | 73.1 | 59.2 | 23244 | 22024 | 27220 | pass | yes |
| serial mq2 n512 k4096 m4096+1024+1024 | 781.9 | 791.5 | 909.2 | 16430 | 16230 | 14131 | pass | yes |
| indep mq2 n512 | 829.2 | 830.0 | 954.6 | 15494 | 15477 | 13459 | pass | yes |
| serial mq3 n128 | 86.8 | 87.6 | 95.3 | 18559 | 18396 | 16903 | pass | yes |
| indep mq3 n128 | 70.0 | 87.7 | 41.3 | 23014 | 18368 | 38990 | pass | yes |
| serial mq3 n512 | 1292.8 | 1278.3 | 849.3 | 9937 | 10050 | 15126 | pass | yes |
| indep mq3 n512 | 980.3 | 1072.5 | 994.6 | 13105 | 11979 | 12918 | pass | yes |
| serial mq5 n128 | 86.5 | 86.4 | 82.4 | 18626 | 18645 | 19550 | pass | yes |
| indep mq5 n128 | 69.0 | 86.3 | 45.5 | 23343 | 18666 | 35420 | pass | yes |
| serial mq5 n512 | 1218.7 |1259.4 | 788.5 | 10543 | 10201 | 16294 | pass | yes |
| indep mq5 n512 | 956.4 |1021.0 | 977.2 | 13434 | 12585 | 13148 | pass | yes |
| serial mq6 n128 | 86.6 |86.8 |85.0 |18607 |18560 |18951 | pass | yes |
| indep mq6 n128 | 68.5 |85.7 |41.8 |23522 |18802 |38520 | pass | yes |
| serial mq6 n512 |1190.3|1220.6|740.4|10794|10526|17353|pass|yes|
| indep mq6 n512 |931.4|992.2|950.0|13792|12949|13527|pass|yes|

*All 16 rows: rel_rms 0.0002, max_abs 2.4–4.4, bit_identical_across_backends true.*

Redline vs hip median ratio (rl/hip):
- serial n128: 0.71 (mq2), 1.10 (mq3), 0.95 (mq5), 0.98 (mq6) — near parity, mq2 faster on redline
- independent n128: 0.85 (mq2), 0.59 (mq3), 0.66 (mq5), 0.61 (mq6) — redline 1.2–1.7× faster
- serial n512: 1.16 (mq2), 0.66 (mq3), 0.65 (mq5), 0.62 (mq6) — redline faster for 3/4, slower for mq2
- independent n512: 1.15 (mq2), 1.01 (mq3), 1.02 (mq5), 1.02 (mq6) — parity

### gfx1151 — smoke, 94 rows, 74 pass (all hip/hipgraph), 20 redline scratch-refusals

Rows failing redline only (scratch):

```
serial_latency/gate_up/gemm_gate_up_mq2g256v2_wmma_gfx11_bt12_mq2_bt12/n16_k512_m64+64 → redline error: kernel gemm_gate_up_mq2g256v2_wmma_gfx11_bt12 uses scratch (private_segment_size=40), Redline refuses scratch kernels
... (same for mq3 BT12 private 48, mq5/6 BT12 private 64)
... 10 kernels ×2 modes =20 rows
  - gate_up BT12: mq2, mq3, mq5, mq6 (4)
  - qkvza BT12: mq3, mq5, mq6 (3)
  - qkv BT12: mq3, mq5, mq6 (3)
Total 10 kernels ×2 =20
```

Remaining 74 rows: all hip/hipgraph/redline pass, bit-identical where redline not scratch (hip/hipgraph always bit-identical; redline matches hip for non-scratch). Example medians (smoke n16 k512):

- qkvza mq2 bt4 serial: hip 28.1 us (GF ~  2.3), redline 5.2 us (0.19×) — redline much faster for small
- gate_up mw4 lds mq3 serial: hip 42 us, redline 38 us (~0.9×)
- residual bt8 mq6 serial: hip 12 us, redline 7 us (0.58×)

Full smoke table truncated; see JSON `results/gfx1151/2026-09-01-baseline.json` (94 rows) and prefill in-progress `gfx1151/2026-09-01-baseline.json` (currently 112 rows, target 188, ~8 mins done, expected ~16 mins total). Prefill will be appended when complete; methodology identical to gfx1201.

### gfx1100 — smoke, 94 rows, same 20 scratch refusals, 74 pass

Identical kernel set to gfx1151 (same 47 descriptors). Smoke shows same pattern: hip/hipgraph pass for all, redline refuses same 10 BT12 scratch kernels (private_segment_size 40/48/64). Non-scratch medians similar to gfx1151 (within 10%). See `results/gfx1100/2026-09-01-baseline.json`. Prefill in-progress.

## Redline vs HIP

- Same HSACO for hip/hipgraph/redline per (arch, profile) — bit-identical outputs for all non-scratch rows (output_sha256 matches across backends). Scratch rows are hip-only (expected; redline closes the lane).
- Small smoke (n16 k512): redline often 0.5–0.2× hip (faster), due to retained PM4 vs HIP stream launch overhead.
- Prefill n128 (gfx1201): redline ~0.6–1.1×, best for independent throughput (multi-queue) on mq3/5/6.
- Large n512: parity for independent, redline wins for serial on mq3/5/6 (0.6–0.7×) but loses on mq2 (1.15×). Suggests mq2's group_bytes 72 (smallest) may stress LDS or RMW differently.

## Gate failures verbatim

All non-scratch rows pass rel_rms ≤0.05 (observed 0.0002). Failures only for redline scratch refusal (20 rows per gfx11 smoke/prefill):

```
kernel gemm_gate_up_mq2g256v2_wmma_gfx11_bt12 uses scratch (private_segment_size=40), Redline refuses scratch kernels
kernel gemm_gate_up_mq3g256v2_wmma_gfx11_bt12 uses scratch (private_segment_size=48), Redline refuses scratch kernels
kernel gemm_gate_up_mq5g256v2_wmma_gfx11_bt12 uses scratch (private_segment_size=64), Redline refuses scratch kernels
kernel gemm_gate_up_mq6g256v2_wmma_gfx11_bt12 uses scratch (private_segment_size=64), Redline refuses scratch kernels
kernel gemm_qkvza_mq3g256v2_wmma_gfx11_bt12 uses scratch (private_segment_size=48), Redline refuses scratch kernels
kernel gemm_qkvza_mq5g256v2_wmma_gfx11_bt12 uses scratch (private_segment_size=64), Redline refuses scratch kernels
kernel gemm_qkvza_mq6g256v2_wmma_gfx11_bt12 uses scratch (private_segment_size=64), Redline refuses scratch kernels
kernel gemm_qkv_mq3g256v2_wmma_gfx11_bt12 uses scratch (private_segment_size=48), Redline refuses scratch kernels
kernel gemm_qkv_mq5g256v2_wmma_gfx11_bt12 uses scratch (private_segment_size=64), Redline refuses scratch kernels
kernel gemm_qkv_mq6g256v2_wmma_gfx11_bt12 uses scratch (private_segment_size=64), Redline refuses scratch kernels
```

Each appears twice (serial + independent). No other failures; hip/hipgraph never fail.

## Retained-tape release gap

The 54 residual FAILs in `results/gfx1151/2026-09-01-prefill-norelease.json` (188 rows, 94 pass, 94 fail) are not a kernel bug: hip and hipgraph pass the same rows at 0.0002. The failure is an L2 coherence hole between the HIP H2D copy that resets Y and the retained PM4 replay that last wrote Y.

*Mechanism:* `src/hip_backend.rs:168-178` `reset_buffers` does `hip.memcpy_htod(&buf, y_init)` (HostToDevice via SDMA) then `hip.device_synchronize()`. `src/redline_backend.rs:339-351` does that, then `ownership_ib.replay_and_wait()` (`acquire_system` at `crates/redline-rocr/src/pm4_gfx10.rs:126` / `pm4.rs:177`) then the timed `ib.replay_and_wait_profiled()`. The profiled tape ends with no system-scope release: it contains only the kernel dispatches and, for serial, `dependency_rmw` boundaries between them (`pm4_gfx10.rs:150-162` for gfx10/11, `pm4.rs:268-280` for gfx12). The last dispatch's Y store stays dirty in GL2. The next round's HIP SDMA H2D copy of y_init lands in memory, but the prior tape's dirty Y+acc is still in GL2; the next round's `acquire_system` does `GL2_WB | GL2_INV` and writes the dirty y+acc back over y_init. The kernel then reads y_init+acc left over from the previous round and adds acc again.

*Three discriminators for residual/gemm_mq5g256v2_residual_wmma_gfx11_bt4, gfx1151, prefill, serial, iterations 1 (reported verbatim, redline rel_rms/max_abs, hip always pass 0.0002):*

```
(1) HSA_ENABLE_SDMA=0 (forces ROCr blit-kernel copies through L2):
  serial n128 k2048 m2048 redline PASS 0.0002/2.474 (was FAIL 1.0/9449.485)
  serial n512 k4096 m4096 PASS 0.0002/3.664

(2) Y_RESET=d2d (hipMemcpyDtoD from a staging device buffer instead of H2D):
  serial n128 PASS 0.0002/2.474 (was FAIL)
  serial n512 PASS 0.0002/3.664

(3) REDLINE_APPEND_RELEASE=1 (append wait_compute_idle + acquire_system at end of profiled IB before SingleQueuePm4Ib::create):
  serial n128 PASS 0.0002/2.474 (was FAIL)
  serial n512 PASS 0.0002/3.664
```

With iterations 4, serial n128 was FAIL 0.9979/37797.938 and is PASS 0.0002/9.894 with any of the three; n512 serial was already PASS. Independent n128 and n512 indep show the same pattern in the full 188-row run (54 residual FAILs total: for each of the 18 residual kernels, 3 of 4 rows fail — n128 serial, n128 indep, n512 indep fail, n512 serial passes — because the 1 MiB n128 case fits in Strix Halo's ~2 MiB L2 and the SDMA copy stays behind dirty GL2, while the 4 KiB n16 smoke case is copied by the blit kernel through L2 and the 8 MiB n512 case is evicted).

*Shape pattern:* n16 Y 4 KiB (64*16*4) is copied by a blit kernel through L2 (coherent) and passes; n128 Y 1 MiB (2048*128*4) is copied by SDMA behind dirty GL2 that still holds the previous round's Y+acc and fits in the 2 MiB L2, so the acquire writes it back; n512 Y 8 MiB is evicted from L2 so serial passes and only the 4-queue independent case (4 distinct Y, each 8 MiB, still hot) flakes.

*Why hipfire-6409 never saw it:* Its stores are Radiowave-certified VMEM with `CachePolicy::Temporal` and its readbacks go through HIP's own system-scope release, not a retained PM4 tape that ends without a release. The 6409 kernels never do a HIP H2D SDMA copy into a buffer the retained replay last wrote.

*Hipfire exposure:* Any HIP H2D SDMA copy into a buffer a retained PM4 replay last wrote, on any arch, whenever the dirty lines still sit in GL2. The fix belongs where the tape is built (the bench, exactly as redline-dispatch's own `graph_pm4.rs` ends every lane with `wait_compute_idle + acquire_system`), not in the crates.

*Fix:* `src/redline_backend.rs` now ends every profiled tape and every independent lane with `wait_compute_idle(); acquire_system();` (gfx10/11: `pm4_gfx10.rs:126`; gfx12: `pm4.rs:177`) before `SingleQueuePm4Ib::create_*` / `MultiQueuePm4Ib::create_*`, default on, opt-out with `REDLINE_APPEND_RELEASE=0`. `Y_RESET` remains diagnostic. No redline crates were touched.

## Reading

- Correctness: CPU reference (pack_blob decode_tile + f64 GEMM) matches GPU for all families/bits/variants within 0.05 rel_rms; the imported kernels are correct on all three archs. Residual's `Y +=` path is exercised via canary `y_init` and `expected_after`.
- LDS: MW-LDS (static 8192 B, block 128/256) shows same correctness as BT (LDS 0) on gfx11; static LDS is correctly reported and not dynamically allocated.
- Scratch: BT12 variants on gfx11 use private scratch (40–64 B) and are correctly refused by Redline (loader checks private_segment_size). HIP executes them.
- Scheduler: only `default` profile was built for baseline; `iterative_ilp` crashes clang-23 RAGreedy on gfx1201 (segfault) and is emitted empty (driver skips). No other profile was A/B'd in this baseline.
- Performance: Redline's retained PM4 is consistently faster for small independent throughput (n16, n128) and competitive for large. The mq2 (2-bit, smallest group_bytes 72) is the outlier where Redline is slower on large serial — worth profiling group_bytes vs cache policy.
- Next: run `examples/hipfire-6409/join_arms.py` with these JSONs as `ref` to compare redline vs hip(hipgraph) ratios; extend to other scheduler profiles once clang fix lands.

## Gfx11 prefill tables (summarize.py, all families)

`python3 examples/hipfire-mqv2/summarize.py results/gfx1151/2026-09-01-prefill.json` (188 rows, now all pass after the trailing release; previously 94/188 failed) and `results/gfx1100` are identical in shape. The failing `...-norelease.json` (94/188 failed) is kept as evidence.

Gate_up variant ranking (n512 k4096 m12288+12288, serial, gfx1151, from the norelease table's hip medians):
- mq3: mw4_lds 4943 us beats bt12 6138 beats bt6 8803 beats mw8 5363 (mw4 best, then mw8, then bt12, then bt6)
- n128 k2048 m6144+6144 serial, same family: bt12 ~294 us best, mw4 ~337, mw8 ~310, bt6 ~417 — so bt12 is best for the small shape, mw4 for the large shape.

BT12 256-VGPR / scratch finding (Radiowave inspection, `crates/radiowave` manifest): BT12 kernels report 256 VGPRs and `private_segment_size` 40 (mq2), 48 (mq3), 64 (mq5/6), while BT4 reports 78–104 VGPRs and MW4 ~92 VGPRs, all with `private_segment_size` 0. Hence BT12 uses scratch and is correctly refused by Redline (pm4_gfx10.rs:219 checks `private_segment_size !=0`), while BT4 and MW pass.

## Gfx11 prefill, with the trailing release (final)

`results/gfx1151/2026-09-01-prefill.json` and `results/gfx1100/2026-09-01-prefill.json`
(188 rows each, `trailing_release: true`, ROCm 10.0, HIP ordinal 1 / 0 on hipx):
hip 188/188, hipgraph 188/188, redline 148/148 non-scratch rows pass and are
bit-identical to hip on both architectures; rel-RMS max 0.00025. The 40
redline refusals per arch are the BT12 scratch kernels (20 kernels x 2 modes).
The failing runs without the release are kept as
`results/<arch>/2026-09-01-prefill-norelease.json`.

Best variant per family/shape, serial, by hip median (summarize.py):

| family | shape | gfx1151 | gfx1100 |
| --- | --- | --- | --- |
| gate_up | n128 k2048 m6144+6144 | mq3/bt12 292 us | mq6/mw8_lds 203 us |
| gate_up | n512 k4096 m12288+12288 | mq3/mw4_lds 4944 us | mq4/mw8_lds 1979 us |
| qkv | n128 k2048 m2048+512+512 | mq6/bt12 91 us | mq3/bt12 133 us |
| qkv | n512 k4096 m4096+1024+1024 | mq5/bt12 933 us | mq3/bt4 668 us |
| qkvza | n128 | mq2/bt4 97 us | mq2/bt4 148 us |
| qkvza | n512 | mq5/bt12 1172 us | mq3/bt12 817 us |
| residual | n128 k2048 m2048 | mq3/mw8_lds 68 us | mq4/mw8_lds 77 us |
| residual | n512 k4096 m4096 | mq4/mw4_lds 848 us | mq4/mw8_lds 417 us |

gate_up at n512 (serial hip medians): gfx1151 mq3 mw4_lds 4944 < mw8 5363 <
bt12 6138 < bt6 8803; gfx1100 mq3 mw8_lds 2001 < mw4 2154 = bt12 2157 < bt6
2769. On both parts the MW-LDS kernels beat the spilling BT12 at the large
prefill shape, and the ordering of MW4 vs MW8 flips between the APU and the
dGPU. hipfire's `mqv2_prefill_batch_tile` selects gate_up BT12 on gfx1151 for
N >= 96 and reserves MW for gfx1100 bits 5/6; the bench says MW4 is 20% faster
than BT12 on gfx1151 at n512 and MW8 is 7% faster than BT12 on gfx1100 at
n512, with BT12 spilling to scratch on both.

Redline / hip median ratio over the 148 non-scratch rows (both modes):
gfx1151 median 0.99 (p25 0.95, p75 1.05); gfx1100 median 0.84 (p25 0.79,
p75 0.97, min 0.50). On these 50 us - 13 ms GEMMs the retained-PM4 dispatch
advantage is small on the APU and 16% at the median on the dGPU, consistent
with the claims-discipline expectation that the advantage compresses with
kernel weight; it is not the 2-4x of the tiny-kernel suite.

## Commits

- Scaffold: `ab121e3`
- Microbench: `ceb0a17` (initial), `fb12dc8` (fix gfx11 umbrella duplicate decode_tile), `5a572ce` (Redline respect HIP_VISIBLE_DEVICES, tolerate gfx substring), `11fd17e` (fix missing profiles var)
- Probes: `ac329f1` (debug_grid/residual_like), `0184691` (ratio histogram), `e836c13` (trailing release default, Y_RESET diagnostic)
- Results: `results/gfx1201/2026-09-01-baseline.json` (16 rows prefill), `results/gfx1151/2026-09-01-prefill.json` (188 rows, now all pass) + `...-norelease.json` (188 rows, 94 failed as evidence), `results/gfx1100/2026-09-01-prefill.json` (188 rows) + `...-norelease.json`; `results/gfx1151/2026-09-01-baseline.json` and `.../gfx1100/...-baseline.json` remain the 94-row smoke baselines for compatibility.
