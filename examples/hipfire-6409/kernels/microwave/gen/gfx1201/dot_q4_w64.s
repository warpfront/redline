BB0:
	v_mov_b32_e32 v1, 0                                         ; 7e020280
	s_clause 0x1                                                ; bf850001
	global_load_b128 v[4:7], v1, s[2:3]                         ; ee05c002 00000004 00000001
	global_load_b96 v[8:10], v1, s[2:3] offset:16               ; ee058002 00000008 00001001
	s_mul_i32 s4, ttmp9, s4                                     ; 96040475
	s_delay_alu instid0(SALU_CYCLE_1)                           ; bf870009
	v_add_nc_u32_e32 v0, s4, v0                                 ; 4a000004
	s_wait_loadcnt 0x1                                          ; bfc00001
	v_readfirstlane_b32 s1, v5                                  ; 7e020505
	v_readfirstlane_b32 s5, v7                                  ; 7e0a0507
	s_wait_loadcnt 0x0                                          ; bfc00000
	v_readfirstlane_b32 s6, v9                                  ; 7e0c0509
	v_readfirstlane_b32 s7, v10                                 ; 7e0e050a
	v_cmpx_gt_u32_e32 v10, v0                                   ; 7d98010a
	s_mov_b32 s4, s1                                            ; be840001
	s_cbranch_execz BB11                                        ; bfa500ab
BB1:
	s_mov_b32 s0, 0                                             ; be800080
	v_mov_b32_e32 v1, 0                                         ; 7e020280
BB2:
	v_mov_b32_e32 v2, 0                                         ; 7e040280
	global_load_b32 v2, v2, s[2:3] offset:28                    ; ee050002 00000002 00001c02
	s_wait_loadcnt 0x0                                          ; bfc00000
	v_readfirstlane_b32 s1, v2                                  ; 7e020502
	s_cmp_ge_u32 s0, s1                                         ; bf090100
	s_cbranch_scc1 BB6                                          ; bfa2008a
