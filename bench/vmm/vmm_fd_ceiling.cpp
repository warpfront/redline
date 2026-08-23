// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
//
// Minimal reproducer: HIP VMM capacity is bounded by RLIMIT_NOFILE, not VRAM.
//
// Each hipMemCreate handle appears to consume one file descriptor, so with the
// common default of `ulimit -n 1024` a process can map only ~2 GiB of VMM
// regardless of how much VRAM is free. Raising the limit removes the ceiling.
//
// Build:
//   hipcc --offload-arch=gfx1201 -O2 vmm_fd_ceiling.cpp -o vmm_fd_ceiling
// Run (both arms):
//   ./vmm_fd_ceiling
//   bash -c 'ulimit -n 65536; ./vmm_fd_ceiling'
//
// Measured on Radeon AI PRO R9700 (gfx1201, 32 GiB), ROCm 7.14.0, 2 MiB
// granularity:
//   ulimit -n 1024   -> 1015 handles  =  1.98 GiB   (31.79 GiB free)
//   ulimit -n 65536  -> 9765 handles  = 19.07 GiB   (31.79 GiB free)
//   ulimit -n 262144 -> 15786 handles = 30.83 GiB   (31.79 GiB free)
#include <hip/hip_runtime.h>
#include <sys/resource.h>
#include <cstdio>
#include <vector>

#define OK(x)                                                                      \
    do {                                                                           \
        hipError_t _e = (x);                                                       \
        if (_e != hipSuccess) {                                                    \
            printf("fatal: %s -> %s\n", #x, hipGetErrorString(_e));                \
            return 2;                                                              \
        }                                                                          \
    } while (0)

int main(int argc, char** argv) {
    const int dev = argc > 1 ? atoi(argv[1]) : 0;
    OK(hipSetDevice(dev));

    hipDeviceProp_t prop_dev;
    OK(hipGetDeviceProperties(&prop_dev, dev));

    rlimit rl{};
    getrlimit(RLIMIT_NOFILE, &rl);

    size_t free_b = 0, total_b = 0;
    OK(hipMemGetInfo(&free_b, &total_b));

    hipMemAllocationProp prop{};
    prop.type = hipMemAllocationTypePinned;
    prop.location.type = hipMemLocationTypeDevice;
    prop.location.id = dev;
    hipMemAccessDesc desc{};
    desc.location.type = hipMemLocationTypeDevice;
    desc.location.id = dev;
    desc.flags = hipMemAccessFlagsProtReadWrite;

    size_t gran = 0;
    OK(hipMemGetAllocationGranularity(&gran, &prop, hipMemAllocationGranularityRecommended));

    // Reserve VA for everything that could possibly fit, so VA is never the limit.
    const size_t want = free_b / gran;
    void* base = nullptr;
    OK(hipMemAddressReserve(&base, want * gran, gran, nullptr, 0));

    printf("%s (%s)\n", prop_dev.name, prop_dev.gcnArchName);
    printf("  RLIMIT_NOFILE soft=%llu hard=%llu\n", (unsigned long long)rl.rlim_cur,
           (unsigned long long)rl.rlim_max);
    printf("  VRAM free %.2f GiB / total %.2f GiB, granularity %zu B\n",
           free_b / 1073741824.0, total_b / 1073741824.0, gran);
    printf("  VA reserved for %zu handles (%.2f GiB)\n\n", want, want * gran / 1073741824.0);

    std::vector<hipMemGenericAllocationHandle_t> handles;
    handles.reserve(want);
    size_t mapped = 0;
    hipError_t last = hipSuccess;

    for (size_t i = 0; i < want; ++i) {
        hipMemGenericAllocationHandle_t h;
        last = hipMemCreate(&h, gran, &prop, 0);
        if (last != hipSuccess) break;
        void* at = static_cast<char*>(base) + i * gran;
        if (hipMemMap(at, gran, 0, h, 0) != hipSuccess) { hipMemRelease(h); break; }
        if (hipMemSetAccess(at, gran, &desc, 1) != hipSuccess) {
            hipMemUnmap(at, gran);
            hipMemRelease(h);
            break;
        }
        handles.push_back(h);
        ++mapped;
    }

    size_t free_after = 0, total_after = 0;
    hipMemGetInfo(&free_after, &total_after);

    printf("  mapped %zu handles = %.2f GiB before failure\n", mapped,
           mapped * gran / 1073741824.0);
    printf("  first failure: %s\n", last == hipSuccess ? "none (VA exhausted)"
                                                       : hipGetErrorString(last));
    printf("  VRAM still free at failure: %.2f GiB\n", free_after / 1073741824.0);
    printf("\n  => %s\n",
           (free_after > 2ull * 1073741824ull && mapped < want)
               ? "FAILED WITH VRAM TO SPARE — descriptor-bound, not memory-bound"
               : "memory-bound (raise ulimit -n and re-run to compare)");

    for (size_t i = 0; i < mapped; ++i)
        hipMemUnmap(static_cast<char*>(base) + i * gran, gran);
    hipMemAddressFree(base, want * gran);
    for (auto& h : handles) hipMemRelease(h);
    return 0;
}
