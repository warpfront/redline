<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# ROCm/ROCm#6529 — gfx1100 address-zero SQC-data VM fault: investigation state

Dated: 2026-07-31, updated 2026-08-01. Written for a fresh-context agent. Read
this first, then work the "Open work" queue in order. Do not re-run completed
audit steps.

## The fault

- Tracker: ROCm/ROCm#6529 (OPEN, `status: triage`, zero comments; verified live
  against the GitHub API 2026-08-01, last updated 2026-07-30T18:21:51Z), filed
  by lhl (shisa-ai/hipEngine). Root thread: ROCm/ROCm#6409#issuecomment-5100210675.
- Symptom (identical tuple on Radeon Pro W7900 and RX 7900 XTX, both gfx1100):
  gfxhub page fault at address 0x0, read, client = SQC (data) (0xa),
  PERMISSION_FAULTS 0x3, MAPPING_ERROR 0x0; ~2 s later MES REMOVE_QUEUE
  failures, repeated MODE1 full-device resets, VRAM loss.
- Trigger shape: intermittent, on entry to the first LONG serial-chain
  retained-PM4 IB row (W7900: row 49; RX: row 173). Not kernel-specific.
- Does NOT reproduce since: 4 clean full 240-row matrices per card on both
  ROCm 7.14 and 7.15 nightly, same binary (sha256 655141e2…), same boot.
- Fault-time host controls: `cwsr_enable=1`, `mcbp=-1` (auto), `amdgpu.runpm=0`.
- lhl's dossier (excellent, read it): branch `redline-integration-spike` of
  shisa-ai/hipEngine, `docs/REDLINE.md` + `benchmarks/results/2026-07-28-*`.
  His framing: "cross-device lifecycle failure / resource-generation lifetime
  hypothesis," ownership explicitly unresolved. Redline is default-off in
  hipEngine until this is resolved.
- **IT HAS FIRED ON OUR HARDWARE — once, unnoticed (found 2026-08-01).** Note
  the wording: we did **not** reproduce it and we have no recipe. Our gfx1100
  exhibited it during unrelated work nine days earlier and nobody looked at the
  kernel log until now. `hipx` GPU 0 — RX 7900 XTX, gfx1100, `0000:66:00.0` —
  logged **78 address-zero page faults on 2026-07-23 between 13:37:49 and
  16:44:44**, across **8 distinct PIDs**, escalating into **29 MES
  `REMOVE_QUEUE` failures and 24 `MODE1` full-device resets**. Every sampled
  fault carries lhl's exact tuple: address `0x0`, `SQC (data) (0xa)`,
  `GCVM_L2_PROTECTION_FAULT_STATUS:0x00801431`, `PERMISSION_FAULTS 0x3`,
  `MAPPING_ERROR 0x0`. Five days before he filed. An earlier pass in this
  document claimed our faults were APU-only with a different status word; that
  was wrong. Do not repeat "cannot reproduce" anywhere. Raw evidence:
  `docs/investigations/artifacts/2026-08-01-gfx1100-6529-reproduction.json`.
- **Since then: silent, exactly like lhl's cards.** He got it on a W7900 and an
  RX 7900 XTX, then 4 clean 240-row matrices per card. We got it on 2026-07-23,
  then nothing — 8/8 clean on the preemption probe and zero faults during the
  full RDNA hipgraph A/B on that same card on 2026-08-01. Whatever the trigger
  is, it is not a function of simply running retained-PM4 work on gfx1100.
- **Temporal correlation worth chasing, not yet causal:** a redline
  **multiqueue** build finished at `/tmp/rl714-multiqueue/…/libredline_dispatch.so`
  at **13:34**, three minutes before the first fault, and another landed at
  13:53 at the tail of the first cluster. Multiqueue work creates and destroys
  many queues, which is precisely the resource-lifecycle stress lhl's leading
  hypothesis implicates. This is a coincidence in time, not evidence of
  causation — the faulting process was `hip_dot_path`, not a multiqueue binary.
