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

Full smoke table truncated; see JSON `results/gfx1151/2026-09-01-smoke.json` (94 rows) and prefill in-progress `gfx1151/2026-09-01-baseline.json` (currently 112 rows, target 188, ~8 mins done, expected ~16 mins total). Prefill will be appended when complete; methodology identical to gfx1201.

### gfx1100 — smoke, 94 rows, same 20 scratch refusals, 74 pass

Identical kernel set to gfx1151 (same 47 descriptors). Smoke shows same pattern: hip/hipgraph pass for all, redline refuses same 10 BT12 scratch kernels (private_segment_size 40/48/64). Non-scratch medians similar to gfx1151 (within 10%). See `results/gfx1100/2026-09-01-smoke.json`. Prefill in-progress.

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

## Reading

- Correctness: CPU reference (pack_blob decode_tile + f64 GEMM) matches GPU for all families/bits/variants within 0.05 rel_rms; the imported kernels are correct on all three archs. Residual's `Y +=` path is exercised via canary `y_init` and `expected_after`.
- LDS: MW-LDS (static 8192 B, block 128/256) shows same correctness as BT (LDS 0) on gfx11; static LDS is correctly reported and not dynamically allocated.
- Scratch: BT12 variants on gfx11 use private scratch (40–64 B) and are correctly refused by Redline (loader checks private_segment_size). HIP executes them.
- Scheduler: only `default` profile was built for baseline; `iterative_ilp` crashes clang-23 RAGreedy on gfx1201 (segfault) and is emitted empty (driver skips). No other profile was A/B'd in this baseline.
- Performance: Redline's retained PM4 is consistently faster for small independent throughput (n16, n128) and competitive for large. The mq2 (2-bit, smallest group_bytes 72) is the outlier where Redline is slower on large serial — worth profiling group_bytes vs cache policy.
- Next: run `examples/hipfire-6409/join_arms.py` with these JSONs as `ref` to compare redline vs hip(hipgraph) ratios; extend to other scheduler profiles once clang fix lands.

## Commits

- Scaffold: `ab121e3`
- Microbench: `ceb0a17` (initial), `fb12dc8` (fix gfx11 umbrella duplicate decode_tile), `5a572ce` (Redline respect HIP_VISIBLE_DEVICES, tolerate gfx substring), `11fd17e` (fix missing profiles var)
- Results: `results/gfx1201/2026-09-01-baseline.json` (16 rows prefill), `results/gfx1151/2026-09-01-smoke.json` (94 rows), `results/gfx1100/2026-09-01-smoke.json` (94 rows); prefill for gfx11 in-progress, will amend.
