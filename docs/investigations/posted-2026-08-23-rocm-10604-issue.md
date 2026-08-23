### Problem Description

`hipMemCreate` consumes one dmabuf file descriptor per physical allocation handle. With the runtime's own recommended granularity of 2 MiB and the stock soft `RLIMIT_NOFILE` of 1024, that caps a process at **1015 handles = 1.98 GiB of VMM**, on every RDNA generation I have, independent of how much VRAM the device has. The call fails with `hipErrorOutOfMemory`, which points at the wrong resource — there are tens of GiB of VRAM free at the point of failure.

Measured on four architectures, ROCm 7.14.0, stock `ulimit -n 1024`:

| GPU | arch | VRAM free | handles created | VMM reached | VRAM still free at stop |
| --- | --- | --- | --- | --- | --- |
| RX 6950 XT | gfx1030 | 15.96 GiB | 1015 | 1.98 GiB | 13.97 GiB |
| RX 7900 XTX | gfx1100 | 23.95 GiB | 1015 | 1.98 GiB | 21.96 GiB |
| Radeon 8060S | gfx1151 | 95.84 GiB | 1015 | 1.98 GiB | **93.86 GiB** |
| Radeon AI PRO R9700 | gfx1201 | 31.79 GiB | 1015 | 1.98 GiB | 29.80 GiB |

The handle count is identical across a 6× spread in VRAM. On the 96 GiB part a process reaches **2.1%** of the device before `hipMemCreate` starts failing.

Attribution is measured rather than inferred: at the point of failure `opendir("/proc/self/fd")` itself fails with `EMFILE`, so descriptor exhaustion is directly observed, and the probe reports which call stopped it:

```
AMD Radeon AI PRO R9700 (gfx1201)
  VRAM free 31.79 GiB / total 31.86 GiB, granularity 2097152 B
  handles created 1015 = 1.98 GiB
  stopping stage: hipMemCreate
  stopping error: out of memory (2)
  VRAM still free at stop: 29.80 GiB
  FDs at stop: open (unknown, opendir failed: Too many open files) / soft 1024
  => FAILED WITH VRAM TO SPARE — descriptor-bound, not memory-bound
```

The descriptors are dmabufs — `readlink` on each new fd shows a `/dmabuf:` target — and `hipMalloc` consumes none. They are returned on `hipMemRelease`, so this is a live cost rather than a leak.

**A caller cannot opt out through the public API.** `hipMemAllocationProp::requestedHandleTypes` declares whether the caller intends to export a handle for sharing, so a caller that never shares has no apparent need for an OS-visible descriptor. It makes no difference:

```
=== AMD Radeon AI PRO R9700 (gfx1201) === 64 handles per arm, 2 MiB each
requestedHandleTypes=None     created  64 | fd delta  +64 | dmabuf delta  +64 | no error
requestedHandleTypes=PosixFd  created  64 | fd delta  +64 | dmabuf delta  +64 | no error
```

Identical on gfx1030, gfx1100 and gfx1151.

### Where it appears to come from, and the actual question

In `projects/rocr-runtime/runtime/hsa-runtime/core/runtime/runtime.cpp` (`release/therock-7.14`), `Runtime::VMemoryHandleCreate` allocates and then unconditionally creates a shareable handle:

```cpp
hsa_status_t status = region->Allocate(size, alloc_flags, &mem, 0);
if (status == HSA_STATUS_SUCCESS) {
  // TODO: Combine the Allocate and CreateShareableHandle into a single function.
  auto ret = agentOwner->driver().CreateShareableHandle(nullptr, mem, size, *agentOwner,
                                                       &driver_handle, &offset);
```

There is no handle-type gate, and the parameter that could carry the caller's declared intent down is named `flags_unused`.

What makes me raise it as a question rather than a bug report is that `KfdDriver::ExportMemoryHandle` already has an on-demand path — when `handle.dmabuf_fd == -1` it exports from `handle.handle` at export time — which suggests the eager creation may not be needed to satisfy a later export. But `driver_handle` is also consumed by the mapping path (`Runtime::VMemoryHandleMap` → `driver().Map(driver_handle, ...)`), and I could not find KFD's `CreateShareableHandle` implementation to tell whether the descriptor is structurally required there.

So: **is the descriptor per handle intended, or could it be deferred to first export?** If mapping genuinely needs it, the ceiling is structural and worth documenting. If it could be lazy, the ceiling largely disappears for the common case of a process that never shares memory.

