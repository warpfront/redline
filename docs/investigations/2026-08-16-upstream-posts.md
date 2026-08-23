<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# Ready-to-post upstream drafts — 2026-08-16

Three drafts, each backed only by measurements taken on this fleet. Nothing here
is inference or extrapolation; every number below was produced by a probe in
this document. Post order is by value-per-effort: (1) is new and unreported,
(2) prevents a fleet-wide regression, (3) narrows an unengaged corruption bug.

Measurement environment for all three:

| Host | GPUs | ROCm | Kernel |
| --- | --- | --- | --- |
| local | 1x gfx1201 (RX 9070 XT) | 7.14.0 (`/opt/rocm/core-7.14`) | 7.0.0-28 |
| hiptrx | 4x gfx1201 (Radeon AI PRO R9700, 32 GiB) | 7.14.0 | 7.0.0-28 |
| hipx | gfx1100 (RX 7900 XTX), gfx1151 (Strix Halo), gfx1010 (RX 5700 XT), gfx1030 (RX 6950 XT) | 7.14.0 | 7.0.0-29 |

---

## 1. NEW ISSUE — HIP VMM capacity is silently capped by the process file-descriptor limit

**Target:** `ROCm/rocm-systems`, component `clr` (HIP VMM). No existing issue found.

**Title:** `[clr] hipMemCreate consumes a file descriptor per handle, capping HIP VMM at ~2 GiB under the default ulimit -n`

### Body

`hipMemCreate` appears to consume one file descriptor per allocation handle, so
the total amount of memory reachable through the HIP virtual-memory-management
API is bounded by `RLIMIT_NOFILE` rather than by available VRAM. With the
common default of `ulimit -n 1024`, a process can map roughly 2 GiB of VMM on a
32 GiB card.

**The failure is reported as `hipErrorOutOfMemory` while ~30 GiB of VRAM is
still free**, which is what makes this hard to diagnose: every signal points at
the GPU being full when the exhausted resource is actually descriptors.

Measured on a Radeon AI PRO R9700 (gfx1201, 32 GiB) under ROCm 7.14.0, with
`hipMemGetAllocationGranularity(..., Recommended)` reporting 2 MiB. VA is
pre-reserved for everything that could fit, so address space is never the limit:

| `ulimit -n` | Handles mapped | Mapped bytes | First failure | VRAM free at failure |
| --- | --- | --- | --- | --- |
| 1024 (default) | 1015 | **1.98 GiB** | `out of memory` | **29.80 GiB** |
| 65536 | 9765 | 19.07 GiB | — | — |
| 262144 | 15786 | **30.83 GiB (96.9%)** | — | — |

The ceiling tracks the FD limit almost exactly — 1015 handles against a 1024
descriptor budget, the remainder being the runtime's own descriptors — and
disappears entirely when the limit is raised. In the raised-limit runs the probe
goes on to consume the whole card (0.00 GiB free, address space exhausted rather
than allocation failing), confirming VRAM was never the binding constraint in
the first row.

### Why this matters

Consumers that build large allocations out of VMM handles hit this well before
they exhaust the device, and the error they see blames the wrong resource. It is
a plausible explanation for why llama.cpp ships `GGML_HIP_NO_VMM` and falls back
to a non-freeing legacy pool, and for long-context out-of-memory reports on
cards with plenty of VRAM free. Because the practical VMM budget is
`(descriptor limit x granularity)`, it also interacts with any change that
raises granularity: at 2 MiB and a 1024 FD limit the ceiling is ~2 GiB, and
forcing a larger granularity would not move it.

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

524288 handles x 2 MiB is ~1 TiB of VMM addressability, so this removes the
ceiling entirely rather than moving it. It should be logged (old and new soft
limit) so a constrained container that cannot raise it is diagnosable rather
than silently capped.

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
    if (hipMemCreate(&h, gran, &prop, 0) != hipSuccess) break;  // stops at ~1015
    hipMemMap((char*)base + i * gran, gran, 0, h, 0);
    hipMemSetAccess((char*)base + i * gran, gran, &desc, 1);
}
printf("mapped %zu handles = %.2f GiB\n", i, i * gran / 1073741824.0);
```

Run it once as-is and once under `ulimit -n 65536`.

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

`hipMemSetAccess` succeeds if and only if **both** the offset and the length are
multiples of the granularity of the mapped handles. This is uniform across all
five architectures:

| Sub-range against 2 MiB handles | Result |
| --- | --- |
| offset 0, len 2 MiB (one handle exactly) | OK |
| offset 2 MiB, len 4 MiB (two handles exactly) | OK |
| offset 0, len 8 MiB (all handles) | OK |
| offset 1 MiB, len 2 MiB (straddles, unaligned) | `invalid argument` |
| offset 1 MiB, len 1 MiB (inside one handle) | `invalid argument` |
| offset 0, len 1 MiB (aligned start, short) | `invalid argument` |
| offset 4 KiB, len 4 KiB (4 KiB slice of a 2 MiB handle) | `invalid argument` |

With 4 KiB handles, 4 KiB sub-ranges are accepted. So the constraint tracks the
handle granularity actually used, not a fixed device requirement.

### Consequence for this PR

Forcing `RECOMMENDED` semantics onto the `MINIMUM` query raises the floor for
every device. Callers doing fine-grained mapping lose capability that works
today, and each handle costs 2 MiB of physical memory minimum — which interacts
badly with the descriptor ceiling described in the issue above, since the
practical VMM budget is (FD limit x granularity). If the goal is to steer
callers toward 2 MiB, `hipMemAllocationGranularityRecommended` already reports
exactly that, and callers who want the larger value can ask for it.

If there is a specific gfx1201 failure behind this change, I have five gfx1201
GPUs across two hosts and am happy to reproduce it — but I could not provoke it
with straightforward 4 KiB VMM use.

---

## 3. COMMENT on `ROCm/ROCm#6603` — two HIP-level mechanisms ruled out

