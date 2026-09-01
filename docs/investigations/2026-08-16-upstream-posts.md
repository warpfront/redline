<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# Ready-to-post upstream drafts — updated 2026-08-23

> **Posting status, 2026-08-23.** Two of these are now posted upstream:
>
> - #10021 confirmation → https://github.com/ROCm/rocm-systems/issues/10021#issuecomment-5388455118
> - #9360 review → https://github.com/ROCm/rocm-systems/pull/9360#issuecomment-5388455446
>
> The #9360 comment differs from the draft below it: the draft argued against the
> patch from the `hipMemSetAccess` rule, but the PR's actual stated motivation is
> *kernel* faults at 4 KiB boundaries. That claim was tested directly with
> `bench/vmm/vmm_4k_kernel_access.cpp` (512 handles, 511 boundaries, three access
> shapes including straddling 8-byte stores, four architectures, zero faults) and
> could not be reproduced. The posted comment leads with that, and adds the
> hypothesis that the real blocker is descriptor exhaustion at minimum
> granularity, which caps a 4 KiB-handle pool at roughly 4 MiB.
>
> **Tracker note, 2026-09-01.** `ROCm/ROCm` (now `ROCm/legacy-rocm-build`) carries
> a README banner: "This repository will be deprecated soon. Use ROCm/TheRock
> moving forward, including for issues and discussions." Its `.github/ISSUE_TEMPLATE`
> is already gone. `#6409`, `#6529` and `#6603` live there; no migration policy is
> stated. AMD's own 2026-08-27 split of `#6409` went to `rocm-systems`
> (`#10834`, `#10836`) and `llvm-project` (`#219248`), so component-level reports
> still belong in `rocm-systems`; TheRock is the user-facing umbrella. Observed
> triage there: a HIP-runtime report (`TheRock#7625`, 08-25) got `status: triage`,
> an assignee and an AMD engineer reply within 7 days, while `#6529` has been
> silent in the legacy repo since 08-04 and `rocm-systems#10604` is unlabelled
> after 9 days. Nothing of ours has been mirrored to TheRock; searches for
> redline / 6409 / 6529 / hipMemCreate / graph replay there return zero.


Five drafts, each supported by measurements taken on this fleet. Numeric
observations below come from those fleet measurements; where a mechanism is
interpreted from the observations, the interpretation is identified as such and
scoped to the probes' conditions. Post order is by value-per-effort: (1) is new
and unreported, (2) prevents a fleet-wide regression, (3) narrows an unengaged
corruption bug.

Measurement environment for all probes in this document:

| Host | GPUs | ROCm | Kernel |
| --- | --- | --- | --- |
| local | 1x gfx1201 (RX 9070 XT) | 7.14.0 (`/opt/rocm/core-7.14`) | 7.0.0-28 |
| hiptrx | 4x gfx1201 (Radeon AI PRO R9700, 32 GiB) | 7.14.0 | 7.0.0-28 |
| hipx | gfx1100 (RX 7900 XTX), gfx1151 (Strix Halo), gfx1010 (RX 5700 XT), gfx1030 (RX 6950 XT) | 7.14.0 | 7.0.0-29 |

DRM 3.64.0 on all hosts. Soft `RLIMIT_NOFILE` 1024 / hard 524288 unless otherwise noted.

---

## 1. NEW ISSUE — HIP VMM capacity is silently capped by the process file-descriptor limit

**Target:** `ROCm/rocm-systems`, component `clr` (HIP VMM). No existing issue found.

**Title:** `[clr] hipMemCreate consumes a file descriptor per handle, capping HIP VMM at ~2 GiB under the default ulimit -n`

### Body

With 2 MiB physical handles and the stock 1024 soft `RLIMIT_NOFILE`, available
descriptor slots cap the process before VRAM does. In general the bound is the
smaller of physical memory and (available descriptor slots x physical
allocation-handle size): whichever is exhausted first determines how many handles
can be successfully created.

**The failure is reported as `hipErrorOutOfMemory` while gigabytes of VRAM remain
free**, which is what makes this hard to diagnose: every signal points at the
GPU being full when the exhausted resource is actually descriptors.

Measured across four architectures under ROCm 7.14.0, recommended allocation
granularity 2 MiB on all of them, with virtual address space pre-reserved so VA
is never the limit. All numbers below are from a corrected probe that
distinguishes descriptor exhaustion from VRAM exhaustion by direct observation.
For descriptor-bound rows the evidence is that `opendir("/proc/self/fd")` itself
fails with `EMFILE` at the same point `hipMemCreate` returns
`hipErrorOutOfMemory` — the process has no descriptor slots left. Memory-bound
rows instead show free VRAM near zero with hundreds of FD slots still available.
This direct `EMFILE` observation is the strongest single fact in the report.

Chunk-size sweep at the stock soft limit (1024), per architecture.
`successfully created physical-handle bytes` is `handles x chunk size`; the
probe loops on `hipMemCreate` only and does not reserve, map, set access, or
touch memory, so this is strictly handle-creation capacity, not mapped-or-touched
bytes.

**gfx1030 — RX 6950 XT, 15.96 GiB free:**

