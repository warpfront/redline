BB0:
	v_lshl_add_u32 v0, s8, 6, v0                                ; d6460000 04010c08
	s_delay_alu instid0(VALU_DEP_1)                             ; bf870001
	v_cmpx_gt_u32_e32 s3, v0                                    ; 7d980003
	s_cbranch_execz BB11                                        ; bfa500c6
BB1:
	s_mov_b32 s0, 0                                             ; be800080
	v_mov_b32_e32 v1, 0                                         ; 7e020280
BB2:
	s_cmp_ge_u32 s0, s4                                         ; bf090400
	s_cbranch_scc1 BB6                                          ; bfa200b9
BB5:
	s_mov_b32 s8, s2                                            ; be880002
	s_movk_i32 s9, 0x8000                                       ; b0098000
	s_load_b256 s[8:15], s[8:9], null                           ; f40c0204 f8000000
	s_mul_i32 s1, s0, s3                                        ; 96010300
	s_delay_alu instid0(SALU_CYCLE_1) | instskip(NEXT) | instid1(VALU_DEP_1) ; bf870099
	v_add_lshl_u32 v2, s1, v0, 4                                ; d6470002 02120001
	v_and_b32_e32 v2, s5, v2                                    ; 36040405
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_3) | instid1(VALU_DEP_3) ; bf8701c1
	v_lshlrev_b32_e32 v3, 2, v2                                 ; 30060482
	v_add_nc_u32_e32 v4, 1, v2                                  ; 4a080481
	v_add_nc_u32_e32 v7, 2, v2                                  ; 4a0e0482
	v_add_nc_u32_e32 v8, 3, v2                                  ; 4a100483
	v_and_b32_e32 v4, s5, v4                                    ; 36080805
	v_add_nc_u32_e32 v11, 4, v2                                 ; 4a160484
	v_add_nc_u32_e32 v12, 5, v2                                 ; 4a180485
	v_and_b32_e32 v7, s5, v7                                    ; 360e0e05
	v_and_b32_e32 v8, s5, v8                                    ; 36101005
	v_lshlrev_b32_e32 v4, 2, v4                                 ; 30080882
	v_and_b32_e32 v11, s5, v11                                  ; 36161605
	v_and_b32_e32 v12, s5, v12                                  ; 36181805
	v_lshlrev_b32_e32 v7, 2, v7                                 ; 300e0e82
	v_add_nc_u32_e32 v15, 6, v2                                 ; 4a1e0486
	v_add_nc_u32_e32 v16, 7, v2                                 ; 4a200487
	v_lshlrev_b32_e32 v8, 2, v8                                 ; 30101082
	v_lshlrev_b32_e32 v11, 2, v11                               ; 30161682
	v_lshlrev_b32_e32 v12, 2, v12                               ; 30181882
	v_and_b32_e32 v15, s5, v15                                  ; 361e1e05
	v_and_b32_e32 v16, s5, v16                                  ; 36202005
	s_delay_alu instid0(VALU_DEP_2) | instskip(SKIP_3) | instid1(VALU_DEP_3) ; bf8701c2
	v_lshlrev_b32_e32 v15, 2, v15                               ; 301e1e82
	v_add_nc_u32_e32 v19, 8, v2                                 ; 4a260488
	v_add_nc_u32_e32 v20, 9, v2                                 ; 4a280489
	v_lshlrev_b32_e32 v16, 2, v16                               ; 30202082
	v_and_b32_e32 v19, s5, v19                                  ; 36262605
	v_and_b32_e32 v20, s5, v20                                  ; 36282805
	s_delay_alu instid0(VALU_DEP_2)                             ; bf870002
	v_lshlrev_b32_e32 v19, 2, v19                               ; 30262682
	v_lshlrev_b32_e32 v20, 2, v20                               ; 30282882
	s_waitcnt lgkmcnt(0)                                        ; bf89fc07
	s_clause 0x9                                                ; bf850009
	buffer_load_b32 v5, v3, s[8:11], 0 offen                    ; e0500000 80420503
	buffer_load_b32 v6, v4, s[8:11], 0 offen                    ; e0500000 80420604
	buffer_load_b32 v9, v7, s[8:11], 0 offen                    ; e0500000 80420907
	buffer_load_b32 v10, v8, s[8:11], 0 offen                   ; e0500000 80420a08
	buffer_load_b32 v13, v11, s[8:11], 0 offen                  ; e0500000 80420d0b
	buffer_load_b32 v14, v12, s[8:11], 0 offen                  ; e0500000 80420e0c
	buffer_load_b32 v17, v15, s[8:11], 0 offen                  ; e0500000 8042110f
	buffer_load_b32 v18, v16, s[8:11], 0 offen                  ; e0500000 80421210
	buffer_load_b32 v21, v19, s[8:11], 0 offen                  ; e0500000 80421513
	buffer_load_b32 v22, v20, s[8:11], 0 offen                  ; e0500000 80421614
	v_add_nc_u32_e32 v23, 10, v2                                ; 4a2e048a
	v_add_nc_u32_e32 v24, 11, v2                                ; 4a30048b
	v_add_nc_u32_e32 v27, 12, v2                                ; 4a36048c
	v_add_nc_u32_e32 v29, 13, v2                                ; 4a3a048d
	v_add_nc_u32_e32 v31, 14, v2                                ; 4a3e048e
	v_and_b32_e32 v23, s5, v23                                  ; 362e2e05
	v_and_b32_e32 v24, s5, v24                                  ; 36303005
	v_and_b32_e32 v27, s5, v27                                  ; 36363605
	v_add_nc_u32_e32 v2, 15, v2                                 ; 4a04048f
	v_and_b32_e32 v29, s5, v29                                  ; 363a3a05
	v_and_b32_e32 v31, s5, v31                                  ; 363e3e05
	v_lshlrev_b32_e32 v23, 2, v23                               ; 302e2e82
	v_lshlrev_b32_e32 v24, 2, v24                               ; 30303082
	v_lshlrev_b32_e32 v27, 2, v27                               ; 30363682
	v_and_b32_e32 v2, s5, v2                                    ; 36040405
	v_lshlrev_b32_e32 v29, 2, v29                               ; 303a3a82
	v_lshlrev_b32_e32 v31, 2, v31                               ; 303e3e82
	s_delay_alu instid0(VALU_DEP_3)                             ; bf870003
	v_lshlrev_b32_e32 v2, 2, v2                                 ; 30040482
	s_clause 0x5                                                ; bf850005
	buffer_load_b32 v25, v23, s[8:11], 0 offen                  ; e0500000 80421917
	buffer_load_b32 v26, v24, s[8:11], 0 offen                  ; e0500000 80421a18
	buffer_load_b32 v28, v27, s[8:11], 0 offen                  ; e0500000 80421c1b
	buffer_load_b32 v30, v29, s[8:11], 0 offen                  ; e0500000 80421e1d
	buffer_load_b32 v32, v31, s[8:11], 0 offen                  ; e0500000 8042201f
	buffer_load_b32 v33, v2, s[8:11], 0 offen                   ; e0500000 80422102
	s_clause 0xf                                                ; bf85000f
	buffer_load_b32 v3, v3, s[12:15], 0 offen                   ; e0500000 80430303
	buffer_load_b32 v4, v4, s[12:15], 0 offen                   ; e0500000 80430404
	buffer_load_b32 v7, v7, s[12:15], 0 offen                   ; e0500000 80430707
	buffer_load_b32 v8, v8, s[12:15], 0 offen                   ; e0500000 80430808
	buffer_load_b32 v11, v11, s[12:15], 0 offen                 ; e0500000 80430b0b
	buffer_load_b32 v12, v12, s[12:15], 0 offen                 ; e0500000 80430c0c
	buffer_load_b32 v15, v15, s[12:15], 0 offen                 ; e0500000 80430f0f
	buffer_load_b32 v16, v16, s[12:15], 0 offen                 ; e0500000 80431010
	buffer_load_b32 v19, v19, s[12:15], 0 offen                 ; e0500000 80431313
	buffer_load_b32 v20, v20, s[12:15], 0 offen                 ; e0500000 80431414
	buffer_load_b32 v23, v23, s[12:15], 0 offen                 ; e0500000 80431717
	buffer_load_b32 v24, v24, s[12:15], 0 offen                 ; e0500000 80431818
	buffer_load_b32 v27, v27, s[12:15], 0 offen                 ; e0500000 80431b1b
	buffer_load_b32 v29, v29, s[12:15], 0 offen                 ; e0500000 80431d1d
	buffer_load_b32 v31, v31, s[12:15], 0 offen                 ; e0500000 80431f1f
	buffer_load_b32 v2, v2, s[12:15], 0 offen                   ; e0500000 80430202
	s_add_u32 s0, s0, 1                                         ; 80008100
	s_waitcnt vmcnt(15)                                         ; bf893ff7
	v_dot4_i32_iu8 v5, v5, v3, v1 neg_lo:[1,1,0]                ; cc164005 7c060705
	s_waitcnt vmcnt(14)                                         ; bf893bf7
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_dot4_i32_iu8 v6, v6, v4, v5 neg_lo:[1,1,0]                ; cc164006 7c160906
	s_waitcnt vmcnt(13)                                         ; bf8937f7
	v_dot4_i32_iu8 v9, v9, v7, v6 neg_lo:[1,1,0]                ; cc164009 7c1a0f09
	s_waitcnt vmcnt(12)                                         ; bf8933f7
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_dot4_i32_iu8 v10, v10, v8, v9 neg_lo:[1,1,0]              ; cc16400a 7c26110a
	s_waitcnt vmcnt(11)                                         ; bf892ff7
	v_dot4_i32_iu8 v13, v13, v11, v10 neg_lo:[1,1,0]            ; cc16400d 7c2a170d
	s_waitcnt vmcnt(10)                                         ; bf892bf7
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_dot4_i32_iu8 v14, v14, v12, v13 neg_lo:[1,1,0]            ; cc16400e 7c36190e
	s_waitcnt vmcnt(9)                                          ; bf8927f7
	v_dot4_i32_iu8 v17, v17, v15, v14 neg_lo:[1,1,0]            ; cc164011 7c3a1f11
	s_waitcnt vmcnt(8)                                          ; bf8923f7
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_dot4_i32_iu8 v18, v18, v16, v17 neg_lo:[1,1,0]            ; cc164012 7c462112
	s_waitcnt vmcnt(7)                                          ; bf891ff7
	v_dot4_i32_iu8 v21, v21, v19, v18 neg_lo:[1,1,0]            ; cc164015 7c4a2715
	s_waitcnt vmcnt(6)                                          ; bf891bf7
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_dot4_i32_iu8 v22, v22, v20, v21 neg_lo:[1,1,0]            ; cc164016 7c562916
	s_waitcnt vmcnt(5)                                          ; bf8917f7
	v_dot4_i32_iu8 v25, v25, v23, v22 neg_lo:[1,1,0]            ; cc164019 7c5a2f19
	s_waitcnt vmcnt(4)                                          ; bf8913f7
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_dot4_i32_iu8 v26, v26, v24, v25 neg_lo:[1,1,0]            ; cc16401a 7c66311a
	s_waitcnt vmcnt(3)                                          ; bf890ff7
	v_dot4_i32_iu8 v28, v28, v27, v26 neg_lo:[1,1,0]            ; cc16401c 7c6a371c
	s_waitcnt vmcnt(2)                                          ; bf890bf7
	s_delay_alu instid0(VALU_DEP_1) | instskip(SKIP_1) | instid1(VALU_DEP_1) ; bf8700a1
	v_dot4_i32_iu8 v30, v30, v29, v28 neg_lo:[1,1,0]            ; cc16401e 7c723b1e
	s_waitcnt vmcnt(1)                                          ; bf8907f7
	v_dot4_i32_iu8 v32, v32, v31, v30 neg_lo:[1,1,0]            ; cc164020 7c7a3f20
	s_waitcnt vmcnt(0)                                          ; bf8903f7
	s_delay_alu instid0(VALU_DEP_1)                             ; bf870001
	v_dot4_i32_iu8 v1, v33, v2, v32 neg_lo:[1,1,0]              ; cc164001 7c820521
	s_branch BB2                                                ; bfa0ff45
BB6:
	s_movk_i32 s3, 0x8000                                       ; b0038000
	s_load_b128 s[0:3], s[2:3], 0x20                            ; f4080001 f8000020
	v_add_nc_u32_e32 v1, s7, v1                                 ; 4a020207
	v_add_lshl_u32 v0, s6, v0, 2                                ; d6470000 020a0006
	s_waitcnt lgkmcnt(0)                                        ; bf89fc07
	buffer_store_b32 v1, v0, s[0:3], 0 offen                    ; e0680000 80400100
BB11:
	s_endpgm                                                    ; bfb00000
 