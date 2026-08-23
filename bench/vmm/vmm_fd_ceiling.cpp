// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
//
// Minimal reproducer: HIP VMM capacity is bounded by RLIMIT_NOFILE, not VRAM.
//
// Each hipMemCreate handle appears to consume one file descriptor, so with the
// common default of `ulimit -n 1024` a process can map only ~2 GiB of VMM
// regardless of how much VRAM is free. Raising the limit removes the ceiling.
//
// Note on address-space sizing: the reservation below is sized to initial free
// VRAM (want = free / granularity) so it is a preselected span, not an
// exhaustive scan of VA. Completing `want` handles therefore proves only that
// the preselected span was filled, not that address space was exhausted.
//
// Build:
//   hipcc --offload-arch=gfx1201 -O2 vmm_fd_ceiling.cpp -o vmm_fd_ceiling
// Run (both arms):
//   ./vmm_fd_ceiling
//   bash -c 'ulimit -n 65536; ./vmm_fd_ceiling'
//   ./vmm_fd_ceiling --raise   # raise soft limit to hard limit in-process
//
// Measured on Radeon AI PRO R9700 (gfx1201, 32 GiB), ROCm 7.14.0, 2 MiB
// granularity:
//   ulimit -n 1024   -> 1015 handles  =  1.98 GiB   (31.79 GiB free)
//   ulimit -n 65536  -> 9765 handles  = 19.07 GiB   (31.79 GiB free)
//   ulimit -n 262144 -> 15786 handles = 30.83 GiB   (31.79 GiB free)
#include <hip/hip_runtime.h>
#include <sys/resource.h>
#include <dirent.h>
#include <cerrno>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <vector>

