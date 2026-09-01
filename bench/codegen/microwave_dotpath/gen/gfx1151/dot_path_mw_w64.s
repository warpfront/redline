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
	s_cbranch_execz BB11                                        ; bfa50174
BB1:
	s_mov_b32 s7, 0                                             ; be870080
	v_mov_b32_e32 v1, 0                                         ; 7e020280
BB2:
	v_mov_b32_e32 v2, 0                                         ; 7e040280
	global_load_b32 v2, v2, s[2:3] offset:28                    ; dc52001c 02020002
	s_waitcnt vmcnt(0)                                          ; bf8903f7
	v_readfirstlane_b32 s8, v2                                  ; 7e100502
	s_cmp_ge_u32 s7, s8                                         ; bf090807
	s_cbranch_scc1 BB6                                          ; bfa2015b
BB5:
	v_mov_b32_e32 v2, 0                                         ; 7e040280
	global_load_b32 v2, v2, s[2:3] offset:32                    ; dc520020 02020002
	s_mul_i32 s8, s7, s6                                        ; 96080607
	v_mov_b32_e32 v4, s1                                        ; 7e080201
	v_mov_b32_e32 v5, s5                                        ; 7e0a0205
	v_add_lshl_u32 v3, s8, v0, 4                                ; d6470003 02120008
	s_waitcnt vmcnt(0)                                          ; bf8903f7
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_and_b32_e32 v3, v2, v3                                    ; 36060702
	v_ashrrev_i32_e32 v7, 31, v3                                ; 340e069f
	v_mov_b32_e32 v6, v3                                        ; 7e0c0303
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_lshlrev_b64 v[6:7], 2, v[6:7]                             ; d73c0006 00020c82
	v_add_co_u32 v10, vcc, s0, v6                               ; d7006a0a 00020c00
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_2) | instid1(VALU_DEP_2) ; bf870131
	v_add_co_ci_u32_e32 v11, vcc, v4, v7, vcc                   ; 40160f04
	v_add_co_u32 v12, vcc, s4, v6                               ; d7006a0c 00020c04
	v_add_nc_u32_e32 v6, 1, v3                                  ; 4a0c0681
	v_add_co_ci_u32_e32 v13, vcc, v5, v7, vcc                   ; 401a0f05
	v_add_nc_u32_e32 v22, 3, v3                                 ; 4a2c0683
	v_and_b32_e32 v6, v2, v6                                    ; 360c0d02
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_ashrrev_i32_e32 v7, 31, v6                                ; 340e0c9f
	v_lshlrev_b64 v[6:7], 2, v[6:7]                             ; d73c0006 00020c82
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_add_co_u32 v14, vcc, s0, v6                               ; d7006a0e 00020c00
	v_add_co_ci_u32_e32 v15, vcc, v4, v7, vcc                   ; 401e0f04
	v_add_co_u32 v16, vcc, s4, v6                               ; d7006a10 00020c04
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_2) | instid1(VALU_DEP_2) ; bf870131
	v_add_co_ci_u32_e32 v17, vcc, v5, v7, vcc                   ; 40220f05
	v_add_nc_u32_e32 v7, 2, v3                                  ; 4a0e0682
	v_add_nc_u32_e32 v26, 4, v3                                 ; 4a340684
	v_and_b32_e32 v6, v2, v7                                    ; 360c0f02
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_ashrrev_i32_e32 v7, 31, v6                                ; 340e0c9f
	v_lshlrev_b64 v[6:7], 2, v[6:7]                             ; d73c0006 00020c82
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_add_co_u32 v18, vcc, s0, v6                               ; d7006a12 00020c00
	v_add_co_ci_u32_e32 v19, vcc, v4, v7, vcc                   ; 40260f04
	v_add_co_u32 v20, vcc, s4, v6                               ; d7006a14 00020c04
	v_and_b32_e32 v6, v2, v22                                   ; 360c2d02
	v_add_nc_u32_e32 v30, 5, v3                                 ; 4a3c0685
	s_delay_alu instid0(VALU_DEP_3) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a3
	v_add_co_ci_u32_e32 v21, vcc, v5, v7, vcc                   ; 402a0f05
	v_ashrrev_i32_e32 v7, 31, v6                                ; 340e0c9f
	v_lshlrev_b64 v[6:7], 2, v[6:7]                             ; d73c0006 00020c82
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_add_co_u32 v22, vcc, s0, v6                               ; d7006a16 00020c00
	v_add_co_ci_u32_e32 v23, vcc, v4, v7, vcc                   ; 402e0f04
	v_add_co_u32 v24, vcc, s4, v6                               ; d7006a18 00020c04
	v_and_b32_e32 v6, v2, v26                                   ; 360c3502
	v_add_nc_u32_e32 v34, 6, v3                                 ; 4a440686
	s_delay_alu instid0(VALU_DEP_3) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a3
	v_add_co_ci_u32_e32 v25, vcc, v5, v7, vcc                   ; 40320f05
	v_ashrrev_i32_e32 v7, 31, v6                                ; 340e0c9f
	v_lshlrev_b64 v[6:7], 2, v[6:7]                             ; d73c0006 00020c82
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_add_co_u32 v26, vcc, s0, v6                               ; d7006a1a 00020c00
	v_add_co_ci_u32_e32 v27, vcc, v4, v7, vcc                   ; 40360f04
	v_add_co_u32 v28, vcc, s4, v6                               ; d7006a1c 00020c04
	v_and_b32_e32 v6, v2, v30                                   ; 360c3d02
	v_add_nc_u32_e32 v38, 7, v3                                 ; 4a4c0687
	s_delay_alu instid0(VALU_DEP_3) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a3
	v_add_co_ci_u32_e32 v29, vcc, v5, v7, vcc                   ; 403a0f05
	v_ashrrev_i32_e32 v7, 31, v6                                ; 340e0c9f
	v_lshlrev_b64 v[6:7], 2, v[6:7]                             ; d73c0006 00020c82
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_add_co_u32 v30, vcc, s0, v6                               ; d7006a1e 00020c00
	v_add_co_ci_u32_e32 v31, vcc, v4, v7, vcc                   ; 403e0f04
	v_add_co_u32 v32, vcc, s4, v6                               ; d7006a20 00020c04
	v_and_b32_e32 v6, v2, v34                                   ; 360c4502
	v_add_nc_u32_e32 v42, 8, v3                                 ; 4a540688
	s_delay_alu instid0(VALU_DEP_3) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a3
	v_add_co_ci_u32_e32 v33, vcc, v5, v7, vcc                   ; 40420f05
	v_ashrrev_i32_e32 v7, 31, v6                                ; 340e0c9f
	v_lshlrev_b64 v[6:7], 2, v[6:7]                             ; d73c0006 00020c82
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_add_co_u32 v34, vcc, s0, v6                               ; d7006a22 00020c00
	v_add_co_ci_u32_e32 v35, vcc, v4, v7, vcc                   ; 40460f04
	v_add_co_u32 v36, vcc, s4, v6                               ; d7006a24 00020c04
	v_and_b32_e32 v6, v2, v38                                   ; 360c4d02
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a2
	v_add_co_ci_u32_e32 v37, vcc, v5, v7, vcc                   ; 404a0f05
	v_ashrrev_i32_e32 v7, 31, v6                                ; 340e0c9f
	v_lshlrev_b64 v[6:7], 2, v[6:7]                             ; d73c0006 00020c82
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_add_co_u32 v38, vcc, s0, v6                               ; d7006a26 00020c00
	v_add_co_ci_u32_e32 v39, vcc, v4, v7, vcc                   ; 404e0f04
	v_add_co_u32 v40, vcc, s4, v6                               ; d7006a28 00020c04
	v_and_b32_e32 v6, v2, v42                                   ; 360c5502
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a2
	v_add_co_ci_u32_e32 v41, vcc, v5, v7, vcc                   ; 40520f05
	v_ashrrev_i32_e32 v7, 31, v6                                ; 340e0c9f
	v_lshlrev_b64 v[6:7], 2, v[6:7]                             ; d73c0006 00020c82
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_add_co_u32 v42, vcc, s0, v6                               ; d7006a2a 00020c00
	v_add_co_ci_u32_e32 v43, vcc, v4, v7, vcc                   ; 40560f04
	v_add_co_u32 v44, vcc, s4, v6                               ; d7006a2c 00020c04
	s_delay_alu instid0(VALU_DEP_1)                             ; bf870001
	v_add_co_ci_u32_e32 v45, vcc, v5, v7, vcc                   ; 405a0f05
	s_clause 0x11                                               ; bf850011
	global_load_b32 v46, v[10:11], off                          ; dc520000 2e7c000a
	global_load_b32 v47, v[12:13], off                          ; dc520000 2f7c000c
	global_load_b32 v6, v[14:15], off                           ; dc520000 067c000e
	global_load_b32 v7, v[16:17], off                           ; dc520000 077c0010
	global_load_b32 v10, v[18:19], off                          ; dc520000 0a7c0012
	global_load_b32 v11, v[20:21], off                          ; dc520000 0b7c0014
	global_load_b32 v12, v[22:23], off                          ; dc520000 0c7c0016
	global_load_b32 v13, v[24:25], off                          ; dc520000 0d7c0018
	global_load_b32 v14, v[26:27], off                          ; dc520000 0e7c001a
	global_load_b32 v15, v[28:29], off                          ; dc520000 0f7c001c
	global_load_b32 v16, v[30:31], off                          ; dc520000 107c001e
	global_load_b32 v17, v[32:33], off                          ; dc520000 117c0020
	global_load_b32 v18, v[34:35], off                          ; dc520000 127c0022
	global_load_b32 v19, v[36:37], off                          ; dc520000 137c0024
	global_load_b32 v20, v[38:39], off                          ; dc520000 147c0026
	global_load_b32 v21, v[40:41], off                          ; dc520000 157c0028
	global_load_b32 v22, v[42:43], off                          ; dc520000 167c002a
	global_load_b32 v23, v[44:45], off                          ; dc520000 177c002c
	v_add_nc_u32_e32 v24, 9, v3                                 ; 4a300689
	v_add_nc_u32_e32 v34, 11, v3                                ; 4a44068b
	s_delay_alu instid0(VALU_DEP_2) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870092
	v_and_b32_e32 v24, v2, v24                                  ; 36303102
	v_ashrrev_i32_e32 v25, 31, v24                              ; 3432309f
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_lshlrev_b64 v[24:25], 2, v[24:25]                         ; d73c0018 00023082
	v_add_co_u32 v26, vcc, s0, v24                              ; d7006a1a 00023000
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_add_co_ci_u32_e32 v27, vcc, v4, v25, vcc                  ; 40363304
	v_add_co_u32 v28, vcc, s4, v24                              ; d7006a1c 00023004
	v_add_co_ci_u32_e32 v29, vcc, v5, v25, vcc                  ; 403a3305
	v_add_nc_u32_e32 v25, 10, v3                                ; 4a32068a
	v_add_nc_u32_e32 v38, 12, v3                                ; 4a4c068c
	s_delay_alu instid0(VALU_DEP_2) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870092
	v_and_b32_e32 v24, v2, v25                                  ; 36303302
	v_ashrrev_i32_e32 v25, 31, v24                              ; 3432309f
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_lshlrev_b64 v[24:25], 2, v[24:25]                         ; d73c0018 00023082
	v_add_co_u32 v30, vcc, s0, v24                              ; d7006a1e 00023000
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_3) | instid1(VALU_DEP_3) ; bf8701c1
	v_add_co_ci_u32_e32 v31, vcc, v4, v25, vcc                  ; 403e3304
	v_add_co_u32 v32, vcc, s4, v24                              ; d7006a20 00023004
	v_and_b32_e32 v24, v2, v34                                  ; 36304502
	v_add_nc_u32_e32 v42, 13, v3                                ; 4a54068d
	v_add_co_ci_u32_e32 v33, vcc, v5, v25, vcc                  ; 40423305
	v_ashrrev_i32_e32 v25, 31, v24                              ; 3432309f
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_lshlrev_b64 v[24:25], 2, v[24:25]                         ; d73c0018 00023082
	v_add_co_u32 v34, vcc, s0, v24                              ; d7006a22 00023000
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_2) | instid1(VALU_DEP_2) ; bf870131
	v_add_co_ci_u32_e32 v35, vcc, v4, v25, vcc                  ; 40463304
	v_add_co_u32 v36, vcc, s4, v24                              ; d7006a24 00023004
	v_and_b32_e32 v24, v2, v38                                  ; 36304d02
	v_add_co_ci_u32_e32 v37, vcc, v5, v25, vcc                  ; 404a3305
	v_ashrrev_i32_e32 v25, 31, v24                              ; 3432309f
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_lshlrev_b64 v[24:25], 2, v[24:25]                         ; d73c0018 00023082
	v_add_co_u32 v38, vcc, s0, v24                              ; d7006a26 00023000
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_2) | instid1(VALU_DEP_2) ; bf870131
	v_add_co_ci_u32_e32 v39, vcc, v4, v25, vcc                  ; 404e3304
	v_add_co_u32 v40, vcc, s4, v24                              ; d7006a28 00023004
	v_and_b32_e32 v24, v2, v42                                  ; 36305502
	v_add_co_ci_u32_e32 v41, vcc, v5, v25, vcc                  ; 40523305
	v_ashrrev_i32_e32 v25, 31, v24                              ; 3432309f
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_lshlrev_b64 v[24:25], 2, v[24:25]                         ; d73c0018 00023082
	v_add_co_u32 v42, vcc, s0, v24                              ; d7006a2a 00023000
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_add_co_ci_u32_e32 v43, vcc, v4, v25, vcc                  ; 40563304
	v_add_co_u32 v44, vcc, s4, v24                              ; d7006a2c 00023004
	v_add_co_ci_u32_e32 v45, vcc, v5, v25, vcc                  ; 405a3305
	s_clause 0x9                                                ; bf850009
	global_load_b32 v24, v[26:27], off                          ; dc520000 187c001a
	global_load_b32 v25, v[28:29], off                          ; dc520000 197c001c
	global_load_b32 v26, v[30:31], off                          ; dc520000 1a7c001e
	global_load_b32 v27, v[32:33], off                          ; dc520000 1b7c0020
	global_load_b32 v28, v[34:35], off                          ; dc520000 1c7c0022
	global_load_b32 v29, v[36:37], off                          ; dc520000 1d7c0024
	global_load_b32 v30, v[38:39], off                          ; dc520000 1e7c0026
	global_load_b32 v31, v[40:41], off                          ; dc520000 1f7c0028
	global_load_b32 v32, v[42:43], off                          ; dc520000 207c002a
	global_load_b32 v33, v[44:45], off                          ; dc520000 217c002c
	v_add_nc_u32_e32 v34, 14, v3                                ; 4a44068e
	v_add_nc_u32_e32 v3, 15, v3                                 ; 4a06068f
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_1) | instid1(VALU_DEP_2) ; bf870122
	v_and_b32_e32 v34, v2, v34                                  ; 36444502
	v_and_b32_e32 v2, v2, v3                                    ; 36040702
	v_ashrrev_i32_e32 v35, 31, v34                              ; 3446449f
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_lshlrev_b64 v[34:35], 2, v[34:35]                         ; d73c0022 00024482
	v_ashrrev_i32_e32 v3, 31, v2                                ; 3406049f
	v_lshlrev_b64 v[2:3], 2, v[2:3]                             ; d73c0002 00020482
	v_add_co_u32 v36, vcc, s0, v34                              ; d7006a24 00024400
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_add_co_ci_u32_e32 v37, vcc, v4, v35, vcc                  ; 404a4704
	v_add_co_u32 v38, vcc, s4, v34                              ; d7006a26 00024404
	v_add_co_ci_u32_e32 v39, vcc, v5, v35, vcc                  ; 404e4705
	v_add_co_u32 v34, vcc, s0, v2                               ; d7006a22 00020400
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_add_co_ci_u32_e32 v35, vcc, v4, v3, vcc                   ; 40460704
	v_add_co_u32 v40, vcc, s4, v2                               ; d7006a28 00020404
	v_add_co_ci_u32_e32 v41, vcc, v5, v3, vcc                   ; 40520705
	s_clause 0x3                                                ; bf850003
	global_load_b32 v36, v[36:37], off                          ; dc520000 247c0024
	global_load_b32 v37, v[38:39], off                          ; dc520000 257c0026
	global_load_b32 v38, v[34:35], off                          ; dc520000 267c0022
	global_load_b32 v39, v[40:41], off                          ; dc520000 277c0028
	s_add_u32 s7, s7, 1                                         ; 80078107
	s_waitcnt vmcnt(30)                                         ; bf897bf7
	v_dot4_i32_iu8 v46, v46, v47, v1 neg_lo:[1,1,0]             ; cc16402e 7c065f2e
	s_waitcnt vmcnt(28)                                         ; bf8973f7
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_dot4_i32_iu8 v6, v6, v7, v46 neg_lo:[1,1,0]               ; cc164006 7cba0f06
	s_waitcnt vmcnt(26)                                         ; bf896bf7
	v_dot4_i32_iu8 v10, v10, v11, v6 neg_lo:[1,1,0]             ; cc16400a 7c1a170a
	s_waitcnt vmcnt(24)                                         ; bf8963f7
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_dot4_i32_iu8 v12, v12, v13, v10 neg_lo:[1,1,0]            ; cc16400c 7c2a1b0c
	s_waitcnt vmcnt(22)                                         ; bf895bf7
	v_dot4_i32_iu8 v14, v14, v15, v12 neg_lo:[1,1,0]            ; cc16400e 7c321f0e
	s_waitcnt vmcnt(20)                                         ; bf8953f7
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_dot4_i32_iu8 v16, v16, v17, v14 neg_lo:[1,1,0]            ; cc164010 7c3a2310
	s_waitcnt vmcnt(18)                                         ; bf894bf7
	v_dot4_i32_iu8 v18, v18, v19, v16 neg_lo:[1,1,0]            ; cc164012 7c422712
	s_waitcnt vmcnt(16)                                         ; bf8943f7
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_dot4_i32_iu8 v20, v20, v21, v18 neg_lo:[1,1,0]            ; cc164014 7c4a2b14
	s_waitcnt vmcnt(14)                                         ; bf893bf7
	v_dot4_i32_iu8 v22, v22, v23, v20 neg_lo:[1,1,0]            ; cc164016 7c522f16
	s_waitcnt vmcnt(12)                                         ; bf8933f7
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_dot4_i32_iu8 v24, v24, v25, v22 neg_lo:[1,1,0]            ; cc164018 7c5a3318
	s_waitcnt vmcnt(10)                                         ; bf892bf7
	v_dot4_i32_iu8 v26, v26, v27, v24 neg_lo:[1,1,0]            ; cc16401a 7c62371a
	s_waitcnt vmcnt(8)                                          ; bf8923f7
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_dot4_i32_iu8 v28, v28, v29, v26 neg_lo:[1,1,0]            ; cc16401c 7c6a3b1c
	s_waitcnt vmcnt(6)                                          ; bf891bf7
	v_dot4_i32_iu8 v30, v30, v31, v28 neg_lo:[1,1,0]            ; cc16401e 7c723f1e
	s_waitcnt vmcnt(4)                                          ; bf8913f7
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_dot4_i32_iu8 v32, v32, v33, v30 neg_lo:[1,1,0]            ; cc164020 7c7a4320
	s_waitcnt vmcnt(2)                                          ; bf890bf7
	v_dot4_i32_iu8 v36, v36, v37, v32 neg_lo:[1,1,0]            ; cc164024 7c824b24
	s_waitcnt vmcnt(0)                                          ; bf8903f7
	s_delay_alu instid0(VALU_DEP_1)                             ; bf870001
	v_dot4_i32_iu8 v1, v38, v39, v36 neg_lo:[1,1,0]             ; cc164001 7c924f26
	s_branch BB2                                                ; bfa0fe9e
BB6:
	v_mov_b32_e32 v2, 0                                         ; 7e040280
	global_load_b32 v2, v2, s[2:3] offset:36                    ; dc520024 02020002
	v_ashrrev_i32_e32 v5, 31, v0                                ; 340a009f
	v_mov_b32_e32 v4, v0                                        ; 7e080300
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_lshlrev_b64 v[4:5], 2, v[4:5]                             ; d73c0004 00020882
	v_add_co_u32 v6, vcc, v8, v4                                ; d7006a06 00020908
	s_waitcnt_depctr 0xfffd                                     ; bf88fffd
	v_add_co_ci_u32_e32 v7, vcc, v9, v5, vcc                    ; 400e0b09
	s_waitcnt vmcnt(0)                                          ; bf8903f7
	v_add_nc_u32_e32 v2, v2, v1                                 ; 4a040302
	global_store_b32 v[6:7], v2, off                            ; dc6a0000 007c0206
BB11:
	s_endpgm                                                    ; bfb00000
 