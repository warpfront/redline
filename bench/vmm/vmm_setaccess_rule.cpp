// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
//
// What are hipMemMap and hipMemSetAccess actually strict about?
//
// Why the probe is shaped this way:
//   An earlier round of measurement concluded that hipMemSetAccess succeeds only
//   when both the offset and the length are multiples of the granularity of the
//   MAPPED HANDLES -- i.e. that a 2 MiB handle demands a 2 MiB-aligned offset.
//   A later probe contradicted that directly: a 2 MiB handle mapped at a merely
//   page-aligned offset was accepted by both calls. The earlier conclusion was
//   therefore either wrong or was really about the minimum granularity, and it
//   must not be published until the actual rule is pinned down.
//
//   So sweep it explicitly. For each handle size, map at a range of offsets and
//   call hipMemSetAccess over a range of lengths, and print the raw accept and
//   reject pattern with the errors. Offsets and lengths are expressed as
//   multiples of the MINIMUM granularity and of the handle size, so whichever
//   quantity the runtime keys on is visible in the result.
//
//   Cleanup is exact: a mapping is unmapped only if hipMemMap succeeded, so a
//   rejected case cannot leak into the next case and change its outcome.
//
//   This probe prints the observed pattern. It draws no conclusion beyond what
//   the pattern shows, and it deliberately reports the errors verbatim rather
//   than bucketing them.
//
// Build: hipcc --offload-arch=gfx1201 -O2 vmm_setaccess_rule.cpp -o vmm_setaccess_rule
// Run:   ./vmm_setaccess_rule

#include <hip/hip_runtime.h>
#include <cstdio>
#include <cstdlib>
#include <vector>

#define OK(x)                                                                    \
    do {                                                                         \
        hipError_t _e = (x);                                                     \
        if (_e != hipSuccess) {                                                  \
            printf("FATAL %s -> %s (%d)\n", #x, hipGetErrorString(_e), (int)_e);  \
            return 1;                                                            \
        }                                                                        \
    } while (0)

static const char* short_err(hipError_t e) {
    switch (e) {
        case hipSuccess:             return "ok";
        case hipErrorInvalidValue:   return "invalid-value";
        case hipErrorOutOfMemory:    return "out-of-memory";
        case hipErrorInvalidHandle:  return "invalid-handle";
        default:                     return hipGetErrorString(e);
    }
}

static void human(size_t b, char* out, size_t n) {
    if (b == 0)                  snprintf(out, n, "0");
    else if (b % (1ull << 20) == 0) snprintf(out, n, "%zuM", b >> 20);
    else if (b % (1ull << 10) == 0) snprintf(out, n, "%zuK", b >> 10);
    else                         snprintf(out, n, "%zuB", b);
}

int main() {
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
    printf("granularity: minimum %zu B | recommended %zu B\n\n", gmin, grec);

    // A generously aligned reservation so the base contributes no skew.
    const size_t span = 256ull << 20;
    void* base = nullptr;
    OK(hipMemAddressReserve(&base, span, grec, nullptr, 0));
    printf("reserved %zu MiB at %p (aligned to request %zu MiB)\n\n",
           span >> 20, base, grec >> 20);

    // Handle sizes to test: the minimum, an intermediate, and the recommended.
    const size_t hsizes[] = { gmin, 64ull << 10, grec };
    // Offsets from the reservation base, in absolute bytes.
    const size_t offsets[] = { 0, gmin, 2 * gmin, 64ull << 10, 1ull << 20, grec };

    for (size_t hs : hsizes) {
        char hbuf[16];
        human(hs, hbuf, sizeof(hbuf));
        printf("---- handle size %s ----\n", hbuf);
        printf("%-10s %-14s %-16s %-16s %s\n",
               "offset", "off/min", "hipMemMap", "SetAccess(len=hs)", "SetAccess(len=min)");

        for (size_t off : offsets) {
            // Skip offsets that cannot host a whole handle inside the span.
            if (off + hs > span) continue;

            char obuf[16];
            human(off, obuf, sizeof(obuf));

            hipMemGenericAllocationHandle_t h;
            hipError_t e_create = hipMemCreate(&h, hs, &prop, 0);
            if (e_create != hipSuccess) {
                printf("%-10s %-14s create failed: %s\n", obuf, "-", short_err(e_create));
                continue;
            }

            char* addr = (char*)base + off;
            hipError_t e_map = hipMemMap(addr, hs, 0, h, 0);

            const char* acc_hs = "-";
            const char* acc_min = "-";
            char acc_hs_buf[48], acc_min_buf[48];

            if (e_map == hipSuccess) {
                hipMemAccessDesc d{};
                d.location.type = hipMemLocationTypeDevice;
                d.location.id = 0;
                d.flags = hipMemAccessFlagsProtReadWrite;

                // Full-handle length.
                hipError_t e1 = hipMemSetAccess(addr, hs, &d, 1);
                snprintf(acc_hs_buf, sizeof(acc_hs_buf), "%s", short_err(e1));
                acc_hs = acc_hs_buf;

                // Sub-handle length: one minimum-granularity unit.
                if (hs > gmin) {
                    hipError_t e2 = hipMemSetAccess(addr, gmin, &d, 1);
                    snprintf(acc_min_buf, sizeof(acc_min_buf), "%s", short_err(e2));
                    acc_min = acc_min_buf;
                } else {
                    acc_min = "(same as hs)";
                }

                (void)hipMemUnmap(addr, hs);
            }

            char offmin[24];
            snprintf(offmin, sizeof(offmin), "%zu x min", gmin ? off / gmin : 0);
            printf("%-10s %-14s %-16s %-16s %s\n",
                   obuf, offmin, short_err(e_map), acc_hs, acc_min);

            (void)hipMemRelease(h);
        }
        printf("\n");
    }

    (void)hipMemAddressFree(base, span);
    printf("Pattern above is raw output. Read the columns to see which quantity\n"
           "the runtime keys on: the minimum granularity, or the handle size.\n");
    return 0;
}
