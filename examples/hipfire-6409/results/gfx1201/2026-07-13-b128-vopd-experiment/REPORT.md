# B128 interleave and wave32 VOPD experiment

This experiment tested the two next-step candidates against the accepted
Radiowave/Redline gfx1201 result. Every measured backend output passed the CPU
oracle. Rejected candidates were removed from the selected source.

## Result

Neither candidate is promoted. The accepted benchmark remains at **121/133**
strict Redline wins over Vulkan.

### B128 input plus buffer output

The candidate combined a B128 input request, explicit buffer output, and a
64-thread workgroup. Its emitted wave64 kernel used 12 VGPRs and 12 SGPRs,
with no private memory or spills, and retained Radiowave's `vmem_only`
certification.

The focused frozen-binary A/B/B/A initially suggested a large-serial benefit:

| Large interleave row | Candidate median | Baseline median | Candidate change |
|---|---:|---:|---:|
| Serial Redline | 10.6675 us | 10.9681 us | -2.74% |
| Aggressive Redline | 9.9600 us | 9.7800 us | +1.84% |

That serial improvement did not survive complete-matrix certification. Across
the two full seven-sample replicates, the candidate's large serial result was
11.3288 us versus 10.9044 us in the prior certified aggregate, a **3.89%
regression**. The aggressive row used the unchanged accepted kernel.

The candidate full aggregate therefore remained at 121/133 and was rejected
as benchmark-order sensitive. The full artifacts are under [`final`](final/),
and the focused frozen-binary runs are the `ab-a*` and `ab-b*` directories.

### Wave32 VOPD

ROCm 7.2 hipcc already enables LLVM's wave32 VOPD pass by default. Compiling
with an explicit `-mllvm -amdgpu-enable-vopd` produced instruction-identical
disassembly, so the flag itself provides no new compiler lever.

The existing wave32 source produced 93 static `v_dual_*` instructions across
the code object but won only 1/12 VOPD rows. A compact unroll-2 experiment
reduced the independent kernel from 279 to 124 static instructions and the
mixed kernel from 502 to 243, but also reduced exposed dual pairs from 19 to 4
and 15 to 2 respectively. It won 3/12 rows and remained slower than the
accepted wave64 selection, which wins 4/12.

The original wave32 run is in [`vopd-wave32-probe`](vopd-wave32-probe/) and
the compact run is in [`vopd-wave32-compact-u2`](vopd-wave32-compact-u2/).

## Conclusion

The remaining interleave loss is not fixed by merely combining B128 input and
buffer output. Likewise, LLVM VOPD formation is not disabled or missing; on
this workload the wave32 occupancy/throughput trade loses more than the dual
pairs recover. The next useful work remains source scheduling for mixed and
dequant wave64 kernels, and a true staged index/value software pipeline for
gather.
