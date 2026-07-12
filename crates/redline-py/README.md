<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- Copyright 2026 Kaden Schutt -->

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

Licensed under Apache-2.0. "Redline" is a trademark of Kaden Schutt.
