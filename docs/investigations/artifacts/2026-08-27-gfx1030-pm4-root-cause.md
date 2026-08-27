# GFX1030 PM4 zero-dispatch root-cause analysis (read-only)

## Q1: What encoder does gfx1030 actually get, and is it byte-identical to gfx1100?

**gfx1030 -> FloorFamily::Gfx10, gfx1100 -> FloorFamily::Gfx11, both -> FloorPm4::Legacy(Gfx10Pm4CommandBuffer) -> byte-identical encoder.**

- `FloorFamily::of` at `dispatch_floor.rs:120-134` branches on `name.starts_with("gfx10")` vs `gfx11`. gfx1030 matches Gfx10, gfx1100 matches Gfx11.
- `FloorPm4` at `dispatch_floor.rs:138-151` maps both `Gfx10|Gfx11 => Self::Legacy(Gfx10Pm4CommandBuffer::new_stateful())`. The type is literally `pub type Gfx11Pm4CommandBuffer = Gfx10Pm4CommandBuffer` at `pm4_gfx10.rs:108`.
- `FloorPm4::create_ib` at `dispatch_floor.rs:184-195` then calls `create_gfx10` for Gfx10 and `create_gfx11` for Gfx11. In `replay.rs:100-153` those two methods are identical: both `ensure_device_family(device, Pm4EncoderMap::Legacy)` then `create_encoded(bytes...)`. The guard at `replay.rs:398-416` accepts any `gfx10*` or `gfx11*` for Legacy.
- `Pm4EncoderMap` at `replay.rs:379-383` is `Legacy` for both, `Gfx12` separate.
- SingleQueue and MultiQueue pairs (`replay.rs:492-527`) are likewise identical.
- Conclusion: for identical `dispatch` call sequence, the PM4 dword stream is byte-identical. Dynamic fields that differ per HSACO (code_entry, Rsrc1/2/3, user SGPRs) come from the kernel image, not the encoder.

## Q2: Which registers does Legacy program, and is it sufficient for RDNA2?

Per `pm4_gfx10.rs:271-299` one `dispatch_image()` emits:

- `COMPUTE_PGM_LO 0x20c` 2 dwords (code_entry>>8, >>40) covers LO/HI (`pm4_gfx10.rs:30,271`)
- `COMPUTE_PGM_RSRC1 0x212` 2 dwords (rsrc1, patched rsrc2 with LDS blocks) (`pm4_gfx10.rs:31,278`)
- `COMPUTE_PGM_RSRC3_GFX10 0x228` 1 dword (`pm4_gfx10.rs:34,279`)
- `COMPUTE_TMPRING_SIZE_GFX10 0x218` 1 dword 0 (`pm4_gfx10.rs:33,280`)
- `COMPUTE_NUM_THREAD_X 0x207` 3 dwords workgroup xyz (`pm4_gfx10.rs:29,281`)
- `COMPUTE_RESOURCE_LIMITS 0x215` 1 dword 0 (`pm4_gfx10.rs:32,289`)
- `COMPUTE_USER_DATA_0 0x240` N dwords if non-empty (`pm4_gfx10.rs:35,290`)
- `PACKET3_DISPATCH_DIRECT 0x15` 4 dwords: workgroups xyz + initiator `1<<0|1<<2|1<<3|W32` (`pm4_gfx10.rs:21,294-299`)

Fence packets: `PACKET3_ACQUIRE_MEM 0x58` (`pm4_gfx10.rs:25,126`), `PACKET3_EVENT_WRITE 0x46` CS_PARTIAL_FLUSH `0x407` (`pm4_gfx10.rs:24,305`), `COPY_DATA 0x40`/`RELEASE_MEM 0x49` for timestamps (`pm4_gfx10.rs:22-23,190`).

`dispatch_floor.rs:361-368` verify builds N dispatches with optional `wait_compute_idle()` between, no leading/trailing ACQUIRE.

