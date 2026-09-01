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
	s_cbranch_execz BB11                                        ; bfa5032f
BB1:
	v_mov_b32_e32 v1, 0                                         ; 7e020280
	s_mov_b32 s7, 0                                             ; be870080
BB2:
	v_mov_b32_e32 v2, 0                                         ; 7e040280
	global_load_b32 v2, v2, s[2:3] offset:28                    ; dc52001c 02020002
	s_waitcnt vmcnt(0)                                          ; bf8903f7
	v_readfirstlane_b32 s8, v2                                  ; 7e100502
	s_cmp_ge_u32 s7, s8                                         ; bf090807
	s_cbranch_scc1 BB6                                          ; bfa20313
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
	s_movk_i32 s9, 0xff00                                       ; b009ff00
	s_movk_i32 s10, 0x80                                        ; b00a0080
	s_waitcnt vmcnt(7)                                          ; bf891ff7
	v_and_b32_e32 v41, 15, v25                                  ; 3652328f
	v_bfe_u32 v40, v24, 16, 4                                   ; d6100028 02112118
	v_bfe_u32 v43, v25, 16, 4                                   ; d610002b 02112119
	v_bfe_u32 v42, v25, 8, 4                                    ; d610002a 02111119
	v_and_b32_e32 v10, 15, v24                                  ; 3614308f
	v_bfe_u32 v11, v24, 8, 4                                    ; d610000b 02111118
	v_bfe_u32 v24, v24, 24, 4                                   ; d6100018 02113118
	v_bfe_u32 v25, v25, 24, 4                                   ; d6100019 02113119
	v_and_b32_e32 v44, 15, v26                                  ; 3658348f
	v_bfe_u32 v45, v26, 8, 4                                    ; d610002d 0211111a
	v_bfe_u32 v46, v26, 16, 4                                   ; d610002e 0211211a
	v_add_nc_u32_e32 v41, -8, v41                               ; 4a5252c8
	v_add_nc_u32_e32 v40, -8, v40                               ; 4a5050c8
	v_bfe_u32 v26, v26, 24, 4                                   ; d610001a 0211311a
	v_add_nc_u32_e32 v43, -8, v43                               ; 4a5656c8
	v_and_b32_e32 v47, 15, v27                                  ; 365e368f
	v_add_nc_u32_e32 v42, -8, v42                               ; 4a5454c8
	v_add_nc_u32_e32 v10, -8, v10                               ; 4a1414c8
	v_bfe_u32 v48, v27, 8, 4                                    ; d6100030 0211111b
	v_add_nc_u32_e32 v11, -8, v11                               ; 4a1616c8
	v_add_nc_u32_e32 v24, -8, v24                               ; 4a3030c8
	v_add_nc_u32_e32 v25, -8, v25                               ; 4a3232c8
	s_waitcnt vmcnt(6)                                          ; bf891bf7
	v_bfe_u32 v50, v28, 0, 8                                    ; d6100032 0221011c
	v_add_nc_u32_e32 v44, -8, v44                               ; 4a5858c8
	v_add_nc_u32_e32 v45, -8, v45                               ; 4a5a5ac8
	v_bfe_u32 v52, v28, 8, 8                                    ; d6100034 0221111c
	v_add_nc_u32_e32 v46, -8, v46                               ; 4a5c5cc8
	v_bfe_u32 v54, v28, 16, 8                                   ; d6100036 0221211c
	v_add_nc_u32_e32 v26, -8, v26                               ; 4a3434c8
	v_bfe_u32 v49, v27, 16, 4                                   ; d6100031 0211211b
	v_add_nc_u32_e32 v47, -8, v47                               ; 4a5e5ec8
	v_lshrrev_b32_e32 v28, 24, v28                              ; 32383898
	v_add_nc_u32_e32 v48, -8, v48                               ; 4a6060c8
	v_bfe_u32 v27, v27, 24, 4                                   ; d610001b 0211311b
	v_add_nc_u32_e32 v51, s9, v50                               ; 4a666409
	v_cmp_le_i32_e32 vcc_lo, s10, v50                           ; 7c86640a
	v_bfe_u32 v57, v29, 0, 8                                    ; d6100039 0221011d
	v_bfe_u32 v59, v29, 8, 8                                    ; d610003b 0221111d
	v_add_nc_u32_e32 v56, s9, v28                               ; 4a703809
	v_bfe_u32 v61, v29, 16, 8                                   ; d610003d 0221211d
	v_dual_cndmask_b32 v50, v50, v51 :: v_dual_add_nc_u32 v53, s9, v52 ; ca606732 32346809
	v_cmp_le_i32_e32 vcc_lo, s10, v52                           ; 7c86680a
	v_add_nc_u32_e32 v58, s9, v57                               ; 4a747209
	v_lshrrev_b32_e32 v29, 24, v29                              ; 323a3a98
	s_delay_alu instid0(VALU_DEP_4) | instskip(SKIP_3) | instid1(VALU_DEP_3) ; bf8701c4
	v_mul_lo_u32 v10, v10, v50                                  ; d72c000a 0002650a
	v_dual_cndmask_b32 v52, v52, v53 :: v_dual_add_nc_u32 v55, s9, v54 ; ca606b34 34366c09
	v_cmp_le_i32_e32 vcc_lo, s10, v54                           ; 7c866c0a
	v_add_nc_u32_e32 v63, s9, v29                               ; 4a7e3a09
	v_mul_lo_u32 v11, v11, v52                                  ; d72c000b 0002690b
	v_dual_cndmask_b32 v54, v54, v55 :: v_dual_add_nc_u32 v49, -8, v49 ; ca606f36 363062c8
	v_cmp_le_i32_e32 vcc_lo, s10, v28                           ; 7c86380a
	v_bfe_u32 v64, v30, 0, 8                                    ; d6100040 0221011e
	s_delay_alu instid0(VALU_DEP_3)                             ; bf870003
	v_mul_lo_u32 v40, v40, v54                                  ; d72c0028 00026d28
	v_dual_cndmask_b32 v28, v28, v56 :: v_dual_add_nc_u32 v27, -8, v27 ; ca60711c 1c1a36c8
	v_cmp_le_i32_e32 vcc_lo, s10, v57                           ; 7c86720a
	v_add_nc_u32_e32 v65, s9, v64                               ; 4a828009
	v_bfe_u32 v66, v30, 8, 8                                    ; d6100042 0221111e
	v_bfe_u32 v68, v30, 16, 8                                   ; d6100044 0221211e
	v_mul_lo_u32 v24, v24, v28                                  ; d72c0018 00023918
	v_dual_cndmask_b32 v57, v57, v58 :: v_dual_add_nc_u32 v60, s9, v59 ; ca607539 393c7609
	v_cmp_le_i32_e32 vcc_lo, s10, v59                           ; 7c86760a
	v_add3_u32 v40, v40, v10, v11                               ; d6550028 042e1528
	s_delay_alu instid0(VALU_DEP_3) | instskip(SKIP_4) | instid1(VALU_DEP_4) ; bf870253
	v_mul_lo_u32 v41, v41, v57                                  ; d72c0029 00027329
	v_dual_cndmask_b32 v59, v59, v60 :: v_dual_add_nc_u32 v62, s9, v61 ; ca60793b 3b3e7a09
	v_cmp_le_i32_e32 vcc_lo, s10, v61                           ; 7c867a0a
	v_add3_u32 v1, v1, v40, v24                                 ; d6550001 04625101
	v_lshrrev_b32_e32 v30, 24, v30                              ; 323c3c98
	v_mul_lo_u32 v42, v42, v59                                  ; d72c002a 0002772a
	v_cndmask_b32_e32 v61, v61, v62, vcc_lo                     ; 027a7d3d
	v_cmp_le_i32_e32 vcc_lo, s10, v29                           ; 7c863a0a
	v_add_nc_u32_e32 v70, s9, v30                               ; 4a8c3c09
	v_bfe_u32 v71, v31, 0, 8                                    ; d6100047 0221011f
	s_delay_alu instid0(VALU_DEP_4)                             ; bf870004
	v_mul_lo_u32 v43, v43, v61                                  ; d72c002b 00027b2b
	v_cndmask_b32_e32 v29, v29, v63, vcc_lo                     ; 023a7f1d
	v_cmp_le_i32_e32 vcc_lo, s10, v64                           ; 7c86800a
	v_add_nc_u32_e32 v72, s9, v71                               ; 4a908e09
	v_bfe_u32 v73, v31, 8, 8                                    ; d6100049 0221111f
	v_bfe_u32 v75, v31, 16, 8                                   ; d610004b 0221211f
	v_mul_lo_u32 v25, v25, v29                                  ; d72c0019 00023b19
	v_dual_cndmask_b32 v64, v64, v65 :: v_dual_add_nc_u32 v67, s9, v66 ; ca608340 40428409
	v_cmp_le_i32_e32 vcc_lo, s10, v66                           ; 7c86840a
	v_add3_u32 v43, v43, v41, v42                               ; d655002b 04aa532b
	s_delay_alu instid0(VALU_DEP_3) | instskip(SKIP_4) | instid1(VALU_DEP_4) ; bf870253
	v_mul_lo_u32 v44, v44, v64                                  ; d72c002c 0002812c
	v_dual_cndmask_b32 v66, v66, v67 :: v_dual_add_nc_u32 v69, s9, v68 ; ca608742 42448809
	v_cmp_le_i32_e32 vcc_lo, s10, v68                           ; 7c86880a
	v_add3_u32 v1, v1, v43, v25                                 ; d6550001 04665701
	v_lshrrev_b32_e32 v31, 24, v31                              ; 323e3e98
	v_mul_lo_u32 v45, v45, v66                                  ; d72c002d 0002852d
	v_cndmask_b32_e32 v68, v68, v69, vcc_lo                     ; 02888b44
	v_cmp_le_i32_e32 vcc_lo, s10, v30                           ; 7c863c0a
	v_add_nc_u32_e32 v77, s9, v31                               ; 4a9a3e09
	s_waitcnt vmcnt(4)                                          ; bf8913f7
	v_bfe_u32 v78, v32, 0, 8                                    ; d610004e 02210120
	s_delay_alu instid0(VALU_DEP_4)                             ; bf870004
	v_mul_lo_u32 v46, v46, v68                                  ; d72c002e 0002892e
	v_cndmask_b32_e32 v30, v30, v70, vcc_lo                     ; 023c8d1e
	v_cmp_le_i32_e32 vcc_lo, s10, v71                           ; 7c868e0a
	v_add_nc_u32_e32 v79, s9, v78                               ; 4a9e9c09
	v_and_b32_e32 v80, 15, v12                                  ; 36a0188f
	v_bfe_u32 v81, v32, 8, 8                                    ; d6100051 02211120
	v_mul_lo_u32 v26, v26, v30                                  ; d72c001a 00023d1a
	v_dual_cndmask_b32 v71, v71, v72 :: v_dual_add_nc_u32 v74, s9, v73 ; ca609147 474a9209
	v_cmp_le_i32_e32 vcc_lo, s10, v73                           ; 7c86920a
	v_add3_u32 v46, v46, v44, v45                               ; d655002e 04b6592e
	v_add_nc_u32_e32 v82, s9, v81                               ; 4aa4a209
	v_bfe_u32 v83, v12, 8, 4                                    ; d6100053 0211110c
	v_bfe_u32 v84, v32, 16, 8                                   ; d6100054 02212120
	v_mul_lo_u32 v47, v47, v71                                  ; d72c002f 00028f2f
	v_dual_cndmask_b32 v73, v73, v74 :: v_dual_add_nc_u32 v76, s9, v75 ; ca609549 494c9609
	v_cmp_le_i32_e32 vcc_lo, s10, v75                           ; 7c86960a
	v_add3_u32 v1, v1, v46, v26                                 ; d6550001 046a5d01
	v_add_nc_u32_e32 v83, -8, v83                               ; 4aa6a6c8
	v_add_nc_u32_e32 v85, s9, v84                               ; 4aaaa809
	v_lshrrev_b32_e32 v32, 24, v32                              ; 32404098
	v_bfe_u32 v86, v12, 16, 4                                   ; d6100056 0211210c
	v_mul_lo_u32 v48, v48, v73                                  ; d72c0030 00029330
	v_cndmask_b32_e32 v75, v75, v76, vcc_lo                     ; 0296994b
	v_cmp_le_i32_e32 vcc_lo, s10, v31                           ; 7c863e0a
	v_bfe_u32 v12, v12, 24, 4                                   ; d610000c 0211310c
	v_add_nc_u32_e32 v86, -8, v86                               ; 4aacacc8
	v_bfe_u32 v88, v33, 0, 8                                    ; d6100058 02210121
	v_mul_lo_u32 v49, v49, v75                                  ; d72c0031 00029731
	v_cndmask_b32_e32 v31, v31, v77, vcc_lo                     ; 023e9b1f
	v_cmp_le_i32_e32 vcc_lo, s10, v78                           ; 7c869c0a
	v_add_nc_u32_e32 v12, -8, v12                               ; 4a1818c8
	v_bfe_u32 v94, v33, 16, 8                                   ; d610005e 02212121
	v_bfe_u32 v91, v33, 8, 8                                    ; d610005b 02211121
	v_mul_lo_u32 v27, v27, v31                                  ; d72c001b 00023f1b
	v_cndmask_b32_e32 v78, v78, v79, vcc_lo                     ; 029c9f4e
	v_cmp_le_i32_e32 vcc_lo, s10, v81                           ; 7c86a20a
	v_add3_u32 v49, v49, v47, v48                               ; d6550031 04c25f31
	v_add_nc_u32_e32 v92, s9, v91                               ; 4ab8b609
	v_bfe_u32 v93, v13, 8, 4                                    ; d610005d 0211110d
	v_dual_cndmask_b32 v81, v81, v82 :: v_dual_add_nc_u32 v80, -8, v80 ; ca60a551 5150a0c8
	v_cmp_le_i32_e32 vcc_lo, s10, v84                           ; 7c86a80a
	v_add3_u32 v1, v1, v49, v27                                 ; d6550001 046e6301
	s_delay_alu instid0(VALU_DEP_3) | instskip(SKIP_4) | instid1(VALU_DEP_3) ; bf8701d3
	v_mul_lo_u32 v83, v83, v81                                  ; d72c0053 0002a353
	v_mul_lo_u32 v80, v80, v78                                  ; d72c0050 00029d50
	v_dual_cndmask_b32 v84, v84, v85 :: v_dual_add_nc_u32 v87, s9, v32 ; ca60ab54 54564009
	v_cmp_le_i32_e32 vcc_lo, s10, v32                           ; 7c86400a
	v_lshrrev_b32_e32 v33, 24, v33                              ; 32424298
	v_mul_lo_u32 v86, v86, v84                                  ; d72c0056 0002a956
	v_dual_cndmask_b32 v32, v32, v87 :: v_dual_add_nc_u32 v89, s9, v88 ; ca60af20 2058b009
	v_cmp_le_i32_e32 vcc_lo, s10, v88                           ; 7c86b00a
	v_add_nc_u32_e32 v2, s9, v33                                ; 4a044209
	v_bfe_u32 v3, v34, 0, 8                                     ; d6100003 02210122
	s_delay_alu instid0(VALU_DEP_4)                             ; bf870004
	v_mul_lo_u32 v12, v12, v32                                  ; d72c000c 0002410c
	v_dual_cndmask_b32 v88, v88, v89 :: v_dual_add_nc_u32 v95, s9, v94 ; ca60b358 585ebc09
	v_cmp_le_i32_e32 vcc_lo, s10, v91                           ; 7c86b60a
	v_add3_u32 v86, v86, v80, v83                               ; d6550056 054ea156
	v_bfe_u32 v25, v34, 16, 8                                   ; d6100019 02212122
	v_and_b32_e32 v11, 15, v14                                  ; 36161c8f
	v_dual_cndmask_b32 v91, v91, v92 :: v_dual_and_b32 v90, 15, v13 ; ca64b95b 5b5a1a8f
	v_cmp_le_i32_e32 vcc_lo, s10, v94                           ; 7c86bc0a
	v_add3_u32 v1, v1, v86, v12                                 ; d6550001 0432ad01
	s_delay_alu instid0(VALU_DEP_3)                             ; bf870003
	v_add_nc_u32_e32 v90, -8, v90                               ; 4ab4b4c8
	v_dual_cndmask_b32 v94, v94, v95 :: v_dual_add_nc_u32 v93, -8, v93 ; ca60bf5e 5e5cbac8
	v_bfe_u32 v95, v13, 16, 4                                   ; d610005f 0211210d
	v_cmp_le_i32_e32 vcc_lo, s10, v33                           ; 7c86420a
	v_bfe_u32 v13, v13, 24, 4                                   ; d610000d 0211310d
	v_bfe_u32 v12, v34, 8, 8                                    ; d610000c 02211122
	v_lshrrev_b32_e32 v34, 24, v34                              ; 32444498
	v_mul_lo_u32 v90, v90, v88                                  ; d72c005a 0002b15a
	v_mul_lo_u32 v93, v93, v91                                  ; d72c005d 0002b75d
	v_add_nc_u32_e32 v95, -8, v95                               ; 4abebec8
	v_dual_cndmask_b32 v33, v33, v2 :: v_dual_add_nc_u32 v10, s9, v3 ; ca600521 210a0609
	v_cmp_le_i32_e32 vcc_lo, s10, v3                            ; 7c86060a
	v_add_nc_u32_e32 v13, -8, v13                               ; 4a1a1ac8
	v_add_nc_u32_e32 v28, s9, v34                               ; 4a384409
	v_bfe_u32 v27, v14, 16, 4                                   ; d610001b 0211210e
	v_bfe_u32 v29, v35, 0, 8                                    ; d610001d 02210123
	v_mul_lo_u32 v95, v95, v94                                  ; d72c005f 0002bd5f
	v_bfe_u32 v24, v14, 8, 4                                    ; d6100018 0211110e
	v_dual_cndmask_b32 v3, v3, v10 :: v_dual_add_nc_u32 v26, s9, v25 ; ca601503 031a3209
	v_cmp_le_i32_e32 vcc_lo, s10, v12                           ; 7c86180a
	v_mul_lo_u32 v13, v13, v33                                  ; d72c000d 0002430d
	v_add_nc_u32_e32 v27, -8, v27                               ; 4a3636c8
	v_add_nc_u32_e32 v30, s9, v29                               ; 4a3c3a09
	v_bfe_u32 v14, v14, 24, 4                                   ; d610000e 0211310e
	v_add3_u32 v95, v95, v90, v93                               ; d655005f 0576b55f
	v_bfe_u32 v32, v35, 8, 8                                    ; d6100020 02211123
	s_delay_alu instid0(VALU_DEP_3) | instskip(SKIP_4) | instid1(VALU_DEP_3) ; bf8701d3
	v_add_nc_u32_e32 v14, -8, v14                               ; 4a1c1cc8
	v_add3_u32 v1, v1, v95, v13                                 ; d6550001 0436bf01
	v_add_nc_u32_e32 v13, s9, v12                               ; 4a1a1809
	v_add_nc_u32_e32 v33, s9, v32                               ; 4a424009
	v_bfe_u32 v42, v15, 16, 4                                   ; d610002a 0211210f
	v_dual_cndmask_b32 v12, v12, v13 :: v_dual_add_nc_u32 v11, -8, v11 ; ca601b0c 0c0a16c8
	v_cmp_le_i32_e32 vcc_lo, s10, v25                           ; 7c86320a
	v_bfe_u32 v40, v35, 16, 8                                   ; d6100028 02212123
	v_add_nc_u32_e32 v42, -8, v42                               ; 4a5454c8
	s_delay_alu instid0(VALU_DEP_4)                             ; bf870004
	v_mul_lo_u32 v11, v11, v3                                   ; d72c000b 0002070b
	v_cndmask_b32_e32 v25, v25, v26, vcc_lo                     ; 02323519
	v_cmp_le_i32_e32 vcc_lo, s10, v34                           ; 7c86440a
	v_add_nc_u32_e32 v41, s9, v40                               ; 4a525009
	v_lshrrev_b32_e32 v35, 24, v35                              ; 32464698
	s_waitcnt vmcnt(2)                                          ; bf890bf7
	v_bfe_u32 v44, v36, 0, 8                                    ; d610002c 02210124
	v_mul_lo_u32 v27, v27, v25                                  ; d72c001b 0002331b
	v_dual_cndmask_b32 v34, v34, v28 :: v_dual_and_b32 v31, 15, v15 ; ca643922 221e1e8f
	v_cmp_le_i32_e32 vcc_lo, s10, v29                           ; 7c863a0a
	s_delay_alu instid0(VALU_DEP_2)                             ; bf870002
	v_add_nc_u32_e32 v31, -8, v31                               ; 4a3e3ec8
	v_mul_lo_u32 v14, v14, v34                                  ; d72c000e 0002450e
	v_bfe_u32 v34, v15, 8, 4                                    ; d6100022 0211110f
	v_bfe_u32 v15, v15, 24, 4                                   ; d610000f 0211310f
	v_dual_cndmask_b32 v29, v29, v30 :: v_dual_add_nc_u32 v24, -8, v24 ; ca603d1d 1d1830c8
	v_cmp_le_i32_e32 vcc_lo, s10, v32                           ; 7c86400a
	v_bfe_u32 v47, v36, 8, 8                                    ; d610002f 02211124
	v_add_nc_u32_e32 v34, -8, v34                               ; 4a4444c8
	s_delay_alu instid0(VALU_DEP_4)                             ; bf870004
	v_mul_lo_u32 v31, v31, v29                                  ; d72c001f 00023b1f
	v_mul_lo_u32 v24, v24, v12                                  ; d72c0018 00021918
	v_dual_cndmask_b32 v32, v32, v33 :: v_dual_add_nc_u32 v43, s9, v35 ; ca604320 202a4609
	v_cmp_le_i32_e32 vcc_lo, s10, v40                           ; 7c86500a
	v_add_nc_u32_e32 v48, s9, v47                               ; 4a605e09
	v_bfe_u32 v49, v16, 8, 4                                    ; d6100031 02111110
	v_bfe_u32 v50, v36, 16, 8                                   ; d6100032 02212124
	v_mul_lo_u32 v34, v34, v32                                  ; d72c0022 00024122
	v_dual_cndmask_b32 v40, v40, v41 :: v_dual_add_nc_u32 v45, s9, v44 ; ca605328 282c5809
	v_cmp_le_i32_e32 vcc_lo, s10, v35                           ; 7c86460a
	v_add3_u32 v27, v27, v11, v24                               ; d655001b 0462171b
	v_add_nc_u32_e32 v49, -8, v49                               ; 4a6262c8
	v_lshrrev_b32_e32 v36, 24, v36                              ; 32484898
	v_add_nc_u32_e32 v51, s9, v50                               ; 4a666409
	v_bfe_u32 v52, v16, 16, 4                                   ; d6100034 02112110
	v_mul_lo_u32 v42, v42, v40                                  ; d72c002a 0002512a
	v_dual_cndmask_b32 v35, v35, v43 :: v_dual_and_b32 v46, 15, v16 ; ca645723 232e208f
	v_cmp_le_i32_e32 vcc_lo, s10, v44                           ; 7c86580a
	v_bfe_u32 v16, v16, 24, 4                                   ; d6100010 02113110
	v_add3_u32 v1, v1, v27, v14                                 ; d6550001 043a3701
	v_add_nc_u32_e32 v53, s9, v36                               ; 4a6a4809
	v_bfe_u32 v54, v37, 0, 8                                    ; d6100036 02210125
	v_add_nc_u32_e32 v52, -8, v52                               ; 4a6868c8
	v_bfe_u32 v60, v37, 16, 8                                   ; d610003c 02212125
	v_and_b32_e32 v56, 15, v17                                  ; 3670228f
	v_dual_cndmask_b32 v44, v44, v45 :: v_dual_add_nc_u32 v15, -8, v15 ; ca605b2c 2c0e1ec8
	v_cmp_le_i32_e32 vcc_lo, s10, v47                           ; 7c865e0a
	v_add3_u32 v42, v42, v31, v34                               ; d655002a 048a3f2a
	v_add_nc_u32_e32 v16, -8, v16                               ; 4a2020c8
	v_bfe_u32 v57, v37, 8, 8                                    ; d6100039 02211125
	v_lshrrev_b32_e32 v37, 24, v37                              ; 324a4a98
	v_mul_lo_u32 v15, v15, v35                                  ; d72c000f 0002470f
	v_dual_cndmask_b32 v47, v47, v48 :: v_dual_add_nc_u32 v46, -8, v46 ; ca60612f 2f2e5cc8
	v_cmp_le_i32_e32 vcc_lo, s10, v50                           ; 7c86640a
	v_add_nc_u32_e32 v58, s9, v57                               ; 4a747209
	v_bfe_u32 v59, v17, 8, 4                                    ; d610003b 02111111
	s_delay_alu instid0(VALU_DEP_4)                             ; bf870004
	v_mul_lo_u32 v49, v49, v47                                  ; d72c0031 00025f31
	v_mul_lo_u32 v46, v46, v44                                  ; d72c002e 0002592e
	v_dual_cndmask_b32 v50, v50, v51 :: v_dual_add_nc_u32 v55, s9, v54 ; ca606732 32366c09
	v_cmp_le_i32_e32 vcc_lo, s10, v36                           ; 7c86480a
	v_add3_u32 v1, v1, v42, v15                                 ; d6550001 043e5501
	v_bfe_u32 v62, v17, 16, 4                                   ; d610003e 02112111
	v_and_b32_e32 v66, 15, v18                                  ; 3684248f
	v_bfe_u32 v17, v17, 24, 4                                   ; d6100011 02113111
	v_mul_lo_u32 v52, v52, v50                                  ; d72c0034 00026534
	v_dual_cndmask_b32 v36, v36, v53 :: v_dual_add_nc_u32 v61, s9, v60 ; ca606b24 243c7809
	v_cmp_le_i32_e32 vcc_lo, s10, v54                           ; 7c866c0a
	v_add_nc_u32_e32 v62, -8, v62                               ; 4a7c7cc8
	v_bfe_u32 v64, v38, 0, 8                                    ; d6100040 02210126
	v_add_nc_u32_e32 v17, -8, v17                               ; 4a2222c8
	v_bfe_u32 v67, v38, 8, 8                                    ; d6100043 02211126
	v_bfe_u32 v69, v18, 8, 4                                    ; d6100045 02111112
	v_mul_lo_u32 v16, v16, v36                                  ; d72c0010 00024910
	v_dual_cndmask_b32 v54, v54, v55 :: v_dual_add_nc_u32 v63, s9, v37 ; ca606f36 363e4a09
	v_cmp_le_i32_e32 vcc_lo, s10, v57                           ; 7c86720a
	v_add3_u32 v52, v52, v46, v49                               ; d6550034 04c65d34
	v_add_nc_u32_e32 v65, s9, v64                               ; 4a828009
	v_add_nc_u32_e32 v68, s9, v67                               ; 4a888609
	v_bfe_u32 v70, v38, 16, 8                                   ; d6100046 02212126
	v_add_nc_u32_e32 v69, -8, v69                               ; 4a8a8ac8
	v_lshrrev_b32_e32 v38, 24, v38                              ; 324c4c98
	v_dual_cndmask_b32 v57, v57, v58 :: v_dual_add_nc_u32 v56, -8, v56 ; ca607539 393870c8
	v_cmp_le_i32_e32 vcc_lo, s10, v60                           ; 7c86780a
	v_add3_u32 v1, v1, v52, v16                                 ; d6550001 04426901
	v_bfe_u32 v72, v18, 16, 4                                   ; d6100048 02112112
	v_add_nc_u32_e32 v71, s9, v70                               ; 4a8e8c09
	v_bfe_u32 v18, v18, 24, 4                                   ; d6100012 02113112
	v_mul_lo_u32 v56, v56, v54                                  ; d72c0038 00026d38
	v_dual_cndmask_b32 v60, v60, v61 :: v_dual_add_nc_u32 v59, -8, v59 ; ca607b3c 3c3a76c8
	v_cmp_le_i32_e32 vcc_lo, s10, v37                           ; 7c864a0a
	v_add_nc_u32_e32 v72, -8, v72                               ; 4a9090c8
	v_bfe_u32 v74, v39, 0, 8                                    ; d610004a 02210127
	v_bfe_u32 v79, v19, 8, 4                                    ; d610004f 02111113
	v_mul_lo_u32 v59, v59, v57                                  ; d72c003b 0002733b
	v_mul_lo_u32 v62, v62, v60                                  ; d72c003e 0002793e
	v_dual_cndmask_b32 v37, v37, v63 :: v_dual_add_nc_u32 v66, -8, v66 ; ca607f25 254284c8
	v_cmp_le_i32_e32 vcc_lo, s10, v64                           ; 7c86800a
	v_bfe_u32 v80, v39, 16, 8                                   ; d6100050 02212127
	v_and_b32_e32 v76, 15, v19                                  ; 3698268f
	s_delay_alu instid0(VALU_DEP_4) | instskip(SKIP_4) | instid1(VALU_DEP_4) ; bf870254
	v_mul_lo_u32 v17, v17, v37                                  ; d72c0011 00024b11
	v_dual_cndmask_b32 v64, v64, v65 :: v_dual_add_nc_u32 v73, s9, v38 ; ca608340 40484c09
	v_cmp_le_i32_e32 vcc_lo, s10, v67                           ; 7c86860a
	v_add3_u32 v62, v62, v56, v59                               ; d655003e 04ee713e
	v_bfe_u32 v77, v39, 8, 8                                    ; d610004d 02211127
	v_mul_lo_u32 v66, v66, v64                                  ; d72c0042 00028142
	v_dual_cndmask_b32 v67, v67, v68 :: v_dual_add_nc_u32 v18, -8, v18 ; ca608943 431224c8
	v_cmp_le_i32_e32 vcc_lo, s10, v70                           ; 7c868c0a
	v_add3_u32 v1, v1, v62, v17                                 ; d6550001 04467d01
	v_add_nc_u32_e32 v78, s9, v77                               ; 4a9c9a09
	v_lshrrev_b32_e32 v39, 24, v39                              ; 324e4e98
	v_bfe_u32 v82, v19, 16, 4                                   ; d6100052 02112113
	v_mul_lo_u32 v69, v69, v67                                  ; d72c0045 00028745
	v_dual_cndmask_b32 v70, v70, v71 :: v_dual_add_nc_u32 v75, s9, v74 ; ca608f46 464a9409
	v_cmp_le_i32_e32 vcc_lo, s10, v38                           ; 7c864c0a
	v_add_nc_u32_e32 v82, -8, v82                               ; 4aa4a4c8
	s_waitcnt vmcnt(0)                                          ; bf8903f7
	v_bfe_u32 v84, v20, 0, 8                                    ; d6100054 02210114
	s_delay_alu instid0(VALU_DEP_4)                             ; bf870004
	v_mul_lo_u32 v72, v72, v70                                  ; d72c0048 00028d48
	v_dual_cndmask_b32 v38, v38, v73 :: v_dual_add_nc_u32 v79, -8, v79 ; ca609326 264e9ec8
	v_cmp_le_i32_e32 vcc_lo, s10, v74                           ; 7c86940a
	v_add_nc_u32_e32 v85, s9, v84                               ; 4aaaa809
	v_bfe_u32 v19, v19, 24, 4                                   ; d6100013 02113113
	v_bfe_u32 v87, v20, 8, 8                                    ; d6100057 02211114
	v_mul_lo_u32 v18, v18, v38                                  ; d72c0012 00024d12
	v_dual_cndmask_b32 v74, v74, v75 :: v_dual_add_nc_u32 v81, s9, v80 ; ca60974a 4a50a009
	v_cmp_le_i32_e32 vcc_lo, s10, v77                           ; 7c869a0a
	v_add3_u32 v72, v72, v66, v69                               ; d6550048 05168548
	v_add_nc_u32_e32 v88, s9, v87                               ; 4ab0ae09
	v_bfe_u32 v89, v4, 8, 4                                     ; d6100059 02111104
	v_bfe_u32 v90, v20, 16, 8                                   ; d610005a 02212114
	v_dual_cndmask_b32 v77, v77, v78 :: v_dual_add_nc_u32 v76, -8, v76 ; ca609d4d 4d4c98c8
	v_cmp_le_i32_e32 vcc_lo, s10, v80                           ; 7c86a00a
	v_add3_u32 v1, v1, v72, v18                                 ; d6550001 044a9101
	v_add_nc_u32_e32 v89, -8, v89                               ; 4ab2b2c8
	v_add_nc_u32_e32 v91, s9, v90                               ; 4ab6b409
	v_lshrrev_b32_e32 v20, 24, v20                              ; 32282898
	v_mul_lo_u32 v79, v79, v77                                  ; d72c004f 00029b4f
	v_mul_lo_u32 v76, v76, v74                                  ; d72c004c 0002954c
	v_bfe_u32 v92, v4, 16, 4                                    ; d610005c 02112104
	v_dual_cndmask_b32 v80, v80, v81 :: v_dual_add_nc_u32 v83, s9, v39 ; ca60a350 50524e09
	v_cmp_le_i32_e32 vcc_lo, s10, v39                           ; 7c864e0a
	v_bfe_u32 v94, v21, 0, 8                                    ; d610005e 02210115
	s_delay_alu instid0(VALU_DEP_4)                             ; bf870004
	v_add_nc_u32_e32 v92, -8, v92                               ; 4ab8b8c8
	v_mul_lo_u32 v82, v82, v80                                  ; d72c0052 0002a152
	v_dual_cndmask_b32 v39, v39, v83 :: v_dual_and_b32 v86, 15, v4 ; ca64a727 2756088f
	v_cmp_le_i32_e32 vcc_lo, s10, v84                           ; 7c86a80a
	v_bfe_u32 v4, v4, 24, 4                                     ; d6100004 02113104
	v_dual_cndmask_b32 v84, v84, v85 :: v_dual_add_nc_u32 v19, -8, v19 ; ca60ab54 541226c8
	v_cmp_le_i32_e32 vcc_lo, s10, v87                           ; 7c86ae0a
	v_add3_u32 v82, v82, v76, v79                               ; d6550052 053e9952
	s_delay_alu instid0(VALU_DEP_4)                             ; bf870004
	v_add_nc_u32_e32 v4, -8, v4                                 ; 4a0808c8
	v_bfe_u32 v2, v21, 8, 8                                     ; d6100002 02211115
	v_mul_lo_u32 v19, v19, v39                                  ; d72c0013 00024f13
	v_dual_cndmask_b32 v87, v87, v88 :: v_dual_add_nc_u32 v86, -8, v86 ; ca60b157 5756acc8
	v_cmp_le_i32_e32 vcc_lo, s10, v90                           ; 7c86b40a
	v_add_nc_u32_e32 v3, s9, v2                                 ; 4a060409
	v_bfe_u32 v12, v5, 16, 4                                    ; d610000c 02112105
	s_delay_alu instid0(VALU_DEP_4)                             ; bf870004
	v_mul_lo_u32 v89, v89, v87                                  ; d72c0059 0002af59
	v_mul_lo_u32 v86, v86, v84                                  ; d72c0056 0002a956
	v_dual_cndmask_b32 v90, v90, v91 :: v_dual_add_nc_u32 v93, s9, v20 ; ca60b75a 5a5c2809
	v_cmp_le_i32_e32 vcc_lo, s10, v20                           ; 7c86280a
	v_add3_u32 v1, v1, v82, v19                                 ; d6550001 044ea501
	v_add_nc_u32_e32 v12, -8, v12                               ; 4a1818c8
	v_bfe_u32 v10, v21, 16, 8                                   ; d610000a 02212115
	v_lshrrev_b32_e32 v21, 24, v21                              ; 322a2a98
	v_mul_lo_u32 v92, v92, v90                                  ; d72c005c 0002b55c
	v_dual_cndmask_b32 v20, v20, v93 :: v_dual_add_nc_u32 v95, s9, v94 ; ca60bb14 145ebc09
	v_cmp_le_i32_e32 vcc_lo, s10, v94                           ; 7c86bc0a
	v_add_nc_u32_e32 v11, s9, v10                               ; 4a161409
	v_and_b32_e32 v16, 15, v6                                   ; 36200c8f
	s_delay_alu instid0(VALU_DEP_4)                             ; bf870004
	v_mul_lo_u32 v4, v4, v20                                    ; d72c0004 00022904
	v_dual_cndmask_b32 v94, v94, v95 :: v_dual_and_b32 v95, 15, v5 ; ca64bf5e 5e5e0a8f
	v_cmp_le_i32_e32 vcc_lo, s10, v2                            ; 7c86040a
	v_add3_u32 v92, v92, v86, v89                               ; d655005c 0566ad5c
	v_bfe_u32 v14, v22, 0, 8                                    ; d610000e 02210116
	v_bfe_u32 v17, v22, 8, 8                                    ; d6100011 02211116
	v_bfe_u32 v19, v6, 8, 4                                     ; d6100013 02111106
	v_add_nc_u32_e32 v95, -8, v95                               ; 4abebec8
	v_dual_cndmask_b32 v2, v2, v3 :: v_dual_add_nc_u32 v13, s9, v21 ; ca600702 020c2a09
	v_cmp_le_i32_e32 vcc_lo, s10, v10                           ; 7c86140a
	v_add3_u32 v1, v1, v92, v4                                  ; d6550001 0412b901
	v_bfe_u32 v4, v5, 8, 4                                      ; d6100004 02111105
	v_bfe_u32 v5, v5, 24, 4                                     ; d6100005 02113105
	v_add_nc_u32_e32 v15, s9, v14                               ; 4a1e1c09
	v_bfe_u32 v20, v22, 16, 8                                   ; d6100014 02212116
	v_add_nc_u32_e32 v18, s9, v17                               ; 4a242209
	v_add_nc_u32_e32 v19, -8, v19                               ; 4a2626c8
	v_mul_lo_u32 v95, v95, v94                                  ; d72c005f 0002bd5f
	v_lshrrev_b32_e32 v22, 24, v22                              ; 322c2c98
	v_bfe_u32 v24, v6, 16, 4                                    ; d6100018 02112106
	v_add_nc_u32_e32 v4, -8, v4                                 ; 4a0808c8
	v_dual_cndmask_b32 v10, v10, v11 :: v_dual_add_nc_u32 v5, -8, v5 ; ca60170a 0a040ac8
	v_cmp_le_i32_e32 vcc_lo, s10, v21                           ; 7c862a0a
	v_bfe_u32 v6, v6, 24, 4                                     ; d6100006 02113106
	v_bfe_u32 v26, v23, 0, 8                                    ; d610001a 02210117
	v_mul_lo_u32 v4, v4, v2                                     ; d72c0004 00020504
	v_mul_lo_u32 v12, v12, v10                                  ; d72c000c 0002150c
	v_dual_cndmask_b32 v21, v21, v13 :: v_dual_add_nc_u32 v16, -8, v16 ; ca601b15 151020c8
	v_cmp_le_i32_e32 vcc_lo, s10, v14                           ; 7c861c0a
	v_add_nc_u32_e32 v6, -8, v6                                 ; 4a0c0cc8
	v_bfe_u32 v31, v7, 8, 4                                     ; d610001f 02111107
	v_bfe_u32 v32, v23, 16, 8                                   ; d6100020 02212117
	v_and_b32_e32 v28, 15, v7                                   ; 36380e8f
	v_mul_lo_u32 v5, v5, v21                                    ; d72c0005 00022b05
	v_add_nc_u32_e32 v21, s9, v20                               ; 4a2a2809
	v_dual_cndmask_b32 v14, v14, v15 :: v_dual_add_nc_u32 v25, s9, v22 ; ca601f0e 0e182c09
	v_cmp_le_i32_e32 vcc_lo, s10, v17                           ; 7c86220a
	v_add3_u32 v12, v12, v95, v4                                ; d655000c 0412bf0c
	v_bfe_u32 v29, v23, 8, 8                                    ; d610001d 02211117
	s_delay_alu instid0(VALU_DEP_4)                             ; bf870004
	v_mul_lo_u32 v16, v16, v14                                  ; d72c0010 00021d10
	v_dual_cndmask_b32 v17, v17, v18 :: v_dual_add_nc_u32 v24, -8, v24 ; ca602511 111830c8
	v_cmp_le_i32_e32 vcc_lo, s10, v20                           ; 7c86280a
	v_add3_u32 v1, v1, v12, v5                                  ; d6550001 04161901
	v_add_nc_u32_e32 v30, s9, v29                               ; 4a3c3a09
	v_lshrrev_b32_e32 v23, 24, v23                              ; 322e2e98
	v_bfe_u32 v34, v7, 16, 4                                    ; d6100022 02112107
	v_mul_lo_u32 v19, v19, v17                                  ; d72c0013 00022313
	v_dual_cndmask_b32 v20, v20, v21 :: v_dual_add_nc_u32 v27, s9, v26 ; ca602b14 141a3409
	v_cmp_le_i32_e32 vcc_lo, s10, v22                           ; 7c862c0a
	v_bfe_u32 v7, v7, 24, 4                                     ; d6100007 02113107
	s_delay_alu instid0(VALU_DEP_3) | instskip(SKIP_3) | instid1(VALU_DEP_3) ; bf8701c3
	v_mul_lo_u32 v24, v24, v20                                  ; d72c0018 00022918
	v_dual_cndmask_b32 v22, v22, v25 :: v_dual_add_nc_u32 v31, -8, v31 ; ca603316 161e3ec8
	v_cmp_le_i32_e32 vcc_lo, s10, v26                           ; 7c86340a
	v_add_nc_u32_e32 v7, -8, v7                                 ; 4a0e0ec8
	v_mul_lo_u32 v6, v6, v22                                    ; d72c0006 00022d06
	v_dual_cndmask_b32 v26, v26, v27 :: v_dual_add_nc_u32 v33, s9, v32 ; ca60371a 1a204009
	v_cmp_le_i32_e32 vcc_lo, s10, v29                           ; 7c863a0a
	v_add3_u32 v24, v24, v16, v19                               ; d6550018 044e2118
	s_add_u32 s7, s7, 1                                         ; 80078107
	v_dual_cndmask_b32 v29, v29, v30 :: v_dual_add_nc_u32 v28, -8, v28 ; ca603d1d 1d1c38c8
	v_cmp_le_i32_e32 vcc_lo, s10, v32                           ; 7c86400a
	s_delay_alu instid0(VALU_DEP_3) | instskip(NEXT) | instid1(VALU_DEP_3) ; bf870193
	v_add3_u32 v1, v1, v24, v6                                  ; d6550001 041a3101
	v_mul_lo_u32 v28, v28, v26                                  ; d72c001c 0002351c
	v_mul_lo_u32 v31, v31, v29                                  ; d72c001f 00023b1f
	v_dual_cndmask_b32 v32, v32, v33 :: v_dual_add_nc_u32 v35, s9, v23 ; ca604320 20222e09
	v_cmp_le_i32_e32 vcc_lo, s10, v23                           ; 7c862e0a
	s_delay_alu instid0(VALU_DEP_2) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870092
	v_dual_cndmask_b32 v23, v23, v35 :: v_dual_add_nc_u32 v34, -8, v34 ; ca604717 172244c8
	v_mul_lo_u32 v34, v34, v32                                  ; d72c0022 00024122
	v_mul_lo_u32 v7, v7, v23                                    ; d72c0007 00022f07
	v_add3_u32 v34, v34, v28, v31                               ; d6550022 047e3922
	s_delay_alu instid0(VALU_DEP_1)                             ; bf870001
	v_add3_u32 v1, v1, v34, v7                                  ; d6550001 041e4501
	s_branch BB2                                                ; bfa0fce6
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