| Chunk | Handles | Successfully created physical-handle bytes | Attribution | Supporting observation |
| --- | --- | --- | --- | --- |
| 2 MiB | 1015 | 1.98 GiB | descriptor-bound | `opendir("/proc/self/fd")` fails `EMFILE` at stop; 14310 MiB VRAM still free |
| 8 MiB | 1015 | 7.93 GiB (4.0x) | descriptor-bound | `EMFILE`; 8220 MiB VRAM still free |
| 32 MiB | 510 | 15.94 GiB | memory-bound | FD headroom 504 slots; 20 MiB VRAM free at stop |
| 128 MiB | 127 | 15.88 GiB | memory-bound | FD headroom 887 |
| 512 MiB | 31 | 15.50 GiB | memory-bound | FD headroom 983 |

**gfx1100 — RX 7900 XTX, 23.95 GiB free:**

| Chunk | Handles | Successfully created physical-handle bytes | Attribution | Supporting observation |
| --- | --- | --- | --- | --- |
| 2 MiB | 1015 | 1.98 GiB | descriptor-bound | `EMFILE`; 22492 MiB VRAM still free |
| 8 MiB | 1015 | 7.93 GiB (4.0x) | descriptor-bound | `EMFILE`; 16402 MiB VRAM still free |
| 32 MiB | 766 | 23.94 GiB | memory-bound | FD headroom 248; 10 MiB free |
| 128 MiB | 191 | 23.88 GiB | memory-bound | FD headroom 823 |
| 512 MiB | 47 | 23.50 GiB | memory-bound | FD headroom 967 |

**gfx1151 — Radeon 8060S / Strix Halo (APU), 95.84 GiB free:**

32 MiB and above were capped by a deliberate 10 GiB safety budget on this shared
APU and are reported as `budget-cap (not a device limit)`, not as a device
limit. Only the rows below the cap are device-relevant:

| Chunk | Handles | Successfully created physical-handle bytes | Attribution | Supporting observation |
| --- | --- | --- | --- | --- |
| 2 MiB | 1015 | 1.98 GiB (2.1% of 95.84 GiB) | descriptor-bound | `EMFILE`; 96114 MiB VRAM still free |
| 8 MiB | 1015 | 7.93 GiB (4.0x) | descriptor-bound | `EMFILE`; 90024 MiB VRAM still free |
| 32 MiB | — | — | budget-cap (not a device limit) | capped by 10 GiB safety budget |
| 128 MiB | — | — | budget-cap (not a device limit) | capped by 10 GiB safety budget |
| 512 MiB | — | — | budget-cap (not a device limit) | capped by 10 GiB safety budget |

**gfx1201 — Radeon AI PRO R9700, 31.79 GiB free:**

| Chunk | Handles | Successfully created physical-handle bytes | Attribution | Supporting observation |
| --- | --- | --- | --- | --- |
| 2 MiB | 1015 | 1.98 GiB | descriptor-bound | `EMFILE`; 30520 MiB VRAM still free |
| 8 MiB | 1015 | 7.93 GiB (4.0x) | descriptor-bound | `EMFILE`; 24430 MiB VRAM still free |
| 32 MiB | 1015 | 31.72 GiB | coincident (both exhausted) | FDs exhausted **and** only 70 MiB VRAM free — cause cannot be separated at this point and is not claimed as either |
| 128 MiB | 254 | 31.75 GiB | memory-bound | FD headroom 760 |
| 512 MiB | 63 | 31.50 GiB | memory-bound | FD headroom 951 |

Reading: at the 2 MiB granularity the runtime itself recommends, all four
architectures stop at 1015 handles / 1.98 GiB with descriptors provably exhausted
(`EMFILE`) and 8–94 GiB of VRAM unused, across a ~6x VRAM spread. 8 MiB scales
exactly 4.0x (7.93 GiB), confirming the cost is one descriptor per handle
regardless of handle size. The crossover to memory-bound tracks VRAM, and on
gfx1201 the two limits land almost exactly together at 32 MiB — the 32 MiB row
is therefore reported as coincident (both exhausted) without attributing a cause.

For rows that now carry memory-bound evidence (FD headroom in the hundreds,
VRAM near zero), the crossover from descriptor-bound to memory-bound was
observed by the corrected probe, which attempts past the precomputed cap. No
crossover is claimed for the gfx1151 rows above the safety budget, because the
loop stopped at the budget, not at a device limit.

### Raised-limit arm, corrected

The earlier raised-limit figures in this document came from a buggy reporting
path: the probe broke at an unreported `hipMemMap`/`hipMemSetAccess` stage and
printed a stale success count. Those numbers have been removed and are not
restated here.

Corrected run, gfx1201, with an in-process `setrlimit(soft=hard)`:

- `RLIMIT_NOFILE` before: soft=1024 hard=524288; after: soft=524288 hard=524288
  (unprivileged, verified by readback)
- Chunk 2 MiB, reservation sized to initial free memory: **16275 handles =
  31.79 GiB**, stopping stage `completed-requested-count`
- VRAM free at stop: **0.00 GiB**, FD headroom **508003**
- Note printed by the probe: completing the requested count proves only that the
  preselected span (sized to initial free memory) was filled — it is not an
  address-space limit

Interpretation, scoped: raising the soft limit removed the 1015-handle ceiling
and the run then consumed all free VRAM with descriptors to spare. This raises
the descriptor-limited capacity far beyond any current device's memory, while
VRAM, VA, mapping and other runtime limits still apply — it does not remove
those other ceilings.

### The mechanism, measured rather than inferred

Descriptor accounting shows one net dmabuf descriptor per successful handle, in
these runs, rather than a formally proven per-call bijection. The
`/proc/self/fd` snapshot is not synchronized against runtime background threads,
so the supporting evidence is the exact delta reproduced on four systems:

