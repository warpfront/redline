# hipgraph-demo — redline-hipgraph drop-in A/B

Exercises `redline-hipgraph`, a drop-in `hipGraph` replacement backed by redline's
retained-PM4 replay engine (RDNA1-4). Compares stock HIP `hipGraph` against the
redline drop-in via `LD_PRELOAD`.

## Build
```
bash build.sh          # graph_demo (statically-compiled kernels)
hipcc --offload-arch=gfx1201 -O2 graph_demo_mod.cpp -o graph_demo_mod   # module-loaded kernels
```
Requires ctr.hsaco for the module demo:
```
hipcc --genco --offload-arch=gfx1201 -o /tmp/ctr.hsaco - <<< '
#include <hip/hip_runtime.h>
extern "C" __global__ void ctr_k(unsigned int* c){ atomicAdd(c,1u); }'
```

## Run
```
# stock hipGraph
ROCR_VISIBLE_DEVICES=0 HIP_VISIBLE_DEVICES=0 ./graph_demo_mod
# redline drop-in
LD_PRELOAD=../../target/release/libredline_hipgraph.so ROCR_VISIBLE_DEVICES=0 HIP_VISIBLE_DEVICES=0 ./graph_demo_mod
```
Env: `GRAPH_N` nodes, `GRAPH_M` timed launches, `GRAPH_TOPO=chain|independent`,
`GRAPH_HSACO`/`GRAPH_SYM` (module demo), `GRAPH_MODE=explicit|capture` (graph_demo).

## Measured (gfx1201 / RDNA4, module-loaded kernels, correctness-gated)
| topology | N | stock hipGraph | redline | speedup |
|---|---|---|---|---|
| chain | 64 | 223 us | 90 us | 2.48x |
| chain | 256 | 612 us | 347 us | 1.76x |
| independent | 64 | 548 us | 27 us | 20.4x |
| independent | 256 | 2024 us | 31 us | 66x |
| independent | 1024 | 7907 us | 68 us | 117x |

One retained PM4 IB + one doorbell stays ~flat as node count grows, while stock
hipGraph pays O(N) per launch. Chains are fence-bound (both runtimes serialize on
GPU) so the win is smaller; wide/independent graphs are where the drop-in shines.

## Scope
- **Accelerated:** kernels loaded via `hipModuleLoadData*` + `hipModuleGetFunction`
  (explicit `hipGraphAddKernelNode` graphs and module-based stream capture).
- **Transparent fallback (correct, no speedup):** statically-compiled kernels
  (`__hipRegisterFunction` / `<<<>>>`) — the interposer keeps a native HIP shadow
  graph. Full static coverage needs `__hipRegisterFatBinary`/`__hipRegisterFunction`
  interception (follow-up). `graph_demo` (static kernels) shows this parity path;
  `graph_demo_mod` (module kernels) shows the accelerated path.
- Unsupported node kinds (memcpy/memset/host/child-graph) forward to native HIP.
