// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
//
// Which ROCm runtime is this process actually using?
//
// Why this exists:
//   A side-by-side ROCm install (for example /opt/rocm/core-7.14 next to
//   /opt/rocm/core-10.0) makes an A/B of two runtimes possible, but only if the
//   process really loads the one intended. That is easy to get wrong: hipcc
//   emits no RPATH, and /opt/rocm/core is a symlink to one specific version, so
//   a binary built against one toolchain will silently load whichever runtime
//   the default search path resolves first. An A/B done that way compares
//   compilers while appearing to compare runtimes, and the giveaway is results
//   that agree suspiciously exactly.
//
//   ldd is not sufficient evidence either: it reports what the loader would
//   resolve for the listed sonames, not what the process ended up using for
//   every transitive ROCm component. So this probe asks the loaded libraries
//   themselves, at runtime:
//     * hipRuntimeGetVersion / hipDriverGetVersion, from the loaded HIP runtime
//     * the on-disk path of the object that actually provided a HIP symbol,
//       via dladdr, which cannot be faked by environment settings
//     * the ROCr version and build id, from the loaded HSA runtime
//     * the same for every ROCm shared object mapped into the process
//
//   Run this immediately before and after any runtime A/B. If the reported
//   paths do not differ between arms, the arms are not different.
//
// Build: hipcc -O0 rocm_ident.cpp -o rocm_ident -ldl -lhsa-runtime64 \
//            -I$ROCM/include -L$ROCM/lib
// Run:   LD_LIBRARY_PATH=/opt/rocm/core-10.0/lib ./rocm_ident

#define _GNU_SOURCE
#include <hip/hip_runtime.h>
#include <hsa/hsa.h>
#include <dlfcn.h>
#include <cstdio>
#include <cstring>
#include <string>
#include <vector>

// Resolve the on-disk object that provided a given symbol address.
static std::string provider_of(void* sym) {
    Dl_info info{};
    if (sym && dladdr(sym, &info) && info.dli_fname) return info.dli_fname;
    return "(unresolved)";
}

int main() {
    // Force the HIP runtime to initialise before we interrogate it.
    int ndev = 0;
    hipError_t e = hipGetDeviceCount(&ndev);

    int rt_ver = -1, drv_ver = -1;
    hipRuntimeGetVersion(&rt_ver);
    hipDriverGetVersion(&drv_ver);

    printf("=== loaded ROCm identity ===\n");
    printf("hipGetDeviceCount      : %s (%d device%s)\n",
           e == hipSuccess ? "ok" : hipGetErrorString(e), ndev,
           ndev == 1 ? "" : "s");
    printf("hipRuntimeGetVersion   : %d\n", rt_ver);
    printf("hipDriverGetVersion    : %d\n", drv_ver);

    // dladdr on a real HIP entry point names the object actually in use. This is
    // the load-bearing line: it is observed from inside the process rather than
    // predicted by the loader.
    printf("libamdhip64 in use     : %s\n",
           provider_of((void*)&hipRuntimeGetVersion).c_str());
    printf("libhsa-runtime in use  : %s\n",
           provider_of((void*)&hsa_init).c_str());

    // ROCr's own version, straight from the loaded HSA runtime.
    if (hsa_init() == HSA_STATUS_SUCCESS) {
        uint16_t major = 0, minor = 0;
        hsa_system_get_info(HSA_SYSTEM_INFO_VERSION_MAJOR, &major);
        hsa_system_get_info(HSA_SYSTEM_INFO_VERSION_MINOR, &minor);
        printf("HSA runtime version    : %u.%u\n", major, minor);
        hsa_shut_down();
    } else {
        printf("HSA runtime version    : (hsa_init failed)\n");
    }

    // Every ROCm object mapped into this process, so a mixed load is visible
    // rather than inferred. /proc/self/maps is the ground truth for what the
    // kernel actually mapped.
    printf("\n=== every ROCm object mapped into this process ===\n");
    FILE* f = fopen("/proc/self/maps", "r");
    if (!f) {
        printf("(cannot read /proc/self/maps)\n");
        return 0;
    }
    std::vector<std::string> seen;
    char line[4096];
    while (fgets(line, sizeof(line), f)) {
        const char* slash = strchr(line, '/');
        if (!slash) continue;
        std::string path(slash);
        while (!path.empty() && (path.back() == '\n' || path.back() == ' '))
            path.pop_back();
        // Only ROCm-tree objects are interesting here.
        if (path.find("/opt/rocm") == std::string::npos &&
            path.find("rocm") == std::string::npos)
            continue;
        bool dup = false;
        for (const auto& s : seen)
            if (s == path) { dup = true; break; }
        if (dup) continue;
        seen.push_back(path);
    }
    fclose(f);

    // Report which version tree each object came from, so a cross-tree load is
    // impossible to miss.
    int from_714 = 0, from_100 = 0, from_other = 0;
    for (const auto& p : seen) {
        const char* tag = "other";
        if (p.find("core-10.0") != std::string::npos) { tag = "10.0"; ++from_100; }
        else if (p.find("core-7.14") != std::string::npos) { tag = "7.14"; ++from_714; }
        else ++from_other;
        printf("  [%-5s] %s\n", tag, p.c_str());
    }
    printf("\nsummary: %d object(s) from core-7.14, %d from core-10.0, %d elsewhere\n",
           from_714, from_100, from_other);
    if (from_714 && from_100)
        printf("WARNING: objects from BOTH trees are mapped -- this process is a\n"
               "mixed load and is not a valid single-version measurement.\n");
    return 0;
}
