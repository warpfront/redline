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
	s_cbranch_execz BB11                                        ; bfa50106
BB1:
	s_mov_b32 s0, 0                                             ; be800080
	v_mov_b32_e32 v1, 0                                         ; 7e020280
BB2:
	v_mov_b32_e32 v2, 0                                         ; 7e040280
	global_load_b32 v2, v2, s[2:3] offset:28                    ; ee050002 00000002 00001c02
	s_wait_loadcnt 0x0                                          ; bfc00000
	v_readfirstlane_b32 s1, v2                                  ; 7e020502
	s_cmp_ge_u32 s0, s1                                         ; bf090100
	s_cbranch_scc1 BB6                                          ; bfa200e5
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
	s_mov_b32 s8, 0x1010101                                     ; be8800ff 01010101
	s_add_co_u32 s0, s0, 1                                      ; 80008100
	s_wait_loadcnt 0x6                                          ; bfc00006
	s_wait_alu 0xfffe                                           ; bf88fffe
	v_dot4_i32_iu8 v11, v14, s8, 0 neg_lo:[1,0,0]               ; cc16400b 3a00110e
	v_dot4_i32_iu8 v9, v12, s8, 0 neg_lo:[1,0,0]                ; cc164009 3a00110c
	v_dot4_i32_iu8 v10, v13, s8, 0 neg_lo:[1,0,0]               ; cc16400a 3a00110d
	s_delay_alu instid0(VALU_DEP_3) | instskip(SKIP_2) | instid1(VALU_DEP_3) ; bf8701b3
	v_lshlrev_b32_e32 v11, 5, v11                               ; 30161685
	v_lshlrev_b32_e32 v9, 5, v9                                 ; 30121285
	v_lshlrev_b32_e32 v10, 5, v10                               ; 30141485
	v_sub_nc_u32_e32 v11, 0, v11                                ; 4c161680
	v_sub_nc_u32_e32 v9, 0, v9                                  ; 4c121280
	v_sub_nc_u32_e32 v10, 0, v10                                ; 4c141480
	s_delay_alu instid0(VALU_DEP_3) | instskip(SKIP_2) | instid1(VALU_DEP_1) ; bf8700b3
	v_dot4_i32_iu8 v14, v14, v30, v11 neg_lo:[1,0,0]            ; cc16400e 3c2e3d0e
	v_dot4_i32_iu8 v12, v12, v28, v9 neg_lo:[1,0,0]             ; cc16400c 3c26390c
	v_dot4_i32_iu8 v13, v13, v29, v10 neg_lo:[1,0,0]            ; cc16400d 3c2a3b0d
	v_add3_u32 v13, v13, v1, v12                                ; d655000d 0432030d
	v_dot4_i32_iu8 v12, v15, s8, 0 neg_lo:[1,0,0]               ; cc16400c 3a00110f
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_lshlrev_b32_e32 v12, 5, v12                               ; 30181885
	v_sub_nc_u32_e32 v12, 0, v12                                ; 4c181880
	s_delay_alu instid0(VALU_DEP_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870091
	v_dot4_i32_iu8 v15, v15, v31, v12 neg_lo:[1,0,0]            ; cc16400f 3c323f0f
	v_add3_u32 v15, v15, v13, v14                               ; d655000f 043a1b0f
	s_wait_loadcnt 0x4                                          ; bfc00004
	v_dot4_i32_iu8 v13, v16, s8, 0 neg_lo:[1,0,0]               ; cc16400d 3a001110
	v_dot4_i32_iu8 v14, v17, s8, 0 neg_lo:[1,0,0]               ; cc16400e 3a001111
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_1) | instid1(VALU_DEP_2) ; bf870122
	v_lshlrev_b32_e32 v13, 5, v13                               ; 301a1a85
	v_lshlrev_b32_e32 v14, 5, v14                               ; 301c1c85
	v_sub_nc_u32_e32 v13, 0, v13                                ; 4c1a1a80
	v_sub_nc_u32_e32 v14, 0, v14                                ; 4c1c1c80
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a2
	v_dot4_i32_iu8 v16, v16, v32, v13 neg_lo:[1,0,0]            ; cc164010 3c364110
	v_dot4_i32_iu8 v17, v17, v33, v14 neg_lo:[1,0,0]            ; cc164011 3c3a4311
	v_add3_u32 v17, v17, v15, v16                               ; d6550011 04421f11
	v_dot4_i32_iu8 v15, v18, s8, 0 neg_lo:[1,0,0]               ; cc16400f 3a001112
	v_dot4_i32_iu8 v16, v19, s8, 0 neg_lo:[1,0,0]               ; cc164010 3a001113
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_1) | instid1(VALU_DEP_2) ; bf870122
	v_lshlrev_b32_e32 v15, 5, v15                               ; 301e1e85
	v_lshlrev_b32_e32 v16, 5, v16                               ; 30202085
	v_sub_nc_u32_e32 v15, 0, v15                                ; 4c1e1e80
	v_sub_nc_u32_e32 v16, 0, v16                                ; 4c202080
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a2
	v_dot4_i32_iu8 v18, v18, v34, v15 neg_lo:[1,0,0]            ; cc164012 3c3e4512
	v_dot4_i32_iu8 v19, v19, v35, v16 neg_lo:[1,0,0]            ; cc164013 3c424713
	v_add3_u32 v19, v19, v17, v18                               ; d6550013 044a2313
	s_wait_loadcnt 0x2                                          ; bfc00002
	v_dot4_i32_iu8 v17, v20, s8, 0 neg_lo:[1,0,0]               ; cc164011 3a001114
	v_dot4_i32_iu8 v18, v21, s8, 0 neg_lo:[1,0,0]               ; cc164012 3a001115
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_1) | instid1(VALU_DEP_2) ; bf870122
	v_lshlrev_b32_e32 v17, 5, v17                               ; 30222285
	v_lshlrev_b32_e32 v18, 5, v18                               ; 30242485
	v_sub_nc_u32_e32 v17, 0, v17                                ; 4c222280
	v_sub_nc_u32_e32 v18, 0, v18                                ; 4c242480
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a2
	v_dot4_i32_iu8 v20, v20, v36, v17 neg_lo:[1,0,0]            ; cc164014 3c464914
	v_dot4_i32_iu8 v21, v21, v37, v18 neg_lo:[1,0,0]            ; cc164015 3c4a4b15
	v_add3_u32 v21, v21, v19, v20                               ; d6550015 04522715
	v_dot4_i32_iu8 v19, v22, s8, 0 neg_lo:[1,0,0]               ; cc164013 3a001116
	v_dot4_i32_iu8 v20, v23, s8, 0 neg_lo:[1,0,0]               ; cc164014 3a001117
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_1) | instid1(VALU_DEP_2) ; bf870122
	v_lshlrev_b32_e32 v19, 5, v19                               ; 30262685
	v_lshlrev_b32_e32 v20, 5, v20                               ; 30282885
	v_sub_nc_u32_e32 v19, 0, v19                                ; 4c262680
	v_sub_nc_u32_e32 v20, 0, v20                                ; 4c282880
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a2
	v_dot4_i32_iu8 v22, v22, v38, v19 neg_lo:[1,0,0]            ; cc164016 3c4e4d16
	v_dot4_i32_iu8 v23, v23, v39, v20 neg_lo:[1,0,0]            ; cc164017 3c524f17
	v_add3_u32 v23, v23, v21, v22                               ; d6550017 045a2b17
	s_wait_loadcnt 0x0                                          ; bfc00000
	v_dot4_i32_iu8 v21, v24, s8, 0 neg_lo:[1,0,0]               ; cc164015 3a001118
	v_dot4_i32_iu8 v22, v25, s8, 0 neg_lo:[1,0,0]               ; cc164016 3a001119
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_1) | instid1(VALU_DEP_2) ; bf870122
	v_lshlrev_b32_e32 v21, 5, v21                               ; 302a2a85
	v_lshlrev_b32_e32 v22, 5, v22                               ; 302c2c85
	v_sub_nc_u32_e32 v21, 0, v21                                ; 4c2a2a80
	v_sub_nc_u32_e32 v22, 0, v22                                ; 4c2c2c80
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a2
	v_dot4_i32_iu8 v24, v24, v40, v21 neg_lo:[1,0,0]            ; cc164018 3c565118
	v_dot4_i32_iu8 v25, v25, v41, v22 neg_lo:[1,0,0]            ; cc164019 3c5a5319
	v_add3_u32 v25, v25, v23, v24                               ; d6550019 04622f19
	v_dot4_i32_iu8 v23, v26, s8, 0 neg_lo:[1,0,0]               ; cc164017 3a00111a
	v_dot4_i32_iu8 v24, v27, s8, 0 neg_lo:[1,0,0]               ; cc164018 3a00111b
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_1) | instid1(VALU_DEP_2) ; bf870122
	v_lshlrev_b32_e32 v23, 5, v23                               ; 302e2e85
	v_lshlrev_b32_e32 v24, 5, v24                               ; 30303085
	v_sub_nc_u32_e32 v23, 0, v23                                ; 4c2e2e80
	v_sub_nc_u32_e32 v24, 0, v24                                ; 4c303080
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a2
	v_dot4_i32_iu8 v26, v26, v42, v23 neg_lo:[1,0,0]            ; cc16401a 3c5e551a
	v_dot4_i32_iu8 v27, v27, v43, v24 neg_lo:[1,0,0]            ; cc16401b 3c62571b
	v_add3_u32 v1, v27, v25, v26                                ; d6550001 046a331b
	s_branch BB2                                                ; bfa0ff13
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