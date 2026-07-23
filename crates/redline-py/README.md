<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# redline-dispatch (Python)

Python bindings for [Redline](https://github.com/Kaden-Schutt/redline) —
lightning-fast kernel dispatch for ROCm. Author and validate graphs, or load
code objects and replay retained PM4 from Python.

Build/install instructions and the shared kernel contract are in
[`docs/INTEGRATION.md`](../../docs/INTEGRATION.md#python).

```python
import redline_dispatch as rl

g = rl.Graph(mode="latency")
acts = g.buffer("activations", 4096)
project = g.kernel("project", (32, 1, 1), (64, 1, 1),
                   accesses=[(acts, 0, 2048, False), (acts, 2048, 2048, True)])
g.kernel("consume", (32, 1, 1), (64, 1, 1),
         accesses=[(acts, 2048, 2048, False)], deps=[project])

exec = g.instantiate()      # == hipGraphInstantiate
exec.launch_mock()          # validate ordering/fences (no GPU)
print(exec.lane_count, exec.fingerprint())
```

The real-GPU API can consume the same certified code-object contract as the
Rust path:

```python
gpu = rl.Gpu(0)
module = gpu.load_module(
    open("decode.hsaco", "rb").read(),
    open("decode.radiowave.json").read(),
)
assert module.radiowave_certified
print(module.scheduler_profile, module.wavefront_size)
```

When `Gpu.build()` inserts a serialized RMW edge, it uses the verified next
consumer's cache classification. VMEM-only consumers get Redline's minimal
vector/L1 acquire; missing manifests or ambiguous kernels automatically retain
the generic scalar/vector/L1 boundary.

The current Python `Gpu.build()` path emits GFX12 direct PM4. Use the C or Rust
architecture-dispatched APIs for GFX10/GFX11 real-GPU replay.

## Per-token decode: build once, patch, replay

The retained IB is the whole point for autoregressive decode — build it **once**,
then every token patch only the scalar/pointer that changed (position, KV-cache
slot, ...) in place and replay. No IB rebuild, one doorbell per token. A leading
same-agent cache acquire is emitted so each replay re-reads the mutated kernargs.

```python
gpu = rl.Gpu(0)
mod = gpu.load_module(open("acc.hsaco", "rb").read(), None)
acc = gpu.alloc(4)
# acc_k(unsigned* acc, unsigned val): acc@0 (8B), val@8 (4B)
kernarg = acc.address().to_bytes(8, "little") + (0).to_bytes(4, "little")
ib = gpu.build(mod, [("acc_k.kd", (1, 1, 1), (1, 1, 1), 0, kernarg, True)])

for token in range(1, T + 1):
    ib.set_kernargs(0, token.to_bytes(4, "little"), byte_offset=8)  # patch val
    ib.replay()
```

`set_kernargs(dispatch_index, data, byte_offset=0)` overwrites the retained
kernarg segment of one dispatch (in `build` record order) in place; the PM4
packet keeps the same address, so no rebuild is needed. See
[`examples/decode_kernargs.py`](examples/decode_kernargs.py) for a
correctness-gated run. The C-ABI mirror is `rl_pm4_ib_set_kernargs`.

Licensed under Apache-2.0. "Redline" is a trademark of Kaden Schutt.
