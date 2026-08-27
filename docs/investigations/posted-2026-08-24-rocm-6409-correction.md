**Correction to my previous comment.** The ROCm 10.0 table in it was not a valid runtime comparison, and I'd rather flag that myself than leave it standing.

The mistake: `hipcc` emits no `RPATH`, and `/opt/rocm/core` is a symlink to one specific version. So both binaries — the one built with the 7.14 toolchain and the one built with 10.0 — resolved `libamdhip64.so.7` through `/opt/rocm/core` to the **same 7.14 runtime** at load time. That table compared two compilers against one runtime, while presenting itself as comparing two runtimes. The tell was results agreeing to three decimal places, which is not what two different runtimes look like.

I've redone it with the runtime switch verified from inside the process rather than assumed, using `dladdr` on a HIP entry point plus a sweep of `/proc/self/maps`:

```
LD_LIBRARY_PATH=/opt/rocm/core-7.14/lib
  hipRuntimeGetVersion : 71460850
  libamdhip64 in use   : /opt/rocm/core-7.14/lib/libamdhip64.so.7
  summary: 16 object(s) from core-7.14, 0 from core-10.0

LD_LIBRARY_PATH=/opt/rocm/core-10.0/lib
  hipRuntimeGetVersion : 71526333
  libamdhip64 in use   : /opt/rocm/core-10.0/lib/libamdhip64.so.7
  summary: 0 object(s) from core-7.14, 16 from core-10.0
```

All 16 ROCm objects move together with no cross-tree leakage, and the runtime reports a different version in each arm, so the two arms are genuinely different runtimes. The two trees are separate packages (`amdrocm-runtime7.14` and `amdrocm-runtime10.0`) with different build hashes, despite both exposing HSA runtime 1.21.

## Corrected measurement

Runtime genuinely switched, same host, same GPU, same binary, median of 200 replays after 20 warmups, N=512, µs per dispatch:

| GPU | arm | runtime 7.14 | runtime 10.0 |
| --- | --- | ---: | ---: |
| gfx1201 (R9700) | stream-loop | 2.559 | 2.545 |
| | per-launch-sync | 18.524 | 18.536 |
| | **graph-replay** | **2.144** | **2.146** |
| gfx1100 (7900 XTX) | stream-loop | 3.134 | 3.135 |
| | **graph-replay** | **2.815** | **2.819** |
| gfx1151 (Strix Halo) | stream-loop | 1.838 | 1.842 |
| | **graph-replay** | **1.752** | **1.750** |
| gfx1030 (RX 6950 XT) | stream-loop | 4.982 | 4.984 |
| | **graph-replay** | **4.523** | **4.525** |

**The conclusion is unchanged: the per-dispatch floor does not move between ROCm 7.14 and 10.0.** Differences are in the third decimal, i.e. run-to-run noise, in both directions. Every row printed `agree` on the two independent clocks and passed its correctness gate. gfx1030 is added here since it was not in the earlier table.

Two things this does *not* establish, to be explicit:

- The binary is compiled once, with the 10.0 toolchain, and run under both runtimes. That deliberately holds codegen constant to isolate the runtime, so it says nothing about whether the 10.0 *compiler* changes anything.
- The event-coalescing change in the 10.0 release notes is real; this measurement simply shows it does not move the kernel-dispatch floor, which is consistent with that note scoping it to event operations.

With the ROCm 7.2 figure quoted earlier, the per-dispatch floor is unchanged across **7.2 → 7.14 → 10.0**.

The identity check is a small standalone file and I'm happy to attach it alongside the reproducer — for anyone doing a side-by-side ROCm A/B it's worth running first, because the failure mode above is silent.
