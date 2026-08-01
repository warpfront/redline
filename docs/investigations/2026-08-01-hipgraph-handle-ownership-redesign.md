<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# redline-hipgraph — handle ownership redesign: native identity instead of our own pointer

Dated: 2026-08-01. **Implemented** — see "Outcome" at the end for what the plan
below actually produced, including where the estimates were wrong. Written for a
fresh-context agent; read "Decision" and "What blocks it" before touching
`crates/redline-hipgraph`.

The measurements in the planning sections were taken against `master` @ `4a8b21a`
and PR #5 @ `1d2a952`. They are preserved as written so the reasoning can be
audited; the Outcome section records where they turned out to be wrong.

## The one-line problem

`hipGraphCreate` hands the application a pointer to **our** heap object, not a
native `ihipGraph*`:

```rust
// lib.rs, allocate_graph()
let handle = Box::into_raw(Box::new(GraphHandle { state: Mutex::new(GraphState { … }) }))
    .cast::<c_void>();
lock(&global().handles).graphs.insert(handle as usize);
handle
```

The real graph lives *inside* as `native_graph: usize`. Same for
`NodeHandle::native_node` and `ExecState::native_exec`.

Every difficulty in this crate descends from that decision:

- Any hipGraph entry point we do not export receives our heap pointer and hands
  it to HIP, which dereferences it as its own type. **Silent memory corruption
  in an application that worked before the preload.** This is what PR #5's 72
  shims exist to prevent.
- The registry (`HandleSets`, lib.rs:156-162) is `HashSet<usize>` — bare
  addresses. `is_graph`/`graph_handle`/`exec_handle`/`node_snapshot`
  (lib.rs:493-515) test membership under a mutex, drop it, then dereference the
  raw pointer as a fabricated `&'static`. Nothing keeps the allocation alive
  between the check and the use, so a concurrent `hipGraphDestroy` frees it
  mid-call. The `&'static` is a lie.
- Address reuse gives ABA: destroy graph X, allocate graph Y at the same
  address, and a stale handle silently resolves to the wrong object.

The failure mode is inverted from what an interposer under someone else's
application should have. Today an entry point we have not thought of
**corrupts**. It should **lose acceleration**.

## Decision

Return the **real native handle** to the application. Keep the retained PM4 plan
in a side table keyed by that native pointer.

```
today:   app ──our GraphHandle*──> interposer ──native_graph inside──> HIP
         app ──our GraphHandle*───────────────(unshimmed call)───────> HIP   ← corruption

proposed: app ──native ihipGraph*─────────────────────────────────────> HIP
          app ──same pointer──> interposer ──side-table lookup──> our plan
```

Consequences:

- **Type confusion becomes structurally impossible.** The application never
  holds our pointer, so no unshimmed entry point can leak one into HIP.
- **Unknown entry points degrade to lost acceleration.** This is the whole
  argument. A future ROCm API we have never heard of works correctly and
  unaccelerated instead of corrupting memory.
- **The lifetime race dissolves.** A side-table miss is a clean miss, not a
  use-after-free. No fabricated `&'static`, and no `Arc` registry needed.
- **Roughly half the interposition deletes itself** (numbers below).

Cost is one hash lookup per intercepted call, against the cost of a HIP call.
Graph *construction* pays slightly more; replay — the hot path — is untouched.

## What blocks it, and why it is not actually blocked

**Stream capture is the one real counterexample.** `hipStreamBeginCapture` calls
`allocate_graph(0)` — our graph exists with **no native twin** — and capture-time
nodes are allocated with `native_node = 0`. Native identity is only attached at
`hipStreamEndCapture`. So there genuinely is per-handle state that predates the
native object, which kills a naive "always key by native pointer" scheme.

It resolves, and more cleanly than the status quo: **during capture the
application never holds the graph handle.** `BeginCapture` does not return one;
the graph first becomes visible at `EndCapture`, by which point the native graph
exists. So:

- Provisional capture state stays internal, keyed by **stream** — which
  `global().captures` already does.
- Native identity is published only at `EndCapture`, where
  `reconcile_captured_native_nodes` (lib.rs:~2175) *already* walks the native
  graph with `hipGraphGetNodes` and reconciles our node bookkeeping against it.
  The mechanism for treating the native graph as ground truth is in-tree today.

