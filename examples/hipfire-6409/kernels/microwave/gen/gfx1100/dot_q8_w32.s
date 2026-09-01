BB0:
	v_mov_b32_e32 v1, 0                                         ; 7e020280
	s_clause 0x1                                                ; bf850001
	global_load_b128 v[4:7], v1, s[2:3]                         ; dc5e0000 04020001
	global_load_b96 v[8:10], v1, s[2:3] offset:16               ; dc5a0010 08020001
	s_mul_i32 s5, s5, s4                                        ; 96050405
	s_delay_alu instid0(SALU_CYCLE_1)                           ; bf870009
	v_add_nc_u32_e32 v0, s5, v0                                 ; 4a000005
	s_waitcnt vmcnt(1)                                          ; bf8907f7
	v_readfirstlane_b32 s0, v4                                  ; 7e000504
	v_readfirstlane_b32 s1, v5                                  ; 7e020505
	v_readfirstlane_b32 s4, v6                                  ; 7e080506
	v_readfirstlane_b32 s5, v7                                  ; 7e0a0507
	s_waitcnt vmcnt(0)                                          ; bf8903f7
	v_readfirstlane_b32 s6, v10                                 ; 7e0c050a
	v_cmpx_gt_u32_e32 v10, v0                                   ; 7d98010a
	s_cbranch_execz BB11                                        ; bfa5009b
BB1:
	v_mov_b32_e32 v1, 0                                         ; 7e020280
	s_mov_b32 s7, 0                                             ; be870080
BB2:
	v_mov_b32_e32 v2, 0                                         ; 7e040280
	global_load_b32 v2, v2, s[2:3] offset:28                    ; dc52001c 02020002
	s_waitcnt vmcnt(0)                                          ; bf8903f7
	v_readfirstlane_b32 s8, v2                                  ; 7e100502
	s_cmp_ge_u32 s7, s8                                         ; bf090807
	s_cbranch_scc1 BB6                                          ; bfa2007f
