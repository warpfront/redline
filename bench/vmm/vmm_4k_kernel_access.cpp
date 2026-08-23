// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
//
// Do GPU kernels fault when accessing VMM memory built from 4 KiB handles?
//
// Why the probe is shaped this way:
//   rocm-systems PR #9360 proposes overriding the reported VMM allocation
//   granularity to 2 MiB for all device allocations, with this stated
//   motivation: "On RDNA4 GPUs (gfx1201), the runtime-reported VMM device
//   granularity is 4 KB, but GPU kernels accessing VMM-mapped memory at 4 KB-
//   aligned boundaries trigger illegal memory access faults."
//
//   That is a claim about kernel access faulting, not about an API rejecting a
//   request, and the PR is careful to separate it from the hipMemSetAccess
//   sub-buffer validation bug (SWDEV-568260). So the only way to speak to it is
//   to build a mapping out of minimum-granularity handles and have a kernel
//   actually read and write across every handle boundary.
//
//   Three access patterns, because "at 4 KB-aligned boundaries" could mean any
//   of them:
//     A. every element across the whole span, so every boundary is touched
//     B. only the words immediately on each side of each boundary
//     C. 8-byte accesses deliberately STRADDLING each boundary, i.e. one access
//        whose bytes live in two different physical handles
//   Pattern C is the one most likely to fault if adjacent handles are not
//   contiguous in the page tables, and is the hardest case for a 4 KiB mapping.
//
//   Every arm verifies by exact readback on the host, and the run also checks
//   for asynchronous faults via hipDeviceSynchronize after each launch. A pass
//   here is evidence only for this access shape on this device and ROCm build;
//   it cannot prove the reporter saw nothing.
//
// Build: hipcc --offload-arch=gfx1201 -O2 vmm_4k_kernel_access.cpp -o vmm_4k_kernel_access
// Run:   ./vmm_4k_kernel_access [num_handles]

#include <hip/hip_runtime.h>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <vector>

