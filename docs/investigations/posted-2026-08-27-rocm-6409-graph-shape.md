Two things in one comment: a **new finding about ROCm 10.0's graph execution**, and a **correction to the graph-shape claim in my first comment**, which the new measurements contradict.

## ROCm 10.0 ships a different graph-exec implementation

`GraphExecSegmented` and `GraphExecClassic` are present in ROCm 10.0's `libamdhip64` and in neither form in 7.14's, and `hip_graph_internal.cpp` grows 2773 → 3348 lines on `release/therock-10.0`, adding a segmentation pass — `ScheduleNodesIntoBatches`, segments derived from execution paths, dependency levels, and a `max_streams_` bound.

That change is invisible to the per-dispatch floor I posted earlier, because that probe only measures a serial chain. It is very visible once the dependency structure varies.

Method: nodes added with `hipGraphAddKernelNode` so the edges are exactly as stated; same kernel and node count in every arm; only the DAG differs. **Matched pairs throughout** — each release is built *and* run with its own toolchain, with the loaded runtime verified per arm from inside the process (16/16 mapped ROCm objects from the intended tree; `hipRuntimeGetVersion` 71460850 vs 71526333). µs per dispatch, median.

N sweep, gfx1100, 60 replays:

| N | chain 7.14 | chain 10.0 | indep 7.14 | indep 10.0 | fanout 7.14 | fanout 10.0 |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 8 | 9.627 | 9.579 | 10.263 | **8.427** | 11.019 | **6.081** |
| 64 | 3.245 | 3.229 | **1.893** | 3.131 | **2.199** | 3.149 |
| 256 | 2.854 | 2.853 | **1.117** | 2.848 | **1.638** | 2.853 |
| 512 | 2.800 | 2.791 | **1.034** | 2.784 | **1.549** | 2.782 |

Three architectures at N=512:

| GPU | ROCm | chain | indep | fanout |
| --- | --- | ---: | ---: | ---: |
| gfx1100 (RX 7900 XTX) | 7.14 | 2.800 | 1.034 | 1.549 |
| | 10.0 | 2.791 | 2.784 | 2.782 |
| gfx1151 (Radeon 8060S) | 7.14 | 1.741 | 0.817 | 0.921 |
| | 10.0 | 1.742 | 1.744 | 1.744 |
| gfx1030 (RX 6950 XT) | 7.14 | 4.458 | 1.870 | 3.147 |
| | 10.0 | 4.457 | 4.451 | 4.451 |

**Serial chains are unchanged** between releases, to three digits, on every architecture and every N — consistent with the floor result already posted. What changes is concurrent shapes: on 7.14, declaring nodes independent is 2.1×–2.7× *cheaper* than a chain; on 10.0, independent and fanout converge onto the chain cost within 0.3%.

## The important nuance: 7.14's fast path is fragile

Before calling that a regression, I swept `GPU_MAX_HW_QUEUES` (gfx1100, N=512):

| ROCm | queues | chain | indep | fanout |
| --- | ---: | ---: | ---: | ---: |
| 7.14 | 4 (default) | 2.800 | **1.039** | 1.564 |
| 7.14 | 8 | 2.799 | **6.789** | 1.530 |
| 7.14 | 16 | 2.791 | **6.856** | **8.847** |
| 10.0 | 4 | 2.791 | 2.786 | 2.783 |
| 10.0 | 8 | 2.792 | 2.786 | 2.784 |
| 10.0 | 16 | 2.791 | 2.786 | 2.783 |

Reproduced, 3 runs each: Q=4 gives independent 1.016 / 1.026 / 1.038; Q=8 gives 6.864 / 6.877 / 6.678. Chain stays 2.790–2.803 throughout, so the effect is specific to concurrent shapes.

So 7.14's 2.7× advantage exists **only at the default queue count**, and raising that count costs 6.6× on the same graph. 10.0 is insensitive to the knob at 4, 8 and 16.

Read fairly, **10.0 traded peak for predictability**: it gave up 7.14's best case (~2.7×) and also 7.14's misconfigured worst case (~2.4×), and it is genuinely better for small graphs (N=8 fanout 11.019 → 6.081). That is a defensible choice, and it is a more accurate description than "10.0 is 2.7× slower". If the convergence onto chain cost is *intended*, this comment is just a datapoint. If it isn't, the N≥64 concurrent numbers are worth a look.

## Correction to my first comment

That comment reported, from a single gfx1201 measurement, that "declaring nodes **independent** is ~3× *more* expensive per dispatch than a strict serial chain" (chain 2.163, independent 6.432, fanout 7.055), and generalised it to "a decode step expressed as a parallel or fanout graph pays roughly 3× the submission cost of the same work expressed as a chain."

**That generalisation was wrong.** Three architectures on 7.14 at default settings show the opposite: independent is 2.1×–2.7× *cheaper*. And the specific figure 6.432 closely resembles 7.14's `GPU_MAX_HW_QUEUES>=8` behaviour (6.79–6.86) rather than its default-queue behaviour, so that measurement may have been taken under a queue count I had not controlled for.

I am therefore withdrawing the gfx1201 shape claim pending re-measurement with matched pairs and an explicit queue count. The host carrying those GPUs is offline as I write this; I will post the gfx1201 numbers when it is back rather than leave the earlier figure standing as if it were general.

Nothing in this correction affects the per-dispatch floor results — those are serial-chain measurements, they were matched-pair verified, and they are unchanged across 7.2 → 7.14 → 10.0.

The graph-shape probe is a single standalone file like the others and I can attach it, along with the runtime-identity checker I now run before any side-by-side ROCm A/B — the failure mode where both arms silently load the same runtime is easy to hit and produces results that look suspiciously clean.