**The other zero-shadow source is a degenerate path, not a design feature.**
`native_graph == 0` outside capture happens only when
`real_symbol("hipGraphCreate")` returns `None` (`native_graph_create`,
lib.rs:822-834) — a runtime with no graph API at all — or when a native create
failed and `hipGraphCreate` swallowed it with `unwrap_or(0)`. Under the new
design the failure is returned to the caller, which is what HIP would have done;
and if the runtime has no graph API, we should not be interposing on graphs.

**Remaining bookkeeping churn:** `ExecState::nodes` / `ExecState::native_nodes`
and `NodeHandle::owner` currently key off our handle addresses. These become
native-keyed. Mechanical, but touch every site that builds them.

## Blast radius, measured

Against PR #5 (`hipgraph/translating-shims` @ `1d2a952`, 4017 lines under
`src/shims/`), bucketing all 72 shimmed entry points:

| bucket | count | fate |
|---|---:|---|
| **DELETE** — pure handle translator, no interposition needed | 23 | removed entirely |
| **KEEP** — mutates, must invalidate our plan (`force_native`) | 32 | shrinks to side-table lookup + flag |
| **KEEP-CREATE** — creates an object we must record | 17 | shrinks to side-table insert |

- ~620 shim function-body lines delete outright; the 49 survivors get materially
  thinner because none of them translate handles any more.
- All five translation helpers die (~104 lines): `native_node_or_passthrough`,
  `native_graph_or_passthrough`, `native_exec_or_passthrough`,
  `translate_dependencies_passthrough`, `intern_native_node`.
  `finish_native_only_node` survives as side-table insert bookkeeping.
- Version script drops **96 → 73** symbols (24 core + 49 KEEP/KEEP-CREATE).
- **`src/shims/introspect.rs` deletes entirely — all 7 entry points.** Three of
  the four `hipGraph*` symbols llama.cpp remaps through
  `ggml/src/ggml-cuda/vendors/hip.h` (`GetNodes`, `NodeGetType`,
  `KernelNodeGetParams`) would need no shim at all; only
  `KernelNodeSetParams` stays.

Undecided: `hipGraphRetainUserObject` / `hipGraphReleaseUserObject`. PR #5 marks
them mutating conservatively, but they do not change PM4 topology, so they may
be DELETE. Resolve during the redesign rather than guessing.

## The counter-risk, and its answer

With native identity, a future unshimmed API that **mutates** a graph would leave
our retained plan stale and we would replay something wrong. That is a real trade
against the corruption being removed.

The answer is only available *because* of the redesign: with the native graph
always present and authoritative, validate our modelled topology against
`hipGraphGetNodes` at **instantiate** time and force native on mismatch. Once per
instantiate, never per replay. The current design cannot do this cheaply because
our graph is the primary artifact rather than a shadow.

## Sequencing

1. **Merge PR #5 first.** It removes a *deterministic* corruption today, while
   the redesign is weeks out. Not wasted work: the expensive, error-prone part —
   the 72/72 signature audit against `hip_runtime_api.h` and the version-node
   mapping read from `readelf --dyn-syms` — carries over untouched, and 49 of
   the 72 shims survive in thinner form.
2. **Redesign as one PR** that deletes more than it adds: native identity,
   stream-capture provisional state keyed by stream, native-keyed side tables,
   instantiate-time topology validation, delete the 23 + the five helpers.
3. **Then** the fmt/clippy paydown, and promote those CI gates to required.

## Why not just make the current design safe

The tactical alternative is an `Arc` registry: `HashMap<usize, Arc<GraphHandle>>`,
clone the `Arc` under the lock, return a real reference instead of a fabricated
`&'static`. It genuinely closes the use-after-free and both residual races
(intern-vs-destroy, mint-vs-destroy), and call-site churn is small because
`Deref` absorbs most of it. Perhaps a day of work.

Rejected as the primary plan: it buys safety for a design whose central decision
is what generates the work, and it leaves every future ROCm entry point a
corruption risk. It is real effort spent making the wrong shape safe.

