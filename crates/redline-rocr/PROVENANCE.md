# Provenance

`redline-rocr` is an independent Rust implementation against the public
ROCr/HSA ABI. The ROCm source tree is not vendored.

The numeric constants, handle types, function signatures, and C layouts in
`src/abi.rs` are transcribed from the MIT-licensed ROCm 7.2 public headers:

- `/opt/rocm/include/hsa/hsa.h`
- `/opt/rocm/include/hsa/hsa_ext_amd.h`

They were cross-checked against the corresponding files in
[`ROCm/rocm-systems`](https://github.com/ROCm/rocm-systems) at commit
`c0430a50286200ab0562f4733445cdee6e48d416`:

- `projects/rocr-runtime/runtime/hsa-runtime/inc/hsa.h`
- `projects/rocr-runtime/runtime/hsa-runtime/inc/hsa_ext_amd.h`

The packet encoders, ownership wrappers, finite polling, fault propagation,
and release-ordered queue publication in `src/packet.rs` and `src/runtime.rs`
were written for Redline against that public API. No ROCm implementation file
was copied into this crate.

## ROCm 7.14 verification (2026-07-22)

`src/abi.rs` was re-audited against the ROCm Core SDK 7.14.0 (TheRock
packaging) public headers, which now live at
`/opt/rocm/core-7.14/include/hsa/{hsa.h,hsa_ext_amd.h}`
(package `amdrocm-runtime-dev7.14` 7.14.0-3; runtime
`libhsa-runtime64.so.1.21.0`, ROCR BUILD ID `1.21.0-local-build-2b22ab01`;
`HSA_AMD_INTERFACE_VERSION` 1.26).

Zero drift was found across all 101 transcribed items: 58 numeric
constants, 7 `u64` handle types, 2 struct layouts (`hsa_queue_t`
large-model, `hsa_amd_profiling_dispatch_time_t`), and 34 resolved
function signatures. The packet layouts in `src/packet.rs`
(`hsa_kernel_dispatch_packet_t`, barrier-AND) were also confirmed
consistent with the 7.14 headers.

## ROCm 7.14 interface-1.26 expansion (2026-07-22)

Appended ROCm Core SDK 7.14 (`HSA_AMD_INTERFACE_VERSION` 1.26) entry points and
constants to `src/abi.rs`, transcribed from:

- `/opt/rocm/core/include/hsa/hsa_ext_amd.h`
- `/opt/rocm/core/include/hsa/hsa.h` (signal condition / wait-state enums)

### Symbols resolved via `Symbols::load` (hard-required; missing →
`requires ROCm >= 7.14`)

- `hsa_amd_queue_cu_set_mask`
- `hsa_amd_queue_cu_get_mask`
- `hsa_amd_queue_set_priority`
- `hsa_amd_counted_queue_acquire`
- `hsa_amd_counted_queue_release`
- `hsa_amd_queue_get_info`
- `hsa_amd_signal_wait_all`
- `hsa_amd_signal_wait_any`
- `hsa_amd_svm_prefetch_async`
- `hsa_amd_svm_discard_batch_async`
- `hsa_amd_profiling_convert_tick_to_system_domain`

### Constants / types

- `HSA_AMD_AGENT_INFO_COMPUTE_UNIT_COUNT` (`0xA002`) → `AMD_AGENT_INFO_COMPUTE_UNIT_COUNT`
- `HSA_AMD_AGENT_INFO_COOPERATIVE_COMPUTE_UNIT_COUNT` (`0xA014`) → `AMD_AGENT_INFO_COOPERATIVE_COMPUTE_UNIT_COUNT`
- `hsa_amd_queue_priority_t` → `QueuePriority` + `QUEUE_PRIORITY_{LOW,NORMAL,HIGH}`
- `hsa_queue_info_attribute_t` → `QueueInfoAttribute` + `QUEUE_INFO_*` (including VM-fault attrs)
- `hsa_signal_condition_t` / `hsa_wait_state_t` → `SignalCondition` / `WaitState` + matching constants