- **OPEN, and it decides the whole reply:** which backend faulted. The process
  was `hip_dot_path`, a hipEngine micro-benchmark whose redline leg and stock
  HIP leg build to the *same binary name*, so the process name does not
  identify it. Note the fault window **overlaps** the surviving result
  directories under `redline-bench-results/gfx1100/` (written 16:01–16:44,
  including one named `q1-recovery-smoke`) — so those runs were themselves
  faulting, and their `summary.json` lists backends `hip` and `vulkan` with
  **`redline` absent**. That absence is suggestive of a redline leg that died
  without writing results, but it is equally consistent with redline never
  having been scheduled in that run. Do not treat it as settled.
  - **redline leg** → the fault is ours to own, and the 13:34/13:53 multiqueue
    builds give a narrow window to bisect. Still not a recipe: one occurrence.
  - **stock HIP leg** → the fault fires with no redline in the process at all,
    which points at ROCm rather than retained-PM4 replay and would largely
    exonerate this project. This is the single highest-value unknown on #6529.

## Accepted validation (context for why this matters)

Same-HSACO W7900: 240/240 correctness rows; Redline beats HIP on 239 rows
(median 2.792x) and Vulkan on 208 rows (median 1.696x); +8.13% decode E2E
(GGUF graph diagnostic, bit-identical, 92.812 -> 100.357 tok/s). Independently:
pwilkin (llama.cpp maintainer "Ilintar") wired Redline into llama.cpp and
closed a ROCm-vs-Vulkan gap from 62 to 68.5-70 tok/s (fork: pwilkin/redline,
PR pending). AMD staff (Uncy_AMD) escalated the AQL-overhead question
internally. Adoption is blocked on this fault.

## Completed audit (do not redo)

### Construction side — ruled out

Static audit of every zero-to-shader path in redline (this repo):

- `crates/redline-rocr/src/pm4_gfx10.rs` — gfx10/gfx11 builder rejects
  code_entry == 0 (InvalidCodeEntry) and null kernarg (NullKernarg in
  `hsa_user_sgprs`) before any stream mutation.
- `crates/redline-rocr/src/pm4.rs` — gfx12 builder, same guards.
- `crates/redline-capi/src/gpu.rs` — rejects cross-agent module mixing
  (RL_ERR_HANDLE); finalized IB owns its modules + kernargs (lifetime sound).
- Conclusion: packet construction cannot emit a zero program address. The
  observed zero entered at runtime, below the packet layer.

### Kernel side — mechanism corroborated, opt-out ruled out (2026-08-01)

Source read of torvalds/linux amdgpu/amdkfd (master @ ~0131b508c0e2). Full
report: `local://dig-kernel.md` (session artifact; key citations reproduced
here because they are load-bearing).

Proven from source:

1. `struct v11_compute_mqd` (`include/v11_structs.h:674-755`) **does** carry
   `compute_pgm_lo/hi` (offsets 0xD/0xE) and `compute_user_data_0..15`
   (0x41..0x50) — and `init_mqd` (`kfd_mqd_manager_v11.c:114-181`) `memset`s
   the MQD to zero and **never writes those fields**. `update_mqd`
   (same file, 208-275) never writes them either. The MQD's SH shadow is
   permanently zero for a normal KFD queue.
2. Host `hqd_load_v11` (`amdgpu_amdkfd_gfx_v11.c:193-199`) MMIO-loads only
   `cp_mqd_base_addr_lo` (MQD offset 0x80) through `CP_HQD_PQ_WPTR_HI`. The
   SH shadows at 0xD/0xE/0x41 are outside that window.
3. The CWSR trap handler saves and restores **wave** state only — VGPR, SGPR,
   LDS, and hwregs (M0, PC, EXEC, STATUS, TRAPSTS, XNACK, MODE, FLAT_SCRATCH).
   No COMPUTE_PGM / USER_DATA / SET_SH restore exists in the trap sources.
4. CWSR is **device-global**: `cwsr_enable` (default 1, `amdgpu_drv.c:758-765`)
   AND `supports_cwsr` gate `kfd->cwsr_enabled` (`kfd_device.c:512-574`).
   `struct kfd_ioctl_create_queue_args` (`uapi/linux/kfd_ioctl.h:73-93`)
   carries CSA pointers and **no** flags field — there is no per-queue
   "no preempt", "no CWSR", or "no QSWITCH" bit. Passing a null CSA is not an
   opt-out: `init_mqd` still sets `QSWITCH_MODE`, just with zero bases.