Keep it in the back pocket for one case only: if PR #5 must be sound *now* and
the redesign slips, the `Arc` change is the correct stopgap and does not conflict
with the eventual move.

## Testability — the part that makes this tractable

None of this needs a GPU. The side table is a pure host-side data structure, so a
multithreaded create/lookup/destroy stress test runs in the CI added in PR #4
(`cargo test --workspace`, no ROCm required). That converts "we reasoned
carefully about the lifetime" into "the machine checks it on every PR," which
this subsystem has never had.

## What this does not fix

An application that destroys a graph while another thread is using it remains
broken — but identically to how it would break without the preload. That is the
correct boundary: our objects get real lifetime, HIP's objects stay HIP's
contract.

## Provenance

- Handle-flow trace and the stream-capture finding: agent `ScopeHandleFlow`,
  2026-08-01, read-only over `master`.
- Bucketing and deletion counts: agent `ScopeDeletionSurface`, 2026-08-01,
  read-only over `hipgraph/translating-shims` @ `1d2a952`.
- The defects motivating this were found by three adversarial reviewers over two
  rounds on PR #5; see that PR's description for the full list, including the
  per-call-vs-per-argument translation flaw inherited from the in-tree
  `unsupported_pointer_node!` macro.

## Outcome

Implemented on `hipgraph/native-handle-identity`. What the plan got right, and
what it got wrong:

### Wrong: the bucketing was too conservative

The scoping pass bucketed `hipGraphClone`, `hipGraphChildGraphNodeGetGraph` and
`hipGraphNodeFindInClone` as KEEP-CREATE, and all thirteen `Add*` entry points as
KEEP-CREATE. Both were habits carried over from the wrapper design.

Under native identity, **absent from the registry already means "not modelled,
forward natively"** — so a graph or node we do not model needs no registration at
all, and the three clone/child entry points became DELETE. The `Add*` family adds
node kinds our PM4 plan cannot represent, so they only need to mark the plan
stale; none needs to register anything, which turned them from KEEP-CREATE into
the thinnest possible KEEP.

That left 44 of the 46 surviving shims as literally the same eight lines, so they
became **three macros plus invocation lists** rather than 44 hand-written
functions:

| macro | count | first argument |
|---|---:|---|
| `graph_mutating_shim!` | 13 | `hipGraph_t` |
| `node_mutating_shim!` | 16 | `hipGraphNode_t`, owner via `node_record` |
| `exec_mutating_shim!` | 15 | `hipGraphExec_t` |
| hand-written | 2 | `hipGraphInstantiateWith{Flags,Params}` |

### Right: the shape, the obstacle, and its resolution

Stream capture was the only real obstacle and it resolved exactly as predicted:
provisional state keyed by stream, adopted under the native pointer at
`hipStreamEndCapture`, with `reconcile_captured_native_nodes` supplying the
pairings. The application never holds a handle in between.

### Measured result

- Entry points: **72 → 46** under `src/shims/`; exported symbols **85 → 59**
  `hipGraph*` (70 total including module/launch/capture).
- Shim tree: **+638 / −3,782 = net −3,144 lines.** All five translation helpers
  deleted; `introspect.rs` deleted entirely.
- `cargo check --workspace --all-targets`: 0 errors, 0 warnings. Clippy on the
  crate: 67, unchanged from the 68 baseline — no lint debt added.
- Tests: **263 pass, 0 fail** (259 + 4 new registry tests).
- Every exported symbol verified to carry the same version tag as the shipped
  `libamdhip64` — 0 mismatches.

### Hardware A/B, gfx1201, `examples/hipgraph-demo/ab.sh`

Median of 7 runs at `GRAPH_M=1000`, all runs `CORRECT=true`:

| mode | stock | master `4a8b21a` | this branch |
|---|---:|---:|---:|
| explicit | 168.7 µs | 91.3 (1.85×) | **84.0 (2.01×)** |
| capture | 169.5 µs | 85.9 (1.97×) | **87.3 (1.94×)** |

No acceleration regression. Explicit is marginally faster, consistent with the
translation indirection being gone; capture is within run-to-run noise. A single
sample was NOT sufficient to conclude this — `STOCK_US` alone swings 172–228 µs
between runs, so early single-shot numbers showed a spurious 30% "regression".

