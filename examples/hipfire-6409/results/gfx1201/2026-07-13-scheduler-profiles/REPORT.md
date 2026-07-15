# Radiowave scheduler-profile experiment

This experiment turns AMDGPU machine scheduling into an explicit Radiowave
policy while preserving the benchmark's main control: HIP, HipGraph, and
Redline load the same selected HSACO. Five scheduler profiles are built for
both wave32 and wave64, embedded independently, verified against their
schema-3 manifests, and selected with `--scheduler-profile`.

## Verdict

The compiler-control layer works, but **no scheduler profile or source
candidate clears the promotion gate yet**. The selected benchmark remains on
the upstream `default` scheduler, B128 aggressive interleave, and the original
mixed-VOPD hash. Every reported timed run passed the four backend CPU oracles.
The final 133-row default regression reproduces **121/133 Redline first places
(90.98%)**, with 12 second-place finishes and no third/fourth places.

This is not a null result. Radiowave can now ask upstream LLVM for materially
different AMDGPU schedules, inspect the resulting ISA, and carry the exact
choice through HIP and retained PM4. The measurements show why a shorter
schedule is evidence, not an automatic optimization.

## Emitted wave64 ISA

| Kernel | Profile/candidate | VGPR | Static | Wait | Delay | Clause | Max VMEM run |
|---|---|---:|---:|---:|---:|---:|---:|
| mixed VOPD | default | 14 | 509 | 87 | 77 | 1 | 1 |
| mixed VOPD | max ILP / pipeline ILP | 20 | 445 | 75 | 65 | 1 | 1 |
| mixed VOPD | iterative ILP | 29 | 509 | 93 | 82 | 1 | 1 |
| chunk-16 dequant | default | 37 | 651 | 174 | 165 | 1 | 1 |
| chunk-16 dequant | max ILP / pipeline ILP | 37 | 587 | 172 | 163 | 1 | 1 |
| chunk-16 dequant | iterative ILP | 39 | 651 | 179 | 168 | 1 | 1 |
| aggressive interleave | selected B128 | 12 | 148 | 32 | 25 | 0 | 1 |
| aggressive interleave | four B32 candidate | 12 | 142 | 34 | 24 | 1 | 4 |
| mixed VOPD | paired hash candidate | 14 | 509 | 86 | 76 | 1 | 1 |

All candidates are spill-free and certify the VMEM-only Redline dependency
path. `memory-clause` leaves the original B128, mixed, and dequant schedules
unchanged. `pipeline-ilp` emits the same measured schedules as `max-ilp` for
the two VOPD targets.

## Timing decisions

- **Max ILP: reject for selection.** On mixed VOPD, one default-to-max pass
  improves all three raw Redline medians, but the counterbalanced repeat has
  max ILP slightly worse in serial and independent modes. Dequant is similarly
  inconsistent after normalizing against the Vulkan control. The instruction
  reduction therefore does not survive the repeat gate.
- **Iterative ILP: reject.** It increases register pressure and wait/delay
  counts without a stable timing benefit.
- **Four B32 interleave clause: reject for selection.** The source shim
  successfully stops LLVM from recombining the operations: the object contains
  four consecutive B32 VMEM loads and an `s_clause`. At the large aggressive
  shape, two candidate measurements have a 10.42 us median versus 10.09 us for
  two counterbalanced B128 controls, a **3.27% regression**. Both shapes can
  beat Vulkan under this clock state, but B128 remains the better HIP kernel.
- **Paired mixed hash: keep experimental.** It removes one wait and one delay
  instruction at unchanged VGPR/static count. Two correctness-passing
  candidate passes show small raw gains in some modes, but the Vulkan-normalized
  result is not yet consistent enough to select it.

## Reproduce focused controls

```bash
./target/release/hipfire-6409-bench \
  --wave-policy radiowave --scheduler-profile max-ilp \
  --filter vopd/variant=mixed-int-float --warmups 3 --samples 7 \
  --out results/gfx1201/manual-max-ilp/results.json

./target/release/hipfire-6409-bench \
  --wave-policy radiowave --scheduler-profile default \
  --interleave-aggressive-b32 --filter memory-waitcnt/variant=interleave4 \
  --warmups 3 --samples 7 \
  --out results/gfx1201/manual-interleave-b32/results.json
```

The principal counterbalanced artifacts are
[mixed default](mixed-default/results.json),
[mixed max ILP](mixed-max-ilp/results.json),
[mixed max ILP repeat](mixed-max-ilp-rep2/results.json),
[mixed default repeat](mixed-default-rep2/results.json),
[B32 candidate A1](interleave-b32-clause/rep1/results.json),
[B128 control B1](interleave-b32-clause/b1-baseline/results.json),
[B32 candidate A2](interleave-b32-clause/a2-candidate/results.json), and
[B128 control B2](interleave-b32-clause/b2-baseline/results.json). The
[full default regression](default-regression/results.json) verifies that the
new profile-selection layer preserves the selected 121/133 result.