- 64 x `hipMemCreate` raises the `/proc/self/fd` count by **exactly 64**;
  every new descriptor has `readlink` target **`/dmabuf:`**
- `hipMemRelease` returns all 64 (delta −64)
- a 64-allocation **`hipMalloc` control changes the count by zero**

This was reproduced identically on gfx1030, gfx1100, gfx1151 and gfx1201. It is
one dmabuf descriptor per handle held for the handle's lifetime, released
correctly, and specific to the VMM path in these runs.

### Successfully created physical-handle bytes scale with handle size until VRAM binds

The same probe at the **stock** `ulimit -n 1024`, varying the per-handle chunk
size. The handle count is identically 1015 at 2 MiB and 8 MiB on every card
until VRAM becomes the binding constraint instead. The corrected probe attempts
past any precomputed cap and reports the actual stopping condition, so for rows
above that now carry memory-bound evidence the crossover was observed; for the
gfx1151 rows truncated by the 10 GiB safety budget, no device crossover is
claimed.

See the per-architecture tables above. Note gfx1201 at 32 MiB is the coincident
case: 1015 x 32 MiB = 31.72 GiB just fits under its 31.79 GiB free, with FDs
exhausted and only 70 MiB free simultaneously — cause is not assigned.

### Two mitigations, both verified

1. **Larger physical chunks. No privileges required.** Because a handle costs one
   descriptor irrespective of size, successfully created physical-handle bytes
   scale directly with chunk size until VRAM becomes the binding constraint.
   With 32 MiB handles the 15.96 GiB and 23.95 GiB cards cross over to
   memory-bound and consume nearly all free VRAM at the stock descriptor limit.
   This is the mitigation that works inside a container that caps `RLIMIT_NOFILE`.
2. **Raise `RLIMIT_NOFILE`.** Soft 1024 against hard 524288 on these hosts, and
   soft may be raised to hard unprivileged. Verified on gfx1201 to remove the
   1015-handle ceiling: 16275 handles / 31.79 GiB with 0.00 GiB free and
   508003 FD slots still available, stopping at `completed-requested-count`
   (filling the preselected span, not an address-space limit).

Neither is documented, and the failure that sends you looking for them reports
`hipErrorOutOfMemory`.

### Why this matters

Consumers that build large allocations out of VMM handles hit this well before
they exhaust the device, and the error they see blames the wrong resource. It is
a plausible explanation for why llama.cpp ships `GGML_HIP_NO_VMM` and falls back
to a non-freeing legacy pool, and for long-context out-of-memory reports on
cards with plenty of VRAM free. Because the practical VMM budget is
`(available descriptor slots x physical allocation-handle size)`, it also
interacts with any change that raises granularity: at 2 MiB and a 1024 FD limit
the ceiling is ~2 GiB, and raising the reported `MINIMUM` from 4 KiB to the
already-recommended 2 MiB would not help callers who already allocate 2 MiB
handles.

### Our own exposure (hipfire, current build)

hipfire does **not** avoid this today, and its allocation shape makes it
somewhat more exposed than a single-arena consumer:

- `DEFAULT_VMM_PHYSICAL_CHUNK_BYTES = 2 * 1024 * 1024`
  (`hipfire/crates/saddle-core/src/kv.rs:61`) — VMM-backed KV grows in 2 MiB
  handles, so one handle is consumed per 2 MiB of growth.
- No `setrlimit` / `RLIMIT_NOFILE` / `NOFILE` reference anywhere under
  `hipfire/crates/` — the 1024 soft limit is inherited and never raised.
- `Gpu::vmm_arenas` is keyed per tensor (`rdna-compute/src/dispatch.rs`,
  `alloc_vmm_tensor` / `grow_vmm_tensor`), so **each** VMM KV tensor owns a
  separate arena while all of them draw handles from the one process-wide
  descriptor budget. A per-layer main + indexer cache layout divides ~1015
  handles across every layer.

Symptom to expect: KV growth fails with `hipErrorOutOfMemory` while the device
still reports tens of GiB free — i.e. the misattribution described above, hitting
our own long-context path.

**Fix, unprivileged and self-contained.** Measured headroom on both hosts:

```
RLIMIT_NOFILE soft=1024  hard=524288
raising soft -> hard as a normal user: verified OK (new soft = 524288)
```

Raise the soft limit to the hard limit once at startup, before any VMM arena is
created:

```c
struct rlimit rl;
if (getrlimit(RLIMIT_NOFILE, &rl) == 0 && rl.rlim_cur < rl.rlim_max) {
    rl.rlim_cur = rl.rlim_max;          // unprivileged: soft may rise to hard
    setrlimit(RLIMIT_NOFILE, &rl);      // non-fatal on failure; log and continue
}
```

524288 handles x 2 MiB is ~1 TiB of VMM addressability, so this raises the
descriptor-limited capacity far beyond any current device's memory, while VRAM,
VA, mapping and other runtime limits still apply. It should be logged (old and
new soft limit) so a constrained container that cannot raise it is diagnosable
rather than silently capped.

### Open question before contacting llama.cpp

llama.cpp disables VMM via `GGML_HIP_NO_VMM` and falls back to a legacy pool
that never frees, which is implicated in long-context OOM. It is **not yet
established** which VMM defect motivated that: the 4 KiB granularity / sub-range
behaviour discussed under `rocm-systems#2516`, or the descriptor ceiling
documented here. These are independent, and only the second is proven by the
measurements in this document. Read their ROCm VMM path first. If the descriptor
ceiling is the actual motivation, the useful contribution is not a bug report —
it is telling them VMM can be re-enabled behind a `setrlimit` call.