BB5:
	v_mov_b32_e32 v2, 0                                         ; 7e040280
	global_load_b32 v2, v2, s[2:3] offset:36                    ; dc520024 02020002
	v_mov_b32_e32 v4, s1                                        ; 7e080201
	s_mul_i32 s8, s7, s6                                        ; 96080607
	s_delay_alu instid0(SALU_CYCLE_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a9
	v_add_lshl_u32 v3, s8, v0, 4                                ; d6470003 02120008
	s_waitcnt vmcnt(0)                                          ; bf8903f7
	v_and_b32_e32 v2, v2, v3                                    ; 36040702
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_lshrrev_b32_e32 v6, 2, v2                                 ; 320c0482
	v_ashrrev_i32_e32 v7, 31, v6                                ; 340e0c9f
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_lshlrev_b64 v[6:7], 4, v[6:7]                             ; d73c0006 00020c84
	v_add_co_u32 v10, vcc_lo, s0, v6                            ; d7006a0a 00020c00
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_2) | instid1(VALU_DEP_1) ; bf8700b1
	v_add_co_ci_u32_e32 v11, vcc_lo, v4, v7, vcc_lo             ; 40160f04
	v_add_co_u32 v12, vcc_lo, s4, v6                            ; d7006a0c 00020c04
	v_dual_mov_b32 v5, s5 :: v_dual_add_nc_u32 v6, 4, v2        ; ca200005 05060484
	v_add_co_ci_u32_e32 v13, vcc_lo, v5, v7, vcc_lo             ; 401a0f05
	v_lshrrev_b32_e32 v6, 2, v6                                 ; 320c0c82
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_ashrrev_i32_e32 v7, 31, v6                                ; 340e0c9f
	v_lshlrev_b64 v[6:7], 4, v[6:7]                             ; d73c0006 00020c84
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_add_co_u32 v14, vcc_lo, s0, v6                            ; d7006a0e 00020c00
	v_add_co_ci_u32_e32 v15, vcc_lo, v4, v7, vcc_lo             ; 401e0f04
	v_add_co_u32 v16, vcc_lo, s4, v6                            ; d7006a10 00020c04
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_2) | instid1(VALU_DEP_2) ; bf870131
	v_add_co_ci_u32_e32 v17, vcc_lo, v5, v7, vcc_lo             ; 40220f05
	v_add_nc_u32_e32 v7, 8, v2                                  ; 4a0e0488
	v_add_nc_u32_e32 v2, 12, v2                                 ; 4a04048c
	v_lshrrev_b32_e32 v6, 2, v7                                 ; 320c0e82
	v_lshrrev_b32_e32 v2, 2, v2                                 ; 32040482
	s_delay_alu instid0(VALU_DEP_2) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870092
	v_ashrrev_i32_e32 v7, 31, v6                                ; 340e0c9f
	v_lshlrev_b64 v[6:7], 4, v[6:7]                             ; d73c0006 00020c84
	v_ashrrev_i32_e32 v3, 31, v2                                ; 3406049f
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_lshlrev_b64 v[2:3], 4, v[2:3]                             ; d73c0002 00020484
	v_add_co_u32 v18, vcc_lo, s0, v6                            ; d7006a12 00020c00
	v_add_co_ci_u32_e32 v19, vcc_lo, v4, v7, vcc_lo             ; 40260f04
	v_add_co_u32 v20, vcc_lo, s4, v6                            ; d7006a14 00020c04
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_add_co_ci_u32_e32 v21, vcc_lo, v5, v7, vcc_lo             ; 402a0f05
	v_add_co_u32 v6, vcc_lo, s0, v2                             ; d7006a06 00020400
	v_add_co_ci_u32_e32 v7, vcc_lo, v4, v3, vcc_lo              ; 400e0704
	v_add_co_u32 v22, vcc_lo, s4, v2                            ; d7006a16 00020404
	s_delay_alu instid0(VALU_DEP_1)                             ; bf870001
	v_add_co_ci_u32_e32 v23, vcc_lo, v5, v3, vcc_lo             ; 402e0705
	s_clause 0x7                                                ; bf850007
	global_load_b128 v[24:27], v[10:11], off                    ; dc5e0000 187c000a
	global_load_b128 v[28:31], v[12:13], off                    ; dc5e0000 1c7c000c
	global_load_b128 v[12:15], v[14:15], off                    ; dc5e0000 0c7c000e
	global_load_b128 v[32:35], v[16:17], off                    ; dc5e0000 207c0010
	global_load_b128 v[16:19], v[18:19], off                    ; dc5e0000 107c0012
	global_load_b128 v[36:39], v[20:21], off                    ; dc5e0000 247c0014
	global_load_b128 v[4:7], v[6:7], off                        ; dc5e0000 047c0006
	global_load_b128 v[20:23], v[22:23], off                    ; dc5e0000 147c0016
	s_add_u32 s7, s7, 1                                         ; 80078107
	s_waitcnt vmcnt(6)                                          ; bf891bf7
	v_dot4_i32_iu8 v24, v24, v28, v1 neg_lo:[1,1,0]             ; cc164018 7c063918
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_dot4_i32_iu8 v25, v25, v29, v24 neg_lo:[1,1,0]            ; cc164019 7c623b19
	v_dot4_i32_iu8 v26, v26, v30, v25 neg_lo:[1,1,0]            ; cc16401a 7c663d1a
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_dot4_i32_iu8 v27, v27, v31, v26 neg_lo:[1,1,0]            ; cc16401b 7c6a3f1b
	s_waitcnt vmcnt(4)                                          ; bf8913f7
	v_dot4_i32_iu8 v12, v12, v32, v27 neg_lo:[1,1,0]            ; cc16400c 7c6e410c
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_dot4_i32_iu8 v13, v13, v33, v12 neg_lo:[1,1,0]            ; cc16400d 7c32430d
	v_dot4_i32_iu8 v14, v14, v34, v13 neg_lo:[1,1,0]            ; cc16400e 7c36450e
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_dot4_i32_iu8 v15, v15, v35, v14 neg_lo:[1,1,0]            ; cc16400f 7c3a470f
	s_waitcnt vmcnt(2)                                          ; bf890bf7
	v_dot4_i32_iu8 v16, v16, v36, v15 neg_lo:[1,1,0]            ; cc164010 7c3e4910
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_dot4_i32_iu8 v17, v17, v37, v16 neg_lo:[1,1,0]            ; cc164011 7c424b11
	v_dot4_i32_iu8 v18, v18, v38, v17 neg_lo:[1,1,0]            ; cc164012 7c464d12
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_dot4_i32_iu8 v19, v19, v39, v18 neg_lo:[1,1,0]            ; cc164013 7c4a4f13
	s_waitcnt vmcnt(0)                                          ; bf8903f7
	v_dot4_i32_iu8 v4, v4, v20, v19 neg_lo:[1,1,0]              ; cc164004 7c4e2904
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_dot4_i32_iu8 v5, v5, v21, v4 neg_lo:[1,1,0]               ; cc164005 7c122b05
	v_dot4_i32_iu8 v6, v6, v22, v5 neg_lo:[1,1,0]               ; cc164006 7c162d06
	s_delay_alu instid0(VALU_DEP_1)                             ; bf870001
	v_dot4_i32_iu8 v1, v7, v23, v6 neg_lo:[1,1,0]               ; cc164001 7c1a2f07
	s_branch BB2                                                ; bfa0ff7a
BB6:
	v_mov_b32_e32 v2, 0                                         ; 7e040280
	global_load_b32 v2, v2, s[2:3] offset:32                    ; dc520020 02020002
	s_waitcnt vmcnt(0)                                          ; bf8903f7
	v_add_nc_u32_e32 v2, v2, v0                                 ; 4a040102
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_ashrrev_i32_e32 v3, 31, v2                                ; 3406049f
	v_lshlrev_b64 v[2:3], 2, v[2:3]                             ; d73c0002 00020482
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_add_co_u32 v4, vcc_lo, v8, v2                             ; d7006a04 00020508
	v_add_co_ci_u32_e32 v5, vcc_lo, v9, v3, vcc_lo              ; 400a0709
	global_load_b32 v3, v[4:5], off                             ; dc520000 037c0004
	s_waitcnt vmcnt(0)                                          ; bf8903f7
	v_add_nc_u32_e32 v3, v3, v1                                 ; 4a060303
	global_store_b32 v[4:5], v3, off                            ; dc6a0000 007c0304
BB11:
	s_endpgm                                                    ; bfb00000
 