#define OK(x)                                                                      \
    do {                                                                           \
        hipError_t _e = (x);                                                       \
        if (_e != hipSuccess) {                                                    \
            printf("fatal: %s -> %s\n", #x, hipGetErrorString(_e));                \
            return 2;                                                              \
        }                                                                          \
    } while (0)

static int count_open_fds() {
    DIR* d = opendir("/proc/self/fd");
    if (!d) return -1;
    int n = 0;
    struct dirent* e;
    while ((e = readdir(d)) != nullptr) {
        if (e->d_name[0] == '.') continue;
        ++n;
    }
    closedir(d);
    return n;
}

int main(int argc, char** argv) {
    // Parse --raise anywhere; first numeric positional arg is device id.
    bool do_raise = false;
    int dev = 0;
    bool dev_set = false;
    for (int i = 1; i < argc; ++i) {
        if (strcmp(argv[i], "--raise") == 0) {
            do_raise = true;
        } else if (!dev_set) {
            char* end = nullptr;
            long v = strtol(argv[i], &end, 10);
            if (end != argv[i] && *end == '\0') {
                dev = (int)v;
                dev_set = true;
            }
        }
    }
    OK(hipSetDevice(dev));

    hipDeviceProp_t prop_dev;
    OK(hipGetDeviceProperties(&prop_dev, dev));

    rlimit rl_before{};
    if (getrlimit(RLIMIT_NOFILE, &rl_before) != 0) {
        printf("getrlimit(RLIMIT_NOFILE) before raise failed: %s (errno %d)\n",
               strerror(errno), errno);
    } else {
        printf("RLIMIT_NOFILE before: soft=%llu hard=%llu\n",
               (unsigned long long)rl_before.rlim_cur,
               (unsigned long long)rl_before.rlim_max);
    }

    if (do_raise) {
        rlimit rl_try = rl_before;
        rl_try.rlim_cur = rl_try.rlim_max;
        if (setrlimit(RLIMIT_NOFILE, &rl_try) != 0) {
            printf("setrlimit(RLIMIT_NOFILE, soft=hard=%llu) failed: %s (errno %d)\n",
                   (unsigned long long)rl_try.rlim_cur, strerror(errno), errno);
        } else {
            printf("setrlimit(RLIMIT_NOFILE, soft=hard=%llu) succeeded\n",
                   (unsigned long long)rl_try.rlim_cur);
        }
    }

    rlimit rl{};
    if (getrlimit(RLIMIT_NOFILE, &rl) != 0) {
        printf("getrlimit(RLIMIT_NOFILE) after failed: %s (errno %d)\n",
               strerror(errno), errno);
        rl = rl_before;
    } else {
        printf("RLIMIT_NOFILE after: soft=%llu hard=%llu\n",
               (unsigned long long)rl.rlim_cur, (unsigned long long)rl.rlim_max);
    }

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

    // Reserve VA for everything that could possibly fit, so VA is never the limit
    // for the requested span. Completing the span proves only that this
    // preselected size was filled (see header note).
    const size_t want = free_b / gran;
    void* base = nullptr;
    OK(hipMemAddressReserve(&base, want * gran, gran, nullptr, 0));

    printf("%s (%s)\n", prop_dev.name, prop_dev.gcnArchName);
    printf("  VRAM free %.2f GiB / total %.2f GiB, granularity %zu B\n",
           free_b / 1073741824.0, total_b / 1073741824.0, gran);
    printf("  VA reserved for %zu handles (%.2f GiB)\n\n", want, want * gran / 1073741824.0);

    std::vector<hipMemGenericAllocationHandle_t> handles;
    handles.reserve(want);
    size_t mapped = 0;

    const char* stop_stage = "completed-requested-count";
    hipError_t stop_err = hipSuccess;

    for (size_t i = 0; i < want; ++i) {
        hipMemGenericAllocationHandle_t h;
        hipError_t e = hipMemCreate(&h, gran, &prop, 0);
        if (e != hipSuccess) {
            stop_stage = "hipMemCreate";
            stop_err = e;
            break;
        }
        void* at = static_cast<char*>(base) + i * gran;
        e = hipMemMap(at, gran, 0, h, 0);
        if (e != hipSuccess) {
            hipMemRelease(h);
            stop_stage = "hipMemMap";
            stop_err = e;
            break;
        }
        e = hipMemSetAccess(at, gran, &desc, 1);
        if (e != hipSuccess) {
            hipMemUnmap(at, gran);
            hipMemRelease(h);
            stop_stage = "hipMemSetAccess";
            stop_err = e;
            break;
        }
        handles.push_back(h);
        ++mapped;
    }

    size_t free_after = 0, total_after = 0;
    hipMemGetInfo(&free_after, &total_after);
    int open_fds = count_open_fds();
    long headroom = -1;
    if (open_fds >= 0) headroom = (long)rl.rlim_cur - (long)open_fds;

    // If we filled the preselected span without hitting any HIP error, keep
    // stop_stage as completed-requested-count; stop_err remains hipSuccess.

    printf("  handles created %zu = %.2f GiB\n", mapped, mapped * gran / 1073741824.0);
    if (strcmp(stop_stage, "completed-requested-count") == 0) {
        printf("  stopping stage: %s\n", stop_stage);
        printf("  note: completed requested count (reservation was sized to initial free memory, so this is not an address-space limit)\n");
    } else {
        printf("  stopping stage: %s\n", stop_stage);
        printf("  stopping error: %s (%d)\n", hipGetErrorString(stop_err), (int)stop_err);
    }
    printf("  VRAM still free at stop: %.2f GiB\n", free_after / 1073741824.0);
    if (open_fds >= 0) {
        printf("  FDs at stop: open %d / soft %llu (headroom %ld)\n",
               open_fds, (unsigned long long)rl.rlim_cur, headroom);
    } else {
        printf("  FDs at stop: open (unknown, opendir failed: %s) / soft %llu\n",
               strerror(errno), (unsigned long long)rl.rlim_cur);
    }

    // Verdict is descriptor-bound only when we actually stopped inside
    // hipMemCreate with VRAM still free. Other stopping stages have different
    // explanations and must not claim a descriptor ceiling.
    const bool failed_with_vram_to_spare =
        (strcmp(stop_stage, "hipMemCreate") == 0 && free_after > 2ull * 1073741824ull);
    printf("\n  => %s\n",
           failed_with_vram_to_spare
               ? "FAILED WITH VRAM TO SPARE — descriptor-bound, not memory-bound"
               : "memory-bound or completed requested count (raise ulimit -n and re-run to compare)");

    for (size_t i = 0; i < mapped; ++i)
        hipMemUnmap(static_cast<char*>(base) + i * gran, gran);
    hipMemAddressFree(base, want * gran);
    for (auto& h : handles) hipMemRelease(h);
    return 0;
}
