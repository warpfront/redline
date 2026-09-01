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
	s_cbranch_execz BB11                                        ; bfa50324
BB1:
	s_mov_b32 s7, 0                                             ; be870080
	v_mov_b32_e32 v1, 0                                         ; 7e020280
BB2:
	v_mov_b32_e32 v2, 0                                         ; 7e040280
	global_load_b32 v2, v2, s[2:3] offset:28                    ; dc52001c 02020002
	s_waitcnt vmcnt(0)                                          ; bf8903f7
	v_readfirstlane_b32 s8, v2                                  ; 7e100502
	s_cmp_ge_u32 s7, s8                                         ; bf090807
	s_cbranch_scc1 BB6                                          ; bfa20307
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
	s_movk_i32 s9, 0xff00                                       ; b009ff00
	s_movk_i32 s10, 0x80                                        ; b00a0080
	s_add_u32 s7, s7, 1                                         ; 80078107
	s_waitcnt vmcnt(7)                                          ; bf891ff7
	v_bfe_u32 v43, v25, 16, 4                                   ; d610002b 02112119
	v_bfe_u32 v40, v24, 16, 4                                   ; d6100028 02112118
	v_bfe_u32 v11, v24, 8, 4                                    ; d610000b 02111118
	v_bfe_u32 v42, v25, 8, 4                                    ; d610002a 02111119
	v_and_b32_e32 v10, 15, v24                                  ; 3614308f
	v_bfe_u32 v24, v24, 24, 4                                   ; d6100018 02113118
	v_and_b32_e32 v41, 15, v25                                  ; 3652328f
	v_bfe_u32 v25, v25, 24, 4                                   ; d6100019 02113119
	v_and_b32_e32 v44, 15, v26                                  ; 3658348f
	v_bfe_u32 v45, v26, 8, 4                                    ; d610002d 0211111a
	s_waitcnt vmcnt(6)                                          ; bf891bf7
	v_bfe_u32 v46, v28, 0, 8                                    ; d610002e 0221011c
	v_add_nc_u32_e32 v43, -8, v43                               ; 4a5656c8
	v_add_nc_u32_e32 v40, -8, v40                               ; 4a5050c8
	v_add_nc_u32_e32 v11, -8, v11                               ; 4a1616c8
	v_add_nc_u32_e32 v42, -8, v42                               ; 4a5454c8
	v_add_nc_u32_e32 v10, -8, v10                               ; 4a1414c8
	v_add_nc_u32_e32 v24, -8, v24                               ; 4a3030c8
	v_add_nc_u32_e32 v41, -8, v41                               ; 4a5252c8
	v_add_nc_u32_e32 v25, -8, v25                               ; 4a3232c8
	v_add_nc_u32_e32 v44, -8, v44                               ; 4a5858c8
	v_bfe_u32 v3, v28, 16, 8                                    ; d6100003 0221211c
	v_add_nc_u32_e32 v45, -8, v45                               ; 4a5a5ac8
	v_add_nc_u32_e32 v47, s9, v46                               ; 4a5e5c09
	v_cmp_le_i32_e32 vcc, s10, v46                              ; 7c865c0a
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_2) | instid1(VALU_DEP_3) ; bf8701b2
	v_cndmask_b32_e32 v46, v46, v47, vcc                        ; 025c5f2e
	v_bfe_u32 v47, v28, 8, 8                                    ; d610002f 0221111c
	v_lshrrev_b32_e32 v28, 24, v28                              ; 32383898
	v_mul_lo_u32 v10, v10, v46                                  ; d72c000a 00025d0a
	v_add_nc_u32_e32 v46, s9, v3                                ; 4a5c0609
	v_cmp_le_i32_e32 vcc, s10, v47                              ; 7c865e0a
	v_add_nc_u32_e32 v2, s9, v47                                ; 4a045e09
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_2) ; bf870121
	v_cndmask_b32_e32 v47, v47, v2, vcc                         ; 025e052f
	v_cmp_le_i32_e32 vcc, s10, v3                               ; 7c86060a
	v_mul_lo_u32 v11, v11, v47                                  ; d72c000b 00025f0b
	v_add_nc_u32_e32 v47, s9, v28                               ; 4a5e3809
	v_cndmask_b32_e32 v3, v3, v46, vcc                          ; 02065d03
	v_cmp_le_i32_e32 vcc, s10, v28                              ; 7c86380a
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_3) | instid1(VALU_DEP_2) ; bf870142
	v_mul_lo_u32 v40, v40, v3                                   ; d72c0028 00020728
	v_bfe_u32 v3, v29, 8, 8                                     ; d6100003 0221111d
	v_cndmask_b32_e32 v28, v28, v47, vcc                        ; 02385f1c
	v_bfe_u32 v47, v29, 0, 8                                    ; d610002f 0221011d
	v_mul_lo_u32 v24, v24, v28                                  ; d72c0018 00023918
	v_add_nc_u32_e32 v2, s9, v47                                ; 4a045e09
	v_cmp_le_i32_e32 vcc, s10, v47                              ; 7c865e0a
	v_add3_u32 v40, v40, v10, v11                               ; d6550028 042e1528
	v_bfe_u32 v11, v29, 16, 8                                   ; d610000b 0221211d
	v_add_nc_u32_e32 v10, s9, v3                                ; 4a140609
	v_lshrrev_b32_e32 v29, 24, v29                              ; 323a3a98
	v_cndmask_b32_e32 v47, v47, v2, vcc                         ; 025e052f
	v_cmp_le_i32_e32 vcc, s10, v3                               ; 7c86060a
	v_add3_u32 v1, v1, v40, v24                                 ; d6550001 04625101
	v_add_nc_u32_e32 v24, s9, v11                               ; 4a301609
	v_add_nc_u32_e32 v28, s9, v29                               ; 4a383a09
	v_mul_lo_u32 v41, v41, v47                                  ; d72c0029 00025f29
	v_cndmask_b32_e32 v3, v3, v10, vcc                          ; 02061503
	v_cmp_le_i32_e32 vcc, s10, v11                              ; 7c86160a
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_2) | instid1(VALU_DEP_2) ; bf870132
	v_mul_lo_u32 v42, v42, v3                                   ; d72c002a 0002072a
	v_cndmask_b32_e32 v11, v11, v24, vcc                        ; 0216310b
	v_cmp_le_i32_e32 vcc, s10, v29                              ; 7c863a0a
	v_mul_lo_u32 v43, v43, v11                                  ; d72c002b 0002172b
	v_cndmask_b32_e32 v29, v29, v28, vcc                        ; 023a391d
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_4) | instid1(VALU_DEP_4) ; bf870251
	v_mul_lo_u32 v25, v25, v29                                  ; d72c0019 00023b19
	v_bfe_u32 v29, v30, 0, 8                                    ; d610001d 0221011e
	v_add3_u32 v43, v43, v41, v42                               ; d655002b 04aa532b
	v_bfe_u32 v41, v30, 8, 8                                    ; d6100029 0221111e
	v_bfe_u32 v47, v26, 16, 4                                   ; d610002f 0211211a
	v_cmp_le_i32_e32 vcc, s10, v29                              ; 7c863a0a
	v_add_nc_u32_e32 v40, s9, v29                               ; 4a503a09
	v_add3_u32 v1, v1, v43, v25                                 ; d6550001 04665701
	v_bfe_u32 v43, v30, 16, 8                                   ; d610002b 0221211e
	v_lshrrev_b32_e32 v30, 24, v30                              ; 323c3c98
	v_add_nc_u32_e32 v42, s9, v41                               ; 4a545209
	v_add_nc_u32_e32 v47, -8, v47                               ; 4a5e5ec8
	v_bfe_u32 v26, v26, 24, 4                                   ; d610001a 0211311a
	v_cndmask_b32_e32 v29, v29, v40, vcc                        ; 023a511d
	v_cmp_le_i32_e32 vcc, s10, v41                              ; 7c86520a
	v_add_nc_u32_e32 v46, s9, v43                               ; 4a5c5609
	v_bfe_u32 v3, v31, 0, 8                                     ; d6100003 0221011f
	v_add_nc_u32_e32 v2, s9, v30                                ; 4a043c09
	v_add_nc_u32_e32 v26, -8, v26                               ; 4a3434c8
	v_and_b32_e32 v11, 15, v27                                  ; 3616368f
	v_mul_lo_u32 v44, v44, v29                                  ; d72c002c 00023b2c
	v_cndmask_b32_e32 v41, v41, v42, vcc                        ; 02525529
	v_cmp_le_i32_e32 vcc, s10, v43                              ; 7c86560a
	v_add_nc_u32_e32 v10, s9, v3                                ; 4a140609
	v_bfe_u32 v24, v31, 8, 8                                    ; d6100018 0221111f
	v_add_nc_u32_e32 v11, -8, v11                               ; 4a1616c8
	v_mul_lo_u32 v45, v45, v41                                  ; d72c002d 0002532d
	v_cndmask_b32_e32 v43, v43, v46, vcc                        ; 02565d2b
	v_cmp_le_i32_e32 vcc, s10, v30                              ; 7c863c0a
	v_add_nc_u32_e32 v25, s9, v24                               ; 4a323009
	v_bfe_u32 v28, v31, 16, 8                                   ; d610001c 0221211f
	s_delay_alu instid0(VALU_DEP_4) | instskip(SKIP_3) | instid1(VALU_DEP_3) ; bf8701c4
	v_mul_lo_u32 v47, v47, v43                                  ; d72c002f 0002572f
	v_cndmask_b32_e32 v30, v30, v2, vcc                         ; 023c051e
	v_cmp_le_i32_e32 vcc, s10, v3                               ; 7c86060a
	v_add_nc_u32_e32 v29, s9, v28                               ; 4a3a3809
	v_mul_lo_u32 v26, v26, v30                                  ; d72c001a 00023d1a
	v_bfe_u32 v30, v27, 16, 4                                   ; d610001e 0211211b
	v_cndmask_b32_e32 v3, v3, v10, vcc                          ; 02061503
	v_cmp_le_i32_e32 vcc, s10, v24                              ; 7c86300a
	v_add3_u32 v47, v47, v44, v45                               ; d655002f 04b6592f
	v_lshrrev_b32_e32 v31, 24, v31                              ; 323e3e98
	v_add_nc_u32_e32 v30, -8, v30                               ; 4a3c3cc8
	v_mul_lo_u32 v11, v11, v3                                   ; d72c000b 0002070b
	v_cndmask_b32_e32 v24, v24, v25, vcc                        ; 02303318
	v_cmp_le_i32_e32 vcc, s10, v28                              ; 7c86380a
	v_add3_u32 v1, v1, v47, v26                                 ; d6550001 046a5f01
	v_bfe_u32 v26, v27, 8, 4                                    ; d610001a 0211111b
	v_bfe_u32 v27, v27, 24, 4                                   ; d610001b 0211311b
	v_add_nc_u32_e32 v40, s9, v31                               ; 4a503e09
	s_waitcnt vmcnt(5)                                          ; bf8917f7
	v_and_b32_e32 v41, 15, v12                                  ; 3652188f
	v_bfe_u32 v42, v12, 8, 4                                    ; d610002a 0211110c
	v_bfe_u32 v43, v12, 16, 4                                   ; d610002b 0211210c
	s_waitcnt vmcnt(4)                                          ; bf8913f7
	v_bfe_u32 v44, v32, 0, 8                                    ; d610002c 02210120
	v_cndmask_b32_e32 v28, v28, v29, vcc                        ; 02383b1c
	v_cmp_le_i32_e32 vcc, s10, v31                              ; 7c863e0a
	v_add_nc_u32_e32 v26, -8, v26                               ; 4a3434c8
	v_add_nc_u32_e32 v27, -8, v27                               ; 4a3636c8
	v_add_nc_u32_e32 v41, -8, v41                               ; 4a5252c8
	v_bfe_u32 v46, v32, 8, 8                                    ; d610002e 02211120
	v_add_nc_u32_e32 v42, -8, v42                               ; 4a5454c8
	v_add_nc_u32_e32 v43, -8, v43                               ; 4a5656c8
	v_add_nc_u32_e32 v45, s9, v44                               ; 4a5a5809
	v_mul_lo_u32 v30, v30, v28                                  ; d72c001e 0002391e
	v_cndmask_b32_e32 v31, v31, v40, vcc                        ; 023e511f
	v_cmp_le_i32_e32 vcc, s10, v44                              ; 7c86580a
	v_mul_lo_u32 v26, v26, v24                                  ; d72c001a 0002311a
	v_add_nc_u32_e32 v47, s9, v46                               ; 4a5e5c09
	v_mul_lo_u32 v27, v27, v31                                  ; d72c001b 00023f1b
	v_cndmask_b32_e32 v44, v44, v45, vcc                        ; 02585b2c
	v_cmp_le_i32_e32 vcc, s10, v46                              ; 7c865c0a
	v_bfe_u32 v12, v12, 24, 4                                   ; d610000c 0211310c
	v_add3_u32 v30, v30, v11, v26                               ; d655001e 046a171e
	s_delay_alu instid0(VALU_DEP_4)                             ; bf870004
	v_mul_lo_u32 v41, v41, v44                                  ; d72c0029 00025929
	v_cndmask_b32_e32 v46, v46, v47, vcc                        ; 025c5f2e
	v_bfe_u32 v47, v32, 16, 8                                   ; d610002f 02212120
	v_lshrrev_b32_e32 v32, 24, v32                              ; 32404098
	v_add_nc_u32_e32 v12, -8, v12                               ; 4a1818c8
	v_add3_u32 v1, v1, v30, v27                                 ; d6550001 046e3d01
	v_bfe_u32 v10, v33, 0, 8                                    ; d610000a 02210121
	v_mul_lo_u32 v42, v42, v46                                  ; d72c002a 00025d2a
	v_cmp_le_i32_e32 vcc, s10, v47                              ; 7c865e0a
	v_add_nc_u32_e32 v2, s9, v47                                ; 4a045e09
	v_add_nc_u32_e32 v3, s9, v32                                ; 4a064009
	v_add_nc_u32_e32 v11, s9, v10                               ; 4a161409
	v_bfe_u32 v24, v33, 8, 8                                    ; d6100018 02211121
	v_bfe_u32 v26, v13, 8, 4                                    ; d610001a 0211110d
	v_cndmask_b32_e32 v47, v47, v2, vcc                         ; 025e052f
	v_cmp_le_i32_e32 vcc, s10, v32                              ; 7c86400a
	v_bfe_u32 v27, v33, 16, 8                                   ; d610001b 02212121
	v_add_nc_u32_e32 v25, s9, v24                               ; 4a323009
	v_add_nc_u32_e32 v26, -8, v26                               ; 4a3434c8
	v_mul_lo_u32 v43, v43, v47                                  ; d72c002b 00025f2b
	v_cndmask_b32_e32 v32, v32, v3, vcc                         ; 02400720
	v_cmp_le_i32_e32 vcc, s10, v10                              ; 7c86140a
	v_add_nc_u32_e32 v28, s9, v27                               ; 4a383609
	v_bfe_u32 v29, v13, 16, 4                                   ; d610001d 0211210d
	v_lshrrev_b32_e32 v33, 24, v33                              ; 32424298
	v_mul_lo_u32 v12, v12, v32                                  ; d72c000c 0002410c
	v_cndmask_b32_e32 v10, v10, v11, vcc                        ; 0214170a
	v_cmp_le_i32_e32 vcc, s10, v24                              ; 7c86300a
	v_add3_u32 v43, v43, v41, v42                               ; d655002b 04aa532b
	v_add_nc_u32_e32 v29, -8, v29                               ; 4a3a3ac8
	v_add_nc_u32_e32 v30, s9, v33                               ; 4a3c4209
	v_cndmask_b32_e32 v24, v24, v25, vcc                        ; 02303318
	v_cmp_le_i32_e32 vcc, s10, v27                              ; 7c86360a
	v_bfe_u32 v31, v34, 0, 8                                    ; d610001f 02210122
	v_add3_u32 v1, v1, v43, v12                                 ; d6550001 04325701
	v_and_b32_e32 v12, 15, v13                                  ; 36181a8f
	v_bfe_u32 v13, v13, 24, 4                                   ; d610000d 0211310d
	v_mul_lo_u32 v26, v26, v24                                  ; d72c001a 0002311a
	v_cndmask_b32_e32 v27, v27, v28, vcc                        ; 0236391b
	v_cmp_le_i32_e32 vcc, s10, v33                              ; 7c86420a
	v_add_nc_u32_e32 v32, s9, v31                               ; 4a403e09
	v_add_nc_u32_e32 v12, -8, v12                               ; 4a1818c8
	v_bfe_u32 v40, v34, 8, 8                                    ; d6100028 02211122
	v_add_nc_u32_e32 v13, -8, v13                               ; 4a1a1ac8
	v_bfe_u32 v42, v14, 8, 4                                    ; d610002a 0211110e
	v_mul_lo_u32 v29, v29, v27                                  ; d72c001d 0002371d
	v_cndmask_b32_e32 v33, v33, v30, vcc                        ; 02423d21
	v_cmp_le_i32_e32 vcc, s10, v31                              ; 7c863e0a
	v_mul_lo_u32 v12, v12, v10                                  ; d72c000c 0002150c
	v_add_nc_u32_e32 v41, s9, v40                               ; 4a525009
	v_bfe_u32 v43, v34, 16, 8                                   ; d610002b 02212122
	v_add_nc_u32_e32 v42, -8, v42                               ; 4a5454c8
	v_mul_lo_u32 v13, v13, v33                                  ; d72c000d 0002430d
	v_and_b32_e32 v33, 15, v14                                  ; 36421c8f
	v_bfe_u32 v45, v14, 16, 4                                   ; d610002d 0211210e
	v_cndmask_b32_e32 v31, v31, v32, vcc                        ; 023e411f
	v_cmp_le_i32_e32 vcc, s10, v40                              ; 7c86500a
	v_add_nc_u32_e32 v44, s9, v43                               ; 4a585609
	v_add3_u32 v29, v29, v12, v26                               ; d655001d 046a191d
	v_lshrrev_b32_e32 v34, 24, v34                              ; 32444498
	v_bfe_u32 v14, v14, 24, 4                                   ; d610000e 0211310e
	v_add_nc_u32_e32 v33, -8, v33                               ; 4a4242c8
	v_add_nc_u32_e32 v45, -8, v45                               ; 4a5a5ac8
	v_cndmask_b32_e32 v40, v40, v41, vcc                        ; 02505328
	v_cmp_le_i32_e32 vcc, s10, v43                              ; 7c86560a
	v_bfe_u32 v47, v35, 0, 8                                    ; d610002f 02210123
	v_add3_u32 v1, v1, v29, v13                                 ; d6550001 04363b01
	v_add_nc_u32_e32 v46, s9, v34                               ; 4a5c4409
	v_add_nc_u32_e32 v14, -8, v14                               ; 4a1c1cc8
	v_and_b32_e32 v3, 15, v15                                   ; 36061e8f
	v_mul_lo_u32 v33, v33, v31                                  ; d72c0021 00023f21
	v_bfe_u32 v10, v35, 8, 8                                    ; d610000a 02211123
	v_mul_lo_u32 v42, v42, v40                                  ; d72c002a 0002512a
	v_cndmask_b32_e32 v43, v43, v44, vcc                        ; 0256592b
	v_cmp_le_i32_e32 vcc, s10, v34                              ; 7c86440a
	v_add_nc_u32_e32 v2, s9, v47                                ; 4a045e09
	v_add_nc_u32_e32 v3, -8, v3                                 ; 4a0606c8
	v_bfe_u32 v12, v15, 8, 4                                    ; d610000c 0211110f
	v_add_nc_u32_e32 v11, s9, v10                               ; 4a161409
	v_bfe_u32 v13, v35, 16, 8                                   ; d610000d 02212123
	v_mul_lo_u32 v45, v45, v43                                  ; d72c002d 0002572d
	v_cndmask_b32_e32 v34, v34, v46, vcc                        ; 02445d22
	v_cmp_le_i32_e32 vcc, s10, v47                              ; 7c865e0a
	v_bfe_u32 v24, v15, 16, 4                                   ; d6100018 0211210f
	v_add_nc_u32_e32 v12, -8, v12                               ; 4a1818c8
	v_lshrrev_b32_e32 v35, 24, v35                              ; 32464698
	v_mul_lo_u32 v14, v14, v34                                  ; d72c000e 0002450e
	v_cndmask_b32_e32 v47, v47, v2, vcc                         ; 025e052f
	v_cmp_le_i32_e32 vcc, s10, v10                              ; 7c86140a
	v_add3_u32 v45, v45, v33, v42                               ; d655002d 04aa432d
	v_add_nc_u32_e32 v24, -8, v24                               ; 4a3030c8
	v_add_nc_u32_e32 v25, s9, v35                               ; 4a324609
	v_bfe_u32 v15, v15, 24, 4                                   ; d610000f 0211310f
	s_waitcnt vmcnt(2)                                          ; bf890bf7
	v_bfe_u32 v26, v36, 0, 8                                    ; d610001a 02210124
	v_mul_lo_u32 v3, v3, v47                                    ; d72c0003 00025f03
	v_cndmask_b32_e32 v10, v10, v11, vcc                        ; 0214170a
	v_cmp_le_i32_e32 vcc, s10, v13                              ; 7c861a0a
	v_add3_u32 v1, v1, v45, v14                                 ; d6550001 043a5b01
	v_add_nc_u32_e32 v14, s9, v13                               ; 4a1c1a09
	v_add_nc_u32_e32 v15, -8, v15                               ; 4a1e1ec8
	v_add_nc_u32_e32 v27, s9, v26                               ; 4a363409
	v_and_b32_e32 v28, 15, v16                                  ; 3638208f
	v_bfe_u32 v29, v36, 8, 8                                    ; d610001d 02211124
	v_mul_lo_u32 v12, v12, v10                                  ; d72c000c 0002150c
	v_bfe_u32 v31, v16, 8, 4                                    ; d610001f 02111110
	v_cndmask_b32_e32 v13, v13, v14, vcc                        ; 021a1d0d
	v_cmp_le_i32_e32 vcc, s10, v35                              ; 7c86460a
	v_add_nc_u32_e32 v28, -8, v28                               ; 4a3838c8
	v_add_nc_u32_e32 v30, s9, v29                               ; 4a3c3a09
	v_bfe_u32 v32, v36, 16, 8                                   ; d6100020 02212124
	v_add_nc_u32_e32 v31, -8, v31                               ; 4a3e3ec8
	v_bfe_u32 v34, v16, 16, 4                                   ; d6100022 02112110
	v_mul_lo_u32 v24, v24, v13                                  ; d72c0018 00021b18
	v_cndmask_b32_e32 v35, v35, v25, vcc                        ; 02463323
	v_cmp_le_i32_e32 vcc, s10, v26                              ; 7c86340a
	v_add_nc_u32_e32 v33, s9, v32                               ; 4a424009
	v_add_nc_u32_e32 v34, -8, v34                               ; 4a4444c8
	v_lshrrev_b32_e32 v36, 24, v36                              ; 32484898
	v_bfe_u32 v16, v16, 24, 4                                   ; d6100010 02113110
	v_mul_lo_u32 v15, v15, v35                                  ; d72c000f 0002470f
	v_cndmask_b32_e32 v26, v26, v27, vcc                        ; 0234371a
	v_cmp_le_i32_e32 vcc, s10, v29                              ; 7c863a0a
	v_add3_u32 v24, v24, v3, v12                                ; d6550018 04320718
	v_add_nc_u32_e32 v35, s9, v36                               ; 4a464809
	v_add_nc_u32_e32 v16, -8, v16                               ; 4a2020c8
	v_mul_lo_u32 v28, v28, v26                                  ; d72c001c 0002351c
	v_cndmask_b32_e32 v29, v29, v30, vcc                        ; 023a3d1d
	v_cmp_le_i32_e32 vcc, s10, v32                              ; 7c86400a
	v_add3_u32 v1, v1, v24, v15                                 ; d6550001 043e3101
	v_and_b32_e32 v41, 15, v17                                  ; 3652228f
	v_bfe_u32 v42, v37, 8, 8                                    ; d610002a 02211125
	v_mul_lo_u32 v31, v31, v29                                  ; d72c001f 00023b1f
	v_cndmask_b32_e32 v32, v32, v33, vcc                        ; 02404320
	v_cmp_le_i32_e32 vcc, s10, v36                              ; 7c86480a
	v_add_nc_u32_e32 v41, -8, v41                               ; 4a5252c8
	v_add_nc_u32_e32 v43, s9, v42                               ; 4a565409
	v_bfe_u32 v44, v17, 8, 4                                    ; d610002c 02111111
	v_bfe_u32 v45, v37, 16, 8                                   ; d610002d 02212125
	v_mul_lo_u32 v34, v34, v32                                  ; d72c0022 00024122
	v_cndmask_b32_e32 v36, v36, v35, vcc                        ; 02484724
	v_add_nc_u32_e32 v44, -8, v44                               ; 4a5858c8
	v_add_nc_u32_e32 v46, s9, v45                               ; 4a5c5a09
	v_bfe_u32 v47, v17, 16, 4                                   ; d610002f 02112111
	s_delay_alu instid0(VALU_DEP_4)                             ; bf870004
	v_mul_lo_u32 v16, v16, v36                                  ; d72c0010 00024910
	v_bfe_u32 v36, v37, 0, 8                                    ; d6100024 02210125
	v_add3_u32 v34, v34, v28, v31                               ; d6550022 047e3922
	v_add_nc_u32_e32 v47, -8, v47                               ; 4a5e5ec8
	v_lshrrev_b32_e32 v37, 24, v37                              ; 324a4a98
	v_bfe_u32 v17, v17, 24, 4                                   ; d6100011 02113111
	v_add_nc_u32_e32 v40, s9, v36                               ; 4a504809
	v_cmp_le_i32_e32 vcc, s10, v36                              ; 7c86480a
	v_add3_u32 v1, v1, v34, v16                                 ; d6550001 04424501
	v_bfe_u32 v3, v38, 0, 8                                     ; d6100003 02210126
	v_add_nc_u32_e32 v2, s9, v37                                ; 4a044a09
	v_add_nc_u32_e32 v17, -8, v17                               ; 4a2222c8
	v_cndmask_b32_e32 v36, v36, v40, vcc                        ; 02485124
	v_cmp_le_i32_e32 vcc, s10, v42                              ; 7c86540a
	v_and_b32_e32 v11, 15, v18                                  ; 3616248f
	v_add_nc_u32_e32 v10, s9, v3                                ; 4a140609
	v_bfe_u32 v12, v38, 8, 8                                    ; d610000c 02211126
	v_mul_lo_u32 v41, v41, v36                                  ; d72c0029 00024929
	v_cndmask_b32_e32 v42, v42, v43, vcc                        ; 0254572a
	v_cmp_le_i32_e32 vcc, s10, v45                              ; 7c865a0a
	v_add_nc_u32_e32 v11, -8, v11                               ; 4a1616c8
	v_add_nc_u32_e32 v13, s9, v12                               ; 4a1a1809
	v_bfe_u32 v14, v18, 8, 4                                    ; d610000e 02111112
	v_bfe_u32 v15, v38, 16, 8                                   ; d610000f 02212126
	v_mul_lo_u32 v44, v44, v42                                  ; d72c002c 0002552c
	v_cndmask_b32_e32 v45, v45, v46, vcc                        ; 025a5d2d
	v_cmp_le_i32_e32 vcc, s10, v37                              ; 7c864a0a
	v_add_nc_u32_e32 v14, -8, v14                               ; 4a1c1cc8
	v_add_nc_u32_e32 v16, s9, v15                               ; 4a201e09
	s_delay_alu instid0(VALU_DEP_4) | instskip(SKIP_3) | instid1(VALU_DEP_3) ; bf8701c4
	v_mul_lo_u32 v47, v47, v45                                  ; d72c002f 00025b2f
	v_cndmask_b32_e32 v37, v37, v2, vcc                         ; 024a0525
	v_cmp_le_i32_e32 vcc, s10, v3                               ; 7c86060a
	v_lshrrev_b32_e32 v38, 24, v38                              ; 324c4c98
	v_mul_lo_u32 v17, v17, v37                                  ; d72c0011 00024b11
	v_cndmask_b32_e32 v3, v3, v10, vcc                          ; 02061503
	v_cmp_le_i32_e32 vcc, s10, v12                              ; 7c86180a
	v_add_nc_u32_e32 v24, s9, v38                               ; 4a304c09
	v_add3_u32 v47, v47, v41, v44                               ; d655002f 04b2532f
	s_delay_alu instid0(VALU_DEP_4)                             ; bf870004
	v_mul_lo_u32 v11, v11, v3                                   ; d72c000b 0002070b
	v_cndmask_b32_e32 v12, v12, v13, vcc                        ; 02181b0c
	v_cmp_le_i32_e32 vcc, s10, v15                              ; 7c861e0a
	v_add3_u32 v1, v1, v47, v17                                 ; d6550001 04465f01
	v_bfe_u32 v17, v18, 16, 4                                   ; d6100011 02112112
	v_bfe_u32 v18, v18, 24, 4                                   ; d6100012 02113112
	v_bfe_u32 v25, v39, 0, 8                                    ; d6100019 02210127
	v_and_b32_e32 v27, 15, v19                                  ; 3636268f
	v_mul_lo_u32 v14, v14, v12                                  ; d72c000e 0002190e
	v_cndmask_b32_e32 v15, v15, v16, vcc                        ; 021e210f
	v_cmp_le_i32_e32 vcc, s10, v38                              ; 7c864c0a
	v_bfe_u32 v28, v39, 8, 8                                    ; d610001c 02211127
	v_add_nc_u32_e32 v17, -8, v17                               ; 4a2222c8
	v_add_nc_u32_e32 v18, -8, v18                               ; 4a2424c8
	v_add_nc_u32_e32 v26, s9, v25                               ; 4a343209
	v_add_nc_u32_e32 v27, -8, v27                               ; 4a3636c8
	v_bfe_u32 v30, v19, 8, 4                                    ; d610001e 02111113
	v_bfe_u32 v31, v39, 16, 8                                   ; d610001f 02212127
	v_cndmask_b32_e32 v38, v38, v24, vcc                        ; 024c3126
	v_cmp_le_i32_e32 vcc, s10, v25                              ; 7c86320a
	v_add_nc_u32_e32 v29, s9, v28                               ; 4a3a3809
	v_mul_lo_u32 v17, v17, v15                                  ; d72c0011 00021f11
	v_bfe_u32 v33, v19, 16, 4                                   ; d6100021 02112113
	v_add_nc_u32_e32 v30, -8, v30                               ; 4a3c3cc8
	v_add_nc_u32_e32 v32, s9, v31                               ; 4a403e09
	v_mul_lo_u32 v18, v18, v38                                  ; d72c0012 00024d12
	v_cndmask_b32_e32 v25, v25, v26, vcc                        ; 02323519
	v_cmp_le_i32_e32 vcc, s10, v28                              ; 7c86380a
	v_lshrrev_b32_e32 v39, 24, v39                              ; 324e4e98
	v_add_nc_u32_e32 v33, -8, v33                               ; 4a4242c8
	v_add3_u32 v17, v17, v11, v14                               ; d6550011 043a1711
	v_bfe_u32 v19, v19, 24, 4                                   ; d6100013 02113113
	s_waitcnt vmcnt(0)                                          ; bf8903f7
	v_bfe_u32 v35, v20, 0, 8                                    ; d6100023 02210114
	v_mul_lo_u32 v27, v27, v25                                  ; d72c001b 0002331b
	v_cndmask_b32_e32 v28, v28, v29, vcc                        ; 02383b1c
	v_cmp_le_i32_e32 vcc, s10, v31                              ; 7c863e0a
	v_add_nc_u32_e32 v34, s9, v39                               ; 4a444e09
	v_add3_u32 v1, v1, v17, v18                                 ; d6550001 044a2301
	v_add_nc_u32_e32 v19, -8, v19                               ; 4a2626c8
	v_add_nc_u32_e32 v36, s9, v35                               ; 4a484609
	v_and_b32_e32 v37, 15, v4                                   ; 364a088f
	v_bfe_u32 v38, v20, 8, 8                                    ; d6100026 02211114
	v_mul_lo_u32 v30, v30, v28                                  ; d72c001e 0002391e
	v_cndmask_b32_e32 v31, v31, v32, vcc                        ; 023e411f
	v_cmp_le_i32_e32 vcc, s10, v39                              ; 7c864e0a
	v_bfe_u32 v40, v4, 8, 4                                     ; d6100028 02111104
	v_add_nc_u32_e32 v37, -8, v37                               ; 4a4a4ac8
	v_bfe_u32 v41, v20, 16, 8                                   ; d6100029 02212114
	v_mul_lo_u32 v33, v33, v31                                  ; d72c0021 00023f21
	v_cndmask_b32_e32 v39, v39, v34, vcc                        ; 024e4527
	v_cmp_le_i32_e32 vcc, s10, v35                              ; 7c86460a
	v_add_nc_u32_e32 v40, -8, v40                               ; 4a5050c8
	v_bfe_u32 v43, v4, 16, 4                                    ; d610002b 02112104
	v_add_nc_u32_e32 v42, s9, v41                               ; 4a545209
	v_lshrrev_b32_e32 v20, 24, v20                              ; 32282898
	v_mul_lo_u32 v19, v19, v39                                  ; d72c0013 00024f13
	v_add_nc_u32_e32 v39, s9, v38                               ; 4a4e4c09
	v_cndmask_b32_e32 v35, v35, v36, vcc                        ; 02464923
	v_cmp_le_i32_e32 vcc, s10, v38                              ; 7c864c0a
	v_add3_u32 v33, v33, v27, v30                               ; d6550021 047a3721
	v_add_nc_u32_e32 v43, -8, v43                               ; 4a5656c8
	v_add_nc_u32_e32 v44, s9, v20                               ; 4a582809
	v_bfe_u32 v4, v4, 24, 4                                     ; d6100004 02113104
	v_bfe_u32 v45, v21, 0, 8                                    ; d610002d 02210115
	v_mul_lo_u32 v37, v37, v35                                  ; d72c0025 00024725
	v_cndmask_b32_e32 v38, v38, v39, vcc                        ; 024c4f26
	v_cmp_le_i32_e32 vcc, s10, v41                              ; 7c86520a
	v_add3_u32 v1, v1, v33, v19                                 ; d6550001 044e4301
	v_and_b32_e32 v47, 15, v5                                   ; 365e0a8f
	v_add_nc_u32_e32 v4, -8, v4                                 ; 4a0808c8
	v_add_nc_u32_e32 v46, s9, v45                               ; 4a5c5a09
	v_bfe_u32 v2, v21, 8, 8                                     ; d6100002 02211115
	v_mul_lo_u32 v40, v40, v38                                  ; d72c0028 00024d28
	v_cndmask_b32_e32 v41, v41, v42, vcc                        ; 02525529
	v_cmp_le_i32_e32 vcc, s10, v20                              ; 7c86280a
	v_add_nc_u32_e32 v47, -8, v47                               ; 4a5e5ec8
	v_add_nc_u32_e32 v3, s9, v2                                 ; 4a060409
	v_bfe_u32 v10, v21, 16, 8                                   ; d610000a 02212115
	v_mul_lo_u32 v43, v43, v41                                  ; d72c002b 0002532b
	v_cndmask_b32_e32 v20, v20, v44, vcc                        ; 02285914
	v_cmp_le_i32_e32 vcc, s10, v45                              ; 7c865a0a
	v_bfe_u32 v12, v5, 16, 4                                    ; d610000c 02112105
	v_add_nc_u32_e32 v11, s9, v10                               ; 4a161409
	v_lshrrev_b32_e32 v21, 24, v21                              ; 322a2a98
	v_mul_lo_u32 v4, v4, v20                                    ; d72c0004 00022904
	v_cndmask_b32_e32 v45, v45, v46, vcc                        ; 025a5d2d
	v_cmp_le_i32_e32 vcc, s10, v2                               ; 7c86040a
	v_add3_u32 v43, v43, v37, v40                               ; d655002b 04a24b2b
	v_add_nc_u32_e32 v12, -8, v12                               ; 4a1818c8
	v_add_nc_u32_e32 v13, s9, v21                               ; 4a1a2a09
	v_mul_lo_u32 v47, v47, v45                                  ; d72c002f 00025b2f
	v_cndmask_b32_e32 v2, v2, v3, vcc                           ; 02040702
	v_cmp_le_i32_e32 vcc, s10, v10                              ; 7c86140a
	v_add3_u32 v1, v1, v43, v4                                  ; d6550001 04125701
	v_bfe_u32 v4, v5, 8, 4                                      ; d6100004 02111105
	v_bfe_u32 v5, v5, 24, 4                                     ; d6100005 02113105
	v_bfe_u32 v14, v22, 0, 8                                    ; d610000e 02210116
	v_and_b32_e32 v16, 15, v6                                   ; 36200c8f
	v_cndmask_b32_e32 v10, v10, v11, vcc                        ; 0214170a
	v_cmp_le_i32_e32 vcc, s10, v21                              ; 7c862a0a
	v_bfe_u32 v17, v22, 8, 8                                    ; d6100011 02211116
	v_add_nc_u32_e32 v4, -8, v4                                 ; 4a0808c8
	v_add_nc_u32_e32 v5, -8, v5                                 ; 4a0a0ac8
	v_add_nc_u32_e32 v15, s9, v14                               ; 4a1e1c09
	v_bfe_u32 v19, v6, 8, 4                                     ; d6100013 02111106
	v_add_nc_u32_e32 v16, -8, v16                               ; 4a2020c8
	v_mul_lo_u32 v12, v12, v10                                  ; d72c000c 0002150c
	v_cndmask_b32_e32 v21, v21, v13, vcc                        ; 022a1b15
	v_cmp_le_i32_e32 vcc, s10, v14                              ; 7c861c0a
	v_bfe_u32 v20, v22, 16, 8                                   ; d6100014 02212116
	v_add_nc_u32_e32 v18, s9, v17                               ; 4a242209
	v_mul_lo_u32 v4, v4, v2                                     ; d72c0004 00020504
	v_add_nc_u32_e32 v19, -8, v19                               ; 4a2626c8
	v_bfe_u32 v24, v6, 16, 4                                    ; d6100018 02112106
	v_mul_lo_u32 v5, v5, v21                                    ; d72c0005 00022b05
	v_cndmask_b32_e32 v14, v14, v15, vcc                        ; 021c1f0e
	v_cmp_le_i32_e32 vcc, s10, v17                              ; 7c86220a
	v_add_nc_u32_e32 v21, s9, v20                               ; 4a2a2809
	v_lshrrev_b32_e32 v22, 24, v22                              ; 322c2c98
	v_add_nc_u32_e32 v24, -8, v24                               ; 4a3030c8
	v_add3_u32 v12, v12, v47, v4                                ; d655000c 04125f0c
	v_bfe_u32 v6, v6, 24, 4                                     ; d6100006 02113106
	v_bfe_u32 v26, v23, 0, 8                                    ; d610001a 02210117
	v_mul_lo_u32 v16, v16, v14                                  ; d72c0010 00021d10
	v_cndmask_b32_e32 v17, v17, v18, vcc                        ; 02222511
	v_cmp_le_i32_e32 vcc, s10, v20                              ; 7c86280a
	v_add_nc_u32_e32 v25, s9, v22                               ; 4a322c09
	v_add3_u32 v1, v1, v12, v5                                  ; d6550001 04161901
	v_add_nc_u32_e32 v6, -8, v6                                 ; 4a0c0cc8
	v_and_b32_e32 v28, 15, v7                                   ; 36380e8f
	v_add_nc_u32_e32 v27, s9, v26                               ; 4a363409
	v_bfe_u32 v29, v23, 8, 8                                    ; d610001d 02211117
	v_mul_lo_u32 v19, v19, v17                                  ; d72c0013 00022313
	v_cndmask_b32_e32 v20, v20, v21, vcc                        ; 02282b14
	v_cmp_le_i32_e32 vcc, s10, v22                              ; 7c862c0a
	v_bfe_u32 v31, v7, 8, 4                                     ; d610001f 02111107
	v_add_nc_u32_e32 v28, -8, v28                               ; 4a3838c8
	v_add_nc_u32_e32 v30, s9, v29                               ; 4a3c3a09
	v_bfe_u32 v32, v23, 16, 8                                   ; d6100020 02212117
	v_mul_lo_u32 v24, v24, v20                                  ; d72c0018 00022918
	v_bfe_u32 v34, v7, 16, 4                                    ; d6100022 02112107
	v_cndmask_b32_e32 v22, v22, v25, vcc                        ; 022c3316
	v_cmp_le_i32_e32 vcc, s10, v26                              ; 7c86340a
	v_add_nc_u32_e32 v31, -8, v31                               ; 4a3e3ec8
	v_add_nc_u32_e32 v33, s9, v32                               ; 4a424009
	v_lshrrev_b32_e32 v23, 24, v23                              ; 322e2e98
	v_add_nc_u32_e32 v34, -8, v34                               ; 4a4444c8
	v_bfe_u32 v7, v7, 24, 4                                     ; d6100007 02113107
	v_mul_lo_u32 v6, v6, v22                                    ; d72c0006 00022d06
	v_add3_u32 v24, v24, v16, v19                               ; d6550018 044e2118
	v_cndmask_b32_e32 v26, v26, v27, vcc                        ; 0234371a
	v_cmp_le_i32_e32 vcc, s10, v29                              ; 7c863a0a
	v_add_nc_u32_e32 v35, s9, v23                               ; 4a462e09
	v_add_nc_u32_e32 v7, -8, v7                                 ; 4a0e0ec8
	s_delay_alu instid0(VALU_DEP_4) | instskip(SKIP_3) | instid1(VALU_DEP_3) ; bf8701c4
	v_mul_lo_u32 v28, v28, v26                                  ; d72c001c 0002351c
	v_cndmask_b32_e32 v29, v29, v30, vcc                        ; 023a3d1d
	v_cmp_le_i32_e32 vcc, s10, v32                              ; 7c86400a
	v_add3_u32 v1, v1, v24, v6                                  ; d6550001 041a3101
	v_mul_lo_u32 v31, v31, v29                                  ; d72c001f 00023b1f
	v_cndmask_b32_e32 v32, v32, v33, vcc                        ; 02404320
	v_cmp_le_i32_e32 vcc, s10, v23                              ; 7c862e0a
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a2
	v_mul_lo_u32 v34, v34, v32                                  ; d72c0022 00024122
	v_cndmask_b32_e32 v23, v23, v35, vcc                        ; 022e4717
	v_mul_lo_u32 v7, v7, v23                                    ; d72c0007 00022f07
	v_add3_u32 v34, v34, v28, v31                               ; d6550022 047e3922
	s_delay_alu instid0(VALU_DEP_1)                             ; bf870001
	v_add3_u32 v1, v1, v34, v7                                  ; d6550001 041e4501
	s_branch BB2                                                ; bfa0fcf2
BB6:
	v_mov_b32_e32 v2, 0                                         ; 7e040280
	global_load_b32 v2, v2, s[2:3] offset:32                    ; dc520020 02020002
	s_waitcnt vmcnt(0)                                          ; bf8903f7
	v_add_nc_u32_e32 v2, v2, v0                                 ; 4a040102
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_ashrrev_i32_e32 v3, 31, v2                                ; 3406049f
	v_lshlrev_b64 v[2:3], 2, v[2:3]                             ; d73c0002 00020482
	s_delay_alu instid0(VALU_DEP_1)                             ; bf870001
	v_add_co_u32 v4, vcc, v8, v2                                ; d7006a04 00020508
	s_waitcnt_depctr 0xfffd                                     ; bf88fffd
	v_add_co_ci_u32_e32 v5, vcc, v9, v3, vcc                    ; 400a0709
	global_load_b32 v3, v[4:5], off                             ; dc520000 037c0004
	s_waitcnt vmcnt(0)                                          ; bf8903f7
	v_add_nc_u32_e32 v3, v3, v1                                 ; 4a060303
	global_store_b32 v[4:5], v3, off                            ; dc6a0000 007c0304
BB11:
	s_endpgm                                                    ; bfb00000
 