<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# gfx1100 gfxhub fault: hipfire retained-PM4 x VMM KV growth (2026-09-01)

**Status: internal. Live incident, hypothesis stage.**

## Event

hipx, boot of 05:03 UTC: serving Ornith-1.5-35B-A3B (mq4r, --kv q8
--kv-backend vmm) via hipfire's redline retained-PM4 path on the gfx1100
(Navi 31, 0000:66:00.0). At 05:17: sq_intr type 2 errors on multiple waves,
then a [gfxhub] page fault storm, vmid:8 pasid:17, two representative
addresses 0x76db1faeb000 and 0x76cb244ee000 (user-VA range, ~64 GiB apart,
consistent with separate K/V VMM arenas), Faulty UTCL2 client unknown(0x1ff),
MORE_FAULTS=1, WALKER_ERROR=0x7, PERMISSION_FAULTS=0xf, MAPPING_ERROR=0x1.
MES declared unrecoverable; GPU reset returned -19 repeatedly; host required
reboot (05:19). Kernel 7.0.0-30, ROCm 10.0.

## The discriminator (user-confirmed)

The identical workload — same model, same VMM q8 KV, same box, same runtime —
**completes cleanly on the stock HIP path and faults on the PM4 path.** The
dispatch path is the only variable.

## Leading hypothesis

`hipfire/crates/hip-bridge/src/vmm.rs` `map_next` reapplies `hipMemSetAccess`
over the ENTIRE live mapped prefix on every KV growth (vmm.rs:251-257) — a
workaround written for the ROCm 7.2 subrange rejection, executed
unconditionally on all runtimes. That rebuilds GPU PTEs for the whole arena
while work is in flight. The HIP runtime can protect its own streams around
access changes; **redline replays on a foreign HSA queue the runtime does not
know about**, so replay reads KV VAs mid-rebuild -> not-present PTE walks ->
MAPPING_ERROR storm. VMM VAs are growth-stable by design, which excludes the
stale-address-in-IB alternative; what changes at grow time is precisely the
PTE state this workaround touches.

Fits every observable: fault addresses inside arena-range VAs; MAPPING (not
permission) faults; PM4-only; storm rather than single fault (whole prefix);
timing during generation (KV growth).

## Alternatives not yet excluded

- Cross-agent capture/replay mismatch: hipx gained a fourth GPU (gfx1010,
  99:00.0) the same morning, shifting device enumeration; hipfire's tree has
  active exact-gfx1010 PM4 work including a Navi10 agent-name matcher.
- Host memory distress as co-factor (sshd kex resets under the serve load).

## Discriminating A/Bs (avoidance arms; the faulting control already exists)

1. PM4 arm with KV growth eliminated (pre-map the full arena before serving).
   Survives -> growth events are necessary for the fault.
2. PM4 arm with segment-only SetAccess (new handle only, no prefix reapply).
   Survives -> the prefix reapply is the mechanism. Precondition: verify the
   whole-handle SetAccess rule on ROCm 10.0 (measured to date on 7.14;
   vmm_setaccess_rule.cpp staged for the 10.0 run).
3. If both fault: revisit cross-agent enumeration.

## Fix shape (pending confirmation)

Gate the prefix reapply to 7.2-class runtimes and genuine peer-device-set
changes; SetAccess only the new segment elsewhere; drain/quiesce redline's
queue around any remaining whole-prefix reapply, since that queue is outside
the HIP runtime's protection.

## Note for #6529

Second gfx1100 incident with the sq_intr -> vmid:8 page-fault signature from
a redline-lineage PM4 path (July campaign binaries were the first, at address
zero). Today's is firmly ours. Hold further #6529 upstream motion until our
own stack is exonerated or convicted.
