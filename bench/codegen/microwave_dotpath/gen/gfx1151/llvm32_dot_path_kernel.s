	.globl	_Z15dot_path_kernelPKjS0_Pijjjj
	.p2align	8
	.type	_Z15dot_path_kernelPKjS0_Pijjjj,@function
_Z15dot_path_kernelPKjS0_Pijjjj:        ; @_Z15dot_path_kernelPKjS0_Pijjjj
	.cfi_startproc
; %bb.0:
	.cfi_escape 0x0f, 0x04, 0x30, 0x36, 0xe9, 0x02 ; CFA is 0 in private_wave aspace
	.cfi_undefined 16
	s_clause 0x1
	s_load_b32 s3, s[0:1], 0x34
	s_load_b128 s[4:7], s[0:1], 0x18
	s_waitcnt lgkmcnt(0)
	s_and_b32 s3, s3, 0xffff
	s_delay_alu instid0(SALU_CYCLE_1)
	v_mad_u64_u32 v[0:1], null, s2, s3, v[0:1]
	s_mov_b32 s2, exec_lo
	v_cmpx_gt_u32_e64 s4, v0
	s_cbranch_execz .LBB0_6
; %bb.1:
	s_clause 0x1
	s_load_b128 s[8:11], s[0:1], 0x0
	s_load_b64 s[0:1], s[0:1], 0x10
	s_cmp_eq_u32 s5, 0
	s_cbranch_scc1 .LBB0_4
; %bb.2:                                ; %.lr.ph.i.preheader
	v_lshl_or_b32 v3, v0, 4, 15
	v_mov_b32_e32 v2, 0
	v_mov_b32_e32 v4, 0
	s_lshl_b32 s2, s4, 4
