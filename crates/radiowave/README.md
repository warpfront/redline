# Radiowave

Radiowave is Redline's policy-driven HIP compiler boundary. Applications give
Radiowave HIP source rather than invoking hipcc directly. Radiowave injects
reviewed lowering helpers, calls the installed ROCm compiler, inspects the
resulting AMDGPU code object, and writes a reproducible JSON manifest.

The first promoted rule is the gfx11/gfx12 buffer-resource load/store layer
derived from the ROCm issue 6409 benchmark. It lets HIP source request the
same 32-bit-offset buffer instruction class that Vulkan SSBOs naturally expose
to RADV/ACO.

```bash
cargo run -p radiowave -- compile \
  --source kernel.hip \
  --output kernel.hsaco \
  --arch gfx1201 \
  --wave32 \
  --scheduler-profile default
```

Radiowave force-includes `radiowave/hip.h`. Kernels can use:

```cpp
const auto resource = radiowave::buffer_resource(words);
uint32_t value = radiowave::buffer_load_u32(resource, word_offset);
auto four_values = radiowave::buffer_load_u32x4(resource, aligned_word_offset);
radiowave::buffer_store_u32(resource, word_offset, value);
```

For experiments which need adjacent scalar VMEM operations to remain distinct,
`buffer_load_bytes_u32` accepts a byte offset and `opaque_vgpr_u32x4` prevents
LLVM from folding four reviewed offsets back into one B128 load. These helpers
are opt-in: inspection proves the requested instruction shape, while benchmark
correctness and timing decide whether it may be promoted.

The promoted gfx1201 benchmark rules now include scalar B32 resource access,
aligned B128 tile loads, and correctness-gated variant selection for wave size,
unroll factor, and workgroup shape. Radiowave does not replace LLVM: it makes
the source and launch-policy decisions explicit, compiles them with upstream
hipcc, and rejects resource or spill regressions using the emitted manifest.

## Scheduler profiles

Radiowave exposes reviewed AMDGPU scheduler configurations as typed profiles,
not arbitrary hidden compiler arguments:

| Profile | LLVM machine-scheduler policy |
|---|---|
| `default` | upstream hipcc defaults |
| `max-ilp` | `gcn-max-ilp` |
| `iterative-ilp` | `gcn-iterative-ilp` |
| `memory-clause` | `gcn-max-memory-clause`, maximum clause length 4 |
| `pipeline-ilp` | max-ILP plus relaxed occupancy, exact igrouplp solving, and the machine pipeliner |

Each profile is compiled into a separate code object. A caller selects the
profile explicitly, and HIP, HipGraph, and Redline then load that identical
object. Radiowave does not promote a profile merely because its disassembly is
shorter: the candidate must also pass the CPU oracle and a counterbalanced GPU
timing comparison.

Each schema-3 compile manifest records source, injected-header, and output
hashes; the complete compiler command and selected scheduler profile; ROCm
version; kernel VGPR/SGPR/private-memory metadata; counts of
buffer/global/flat/scalar memory instructions; static instruction, wait,
`s_delay_alu`, and `s_clause` counts; the longest consecutive VMEM run; and a
fail-closed mutable-read cache classification. Scalar loads from the live
kernarg pointer are classified as immutable prologue reads. A kernel is
`vmem_only` only when every other resource read is VMEM; scalar-buffer loads,
loads through an overwritten kernarg SGPR pair, and unknown scalar-memory
forms classify as `scalar_or_unknown`.

Redline verifies the embedded manifest schema, wavefront, and code-object
SHA-256 before using that classification. A `vmem_only` consumer can use the
gfx12 same-agent `CS_PARTIAL_FLUSH + GLV/GL1` boundary (`GCR=0x00300`). Missing,
stale, or scalar/unknown evidence falls back to `CS_PARTIAL_FLUSH + GLK/GLV/GL1`
(`0x00380`); certification never weakens the completion edge.

Runtime adapters should construct `CodeObjectCertification::from_json` from
the exact bytes they will load. The C and PyO3 bindings use this same reusable
validator; neither binding independently interprets or trusts manifest JSON.

## Current safety boundary

- The injected descriptor configuration is currently pinned to gfx11/gfx12.
- Buffer byte offsets are 32-bit. Allocations larger than 4 GiB must rebase the
  descriptor instead of allowing an offset to wrap.
- Radiowave does not silently rewrite arbitrary pointer accesses yet. Kernels
  opt into the helper where bounds, alignment, aliasing, and coherence are
  known.
- Cache certification is intentionally conservative and instruction-class
  based. It proves which shader read caches a same-agent consumer may use; it
  does not prove resource ownership or remove the required producer-completion
  wait.
- Inspection is evidence, not binary rewriting. Retuning happens by compiling
  another source/IR variant and correctness-gating it.

The next stages are typed buffer views, bounded source transformations,
cache-policy variants, and reusable benchmark-fed selection manifests.
LLVM/hipcc remains the backend; Radiowave owns the policy.
