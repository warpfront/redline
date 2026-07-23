# hipengine-714 — clean drop-in A/B on the real hipEngine bench suite

A clean, honest test: run the **pristine** hipEngine dispatch microbench
(`.engines/hipEngine/scripts/graph_node_microbench.py`, stock `hipGraph`
capture → `hipGraphLaunch`) as the control, then run the *identical* binary
with `LD_PRELOAD=libredline_hipgraph.so` — the redline-hipgraph drop-in — and
measure the delta.

**No configuration. No hand-lowering.** The only difference between the two runs
is the preloaded `.so`, which transparently intercepts the app's `hipStream*`
capture + `hipGraph*` calls (and `__hipRegisterFatBinary/Function` for the
statically-compiled kernels) and replays the captured graph as a retained GFX12
PM4 indirect buffer.

## Run
```
bash run_ab.sh                 # counts 1,50,200,941 ; 40 reps
bash run_ab.sh 1,50,200,941 60 # custom counts / reps
```
Requires: ROCm 7.14 (TheRock `/opt/rocm/core`), a gfx1201 GPU, and the pristine
engine at `../../.engines/hipEngine`. The default (Python-free) redline-hipgraph
build is used so `LD_PRELOAD` is safe for the `hipcc` subprocess the harness
spawns.

## Result (gfx1201 / RDNA4, serial_latency, host clock, correctness gated)
Representative run (host timing has run-to-run variance at these scales):

| count | stock hipGraph | redline drop-in | speedup |
|---|---|---|---|
| 1 | ~33 us | ~30-37 us | ~0.9-1.2x (single node: no batching win) |
| 50 | ~191 us | ~90-142 us | 1.4x - 2.1x |
| 200 | ~645 us | ~255-493 us | 1.3x - 2.5x |
| 941 | ~2640 us | ~1062-2224 us | 1.2x - 2.5x |

Correctness passes on every row (device-side output validated by the microbench
against its CPU reference; the drop-in retains a native-HIP shadow graph and
falls back transparently for anything it cannot lower).

This is `serial_latency` — a true dependency chain, so each `hipGraphLaunch` also
waits for on-GPU execution and the pure host-submission advantage is diluted; the
win is still 1.2-2.5x. Independent/wide graphs (host-submission-bound) show far
larger gains — see `examples/hipgraph-demo` (up to ~100x).

## How to see it engage (diagnostics)
```
REDLINE_HG_DEBUG=1 LD_PRELOAD=<so> python3 <engine>/scripts/graph_node_microbench.py --counts 50 ... 2>&1 | grep redline-hg
```
Expected: `bundle ... selected=gfx1201`, `hipStreamBeginCapture ... real_symbol=handle native_shadow=true`,
`append_capture_kernel ... function_record=resolved pm4_node=appended`,
`hipGraphInstantiate build_pm4_replay=ok`, `hipGraphLaunch branch=pm4 replay`.

## Why it works (what had to be solved for a real Python HIP app)
1. **Symbol versioning** — apps reference versioned HIP symbols (`hipGraphLaunch@hip_4.3`);
   the interposer exports matching `@@hip_4.x` so LD_PRELOAD actually preempts libamdhip64.
2. **Static kernels** — `__hipRegisterFatBinary/Function` interception + `__CLANG_OFFLOAD_BUNDLE__`
   extraction resolves `<<<>>>`/`hipLaunchKernelGGL` kernels to code objects.
3. **RTLD_LOCAL** — hipEngine loads libamdhip64 via `ctypes.CDLL` (RTLD_LOCAL), so real-HIP
   resolution uses an explicit `dlopen(libamdhip64, RTLD_NOLOAD)` handle, not `RTLD_NEXT`.
4. **Native shadow** — real capture is always driven so a PM4-build failure falls back to
   native HIP instead of erroring.
