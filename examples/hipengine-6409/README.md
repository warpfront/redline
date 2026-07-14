# HipEngine #6409 integration

This directory runs the pinned HipEngine micro suite with HIP, Vulkan, and the
combined Radiowave + Redline stack without modifying the HipEngine checkout.

The `toolchain/hipcc` adapter compiles the original HIP source through
Radiowave and emits a hash-bound inspection manifest. Synthetic C++ runners use
`redline_timing_override.hpp`; production-shaped Python runners use
`redline_hip_timing.py`. Both paths capture launch metadata once, load only a
Radiowave-certified module through the C ABI, build a stateful retained PM4 IB,
and select serialized RMW boundaries from the verified next consumer.

The current gfx1201 result beats Vulkan in **192/212 matched kernel rows
(90.57%)**, up from 186/212 before the complete stack was lowered into the
integration. See the [full report](results/gfx1201/2026-07-13-radiowave-redline/REPORT.md)
and [machine-readable summary](results/gfx1201/2026-07-13-radiowave-redline/summary.json).

The adapters are deliberately fail-closed. A stale/missing manifest, absent
kernel inspection, unknown consumer, or scalar mutable read cannot select the
narrow VMEM boundary. HipEngine's invalid independent HIP sampler is retained
as a rejected artifact rather than converted into a timing row.

For production-slice clock preconditioning, `redline_hip_timing.py` accepts
`REDLINE_PREHEAT_REPLAYS=N`. It replays each already-recorded measured IB `N`
times outside the returned GPU sample set, so a large GPU preheat does not also
multiply HipEngine's expensive deterministic CPU-oracle fixtures. The
normalizer records the count and scope in `redline_provenance`. This opt-in is
appropriate for the overwrite-style production rows; RMW benchmarks must
reset their mutable state explicitly instead.

The first preheated production control passes 56/56 correctness gates and is
retained in
[`results/gfx1201/2026-07-13-hipengine-production-preheated/matrix.json`](results/gfx1201/2026-07-13-hipengine-production-preheated/matrix.json).