BB5:
	v_mov_b32_e32 v2, 0                                         ; 7e040280
	global_load_b32 v2, v2, s[2:3] offset:36                    ; ee050002 00000002 00002402
	s_mul_i32 s1, s0, s7                                        ; 96010700
	s_wait_alu 0xfffe                                           ; bf88fffe
	v_add_lshl_u32 v3, s1, v0, 4                                ; d6470003 02120001
	s_wait_loadcnt 0x0                                          ; bfc00000
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_and_b32_e32 v2, v2, v3                                    ; 36040702
	v_add_nc_u32_e32 v7, 8, v2                                  ; 4a0e0488
	v_lshrrev_b32_e32 v10, 2, v2                                ; 32140482
	v_add_nc_u32_e32 v5, 4, v2                                  ; 4a0a0484
	s_delay_alu instid0(VALU_DEP_2) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870092
	v_ashrrev_i32_e32 v11, 31, v10                              ; 3416149f
	v_lshlrev_b64_e64 v[10:11], 4, v[10:11]                     ; d51f000a 00021484
	s_delay_alu instid0(VALU_DEP_1)                             ; bf870001
	v_add_co_u32 v12, vcc, v4, v10                              ; d7006a0c 00021504
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_add_co_ci_u32_e32 v13, vcc, s4, v11, vcc                  ; 401a1604
	v_add_co_u32 v14, vcc, v6, v10                              ; d7006a0e 00021506
	v_lshrrev_b32_e32 v10, 2, v5                                ; 32140a82
	v_add_nc_u32_e32 v2, 12, v2                                 ; 4a04048c
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_add_co_ci_u32_e32 v15, vcc, s5, v11, vcc                  ; 401e1605
	v_ashrrev_i32_e32 v11, 31, v10                              ; 3416149f
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_2) ; bf870121
	v_lshlrev_b64_e64 v[10:11], 4, v[10:11]                     ; d51f000a 00021484
	v_lshrrev_b32_e32 v2, 2, v2                                 ; 32040482
	v_add_co_u32 v16, vcc, v4, v10                              ; d7006a10 00021504
	v_ashrrev_i32_e32 v3, 31, v2                                ; 3406049f
	s_delay_alu instid0(VALU_DEP_1)                             ; bf870001
	v_lshlrev_b64_e64 v[2:3], 4, v[2:3]                         ; d51f0002 00020484
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_add_co_ci_u32_e32 v17, vcc, s4, v11, vcc                  ; 40221604
	v_add_co_u32 v18, vcc, v6, v10                              ; d7006a12 00021506
	v_lshrrev_b32_e32 v10, 2, v7                                ; 32140e82
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_add_co_ci_u32_e32 v19, vcc, s5, v11, vcc                  ; 40261605
	v_ashrrev_i32_e32 v11, 31, v10                              ; 3416149f
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_lshlrev_b64_e64 v[10:11], 4, v[10:11]                     ; d51f000a 00021484
	v_add_co_u32 v20, vcc, v4, v10                              ; d7006a14 00021504
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_add_co_ci_u32_e32 v21, vcc, s4, v11, vcc                  ; 402a1604
	v_add_co_u32 v22, vcc, v6, v10                              ; d7006a16 00021506
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_add_co_ci_u32_e32 v23, vcc, s5, v11, vcc                  ; 402e1605
	v_add_co_u32 v10, vcc, v4, v2                               ; d7006a0a 00020504
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_add_co_ci_u32_e32 v11, vcc, s4, v3, vcc                   ; 40160604
	v_add_co_u32 v24, vcc, v6, v2                               ; d7006a18 00020506
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_add_co_ci_u32_e32 v25, vcc, s5, v3, vcc                   ; 40320605
	s_clause 0x7                                                ; bf850007
	global_load_b128 v[28:31], v[12:13], off                    ; ee05c07c 0000001c 0000000c
	global_load_b128 v[12:15], v[14:15], off                    ; ee05c07c 0000000c 0000000e
	global_load_b128 v[32:35], v[16:17], off                    ; ee05c07c 00000020 00000010
	global_load_b128 v[16:19], v[18:19], off                    ; ee05c07c 00000010 00000012
	global_load_b128 v[36:39], v[20:21], off                    ; ee05c07c 00000024 00000014
	global_load_b128 v[20:23], v[22:23], off                    ; ee05c07c 00000014 00000016
	global_load_b128 v[40:43], v[10:11], off                    ; ee05c07c 00000028 0000000a
	global_load_b128 v[24:27], v[24:25], off                    ; ee05c07c 00000018 00000018
	s_add_co_u32 s0, s0, 1                                      ; 80008100
	s_wait_loadcnt 0x6                                          ; bfc00006
	v_dot4_i32_iu8 v12, v12, v28, v1 neg_lo:[1,0,0]             ; cc16400c 3c06390c
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_dot4_i32_iu8 v13, v13, v29, v12 neg_lo:[1,0,0]            ; cc16400d 3c323b0d
	v_dot4_i32_iu8 v14, v14, v30, v13 neg_lo:[1,0,0]            ; cc16400e 3c363d0e
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_dot4_i32_iu8 v15, v15, v31, v14 neg_lo:[1,0,0]            ; cc16400f 3c3a3f0f
	s_wait_loadcnt 0x4                                          ; bfc00004
	v_dot4_i32_iu8 v16, v16, v32, v15 neg_lo:[1,0,0]            ; cc164010 3c3e4110
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_dot4_i32_iu8 v17, v17, v33, v16 neg_lo:[1,0,0]            ; cc164011 3c424311
	v_dot4_i32_iu8 v18, v18, v34, v17 neg_lo:[1,0,0]            ; cc164012 3c464512
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_dot4_i32_iu8 v19, v19, v35, v18 neg_lo:[1,0,0]            ; cc164013 3c4a4713
	s_wait_loadcnt 0x2                                          ; bfc00002
	v_dot4_i32_iu8 v20, v20, v36, v19 neg_lo:[1,0,0]            ; cc164014 3c4e4914
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_dot4_i32_iu8 v21, v21, v37, v20 neg_lo:[1,0,0]            ; cc164015 3c524b15
	v_dot4_i32_iu8 v22, v22, v38, v21 neg_lo:[1,0,0]            ; cc164016 3c564d16
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_dot4_i32_iu8 v23, v23, v39, v22 neg_lo:[1,0,0]            ; cc164017 3c5a4f17
	s_wait_loadcnt 0x0                                          ; bfc00000
	v_dot4_i32_iu8 v24, v24, v40, v23 neg_lo:[1,0,0]            ; cc164018 3c5e5118
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_dot4_i32_iu8 v25, v25, v41, v24 neg_lo:[1,0,0]            ; cc164019 3c625319
	v_dot4_i32_iu8 v26, v26, v42, v25 neg_lo:[1,0,0]            ; cc16401a 3c66551a
	s_delay_alu instid0(VALU_DEP_1)                             ; bf870001
	v_dot4_i32_iu8 v1, v27, v43, v26 neg_lo:[1,0,0]             ; cc164001 3c6a571b
	s_branch BB2                                                ; bfa0ff6e
BB6:
	v_mov_b32_e32 v2, 0                                         ; 7e040280
	global_load_b32 v2, v2, s[2:3] offset:32                    ; ee050002 00000002 00002002
	s_wait_loadcnt 0x0                                          ; bfc00000
	v_add_nc_u32_e32 v2, v2, v0                                 ; 4a040102
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_ashrrev_i32_e32 v3, 31, v2                                ; 3406049f
	v_lshlrev_b64_e64 v[2:3], 2, v[2:3]                         ; d51f0002 00020482
	s_delay_alu instid0(VALU_DEP_1)                             ; bf870001
	v_add_co_u32 v4, vcc, v8, v2                                ; d7006a04 00020508
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_add_co_ci_u32_e32 v5, vcc, s6, v3, vcc                    ; 400a0606
	global_load_b32 v3, v[4:5], off                             ; ee05007c 00000003 00000004
	s_wait_loadcnt 0x0                                          ; bfc00000
	v_add_nc_u32_e32 v3, v3, v1                                 ; 4a060303
	global_store_b32 v[4:5], v3, off                            ; ee06807c 01800000 00000004
BB11:
	s_nop 0                                                     ; bf800000
	s_sendmsg sendmsg(MSG_DEALLOC_VGPRS)                        ; bfb60003
	s_endpgm                                                    ; bfb00000
 