### Reproducer

Self-contained, no framework required (full source attached / available on
request):

```
hipMemGetAllocationGranularity(&gran, &prop, Recommended);   // 2 MiB on RDNA
hipMemAddressReserve(&base, gran * N, gran, nullptr, 0);
for (i = 0; i < N; ++i) {
    if (hipMemCreate(&h, gran, &prop, 0) != hipSuccess) break;  // stops at ~1015 at stock limit
    hipMemMap((char*)base + i * gran, gran, 0, h, 0);
    hipMemSetAccess((char*)base + i * gran, gran, &desc, 1);
}
printf("mapped %zu handles = %.2f GiB\n", i, i * gran / 1073741824.0);
```

The corrected probe also checks `opendir("/proc/self/fd")` with `EMFILE` at the
stopping point and reports FD headroom and VRAM free, so descriptor-bound and
memory-bound stops can be distinguished. Run it once as-is and once after
raising the soft limit to the hard limit in-process.

### Questions

1. Is the descriptor-per-handle cost intended, or an artifact of how handles are
   exported to the kernel driver?
2. Should HIP surface a distinct error (or a diagnostic) when handle creation
   fails because of `RLIMIT_NOFILE` rather than VRAM?
3. Is raising `ulimit -n` the supported mitigation for VMM-heavy consumers, and
   if so is it documented anywhere?

---

## 2. COMMENT on `rocm-systems#9360` — the 2 MB granularity override is not needed on RDNA, and would reduce capability

**Target:** PR `ROCm/rocm-systems#9360` (open, no human review since 2026-08-02).

### What the PR does

Read from the diff directly. In `projects/clr/hipamd/src/hip_vm.cpp`,
`hipMemGetAllocationGranularity` computes min/recommended from device info and
then, when `!useHostDevice`, **unconditionally** overwrites the result with
`2 * 1024 * 1024`, with a `HIP_VMM_GRANULARITY` environment escape. The
host/pinned path is unchanged. Three properties of the patch are worth stating
explicitly, because the accompanying test only exercises the third:

1. **It is not architecture-gated.** The comment says "On RDNA4 (gfx1201)", but
   the code applies to every device that is not the host pseudo-device.
2. **It overrides the `Minimum` query as well as `Recommended`.** The assignment
   happens after the branch that selects between them, so
   `hipMemAllocationGranularityMinimum` also returns 2 MB and callers lose any
   way to discover the real minimum.
3. The new test asserts `min_granularity >= 2 MB` only when `gcnArchName`
   contains `gfx1201`, so the universal behaviour change is untested.

### Measurement

The stated justification is, verbatim from the patch comment:

> On RDNA4 (gfx1201), the runtime-reported device VMM granularity (4 KB) is
> insufficient: GPU kernels accessing VMM-mapped memory at 4 KB-aligned
> boundaries trigger illegal memory access faults.

That is a specific and testable claim, and it does not reproduce here. I ran
exactly that shape — a 4 KiB VMM mapping written **by a kernel** — across five
RDNA generations on ROCm 7.14.0:

| Arch | GPU | MINIMUM | RECOMMENDED | 4 KiB map + SetAccess + kernel write + readback |
| --- | --- | --- | --- | --- |
| gfx1010 | RX 5700 XT | 4 KiB | 2 MiB | pass, exact |
| gfx1030 | RX 6950 XT | 4 KiB | 2 MiB | pass, exact |
| gfx1100 | RX 7900 XTX | 4 KiB | 2 MiB | pass, exact |
| gfx1151 | Strix Halo | 4 KiB | 2 MiB | pass, exact |
| gfx1201 | RX 9070 XT / R9700 | 4 KiB | 2 MiB | pass, exact |

No illegal-access fault on any of them, including gfx1201 on two different
boards. A single `hipMemSetAccess` spanning 64 separate 4 KiB handles also
succeeds and reads back bit-exact. If the fault is real it must depend on
something narrower than "kernels accessing 4 KB-aligned VMM memory" — a
particular access pattern, allocation flag, or driver version — and it would be
worth pinning down, because the current patch treats it as universal.

### What the real rule appears to be

`hipMemSetAccess` succeeds if and only if the range covers a whole number of
**complete mapped handles** — it must begin on a handle boundary and end on a
handle boundary. The constraint is alignment *relative to where handles were
mapped*, not absolute alignment of the address, and it tracks the handle size
actually used rather than any fixed device requirement.

Against a contiguous run of 2 MiB handles mapped from the reservation base, so
that handle boundaries coincide with 2 MiB offsets:

| Sub-range against 2 MiB handles | Result |
| --- | --- |
| offset 0, len 2 MiB (one handle exactly) | OK |
| offset 2 MiB, len 4 MiB (two handles exactly) | OK |
| offset 0, len 8 MiB (all handles) | OK |
| offset 1 MiB, len 2 MiB (straddles two handles) | `invalid argument` |
| offset 1 MiB, len 1 MiB (inside one handle) | `invalid argument` |
| offset 0, len 1 MiB (starts on a boundary, ends inside) | `invalid argument` |
| offset 4 KiB, len 4 KiB (4 KiB slice of a 2 MiB handle) | `invalid argument` |

It is worth being precise that this is a boundary rule and not an
absolute-alignment rule, because the two are easy to conflate and only the
second would justify raising the reported minimum. Mapping a single 2 MiB handle
at a merely page-aligned offset — base + 4 KiB, which is *not* 2 MiB aligned —
and then setting access over exactly that handle is accepted:

```
---- handle size 2M ----
offset     off/min        hipMemMap        SetAccess(len=hs) SetAccess(len=min)
0          0 x min        ok               ok               invalid-value
4K         1 x min        ok               ok               invalid-value
8K         2 x min        ok               ok               invalid-value
64K        16 x min       ok               ok               invalid-value
1M         256 x min      ok               ok               invalid-value
2M         512 x min      ok               ok               invalid-value
```

Every offset in that sweep is accepted at full-handle length, and every one is
rejected at a 4 KiB sub-length. With 4 KiB handles the same sweep accepts 4 KiB
lengths at every offset. So what the runtime enforces is whole-handle coverage;
the reported *minimum* granularity is not what is being checked.

### Consequence for this PR

Forcing `RECOMMENDED` semantics onto the `MINIMUM` query raises the floor for
every device. Callers doing fine-grained mapping lose capability that works
today, and each handle costs 2 MiB of physical memory minimum — which interacts
with the descriptor ceiling described in the issue above, since the practical
VMM budget is (available descriptor slots x physical allocation-handle size). If
the goal is to steer callers toward 2 MiB,
`hipMemAllocationGranularityRecommended` already reports exactly that, and
callers who want the larger value can ask for it. Raising the reported
`MINIMUM` from 4 KiB to the already-recommended 2 MiB would not help callers
who already allocate 2 MiB handles.

If there is a specific gfx1201 failure behind this change, I have five gfx1201
GPUs across two hosts and am happy to reproduce it — but I could not provoke it
with straightforward 4 KiB VMM use.

---

## 3. COMMENT on `ROCm/ROCm#6603` — two narrow pure-HIP probes did not reproduce corruption under the listed conditions

**Target:** `ROCm/ROCm#6603`. Rechecked 2026-08-23: OPEN, **9 comments**, opening
post **rewritten by the reporter on 2026-08-20**, still **zero AMD engagement**.
Note the umbrella repo has been renamed to `ROCm/legacy-rocm-build`; issue URLs
still resolve.

**Draft revised 2026-08-23.** An earlier revision of this comment argued from the
allocation-failure/retry path, because the original report tied corruption to the
first `num_alloc_retries`. **The reporter has since withdrawn that claim**: the
rewritten OP records failure at cycle 113–114 with `num_alloc_retries` still
**0**. Posting the old framing would have argued against a retracted claim. The
other withdrawn claims are `expandable_segments` alone being sufficient, a
75–80% rate, and 20 MiB of damage.

### Body

Thanks to @doplxyz for the rewritten writeup and the evidence repo, and to
@Only8Bits for the gfx1100 confirmation. The normalized matrix — same base image
and script, torch **2.12.0** held constant, 0/10 corrupt on ROCm 7.2 versus
10/10 on 7.14 — is the strongest constraint in the thread, because it isolates
the delta to ROCm userspace rather than the framework.

I tried to reproduce the silent zeros at the HIP level with PyTorch removed
entirely, and did not reproduce corruption under the listed conditions. Two
narrow pure-HIP probes did not reproduce corruption, which is worth recording so
nobody re-derives them — but this explicitly does not rule out the mechanism
under PyTorch's exact allocator behaviour, which may exercise different ordering,
concurrency, or sub-range patterns than the probes below.

**1. Expandable-segment churn alone, as modeled here, did not reproduce corruption.**

A pure-HIP model of an expandable segment: one reserved VA range, handles mapped
into it, a 2 MiB canary written **once** into an early chunk and never rewritten,
then repeated cycles of growth, strided unmap/release of *other* handles, and
remapping different handles at the same offsets (VA reuse). The canary is read
back every cycle. Churn is driven by `bench/vmm/vmm_expandable_canary.cpp`
(now committed in this repo). On gfx1201 / ROCm 7.14.0:

| Arm | Working set | Cycles | Canary |
| --- | --- | --- | --- |
| A | 1.98 GiB | 120 | intact |
| B | 19.07 GiB | 120 | intact |
| C | 30.83 GiB (96.9% of VRAM) | 150 | intact |

Arm C is the closest to the reported conditions — near-full VRAM, well past the
cycle count at which the canary dies upstream — and it stayed bit-exact. So
map/unmap/VA-reuse churn as modeled here, without PyTorch's allocator, is not
sufficient to reproduce the corruption. This does not rule out the mechanism
under PyTorch's exact allocation pattern.

**2. A rejected `hipMemSetAccess` showed no data side effect under the tested ranges.**

Because `max_split_size_mb:128` specifically is required to trigger this,
sub-granularity ranges looked like a promising lead: `hipMemSetAccess` rejects
any range whose offset or length is not a multiple of the mapped handles'
granularity. If a rejected call were *partially applied*, it could revoke access
on a correctly written region, which would read back as zeros rather than
faulting — matching the reported symptom, including zeros starting at offset 0.

Under the five tested rejected ranges, no data side effect was observed and a
later valid call succeeded: I wrote a pattern across four mapped handles,
verified it, then issued five different rejected calls (unaligned offset,
straddling, short length, half length, 4 KiB slice) and re-verified after each.
Every one returned `invalid argument` and left the data bit-exact, and a
following valid whole-range call still succeeded. This is the observation for
these five rejected ranges only, not a proof that the call is atomic in general.