Comparison to Gfx12 at `pm4.rs:27-34,404-425`: same PGM_LO/RSRC1, but `RSRC3 0x223` not 0x228, `TMPRING 0x216` not 0x218, `RESOURCE_LIMITS 0x3ff` not 0, plus `STATIC_THREAD_MGMT_SE0 0x230` 4x MAX absent on legacy, and initiator `1<<0|1<<2|1<<5|W32` (USE_THREAD_DIMENSIONS with workitems) vs legacy `1<<3` ORDER_MODE with workgroup counts (`pm4.rs:431-434` vs `pm4_gfx10.rs:296`).

Source claims sufficiency: `pm4_gfx10.rs:27-28` "matching ROCr's legacy compute builder and Hipfire's independently exercised gfx1010 implementation." No per-gfx1030 exception exists. Risks: TMPRING/RSRC3 offset if Navi21 differs from Navi10, RESOURCE_LIMITS 0 vs 0x3ff, initiator bit 3 vs 5 (gate uses 1/1 where both encodings coincidentally equal).

## Q3: Was GFX10 path ever validated on gfx1030?

No. Compilation yes, PM4 gate no.

- `pm4_gfx10.rs:28` cites Hipfire gfx1010, not gfx1030.
- `pm4_gfx10_smoke.rs:16-19,86-93` requires gfx10/gfx11 smoke but no CI run or artifact for gfx1030.
- `README.md:39` AQL replay exercised on gfx1010, gfx1030, gfx1100, gfx1151, gfx1201 is AQL, not PM4. `README.md:40` retained PM4 is family-specific separate row.
- `2026-08-27-pm4-under-rocm10.md:91-95` and `DISPATCH-FLOOR.md` show dispatch_floor was gfx1201-only before FloorFamily fix; master now supports families but table shows unmeasured for gfx1100/1151 and no gfx1030 PM4 numbers.
- Unit tests at `replay.rs:3385-3388, pm4_gfx10.rs:489-553, pm4.rs:599-857` are host-side dword checks; `.github/workflows/ci.yml:65-68` says tests are host-side, no /dev/kfd.
- Artifact glob `docs/investigations/artifacts/*` contains gfx1100 probes, no gfx1030 PM4 PASS JSON.

Stated plainly: gfx1030 PM4 support was never real; inferred from gfx1010.

## Q4: Ranked candidates (cheapest discriminating observation each)

1. **Cache coherency / GL2 writeback (executed but invisible):** dispatch executed, atomic stays in L1/L2, completion fires before WB. Both serialized and minimal fail because both lack trailing WB. gfx1100 Navi3x auto-WB or different MES completion hides bug. `wait_compute_idle` at `pm4_gfx10.rs:305` is flush only, no `GL2_WB`; `acquire_system` GCR `0x1FFFF` at `pm4_gfx10.rs:135` includes GL2 but gate never emits it, while smoke does `wait+acquire` at `pm4_gfx10_smoke.rs:65-68`. **Kill/confirm:** after loop insert `cmd.wait_compute_idle(); cmd.acquire_system();` before `create_ib` on gfx1030 and rerun; if 512/512, confirmed.

2. **RSRC3/TMPRING offset mismatch (0x228/0x218 wrong for Navi21):** CP drops DISPATCH_DIRECT if SH offset wrong, silent zero. Hipfire validated gfx1010 offset. **Kill/confirm:** dump PM4 hex dry-run (`REDLINE_PM4_DRY_RUN` path `pm4_gfx10_smoke.rs:73`) on both devices, compare 0x228 write vs ROCm header `gfx1030` mmCOMPUTE.

3. **Initiator / dimension encoding (0x800d ORDER_MODE with workgroup counts vs required 0x8025 USE_THREAD_DIMENSIONS):** could launch wrong grid but gate uses 1/1 where they coincide. **Kill/confirm:** emit alternate stream with bit5+workitems and compare; or trace AQL initiator for same kernel on gfx1030 vs PM4 initiator.