5. `mcbp=-1` leaves `adev->gfx.mcbp` **false** on non-SR-IOV hosts —
   `amdgpu_device_set_mcbp` (`amdgpu_device.c:3632-3643`) only forces it on
   for `mcbp=1` or an SR-IOV VF. MCBP is a graphics-ring path
   (`IB_FLAG_PREEMPT`), not the KFD compute HQD/MES path. **The faulting host
   was CWSR-on, MCBP-off.**
6. MES programs a 10 ms process quantum and a 1 ms gang quantum on every
   `add_queue_mes` (`kfd_device_queue_manager.c:226-232`), so a long retained
   IB can be quantum-switched with no SVM/TTM/USERPTR eviction event at all.

Not provable from open source: whether CP/MES actually reloads SH registers
from the (zero) MQD on reconnect, versus clearing them. `PRELOAD_REQ` +
`PRELOAD_SIZE=0x55` + `QSWITCH_MODE` are programmed by the driver but their
microcode semantics are not public. **Either behaviour produces PGM = 0 at
the next elided DISPATCH_DIRECT**, which is the observed tuple. This is the
one question that needs an AMD firmware answer.

### Userspace side — no opt-out (2026-08-01)

Source read of ROCR-Runtime + libhsakmt. Full report: `local://dig-user.md`.

- Redline creates a normal AQL queue via public `hsa_queue_create`
  (`crates/redline-rocr/src/runtime.rs:991-1001`, `QUEUE_TYPE_MULTI`).
  ROCr's `AqlQueue` passes `HSA_QUEUE_COMPUTE_AQL` with fixed
  `HSA_QUEUE_PRIORITY_NORMAL` and 100% queue percentage
  (`amd_aql_queue.cpp:97, 269-273`).
- libhsakmt **unconditionally** sizes and allocates a context-save-restore
  area for gfx >= Carrizo with FCompute cores (`libhsakmt/src/queues.c:282-317`,
  alloc at 519-585) and passes it on `AMDKFD_IOC_CREATE_QUEUE`. If that
  allocation fails, queue creation fails — it does not fall back to a
  non-CWSR queue.
- No public HSA/AMD API, create flag, or ROCr env var disables mid-IB compute
  preemption. `hsa_amd_queue_set_priority` maps LOW/NORMAL/HIGH to KMT
  priorities that feed MES `inprocess_gang_priority` under a fixed NORMAL
  global level (`hsa_ext_amd.cpp:1173-1196`); it changes who wins a quantum,
  not whether a queue may be preempted.
- Why AQL is immune: every `hsa_kernel_dispatch_packet_t` carries
  `kernel_object`, `kernarg_address`, and all sizes, so CP materialises
  COMPUTE_PGM_* and USER_DATA per dispatch. ROCr never emits SET_SH_REG
  COMPUTE_PGM_LO/HI for user dispatches. Retained PM4 with SH elision is the
  only thing on that queue depending on SH persistence.
- ROCr's own `AqlQueue::ExecutePM4` wraps PM4 in a vendor AQL packet on the
  same CWSR queue — but those are short, fully self-describing control blasts,
  not long multi-dispatch chains that elide state.

