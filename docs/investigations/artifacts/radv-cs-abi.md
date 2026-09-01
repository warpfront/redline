# RADV CS ABI notes (Mesa mesa-25.2.8)

Source: `mesa-25.2.8` tag (gitlab.freedesktop.org/mesa/mesa). Paths relative to tree root.

## 1. User-SGPR / sysval layout (microwave-style CS)

Scenario: compute, no descriptor sets, 3-dword push constant (`uint64` + `uint`), `gl_WorkGroupID.x` + `gl_LocalInvocationID.x` only, no `gl_NumWorkGroups`/subgroups/LDS/scratch.

### Push-constant inlining rule

- Caps: `AC_MAX_INLINE_PUSH_CONSTS=32` (all-inline path), `AC_MAX_INLINE_PUSH_CONSTS_WITH_INDIRECT=8` (`ac_shader_args.h:14-16`).
- Dword mask: const-offset `load_push_constant` with `bit_size>=32` sets `inline_push_constant_mask` bits; non-const or OOB clears `can_inline_all_push_constants` (`radv_shader_info.c:198-205`). Default `can_inline_all_push_constants=true` (`:1071-1072`).
- Allocation (`radv_shader_args.c:25-51`): if `num_push_consts <= MIN2(remaining_sgprs+1, 32)` **and** `can_inline_all_push_constants` **and** `!loads_dynamic_offsets` → `inlined_all_push_consts=true` (frees the indirect push-const pointer slot by `remaining_sgprs++`). Else clamp mask to `MIN2(remaining, 8)` by dropping high dwords.
- CS user-SGPR budget is **16** (`:899`). First pass always spends 2 on `ring_offsets`; if `loads_push_constants`, one more is reserved before remaining is computed (`:893-900`).
- **This 3-dword PC always fully inlines** under the scenario (3 ≤ remaining+1, no dynamic offsets). No `push_constants` pointer SGPR (`:101-104`).

Inline dword *i* maps to `inline_push_consts[popcount(mask & ((1<<i)-1))]` (`radv_nir_apply_pipeline_layout.c:365-369`).

### Exact layout

Args get consecutive SGPR/VGPR offsets via `ac_add_arg` (`ac_shader_args.c:17-28`). Order for CS (`radv_shader_args.c:566`, `:584-631`):

| Slot | Contents |
|------|----------|
| **s[0:1]** | `ring_offsets` (always; `AC_UD_SCRATCH_RING_OFFSETS`) |
| **s[2], s[3], s[4]** | inlined PC dwords 0,1,2 (`uint64` lo/hi, then `uint`) |
| **gfx11 (gfx1100/gfx1151): s[5]** | `workgroup_ids[0]` if `uses_block_id[0]` (`:613-618`) — system SGPR after user SGPRs |
| **gfx12 (gfx1201): ttmp9** | `workgroup_id.x` — **not** an input SGPR; only `.used=true` (`:615-616`). HW: TTMP9 / TTMP7 y/z (`ac_shader_args.h:180-181`) |
| **v0** | `local_invocation_ids_packed` on GFX11+ (`:630-631`). Pack: 10 bits/component in VGPR0 (`ac_nir_lower_intrinsics_to_args.c:175-184`). If `local_size_y=z=1`, X is the whole v0 (extract 32 bits). |

**Not present** in this scenario:

- `tg_size` — only if `uses_local_invocation_idx` (local **index** / subgroup id / LDS clear), not plain `LocalInvocationID` (`radv_shader_info.c:1135-1138`, `radv_shader_args.c:622-623`).
- `num_work_groups`, desc sets, scratch_offset (scratch_offset only GFX\<11 with `explicit_scratch_args`, `:626-628`).

### Mechanical `p_startpgm` ABI read (ACO IR)

IR dump is **pre-RA** (`aco_interface.cpp:107-118`). `add_startpgm` precolors each arg def to `PhysReg{sgpr_off}` or `PhysReg{vgpr_off+256}` (`aco_isel_helpers.cpp:675-698`). GFX12 CS appends three extra defs precolored `PhysReg(108+9/8/7)` = ISA ttmp9/8/7 (`:707-716`; GFX9+ ttmp0 encodes as 108 per trap handler).

Print form (tests):  
`s2: %0:s[0-1], s1: %1:s[2], …, v1: %N:v[0][, s1: %T:s[117] …] = p_startpgm`  
(`aco_print_ir.cpp:88-120`, `print_definition` with fixed phys). Parse fixed `s[…]`/`v[…]` on `p_startpgm` defs left-to-right = arg order. GFX12 workgroup X is the def fixed to encoding **117** (LLVM asm name **ttmp9**). CS `load_workgroup_id` becomes `p_create_vector` of `ctx->workgroup_id[]` (`aco_select_nir_intrinsics.cpp:4164-4169`).

## 2. `aco_print_asm` "Assembly" (LLVM path)