.LBB0_3:                                ; %.lr.ph.i
                                        ; =>This Inner Loop Header: Depth=1
	s_delay_alu instid0(VALU_DEP_3)
	v_add_nc_u32_e32 v1, -15, v3
	v_add_nc_u32_e32 v7, -14, v3
	v_add_nc_u32_e32 v9, -13, v3
	v_add_nc_u32_e32 v13, -12, v3
	v_add_nc_u32_e32 v14, -11, v3
	v_and_b32_e32 v1, s6, v1
	v_add_nc_u32_e32 v15, -10, v3
	v_add_nc_u32_e32 v16, -9, v3
	v_add_nc_u32_e32 v17, -8, v3
	v_add_nc_u32_e32 v18, -7, v3
	v_lshlrev_b64 v[5:6], 2, v[1:2]
	v_and_b32_e32 v1, s6, v7
	v_add_nc_u32_e32 v19, -6, v3
	v_add_nc_u32_e32 v20, -5, v3
	v_add_nc_u32_e32 v21, -4, v3
	v_add_nc_u32_e32 v22, -3, v3
	v_lshlrev_b64 v[7:8], 2, v[1:2]
	v_and_b32_e32 v1, s6, v9
	s_waitcnt lgkmcnt(0)
	v_add_co_u32 v9, vcc_lo, s8, v5
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1)
	v_add_co_ci_u32_e64 v10, null, s9, v6, vcc_lo
	v_add_co_u32 v5, vcc_lo, s10, v5
	v_add_co_ci_u32_e64 v6, null, s11, v6, vcc_lo
	global_load_b32 v25, v[9:10], off
	global_load_b32 v26, v[5:6], off
	v_add_co_u32 v5, vcc_lo, s8, v7
	v_lshlrev_b64 v[11:12], 2, v[1:2]
	v_add_co_ci_u32_e64 v6, null, s9, v8, vcc_lo
	v_and_b32_e32 v1, s6, v13
	v_add_co_u32 v7, vcc_lo, s10, v7
	s_delay_alu instid0(VALU_DEP_1)
	v_add_co_ci_u32_e64 v8, null, s11, v8, vcc_lo
	global_load_b32 v27, v[5:6], off
	v_add_co_u32 v5, vcc_lo, s8, v11
	v_lshlrev_b64 v[9:10], 2, v[1:2]
	v_add_co_ci_u32_e64 v6, null, s9, v12, vcc_lo
	v_add_co_u32 v11, vcc_lo, s10, v11
	v_and_b32_e32 v1, s6, v14
	v_add_co_ci_u32_e64 v12, null, s11, v12, vcc_lo
	global_load_b32 v28, v[7:8], off
	v_add_nc_u32_e32 v23, -2, v3
	v_lshlrev_b64 v[7:8], 2, v[1:2]
	v_and_b32_e32 v1, s6, v15
	global_load_b32 v15, v[5:6], off
	global_load_b32 v29, v[11:12], off
	v_add_co_u32 v5, vcc_lo, s8, v9
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1)
	v_add_co_ci_u32_e64 v6, null, s9, v10, vcc_lo
	v_add_co_u32 v9, vcc_lo, s10, v9
	v_add_co_ci_u32_e64 v10, null, s11, v10, vcc_lo
	v_lshlrev_b64 v[11:12], 2, v[1:2]
	v_and_b32_e32 v1, s6, v16
	global_load_b32 v16, v[5:6], off
	v_add_co_u32 v5, vcc_lo, s8, v7
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1)
	v_add_co_ci_u32_e64 v6, null, s9, v8, vcc_lo
	v_add_co_u32 v7, vcc_lo, s10, v7
	v_add_co_ci_u32_e64 v8, null, s11, v8, vcc_lo
	global_load_b32 v30, v[9:10], off
	v_lshlrev_b64 v[9:10], 2, v[1:2]
	v_and_b32_e32 v1, s6, v17
	global_load_b32 v17, v[5:6], off
	global_load_b32 v31, v[7:8], off
	v_add_co_u32 v5, vcc_lo, s8, v11
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1)
	v_add_co_ci_u32_e64 v6, null, s9, v12, vcc_lo
	v_add_co_u32 v7, vcc_lo, s10, v11
	v_add_co_ci_u32_e64 v8, null, s11, v12, vcc_lo
	v_lshlrev_b64 v[11:12], 2, v[1:2]
	v_and_b32_e32 v1, s6, v18
	global_load_b32 v18, v[5:6], off
	v_add_co_u32 v5, vcc_lo, s8, v9
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1)
	v_add_co_ci_u32_e64 v6, null, s9, v10, vcc_lo
	v_add_co_u32 v9, vcc_lo, s10, v9
	v_add_co_ci_u32_e64 v10, null, s11, v10, vcc_lo
	global_load_b32 v32, v[7:8], off
	v_lshlrev_b64 v[7:8], 2, v[1:2]
	v_and_b32_e32 v1, s6, v19
	global_load_b32 v19, v[5:6], off
	global_load_b32 v33, v[9:10], off
	v_add_co_u32 v5, vcc_lo, s8, v11
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1)
	v_add_co_ci_u32_e64 v6, null, s9, v12, vcc_lo
	v_add_co_u32 v9, vcc_lo, s10, v11
	v_add_co_ci_u32_e64 v10, null, s11, v12, vcc_lo
	v_lshlrev_b64 v[11:12], 2, v[1:2]
	v_and_b32_e32 v1, s6, v20
	global_load_b32 v20, v[5:6], off
	v_add_co_u32 v5, vcc_lo, s8, v7
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1)
	v_add_co_ci_u32_e64 v6, null, s9, v8, vcc_lo
	v_add_co_u32 v7, vcc_lo, s10, v7
	v_add_co_ci_u32_e64 v8, null, s11, v8, vcc_lo
	global_load_b32 v34, v[9:10], off
	v_lshlrev_b64 v[9:10], 2, v[1:2]
	v_and_b32_e32 v1, s6, v21
	global_load_b32 v21, v[5:6], off
	global_load_b32 v35, v[7:8], off
	v_add_co_u32 v5, vcc_lo, s8, v11
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1)
	v_add_co_ci_u32_e64 v6, null, s9, v12, vcc_lo
	v_add_co_u32 v7, vcc_lo, s10, v11
	v_add_co_ci_u32_e64 v8, null, s11, v12, vcc_lo
	v_lshlrev_b64 v[11:12], 2, v[1:2]
	v_and_b32_e32 v1, s6, v22
	global_load_b32 v22, v[5:6], off
	v_add_co_u32 v5, vcc_lo, s8, v9
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1)
	v_add_co_ci_u32_e64 v6, null, s9, v10, vcc_lo
	v_add_co_u32 v9, vcc_lo, s10, v9
	v_add_co_ci_u32_e64 v10, null, s11, v10, vcc_lo
	global_load_b32 v36, v[7:8], off
	v_lshlrev_b64 v[7:8], 2, v[1:2]
	v_and_b32_e32 v1, s6, v23
	global_load_b32 v23, v[5:6], off
	global_load_b32 v37, v[9:10], off
	v_add_co_u32 v5, vcc_lo, s8, v11
	v_add_nc_u32_e32 v24, -1, v3
	v_add_co_ci_u32_e64 v6, null, s9, v12, vcc_lo
	v_add_co_u32 v9, vcc_lo, s10, v11
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_4) | instid1(VALU_DEP_1)
	v_add_co_ci_u32_e64 v10, null, s11, v12, vcc_lo
	v_lshlrev_b64 v[11:12], 2, v[1:2]
	v_and_b32_e32 v1, s6, v24
	global_load_b32 v24, v[5:6], off
	v_add_co_u32 v5, vcc_lo, s8, v7
	v_add_co_ci_u32_e64 v6, null, s9, v8, vcc_lo
	v_add_co_u32 v7, vcc_lo, s10, v7
	s_delay_alu instid0(VALU_DEP_1)
	v_add_co_ci_u32_e64 v8, null, s11, v8, vcc_lo
	global_load_b32 v38, v[9:10], off
	v_lshlrev_b64 v[9:10], 2, v[1:2]
	v_and_b32_e32 v1, s6, v3
	global_load_b32 v39, v[5:6], off
	global_load_b32 v40, v[7:8], off
	v_add_co_u32 v5, vcc_lo, s8, v11
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1)
	v_add_co_ci_u32_e64 v6, null, s9, v12, vcc_lo
	v_add_co_u32 v7, vcc_lo, s10, v11
	v_add_co_ci_u32_e64 v8, null, s11, v12, vcc_lo
	v_lshlrev_b64 v[11:12], 2, v[1:2]
	v_add_co_u32 v13, vcc_lo, s8, v9
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1)
	v_add_co_ci_u32_e64 v14, null, s9, v10, vcc_lo
	v_add_co_u32 v9, vcc_lo, s10, v9
	v_add_co_ci_u32_e64 v10, null, s11, v10, vcc_lo
	global_load_b32 v1, v[5:6], off
	global_load_b32 v41, v[7:8], off
	global_load_b32 v13, v[13:14], off
	global_load_b32 v9, v[9:10], off
	v_add_co_u32 v5, vcc_lo, s8, v11
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1)
	v_add_co_ci_u32_e64 v6, null, s9, v12, vcc_lo
	v_add_co_u32 v7, vcc_lo, s10, v11
	v_add_co_ci_u32_e64 v8, null, s11, v12, vcc_lo
	global_load_b32 v5, v[5:6], off
	global_load_b32 v6, v[7:8], off
	v_add_nc_u32_e32 v3, s2, v3
	s_add_i32 s5, s5, -1
	s_delay_alu instid0(SALU_CYCLE_1) | instskip(SKIP_3) | instid1(VALU_DEP_1)
	s_cmp_lg_u32 s5, 0
	s_waitcnt vmcnt(30)
	v_dot4_i32_iu8 v4, v25, v26, v4 neg_lo:[1,1,0]
	s_waitcnt vmcnt(28)
	v_dot4_i32_iu8 v4, v27, v28, v4 neg_lo:[1,1,0]
	s_waitcnt vmcnt(26)
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1)
	v_dot4_i32_iu8 v4, v15, v29, v4 neg_lo:[1,1,0]
	s_waitcnt vmcnt(24)
	v_dot4_i32_iu8 v4, v16, v30, v4 neg_lo:[1,1,0]
	s_waitcnt vmcnt(22)
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1)
	v_dot4_i32_iu8 v4, v17, v31, v4 neg_lo:[1,1,0]
	s_waitcnt vmcnt(20)
	v_dot4_i32_iu8 v4, v18, v32, v4 neg_lo:[1,1,0]
	s_waitcnt vmcnt(18)
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1)
	v_dot4_i32_iu8 v4, v19, v33, v4 neg_lo:[1,1,0]
	s_waitcnt vmcnt(16)
	v_dot4_i32_iu8 v4, v20, v34, v4 neg_lo:[1,1,0]
	s_waitcnt vmcnt(14)
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1)
	v_dot4_i32_iu8 v4, v21, v35, v4 neg_lo:[1,1,0]
	s_waitcnt vmcnt(12)
	v_dot4_i32_iu8 v4, v22, v36, v4 neg_lo:[1,1,0]
	s_waitcnt vmcnt(10)
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1)
	v_dot4_i32_iu8 v4, v23, v37, v4 neg_lo:[1,1,0]
	s_waitcnt vmcnt(8)
	v_dot4_i32_iu8 v4, v24, v38, v4 neg_lo:[1,1,0]
	s_waitcnt vmcnt(6)
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1)
	v_dot4_i32_iu8 v4, v39, v40, v4 neg_lo:[1,1,0]
	s_waitcnt vmcnt(4)
	v_dot4_i32_iu8 v1, v1, v41, v4 neg_lo:[1,1,0]
	s_waitcnt vmcnt(2)
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1)
	v_dot4_i32_iu8 v1, v13, v9, v1 neg_lo:[1,1,0]
	s_waitcnt vmcnt(0)
	v_dot4_i32_iu8 v4, v5, v6, v1 neg_lo:[1,1,0]
	s_cbranch_scc1 .LBB0_3
	s_branch .LBB0_5
