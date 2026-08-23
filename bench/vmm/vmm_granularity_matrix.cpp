// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
//
// VMM granularity / hipMemSetAccess sub-range matrix, run over every visible device.
//
// Answers, per architecture:
//   1. what granularity does the runtime report (MINIMUM vs RECOMMENDED)?
//   2. does a MINIMUM-granularity (4 KiB) map work end to end, INCLUDING a
//      kernel write and an exact readback?
//   3. is per-segment hipMemSetAccess at successive 4 KiB offsets accepted?
//   4. exactly which sub-ranges does hipMemSetAccess accept?
//
// Build (add every arch you want to cover):
//   hipcc --offload-arch=gfx1010 --offload-arch=gfx1030 \
//         --offload-arch=gfx1100 --offload-arch=gfx1151 \
//         --offload-arch=gfx1201 -O2 vmm_granularity_matrix.cpp -o vmm_matrix
//
// Measured on ROCm 7.14.0 across gfx1010 / gfx1030 / gfx1100 / gfx1151 / gfx1201:
// every one reports MINIMUM=4 KiB, RECOMMENDED=2 MiB; 4 KiB maps pass with an
// exact kernel-written readback; and hipMemSetAccess is accepted iff BOTH the
// offset and the length are multiples of the mapped handles' granularity.
#include <hip/hip_runtime.h>
#include <cstdio>
#include <vector>

__global__ void touch(int* p, size_t n) {
    size_t i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i < n) p[i] = static_cast<int>(i + 1);
}

struct Env {
    hipMemAllocationProp prop{};
    hipMemAccessDesc desc{};
    explicit Env(int d) {
        prop.type = hipMemAllocationTypePinned;
        prop.location.type = hipMemLocationTypeDevice;
        prop.location.id = d;
        desc.location.type = hipMemLocationTypeDevice;
        desc.location.id = d;
        desc.flags = hipMemAccessFlagsProtReadWrite;
    }
};

// (2) map `sz`, set access, kernel-write, verify.
static const char* map_touch(Env& e, size_t sz) {
    hipMemGenericAllocationHandle_t h;
    if (hipMemCreate(&h, sz, &e.prop, 0) != hipSuccess) return "create-FAIL";
    void* p = nullptr;
    if (hipMemAddressReserve(&p, sz, sz, nullptr, 0) != hipSuccess) {
        hipMemRelease(h);
        return "reserve-FAIL";
    }
    if (hipMemMap(p, sz, 0, h, 0) != hipSuccess) {
        hipMemAddressFree(p, sz);
        hipMemRelease(h);
        return "map-FAIL";
    }
    const char* r = "ok";
    if (hipMemSetAccess(p, sz, &e.desc, 1) != hipSuccess) {
        r = "setaccess-FAIL";
    } else {
        size_t n = sz / sizeof(int);
        touch<<<(n + 255) / 256, 256>>>(static_cast<int*>(p), n);
        if (hipDeviceSynchronize() != hipSuccess) {
            r = "kernel-FAULT";
        } else {
            std::vector<int> hb(n);
            hipMemcpy(hb.data(), p, sz, hipMemcpyDeviceToHost);
            for (size_t i = 0; i < n; ++i)
                if (hb[i] != static_cast<int>(i + 1)) { r = "readback-WRONG"; break; }
        }
    }
    hipMemUnmap(p, sz);
    hipMemAddressFree(p, sz);
    hipMemRelease(h);
    return r;
}

// (3) map segment i then SetAccess ONLY that segment. Returns first rejected
// offset, or -1 when every segment was accepted.
static long incr_first_reject(Env& e, size_t gran, int nseg) {
    size_t total = gran * nseg;
    void* base = nullptr;
    if (hipMemAddressReserve(&base, total, gran, nullptr, 0) != hipSuccess) return -2;
    std::vector<hipMemGenericAllocationHandle_t> hs;
    long bad = -1;
    size_t mapped = 0;
    for (int i = 0; i < nseg; ++i) {
        hipMemGenericAllocationHandle_t h;
        if (hipMemCreate(&h, gran, &e.prop, 0) != hipSuccess) { bad = -3; break; }
        void* a = static_cast<char*>(base) + mapped;
        if (hipMemMap(a, gran, 0, h, 0) != hipSuccess) { hipMemRelease(h); bad = -4; break; }
        hs.push_back(h);
        if (hipMemSetAccess(a, gran, &e.desc, 1) != hipSuccess && bad == -1)
            bad = static_cast<long>(mapped);
        mapped += gran;
    }
    for (size_t i = 0; i < hs.size(); ++i)
        hipMemUnmap(static_cast<char*>(base) + i * gran, gran);
    hipMemAddressFree(base, total);
    for (auto& h : hs) hipMemRelease(h);
    return bad;
}