4. **Vendor AQL INDIRECT_BUFFER packet / barrier header:** `packet.rs:421-455` uses `1<<8 BARRIER` and `PACKET3_INDIRECT_BUFFER 0x3f | IB_VALID 1<<23`. If RDNA2 MES requires different barrier/type, CP consumes header but discards PM4, signals completion early. **Kill/confirm:** chain a second PM4 IB with COPY_DATA sentinel; if sentinel appears while counter zero, IB consumed.

5. **Kernel descriptor / code_entry:** loader at `runtime.rs:2718` extracts code_entry, `pm4_gfx10.rs:255` checks align. Wrong entry would fault to 0, but then KFD fault callback at `runtime.rs:51` would fire, not silent zero. **Kill/confirm:** print `kernel.pm4_metadata()` and PGM_LO dwords on gfx1030.

6. **Wave32 vs wave64:** `pm4_gfx10.rs:93,294` wave32 bit15 follows kernel property. Mismatch would mis-set SGPR layout. **Kill/confirm:** print `image.wave32` and initiator.

## Q5: Did-not-execute vs executed-but-invisible

Executed does NOT imply visible at gate read.

- Counter is `KernargPool::allocate_executable_bytes(4)` at `dispatch_floor.rs:355`, pool discovered as `FINE_GRAINED|KERNARG_INIT` at `runtime.rs:1898-1941`. Fine-grained is HSA-coherent but GPU L1/L2 still needs flush; AQL inserts system fences that PM4 omits.
- Gate stream has no trailing flush; smoke does append `wait_compute_idle(); acquire_system();` before completion (`pm4_gfx10_smoke.rs:65-68` comment "Flush completed shader writes before ROCr publishes its completion signal"). Verify omits this (`dispatch_floor.rs:369`).
- `Gfx10 acquire_system` at `pm4_gfx10.rs:135` includes `GLK|GLV|GL1|GL2|WB|SEQ`, none emitted at tail. `wait_compute_idle` alone is `0x407` flush without WB.
- Vendor packet at `packet.rs:450` is waited via `wait_signal` at `replay.rs:357-360` which waits for CP to decrement signal after fetching IB, not for wave retirement, unless IB contains explicit flush.
- Hence `0/512` is consistent with executed-but-invisible on gfx1030 where atomics stay in GL2, while gfx1100 may auto-WB or order completion after GL2, passing by accident. Gate comment at `dispatch_floor.rs:339-343` explicitly says must equal N iff executed AND completion waited for wave retirement, showing author knew this gap.
- Distinction changes fix: register-map bug needs offset fix; visibility bug needs trailing `wait_compute_idle+acquire_system` (or RELEASE_MEM) mirroring smoke.

## Most likely and minimal experiment

**Most likely: rank 1, missing trailing compute-idle + cache flush (executed-but-invisible).** Byte-identical encoder (Q1), validated on another RDNA2, both fence modes fail equally, smoke's trailing flush pattern, and Navi21 vs Navi31 L2 policy delta all point here.

**Parent hardware experiment (no refactor, parent owns execution):**

1. Control: on gfx1030 same ROCm10, run AQL (non-PM4) gate for ctr_k 512 times; expect PASS (proves HSACO/memory healthy).
2. PM4 flush test: patch only `verify_pm4_execution` after loop insert `cmd.wait_compute_idle(); cmd.acquire_system();` (or `dependency_rmw_global()`) before `create_ib`; rebuild; `REDLINE_FLOOR_VERIFY_HSACO=bench/floor_kernel_ctr.hsaco REDLINE_FLOOR_HSACO=... REDLINE_FLOOR_N=512 cargo run --example dispatch_floor -p redline-dispatch` on gfx1030. If `512/512` then bug is GL2 WB; if still `0/512` then proceed to dump PM4 hex and compare 0x228/0x218 vs header.
3. Control stay: same patched binary on gfx1100 should remain PASS.

Cost: 2 replays gfx1030, 1 gfx1100, <10 lines. Fix if confirmed: add trailing flush+idle to all retained Gfx10 IBs as in smoke.

No large refactor proposed.