**Where that leaves it.** Neither VMM churn as modeled here nor failed
access-setting for the five tested ranges reproduced corruption at the HIP
level, which is consistent with the reporter's own finding that the allocator
trace shows no unmap of the live canary range. Both are clean results under
specific conditions, not exclusions of the underlying mechanism.

### An offer that might narrow the version window

The 7.2-good / 7.14-bad boundary is wide. A tighter bisect is available but not
the way it might appear: TheRock's multi-arch indexes **do not publish any
7.2-series build** — the oldest retained artifact is `7.13.0a20260425`, and the
stable channel carries only `7.13.0`, `7.14.0` and `7.14.0.post1`. So 7.2 has to
come from classic OS packages, but **7.13.0 versus 7.14.0 can be A/B'd today**
from `repo.amd.com` wheels in two venvs, without disturbing a system ROCm.

If the canary is clean on 7.13.0 and dies on 7.14.0, the regression is confined
to that one release window; if it already dies on 7.13.0, the window moves
earlier and 7.2-vs-7.13 becomes the question. Either outcome is a real
narrowing, and I have not seen it done in the thread. I have gfx1201 (two
boards) and gfx1100/gfx1151/gfx1010/gfx1030 available on 7.14 and can run it.

## 4. COMMENT on `rocm-systems#10021` / PR `#10022` — reproduced on gfx1201

**Target:** issue `ROCm/rocm-systems#10021`, and/or its fix PR `#10022`. PR state
2026-08-23: OPEN, `REVIEW_REQUIRED`, merge `BLOCKED`, policy checks green, not a
draft. `@chrispaquot` asked an inline question on 2026-08-14 which the author
answered the same day; nothing has moved since. Posting confirmation from a
second architecture may help unstick it.

### Body

Confirming this on **gfx1201** (Radeon AI PRO R9700), stock ROCm **7.14.0**, so
the defect also reproduces on gfx1201 — the original report was on gfx1100.
That is consistent with the code: neither the unchecked `getGraphKernArg()`
result in the graph-capture branch of `submitKernelInternal` nor
`GraphKernelArgManager::AllocKernArg` returning `nullptr` on a failed pool grow
carries any architecture predicate. This shows the defect also reproduces on
gfx1201; it does not establish that the path is independent of architecture in
general.

A prior version of this probe was invalid: the exhaustion loop calls `hipMalloc`
until it fails, leaving a sticky `hipErrorOutOfMemory`, which the first
`hipGetLastError()` in the update loop misreported as an in-capture launch
failure. The current probe clears that sticky error and fail-fasts on every
HIP/hipGraph call with its enum printed, and adds a `--update-only` arm with no
launches between updates.

Reproducer is a single self-contained HIP file, no framework: capture a graph of
512 kernel nodes, instantiate and launch it once, exhaust device memory with
descending `hipMalloc` sizes, then drive `hipGraphExecUpdate` repeatedly so the
exec has to re-capture kernargs with nothing available. Every call before the
crash is verified successful with its enum printed.

Arm 1, launches between updates (verbatim):

```
=== AMD Radeon AI PRO R9700 (gfx1201) ===
free 31.79 GiB / total 31.86 GiB | graph nodes 512 | update rounds 8

baseline: graph of 512 nodes captured, instantiated, launched OK
exhausted: held 31.65 GiB in 36 blocks, 0.0 MiB reported free

round 1: hipGraphExecUpdate under exhaustion ... ok (result=0, bad_node=(nil))
         launch no error / sync no error (result=0, bad_node=(nil))
round 2: hipGraphExecUpdate under exhaustion ... ok (result=0, bad_node=(nil))
         launch no error / sync no error (result=0, bad_node=(nil))
round 3: hipGraphExecUpdate under exhaustion ... ok (result=0, bad_node=(nil))
         launch no error / sync no error (result=0, bad_node=(nil))
round 4: hipGraphExecUpdate under exhaustion ...
Segmentation fault (core dumped)
```

Every call before the crash is checked and reported: the reproducer aborts on any
non-success from begin-capture, the in-capture launch, end-capture, the update,
the post-update launch, the sync, or graph destroy. Nothing is accumulated and
ignored.

Deterministic: 3/3 with launches between updates, and 3/3 with `--update-only`
(no launches at all between updates) — 6/6 total, always round 4, exit 139.
The `--update-only` arm matters: with no launches between updates, a
poisoned-stream explanation is excluded.

The first three `hipGraphExecUpdate` calls returned success (`result=0`,
`bad_node=(nil)`) and the fourth crashed, which is consistent with kernarg-pool
growth but does not identify the exact grow attempt that faulted.

Arm 2, `--update-only` under `rocgdb`, showing the fault is entirely inside the
runtime (verbatim):

```
round 1: hipGraphExecUpdate under exhaustion ... ok (result=0, bad_node=(nil))
round 2: hipGraphExecUpdate under exhaustion ... ok (result=0, bad_node=(nil))
round 3: hipGraphExecUpdate under exhaustion ... ok (result=0, bad_node=(nil))
round 4: hipGraphExecUpdate under exhaustion ...

Thread 1 "geo" received signal SIGSEGV, Segmentation fault.
0x00007ffff6ab9f78 in ?? () from /opt/rocm/core/lib/libamdhip64.so.7
#0  0x00007ffff6ab9f78 in ?? () from /opt/rocm/core/lib/libamdhip64.so.7
#1  0x00007ffff6abafa8 in ?? () from /opt/rocm/core/lib/libamdhip64.so.7
#2  0x00007ffff664bfb3 in ?? () from /opt/rocm/core/lib/libamdhip64.so.7
#3  0x00007ffff6646a85 in ?? () from /opt/rocm/core/lib/libamdhip64.so.7
#4  0x00007ffff66bb6e0 in ?? () from /opt/rocm/core/lib/libamdhip64.so.7
#5  0x00007ffff695fdff in hipGraphExecUpdate () from /opt/rocm/core/lib/libamdhip64.so.7
#6  0x0000555555559e99 in main ()

rdi            0x0                 0
rsi            0x555555860130      93824995426608
```