// (4) four handles of size C; which sub-ranges does SetAccess accept?
static void rule_table(Env& e, size_t C) {
    const int N = 4;
    void* base = nullptr;
    if (hipMemAddressReserve(&base, C * N, C, nullptr, 0) != hipSuccess) {
        printf("      reserve failed\n");
        return;
    }
    std::vector<hipMemGenericAllocationHandle_t> hs;
    for (int i = 0; i < N; ++i) {
        hipMemGenericAllocationHandle_t h;
        if (hipMemCreate(&h, C, &e.prop, 0) != hipSuccess) break;
        if (hipMemMap(static_cast<char*>(base) + i * C, C, 0, h, 0) != hipSuccess) {
            hipMemRelease(h);
            break;
        }
        hs.push_back(h);
    }
    struct Case { size_t off, len; const char* note; } cs[] = {
        {0, C, "handle 0 exactly       "},
        {C, C, "handle 1 exactly       "},
        {0, static_cast<size_t>(N) * C, "all handles exactly    "},
        {C / 2, C, "unaligned off, straddle"},
        {C / 2, C / 2, "unaligned off, in-handle"},
        {0, C / 2, "aligned off, short len "},
        {C, C / 2, "aligned off, half len  "},
        {4096, 4096, "4 KiB slice in handle  "},
    };
    for (auto& c : cs) {
        hipError_t r = hipMemSetAccess(static_cast<char*>(base) + c.off, c.len, &e.desc, 1);
        printf("      %s off=%-9zu len=%-9zu -> %s\n", c.note, c.off, c.len,
               r == hipSuccess ? "OK" : hipGetErrorString(r));
    }
    for (size_t i = 0; i < hs.size(); ++i)
        hipMemUnmap(static_cast<char*>(base) + i * C, C);
    hipMemAddressFree(base, C * N);
    for (auto& h : hs) hipMemRelease(h);
}

int main() {
    int nd = 0;
    if (hipGetDeviceCount(&nd) != hipSuccess) { printf("hipGetDeviceCount failed\n"); return 1; }
    printf("ROCm VMM granularity matrix over %d visible device(s)\n", nd);
    for (int d = 0; d < nd; ++d) {
        hipDeviceProp_t p;
        if (hipGetDeviceProperties(&p, d) != hipSuccess) continue;
        int vmm = 0;
        hipDeviceGetAttribute(&vmm, hipDeviceAttributeVirtualMemoryManagementSupported, d);
        printf("\n===== dev %d: %-28s [%s] VMM=%d =====\n", d, p.name, p.gcnArchName, vmm);
        if (!vmm) { printf("   VMM unsupported -> skip\n"); continue; }
        if (hipSetDevice(d) != hipSuccess) { printf("   setDevice failed\n"); continue; }
        Env e(d);
        size_t gmin = 0, grec = 0;
        hipMemGetAllocationGranularity(&gmin, &e.prop, hipMemAllocationGranularityMinimum);
        hipMemGetAllocationGranularity(&grec, &e.prop, hipMemAllocationGranularityRecommended);
        printf("   granularity: MINIMUM=%zu (%.0f KiB)  RECOMMENDED=%zu (%.0f KiB)\n", gmin,
               gmin / 1024.0, grec, grec / 1024.0);
        printf("   (2) map+SetAccess+kernel write+verify @MIN : %s\n", map_touch(e, gmin));
        printf("   (2) map+SetAccess+kernel write+verify @REC : %s\n", map_touch(e, grec));
        long b = incr_first_reject(e, gmin, 8);
        if (b == -1)
            printf("   (3) per-segment SetAccess, 8 x %zu B: ALL ACCEPTED (incl. base+16 KiB)\n", gmin);
        else if (b >= 0)
            printf("   (3) per-segment SetAccess: FIRST REJECT at base+%ld\n", b);
        else
            printf("   (3) per-segment SetAccess: setup error %ld\n", b);
        printf("   (4) sub-range rule at handle size %zu:\n", grec);
        rule_table(e, grec);
    }
    printf("\n===== matrix complete =====\n");
    return 0;
}
