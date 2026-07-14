// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
//
// ABI-neutral GFX12 form of the ACO dispatch-floor shader. Redline supplies:
//   s[0:1] = pointer to one 16-byte storage-buffer SRD
//   s2      = element count
//   s3      = reserved
//   s4      = output element base
// Firmware supplies v0 (local id) and ttmp9 (workgroup id), as for the HSA
// kernels exercised by the same PM4 encoder.
//
// Build raw text bytes:
//   llvm-mc -triple=amdgcn-amd-amdhsa -mcpu=gfx1201 -filetype=obj \
//     gmb_aco_style.s -o gmb_aco_style.o
//   llvm-objcopy --dump-section .text=gmb_aco_style.bin gmb_aco_style.o

        .text
        .globl  gmb_aco_style
        .p2align 8
gmb_aco_style:
        v_lshl_add_u32 v0, ttmp9, 8, v0
        s_delay_alu instid0(VALU_DEP_1)
        v_cmpx_gt_u32_e32 s2, v0
        s_cbranch_execz .Lend
        s_load_b128 s[0:3], s[0:1], 0x0
        v_add_lshl_u32 v0, s4, v0, 2
        s_wait_kmcnt 0x0
        buffer_load_b32 v1, v0, s[0:3], null offen
        s_wait_loadcnt 0x0
        v_add_f32_e32 v1, 1.0, v1
        buffer_store_b32 v1, v0, s[0:3], null offen
.Lend:
        s_nop 0
        s_sendmsg sendmsg(MSG_DEALLOC_VGPRS)
        s_endpgm
