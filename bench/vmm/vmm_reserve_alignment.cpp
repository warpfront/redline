// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
//
// What alignment does hipMemAddressReserve give when the caller passes
// alignment = 0, and does a misaligned base make hipMemSetAccess fail?
//
// Why the probe is shaped this way:
//   llama.cpp's VMM pool (ggml/src/ggml-cuda/ggml-cuda.cu) reserves its pool
//   with `cuMemAddressReserve(&pool_addr, MAX_SIZE, 0, 0, 0)` -- alignment 0 --
//   and then maps each new chunk at `pool_addr + pool_size`, where pool_size
//   only ever grows by multiples of the RECOMMENDED granularity. So every
//   mapping offset inherits the alignment of pool_addr. Separately we measured
//   that hipMemSetAccess succeeds only when both the offset and the length are
//   multiples of the mapped handles' granularity.
//
//   If hipMemAddressReserve(alignment = 0) can return an address that is only
//   page-aligned rather than granularity-aligned, those two facts together
//   would explain llama.cpp PR #26054: hipMemSetAccess returning
//   hipErrorInvalidValue on gfx1201, which is why that project ships with
//   GGML_HIP_NO_VMM defaulted ON.
//
//   Arm 1 reserves with alignment 0 and reports the alignment actually obtained.
//   Arm 2 reserves with alignment = granularity for comparison.
//   Arm 3 deliberately maps at a granularity-aligned base plus a sub-granularity
//   offset and confirms which call rejects it.
//
//   This probe reports observed alignments and error codes only. It does not
//   claim what alignment the API is contractually required to return.
//
// Build: hipcc --offload-arch=gfx1201 -O2 vmm_reserve_alignment.cpp -o vmm_reserve_alignment
// Run:   ./vmm_reserve_alignment

#include <hip/hip_runtime.h>
#include <cstdio>
#include <cstdlib>

#define OK(x)                                                                    \
    do {                                                                         \
        hipError_t _e = (x);                                                     \
        if (_e != hipSuccess) {                                                  \
            printf("  FATAL %s -> %s (%d)\n", #x, hipGetErrorString(_e), (int)_e); \
            return 1;                                                            \
        }                                                                        \
    } while (0)

// Largest power-of-two alignment satisfied by p, capped for readability.
static const char* align_str(unsigned long long p) {
    static char buf[64];
    if (p == 0) { snprintf(buf, sizeof(buf), "null"); return buf; }
    unsigned long long a = p & (~p + 1ull);  // lowest set bit
    if (a >= (1ull << 30))      snprintf(buf, sizeof(buf), "%llu GiB", a >> 30);
    else if (a >= (1ull << 20)) snprintf(buf, sizeof(buf), "%llu MiB", a >> 20);
    else if (a >= (1ull << 10)) snprintf(buf, sizeof(buf), "%llu KiB", a >> 10);
    else                        snprintf(buf, sizeof(buf), "%llu B", a);
    return buf;
}

int main() {
    OK(hipSetDevice(0));
    hipDeviceProp_t p;
    OK(hipGetDeviceProperties(&p, 0));

    hipMemAllocationProp prop{};
    prop.type = hipMemAllocationTypePinned;
    prop.location.type = hipMemLocationTypeDevice;
    prop.location.id = 0;

    size_t gran_min = 0, gran_rec = 0;
    OK(hipMemGetAllocationGranularity(&gran_min, &prop, hipMemAllocationGranularityMinimum));
    OK(hipMemGetAllocationGranularity(&gran_rec, &prop, hipMemAllocationGranularityRecommended));

    printf("=== %s (%s) ===\n", p.name, p.gcnArchName);
    printf("granularity: minimum %zu B (%zu KiB) | recommended %zu B (%zu MiB)\n\n",
           gran_min, gran_min >> 10, gran_rec, gran_rec >> 20);

    // llama.cpp reserves 4 GiB of VA for its pool; mirror that magnitude.
    const size_t reserve_span = 4ull << 30;

    // --- Arm 1: alignment = 0, exactly what llama.cpp passes ---
    void* base0 = nullptr;
    OK(hipMemAddressReserve(&base0, reserve_span, 0, nullptr, 0));
    const bool ok0 = ((unsigned long long)base0 % gran_rec) == 0;
    printf("arm 1  hipMemAddressReserve(alignment=0)        -> %p\n", base0);
    printf("       satisfies alignment: %-10s | multiple of recommended granularity: %s\n",
           align_str((unsigned long long)base0), ok0 ? "YES" : "NO");

    // --- Arm 2: alignment = recommended granularity ---
    void* base1 = nullptr;
    OK(hipMemAddressReserve(&base1, reserve_span, gran_rec, nullptr, 0));
    const bool ok1 = ((unsigned long long)base1 % gran_rec) == 0;
    printf("arm 2  hipMemAddressReserve(alignment=%zu MiB) -> %p\n", gran_rec >> 20, base1);
    printf("       satisfies alignment: %-10s | multiple of recommended granularity: %s\n\n",
           align_str((unsigned long long)base1), ok1 ? "YES" : "NO");

    // --- Arm 3: which call rejects a sub-granularity offset? ---
    // Map a recommended-granularity handle at base + gran_min (4 KiB on RDNA),
    // i.e. page-aligned but NOT granularity-aligned, and see where it fails.
    printf("arm 3  mapping a %zu MiB handle at base + %zu B (page-aligned, not\n"
           "       granularity-aligned) to see which call rejects the offset:\n",
           gran_rec >> 20, gran_min);

    hipMemGenericAllocationHandle_t h;
    OK(hipMemCreate(&h, gran_rec, &prop, 0));

    char* skewed = (char*)base1 + gran_min;
    hipError_t e_map = hipMemMap(skewed, gran_rec, 0, h, 0);
    printf("       hipMemMap      -> %s (%d)\n", hipGetErrorString(e_map), (int)e_map);

    if (e_map == hipSuccess) {
        hipMemAccessDesc d{};
        d.location.type = hipMemLocationTypeDevice;
        d.location.id = 0;
        d.flags = hipMemAccessFlagsProtReadWrite;
        hipError_t e_acc = hipMemSetAccess(skewed, gran_rec, &d, 1);
        printf("       hipMemSetAccess -> %s (%d)%s\n",
               hipGetErrorString(e_acc), (int)e_acc,
               e_acc == hipErrorInvalidValue
                   ? "   <-- same error llama.cpp PR #26054 reports" : "");
        (void)hipMemUnmap(skewed, gran_rec);
    }

    (void)hipMemRelease(h);
    (void)hipMemAddressFree(base0, reserve_span);
    (void)hipMemAddressFree(base1, reserve_span);

    printf("\n--- reading ---\n");
    if (!ok0) {
        printf("alignment=0 returned a base that is NOT a multiple of the recommended\n"
               "granularity. A pool that maps at base + (multiples of granularity)\n"
               "therefore inherits a misaligned offset on every chunk.\n");
    } else {
        printf("alignment=0 returned a granularity-aligned base in this run, so a\n"
               "misaligned pool base is not reproduced here. Observed alignment was %s;\n"
               "this is one sample and not a guarantee about the API's contract.\n",
               align_str((unsigned long long)base0));
    }
    return 0;
}