#define OK(x)                                                                     \
    do {                                                                          \
        hipError_t _e = (x);                                                      \
        if (_e != hipSuccess) {                                                    \
            printf("FATAL %s -> %s (%d)\n", #x, hipGetErrorString(_e), (int)_e);   \
            return 1;                                                             \
        }                                                                         \
    } while (0)

// A: touch every 32-bit word in the span.
__global__ void write_all(unsigned* p, size_t n) {
    size_t i = (size_t)blockIdx.x * blockDim.x + threadIdx.x;
    if (i < n) p[i] = (unsigned)(i * 2654435761u);
}

// B: touch only the last word of each page and the first word of the next.
__global__ void write_edges(unsigned* p, size_t words_per_page, size_t pages) {
    size_t pg = (size_t)blockIdx.x * blockDim.x + threadIdx.x;
    if (pg >= pages) return;
    size_t last = pg * words_per_page + (words_per_page - 1);
    p[last] = 0xED6E0000u | (unsigned)pg;
    if (pg + 1 < pages) {
        size_t first = (pg + 1) * words_per_page;
        p[first] = 0x0000ED6Eu | (unsigned)pg;
    }
}

// C: one 8-byte access straddling each page boundary. Bytes land in two handles.
__global__ void write_straddle(char* base, size_t page_bytes, size_t pages) {
    size_t pg = (size_t)blockIdx.x * blockDim.x + threadIdx.x;
    if (pg + 1 >= pages) return;
    // Centre an 8-byte store on the boundary: 4 bytes either side.
    char* at = base + (pg + 1) * page_bytes - 4;
    unsigned long long v = 0xABCD000000000000ull | (unsigned long long)pg;
    __builtin_memcpy(at, &v, sizeof(v));
}

int main(int argc, char** argv) {
    const size_t nh = argc > 1 ? (size_t)atoll(argv[1]) : 512;
    OK(hipSetDevice(0));

    hipDeviceProp_t p;
    OK(hipGetDeviceProperties(&p, 0));

    hipMemAllocationProp prop{};
    prop.type = hipMemAllocationTypePinned;
    prop.location.type = hipMemLocationTypeDevice;
    prop.location.id = 0;

    size_t gmin = 0, grec = 0;
    OK(hipMemGetAllocationGranularity(&gmin, &prop, hipMemAllocationGranularityMinimum));
    OK(hipMemGetAllocationGranularity(&grec, &prop, hipMemAllocationGranularityRecommended));
    printf("=== %s (%s) ===\n", p.name, p.gcnArchName);
    printf("minimum granularity %zu B | recommended %zu B\n", gmin, grec);
    printf("building a contiguous span from %zu handles of %zu B each = %zu KiB\n\n",
           nh, gmin, (nh * gmin) >> 10);

    const size_t span = nh * gmin;
    void* base = nullptr;
    OK(hipMemAddressReserve(&base, span, gmin, nullptr, 0));

    // One minimum-granularity physical handle per page, mapped back to back, so
    // every page boundary in the span is also a physical-handle boundary.
    std::vector<hipMemGenericAllocationHandle_t> hs;
    hs.reserve(nh);
    for (size_t i = 0; i < nh; ++i) {
        hipMemGenericAllocationHandle_t h;
        OK(hipMemCreate(&h, gmin, &prop, 0));
        hs.push_back(h);
        OK(hipMemMap((char*)base + i * gmin, gmin, 0, h, 0));
    }

    hipMemAccessDesc desc{};
    desc.location.type = hipMemLocationTypeDevice;
    desc.location.id = 0;
    desc.flags = hipMemAccessFlagsProtReadWrite;
    OK(hipMemSetAccess(base, span, &desc, 1));
    printf("mapped %zu handles and set access over the whole span: ok\n", nh);

    const size_t words = span / sizeof(unsigned);
    const size_t wpp = gmin / sizeof(unsigned);
    std::vector<unsigned> host(words);
    std::vector<char> hostb(span);
    int failures = 0;

    // ---- Arm A: every word ----
    OK(hipMemsetD8(base, 0, span));
    write_all<<<(words + 255) / 256, 256>>>((unsigned*)base, words);
    hipError_t eA = hipGetLastError();
    hipError_t sA = hipDeviceSynchronize();
    OK(hipMemcpy(host.data(), base, span, hipMemcpyDeviceToHost));
    size_t badA = 0;
    for (size_t i = 0; i < words; ++i)
        if (host[i] != (unsigned)(i * 2654435761u)) ++badA;
    printf("arm A  every word across %zu handle boundaries: launch %s | sync %s | mismatches %zu\n",
           nh - 1, hipGetErrorString(eA), hipGetErrorString(sA), badA);
    if (eA != hipSuccess || sA != hipSuccess || badA) ++failures;

    // ---- Arm B: words adjacent to each boundary ----
    OK(hipMemsetD8(base, 0, span));
    write_edges<<<(nh + 255) / 256, 256>>>((unsigned*)base, wpp, nh);
    hipError_t eB = hipGetLastError();
    hipError_t sB = hipDeviceSynchronize();
    OK(hipMemcpy(host.data(), base, span, hipMemcpyDeviceToHost));
    size_t badB = 0;
    for (size_t pg = 0; pg < nh; ++pg) {
        if (host[pg * wpp + (wpp - 1)] != (0xED6E0000u | (unsigned)pg)) ++badB;
        if (pg + 1 < nh && host[(pg + 1) * wpp] != (0x0000ED6Eu | (unsigned)pg)) ++badB;
    }
    printf("arm B  words on both sides of each boundary:       launch %s | sync %s | mismatches %zu\n",
           hipGetErrorString(eB), hipGetErrorString(sB), badB);
    if (eB != hipSuccess || sB != hipSuccess || badB) ++failures;

    // ---- Arm C: 8-byte accesses straddling each boundary ----
    OK(hipMemsetD8(base, 0, span));
    write_straddle<<<(nh + 255) / 256, 256>>>((char*)base, gmin, nh);
    hipError_t eC = hipGetLastError();
    hipError_t sC = hipDeviceSynchronize();
    OK(hipMemcpy(hostb.data(), base, span, hipMemcpyDeviceToHost));
    size_t badC = 0;
    for (size_t pg = 0; pg + 1 < nh; ++pg) {
        unsigned long long got = 0, want = 0xABCD000000000000ull | (unsigned long long)pg;
        memcpy(&got, hostb.data() + (pg + 1) * gmin - 4, sizeof(got));
        if (got != want) ++badC;
    }
    printf("arm C  8B stores STRADDLING each boundary:         launch %s | sync %s | mismatches %zu\n",
           hipGetErrorString(eC), hipGetErrorString(sC), badC);
    if (eC != hipSuccess || sC != hipSuccess || badC) ++failures;

    OK(hipMemUnmap(base, span));
    for (auto& h : hs) (void)hipMemRelease(h);
    (void)hipMemAddressFree(base, span);

    printf("\n%s\n", failures == 0
        ? "No fault and no corruption in any arm. Evidence for these access shapes\n"
          "on this device and ROCm build only; it cannot prove the reporter saw nothing."
        : "At least one arm faulted or mis-compared -- see the per-arm lines above.");
    return failures == 0 ? 0 : 2;
}
