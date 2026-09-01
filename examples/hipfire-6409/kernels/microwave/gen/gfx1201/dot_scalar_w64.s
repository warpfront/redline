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
	s_cbranch_execz BB11                                        ; bfa5037f
BB1:
	s_mov_b32 s0, 0                                             ; be800080
	v_mov_b32_e32 v1, 0                                         ; 7e020280
BB2:
	v_mov_b32_e32 v2, 0                                         ; 7e040280
	global_load_b32 v2, v2, s[2:3] offset:28                    ; ee050002 00000002 00001c02
	s_wait_loadcnt 0x0                                          ; bfc00000
	v_readfirstlane_b32 s1, v2                                  ; 7e020502
	s_cmp_ge_u32 s0, s1                                         ; bf090100
	s_cbranch_scc1 BB6                                          ; bfa2035e
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
	s_movk_i32 s8, 0xff00                                       ; b008ff00
	s_movk_i32 s9, 0x80                                         ; b0090080
	s_add_co_u32 s0, s0, 1                                      ; 80008100
	s_wait_loadcnt 0x7                                          ; bfc00007
	v_bfe_u32 v11, v28, 16, 4                                   ; d610000b 0211211c
	v_and_b32_e32 v44, 15, v29                                  ; 36583a8f
	v_bfe_u32 v45, v29, 8, 4                                    ; d610002d 0211111d
	v_and_b32_e32 v9, 15, v28                                   ; 3612388f
	v_bfe_u32 v46, v29, 16, 4                                   ; d610002e 0211211d
	v_bfe_u32 v29, v29, 24, 4                                   ; d610001d 0211311d
	v_bfe_u32 v10, v28, 8, 4                                    ; d610000a 0211111c
	v_bfe_u32 v28, v28, 24, 4                                   ; d610001c 0211311c
	v_and_b32_e32 v47, 15, v30                                  ; 365e3c8f
	s_wait_loadcnt 0x6                                          ; bfc00006
	v_bfe_u32 v2, v12, 0, 8                                     ; d6100002 0221010c
	v_add_nc_u32_e32 v11, -8, v11                               ; 4a1616c8
	v_add_nc_u32_e32 v44, -8, v44                               ; 4a5858c8
	v_add_nc_u32_e32 v45, -8, v45                               ; 4a5a5ac8
	v_bfe_u32 v5, v12, 8, 8                                     ; d6100005 0221110c
	v_add_nc_u32_e32 v9, -8, v9                                 ; 4a1212c8
	v_add_nc_u32_e32 v46, -8, v46                               ; 4a5c5cc8
	v_add_nc_u32_e32 v29, -8, v29                               ; 4a3a3ac8
	v_add_nc_u32_e32 v10, -8, v10                               ; 4a1414c8
	v_add_nc_u32_e32 v28, -8, v28                               ; 4a3838c8
	v_add_nc_u32_e32 v47, -8, v47                               ; 4a5e5ec8
	s_wait_alu 0xfffe                                           ; bf88fffe
	v_add_nc_u32_e32 v3, s8, v2                                 ; 4a060408
	v_cmp_le_i32_e32 vcc, s9, v2                                ; 7c860409
	v_add_nc_u32_e32 v7, s8, v5                                 ; 4a0e0a08
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v2, v2, v3, vcc                           ; 02040702
	v_cmp_le_i32_e32 vcc, s9, v5                                ; 7c860a09
	s_delay_alu instid0(VALU_DEP_2)                             ; bf870002
	v_mul_lo_u32 v9, v9, v2                                     ; d72c0009 00020509
	v_bfe_u32 v2, v12, 16, 8                                    ; d6100002 0221210c
	v_lshrrev_b32_e32 v12, 24, v12                              ; 32181898
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v5, v5, v7, vcc                           ; 020a0f05
	v_bfe_u32 v7, v13, 0, 8                                     ; d6100007 0221010d
	s_delay_alu instid0(VALU_DEP_4)                             ; bf870004
	v_add_nc_u32_e32 v3, s8, v2                                 ; 4a060408
	v_cmp_le_i32_e32 vcc, s9, v2                                ; 7c860409
	v_mul_lo_u32 v10, v10, v5                                   ; d72c000a 00020b0a
	v_add_nc_u32_e32 v5, s8, v12                                ; 4a0a1808
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v2, v2, v3, vcc                           ; 02040702
	v_cmp_le_i32_e32 vcc, s9, v12                               ; 7c861809
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_3) | instid1(VALU_DEP_2) ; bf870142
	v_mul_lo_u32 v11, v11, v2                                   ; d72c000b 0002050b
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v12, v12, v5, vcc                         ; 02180b0c
	v_cmp_le_i32_e32 vcc, s9, v7                                ; 7c860e09
	v_mul_lo_u32 v28, v28, v12                                  ; d72c001c 0002191c
	v_bfe_u32 v12, v13, 16, 8                                   ; d610000c 0221210d
	v_add3_u32 v11, v11, v9, v10                                ; d655000b 042a130b
	v_bfe_u32 v10, v13, 8, 8                                    ; d610000a 0221110d
	v_add_nc_u32_e32 v9, s8, v7                                 ; 4a120e08
	v_lshrrev_b32_e32 v13, 24, v13                              ; 321a1a98
	s_delay_alu instid0(VALU_DEP_4)                             ; bf870004
	v_add3_u32 v1, v1, v11, v28                                 ; d6550001 04721701
	v_add_nc_u32_e32 v28, s8, v12                               ; 4a381808
	v_add_nc_u32_e32 v11, s8, v10                               ; 4a161408
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v7, v7, v9, vcc                           ; 020e1307
	v_cmp_le_i32_e32 vcc, s9, v10                               ; 7c861409
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_4) | instid1(VALU_DEP_3) ; bf8701d2
	v_mul_lo_u32 v44, v44, v7                                   ; d72c002c 00020f2c
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v10, v10, v11, vcc                        ; 0214170a
	v_cmp_le_i32_e32 vcc, s9, v12                               ; 7c861809
	v_bfe_u32 v2, v14, 8, 8                                     ; d6100002 0221110e
	v_mul_lo_u32 v45, v45, v10                                  ; d72c002d 0002152d
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v12, v12, v28, vcc                        ; 0218390c
	v_cmp_le_i32_e32 vcc, s9, v13                               ; 7c861a09
	v_add_nc_u32_e32 v3, s8, v2                                 ; 4a060408
	v_bfe_u32 v5, v30, 8, 4                                     ; d6100005 0211111e
	v_bfe_u32 v7, v14, 16, 8                                    ; d6100007 0221210e
	v_mul_lo_u32 v46, v46, v12                                  ; d72c002e 0002192e
	v_add_nc_u32_e32 v5, -8, v5                                 ; 4a0a0ac8
	v_add_nc_u32_e32 v9, s8, v7                                 ; 4a120e08
	v_bfe_u32 v10, v30, 16, 4                                   ; d610000a 0211211e
	s_delay_alu instid0(VALU_DEP_4)                             ; bf870004
	v_add3_u32 v46, v46, v44, v45                               ; d655002e 04b6592e
	v_bfe_u32 v45, v14, 0, 8                                    ; d610002d 0221010e
	v_add_nc_u32_e32 v44, s8, v13                               ; 4a581a08
	v_add_nc_u32_e32 v10, -8, v10                               ; 4a1414c8
	v_lshrrev_b32_e32 v14, 24, v14                              ; 321c1c98
	s_wait_alu 0xfffd                                           ; bf88fffd
	s_delay_alu instid0(VALU_DEP_3) | instskip(SKIP_3) | instid1(VALU_DEP_4) ; bf870243
	v_cndmask_b32_e32 v13, v13, v44, vcc                        ; 021a590d
	v_cmp_le_i32_e32 vcc, s9, v45                               ; 7c865a09
	v_bfe_u32 v30, v30, 24, 4                                   ; d610001e 0211311e
	v_add_nc_u32_e32 v11, s8, v14                               ; 4a161c08
	v_mul_lo_u32 v29, v29, v13                                  ; d72c001d 00021b1d
	v_add_nc_u32_e32 v30, -8, v30                               ; 4a3c3cc8
	v_bfe_u32 v12, v15, 0, 8                                    ; d610000c 0221010f
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_3) | instid1(VALU_DEP_1) ; bf8700c1
	v_add_nc_u32_e32 v13, s8, v12                               ; 4a1a1808
	v_add3_u32 v1, v1, v46, v29                                 ; d6550001 04765d01
	v_add_nc_u32_e32 v46, s8, v45                               ; 4a5c5a08
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v45, v45, v46, vcc                        ; 025a5d2d
	v_cmp_le_i32_e32 vcc, s9, v2                                ; 7c860409
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_4) | instid1(VALU_DEP_3) ; bf8701d2
	v_mul_lo_u32 v47, v47, v45                                  ; d72c002f 00025b2f
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v2, v2, v3, vcc                           ; 02040702
	v_cmp_le_i32_e32 vcc, s9, v7                                ; 7c860e09
	v_bfe_u32 v28, v15, 8, 8                                    ; d610001c 0221110f
	v_mul_lo_u32 v5, v5, v2                                     ; d72c0005 00020505
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v7, v7, v9, vcc                           ; 020e1307
	v_cmp_le_i32_e32 vcc, s9, v14                               ; 7c861c09
	v_add_nc_u32_e32 v29, s8, v28                               ; 4a3a3808
	v_bfe_u32 v44, v15, 16, 8                                   ; d610002c 0221210f
	s_delay_alu instid0(VALU_DEP_4)                             ; bf870004
	v_mul_lo_u32 v10, v10, v7                                   ; d72c000a 00020f0a
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v14, v14, v11, vcc                        ; 021c170e
	v_cmp_le_i32_e32 vcc, s9, v12                               ; 7c861809
	v_add_nc_u32_e32 v45, s8, v44                               ; 4a5a5808
	v_bfe_u32 v46, v31, 16, 4                                   ; d610002e 0211211f
	s_delay_alu instid0(VALU_DEP_4)                             ; bf870004
	v_mul_lo_u32 v30, v30, v14                                  ; d72c001e 00021d1e
	v_and_b32_e32 v14, 15, v31                                  ; 361c3e8f
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v12, v12, v13, vcc                        ; 02181b0c
	v_cmp_le_i32_e32 vcc, s9, v28                               ; 7c863809
	v_add3_u32 v10, v10, v47, v5                                ; d655000a 04165f0a
	v_add_nc_u32_e32 v46, -8, v46                               ; 4a5c5cc8
	v_lshrrev_b32_e32 v15, 24, v15                              ; 321e1e98
	v_add_nc_u32_e32 v14, -8, v14                               ; 4a1c1cc8
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v28, v28, v29, vcc                        ; 02383b1c
	v_cmp_le_i32_e32 vcc, s9, v44                               ; 7c865809
	v_add3_u32 v1, v1, v10, v30                                 ; d6550001 047a1501
	v_bfe_u32 v30, v31, 8, 4                                    ; d610001e 0211111f
	v_bfe_u32 v31, v31, 24, 4                                   ; d610001f 0211311f
	v_add_nc_u32_e32 v47, s8, v15                               ; 4a5e1e08
	v_mul_lo_u32 v14, v14, v12                                  ; d72c000e 0002190e
	s_wait_loadcnt 0x5                                          ; bfc00005
	v_bfe_u32 v2, v32, 8, 4                                     ; d6100002 02111120
	v_bfe_u32 v3, v32, 16, 4                                    ; d6100003 02112120
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v44, v44, v45, vcc                        ; 02585b2c
	v_cmp_le_i32_e32 vcc, s9, v15                               ; 7c861e09
	v_add_nc_u32_e32 v30, -8, v30                               ; 4a3c3cc8
	v_add_nc_u32_e32 v31, -8, v31                               ; 4a3e3ec8
	v_and_b32_e32 v5, 15, v33                                   ; 360a428f
	v_bfe_u32 v7, v33, 8, 4                                     ; d6100007 02111121
	v_add_nc_u32_e32 v2, -8, v2                                 ; 4a0404c8
	v_add_nc_u32_e32 v3, -8, v3                                 ; 4a0606c8
	v_bfe_u32 v9, v33, 16, 4                                    ; d6100009 02112121
	v_mul_lo_u32 v46, v46, v44                                  ; d72c002e 0002592e
	v_bfe_u32 v33, v33, 24, 4                                   ; d6100021 02113121
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v15, v15, v47, vcc                        ; 021e5f0f
	v_and_b32_e32 v47, 15, v32                                  ; 365e408f
	v_bfe_u32 v32, v32, 24, 4                                   ; d6100020 02113120
	v_mul_lo_u32 v30, v30, v28                                  ; d72c001e 0002391e
	v_and_b32_e32 v10, 15, v34                                  ; 3614448f
	v_add_nc_u32_e32 v5, -8, v5                                 ; 4a0a0ac8
	v_add_nc_u32_e32 v7, -8, v7                                 ; 4a0e0ec8
	v_bfe_u32 v11, v34, 8, 4                                    ; d610000b 02111122
	v_add_nc_u32_e32 v9, -8, v9                                 ; 4a1212c8
	v_bfe_u32 v12, v34, 16, 4                                   ; d610000c 02112122
	v_add_nc_u32_e32 v33, -8, v33                               ; 4a4242c8
	v_mul_lo_u32 v31, v31, v15                                  ; d72c001f 00021f1f
	v_add_nc_u32_e32 v47, -8, v47                               ; 4a5e5ec8
	s_wait_loadcnt 0x4                                          ; bfc00004
	v_bfe_u32 v13, v16, 0, 8                                    ; d610000d 02210110
	v_add_nc_u32_e32 v32, -8, v32                               ; 4a4040c8
	v_bfe_u32 v15, v16, 8, 8                                    ; d610000f 02211110
	v_add_nc_u32_e32 v10, -8, v10                               ; 4a1414c8
	v_add3_u32 v46, v46, v14, v30                               ; d655002e 047a1d2e
	v_add_nc_u32_e32 v11, -8, v11                               ; 4a1616c8
	v_add_nc_u32_e32 v12, -8, v12                               ; 4a1818c8
	v_bfe_u32 v29, v16, 16, 8                                   ; d610001d 02212110
	v_cmp_le_i32_e32 vcc, s9, v13                               ; 7c861a09
	v_add_nc_u32_e32 v14, s8, v13                               ; 4a1c1a08
	v_lshrrev_b32_e32 v16, 24, v16                              ; 32202098
	v_add_nc_u32_e32 v28, s8, v15                               ; 4a381e08
	v_add3_u32 v1, v1, v46, v31                                 ; d6550001 047e5d01
	v_add_nc_u32_e32 v30, s8, v29                               ; 4a3c3a08
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v13, v13, v14, vcc                        ; 021a1d0d
	v_cmp_le_i32_e32 vcc, s9, v15                               ; 7c861e09
	v_add_nc_u32_e32 v31, s8, v16                               ; 4a3e2008
	s_delay_alu instid0(VALU_DEP_3) | instskip(SKIP_4) | instid1(VALU_DEP_3) ; bf8701d3
	v_mul_lo_u32 v47, v47, v13                                  ; d72c002f 00021b2f
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v15, v15, v28, vcc                        ; 021e390f
	v_cmp_le_i32_e32 vcc, s9, v29                               ; 7c863a09
	v_bfe_u32 v45, v17, 8, 8                                    ; d610002d 02211111
	v_mul_lo_u32 v2, v2, v15                                    ; d72c0002 00021f02
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v29, v29, v30, vcc                        ; 023a3d1d
	v_cmp_le_i32_e32 vcc, s9, v16                               ; 7c862009
	v_add_nc_u32_e32 v46, s8, v45                               ; 4a5c5a08
	s_delay_alu instid0(VALU_DEP_3) | instskip(SKIP_2) | instid1(VALU_DEP_1) ; bf8700b3
	v_mul_lo_u32 v3, v3, v29                                    ; d72c0003 00023b03
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v16, v16, v31, vcc                        ; 02203f10
	v_mul_lo_u32 v32, v32, v16                                  ; d72c0020 00022120
	v_add3_u32 v3, v3, v47, v2                                  ; d6550003 040a5f03
	v_bfe_u32 v47, v17, 16, 8                                   ; d610002f 02212111
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_3) | instid1(VALU_DEP_2) ; bf870141
	v_add_nc_u32_e32 v2, s8, v47                                ; 4a045e08
	v_add3_u32 v1, v1, v3, v32                                  ; d6550001 04820701
	v_bfe_u32 v32, v17, 0, 8                                    ; d6100020 02210111
	v_lshrrev_b32_e32 v17, 24, v17                              ; 32222298
	v_add_nc_u32_e32 v44, s8, v32                               ; 4a584008
	v_cmp_le_i32_e32 vcc, s9, v32                               ; 7c864009
	v_add_nc_u32_e32 v3, s8, v17                                ; 4a062208
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v32, v32, v44, vcc                        ; 02405920
	v_cmp_le_i32_e32 vcc, s9, v45                               ; 7c865a09
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_3) | instid1(VALU_DEP_2) ; bf870142
	v_mul_lo_u32 v5, v5, v32                                    ; d72c0005 00024105
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v45, v45, v46, vcc                        ; 025a5d2d
	v_cmp_le_i32_e32 vcc, s9, v47                               ; 7c865e09
	v_mul_lo_u32 v7, v7, v45                                    ; d72c0007 00025b07
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v47, v47, v2, vcc                         ; 025e052f
	v_cmp_le_i32_e32 vcc, s9, v17                               ; 7c862209
	v_bfe_u32 v14, v18, 16, 8                                   ; d610000e 02212112
	s_delay_alu instid0(VALU_DEP_3) | instskip(SKIP_3) | instid1(VALU_DEP_2) ; bf870143
	v_mul_lo_u32 v9, v9, v47                                    ; d72c0009 00025f09
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v17, v17, v3, vcc                         ; 02220711
	v_add_nc_u32_e32 v15, s8, v14                               ; 4a1e1c08
	v_mul_lo_u32 v33, v33, v17                                  ; d72c0021 00022321
	v_add3_u32 v9, v9, v5, v7                                   ; d6550009 041e0b09
	v_bfe_u32 v5, v18, 0, 8                                     ; d6100005 02210112
	s_delay_alu instid0(VALU_DEP_1)                             ; bf870001
	v_cmp_le_i32_e32 vcc, s9, v5                                ; 7c860a09
	v_add_nc_u32_e32 v7, s8, v5                                 ; 4a0e0a08
	v_add3_u32 v1, v1, v9, v33                                  ; d6550001 04861301
	v_bfe_u32 v9, v18, 8, 8                                     ; d6100009 02211112
	v_lshrrev_b32_e32 v18, 24, v18                              ; 32242498
	v_bfe_u32 v34, v34, 24, 4                                   ; d6100022 02113122
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v5, v5, v7, vcc                           ; 020a0f05
	v_bfe_u32 v17, v19, 0, 8                                    ; d6100011 02210113
	v_add_nc_u32_e32 v13, s8, v9                                ; 4a1a1208
	v_cmp_le_i32_e32 vcc, s9, v9                                ; 7c861209
	v_add_nc_u32_e32 v16, s8, v18                               ; 4a202408
	v_add_nc_u32_e32 v34, -8, v34                               ; 4a4444c8
	v_and_b32_e32 v28, 15, v35                                  ; 3638468f
	v_mul_lo_u32 v10, v10, v5                                   ; d72c000a 00020b0a
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v9, v9, v13, vcc                          ; 02121b09
	v_cmp_le_i32_e32 vcc, s9, v14                               ; 7c861c09
	v_bfe_u32 v29, v19, 8, 8                                    ; d610001d 02211113
	v_add_nc_u32_e32 v28, -8, v28                               ; 4a3838c8
	v_bfe_u32 v31, v35, 8, 4                                    ; d610001f 02111123
	v_mul_lo_u32 v11, v11, v9                                   ; d72c000b 0002130b
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v14, v14, v15, vcc                        ; 021c1f0e
	v_cmp_le_i32_e32 vcc, s9, v18                               ; 7c862409
	v_add_nc_u32_e32 v30, s8, v29                               ; 4a3c3a08
	v_add_nc_u32_e32 v31, -8, v31                               ; 4a3e3ec8
	v_bfe_u32 v32, v19, 16, 8                                   ; d6100020 02212113
	v_mul_lo_u32 v12, v12, v14                                  ; d72c000c 00021d0c
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v18, v18, v16, vcc                        ; 02242112
	v_cmp_le_i32_e32 vcc, s9, v17                               ; 7c862209
	v_add_nc_u32_e32 v33, s8, v32                               ; 4a424008
	v_lshrrev_b32_e32 v19, 24, v19                              ; 32262698
	s_delay_alu instid0(VALU_DEP_4) | instskip(SKIP_4) | instid1(VALU_DEP_3) ; bf8701d4
	v_mul_lo_u32 v34, v34, v18                                  ; d72c0022 00022522
	v_add_nc_u32_e32 v18, s8, v17                               ; 4a242208
	v_add3_u32 v12, v12, v10, v11                               ; d655000c 042e150c
	v_add_nc_u32_e32 v44, s8, v19                               ; 4a582608
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v17, v17, v18, vcc                        ; 02222511
	v_cmp_le_i32_e32 vcc, s9, v29                               ; 7c863a09
	v_add3_u32 v1, v1, v12, v34                                 ; d6550001 048a1901
	v_bfe_u32 v34, v35, 16, 4                                   ; d6100022 02112123
	v_bfe_u32 v35, v35, 24, 4                                   ; d6100023 02113123
	s_wait_loadcnt 0x2                                          ; bfc00002
	v_bfe_u32 v45, v20, 0, 8                                    ; d610002d 02210114
	v_mul_lo_u32 v28, v28, v17                                  ; d72c001c 0002231c
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v29, v29, v30, vcc                        ; 023a3d1d
	v_cmp_le_i32_e32 vcc, s9, v32                               ; 7c864009
	v_add_nc_u32_e32 v34, -8, v34                               ; 4a4444c8
	v_add_nc_u32_e32 v35, -8, v35                               ; 4a4646c8
	v_and_b32_e32 v47, 15, v36                                  ; 365e488f
	v_add_nc_u32_e32 v46, s8, v45                               ; 4a5c5a08
	v_bfe_u32 v2, v20, 8, 8                                     ; d6100002 02211114
	v_mul_lo_u32 v31, v31, v29                                  ; d72c001f 00023b1f
	v_bfe_u32 v5, v36, 8, 4                                     ; d6100005 02111124
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v32, v32, v33, vcc                        ; 02404320
	v_cmp_le_i32_e32 vcc, s9, v19                               ; 7c862609
	v_add_nc_u32_e32 v47, -8, v47                               ; 4a5e5ec8
	v_add_nc_u32_e32 v3, s8, v2                                 ; 4a060408
	v_bfe_u32 v7, v20, 16, 8                                    ; d6100007 02212114
	v_add_nc_u32_e32 v5, -8, v5                                 ; 4a0a0ac8
	v_bfe_u32 v10, v36, 16, 4                                   ; d610000a 02112124
	v_mul_lo_u32 v34, v34, v32                                  ; d72c0022 00024122
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v19, v19, v44, vcc                        ; 02265913
	v_cmp_le_i32_e32 vcc, s9, v45                               ; 7c865a09
	v_add_nc_u32_e32 v9, s8, v7                                 ; 4a120e08
	v_lshrrev_b32_e32 v20, 24, v20                              ; 32282898
	v_add_nc_u32_e32 v10, -8, v10                               ; 4a1414c8
	v_bfe_u32 v36, v36, 24, 4                                   ; d6100024 02113124
	v_mul_lo_u32 v35, v35, v19                                  ; d72c0023 00022723
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v45, v45, v46, vcc                        ; 025a5d2d
	v_cmp_le_i32_e32 vcc, s9, v2                                ; 7c860409
	v_add3_u32 v34, v34, v28, v31                               ; d6550022 047e3922
	v_add_nc_u32_e32 v11, s8, v20                               ; 4a162808
	v_add_nc_u32_e32 v36, -8, v36                               ; 4a4848c8
	v_bfe_u32 v12, v21, 0, 8                                    ; d610000c 02210115
	v_mul_lo_u32 v47, v47, v45                                  ; d72c002f 00025b2f
	v_and_b32_e32 v14, 15, v37                                  ; 361c4a8f
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v2, v2, v3, vcc                           ; 02040702
	v_cmp_le_i32_e32 vcc, s9, v7                                ; 7c860e09
	v_add3_u32 v1, v1, v34, v35                                 ; d6550001 048e4501
	v_add_nc_u32_e32 v13, s8, v12                               ; 4a1a1808
	v_bfe_u32 v15, v21, 8, 8                                    ; d610000f 02211115
	v_bfe_u32 v17, v37, 8, 4                                    ; d6100011 02111125
	v_add_nc_u32_e32 v14, -8, v14                               ; 4a1c1cc8
	v_mul_lo_u32 v5, v5, v2                                     ; d72c0005 00020505
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v7, v7, v9, vcc                           ; 020e1307
	v_cmp_le_i32_e32 vcc, s9, v20                               ; 7c862809
	v_add_nc_u32_e32 v16, s8, v15                               ; 4a201e08
	v_add_nc_u32_e32 v17, -8, v17                               ; 4a2222c8
	v_bfe_u32 v18, v21, 16, 8                                   ; d6100012 02212115
	v_mul_lo_u32 v10, v10, v7                                   ; d72c000a 00020f0a
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v20, v20, v11, vcc                        ; 02281714
	v_cmp_le_i32_e32 vcc, s9, v12                               ; 7c861809
	v_add_nc_u32_e32 v19, s8, v18                               ; 4a262408
	v_lshrrev_b32_e32 v21, 24, v21                              ; 322a2a98
	s_delay_alu instid0(VALU_DEP_4)                             ; bf870004
	v_mul_lo_u32 v36, v36, v20                                  ; d72c0024 00022924
	v_bfe_u32 v20, v37, 16, 4                                   ; d6100014 02112125
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v12, v12, v13, vcc                        ; 02181b0c
	v_cmp_le_i32_e32 vcc, s9, v15                               ; 7c861e09
	v_add3_u32 v10, v10, v47, v5                                ; d655000a 04165f0a
	v_add_nc_u32_e32 v28, s8, v21                               ; 4a382a08
	v_bfe_u32 v37, v37, 24, 4                                   ; d6100025 02113125
	v_bfe_u32 v29, v22, 0, 8                                    ; d610001d 02210116
	v_add_nc_u32_e32 v20, -8, v20                               ; 4a2828c8
	v_mul_lo_u32 v14, v14, v12                                  ; d72c000e 0002190e
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v15, v15, v16, vcc                        ; 021e210f
	v_cmp_le_i32_e32 vcc, s9, v18                               ; 7c862409
	v_add3_u32 v1, v1, v10, v36                                 ; d6550001 04921501
	v_add_nc_u32_e32 v37, -8, v37                               ; 4a4a4ac8
	v_add_nc_u32_e32 v30, s8, v29                               ; 4a3c3a08
	v_and_b32_e32 v31, 15, v38                                  ; 363e4c8f
	v_bfe_u32 v32, v22, 8, 8                                    ; d6100020 02211116
	v_mul_lo_u32 v17, v17, v15                                  ; d72c0011 00021f11
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v18, v18, v19, vcc                        ; 02242712
	v_cmp_le_i32_e32 vcc, s9, v21                               ; 7c862a09
	v_bfe_u32 v34, v38, 8, 4                                    ; d6100022 02111126
	v_add_nc_u32_e32 v31, -8, v31                               ; 4a3e3ec8
	v_add_nc_u32_e32 v33, s8, v32                               ; 4a424008
	v_bfe_u32 v35, v22, 16, 8                                   ; d6100023 02212116
	v_mul_lo_u32 v20, v20, v18                                  ; d72c0014 00022514
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v21, v21, v28, vcc                        ; 022a3915
	v_cmp_le_i32_e32 vcc, s9, v29                               ; 7c863a09
	v_add_nc_u32_e32 v34, -8, v34                               ; 4a4444c8
	v_add_nc_u32_e32 v36, s8, v35                               ; 4a484608
	v_lshrrev_b32_e32 v22, 24, v22                              ; 322c2c98
	v_mul_lo_u32 v37, v37, v21                                  ; d72c0025 00022b25
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v29, v29, v30, vcc                        ; 023a3d1d
	v_cmp_le_i32_e32 vcc, s9, v32                               ; 7c864009
	v_add3_u32 v20, v20, v14, v17                               ; d6550014 04461d14
	v_add_nc_u32_e32 v44, s8, v22                               ; 4a582c08
	s_delay_alu instid0(VALU_DEP_4)                             ; bf870004
	v_mul_lo_u32 v31, v31, v29                                  ; d72c001f 00023b1f
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v32, v32, v33, vcc                        ; 02404320
	v_cmp_le_i32_e32 vcc, s9, v35                               ; 7c864609
	v_add3_u32 v1, v1, v20, v37                                 ; d6550001 04962901
	v_bfe_u32 v37, v38, 16, 4                                   ; d6100025 02112126
	v_bfe_u32 v38, v38, 24, 4                                   ; d6100026 02113126
	v_bfe_u32 v45, v23, 0, 8                                    ; d610002d 02210117
	v_and_b32_e32 v47, 15, v39                                  ; 365e4e8f
	v_mul_lo_u32 v34, v34, v32                                  ; d72c0022 00024122
	v_bfe_u32 v2, v23, 8, 8                                     ; d6100002 02211117
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v35, v35, v36, vcc                        ; 02464923
	v_cmp_le_i32_e32 vcc, s9, v22                               ; 7c862c09
	v_add_nc_u32_e32 v37, -8, v37                               ; 4a4a4ac8
	v_add_nc_u32_e32 v38, -8, v38                               ; 4a4c4cc8
	v_add_nc_u32_e32 v46, s8, v45                               ; 4a5c5a08
	v_add_nc_u32_e32 v47, -8, v47                               ; 4a5e5ec8
	v_bfe_u32 v5, v39, 8, 4                                     ; d6100005 02111127
	v_add_nc_u32_e32 v3, s8, v2                                 ; 4a060408
	v_bfe_u32 v7, v23, 16, 8                                    ; d6100007 02212117
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v22, v22, v44, vcc                        ; 022c5916
	v_cmp_le_i32_e32 vcc, s9, v45                               ; 7c865a09
	v_mul_lo_u32 v37, v37, v35                                  ; d72c0025 00024725
	v_bfe_u32 v10, v39, 16, 4                                   ; d610000a 02112127
	v_add_nc_u32_e32 v5, -8, v5                                 ; 4a0a0ac8
	v_add_nc_u32_e32 v9, s8, v7                                 ; 4a120e08
	v_mul_lo_u32 v38, v38, v22                                  ; d72c0026 00022d26
	v_lshrrev_b32_e32 v23, 24, v23                              ; 322e2e98
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v45, v45, v46, vcc                        ; 025a5d2d
	v_cmp_le_i32_e32 vcc, s9, v2                                ; 7c860409
	v_add_nc_u32_e32 v10, -8, v10                               ; 4a1414c8
	v_add3_u32 v37, v37, v31, v34                               ; d6550025 048a3f25
	v_bfe_u32 v39, v39, 24, 4                                   ; d6100027 02113127
	s_wait_loadcnt 0x0                                          ; bfc00000
	v_bfe_u32 v12, v24, 0, 8                                    ; d610000c 02210118
	v_add_nc_u32_e32 v11, s8, v23                               ; 4a162e08
	v_mul_lo_u32 v47, v47, v45                                  ; d72c002f 00025b2f
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v2, v2, v3, vcc                           ; 02040702
	v_cmp_le_i32_e32 vcc, s9, v7                                ; 7c860e09
	v_add3_u32 v1, v1, v37, v38                                 ; d6550001 049a4b01
	v_add_nc_u32_e32 v39, -8, v39                               ; 4a4e4ec8
	v_add_nc_u32_e32 v13, s8, v12                               ; 4a1a1808
	v_and_b32_e32 v14, 15, v40                                  ; 361c508f
	v_bfe_u32 v15, v24, 8, 8                                    ; d610000f 02211118
	v_mul_lo_u32 v5, v5, v2                                     ; d72c0005 00020505
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v7, v7, v9, vcc                           ; 020e1307
	v_cmp_le_i32_e32 vcc, s9, v23                               ; 7c862e09
	v_bfe_u32 v17, v40, 8, 4                                    ; d6100011 02111128
	v_add_nc_u32_e32 v14, -8, v14                               ; 4a1c1cc8
	v_add_nc_u32_e32 v16, s8, v15                               ; 4a201e08
	v_bfe_u32 v18, v24, 16, 8                                   ; d6100012 02212118
	v_mul_lo_u32 v10, v10, v7                                   ; d72c000a 00020f0a
	v_bfe_u32 v20, v40, 16, 4                                   ; d6100014 02112128
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v23, v23, v11, vcc                        ; 022e1717
	v_cmp_le_i32_e32 vcc, s9, v12                               ; 7c861809
	v_add_nc_u32_e32 v17, -8, v17                               ; 4a2222c8
	v_add_nc_u32_e32 v19, s8, v18                               ; 4a262408
	v_lshrrev_b32_e32 v24, 24, v24                              ; 32303098
	v_add_nc_u32_e32 v20, -8, v20                               ; 4a2828c8
	v_bfe_u32 v40, v40, 24, 4                                   ; d6100028 02113128
	v_mul_lo_u32 v39, v39, v23                                  ; d72c0027 00022f27
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v12, v12, v13, vcc                        ; 02181b0c
	v_cmp_le_i32_e32 vcc, s9, v15                               ; 7c861e09
	v_add3_u32 v10, v10, v47, v5                                ; d655000a 04165f0a
	v_add_nc_u32_e32 v21, s8, v24                               ; 4a2a3008
	v_add_nc_u32_e32 v40, -8, v40                               ; 4a5050c8
	v_bfe_u32 v22, v25, 0, 8                                    ; d6100016 02210119
	v_mul_lo_u32 v14, v14, v12                                  ; d72c000e 0002190e
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v15, v15, v16, vcc                        ; 021e210f
	v_cmp_le_i32_e32 vcc, s9, v18                               ; 7c862409
	v_add3_u32 v1, v1, v10, v39                                 ; d6550001 049e1501
	v_add_nc_u32_e32 v23, s8, v22                               ; 4a2e2c08
	v_bfe_u32 v28, v25, 8, 8                                    ; d610001c 02211119
	v_mul_lo_u32 v17, v17, v15                                  ; d72c0011 00021f11
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v18, v18, v19, vcc                        ; 02242712
	v_cmp_le_i32_e32 vcc, s9, v24                               ; 7c863009
	v_bfe_u32 v30, v41, 8, 4                                    ; d610001e 02111129
	v_add_nc_u32_e32 v29, s8, v28                               ; 4a3a3808
	v_bfe_u32 v31, v25, 16, 8                                   ; d610001f 02212119
	v_mul_lo_u32 v20, v20, v18                                  ; d72c0014 00022514
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v24, v24, v21, vcc                        ; 02302b18
	v_cmp_le_i32_e32 vcc, s9, v22                               ; 7c862c09
	v_add_nc_u32_e32 v30, -8, v30                               ; 4a3c3cc8
	v_add_nc_u32_e32 v32, s8, v31                               ; 4a403e08
	v_bfe_u32 v33, v41, 16, 4                                   ; d6100021 02112129
	v_lshrrev_b32_e32 v25, 24, v25                              ; 32323298
	v_mul_lo_u32 v40, v40, v24                                  ; d72c0028 00023128
	v_and_b32_e32 v24, 15, v41                                  ; 3630528f
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v22, v22, v23, vcc                        ; 022c2f16
	v_cmp_le_i32_e32 vcc, s9, v28                               ; 7c863809
	v_add3_u32 v20, v20, v14, v17                               ; d6550014 04461d14
	v_add_nc_u32_e32 v33, -8, v33                               ; 4a4242c8
	v_add_nc_u32_e32 v34, s8, v25                               ; 4a443208
	v_bfe_u32 v41, v41, 24, 4                                   ; d6100029 02113129
	v_bfe_u32 v35, v26, 0, 8                                    ; d6100023 0221011a
	v_add_nc_u32_e32 v24, -8, v24                               ; 4a3030c8
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v28, v28, v29, vcc                        ; 02383b1c
	v_cmp_le_i32_e32 vcc, s9, v31                               ; 7c863e09
	v_add3_u32 v1, v1, v20, v40                                 ; d6550001 04a22901
	v_and_b32_e32 v37, 15, v42                                  ; 364a548f
	v_add_nc_u32_e32 v41, -8, v41                               ; 4a5252c8
	v_add_nc_u32_e32 v36, s8, v35                               ; 4a484608
	v_bfe_u32 v38, v26, 8, 8                                    ; d6100026 0221111a
	v_mul_lo_u32 v24, v24, v22                                  ; d72c0018 00022d18
	v_mul_lo_u32 v30, v30, v28                                  ; d72c001e 0002391e
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v31, v31, v32, vcc                        ; 023e411f
	v_cmp_le_i32_e32 vcc, s9, v25                               ; 7c863209
	v_add_nc_u32_e32 v37, -8, v37                               ; 4a4a4ac8
	v_bfe_u32 v40, v42, 8, 4                                    ; d6100028 0211112a
	v_add_nc_u32_e32 v39, s8, v38                               ; 4a4e4c08
	v_mul_lo_u32 v33, v33, v31                                  ; d72c0021 00023f21
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v25, v25, v34, vcc                        ; 02324519
	v_cmp_le_i32_e32 vcc, s9, v35                               ; 7c864609
	v_add_nc_u32_e32 v40, -8, v40                               ; 4a5050c8
	v_bfe_u32 v45, v42, 16, 4                                   ; d610002d 0211212a
	s_delay_alu instid0(VALU_DEP_4)                             ; bf870004
	v_mul_lo_u32 v41, v41, v25                                  ; d72c0029 00023329
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v35, v35, v36, vcc                        ; 02464923
	v_cmp_le_i32_e32 vcc, s9, v38                               ; 7c864c09
	v_add3_u32 v33, v33, v24, v30                               ; d6550021 047a3121
	v_add_nc_u32_e32 v45, -8, v45                               ; 4a5a5ac8
	v_bfe_u32 v42, v42, 24, 4                                   ; d610002a 0211312a
	v_mul_lo_u32 v37, v37, v35                                  ; d72c0025 00024725
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v38, v38, v39, vcc                        ; 024c4f26
	v_add3_u32 v1, v1, v33, v41                                 ; d6550001 04a64301
	v_bfe_u32 v41, v26, 16, 8                                   ; d6100029 0221211a
	v_lshrrev_b32_e32 v26, 24, v26                              ; 32343498
	v_add_nc_u32_e32 v42, -8, v42                               ; 4a5454c8
	v_bfe_u32 v47, v27, 0, 8                                    ; d610002f 0221011b
	v_and_b32_e32 v3, 15, v43                                   ; 3606568f
	v_mul_lo_u32 v40, v40, v38                                  ; d72c0028 00024d28
	v_add_nc_u32_e32 v44, s8, v41                               ; 4a585208
	v_cmp_le_i32_e32 vcc, s9, v41                               ; 7c865209
	v_add_nc_u32_e32 v46, s8, v26                               ; 4a5c3408
	v_add_nc_u32_e32 v2, s8, v47                                ; 4a045e08
	v_bfe_u32 v5, v27, 8, 8                                     ; d6100005 0221111b
	v_add_nc_u32_e32 v3, -8, v3                                 ; 4a0606c8
	v_bfe_u32 v9, v43, 8, 4                                     ; d6100009 0211112b
	v_bfe_u32 v10, v27, 16, 8                                   ; d610000a 0221211b
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v41, v41, v44, vcc                        ; 02525929
	v_cmp_le_i32_e32 vcc, s9, v26                               ; 7c863409
	v_add_nc_u32_e32 v7, s8, v5                                 ; 4a0e0a08
	v_bfe_u32 v12, v43, 16, 4                                   ; d610000c 0211212b
	v_add_nc_u32_e32 v9, -8, v9                                 ; 4a1212c8
	v_add_nc_u32_e32 v11, s8, v10                               ; 4a161408
	v_mul_lo_u32 v45, v45, v41                                  ; d72c002d 0002532d
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v26, v26, v46, vcc                        ; 02345d1a
	v_cmp_le_i32_e32 vcc, s9, v47                               ; 7c865e09
	v_add_nc_u32_e32 v12, -8, v12                               ; 4a1818c8
	v_lshrrev_b32_e32 v27, 24, v27                              ; 32363698
	v_bfe_u32 v43, v43, 24, 4                                   ; d610002b 0211312b
	v_mul_lo_u32 v42, v42, v26                                  ; d72c002a 0002352a
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v47, v47, v2, vcc                         ; 025e052f
	v_cmp_le_i32_e32 vcc, s9, v5                                ; 7c860a09
	v_add3_u32 v45, v45, v37, v40                               ; d655002d 04a24b2d
	v_add_nc_u32_e32 v13, s8, v27                               ; 4a1a3608
	v_add_nc_u32_e32 v43, -8, v43                               ; 4a5656c8
	v_mul_lo_u32 v3, v3, v47                                    ; d72c0003 00025f03
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v5, v5, v7, vcc                           ; 020a0f05
	v_cmp_le_i32_e32 vcc, s9, v10                               ; 7c861409
	v_add3_u32 v1, v1, v45, v42                                 ; d6550001 04aa5b01
	s_delay_alu instid0(VALU_DEP_3) | instskip(SKIP_3) | instid1(VALU_DEP_2) ; bf870143
	v_mul_lo_u32 v9, v9, v5                                     ; d72c0009 00020b09
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v10, v10, v11, vcc                        ; 0214170a
	v_cmp_le_i32_e32 vcc, s9, v27                               ; 7c863609
	v_mul_lo_u32 v12, v12, v10                                  ; d72c000c 0002150c
	s_wait_alu 0xfffd                                           ; bf88fffd
	v_cndmask_b32_e32 v27, v27, v13, vcc                        ; 02361b1b
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_mul_lo_u32 v43, v43, v27                                  ; d72c002b 0002372b
	v_add3_u32 v12, v12, v3, v9                                 ; d655000c 0426070c
	v_add3_u32 v1, v1, v12, v43                                 ; d6550001 04ae1901
	s_branch BB2                                                ; bfa0fc9a
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