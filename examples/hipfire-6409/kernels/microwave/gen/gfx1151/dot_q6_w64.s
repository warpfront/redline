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
	s_cbranch_execz BB11                                        ; bfa500ee
BB1:
	s_mov_b32 s7, 0                                             ; be870080
	v_mov_b32_e32 v1, 0                                         ; 7e020280
BB2:
	v_mov_b32_e32 v2, 0                                         ; 7e040280
	global_load_b32 v2, v2, s[2:3] offset:28                    ; dc52001c 02020002
	s_waitcnt vmcnt(0)                                          ; bf8903f7
	v_readfirstlane_b32 s8, v2                                  ; 7e100502
	s_cmp_ge_u32 s7, s8                                         ; bf090807
	s_cbranch_scc1 BB6                                          ; bfa200d2
BB5:
	v_mov_b32_e32 v2, 0                                         ; 7e040280
	global_load_b32 v2, v2, s[2:3] offset:36                    ; dc520024 02020002
	s_mul_i32 s8, s7, s6                                        ; 96080607
	v_mov_b32_e32 v4, s1                                        ; 7e080201
	v_mov_b32_e32 v5, s5                                        ; 7e0a0205
	v_add_lshl_u32 v3, s8, v0, 4                                ; d6470003 02120008
	s_waitcnt vmcnt(0)                                          ; bf8903f7
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_and_b32_e32 v2, v2, v3                                    ; 36040702
	v_lshrrev_b32_e32 v6, 2, v2                                 ; 320c0482
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_ashrrev_i32_e32 v7, 31, v6                                ; 340e0c9f
	v_lshlrev_b64 v[6:7], 4, v[6:7]                             ; d73c0006 00020c84
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_add_co_u32 v10, vcc, s0, v6                               ; d7006a0a 00020c00
	v_add_co_ci_u32_e32 v11, vcc, v4, v7, vcc                   ; 40160f04
	v_add_co_u32 v12, vcc, s4, v6                               ; d7006a0c 00020c04
	v_add_nc_u32_e32 v6, 4, v2                                  ; 4a0c0484
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a2
	v_add_co_ci_u32_e32 v13, vcc, v5, v7, vcc                   ; 401a0f05
	v_lshrrev_b32_e32 v6, 2, v6                                 ; 320c0c82
	v_ashrrev_i32_e32 v7, 31, v6                                ; 340e0c9f
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_lshlrev_b64 v[6:7], 4, v[6:7]                             ; d73c0006 00020c84
	v_add_co_u32 v14, vcc, s0, v6                               ; d7006a0e 00020c00
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_add_co_ci_u32_e32 v15, vcc, v4, v7, vcc                   ; 401e0f04
	v_add_co_u32 v16, vcc, s4, v6                               ; d7006a10 00020c04
	v_add_co_ci_u32_e32 v17, vcc, v5, v7, vcc                   ; 40220f05
	v_add_nc_u32_e32 v7, 8, v2                                  ; 4a0e0488
	v_add_nc_u32_e32 v2, 12, v2                                 ; 4a04048c
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_1) | instid1(VALU_DEP_2) ; bf870122
	v_lshrrev_b32_e32 v6, 2, v7                                 ; 320c0e82
	v_lshrrev_b32_e32 v2, 2, v2                                 ; 32040482
	v_ashrrev_i32_e32 v7, 31, v6                                ; 340e0c9f
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_lshlrev_b64 v[6:7], 4, v[6:7]                             ; d73c0006 00020c84
	v_ashrrev_i32_e32 v3, 31, v2                                ; 3406049f
	v_lshlrev_b64 v[2:3], 4, v[2:3]                             ; d73c0002 00020484
	v_add_co_u32 v18, vcc, s0, v6                               ; d7006a12 00020c00
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_add_co_ci_u32_e32 v19, vcc, v4, v7, vcc                   ; 40260f04
	v_add_co_u32 v20, vcc, s4, v6                               ; d7006a14 00020c04
	v_add_co_ci_u32_e32 v21, vcc, v5, v7, vcc                   ; 402a0f05
	v_add_co_u32 v6, vcc, s0, v2                                ; d7006a06 00020400
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_add_co_ci_u32_e32 v7, vcc, v4, v3, vcc                    ; 400e0704
	v_add_co_u32 v22, vcc, s4, v2                               ; d7006a16 00020404
	v_add_co_ci_u32_e32 v23, vcc, v5, v3, vcc                   ; 402e0705
	s_clause 0x7                                                ; bf850007
	global_load_b128 v[24:27], v[10:11], off                    ; dc5e0000 187c000a
	global_load_b128 v[28:31], v[12:13], off                    ; dc5e0000 1c7c000c
	global_load_b128 v[12:15], v[14:15], off                    ; dc5e0000 0c7c000e
	global_load_b128 v[32:35], v[16:17], off                    ; dc5e0000 207c0010
	global_load_b128 v[16:19], v[18:19], off                    ; dc5e0000 107c0012
	global_load_b128 v[36:39], v[20:21], off                    ; dc5e0000 247c0014
	global_load_b128 v[4:7], v[6:7], off                        ; dc5e0000 047c0006
	global_load_b128 v[20:23], v[22:23], off                    ; dc5e0000 147c0016
	s_mov_b32 s9, 0x1010101                                     ; be8900ff 01010101
	s_add_u32 s7, s7, 1                                         ; 80078107
	s_waitcnt vmcnt(6)                                          ; bf891bf7
	v_dot4_i32_iu8 v11, v29, s9, 0 neg_lo:[1,0,0]               ; cc16400b 3a00131d
	v_dot4_i32_iu8 v10, v28, s9, 0 neg_lo:[1,0,0]               ; cc16400a 3a00131c
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_1) | instid1(VALU_DEP_2) ; bf870122
	v_lshlrev_b32_e32 v11, 5, v11                               ; 30161685
	v_lshlrev_b32_e32 v10, 5, v10                               ; 30141485
	v_sub_nc_u32_e32 v11, 0, v11                                ; 4c161680
	v_sub_nc_u32_e32 v10, 0, v10                                ; 4c141480
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_3) | instid1(VALU_DEP_3) ; bf8701c2
	v_dot4_i32_iu8 v29, v29, v25, v11 neg_lo:[1,0,0]            ; cc16401d 3c2e331d
	v_dot4_i32_iu8 v25, v31, s9, 0 neg_lo:[1,0,0]               ; cc164019 3a00131f
	v_dot4_i32_iu8 v28, v28, v24, v10 neg_lo:[1,0,0]            ; cc16401c 3c2a311c
	v_dot4_i32_iu8 v24, v30, s9, 0 neg_lo:[1,0,0]               ; cc164018 3a00131e
	v_lshlrev_b32_e32 v25, 5, v25                               ; 30323285
	v_add3_u32 v29, v29, v1, v28                                ; d655001d 0472031d
	s_waitcnt vmcnt(4)                                          ; bf8913f7
	v_dot4_i32_iu8 v28, v34, s9, 0 neg_lo:[1,0,0]               ; cc16401c 3a001322
	v_lshlrev_b32_e32 v24, 5, v24                               ; 30303085
	s_delay_alu instid0(VALU_DEP_4) | instskip(NEXT) | instid1(VALU_DEP_3) ; bf870194
	v_sub_nc_u32_e32 v25, 0, v25                                ; 4c323280
	v_lshlrev_b32_e32 v28, 5, v28                               ; 30383885
	v_sub_nc_u32_e32 v24, 0, v24                                ; 4c303080
	s_delay_alu instid0(VALU_DEP_3) | instskip(SKIP_4) | instid1(VALU_DEP_4) ; bf870253
	v_dot4_i32_iu8 v31, v31, v27, v25 neg_lo:[1,0,0]            ; cc16401f 3c66371f
	v_dot4_i32_iu8 v27, v33, s9, 0 neg_lo:[1,0,0]               ; cc16401b 3a001321
	v_sub_nc_u32_e32 v28, 0, v28                                ; 4c383880
	v_dot4_i32_iu8 v30, v30, v26, v24 neg_lo:[1,0,0]            ; cc16401e 3c62351e
	v_dot4_i32_iu8 v26, v32, s9, 0 neg_lo:[1,0,0]               ; cc16401a 3a001320
	v_lshlrev_b32_e32 v27, 5, v27                               ; 30363685
	v_dot4_i32_iu8 v34, v34, v14, v28 neg_lo:[1,0,0]            ; cc164022 3c721d22
	v_add3_u32 v31, v31, v29, v30                               ; d655001f 047a3b1f
	v_dot4_i32_iu8 v29, v35, s9, 0 neg_lo:[1,0,0]               ; cc16401d 3a001323
	s_waitcnt vmcnt(2)                                          ; bf890bf7
	v_dot4_i32_iu8 v30, v36, s9, 0 neg_lo:[1,0,0]               ; cc16401e 3a001324
	v_lshlrev_b32_e32 v26, 5, v26                               ; 30343485
	v_sub_nc_u32_e32 v27, 0, v27                                ; 4c363680
	s_delay_alu instid0(VALU_DEP_4) | instskip(SKIP_3) | instid1(VALU_DEP_4) ; bf870244
	v_lshlrev_b32_e32 v29, 5, v29                               ; 303a3a85
	v_lshlrev_b32_e32 v30, 5, v30                               ; 303c3c85
	v_sub_nc_u32_e32 v26, 0, v26                                ; 4c343480
	v_dot4_i32_iu8 v33, v33, v13, v27 neg_lo:[1,0,0]            ; cc164021 3c6e1b21
	v_sub_nc_u32_e32 v29, 0, v29                                ; 4c3a3a80
	v_sub_nc_u32_e32 v30, 0, v30                                ; 4c3c3c80
	v_dot4_i32_iu8 v32, v32, v12, v26 neg_lo:[1,0,0]            ; cc164020 3c6a1920
	s_delay_alu instid0(VALU_DEP_3) | instskip(SKIP_4) | instid1(VALU_DEP_3) ; bf8701d3
	v_dot4_i32_iu8 v35, v35, v15, v29 neg_lo:[1,0,0]            ; cc164023 3c761f23
	v_dot4_i32_iu8 v36, v36, v16, v30 neg_lo:[1,0,0]            ; cc164024 3c7a2124
	v_add3_u32 v33, v33, v31, v32                               ; d6550021 04823f21
	v_dot4_i32_iu8 v31, v37, s9, 0 neg_lo:[1,0,0]               ; cc16401f 3a001325
	v_dot4_i32_iu8 v32, v38, s9, 0 neg_lo:[1,0,0]               ; cc164020 3a001326
	v_add3_u32 v35, v35, v33, v34                               ; d6550023 048a4323
	v_dot4_i32_iu8 v33, v39, s9, 0 neg_lo:[1,0,0]               ; cc164021 3a001327
	s_waitcnt vmcnt(0)                                          ; bf8903f7
	v_dot4_i32_iu8 v34, v20, s9, 0 neg_lo:[1,0,0]               ; cc164022 3a001314
	v_lshlrev_b32_e32 v31, 5, v31                               ; 303e3e85
	v_lshlrev_b32_e32 v32, 5, v32                               ; 30404085
	s_delay_alu instid0(VALU_DEP_4) | instskip(SKIP_3) | instid1(VALU_DEP_4) ; bf870244
	v_lshlrev_b32_e32 v33, 5, v33                               ; 30424285
	v_lshlrev_b32_e32 v34, 5, v34                               ; 30444485
	v_sub_nc_u32_e32 v31, 0, v31                                ; 4c3e3e80
	v_sub_nc_u32_e32 v32, 0, v32                                ; 4c404080
	v_sub_nc_u32_e32 v33, 0, v33                                ; 4c424280
	v_sub_nc_u32_e32 v34, 0, v34                                ; 4c444480
	v_dot4_i32_iu8 v37, v37, v17, v31 neg_lo:[1,0,0]            ; cc164025 3c7e2325
	v_dot4_i32_iu8 v38, v38, v18, v32 neg_lo:[1,0,0]            ; cc164026 3c822526
	s_delay_alu instid0(VALU_DEP_4) | instskip(SKIP_4) | instid1(VALU_DEP_3) ; bf8701d4
	v_dot4_i32_iu8 v39, v39, v19, v33 neg_lo:[1,0,0]            ; cc164027 3c862727
	v_dot4_i32_iu8 v20, v20, v4, v34 neg_lo:[1,0,0]             ; cc164014 3c8a0914
	v_add3_u32 v37, v37, v35, v36                               ; d6550025 04924725
	v_dot4_i32_iu8 v35, v21, s9, 0 neg_lo:[1,0,0]               ; cc164023 3a001315
	v_dot4_i32_iu8 v36, v22, s9, 0 neg_lo:[1,0,0]               ; cc164024 3a001316
	v_add3_u32 v39, v39, v37, v38                               ; d6550027 049a4b27
	v_dot4_i32_iu8 v37, v23, s9, 0 neg_lo:[1,0,0]               ; cc164025 3a001317
	v_lshlrev_b32_e32 v35, 5, v35                               ; 30464685
	v_lshlrev_b32_e32 v36, 5, v36                               ; 30484885
	s_delay_alu instid0(VALU_DEP_3) | instskip(SKIP_2) | instid1(VALU_DEP_3) ; bf8701b3
	v_lshlrev_b32_e32 v37, 5, v37                               ; 304a4a85
	v_sub_nc_u32_e32 v35, 0, v35                                ; 4c464680
	v_sub_nc_u32_e32 v36, 0, v36                                ; 4c484880
	v_sub_nc_u32_e32 v37, 0, v37                                ; 4c4a4a80
	v_dot4_i32_iu8 v21, v21, v5, v35 neg_lo:[1,0,0]             ; cc164015 3c8e0b15
	v_dot4_i32_iu8 v22, v22, v6, v36 neg_lo:[1,0,0]             ; cc164016 3c920d16
	s_delay_alu instid0(VALU_DEP_3) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a3
	v_dot4_i32_iu8 v23, v23, v7, v37 neg_lo:[1,0,0]             ; cc164017 3c960f17
	v_add3_u32 v21, v21, v39, v20                               ; d6550015 04524f15
	v_add3_u32 v1, v23, v21, v22                                ; d6550001 045a2b17
	s_branch BB2                                                ; bfa0ff27
BB6:
	v_mov_b32_e32 v2, 0                                         ; 7e040280
	global_load_b32 v2, v2, s[2:3] offset:32                    ; dc520020 02020002
	s_waitcnt vmcnt(0)                                          ; bf8903f7
	v_add_nc_u32_e32 v2, v2, v0                                 ; 4a040102
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_ashrrev_i32_e32 v3, 31, v2                                ; 3406049f
	v_lshlrev_b64 v[2:3], 2, v[2:3]                             ; d73c0002 00020482
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_add_co_u32 v4, vcc, v8, v2                                ; d7006a04 00020508
	v_add_co_ci_u32_e32 v5, vcc, v9, v3, vcc                    ; 400a0709
	global_load_b32 v3, v[4:5], off                             ; dc520000 037c0004
	s_waitcnt vmcnt(0)                                          ; bf8903f7
	v_add_nc_u32_e32 v3, v3, v1                                 ; 4a060303
	global_store_b32 v[4:5], v3, off                            ; dc6a0000 007c0304
BB11:
	s_endpgm                                                    ; bfb00000
 