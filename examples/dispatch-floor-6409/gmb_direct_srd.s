// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
//
// Redline-native buffer ABI for the dispatch floor. Unlike Vulkan's descriptor
// table ABI, the retained PM4 image binds the complete storage-buffer SRD in
// user SGPRs, so each dispatch avoids a scalar descriptor load and its wait:
//   s[0:3] = 16-byte storage-buffer SRD
//   s4      = element count
// Firmware supplies v0 (local id) and ttmp9 (workgroup id).
//
// Build raw text bytes:
//   llvm-mc -triple=amdgcn-amd-amdhsa -mcpu=gfx1201 -filetype=obj \
//     gmb_direct_srd.s -o gmb_direct_srd.o
//   llvm-objcopy --dump-section .text=gmb_direct_srd.bin gmb_direct_srd.o

        .text
        .globl  gmb_direct_srd
        .p2align 8
gmb_direct_srd:
        v_lshl_add_u32 v0, ttmp9, 8, v0
        s_delay_alu instid0(VALU_DEP_1)
        v_cmpx_gt_u32_e32 s4, v0
        s_cbranch_execz .Lend
        v_lshlrev_b32_e32 v0, 2, v0
        buffer_load_b32 v1, v0, s[0:3], null offen
        s_wait_loadcnt 0x0
        v_add_f32_e32 v1, 1.0, v1
        buffer_store_b32 v1, v0, s[0:3], null offen
.Lend:
        s_nop 0
        s_sendmsg sendmsg(MSG_DEALLOC_VGPRS)
        s_endpgm