The faulting frame is five stack levels below `hipGraphExecUpdate` (#0 vs #5),
with no application frame in between. `rdi` is `0x0` at the faulting
instruction, which is consistent with the null-kernarg path #10022 addresses —
`getGraphKernArg()` returning null and the result being used without a check —
though with a stripped release build I cannot name the exact callee.

### Why it is worth fixing rather than treating as an out-of-memory edge case

`hipGraphExecUpdate` is on the decode path of real inference stacks — llama.cpp
builds with `GGML_HIP_GRAPHS` on by default and calls the CUDA equivalent per
step — and running near the VRAM ceiling is the normal operating point for
long-context serving, not an unusual one. The observable difference between the
current behaviour and the fix is a returned error versus a process crash inside
the runtime, which an application cannot defend against.

Reproducer source: `bench/vmm/graph_execupdate_oom.cpp` in
https://github.com/warpfront/redline (exit 0 means the runtime handled
exhaustion, 3 means it reported an error, 139 means it reproduced).

---

## 5. REPLY for `ROCm/ROCm#6529` — rev 4, READY TO POST

**Target:** `ROCm/ROCm#6529` (now `ROCm/legacy-rocm-build`). State 2026-08-23:
OPEN, `status: triage`, assignee `@schung-amd`, **1 comment**, untouched since
2026-08-04. lhl's direct question in `#6409` has been unanswered for 26 days.

Rev 3 was blocked because it argued from "we cannot reproduce this," which was
false. Rev 4 leads with the correction instead of burying it.

---

Answering your direct question first, and correcting my own earlier silence: **yes,
I have seen this, on my own gfx1100, before you filed.** An RX 7900 XTX here
logged **78 address-zero page faults on 2026-07-23 between 13:37:49 and
16:44:44**, across **8 distinct PIDs**, escalating into **29 MES `REMOVE_QUEUE`
failures and 24 MODE1 full-device resets**. Every sampled fault carries your
exact tuple: address `0x0`, client `SQC (data) (0xa)`,
`GCVM_L2_PROTECTION_FAULT_STATUS:0x00801431`, `PERMISSION_FAULTS 0x3`,
`MAPPING_ERROR 0x0`. That is five days before you opened this issue. I did not
connect it at the time because nobody looked at the kernel log until later, and
I should have told you sooner.

I have not reproduced it deliberately since, and I have no recipe. What I do have
is two clean stress results under specific conditions, and one source answer to
your question 4.

**1. Routine mid-IB preemption did not reproduce it under these conditions.**
Retained-PM4 IBs of 24000 dispatches against 16 competing GPU processes, with
SH-register elision on and off. That genuinely oversubscribed the device — VRAM
28 MB to 3.89 GB, replay slowing from ~18 to ~550 us/token — so each IB spanned
~13 s against MES's 10 ms process quantum, on the order of 1300 quantum switches
inside a single indirect buffer and several thousand across the run. **8/8 arms
passed with exact results and zero new fault lines.** The kernel-source facts
that motivated this are still facts: `v11_compute_mqd` carries
`compute_pgm_lo/hi` and `compute_user_data_0..15`, `init_mqd` memsets and never
writes them, `update_mqd` never writes them, `hqd_load_v11` MMIO-loads only
`cp_mqd_base_addr_lo` through `CP_HQD_PQ_WPTR_HI`, and the CWSR trap handler
saves wave state only. But the inference I drew from them is empirically false
for the common path under these conditions: CP/MES evidently preserves or
correctly re-establishes that state here.

**2. HIP stream/event/allocation churn did not reproduce the fault under the
tested workload.** This is your leading hypothesis, so I built the smallest
thing that tests it with **no Redline and no retained PM4 in the process at
all** — the point being to find out whether this needs a retained-PM4 submitter
or not. Each cycle creates HIP streams, allocates device buffers, dispatches a
real kernel that *reads* device memory (the fault client is SQC data, so a
shader data fetch has to be in the loop), records and waits events, then
destroys the stream first, then the signal it referenced, then the memory the
packets pointed at, so the next cycle reuses those signal slots and device
addresses. Critically, `hipStreamCreateWithFlags` /
`hipStreamDestroy` are **not** hardware-queue create/destroy — ROCm 7.14 CLR
pools ordinary hardware queues, verified in source on `release/therock-7.14`:

- draws queues from `queuePool_`: `projects/clr/rocclr/device/rocm/rocdevice.cpp:3069-3153`
- reuses them once the configured limit is reached: `rocdevice.cpp:3223-3232`
- inserts newly created queues into the pool: `rocdevice.cpp:3395-3411`
- ordinary release only decrements a refcount; only CU-masked/cooperative queues are destroyed there: `rocdevice.cpp:3440-3468`
- pooled queues are destroyed at device/process teardown: `rocdevice.cpp:224-234`
- stream teardown calls `releaseQueue`, not `hsa_queue_destroy`: `projects/clr/rocclr/device/rocm/rocvirtual.cpp:2008-2074`
- `GPU_MAX_HW_QUEUES` defaults to 4: `projects/clr/rocclr/utils/flags.hpp:140`
So the run was 272,000 HIP stream create/destroy calls over at most 4 long-lived
hardware queues per process — not repeated hardware-queue creation and destruction
— and it does not test repeated KFD/ROCr queue retirement. What it does genuinely
exercise is repeated retirement and reuse of device allocations and completion
signals around live dispatches.

