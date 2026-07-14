<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev> -->

# redline-dispatch (Python)

Python bindings for [Redline](https://github.com/Kaden-Schutt/redline) — a
leaner, safer HipGraph. Capture a dispatch graph, instantiate, and launch.

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

Licensed under Apache-2.0. "Redline" is a trademark of Kaden Schutt.