On LLVM-enabled builds, GFX8+ uses `print_asm_llvm` (`aco_print_asm.cpp:409-418`).

- **Labels:** `BB%u:\n` for referenced blocks (`:48-49`); symbols registered as `"BB%u"` at byte offset (`:312-316`) so LLVM branch operands print as **BB labels**, not raw numeric offsets.
- **Line shape:** `%-60s ;` + little-endian encoding dwords (`:58-61`). Leading tab from LLVM text.
- **Annotations to strip before feeding an assembler:** trailing ` ; <hex…>`; `\t(then repeated N times)\n` compression (`:344-345`) — must **expand**; optional trailing `/* constant data */` dump (`:66-82`); `(invalid instruction)` placeholders (`:291-294`).
- **`s_endpgm`:** isel emits `s_endpgm` when `need_endpgm` for a normal CS (`aco_select_nir.cpp:1353-1361`, single shader `need_endpgm=true` at `:1453`). Assembler may also append five `0xbf9f0000` UMR end markers after real code (`aco_assembler.cpp:1822-1824`) — not executable body. Empty/epilog paths may omit; typical CS binary ends with real `s_endpgm`.

## 3. Pipeline executable IR / stats names

**IR** (`radv_pipeline.c:1069-1098`):

| name | notes |
|------|-------|
| `"NIR Shader(s)"` | optimized NIR |
| `"ACO IR"` | or `"LLVM IR"` if `radv_use_llvm_for_stage` |
| `"Assembly"` | final disasm; only if `shader->disasm_string` |

**Stats** — `vk_add_amd_stats` uses XML `name=` strings (`shader_stats.xml:96-121`, via `process_shader_stats.py`):  
`"Driver pipeline hash"`, `"SGPRs"`, `"VGPRs"`, `"Spilled SGPRs"`, `"Spilled VGPRs"`, `"Code size"`, `"LDS size"`, `"Scratch size"`, `"Subgroups per SIMD"`, `"Combined inputs"`, `"Combined outputs"`, `"Hash"`, `"Instructions"`, `"Copies"`, `"Branches"`, `"Latency"`, `"Inverse Throughput"`, `"VMEM Clause"`, `"SMEM Clause"`, `"Pre-Sched SGPRs"`, `"Pre-Sched VGPRs"`, `"VALU"`, `"SALU"`, `"VMEM"`, `"SMEM"`, `"VOPD"`.

**Capture / cache:** IR kept if `VK_PIPELINE_CREATE_2_CAPTURE_INTERNAL_REPRESENTATIONS_BIT_KHR` **or** `RADV_DEBUG=shaders` **or** `device->keep_shader_info` (`radv_pipeline.c:53-54`). That same CAPTURE bit (or `CAPTURE_DATA`) **skips the shader cache** (`:75-82`). There is **no** `RADV_DEBUG=nocache` requirement in these paths; `nocache` is unrelated here. Stats also via `CAPTURE_STATISTICS` / `RADV_DEBUG=shader_stats|pso_history` / RGP (`:64-66`).

## 4. Prologue assumptions vs bare HSA dispatch

| Assumption | Source | HSA note |
|------------|--------|----------|
| `s[0:1]=ring_offsets` always declared | `radv_shader_args.c:566` | HSA kernarg ABI has no RADV ring descriptor pair |
| Scratch init `p_init_scratch` → flat_scratch via `s_setreg` | only if `scratch_offset.used` and GFX8–10.3 (`aco_isel_helpers.cpp:737-755`, `aco_lower_to_hw_instr.cpp:2687-2706`, `hw_init_scratch` `:2198-2226`) | **GFX11/12 CS with no spill: no scratch prologue** |
| GFX12 TGID in **ttmp9** (y/z in ttmp7); ttmp8 for subgroup id | `:707-716`, `aco_select_nir_intrinsics.cpp:4175-4178` | Must match GFX12 SPI ttmp fill; not user SGPRs |
| GFX11 TGID in **system SGPRs after user SGPRs** | `radv_shader_args.c:618` | HSA may place TGID differently |
| Packed local ids in **v0** (GFX11+) | `:630-631` | Same as AMD compute SPI packing |
| `exec` left as HW-init for CS; only forced `-1` if `uses_full_subgroups` | `aco_insert_exec_mask.cpp:181-185` | Wave must launch with correct exec |
| `m0` not required GFX9+ for LDS size | `aco_isel_helpers.cpp:314-316` | N/A if no LDS |
| No `s_getpc` in normal CS startpgm path | scratch/symbol path only in lower | HSA may use different PC-relative setup |

**Unknowns:** exact USER_SGPR count field programming and whether unused `ring_offsets` stays live after RA (RA may reuse s0/s1 if dead); Assembly branch text may vary slightly by LLVM version; constexpr `PhysReg ttmp9{121}` in `aco_ir.h:443` is GFX6–8 numbering — GFX9+ ISA encoding uses base 108 (`108+9=117` for ttmp9).