On the same card model as your second one, ROCm 7.14.0:

| Arm | Processes | Cycles | Dispatches | HIP stream create/destroy calls | Failures | Kernel faults |
| --- | --- | --- | --- | --- | --- | --- |
| single | 1 | 20000 | 80000 | 80000 | 0 | 0 |
| concurrent | 6 | 8000 each | 192000 | 192000 | 0 | 0 |

272,000 dispatches and 272,000 HIP stream create/destroy calls across a
one-process arm and a separate six-process arm (not "across 6 PIDs" for the
whole run), zero API failures, zero new kernel fault lines. This establishes a
clean result for this runtime-managed HIP workload only; it does not test
repeated KFD/ROCr queue retirement. Testing hardware-queue retirement directly
requires a ROCr-level arm that calls `hsa_queue_create` / `hsa_queue_destroy`
per cycle — a natural next probe from here.

**3. Your question 4, from source.** Successful `hsa_queue_destroy` does **not**
guarantee hardware can no longer reference completion signals, kernargs or
indirect IBs. `AqlQueue::~AqlQueue` waits for the error handler and frees what
the *queue object* owns, then `AqlQueue::Inactivate` does
`active_.exchange(false)` followed by `agent_->driver().DestroyQueue(queue_id_)`;
`KfdDriver::DestroyQueue` wraps `hsaKmtDestroyQueue`, which issues
`AMDKFD_IOC_DESTROY_QUEUE` and then does userspace bookkeeping in `free_queue`.
There is no completion-signal, kernarg or IB walk or drain anywhere on that path.
Additional software retirement is required before those addresses are safe to
free or reuse. Unread MES `REMOVE_QUEUE` may drain CP from the hardware's point
of view, but the userspace ABI does not document destroy as pointee retirement.

**One thing about your controls that may matter.** `rocm-systems#8113`, the
recycled-completion-signal fix you cite as a precedent, merged to `develop` on
2026-07-06 but was **not in stock ROCm 7.14.0**. The cherry-pick into
`release/therock-7.14` (`rocm-systems#10005`) only merged on **2026-08-13**. So
the four clean 240-row processes per card you ran on production 7.14 did not
include it, and neither did mine. If that fix is relevant, both of our 7.14
control sets were run without it.

### What I changed on my side, and what I did not

To be explicit, because it would be easy to misread: **none of this fixes the
address-zero fault.** The root cause is still unknown. What these do is remove a
different bug class from suspicion and convert a would-be device reset into a
named error:

- A **finalize-time PM4 stream validator** that rejects any indirect buffer which
  could dispatch from a zero program address, and refuses `DISPATCH_DIRECT`
  without a nonzero program address earlier in the stream. Wired into `finalize_ib`
  and `build_multi_ib`, so an integrator now gets `RL_ERR_COMPILE` with a reason
  instead of a fault. This is why I can say construction is validated rather than
  merely audited: both encoders already rejected `code_entry == 0` and null
  kernarg before any stream mutation, and the validator now enforces it on the
  finished stream too.
- **Encoder/device-family mismatch refused at IB construction**, and the two
  `hipengine_exact*` examples fail closed off gfx12 — they hardcoded gfx12
  command buffers and would previously have emitted them anywhere.
- **Ambiguous device selectors refused outright.** Worth flagging for your
  two-card host specifically: `GpuDevice::name()` is the HSA agent name, so both
  of your cards report `gfx1100` and any name-based selector is inherently
  ambiguous there. It now lists every candidate and refuses rather than binding
  the first match.
- A **host device manifest** with two refusal tiers, so a machine carrying a
  shared APU next to discrete boards can mark it as refused for reset-provoking
  runs while keeping it usable for normal work.

Reproducers and the full investigation record are public, if you want to run any
of it on your cards:

- Recreate/reuse stress (arm 2 above): `bench/vmm/queue_signal_reuse_stress.cpp`
- Preemption probe (arm 1 above): `scripts/6529-preemption-probe.sh`
- Contention A/B harness: `scripts/6529-contention-ab.sh`
- Investigation record, including the fault-log evidence:
  `docs/investigations/2026-07-31-rocm-6529-address-zero-sqc-fault.md`

All in https://github.com/warpfront/redline

### What I still cannot answer

Which backend faulted in my 2026-07-23 cluster. The faulting process was
`hip_dot_path`, whose Redline leg and stock-HIP leg build to the same binary
name, so the process name does not identify it. The surviving result directories
from inside that fault window list backends `hip` and `vulkan` with `redline`
absent, which is *suggestive* of a Redline leg that died without writing results
— but it is equally consistent with Redline never having been scheduled in that
run, and I am not going to present it as settled. If it was the stock-HIP leg,
this fires with no retained-PM4 replay in the process at all, and my arm-2
negative above becomes much more interesting.

---

## Still blocked, not drafted here

Nothing. The #6529 reply above supersedes the earlier "blocked" note: the two
facts that were missing when rev 3 was written (the queue-destroy contract and
#8113's absence from 7.14.0) are now in rev 4, and the backend question is stated
as an open unknown rather than gating the whole reply.
