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
	s_cbranch_execz BB11                                        ; bfa50383
BB1:
	v_mov_b32_e32 v1, 0                                         ; 7e020280
	s_mov_b32 s0, 0                                             ; be800080
BB2:
	v_mov_b32_e32 v2, 0                                         ; 7e040280
	global_load_b32 v2, v2, s[2:3] offset:28                    ; ee050002 00000002 00001c02
	s_wait_loadcnt 0x0                                          ; bfc00000
	v_readfirstlane_b32 s1, v2                                  ; 7e020502
	s_cmp_ge_u32 s0, s1                                         ; bf090100
	s_cbranch_scc1 BB6                                          ; bfa20362
BB5:
	v_mov_b32_e32 v2, 0                                         ; 7e040280
	global_load_b32 v2, v2, s[2:3] offset:36                    ; ee050002 00000002 00002402
	s_mul_i32 s1, s0, s7                                        ; 96010700
	s_wait_alu 0xfffe                                           ; bf88fffe
	v_add_lshl_u32 v3, s1, v0, 4                                ; d6470003 02120001
	s_wait_loadcnt 0x0                                          ; bfc00000
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_and_b32_e32 v2, v2, v3                                    ; 36040702
	v_lshrrev_b32_e32 v10, 2, v2                                ; 32140482
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_2) ; bf870121
	v_ashrrev_i32_e32 v11, 31, v10                              ; 3416149f
	v_add_nc_u32_e32 v5, 4, v2                                  ; 4a0a0484
	v_lshlrev_b64_e64 v[10:11], 4, v[10:11]                     ; d51f000a 00021484
	s_delay_alu instid0(VALU_DEP_1)                             ; bf870001
	v_add_co_u32 v12, vcc_lo, v4, v10                           ; d7006a0c 00021504
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_add_co_ci_u32_e32 v13, vcc_lo, s4, v11, vcc_lo            ; 401a1604
	v_add_co_u32 v14, vcc_lo, v6, v10                           ; d7006a0e 00021506
	v_lshrrev_b32_e32 v10, 2, v5                                ; 32140a82
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_add_co_ci_u32_e32 v15, vcc_lo, s5, v11, vcc_lo            ; 401e1605
	v_ashrrev_i32_e32 v11, 31, v10                              ; 3416149f
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_lshlrev_b64_e64 v[10:11], 4, v[10:11]                     ; d51f000a 00021484
	v_add_co_u32 v20, vcc_lo, v4, v10                           ; d7006a14 00021504
	v_add_nc_u32_e32 v7, 8, v2                                  ; 4a0e0488
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_add_co_ci_u32_e32 v21, vcc_lo, s4, v11, vcc_lo            ; 402a1604
	v_add_co_u32 v22, vcc_lo, v6, v10                           ; d7006a16 00021506
	v_lshrrev_b32_e32 v10, 2, v7                                ; 32140e82
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_add_co_ci_u32_e32 v23, vcc_lo, s5, v11, vcc_lo            ; 402e1605
	v_ashrrev_i32_e32 v11, 31, v10                              ; 3416149f
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_lshlrev_b64_e64 v[10:11], 4, v[10:11]                     ; d51f000a 00021484
	v_add_co_u32 v28, vcc_lo, v4, v10                           ; d7006a1c 00021504
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_add_co_ci_u32_e32 v29, vcc_lo, s4, v11, vcc_lo            ; 403a1604
	v_add_co_u32 v30, vcc_lo, v6, v10                           ; d7006a1e 00021506
	v_add_nc_u32_e32 v2, 12, v2                                 ; 4a04048c
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_add_co_ci_u32_e32 v31, vcc_lo, s5, v11, vcc_lo            ; 403e1605
	v_lshrrev_b32_e32 v2, 2, v2                                 ; 32040482
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_ashrrev_i32_e32 v3, 31, v2                                ; 3406049f
	v_lshlrev_b64_e64 v[2:3], 4, v[2:3]                         ; d51f0002 00020484
	s_delay_alu instid0(VALU_DEP_1)                             ; bf870001
	v_add_co_u32 v10, vcc_lo, v4, v2                            ; d7006a0a 00020504
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_add_co_ci_u32_e32 v11, vcc_lo, s4, v3, vcc_lo             ; 40160604
	v_add_co_u32 v36, vcc_lo, v6, v2                            ; d7006a24 00020506
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_add_co_ci_u32_e32 v37, vcc_lo, s5, v3, vcc_lo             ; 404a0605
	s_clause 0x7                                                ; bf850007
	global_load_b128 v[16:19], v[12:13], off                    ; ee05c07c 00000010 0000000c
	global_load_b128 v[12:15], v[14:15], off                    ; ee05c07c 0000000c 0000000e
	global_load_b128 v[24:27], v[20:21], off                    ; ee05c07c 00000018 00000014
	global_load_b128 v[20:23], v[22:23], off                    ; ee05c07c 00000014 00000016
	global_load_b128 v[32:35], v[28:29], off                    ; ee05c07c 00000020 0000001c
	global_load_b128 v[28:31], v[30:31], off                    ; ee05c07c 0000001c 0000001e
	global_load_b128 v[40:43], v[10:11], off                    ; ee05c07c 00000028 0000000a
	global_load_b128 v[36:39], v[36:37], off                    ; ee05c07c 00000024 00000024
	s_movk_i32 s8, 0xff00                                       ; b008ff00
	s_movk_i32 s9, 0x80                                         ; b0090080
	s_add_co_u32 s0, s0, 1                                      ; 80008100
	s_wait_loadcnt 0x7                                          ; bfc00007
	v_and_b32_e32 v9, 15, v16                                   ; 3612208f
	v_bfe_u32 v46, v17, 16, 4                                   ; d610002e 02112111
	v_bfe_u32 v10, v16, 8, 4                                    ; d610000a 02111110
	v_bfe_u32 v45, v17, 8, 4                                    ; d610002d 02111111
	v_bfe_u32 v11, v16, 16, 4                                   ; d610000b 02112110
	v_bfe_u32 v16, v16, 24, 4                                   ; d6100010 02113110
	v_and_b32_e32 v44, 15, v17                                  ; 3658228f
	v_bfe_u32 v17, v17, 24, 4                                   ; d6100011 02113111
	v_and_b32_e32 v47, 15, v18                                  ; 365e248f
	v_bfe_u32 v48, v18, 8, 4                                    ; d6100030 02111112
	v_bfe_u32 v49, v18, 16, 4                                   ; d6100031 02112112
	v_add_nc_u32_e32 v9, -8, v9                                 ; 4a1212c8
	v_add_nc_u32_e32 v46, -8, v46                               ; 4a5c5cc8
	v_bfe_u32 v18, v18, 24, 4                                   ; d6100012 02113112
	v_add_nc_u32_e32 v10, -8, v10                               ; 4a1414c8
	v_and_b32_e32 v50, 15, v19                                  ; 3664268f
	v_add_nc_u32_e32 v45, -8, v45                               ; 4a5a5ac8
	v_add_nc_u32_e32 v11, -8, v11                               ; 4a1616c8
	v_bfe_u32 v51, v19, 8, 4                                    ; d6100033 02111113
	v_add_nc_u32_e32 v16, -8, v16                               ; 4a2020c8
	v_add_nc_u32_e32 v44, -8, v44                               ; 4a5858c8
	v_add_nc_u32_e32 v17, -8, v17                               ; 4a2222c8
	v_bfe_u32 v52, v19, 16, 4                                   ; d6100034 02112113
	v_add_nc_u32_e32 v47, -8, v47                               ; 4a5e5ec8
	v_bfe_u32 v19, v19, 24, 4                                   ; d6100013 02113113
	v_add_nc_u32_e32 v48, -8, v48                               ; 4a6060c8
	s_wait_loadcnt 0x6                                          ; bfc00006
	v_bfe_u32 v53, v12, 0, 8                                    ; d6100035 0221010c
	v_add_nc_u32_e32 v49, -8, v49                               ; 4a6262c8
	v_add_nc_u32_e32 v18, -8, v18                               ; 4a2424c8
	v_bfe_u32 v55, v12, 8, 8                                    ; d6100037 0221110c
	v_add_nc_u32_e32 v50, -8, v50                               ; 4a6464c8
	v_bfe_u32 v57, v12, 16, 8                                   ; d6100039 0221210c
	v_add_nc_u32_e32 v51, -8, v51                               ; 4a6666c8
	v_lshrrev_b32_e32 v12, 24, v12                              ; 32181898
	v_bfe_u32 v60, v13, 0, 8                                    ; d610003c 0221010d
	v_add_nc_u32_e32 v19, -8, v19                               ; 4a2626c8
	s_wait_alu 0xfffe                                           ; bf88fffe
	v_cmp_le_i32_e32 vcc_lo, s9, v53                            ; 7c866a09
	v_add_nc_u32_e32 v54, s8, v53                               ; 4a6c6a08
	v_bfe_u32 v62, v13, 8, 8                                    ; d610003e 0221110d
	v_bfe_u32 v64, v13, 16, 8                                   ; d6100040 0221210d
	v_add_nc_u32_e32 v59, s8, v12                               ; 4a761808
	s_wait_alu 0xfffd                                           ; bf88fffd
	s_delay_alu instid0(VALU_DEP_4) | instskip(SKIP_1) | instid1(VALU_DEP_2) ; bf870124
	v_dual_cndmask_b32 v53, v53, v54 :: v_dual_add_nc_u32 v56, s8, v55 ; ca606d35 35386e08
	v_cmp_le_i32_e32 vcc_lo, s9, v55                            ; 7c866e09
	v_mul_lo_u32 v9, v9, v53                                    ; d72c0009 00026b09
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v55, v55, v56 :: v_dual_add_nc_u32 v58, s8, v57 ; ca607137 373a7208
	v_cmp_le_i32_e32 vcc_lo, s9, v57                            ; 7c867209
	v_bfe_u32 v67, v14, 0, 8                                    ; d6100043 0221010e
	v_bfe_u32 v69, v14, 8, 8                                    ; d6100045 0221110e
	v_bfe_u32 v71, v14, 16, 8                                   ; d6100047 0221210e
	v_lshrrev_b32_e32 v14, 24, v14                              ; 321c1c98
	v_lshrrev_b32_e32 v13, 24, v13                              ; 321a1a98
	v_mul_lo_u32 v10, v10, v55                                  ; d72c000a 00026f0a
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v57, v57, v58 :: v_dual_add_nc_u32 v52, -8, v52 ; ca607539 393468c8
	v_cmp_le_i32_e32 vcc_lo, s9, v12                            ; 7c861809
	v_add_nc_u32_e32 v68, s8, v67                               ; 4a888608
	v_add_nc_u32_e32 v66, s8, v13                               ; 4a841a08
	s_delay_alu instid0(VALU_DEP_4) | instskip(SKIP_3) | instid1(VALU_DEP_2) ; bf870144
	v_mul_lo_u32 v11, v11, v57                                  ; d72c000b 0002730b
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v12, v12, v59 :: v_dual_add_nc_u32 v61, s8, v60 ; ca60770c 0c3c7808
	v_cmp_le_i32_e32 vcc_lo, s9, v60                            ; 7c867809
	v_mul_lo_u32 v16, v16, v12                                  ; d72c0010 00021910
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v60, v60, v61 :: v_dual_add_nc_u32 v63, s8, v62 ; ca607b3c 3c3e7c08
	v_cmp_le_i32_e32 vcc_lo, s9, v62                            ; 7c867c09
	v_add3_u32 v11, v11, v9, v10                                ; d655000b 042a130b
	v_bfe_u32 v74, v15, 0, 8                                    ; d610004a 0221010f
	v_bfe_u32 v76, v15, 8, 8                                    ; d610004c 0221110f
	v_bfe_u32 v78, v15, 16, 8                                   ; d610004e 0221210f
	v_mul_lo_u32 v44, v44, v60                                  ; d72c002c 0002792c
	v_lshrrev_b32_e32 v15, 24, v15                              ; 321e1e98
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v62, v62, v63 :: v_dual_add_nc_u32 v65, s8, v64 ; ca607f3e 3e408008
	v_cmp_le_i32_e32 vcc_lo, s9, v64                            ; 7c868009
	v_add3_u32 v1, v1, v11, v16                                 ; d6550001 04421701
	s_wait_loadcnt 0x5                                          ; bfc00005
	v_bfe_u32 v82, v24, 8, 4                                    ; d6100052 02111118
	v_add_nc_u32_e32 v75, s8, v74                               ; 4a969408
	v_add_nc_u32_e32 v77, s8, v76                               ; 4a9a9808
	v_mul_lo_u32 v45, v45, v62                                  ; d72c002d 00027d2d
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v64, v64, v65 :: v_dual_add_nc_u32 v73, s8, v14 ; ca608340 40481c08
	v_cmp_le_i32_e32 vcc_lo, s9, v13                            ; 7c861a09
	v_add_nc_u32_e32 v79, s8, v78                               ; 4a9e9c08
	s_delay_alu instid0(VALU_DEP_3) | instskip(SKIP_4) | instid1(VALU_DEP_3) ; bf8701d3
	v_mul_lo_u32 v46, v46, v64                                  ; d72c002e 0002812e
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v13, v13, v66 :: v_dual_add_nc_u32 v70, s8, v69 ; ca60850d 0d468a08
	v_cmp_le_i32_e32 vcc_lo, s9, v67                            ; 7c868609
	v_bfe_u32 v83, v24, 16, 4                                   ; d6100053 02112118
	v_mul_lo_u32 v17, v17, v13                                  ; d72c0011 00021b11
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v67, v67, v68 :: v_dual_add_nc_u32 v72, s8, v71 ; ca608943 43488e08
	v_cmp_le_i32_e32 vcc_lo, s9, v69                            ; 7c868a09
	v_add3_u32 v46, v46, v44, v45                               ; d655002e 04b6592e
	s_delay_alu instid0(VALU_DEP_3)                             ; bf870003
	v_mul_lo_u32 v47, v47, v67                                  ; d72c002f 0002872f
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v69, v69, v70 :: v_dual_add_nc_u32 v80, s8, v15 ; ca608d45 45501e08
	v_cmp_le_i32_e32 vcc_lo, s9, v71                            ; 7c868e09
	v_add3_u32 v1, v1, v46, v17                                 ; d6550001 04465d01
	v_and_b32_e32 v84, 15, v25                                  ; 36a8328f
	s_delay_alu instid0(VALU_DEP_4)                             ; bf870004
	v_mul_lo_u32 v48, v48, v69                                  ; d72c0030 00028b30
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v71, v71, v72 :: v_dual_add_nc_u32 v82, -8, v82 ; ca609147 4752a4c8
	v_cmp_le_i32_e32 vcc_lo, s9, v14                            ; 7c861c09
	v_bfe_u32 v85, v25, 8, 4                                    ; d6100055 02111119
	v_add_nc_u32_e32 v84, -8, v84                               ; 4aa8a8c8
	s_delay_alu instid0(VALU_DEP_4)                             ; bf870004
	v_mul_lo_u32 v49, v49, v71                                  ; d72c0031 00028f31
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v14, v14, v73 :: v_dual_and_b32 v81, 15, v24 ; ca64930e 0e50308f
	v_bfe_u32 v24, v24, 24, 4                                   ; d6100018 02113118
	v_cmp_le_i32_e32 vcc_lo, s9, v74                            ; 7c869409
	v_bfe_u32 v86, v25, 16, 4                                   ; d6100056 02112119
	s_delay_alu instid0(VALU_DEP_4)                             ; bf870004
	v_mul_lo_u32 v18, v18, v14                                  ; d72c0012 00021d12
	v_add_nc_u32_e32 v24, -8, v24                               ; 4a3030c8
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v74, v74, v75 :: v_dual_add_nc_u32 v81, -8, v81 ; ca60974a 4a50a2c8
	v_cmp_le_i32_e32 vcc_lo, s9, v76                            ; 7c869809
	v_add3_u32 v49, v49, v47, v48                               ; d6550031 04c25f31
	v_bfe_u32 v25, v25, 24, 4                                   ; d6100019 02113119
	v_and_b32_e32 v87, 15, v26                                  ; 36ae348f
	v_bfe_u32 v88, v26, 8, 4                                    ; d6100058 0211111a
	v_bfe_u32 v89, v26, 16, 4                                   ; d6100059 0211211a
	v_mul_lo_u32 v50, v50, v74                                  ; d72c0032 00029532
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v76, v76, v77 :: v_dual_add_nc_u32 v83, -8, v83 ; ca609b4c 4c52a6c8
	v_cmp_le_i32_e32 vcc_lo, s9, v78                            ; 7c869c09
	v_add3_u32 v1, v1, v49, v18                                 ; d6550001 044a6301
	v_add_nc_u32_e32 v25, -8, v25                               ; 4a3232c8
	v_add_nc_u32_e32 v87, -8, v87                               ; 4aaeaec8
	v_add_nc_u32_e32 v88, -8, v88                               ; 4ab0b0c8
	v_bfe_u32 v26, v26, 24, 4                                   ; d610001a 0211311a
	v_add_nc_u32_e32 v89, -8, v89                               ; 4ab2b2c8
	v_and_b32_e32 v90, 15, v27                                  ; 36b4368f
	v_bfe_u32 v91, v27, 8, 4                                    ; d610005b 0211111b
	s_wait_loadcnt 0x4                                          ; bfc00004
	v_bfe_u32 v93, v20, 0, 8                                    ; d610005d 02210114
	v_mul_lo_u32 v51, v51, v76                                  ; d72c0033 00029933
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v78, v78, v79 :: v_dual_add_nc_u32 v85, -8, v85 ; ca609f4e 4e54aac8
	v_cmp_le_i32_e32 vcc_lo, s9, v15                            ; 7c861e09
	v_bfe_u32 v92, v27, 16, 4                                   ; d610005c 0211211b
	v_bfe_u32 v27, v27, 24, 4                                   ; d610001b 0211311b
	v_bfe_u32 v95, v20, 8, 8                                    ; d610005f 02211114
	v_add_nc_u32_e32 v26, -8, v26                               ; 4a3434c8
	v_add_nc_u32_e32 v91, -8, v91                               ; 4ab6b6c8
	v_add_nc_u32_e32 v94, s8, v93                               ; 4abcba08
	v_add_nc_u32_e32 v90, -8, v90                               ; 4ab4b4c8
	v_bfe_u32 v3, v20, 16, 8                                    ; d6100003 02212114
	v_mul_lo_u32 v52, v52, v78                                  ; d72c0034 00029d34
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v15, v15, v80 :: v_dual_add_nc_u32 v86, -8, v86 ; ca60a10f 0f56acc8
	v_cmp_le_i32_e32 vcc_lo, s9, v93                            ; 7c86ba09
	v_lshrrev_b32_e32 v20, 24, v20                              ; 32282898
	v_add_nc_u32_e32 v27, -8, v27                               ; 4a3636c8
	v_bfe_u32 v9, v21, 0, 8                                     ; d6100009 02210115
	v_add_nc_u32_e32 v5, s8, v3                                 ; 4a0a0608
	v_bfe_u32 v11, v21, 8, 8                                    ; d610000b 02211115
	v_mul_lo_u32 v19, v19, v15                                  ; d72c0013 00021f13
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v93, v93, v94 :: v_dual_add_nc_u32 v2, s8, v95 ; ca60bd5d 5d02be08
	v_cmp_le_i32_e32 vcc_lo, s9, v95                            ; 7c86be09
	v_add3_u32 v52, v52, v50, v51                               ; d6550034 04ce6534
	v_add_nc_u32_e32 v7, s8, v20                                ; 4a0e2808
	v_add_nc_u32_e32 v10, s8, v9                                ; 4a141208
	v_bfe_u32 v13, v21, 16, 8                                   ; d610000d 02212115
	v_mul_lo_u32 v81, v81, v93                                  ; d72c0051 0002bb51
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v95, v95, v2, vcc_lo                      ; 02be055f
	v_cmp_le_i32_e32 vcc_lo, s9, v3                             ; 7c860609
	v_add3_u32 v1, v1, v52, v19                                 ; d6550001 044e6901
	v_lshrrev_b32_e32 v21, 24, v21                              ; 322a2a98
	s_delay_alu instid0(VALU_DEP_4)                             ; bf870004
	v_mul_lo_u32 v82, v82, v95                                  ; d72c0052 0002bf52
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v3, v3, v5 :: v_dual_add_nc_u32 v92, -8, v92 ; ca600b03 035cb8c8
	v_cmp_le_i32_e32 vcc_lo, s9, v20                            ; 7c862809
	v_add_nc_u32_e32 v15, s8, v21                               ; 4a1e2a08
	v_bfe_u32 v16, v22, 0, 8                                    ; d6100010 02210116
	v_bfe_u32 v18, v22, 8, 8                                    ; d6100012 02211116
	v_mul_lo_u32 v83, v83, v3                                   ; d72c0053 00020753
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v20, v20, v7, vcc_lo                      ; 02280f14
	v_cmp_le_i32_e32 vcc_lo, s9, v9                             ; 7c861209
	v_add_nc_u32_e32 v17, s8, v16                               ; 4a222008
	s_delay_alu instid0(VALU_DEP_3)                             ; bf870003
	v_mul_lo_u32 v24, v24, v20                                  ; d72c0018 00022918
	v_bfe_u32 v20, v22, 16, 8                                   ; d6100014 02212116
	v_lshrrev_b32_e32 v22, 24, v22                              ; 322c2c98
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v9, v9, v10 :: v_dual_add_nc_u32 v12, s8, v11 ; ca601509 090c1608
	v_cmp_le_i32_e32 vcc_lo, s9, v11                            ; 7c861609
	v_add3_u32 v83, v83, v81, v82                               ; d6550053 054aa353
	s_delay_alu instid0(VALU_DEP_3)                             ; bf870003
	v_mul_lo_u32 v84, v84, v9                                   ; d72c0054 00021354
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v11, v11, v12 :: v_dual_add_nc_u32 v14, s8, v13 ; ca60190b 0b0e1a08
	v_cmp_le_i32_e32 vcc_lo, s9, v13                            ; 7c861a09
	v_add3_u32 v1, v1, v83, v24                                 ; d6550001 0462a701
	v_bfe_u32 v44, v23, 8, 8                                    ; d610002c 02211117
	s_delay_alu instid0(VALU_DEP_4) | instskip(SKIP_4) | instid1(VALU_DEP_3) ; bf8701d4
	v_mul_lo_u32 v85, v85, v11                                  ; d72c0055 00021755
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v13, v13, v14, vcc_lo                     ; 021a1d0d
	v_cmp_le_i32_e32 vcc_lo, s9, v21                            ; 7c862a09
	v_bfe_u32 v46, v23, 16, 8                                   ; d610002e 02212117
	v_mul_lo_u32 v86, v86, v13                                  ; d72c0056 00021b56
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v21, v21, v15 :: v_dual_add_nc_u32 v24, s8, v22 ; ca601f15 15182c08
	v_cmp_le_i32_e32 vcc_lo, s9, v16                            ; 7c862009
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_4) | instid1(VALU_DEP_3) ; bf8701d2
	v_mul_lo_u32 v25, v25, v21                                  ; d72c0019 00022b19
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v16, v16, v17 :: v_dual_add_nc_u32 v19, s8, v18 ; ca602310 10122408
	v_cmp_le_i32_e32 vcc_lo, s9, v18                            ; 7c862409
	v_add3_u32 v86, v86, v84, v85                               ; d6550056 0556a956
	v_mul_lo_u32 v87, v87, v16                                  ; d72c0057 00022157
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v18, v18, v19 :: v_dual_add_nc_u32 v21, s8, v20 ; ca602712 12142808
	v_cmp_le_i32_e32 vcc_lo, s9, v20                            ; 7c862809
	v_add3_u32 v1, v1, v86, v25                                 ; d6550001 0466ad01
	v_bfe_u32 v25, v23, 0, 8                                    ; d6100019 02210117
	v_lshrrev_b32_e32 v23, 24, v23                              ; 322e2e98
	s_wait_loadcnt 0x2                                          ; bfc00002
	v_bfe_u32 v49, v28, 0, 8                                    ; d6100031 0221011c
	v_bfe_u32 v52, v28, 8, 8                                    ; d6100034 0221111c
	v_mul_lo_u32 v88, v88, v18                                  ; d72c0058 00022558
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v20, v20, v21 :: v_dual_add_nc_u32 v45, s8, v44 ; ca602b14 142c5808
	v_cmp_le_i32_e32 vcc_lo, s9, v22                            ; 7c862c09
	v_add_nc_u32_e32 v48, s8, v23                               ; 4a602e08
	v_bfe_u32 v55, v28, 16, 8                                   ; d6100037 0221211c
	s_delay_alu instid0(VALU_DEP_4) | instskip(SKIP_4) | instid1(VALU_DEP_3) ; bf8701d4
	v_mul_lo_u32 v89, v89, v20                                  ; d72c0059 00022959
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v22, v22, v24 :: v_dual_add_nc_u32 v47, s8, v46 ; ca603116 162e5c08
	v_cmp_le_i32_e32 vcc_lo, s9, v25                            ; 7c863209
	v_lshrrev_b32_e32 v28, 24, v28                              ; 32383898
	v_mul_lo_u32 v26, v26, v22                                  ; d72c001a 00022d1a
	v_add3_u32 v89, v89, v87, v88                               ; d6550059 0562af59
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_2) | instid1(VALU_DEP_1) ; bf8700b1
	v_add3_u32 v1, v1, v89, v26                                 ; d6550001 046ab301
	v_add_nc_u32_e32 v26, s8, v25                               ; 4a343208
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v25, v25, v26 :: v_dual_add_nc_u32 v50, s8, v49 ; ca603519 19326208
	v_cmp_le_i32_e32 vcc_lo, s9, v44                            ; 7c865809
	v_bfe_u32 v54, v32, 8, 4                                    ; d6100036 02111120
	s_delay_alu instid0(VALU_DEP_3)                             ; bf870003
	v_mul_lo_u32 v90, v90, v25                                  ; d72c005a 0002335a
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v44, v44, v45 :: v_dual_and_b32 v51, 15, v32 ; ca645b2c 2c32408f
	v_cmp_le_i32_e32 vcc_lo, s9, v46                            ; 7c865c09
	v_bfe_u32 v57, v32, 16, 4                                   ; d6100039 02112120
	v_and_b32_e32 v61, 15, v33                                  ; 367a428f
	s_delay_alu instid0(VALU_DEP_4)                             ; bf870004
	v_mul_lo_u32 v91, v91, v44                                  ; d72c005b 0002595b
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v46, v46, v47 :: v_dual_add_nc_u32 v53, s8, v52 ; ca605f2e 2e346808
	v_cmp_le_i32_e32 vcc_lo, s9, v23                            ; 7c862e09
	v_add_nc_u32_e32 v57, -8, v57                               ; 4a7272c8
	v_bfe_u32 v32, v32, 24, 4                                   ; d6100020 02113120
	s_delay_alu instid0(VALU_DEP_4)                             ; bf870004
	v_mul_lo_u32 v92, v92, v46                                  ; d72c005c 00025d5c
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v23, v23, v48 :: v_dual_add_nc_u32 v56, s8, v55 ; ca606117 17386e08
	v_cmp_le_i32_e32 vcc_lo, s9, v49                            ; 7c866209
	v_add_nc_u32_e32 v32, -8, v32                               ; 4a4040c8
	v_bfe_u32 v59, v29, 0, 8                                    ; d610003b 0221011d
	v_bfe_u32 v62, v29, 8, 8                                    ; d610003e 0221111d
	v_bfe_u32 v64, v33, 8, 4                                    ; d6100040 02111121
	v_mul_lo_u32 v27, v27, v23                                  ; d72c001b 00022f1b
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v49, v49, v50 :: v_dual_add_nc_u32 v58, s8, v28 ; ca606531 313a3808
	v_cmp_le_i32_e32 vcc_lo, s9, v52                            ; 7c866809
	v_add3_u32 v92, v92, v90, v91                               ; d655005c 056eb55c
	v_add_nc_u32_e32 v60, s8, v59                               ; 4a787608
	v_add_nc_u32_e32 v63, s8, v62                               ; 4a7e7c08
	v_bfe_u32 v65, v29, 16, 8                                   ; d6100041 0221211d
	v_add_nc_u32_e32 v64, -8, v64                               ; 4a8080c8
	v_lshrrev_b32_e32 v29, 24, v29                              ; 323a3a98
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v52, v52, v53 :: v_dual_add_nc_u32 v51, -8, v51 ; ca606b34 343266c8
	v_cmp_le_i32_e32 vcc_lo, s9, v55                            ; 7c866e09
	v_add3_u32 v1, v1, v92, v27                                 ; d6550001 046eb901
	v_bfe_u32 v67, v33, 16, 4                                   ; d6100043 02112121
	v_bfe_u32 v33, v33, 24, 4                                   ; d6100021 02113121
	v_add_nc_u32_e32 v66, s8, v65                               ; 4a848208
	v_mul_lo_u32 v51, v51, v49                                  ; d72c0033 00026333
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v55, v55, v56 :: v_dual_add_nc_u32 v54, -8, v54 ; ca607137 37366cc8
	v_cmp_le_i32_e32 vcc_lo, s9, v28                            ; 7c863809
	v_add_nc_u32_e32 v67, -8, v67                               ; 4a8686c8
	v_bfe_u32 v69, v30, 0, 8                                    ; d6100045 0221011e
	v_bfe_u32 v74, v34, 8, 4                                    ; d610004a 02111122
	v_mul_lo_u32 v54, v54, v52                                  ; d72c0036 00026936
	v_mul_lo_u32 v57, v57, v55                                  ; d72c0039 00026f39
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v28, v28, v58 :: v_dual_add_nc_u32 v61, -8, v61 ; ca60751c 1c3c7ac8
	v_cmp_le_i32_e32 vcc_lo, s9, v59                            ; 7c867609
	v_bfe_u32 v75, v30, 16, 8                                   ; d610004b 0221211e
	v_and_b32_e32 v71, 15, v34                                  ; 368e448f
	s_delay_alu instid0(VALU_DEP_4)                             ; bf870004
	v_mul_lo_u32 v32, v32, v28                                  ; d72c0020 00023920
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v59, v59, v60 :: v_dual_add_nc_u32 v68, s8, v29 ; ca60793b 3b443a08
	v_cmp_le_i32_e32 vcc_lo, s9, v62                            ; 7c867c09
	v_add3_u32 v57, v57, v51, v54                               ; d6550039 04da6739
	v_bfe_u32 v72, v30, 8, 8                                    ; d6100048 0221111e
	s_delay_alu instid0(VALU_DEP_4)                             ; bf870004
	v_mul_lo_u32 v61, v61, v59                                  ; d72c003d 0002773d
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v62, v62, v63 :: v_dual_add_nc_u32 v33, -8, v33 ; ca607f3e 3e2042c8
	v_cmp_le_i32_e32 vcc_lo, s9, v65                            ; 7c868209
	v_add3_u32 v1, v1, v57, v32                                 ; d6550001 04827301
	v_add_nc_u32_e32 v73, s8, v72                               ; 4a929008
	v_lshrrev_b32_e32 v30, 24, v30                              ; 323c3c98
	v_bfe_u32 v77, v34, 16, 4                                   ; d610004d 02112122
	v_mul_lo_u32 v64, v64, v62                                  ; d72c0040 00027d40
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v65, v65, v66 :: v_dual_add_nc_u32 v70, s8, v69 ; ca608541 41468a08
	v_cmp_le_i32_e32 vcc_lo, s9, v29                            ; 7c863a09
	v_add_nc_u32_e32 v77, -8, v77                               ; 4a9a9ac8
	v_bfe_u32 v79, v31, 0, 8                                    ; d610004f 0221011f
	s_delay_alu instid0(VALU_DEP_4)                             ; bf870004
	v_mul_lo_u32 v67, v67, v65                                  ; d72c0043 00028343
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v29, v29, v68 :: v_dual_add_nc_u32 v74, -8, v74 ; ca60891d 1d4a94c8
	v_cmp_le_i32_e32 vcc_lo, s9, v69                            ; 7c868a09
	v_add_nc_u32_e32 v80, s8, v79                               ; 4aa09e08
	v_bfe_u32 v34, v34, 24, 4                                   ; d6100022 02113122
	v_bfe_u32 v82, v31, 8, 8                                    ; d6100052 0221111f
	v_mul_lo_u32 v33, v33, v29                                  ; d72c0021 00023b21
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v69, v69, v70 :: v_dual_add_nc_u32 v76, s8, v75 ; ca608d45 454c9608
	v_cmp_le_i32_e32 vcc_lo, s9, v72                            ; 7c869009
	v_add3_u32 v67, v67, v61, v64                               ; d6550043 05027b43
	v_add_nc_u32_e32 v83, s8, v82                               ; 4aa6a408
	v_bfe_u32 v84, v35, 8, 4                                    ; d6100054 02111123
	v_bfe_u32 v85, v31, 16, 8                                   ; d6100055 0221211f
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v72, v72, v73 :: v_dual_add_nc_u32 v71, -8, v71 ; ca609348 48468ec8
	v_cmp_le_i32_e32 vcc_lo, s9, v75                            ; 7c869609
	v_add3_u32 v1, v1, v67, v33                                 ; d6550001 04868701
	v_lshrrev_b32_e32 v31, 24, v31                              ; 323e3e98
	v_add_nc_u32_e32 v84, -8, v84                               ; 4aa8a8c8
	v_add_nc_u32_e32 v86, s8, v85                               ; 4aacaa08
	v_bfe_u32 v87, v35, 16, 4                                   ; d6100057 02112123
	v_mul_lo_u32 v71, v71, v69                                  ; d72c0047 00028b47
	v_mul_lo_u32 v74, v74, v72                                  ; d72c004a 0002914a
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v75, v75, v76 :: v_dual_add_nc_u32 v78, s8, v30 ; ca60994b 4b4e3c08
	v_cmp_le_i32_e32 vcc_lo, s9, v30                            ; 7c863c09
	v_add_nc_u32_e32 v88, s8, v31                               ; 4ab03e08
	v_add_nc_u32_e32 v87, -8, v87                               ; 4aaeaec8
	s_wait_loadcnt 0x0                                          ; bfc00000
	v_bfe_u32 v89, v36, 0, 8                                    ; d6100059 02210124
	v_mul_lo_u32 v77, v77, v75                                  ; d72c004d 0002974d
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v30, v30, v78 :: v_dual_and_b32 v81, 15, v35 ; ca649d1e 1e50468f
	v_cmp_le_i32_e32 vcc_lo, s9, v79                            ; 7c869e09
	v_bfe_u32 v35, v35, 24, 4                                   ; d6100023 02113123
	v_bfe_u32 v94, v40, 8, 4                                    ; d610005e 02111128
	v_bfe_u32 v95, v36, 16, 8                                   ; d610005f 02212124
	v_and_b32_e32 v91, 15, v40                                  ; 36b6508f
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v79, v79, v80 :: v_dual_add_nc_u32 v34, -8, v34 ; ca60a14f 4f2244c8
	v_cmp_le_i32_e32 vcc_lo, s9, v82                            ; 7c86a409
	v_add3_u32 v77, v77, v71, v74                               ; d655004d 052a8f4d
	v_add_nc_u32_e32 v35, -8, v35                               ; 4a4646c8
	v_bfe_u32 v92, v36, 8, 8                                    ; d610005c 02211124
	v_add_nc_u32_e32 v91, -8, v91                               ; 4ab6b6c8
	v_bfe_u32 v3, v40, 16, 4                                    ; d6100003 02112128
	v_mul_lo_u32 v34, v34, v30                                  ; d72c0022 00023d22
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v82, v82, v83 :: v_dual_add_nc_u32 v81, -8, v81 ; ca60a752 5250a2c8
	v_cmp_le_i32_e32 vcc_lo, s9, v85                            ; 7c86aa09
	v_add_nc_u32_e32 v93, s8, v92                               ; 4abab808
	v_add_nc_u32_e32 v3, -8, v3                                 ; 4a0606c8
	v_lshrrev_b32_e32 v36, 24, v36                              ; 32484898
	v_mul_lo_u32 v84, v84, v82                                  ; d72c0054 0002a554
	v_mul_lo_u32 v81, v81, v79                                  ; d72c0051 00029f51
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v85, v85, v86 :: v_dual_add_nc_u32 v90, s8, v89 ; ca60ad55 555ab208
	v_cmp_le_i32_e32 vcc_lo, s9, v31                            ; 7c863e09
	v_add3_u32 v1, v1, v77, v34                                 ; d6550001 048a9b01
	v_bfe_u32 v40, v40, 24, 4                                   ; d6100028 02113128
	v_bfe_u32 v7, v37, 0, 8                                     ; d6100007 02210125
	v_and_b32_e32 v10, 15, v41                                  ; 3614528f
	v_mul_lo_u32 v87, v87, v85                                  ; d72c0057 0002ab57
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v31, v31, v88 :: v_dual_add_nc_u32 v94, -8, v94 ; ca60b11f 1f5ebcc8
	v_cmp_le_i32_e32 vcc_lo, s9, v89                            ; 7c86b209
	v_bfe_u32 v11, v37, 8, 8                                    ; d610000b 02211125
	v_bfe_u32 v13, v41, 8, 4                                    ; d610000d 02111129
	s_delay_alu instid0(VALU_DEP_4)                             ; bf870004
	v_mul_lo_u32 v35, v35, v31                                  ; d72c0023 00023f23
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v89, v89, v90 :: v_dual_add_nc_u32 v2, s8, v95 ; ca60b559 5902be08
	v_cmp_le_i32_e32 vcc_lo, s9, v92                            ; 7c86b809
	v_add3_u32 v87, v87, v81, v84                               ; d6550057 0552a357
	v_add_nc_u32_e32 v13, -8, v13                               ; 4a1a1ac8
	v_bfe_u32 v14, v37, 16, 8                                   ; d610000e 02212125
	v_mul_lo_u32 v91, v91, v89                                  ; d72c005b 0002b35b
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v92, v92, v93 :: v_dual_add_nc_u32 v5, s8, v36 ; ca60bb5c 5c044808
	v_cmp_le_i32_e32 vcc_lo, s9, v95                            ; 7c86be09
	v_add3_u32 v1, v1, v87, v35                                 ; d6550001 048eaf01
	v_add_nc_u32_e32 v15, s8, v14                               ; 4a1e1c08
	v_lshrrev_b32_e32 v37, 24, v37                              ; 324a4a98
	v_bfe_u32 v16, v41, 16, 4                                   ; d6100010 02112129
	v_mul_lo_u32 v94, v94, v92                                  ; d72c005e 0002b95e
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v95, v95, v2 :: v_dual_add_nc_u32 v40, -8, v40 ; ca60055f 5f2850c8
	v_cmp_le_i32_e32 vcc_lo, s9, v36                            ; 7c864809
	v_add_nc_u32_e32 v16, -8, v16                               ; 4a2020c8
	v_bfe_u32 v18, v38, 0, 8                                    ; d6100012 02210126
	s_delay_alu instid0(VALU_DEP_4)                             ; bf870004
	v_mul_lo_u32 v3, v3, v95                                    ; d72c0003 0002bf03
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v36, v36, v5 :: v_dual_add_nc_u32 v9, s8, v7 ; ca600b24 24080e08
	v_cmp_le_i32_e32 vcc_lo, s9, v7                             ; 7c860e09
	v_add_nc_u32_e32 v19, s8, v18                               ; 4a262408
	v_bfe_u32 v41, v41, 24, 4                                   ; d6100029 02113129
	v_bfe_u32 v21, v38, 8, 8                                    ; d6100015 02211126
	v_mul_lo_u32 v40, v40, v36                                  ; d72c0028 00024928
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v7, v7, v9 :: v_dual_add_nc_u32 v12, s8, v11 ; ca601307 070c1608
	v_cmp_le_i32_e32 vcc_lo, s9, v11                            ; 7c861609
	v_add3_u32 v3, v3, v91, v94                                 ; d6550003 057ab703
	v_add_nc_u32_e32 v22, s8, v21                               ; 4a2c2a08
	v_bfe_u32 v23, v42, 8, 4                                    ; d6100017 0211112a
	v_bfe_u32 v24, v38, 16, 8                                   ; d6100018 02212126
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v11, v11, v12 :: v_dual_add_nc_u32 v10, -8, v10 ; ca60190b 0b0a14c8
	v_cmp_le_i32_e32 vcc_lo, s9, v14                            ; 7c861c09
	v_add3_u32 v1, v1, v3, v40                                  ; d6550001 04a20701
	v_add_nc_u32_e32 v23, -8, v23                               ; 4a2e2ec8
	v_lshrrev_b32_e32 v38, 24, v38                              ; 324c4c98
	v_add_nc_u32_e32 v25, s8, v24                               ; 4a323008
	v_bfe_u32 v26, v42, 16, 4                                   ; d610001a 0211212a
	v_mul_lo_u32 v13, v13, v11                                  ; d72c000d 0002170d
	v_mul_lo_u32 v10, v10, v7                                   ; d72c000a 00020f0a
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v14, v14, v15 :: v_dual_add_nc_u32 v17, s8, v37 ; ca601f0e 0e104a08
	v_cmp_le_i32_e32 vcc_lo, s9, v37                            ; 7c864a09
	v_add_nc_u32_e32 v27, s8, v38                               ; 4a364c08
	v_add_nc_u32_e32 v26, -8, v26                               ; 4a3434c8
	v_bfe_u32 v28, v39, 0, 8                                    ; d610001c 02210127
	v_mul_lo_u32 v16, v16, v14                                  ; d72c0010 00021d10
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v37, v37, v17 :: v_dual_and_b32 v20, 15, v42 ; ca642325 2514548f
	v_cmp_le_i32_e32 vcc_lo, s9, v18                            ; 7c862409
	v_bfe_u32 v42, v42, 24, 4                                   ; d610002a 0211312a
	v_bfe_u32 v34, v39, 16, 8                                   ; d6100022 02212127
	v_and_b32_e32 v30, 15, v43                                  ; 363c568f
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v18, v18, v19 :: v_dual_add_nc_u32 v41, -8, v41 ; ca602712 122852c8
	v_cmp_le_i32_e32 vcc_lo, s9, v21                            ; 7c862a09
	v_add_nc_u32_e32 v42, -8, v42                               ; 4a5454c8
	v_add3_u32 v16, v16, v10, v13                               ; d6550010 04361510
	v_bfe_u32 v31, v39, 8, 8                                    ; d610001f 02211127
	v_lshrrev_b32_e32 v39, 24, v39                              ; 324e4e98
	v_mul_lo_u32 v41, v41, v37                                  ; d72c0029 00024b29
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v21, v21, v22 :: v_dual_add_nc_u32 v20, -8, v20 ; ca602d15 151428c8
	v_cmp_le_i32_e32 vcc_lo, s9, v24                            ; 7c863009
	v_add_nc_u32_e32 v32, s8, v31                               ; 4a403e08
	v_bfe_u32 v33, v43, 8, 4                                    ; d6100021 0211112b
	s_delay_alu instid0(VALU_DEP_4)                             ; bf870004
	v_mul_lo_u32 v23, v23, v21                                  ; d72c0017 00022b17
	v_mul_lo_u32 v20, v20, v18                                  ; d72c0014 00022514
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v24, v24, v25 :: v_dual_add_nc_u32 v29, s8, v28 ; ca603318 181c3808
	v_cmp_le_i32_e32 vcc_lo, s9, v38                            ; 7c864c09
	v_add3_u32 v1, v1, v16, v41                                 ; d6550001 04a62101
	v_bfe_u32 v36, v43, 16, 4                                   ; d6100024 0211212b
	v_bfe_u32 v43, v43, 24, 4                                   ; d610002b 0211312b
	v_mul_lo_u32 v26, v26, v24                                  ; d72c001a 0002311a
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v38, v38, v27 :: v_dual_add_nc_u32 v35, s8, v34 ; ca603726 26224408
	v_cmp_le_i32_e32 vcc_lo, s9, v28                            ; 7c863809
	v_add_nc_u32_e32 v43, -8, v43                               ; 4a5656c8
	s_delay_alu instid0(VALU_DEP_3)                             ; bf870003
	v_mul_lo_u32 v42, v42, v38                                  ; d72c002a 00024d2a
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v28, v28, v29 :: v_dual_add_nc_u32 v37, s8, v39 ; ca603b1c 1c244e08
	v_cmp_le_i32_e32 vcc_lo, s9, v31                            ; 7c863e09
	v_add3_u32 v26, v26, v20, v23                               ; d655001a 045e291a
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v31, v31, v32 :: v_dual_add_nc_u32 v30, -8, v30 ; ca60411f 1f1e3cc8
	v_cmp_le_i32_e32 vcc_lo, s9, v34                            ; 7c864409
	v_add3_u32 v1, v1, v26, v42                                 ; d6550001 04aa3501
	s_delay_alu instid0(VALU_DEP_3) | instskip(SKIP_3) | instid1(VALU_DEP_2) ; bf870143
	v_mul_lo_u32 v30, v30, v28                                  ; d72c001e 0002391e
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v34, v34, v35 :: v_dual_add_nc_u32 v33, -8, v33 ; ca604722 222042c8
	v_cmp_le_i32_e32 vcc_lo, s9, v39                            ; 7c864e09
	v_mul_lo_u32 v33, v33, v31                                  ; d72c0021 00023f21
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_dual_cndmask_b32 v39, v39, v37 :: v_dual_add_nc_u32 v36, -8, v36 ; ca604b27 272448c8
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_2) | instid1(VALU_DEP_1) ; bf8700b1
	v_mul_lo_u32 v36, v36, v34                                  ; d72c0024 00024524
	v_mul_lo_u32 v43, v43, v39                                  ; d72c002b 00024f2b
	v_add3_u32 v36, v36, v30, v33                               ; d6550024 04863d24
	v_add3_u32 v1, v1, v36, v43                                 ; d6550001 04ae4901
	s_branch BB2                                                ; bfa0fc96