**Answer to the old open-work item 4 ("can a raw queue opt out of
preemption?"): No. Not per-queue, not from userspace, not at any ROCm
version examined.**

## Hardware experiment, gfx1100 (2026-08-01) — the preemption hypothesis did NOT survive

Run on `hipx` GPU 0: RX 7900 XTX, gfx1100, `uuid:GPU-43390a851e296ee5`,
`0000:66:00.0` — the same card model as lhl's second faulting card, with
`cwsr_enable=1` and `mcbp=-1` matching his fault-time controls. Differs on
ROCm (7.14 production, not a 7.15 TheRock nightly) and kernel (Ubuntu
7.0.0-28, not CachyOS 7.1.3). Harnesses: `crates/redline-capi/examples/`
`{gpu_smoke,decode_chain_ab}.c` against a locally built `libredline_dispatch`.

Machine-generated artifact:
`docs/investigations/artifacts/2026-08-01-gfx1100-preemption-probe.json`,
produced by `scripts/6529-preemption-probe.sh` (reproducible; drives the C-ABI
harnesses only, so it does not need the Vulkan/glslc bench stack).

| Arm | Shape | Contention | Functional | Kernel faults |
| --- | --- | --- | --- | --- |
| elided | 4096 disp/IB, 10 reps | 1 process | 10/10 pass | *unobserved* |
| full-state | 4096 disp/IB, 10 reps | 1 process | 10/10 pass | *unobserved* |
| elided | 24000 disp/IB, 6 reps | 4 processes | 6/6 pass | *unobserved* |
| full-state | 24000 disp/IB, 6 reps | 4 processes | 6/6 pass | *unobserved* |
| elided | 24000 disp/IB, 6 reps | 16 processes | 6/6 pass | *unobserved* |
| **elided** | **24000 disp/IB, 4 reps** | **16 processes** | **4/4 pass** | **0 (journalctl)** |
| **full-state** | **24000 disp/IB, 4 reps** | **16 processes** | **4/4 pass** | **0 (journalctl)** |

### Correction: the first five arms could not see faults at all

`hipx` runs `kernel.dmesg_restrict=1`, so unprivileged `dmesg` fails. The
detection used during the exploratory arms was
`dmesg -T 2>/dev/null | grep -c …`, whose exit status is `grep`'s, not
`dmesg`'s — it counted matches in an **empty stream** and dutifully reported
`0`. Those five arms therefore prove only that the runs completed with exact
results; they say nothing about kernel faults. `journalctl -k` is readable and
is what the committed script now uses, falling back to reporting `null` rather
than `0` when no log source exists. The last two arms are the evidenced ones.

Independently confirmed afterwards: `journalctl -k --since "90 min ago"`
covering the whole session shows **0** lines matching
`VM_L2_PROTECTION_FAULT|SQC|REMOVE_QUEUE|GPU reset|page fault`. That part
stands: the probe arms themselves provoked nothing.

> **CORRECTION (2026-08-01, later the same day).** The next sentence used to
> read: "The host does carry 286 such lines earlier in this boot, all on
> `0000:bf:00.0` (the APU) on Jul 25 and Jul 27, status `0x00801030`/`0x00801031`
> — unrelated to this work and a different status word from lhl's `0x00801431`."
> **That was wrong, and it was the load-bearing claim behind "we cannot
> reproduce."** A full-boot sweep with `journalctl -k --since 2026-07-01` finds
> page faults on **two** BDFs, not one:
>
> | BDF | device | count |
> |---|---|---:|
> | `0000:66:00.0` | RX 7900 XTX, **gfx1100** | **78** |
> | `0000:bf:00.0` | Strix Halo APU, gfx1151 | 30 |
>
> The 78 on the gfx1100 are dated **2026-07-23, 13:37:49 - 16:44:44**, spread
> over **8 distinct PIDs**, escalating into **29 MES `REMOVE_QUEUE` failures and
> 24 `MODE1` full-device resets**. Every sampled fault carries
> `GCVM_L2_PROTECTION_FAULT_STATUS:0x00801431` with client `SQC (data) (0xa)` at
> address `0x0000000000000000`, `PERMISSION_FAULTS: 0x3`, `MAPPING_ERROR: 0x0`,
> escalating ~4 s later through `MES failed to respond to msg=REMOVE_QUEUE` and
> `MES might be in unrecoverable state` into `GPU reset begin!` / `MODE1 reset`.
>
> That is lhl's exact tuple and his exact escalation chain, on the same card
> model, **five days before he filed #6529**. The earlier claim almost certainly
> came from a search window that started after Jul 23 — the same class of
> mistake as the `dmesg` error documented directly above, and made while
> correcting it.
>
> **Consequence: "does not reproduce on our hardware" is false.** Any reply
> built on it must be rewritten. See "Draft reply" at the end, which is now
> marked DO NOT POST.

The 16-process arms are the ones that count. They genuinely oversubscribed the
device: VRAM went 28 MB -> 3.89 GB and retained replay slowed from ~18 to
~550 us/token, a **~30x slowdown**. Each 24000-dispatch IB then spanned ~13 s
of wall time against MES's 10 ms process quantum, so on the order of 1300
quantum switches elapsed *inside a single indirect buffer*, several thousand
across the arms. Not one address-zero fault.

**Conclusion: routine MES time-slicing does not lose SH state on gfx1100.**
If CP/MES reloaded COMPUTE_PGM_LO/HI from the zeroed MQD shadow on every
reconnect, an elided IB would fault almost immediately under this load. It
did not, through thousands of preemption opportunities. The kernel-source
findings remain factually correct — the MQD SH shadow really is zero, and
CWSR really does not save COMPUTE_PGM/USER_DATA — but the inference drawn
from them ("therefore a preempted elided IB dispatches from address 0") is
empirically false for the common path. CP/MES evidently preserves or
correctly re-establishes that state.

[INFERENCE] The 32x slowdown proves heavy contention; that the redline queue
was specifically descheduled *mid-IB* is inferred from 16 competing processes
against a 10 ms process quantum, not directly observed. Directly observing it
would need MES/HQD telemetry we do not have.

## Working hypothesis (revised after the experiment above)

1. **Queue/signal/allocation retirement and reuse** — lhl's original
   hypothesis, and now the leading one again by elimination. He found real
   upstream precedents (rocm-systems#8113 recycled completion signal,
   rocm-systems#6750 signal destroyed while a packet still referenced it).
2. **A rare preemption *variant*, not routine time-slicing.** The common
   quantum-switch path is now measured clean, so if preemption is involved at
   all it would have to be a less-travelled path: CWSR wave-save on eviction
   (TTM/USERPTR/SVM migration), suspend/resume, debugger attach, or a
   reset-recovery path. Untested here — an eviction-driven test needs memory
   pressure, which was deliberately avoided on a shared host.
3. Cross-device binding (two-card host) — mostly ruled out for the W7900 run,
   RX run's path unverified.

Do not present preemption as the explanation publicly. The honest public
contribution is now: construction ruled out, source facts about MQD/CWSR
state, AND a negative result that narrows the search away from routine
preemption.

## Landed in-tree (UNCOMMITTED as of 2026-08-01 — check `git status`)

Nothing here is a fix for #6529. Two items are guards for a *different*
(construction) bug class, one is a diagnostic A/B knob, the rest are
integration hardening found while auditing. Say exactly this in public.

- `crates/redline-capi/src/validate.rs` (new): finalize-time stream validator;
  rejects zero COMPUTE_PGM_LO writes and DISPATCH_DIRECT without a nonzero
  program address earlier in the stream. Wired into `finalize_ib` and
  `build_multi_ib` -> RL_ERR_COMPILE + stderr instead of a device reset.
- `crates/redline-capi/src/gpu.rs`: `REDLINE_PM4_FULL_STATE=1` builder opt-out
  of SH elision (full register state per dispatch), read once through a
  `LazyLock` in a single shared constructor so no future builder path can
  miss it; `Pm4Commands::dwords()`.
- `crates/redline-capi/src/gpu.rs`: finalize-time diagnostics. Every
  IB-creation error now prints its cause instead of collapsing to a bare
  `RL_ERR_COMPILE`, plus a specific pre-check for the 20-bit
  `INDIRECT_BUFFER` size ceiling naming the recorded dword count.
- `crates/redline-rocr/src/runtime.rs`: `GpuSelector::NameContains` now
  refuses an ambiguous match (`GpuNameAmbiguous`) listing every candidate as
  `ordinal:name (pci)`, instead of silently binding the first hit. Matching is
  on the agent name only; the PCI identity is reporting, never a selector.
- `examples/hipfire-6409/src/bin/hipengine_exact{,_memory}.rs`: fail closed
  unless the selected device is gfx12 (both hardcode gfx12 command buffers).
- `crates/redline-dispatch/src/aql/replay.rs` + that crate's examples:
  encoder/device family mismatch is refused at IB construction (see
  FamilyGuard notes in this session; verify with `git diff`).
- `scripts/6529-contention-ab.sh` + `bench/contention_load.hip` (new):
  contention A/B harness for gfx1100 hosts. Refuses to run on non-gfx11
  without `--force`, requires an explicit `ROCR_VISIBLE_DEVICES`, records
  `cwsr_enable`/`mcbp` per run, sweeps `REDLINE_PM4_FULL_STATE` x contention,
  captures per-cell dmesg deltas.
- `docs/INTEGRATION.md`: retained-IB size ceiling section, contention A/B
  procedure, corrected module-parameter guidance.

### Hardware smoke (gfx1201 — no gfx1100 available on this workstation)

Run with `crates/redline-capi/examples/{gpu_smoke,decode_chain_ab}.c` against
`libredline_dispatch.so` built from this tree:

- `gpu_smoke` 1024 and 256 dispatches: counter exact, PASS, both modes.
- `decode_chain_ab` 2048-token 2-kernel dependent chain (4096 dispatches in
  one retained IB): `acc` exact in both modes; 21.34 us/token elided vs
  19.68 us/token full-state (host-timed, difference inside run-to-run noise).
- The validator accepts every real stream produced; no false rejections.
- Full-state mode measurably changes what is emitted: the IB ceiling moves
  from ~58k dispatches to ~22k. Exact densities from the new diagnostic:
  **18.0 dwords/dispatch elided, 47.0 full-state (2.61x)**.

That last number is the useful A/B cost estimate for gfx10/11 integrators:
full-state does not measurably cost host time on this shape, but it costs
2.61x the IB, so long graphs may need splitting.

## Open work (in order)

Issue state re-checked live against the GitHub API on 2026-08-01: #6529 is
OPEN, `status: triage`, **zero comments**, last updated 2026-07-30T18:21:51Z.
No AMD response. #6409 unchanged since lhl's 2026-07-28 comment.

1. **Post the reply (now — do not gate this on gfx1100 hardware).** Two
   reasons this moved to the front. First, lhl asked a direct question in
   ROCm/ROCm#6409#issuecomment-5100210675 — "Before I go chasing for repro, is
   this something you've observed?" — and it has been unanswered since
   2026-07-28. Second, and more important: his stated next diagnostic is a
   recreate-per-cycle retained IB with queue/ring/doorbell generation
   telemetry and reuse-versus-recreate controls. That instrument targets his
   retirement/reuse hypothesis. If the preemption mechanism is right, it comes
   back clean and costs him days. Tell him before he builds it. Draft below.
2. **gfx1100 hardware A/B — we can run this ourselves.** An earlier revision of
   this doc said it "needs lhl or an RDNA3 host"; that was wrong. The `hipx`
   host carries an RX 7900 XTX (gfx1100, `uuid:GPU-43390a851e296ee5`,
   `0000:66:00.0`, ROCr index 0) — the same card model as lhl's second faulting
   card — with `cwsr_enable=1` and `mcbp=-1`, matching his fault-time controls.
   It differs on ROCm (7.14 production, not a 7.15 TheRock nightly) and kernel
   (Ubuntu 7.0.0-28, not CachyOS 7.1.3). Toolchain present: cargo + hipcc.
   Run `scripts/6529-contention-ab.sh --device uuid:GPU-43390a851e296ee5`,
   sweeping `cwsr_enable=0` vs `1` and `REDLINE_PM4_FULL_STATE=0` vs `1`.
   Faults tracking `cwsr_enable=1` + elision confirm the mechanism.
   NOTE: hipx GPU 1 is a Strix Halo APU — deny it in the manifest before
   running anything that provokes device resets.
3. **Decide the shippable default.** Given that no per-queue opt-out exists,
   the honest options for gfx10/11 are: (a) make full-state the default on
   gfx10/11 and keep elision for gfx12, or (b) keep elision and document
   `cwsr_enable=0` as a system requirement. (a) costs 2.61x IB size and
   [INFERENCE] some GPU-side CP fetch time not yet measured on gfx1100; (b)
   pushes a device-wide, debugger-breaking toggle onto users. Measure (a) on
   gfx1100 before choosing.
4. **The one question for AMD** (only they can answer): on a CWSR queue with
   `QSWITCH_MODE` and `PRELOAD_REQ | PRELOAD_SIZE=0x55`, does CP/MES restore
   COMPUTE_PGM_LO/HI and COMPUTE_USER_DATA from the MQD on reconnect? The
   driver leaves those MQD dwords zero. If the answer is yes, retained PM4
   with SH elision is unsound on any preemptible queue, by construction, and
   should be documented as such.
5. Consider proposing a KFD uapi extension (per-queue no-preempt or an
   MQD-seeded SH block) only after (4) is answered — not before.

## Gotchas (hard-won, do not relearn)

- HIP_VISIBLE_DEVICES is a CLR (HIP) filter; Redline enumerates via
  `hsa_iterate_agents`, which honors ROCR_VISIBLE_DEVICES only. On multi-card
  hosts pin ROCR_VISIBLE_DEVICES explicitly.
- `GpuDevice::name()` is the HSA `AGENT_INFO_NAME` — `gfx1100`, **not** the
  marketing name. On lhl's W7900 + RX 7900 XTX host both agents report
  `gfx1100`, so a `NameContains("7900")` selector matches *neither* card and
  any name-based selector is inherently ambiguous there. (An earlier revision
  of this doc claimed it matched both; that was wrong.) Ambiguous name
  selection is now refused outright.
- gfx10/gfx11 share the legacy compute register map; gfx12 differs, but
  COMPUTE_PGM_LO is 0x20c in both (validator is family-agnostic).
- `examples/hipfire-6409/src/main.rs` — the binary that actually produced the
  fault reports — derives its PM4 family from the device name via
  `Pm4Family::from_device` (`redline_backend.rs:103-114`) and bails on an
  unsupported architecture. It was never at risk of misencoding. The
  `hipengine_exact*` binaries were, and are now guarded.
- lhl ran OUR example binary from this repo
  (redline 33683f3 + hipfire bridge 455ffb9). Keep that example reproducible.
- One retained PM4 IB holds at most 1,048,575 dwords (20-bit INDIRECT_BUFFER
  size field, `packet.rs:431`). This fails closed and is now diagnosed by
  name; it is not a fault source, but it *is* what an integrator hits first
  when they enable full-state mode on a long graph.
- Reply discipline: credit lhl's non-reproduction data first; propose
  experiments, don't assert; never claim a fix that isn't demonstrated.

## Draft reply for ROCm/ROCm#6529 (rev 3 — **DO NOT POST**, superseded)

> **BLOCKED 2026-08-01.** Rev 3 argues from "we cannot reproduce this on our
> gfx1100." That premise is false — see the CORRECTION above: our gfx1100 threw
> 78 faults with lhl's exact signature on 2026-07-23, five days before he filed.
> Posting rev 3 would put a demonstrably wrong claim on a public AMD tracker
> that lhl is reading.
>
> Rev 4 needs the backend question answered first (redline leg vs stock HIP
> leg), because the two answers produce opposite messages. The decisive test is
> a live reproduction attempt on hipx's gfx1100 running each backend in turn;
> it risks provoking MODE1 resets on that card, which is recoverable and does
> NOT touch the deny-listed APU, but needs an explicit go-ahead before running.
>
> Rev 3 is kept below unedited, because its reasoning about the preemption
> hypothesis and the MQD/CWSR audit is still sound and rev 4 should reuse it.

Rev 3 exists because rev 2 was wrong. Rev 2 argued mid-IB preemption as the
mechanism; the gfx1100 experiment above then failed to reproduce a fault
through ~8400 quantum-switch opportunities, which contradicts it. Posting rev
2 would have sent lhl away from his own (now better-supported) hypothesis on
the strength of an inference we had already disproved in-house.

---

Answering your question first: I have not reproduced it. I do have comparable
hardware — an RX 7900 XTX (gfx1100, same model as your second card) on a host
with `cwsr_enable=1` / `mcbp=-1` matching your fault-time controls, though on
ROCm 7.14 production and a 7.0 Ubuntu kernel rather than your 7.15 TheRock
nightly on CachyOS 7.1.3. I ran an experiment against a specific mechanism and
it came back negative, which I think is worth more to you than another
speculation, so: details below, and the short version is **your
retirement/reuse hypothesis survives and mine didn't.**

First, construction is ruled out. I audited every path by which a zero can
reach the shader in current redline: both encoders reject `code_entry == 0`
and a null kernarg host-side before any stream mutation, the C API refuses
cross-agent module mixing, and the finalized IB owns its modules and kernargs.
There is now also a finalize-time validator that rejects any IB which could
dispatch with a zero program address. Packet construction cannot emit your
tuple.

What I chased instead: retained PM4 depends on SH-register persistence across
an IB. Redline writes COMPUTE_PGM_LO/HI and the user-data SGPRs once and lets
later `DISPATCH_DIRECT`s inherit them, whereas every AQL packet carries
`kernel_object` and the kernarg pointer so CP re-materialises that state per
dispatch. In amdkfd, three things are true and initially looked damning:

- `struct v11_compute_mqd` carries `compute_pgm_lo/hi` (MQD offsets 0xD/0xE)
  and `compute_user_data_0..15`, and `init_mqd` memsets the MQD and never
  writes them; `update_mqd` never writes them either. That shadow is zero for
  the life of a normal KFD queue.
- `hqd_load_v11` MMIO-loads only `cp_mqd_base_addr_lo` (0x80) through
  `CP_HQD_PQ_WPTR_HI`, so those SH dwords are outside the host-loaded window.
- The CWSR trap handler saves and restores wave state only — VGPRs, SGPRs,
  LDS, hwregs. No COMPUTE_PGM or USER_DATA save/restore appears in it.

So: if CP/MES re-established SH state from that zero MQD on reconnect, a
preempted elided IB would dispatch from address 0 — your exact tuple. I tested
it. On the 7900 XTX I ran 24000-dispatch retained IBs against 16 competing GPU
processes, both with SH elision on and with it off. That genuinely
oversubscribed the device: VRAM 28 MB -> 3.89 GB, and retained replay slowed
from ~18 to ~550 us/token, a ~30x slowdown. Each IB then spanned ~13 s of wall
time against MES's 10 ms process quantum, so roughly 1300 quantum switches
elapsed inside a single indirect buffer, several thousand across the run.

**8/8 runs passed with exact results, and `journalctl -k` recorded zero new
`VM_L2_PROTECTION_FAULT` / `SQC` / `REMOVE_QUEUE` / `GPU reset` lines across
the whole session.** (Worth stating because I got this wrong first time: that
host has `kernel.dmesg_restrict=1`, and my initial check piped `dmesg` into
`grep -c`, which happily counts zero matches in an empty stream. The numbers
above are from `journalctl -k`, which is readable there.) Script and JSON
artifact are in the redline tree if you want to run it on your cards:
`scripts/6529-preemption-probe.sh`.

I read that as: routine MES time-slicing does not lose IB-written SH state on
gfx1100. The three source facts above are still facts, but the conclusion I
drew from them is empirically false for the common path — CP/MES evidently
preserves or correctly re-establishes that state. So preemption is not your
bug, unless it is a much rarer path than time-slicing (CWSR wave-save on
TTM/USERPTR/SVM eviction, suspend/resume, or debugger attach), which I have
not tested and which your workload could plausibly hit where mine did not.

That pushes me back toward what you already said: queue/signal/GPU-visible
allocation retirement and reuse. Your recreate-per-cycle harness with
generation telemetry looks like the right instrument, and rocm-systems#8113
and #6750 are the right precedents. I have nothing better to offer than
agreement there.

One correction to the controls, in case it saves a reboot: `mcbp=-1` leaves
`adev->gfx.mcbp` false on any non-SR-IOV host (`amdgpu_device_set_mcbp` only
forces it on for `mcbp=1` or a VF), and MCBP governs the graphics ring, not
the KFD compute HQD/MES path. Your fault host was CWSR-on, MCBP-off; setting
`mcbp=0` should be a no-op. Relatedly, for anyone who goes looking: there is
no per-queue preemption opt-out to reach for. `kfd_ioctl_create_queue_args`
has CSA pointers and no flags field, libhsakmt allocates a save area
unconditionally on modern parts, and no HSA/AMD API or ROCr env var disables
it — `hsa_amd_queue_set_priority` only changes who wins a quantum.

Still worth an AMD answer, though it is now a smaller question: on a CWSR
queue with `QSWITCH_MODE` and `PRELOAD_REQ | PRELOAD_SIZE=0x55`, what
re-establishes COMPUTE_PGM_LO/HI and COMPUTE_USER_DATA on reconnect, given the
driver leaves those MQD dwords zero? Empirically something does. Knowing
whether that holds across *all* preempt paths — including eviction and
suspend, not just quantum switch — would either close this line or reopen it
precisely.

On the redline side, for completeness: `REDLINE_PM4_FULL_STATE=1` disables SH
elision so every dispatch re-emits full state. On gfx1100 that costs 18.0 ->
41.0 dwords per dispatch (2.28x the IB, dropping the per-IB ceiling from
~58k to ~26k dispatches), and its steady-state replay cost is within noise on
a two-kernel decode chain (18.4 vs 18.7 us/token at 12000 tokens). If you ever
want to A/B it against a fault, it is one environment variable — but given the
result above I would not expect it to change anything, and I am no longer
suggesting it as a diagnostic.
