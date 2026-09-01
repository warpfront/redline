// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
//
// Microwave packaging proof (2026-09-01): a HIP kernel whose entire body is
// foreign ISA text, standing in for an ACO-emitted body. hipcc/LLVM owns the
// kernarg prologue, the amdhsa kernel descriptor and the VGPR/SGPR accounting
// (from the clobber list); the body sees a fixed register ABI we choose. The
// resulting HSACO launches through plain HIP, redline PM4, or any HSACO loader.
//
// Build: hipcc --offload-arch=gfx1201 -O3 microwave_pack_proof.hip -o mpp
// Verified: gfx1201, ROCm 10.0, n=1048576 mismatches=0 (--save-temps shows the
// 4-instruction prologue and next_free_vgpr 10 / next_free_sgpr 17).
//
// Radiowave tunes what LLVM emits; Microwave reheats what ACO already cooked.

#include <hip/hip_runtime.h>
#include <cstdio>
#include <vector>

extern "C" __global__ void __launch_bounds__(64)
foreign_add(const int* __restrict__ a, const int* __restrict__ b, int* __restrict__ out, int n) {
    // ABI handed to the body: s[8:9]=a s[10:11]=b s[12:13]=out s14=n s15=tgid.x v0=tid.x
    asm volatile(R"(
        s_lshl_b32 s15, s15, 6
        v_add_nc_u32 v1, s15, v0
        v_cmp_gt_i32 vcc_lo, s14, v1
        s_and_saveexec_b32 s16, vcc_lo
        s_cbranch_execz 1f
        v_ashrrev_i32 v2, 31, v1
        v_lshlrev_b64 v[2:3], 2, v[1:2]
        v_add_co_u32 v4, vcc_lo, s8, v2
        v_add_co_ci_u32 v5, vcc_lo, s9, v3, vcc_lo
        v_add_co_u32 v6, vcc_lo, s10, v2
        v_add_co_ci_u32 v7, vcc_lo, s11, v3, vcc_lo
        v_add_co_u32 v8, vcc_lo, s12, v2
        v_add_co_ci_u32 v9, vcc_lo, s13, v3, vcc_lo
        global_load_b32 v4, v[4:5], off
        global_load_b32 v6, v[6:7], off
        s_wait_loadcnt 0x0
        v_add_nc_u32 v4, v4, v6
        global_store_b32 v[8:9], v4, off
    1:
        s_endpgm
    )"
                 :
                 : "{s[8:9]}"(a), "{s[10:11]}"(b), "{s[12:13]}"(out), "{s14}"(n),
                   "{s15}"(__builtin_amdgcn_workgroup_id_x()),
                   "{v0}"(__builtin_amdgcn_workitem_id_x())
                 : "v1", "v2", "v3", "v4", "v5", "v6", "v7", "v8", "v9", "s16", "vcc",
                   "memory");
}

#define OK(x) do { hipError_t e = (x); if (e != hipSuccess) { printf("FATAL %s -> %s\n", #x, hipGetErrorString(e)); return 1; } } while (0)

int main() {
    const int n = 1 << 20;
    std::vector<int> ha(n), hb(n), ho(n, -1);
    for (int i = 0; i < n; ++i) { ha[i] = i * 3; hb[i] = 7 - i; }
    int *da, *db, *dout;
    OK(hipMalloc(&da, n * 4)); OK(hipMalloc(&db, n * 4)); OK(hipMalloc(&dout, n * 4));
    OK(hipMemcpy(da, ha.data(), n * 4, hipMemcpyHostToDevice));
    OK(hipMemcpy(db, hb.data(), n * 4, hipMemcpyHostToDevice));
    OK(hipMemset(dout, 0xff, n * 4));
    hipLaunchKernelGGL(foreign_add, dim3((n + 63) / 64), dim3(64), 0, 0, da, db, dout, n);
    OK(hipGetLastError());
    OK(hipDeviceSynchronize());
    OK(hipMemcpy(ho.data(), dout, n * 4, hipMemcpyDeviceToHost));
    long bad = 0;
    for (int i = 0; i < n; ++i) if (ho[i] != ha[i] + hb[i]) ++bad;
    hipDeviceProp_t p; OK(hipGetDeviceProperties(&p, 0));
    printf("%s (%s): n=%d mismatches=%ld -> %s\n", p.name, p.gcnArchName, n, bad, bad ? "FAIL" : "PASS");
    return bad ? 2 : 0;
}