### A latent assumption this surfaced

Putting the state in a `static` forced a `Send` check the old design never had:
`Pm4GraphReplay` reaches `NonNull<Queue>` and `Option<NonNull<u8>>`. The previous
code boxed the same state and kept only its address in a `HashSet<usize>`, so the
state was already shared across threads with no compiler check at all. The
redesign required writing the obligation down — see the `unsafe impl Send` block
and its safety argument in `lib.rs`. The obligation was always there.

### Still open

`reconcile_captured_native_nodes` pairs HIP's `hipGraphGetNodes` order
positionally with our `NodeId` creation order. This is **pre-existing** —
`master` does the same `zip` — and is guarded by an exact count match plus a
kernel-node-type check on every entry, but it is still an assumption rather than
a proof. A mismatch would associate the wrong native node with a modelled node
and mis-target a later parameter update. Capture produces a linear chain so the
orders coincide in practice. Fixing it properly means matching on kernel identity
rather than position, and is deliberately not in this change.

### Cross-architecture validation (added after merge)

The redesign was originally verified on gfx1201 only. Re-run across the full RDNA
lineup on hipx, master `4a8b21a` (fabricated handles + 72 shims) against the
redesign, median of 5 runs at `GRAPH_M=1000`, `CORRECT=true` in every run:

| arch | gen | mode | stock | master | redesign | delta |
|---|---|---|---:|---:|---:|---:|
| gfx1010 | RDNA1 | explicit | 283.6 µs | 235.4 (1.20×) | 235.3 (1.21×) | −0.0% |
| gfx1010 | RDNA1 | capture  | 284.4 µs | 235.2 (1.21×) | 232.7 (1.22×) | −1.1% |
| gfx1030 | RDNA2 | explicit | 287.1 µs | 210.5 (1.36×) | 210.5 (1.36×) | +0.0% |
| gfx1030 | RDNA2 | capture  | 288.2 µs | 210.0 (1.37×) | 210.4 (1.37×) | +0.2% |
| gfx1100 | RDNA3 | explicit | 198.8 µs | 98.4 (2.02×) | 98.6 (2.02×) | +0.2% |
| gfx1100 | RDNA3 | capture  | 198.9 µs | 98.3 (2.02×) | 97.4 (2.04×) | −0.9% |
| gfx1201 | RDNA4 | explicit | 168.7 µs | 91.3 (1.85×) | 84.0 (2.01×) | −8.0% |
| gfx1201 | RDNA4 | capture  | 169.5 µs | 85.9 (1.97×) | 87.3 (1.94×) | +1.6% |

Every RDNA generation from 1 through 4 is correct and free of regression; the
worst case is +1.6% and within run-to-run noise. The gfx1151 Strix Halo APU on
the same host is deny-listed and was not exercised.

Note the speedup is strongly architecture-dependent — 1.2× on RDNA1 versus 2.0×
on RDNA3 — which is a property of the workload and the baseline runtime, not of
this change: master shows the same spread.

#### Operational note: `ROCR_VISIBLE_DEVICES` and `HIP_VISIBLE_DEVICES` do not compose

The first attempt at this matrix produced no output on two of three cards. The
cause was setting BOTH variables to the same index: `ROCR_VISIBLE_DEVICES=2`
filters ROCr down to a single agent, after which `HIP_VISIBLE_DEVICES=2` selects
index 2 of a one-device list and finds nothing. It appeared to work on the first
card only because `0` is consistent under either interpretation.

Set `ROCR_VISIBLE_DEVICES` to pick the physical device and leave
`HIP_VISIBLE_DEVICES=0`, since HIP enumerates through ROCr and inherits the
filter. This is the same index-drift hazard the device resolver exists to remove,
encountered while validating a different change.

On this host the ROCr GPU-agent order and the HIP device order happen to agree
(`bdfid` 26112/48896/28160/39168 = buses `0x66`/`0xBF`/`0x6E`/`0x99`), but
rocm-smi's order does NOT: its `GPU[1]` is the RX 5700 XT that HIP calls
`hip[2]`. Never anchor on a rocm-smi index.