BB6:
	v_mov_b32_e32 v2, 0                                         ; 7e040280
	global_load_b32 v2, v2, s[2:3] offset:32                    ; ee050002 00000002 00002002
	s_wait_loadcnt 0x0                                          ; bfc00000
	v_add_nc_u32_e32 v2, v2, v0                                 ; 4a040102
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_ashrrev_i32_e32 v3, 31, v2                                ; 3406049f
	v_lshlrev_b64_e64 v[2:3], 2, v[2:3]                         ; d51f0002 00020482
	s_delay_alu instid0(VALU_DEP_1)                             ; bf870001
	v_add_co_u32 v4, vcc_lo, v8, v2                             ; d7006a04 00020508
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_add_co_ci_u32_e32 v5, vcc_lo, s6, v3, vcc_lo              ; 400a0606
	global_load_b32 v3, v[4:5], off                             ; ee05007c 00000003 00000004
	s_wait_loadcnt 0x0                                          ; bfc00000
	v_add_nc_u32_e32 v3, v3, v1                                 ; 4a060303
	global_store_b32 v[4:5], v3, off                            ; ee06807c 01800000 00000004
BB11:
	s_nop 0                                                     ; bf800000
	s_sendmsg sendmsg(MSG_DEALLOC_VGPRS)                        ; bfb60003
	s_endpgm                                                    ; bfb00000
 