### Two mitigations that work today, for anyone hitting this

1. **Larger physical handles, no privileges needed.** A handle costs one descriptor regardless of size, so capacity is descriptors × handle size. On gfx1201 at the stock limit: 2 MiB handles → 1.98 GiB, 8 MiB → 7.93 GiB (exactly 4.0×). Beyond that, physical memory legitimately binds first.
2. **Raise the soft limit in-process.** `RLIMIT_NOFILE` hard is 524288 here, and soft→hard needs no privileges:

```
RLIMIT_NOFILE before: soft=1024 hard=524288
setrlimit(RLIMIT_NOFILE, soft=hard=524288) succeeded
RLIMIT_NOFILE after: soft=524288 hard=524288
  handles created 16275 = 31.79 GiB
  stopping stage: completed-requested-count
  VRAM still free at stop: 0.00 GiB
  FDs at stop: open 16285 / soft 524288 (headroom 508003)
```

That reaches all free VRAM with descriptors to spare. To be precise about what it does and does not show: the reservation was sized to initial free memory, so completing it proves the preselected span was filled, not that any address-space limit was reached.

### Why this may matter beyond the raw number

While reviewing #9360 — which raises the reported granularity to 2 MiB for all device allocations, motivated by VMM being unusable on RDNA4 for llama.cpp and vLLM — I noticed this ceiling produces that symptom directly. A consumer that queries `hipMemAllocationGranularityMinimum` (4 KiB on RDNA) and builds a pool from minimum-granularity handles runs out of descriptors at roughly **4 MiB** of VMM:

```
building a contiguous span from 1024 handles of 4096 B each = 4096 KiB
FATAL hipMemCreate(&h, gmin, &prop, 0) -> out of memory (2)
```

That would present as "VMM-based memory management is blocked", and raising the reported granularity works around it by making each descriptor cover 512× more memory. I could not reproduce the kernel-fault mechanism #9360 describes (details in that PR), so it seems worth checking whether descriptor exhaustion is the mechanism actually being worked around there. If it is, fixing the descriptor cost would address it without giving up fine-grained mapping.

For context on the consumer side, llama.cpp currently defaults `GGML_HIP_NO_VMM` to `ON` while `GGML_CUDA_NO_VMM` defaults to `OFF`, i.e. HIP VMM is shipped disabled.

### Operating System

Ubuntu 26.04 LTS

### CPU

AMD Ryzen Threadripper 9970X (gfx1201 host), AMD Ryzen AI MAX+ 395 (gfx1151/gfx1100/gfx1030 host)

### GPU

AMD Radeon AI PRO R9700 (gfx1201), AMD Radeon RX 7900 XTX (gfx1100), AMD Radeon 8060S (gfx1151), AMD Radeon RX 6950 XT (gfx1030)

### ROCm Version

ROCm 7.14.0 (`hipconfig --version` 7.14.60850-0000000)

### ROCm Component

clr (HIP VMM) / rocr-runtime

### Steps to Reproduce

Standalone HIP, no framework:

1. `hipMemGetAllocationGranularity(&g, &prop, hipMemAllocationGranularityRecommended)` → 2 MiB on all four parts.
2. `hipMemAddressReserve` a span sized to initial free VRAM.
3. Loop `hipMemCreate(&h, g, &prop, 0)` with `hipMemAllocationProp prop{}` zero-initialised, so `requestedHandleTypes` is `hipMemHandleTypeNone`.
4. Count open descriptors via `/proc/self/fd` and `readlink` each target before and after.

Observed: descriptor count rises by exactly one per successful handle, all with `/dmabuf:` targets; `hipMemCreate` returns `hipErrorOutOfMemory` at 1015 handles with `EMFILE` observable on `opendir`, while `hipMemGetInfo` still reports most of the device free.

Control: the same loop with `hipMalloc` adds no descriptors.

I can attach the four probes used here (ceiling with stopping-stage attribution, chunk-size sweep, `requestedHandleTypes` A/B, and the minimum-granularity kernel-access test), and I'm happy to test a candidate patch on all four architectures.

### Additional Information

Fleet is ROCm 7.14.0 on Ubuntu 26.04, `RLIMIT_NOFILE` soft 1024 / hard 524288. Related: #9360 (granularity override, same underlying symptom in my reading); #8517 is a different dmabuf bug — an application explicitly exporting and corrupting ROCr state, already root-caused there.