**Target:** `ROCm/ROCm#6603` (open, 8 comments, no AMD engagement as of 2026-08-16).

### Body

Thanks to @doplxyz for the minimal canary reproducer and @Only8Bits for the
gfx1100 confirmation — the fact that it needs **both** `max_split_size_mb` and
`expandable_segments:True`, and that ROCm 7.2 is clean while 7.14 is not, is the
most useful pair of constraints in the thread.

I tried to reproduce the silent zeros at the HIP level, with PyTorch removed
from the loop entirely, and could not. Two candidate mechanisms are now ruled
out, which I think is worth recording so nobody re-derives them.

**1. Plain expandable-segment churn does not corrupt a live mapping.**

A pure-HIP model of an expandable segment: one reserved VA range, handles mapped
into it, a canary written **once** into an early chunk and never rewritten, then
repeated cycles of growth, strided unmap/release of other handles, and remapping
different handles at the same offsets (VA reuse). The canary is read back every
cycle. On gfx1201 / ROCm 7.14.0:

| Arm | Working set | Cycles | Retries | Canary |
| --- | --- | --- | --- | --- |
| A (default `ulimit -n`) | 1.98 GiB (FD-capped) | 120 | 120 | intact |
| B (`ulimit -n 65536`) | 19.07 GiB | 120 | 0 | intact |
| C (`ulimit -n 262144`) | 30.83 GiB (96.9% of VRAM) | 150 | 0 | intact |

Arm A is the interesting one, because it forces the allocation-failure/retry
path on every cycle — which is where the report says corruption begins — and the
canary still survived 120 consecutive retries.

**2. A rejected `hipMemSetAccess` has no side effect on live data.**

Because `max_split_size_mb` is required to trigger this, sub-granularity ranges
seemed like a promising lead: `hipMemSetAccess` rejects any range whose offset or
length is not a multiple of the mapped handles' granularity (see table in the
`rocm-systems#9360` discussion). If a rejected call were partially applied, it
could revoke access on a region that had been written correctly, which would
read back as zeros rather than faulting — exactly the reported symptom.

It does not. I wrote a pattern across four mapped handles, verified it, then
issued five different rejected `hipMemSetAccess` calls (unaligned offset,
straddling, short length, half length, 4 KiB slice) and re-verified after each.
Every rejected call returned `invalid argument` and left the data bit-exact, and
a subsequent valid whole-range call still succeeded. The call behaves atomically.

**Where that leaves it.** The mechanism does not appear to be VMM map/unmap
churn or failed access-setting at the HIP level, which shifts suspicion toward
the caching allocator's own expandable-segment bookkeeping — consistent with the
observation that the allocator trace shows no unmap of the live canary range.

One note on a possibly-relevant confound: HIP VMM handle count is limited by the
process file-descriptor limit (~2 GiB of VMM at the default `ulimit -n 1024`;
see the separate `rocm-systems` issue). If the allocator's `num_alloc_retries`
are being driven by descriptor exhaustion rather than VRAM pressure, the retry
path may be entered far earlier than expected, and raising `ulimit -n` is worth
testing as a variable in the reproducer.

I have gfx1100/gfx1151/gfx1010/gfx1030/gfx1201 on ROCm 7.14 and can run further
arms, including a 7.2-vs-7.14 bisect using TheRock nightly wheels in a venv, if
that would help isolate the version boundary.

---

## Still blocked, not drafted here

`ROCm/ROCm#6529` rev 4 remains blocked on the same unknown as before: which
backend faulted in the 2026-07-23 gfx1100 cluster (redline leg vs stock HIP
leg). @schung-amd said on 2026-08-04 he was attempting a reproduction and there
has been no follow-up in the 12 days since. Two facts are now available that
were not when rev 3 was written and should go into rev 4:

- Successful `hsa_queue_destroy` does **not** retire packet pointees.
  `AqlQueue::Inactivate` -> `KfdDriver::DestroyQueue` -> `AMDKFD_IOC_DESTROY_QUEUE`
  performs no completion-signal, kernarg, or IB drain. This answers lhl's
  question 4 directly.
- `rocm-systems#8113` (recycled completion signal used for HIP hardware-queue
  idle/release decisions) merged to `develop` on 2026-07-06 but was **absent
  from stock ROCm 7.14.0**; the cherry-pick into `release/therock-7.14`
  (`rocm-systems#10005`) only merged on 2026-08-13. Any 7.14.0-based control run
  did not include it.
