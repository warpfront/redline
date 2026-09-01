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
	s_cbranch_execz BB11                                        ; bfa5016f
BB1:
	v_mov_b32_e32 v1, 0                                         ; 7e020280
	s_mov_b32 s7, 0                                             ; be870080
BB2:
	v_mov_b32_e32 v2, 0                                         ; 7e040280
	global_load_b32 v2, v2, s[2:3] offset:28                    ; dc52001c 02020002
	s_waitcnt vmcnt(0)                                          ; bf8903f7
	v_readfirstlane_b32 s8, v2                                  ; 7e100502
	s_cmp_ge_u32 s7, s8                                         ; bf090807
	s_cbranch_scc1 BB6                                          ; bfa20156
BB5:
	v_mov_b32_e32 v2, 0                                         ; 7e040280
	global_load_b32 v2, v2, s[2:3] offset:32                    ; dc520020 02020002
	s_mul_i32 s8, s7, s6                                        ; 96080607
	s_delay_alu instid0(SALU_CYCLE_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a9
	v_add_lshl_u32 v3, s8, v0, 4                                ; d6470003 02120008
	s_waitcnt vmcnt(0)                                          ; bf8903f7
	v_dual_mov_b32 v4, s1 :: v_dual_and_b32 v3, v2, v3          ; ca240001 04020702
	s_delay_alu instid0(VALU_DEP_1)                             ; bf870001
	v_mov_b32_e32 v6, v3                                        ; 7e0c0303
	v_ashrrev_i32_e32 v7, 31, v3                                ; 340e069f
	v_add_nc_u32_e32 v22, 3, v3                                 ; 4a2c0683
	v_add_nc_u32_e32 v26, 4, v3                                 ; 4a340684
	v_add_nc_u32_e32 v30, 5, v3                                 ; 4a3c0685
	v_add_nc_u32_e32 v34, 6, v3                                 ; 4a440686
	v_add_nc_u32_e32 v38, 7, v3                                 ; 4a4c0687
	v_add_nc_u32_e32 v42, 8, v3                                 ; 4a540688
	v_lshlrev_b64 v[6:7], 2, v[6:7]                             ; d73c0006 00020c82
	v_add_nc_u32_e32 v46, 9, v3                                 ; 4a5c0689
	v_add_nc_u32_e32 v50, 10, v3                                ; 4a64068a
	v_add_nc_u32_e32 v54, 11, v3                                ; 4a6c068b
	v_add_nc_u32_e32 v58, 12, v3                                ; 4a74068c
	v_add_nc_u32_e32 v62, 13, v3                                ; 4a7c068d
	v_add_nc_u32_e32 v66, 14, v3                                ; 4a84068e
	v_add_co_u32 v10, vcc_lo, s0, v6                            ; d7006a0a 00020c00
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_2) | instid1(VALU_DEP_1) ; bf8700b1
	v_add_co_ci_u32_e32 v11, vcc_lo, v4, v7, vcc_lo             ; 40160f04
	v_add_co_u32 v12, vcc_lo, s4, v6                            ; d7006a0c 00020c04
	v_add_nc_u32_e32 v6, 1, v3                                  ; 4a0c0681
	v_dual_mov_b32 v5, s5 :: v_dual_and_b32 v6, v2, v6          ; ca240005 05060d02
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_add_co_ci_u32_e32 v13, vcc_lo, v5, v7, vcc_lo             ; 401a0f05
	v_ashrrev_i32_e32 v7, 31, v6                                ; 340e0c9f
	v_lshlrev_b64 v[6:7], 2, v[6:7]                             ; d73c0006 00020c82
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_add_co_u32 v14, vcc_lo, s0, v6                            ; d7006a0e 00020c00
	v_add_co_ci_u32_e32 v15, vcc_lo, v4, v7, vcc_lo             ; 401e0f04
	v_add_co_u32 v16, vcc_lo, s4, v6                            ; d7006a10 00020c04
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_2) | instid1(VALU_DEP_2) ; bf870131
	v_add_co_ci_u32_e32 v17, vcc_lo, v5, v7, vcc_lo             ; 40220f05
	v_add_nc_u32_e32 v7, 2, v3                                  ; 4a0e0682
	v_add_nc_u32_e32 v3, 15, v3                                 ; 4a06068f
	v_and_b32_e32 v6, v2, v7                                    ; 360c0f02
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_ashrrev_i32_e32 v7, 31, v6                                ; 340e0c9f
	v_lshlrev_b64 v[6:7], 2, v[6:7]                             ; d73c0006 00020c82
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_add_co_u32 v18, vcc_lo, s0, v6                            ; d7006a12 00020c00
	v_add_co_ci_u32_e32 v19, vcc_lo, v4, v7, vcc_lo             ; 40260f04
	v_add_co_u32 v20, vcc_lo, s4, v6                            ; d7006a14 00020c04
	v_and_b32_e32 v6, v2, v22                                   ; 360c2d02
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a2
	v_add_co_ci_u32_e32 v21, vcc_lo, v5, v7, vcc_lo             ; 402a0f05
	v_ashrrev_i32_e32 v7, 31, v6                                ; 340e0c9f
	v_lshlrev_b64 v[6:7], 2, v[6:7]                             ; d73c0006 00020c82
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_add_co_u32 v22, vcc_lo, s0, v6                            ; d7006a16 00020c00
	v_add_co_ci_u32_e32 v23, vcc_lo, v4, v7, vcc_lo             ; 402e0f04
	v_add_co_u32 v24, vcc_lo, s4, v6                            ; d7006a18 00020c04
	v_and_b32_e32 v6, v2, v26                                   ; 360c3502
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a2
	v_add_co_ci_u32_e32 v25, vcc_lo, v5, v7, vcc_lo             ; 40320f05
	v_ashrrev_i32_e32 v7, 31, v6                                ; 340e0c9f
	v_lshlrev_b64 v[6:7], 2, v[6:7]                             ; d73c0006 00020c82
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_add_co_u32 v26, vcc_lo, s0, v6                            ; d7006a1a 00020c00
	v_add_co_ci_u32_e32 v27, vcc_lo, v4, v7, vcc_lo             ; 40360f04
	v_add_co_u32 v28, vcc_lo, s4, v6                            ; d7006a1c 00020c04
	v_and_b32_e32 v6, v2, v30                                   ; 360c3d02
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a2
	v_add_co_ci_u32_e32 v29, vcc_lo, v5, v7, vcc_lo             ; 403a0f05
	v_ashrrev_i32_e32 v7, 31, v6                                ; 340e0c9f
	v_lshlrev_b64 v[6:7], 2, v[6:7]                             ; d73c0006 00020c82
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_add_co_u32 v30, vcc_lo, s0, v6                            ; d7006a1e 00020c00
	v_add_co_ci_u32_e32 v31, vcc_lo, v4, v7, vcc_lo             ; 403e0f04
	v_add_co_u32 v32, vcc_lo, s4, v6                            ; d7006a20 00020c04
	v_and_b32_e32 v6, v2, v34                                   ; 360c4502
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a2
	v_add_co_ci_u32_e32 v33, vcc_lo, v5, v7, vcc_lo             ; 40420f05
	v_ashrrev_i32_e32 v7, 31, v6                                ; 340e0c9f
	v_lshlrev_b64 v[6:7], 2, v[6:7]                             ; d73c0006 00020c82
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_add_co_u32 v34, vcc_lo, s0, v6                            ; d7006a22 00020c00
	v_add_co_ci_u32_e32 v35, vcc_lo, v4, v7, vcc_lo             ; 40460f04
	v_add_co_u32 v36, vcc_lo, s4, v6                            ; d7006a24 00020c04
	v_and_b32_e32 v6, v2, v38                                   ; 360c4d02
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a2
	v_add_co_ci_u32_e32 v37, vcc_lo, v5, v7, vcc_lo             ; 404a0f05
	v_ashrrev_i32_e32 v7, 31, v6                                ; 340e0c9f
	v_lshlrev_b64 v[6:7], 2, v[6:7]                             ; d73c0006 00020c82
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_add_co_u32 v38, vcc_lo, s0, v6                            ; d7006a26 00020c00
	v_add_co_ci_u32_e32 v39, vcc_lo, v4, v7, vcc_lo             ; 404e0f04
	v_add_co_u32 v40, vcc_lo, s4, v6                            ; d7006a28 00020c04
	v_and_b32_e32 v6, v2, v42                                   ; 360c5502
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a2
	v_add_co_ci_u32_e32 v41, vcc_lo, v5, v7, vcc_lo             ; 40520f05
	v_ashrrev_i32_e32 v7, 31, v6                                ; 340e0c9f
	v_lshlrev_b64 v[6:7], 2, v[6:7]                             ; d73c0006 00020c82
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_add_co_u32 v42, vcc_lo, s0, v6                            ; d7006a2a 00020c00
	v_add_co_ci_u32_e32 v43, vcc_lo, v4, v7, vcc_lo             ; 40560f04
	v_add_co_u32 v44, vcc_lo, s4, v6                            ; d7006a2c 00020c04
	v_and_b32_e32 v6, v2, v46                                   ; 360c5d02
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a2
	v_add_co_ci_u32_e32 v45, vcc_lo, v5, v7, vcc_lo             ; 405a0f05
	v_ashrrev_i32_e32 v7, 31, v6                                ; 340e0c9f
	v_lshlrev_b64 v[6:7], 2, v[6:7]                             ; d73c0006 00020c82
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_add_co_u32 v46, vcc_lo, s0, v6                            ; d7006a2e 00020c00
	v_add_co_ci_u32_e32 v47, vcc_lo, v4, v7, vcc_lo             ; 405e0f04
	v_add_co_u32 v48, vcc_lo, s4, v6                            ; d7006a30 00020c04
	v_and_b32_e32 v6, v2, v50                                   ; 360c6502
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a2
	v_add_co_ci_u32_e32 v49, vcc_lo, v5, v7, vcc_lo             ; 40620f05
	v_ashrrev_i32_e32 v7, 31, v6                                ; 340e0c9f
	v_lshlrev_b64 v[6:7], 2, v[6:7]                             ; d73c0006 00020c82
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_add_co_u32 v50, vcc_lo, s0, v6                            ; d7006a32 00020c00
	v_add_co_ci_u32_e32 v51, vcc_lo, v4, v7, vcc_lo             ; 40660f04
	v_add_co_u32 v52, vcc_lo, s4, v6                            ; d7006a34 00020c04
	v_and_b32_e32 v6, v2, v54                                   ; 360c6d02
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a2
	v_add_co_ci_u32_e32 v53, vcc_lo, v5, v7, vcc_lo             ; 406a0f05
	v_ashrrev_i32_e32 v7, 31, v6                                ; 340e0c9f
	v_lshlrev_b64 v[6:7], 2, v[6:7]                             ; d73c0006 00020c82
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_add_co_u32 v54, vcc_lo, s0, v6                            ; d7006a36 00020c00
	v_add_co_ci_u32_e32 v55, vcc_lo, v4, v7, vcc_lo             ; 406e0f04
	v_add_co_u32 v56, vcc_lo, s4, v6                            ; d7006a38 00020c04
	v_and_b32_e32 v6, v2, v58                                   ; 360c7502
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a2
	v_add_co_ci_u32_e32 v57, vcc_lo, v5, v7, vcc_lo             ; 40720f05
	v_ashrrev_i32_e32 v7, 31, v6                                ; 340e0c9f
	v_lshlrev_b64 v[6:7], 2, v[6:7]                             ; d73c0006 00020c82
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_add_co_u32 v58, vcc_lo, s0, v6                            ; d7006a3a 00020c00
	v_add_co_ci_u32_e32 v59, vcc_lo, v4, v7, vcc_lo             ; 40760f04
	v_add_co_u32 v60, vcc_lo, s4, v6                            ; d7006a3c 00020c04
	v_and_b32_e32 v6, v2, v62                                   ; 360c7d02
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a2
	v_add_co_ci_u32_e32 v61, vcc_lo, v5, v7, vcc_lo             ; 407a0f05
	v_ashrrev_i32_e32 v7, 31, v6                                ; 340e0c9f
	v_lshlrev_b64 v[6:7], 2, v[6:7]                             ; d73c0006 00020c82
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_add_co_u32 v62, vcc_lo, s0, v6                            ; d7006a3e 00020c00
	v_add_co_ci_u32_e32 v63, vcc_lo, v4, v7, vcc_lo             ; 407e0f04
	v_add_co_u32 v64, vcc_lo, s4, v6                            ; d7006a40 00020c04
	v_and_b32_e32 v6, v2, v66                                   ; 360c8502
	v_and_b32_e32 v2, v2, v3                                    ; 36040702
	s_delay_alu instid0(VALU_DEP_3) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a3
	v_add_co_ci_u32_e32 v65, vcc_lo, v5, v7, vcc_lo             ; 40820f05
	v_ashrrev_i32_e32 v7, 31, v6                                ; 340e0c9f
	v_lshlrev_b64 v[6:7], 2, v[6:7]                             ; d73c0006 00020c82
	v_ashrrev_i32_e32 v3, 31, v2                                ; 3406049f
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_lshlrev_b64 v[2:3], 2, v[2:3]                             ; d73c0002 00020482
	v_add_co_u32 v66, vcc_lo, s0, v6                            ; d7006a42 00020c00
	v_add_co_ci_u32_e32 v67, vcc_lo, v4, v7, vcc_lo             ; 40860f04
	v_add_co_u32 v68, vcc_lo, s4, v6                            ; d7006a44 00020c04
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_add_co_ci_u32_e32 v69, vcc_lo, v5, v7, vcc_lo             ; 408a0f05
	v_add_co_u32 v6, vcc_lo, s0, v2                             ; d7006a06 00020400
	v_add_co_ci_u32_e32 v7, vcc_lo, v4, v3, vcc_lo              ; 400e0704
	v_add_co_u32 v70, vcc_lo, s4, v2                            ; d7006a46 00020404
	s_delay_alu instid0(VALU_DEP_1)                             ; bf870001
	v_add_co_ci_u32_e32 v71, vcc_lo, v5, v3, vcc_lo             ; 408e0705
	s_clause 0x1f                                               ; bf85001f
	global_load_b32 v72, v[10:11], off                          ; dc520000 487c000a
	global_load_b32 v73, v[12:13], off                          ; dc520000 497c000c
	global_load_b32 v74, v[14:15], off                          ; dc520000 4a7c000e
	global_load_b32 v75, v[16:17], off                          ; dc520000 4b7c0010
	global_load_b32 v76, v[18:19], off                          ; dc520000 4c7c0012
	global_load_b32 v77, v[20:21], off                          ; dc520000 4d7c0014
	global_load_b32 v78, v[22:23], off                          ; dc520000 4e7c0016
	global_load_b32 v79, v[24:25], off                          ; dc520000 4f7c0018
	global_load_b32 v80, v[26:27], off                          ; dc520000 507c001a
	global_load_b32 v81, v[28:29], off                          ; dc520000 517c001c
	global_load_b32 v82, v[30:31], off                          ; dc520000 527c001e
	global_load_b32 v83, v[32:33], off                          ; dc520000 537c0020
	global_load_b32 v84, v[34:35], off                          ; dc520000 547c0022
	global_load_b32 v85, v[36:37], off                          ; dc520000 557c0024
	global_load_b32 v86, v[38:39], off                          ; dc520000 567c0026
	global_load_b32 v87, v[40:41], off                          ; dc520000 577c0028
	global_load_b32 v88, v[42:43], off                          ; dc520000 587c002a
	global_load_b32 v89, v[44:45], off                          ; dc520000 597c002c
	global_load_b32 v90, v[46:47], off                          ; dc520000 5a7c002e
	global_load_b32 v91, v[48:49], off                          ; dc520000 5b7c0030
	global_load_b32 v92, v[50:51], off                          ; dc520000 5c7c0032
	global_load_b32 v93, v[52:53], off                          ; dc520000 5d7c0034
	global_load_b32 v94, v[54:55], off                          ; dc520000 5e7c0036
	global_load_b32 v95, v[56:57], off                          ; dc520000 5f7c0038
	global_load_b32 v2, v[58:59], off                           ; dc520000 027c003a
	global_load_b32 v3, v[60:61], off                           ; dc520000 037c003c
	global_load_b32 v4, v[62:63], off                           ; dc520000 047c003e
	global_load_b32 v5, v[64:65], off                           ; dc520000 057c0040
	global_load_b32 v10, v[66:67], off                          ; dc520000 0a7c0042
	global_load_b32 v11, v[68:69], off                          ; dc520000 0b7c0044
	global_load_b32 v12, v[6:7], off                            ; dc520000 0c7c0006
	global_load_b32 v13, v[70:71], off                          ; dc520000 0d7c0046
	s_add_u32 s7, s7, 1                                         ; 80078107
	s_waitcnt vmcnt(30)                                         ; bf897bf7
	v_dot4_i32_iu8 v72, v72, v73, v1 neg_lo:[1,1,0]             ; cc164048 7c069348
	s_waitcnt vmcnt(28)                                         ; bf8973f7
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_dot4_i32_iu8 v74, v74, v75, v72 neg_lo:[1,1,0]            ; cc16404a 7d22974a
	s_waitcnt vmcnt(26)                                         ; bf896bf7
	v_dot4_i32_iu8 v76, v76, v77, v74 neg_lo:[1,1,0]            ; cc16404c 7d2a9b4c
	s_waitcnt vmcnt(24)                                         ; bf8963f7
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_dot4_i32_iu8 v78, v78, v79, v76 neg_lo:[1,1,0]            ; cc16404e 7d329f4e
	s_waitcnt vmcnt(22)                                         ; bf895bf7
	v_dot4_i32_iu8 v80, v80, v81, v78 neg_lo:[1,1,0]            ; cc164050 7d3aa350
	s_waitcnt vmcnt(20)                                         ; bf8953f7
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_dot4_i32_iu8 v82, v82, v83, v80 neg_lo:[1,1,0]            ; cc164052 7d42a752
	s_waitcnt vmcnt(18)                                         ; bf894bf7
	v_dot4_i32_iu8 v84, v84, v85, v82 neg_lo:[1,1,0]            ; cc164054 7d4aab54
	s_waitcnt vmcnt(16)                                         ; bf8943f7
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_dot4_i32_iu8 v86, v86, v87, v84 neg_lo:[1,1,0]            ; cc164056 7d52af56
	s_waitcnt vmcnt(14)                                         ; bf893bf7
	v_dot4_i32_iu8 v88, v88, v89, v86 neg_lo:[1,1,0]            ; cc164058 7d5ab358
	s_waitcnt vmcnt(12)                                         ; bf8933f7
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_dot4_i32_iu8 v90, v90, v91, v88 neg_lo:[1,1,0]            ; cc16405a 7d62b75a
	s_waitcnt vmcnt(10)                                         ; bf892bf7
	v_dot4_i32_iu8 v92, v92, v93, v90 neg_lo:[1,1,0]            ; cc16405c 7d6abb5c
	s_waitcnt vmcnt(8)                                          ; bf8923f7
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_dot4_i32_iu8 v94, v94, v95, v92 neg_lo:[1,1,0]            ; cc16405e 7d72bf5e
	s_waitcnt vmcnt(6)                                          ; bf891bf7
	v_dot4_i32_iu8 v2, v2, v3, v94 neg_lo:[1,1,0]               ; cc164002 7d7a0702
	s_waitcnt vmcnt(4)                                          ; bf8913f7
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_dot4_i32_iu8 v4, v4, v5, v2 neg_lo:[1,1,0]                ; cc164004 7c0a0b04
	s_waitcnt vmcnt(2)                                          ; bf890bf7
	v_dot4_i32_iu8 v10, v10, v11, v4 neg_lo:[1,1,0]             ; cc16400a 7c12170a
	s_waitcnt vmcnt(0)                                          ; bf8903f7
	s_delay_alu instid0(VALU_DEP_1)                             ; bf870001
	v_dot4_i32_iu8 v1, v12, v13, v10 neg_lo:[1,1,0]             ; cc164001 7c2a1b0c
	s_branch BB2                                                ; bfa0fea3
BB6:
	v_mov_b32_e32 v2, 0                                         ; 7e040280
	global_load_b32 v2, v2, s[2:3] offset:36                    ; dc520024 02020002
	v_mov_b32_e32 v4, v0                                        ; 7e080300
	v_ashrrev_i32_e32 v5, 31, v0                                ; 340a009f
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_lshlrev_b64 v[4:5], 2, v[4:5]                             ; d73c0004 00020882
	v_add_co_u32 v6, vcc_lo, v8, v4                             ; d7006a06 00020908
	s_delay_alu instid0(VALU_DEP_1)                             ; bf870001
	v_add_co_ci_u32_e32 v7, vcc_lo, v9, v5, vcc_lo              ; 400e0b09
	s_waitcnt vmcnt(0)                                          ; bf8903f7
	v_add_nc_u32_e32 v2, v2, v1                                 ; 4a040302
	global_store_b32 v[6:7], v2, off                            ; dc6a0000 007c0206
BB11:
	s_endpgm                                                    ; bfb00000
 