.LBB0_4:
	v_mov_b32_e32 v4, 0
.LBB0_5:                                ; %_Z9run_valuePKjS0_jjjj.exit
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1)
	v_dual_mov_b32 v1, 0 :: v_dual_add_nc_u32 v2, s7, v4
	v_lshlrev_b64 v[0:1], 2, v[0:1]
	s_waitcnt lgkmcnt(0)
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1)
	v_add_co_u32 v0, vcc_lo, s0, v0
	v_add_co_ci_u32_e64 v1, null, s1, v1, vcc_lo
	global_store_b32 v[0:1], v2, off
.LBB0_6:
	s_endpgm
.Lfunc_end0:
	.size	_Z15dot_path_kernelPKjS0_Pijjjj, .Lfunc_end0-_Z15dot_path_kernelPKjS0_Pijjjj
	.cfi_endproc
	.section	.rodata,"a",@progbits
	.p2align	6, 0x0
	.amdhsa_kernel _Z15dot_path_kernelPKjS0_Pijjjj
		.amdhsa_group_segment_fixed_size 0
		.amdhsa_private_segment_fixed_size 0
		.amdhsa_kernarg_size 296
		.amdhsa_user_sgpr_count 2
		.amdhsa_user_sgpr_dispatch_ptr 0
		.amdhsa_user_sgpr_queue_ptr 0
		.amdhsa_user_sgpr_kernarg_segment_ptr 1
		.amdhsa_user_sgpr_dispatch_id 0
		.amdhsa_user_sgpr_private_segment_size 0
		.amdhsa_wavefront_size32 1
		.amdhsa_uses_dynamic_stack 0
		.amdhsa_enable_private_segment 0
		.amdhsa_system_sgpr_workgroup_id_x 1
		.amdhsa_system_sgpr_workgroup_id_y 0
		.amdhsa_system_sgpr_workgroup_id_z 0
		.amdhsa_system_sgpr_workgroup_info 0
		.amdhsa_system_vgpr_workitem_id 0
		.amdhsa_next_free_vgpr 42
		.amdhsa_next_free_sgpr 12
		.amdhsa_reserve_vcc 1
		.amdhsa_float_round_mode_32 0
		.amdhsa_float_round_mode_16_64 0
		.amdhsa_float_denorm_mode_32 3
		.amdhsa_float_denorm_mode_16_64 3
		.amdhsa_dx10_clamp 1
		.amdhsa_ieee_mode 1
		.amdhsa_fp16_overflow 0
		.amdhsa_workgroup_processor_mode 1
		.amdhsa_memory_ordered 1
		.amdhsa_forward_progress 1
		.amdhsa_shared_vgpr_count 0
		.amdhsa_inst_pref_size ((instprefsize(.Lfunc_end0-_Z15dot_path_kernelPKjS0_Pijjjj)<<4)&1008)>>4
		.amdhsa_exception_fp_ieee_invalid_op 0
		.amdhsa_exception_fp_denorm_src 0
		.amdhsa_exception_fp_ieee_div_zero 0
		.amdhsa_exception_fp_ieee_overflow 0
		.amdhsa_exception_fp_ieee_underflow 0
		.amdhsa_exception_fp_ieee_inexact 0
		.amdhsa_exception_int_div_zero 0
	.end_amdhsa_kernel
	.text
                                        ; -- End function
	.set .L_Z15dot_path_kernelPKjS0_Pijjjj.num_vgpr, 42
	.set .L_Z15dot_path_kernelPKjS0_Pijjjj.num_agpr, 0
	.set .L_Z15dot_path_kernelPKjS0_Pijjjj.numbered_sgpr, 12
	.set .L_Z15dot_path_kernelPKjS0_Pijjjj.num_named_barrier, 0
	.set .L_Z15dot_path_kernelPKjS0_Pijjjj.private_seg_size, 0
	.set .L_Z15dot_path_kernelPKjS0_Pijjjj.uses_vcc, 1
	.set .L_Z15dot_path_kernelPKjS0_Pijjjj.uses_flat_scratch, 0
	.set .L_Z15dot_path_kernelPKjS0_Pijjjj.has_dyn_sized_stack, 0
	.set .L_Z15dot_path_kernelPKjS0_Pijjjj.has_recursion, 0
	.set .L_Z15dot_path_kernelPKjS0_Pijjjj.has_indirect_call, 0
	.section	.AMDGPU.csdata,"",@progbits
; Kernel info:
; codeLenInByte = 1500
; TotalNumSgprs: 14
; NumVgprs: 42
; ScratchSize: 0
; MemoryBound: 0
; FloatMode: 240
; IeeeMode: 1
; LDSByteSize: 0 bytes/workgroup (compile time only)
; SGPRBlocks: 0
; VGPRBlocks: 5
; NumSGPRsForWavesPerEU: 14
; NumVGPRsForWavesPerEU: 42
; Occupancy: 16
; WaveLimiterHint : 0
; COMPUTE_PGM_RSRC2:SCRATCH_EN: 0
; COMPUTE_PGM_RSRC2:USER_SGPR: 2
; COMPUTE_PGM_RSRC2:TRAP_HANDLER: 0
; COMPUTE_PGM_RSRC2:TGID_X_EN: 1
; COMPUTE_PGM_RSRC2:TGID_Y_EN: 0
; COMPUTE_PGM_RSRC2:TGID_Z_EN: 0
; COMPUTE_PGM_RSRC2:TIDIG_COMP_CNT: 0
	.text
	.protected	_Z18dot_path_kernel_u4PKjS0_Pijjjj ; -- Begin function _Z18dot_path_kernel_u4PKjS0_Pijjjj
