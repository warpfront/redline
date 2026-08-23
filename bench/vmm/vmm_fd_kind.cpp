// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
//
// Proves *mechanically* what vmm_fd_ceiling.cpp only inferred from a numeric
// coincidence: that hipMemCreate consumes one file descriptor per VMM handle,
// and identifies what kind of descriptor it is.
//
// Counts /proc/self/fd across each phase and reports the delta plus the
// readlink target of the descriptors that appeared, so the issue report can say
// exactly which resource is exhausted rather than guessing from the fact that
// the ceiling happens to sit near 1024.
//
// Build:  hipcc --offload-arch=gfx1201 -O2 vmm_fd_kind.cpp -o vmm_fd_kind
// Run:    ./vmm_fd_kind [handles]
#include <hip/hip_runtime.h>
#include <dirent.h>
#include <unistd.h>
#include <cstdio>
#include <cstring>
#include <map>
#include <string>
#include <vector>

static std::map<std::string, int> fd_targets() {
    std::map<std::string, int> out;
    DIR* d = opendir("/proc/self/fd");
    if (!d) return out;
    while (dirent* e = readdir(d)) {
        if (e->d_name[0] == '.') continue;
        char path[256], target[512];
        snprintf(path, sizeof path, "/proc/self/fd/%s", e->d_name);
        ssize_t n = readlink(path, target, sizeof target - 1);
        if (n <= 0) continue;
        target[n] = '\0';
        // Keep the readlink target verbatim. Which *kind* of descriptor this is
        // decides which component owns the behaviour, so bucketing it away would
        // discard the most important detail.
        out[std::string(target)]++;
    }
    closedir(d);
    return out;
}

static int total(const std::map<std::string, int>& m) {
    int n = 0;
    for (auto& kv : m) n += kv.second;
    return n;
}

static void report_delta(const char* label, const std::map<std::string, int>& before,
                         const std::map<std::string, int>& after) {
    printf("  %-28s total %4d -> %4d (delta %+d)\n", label, total(before), total(after),
           total(after) - total(before));
    for (auto& kv : after) {
        auto it = before.find(kv.first);
        int was = it == before.end() ? 0 : it->second;
        if (kv.second != was)
            printf("      %+5d  %s\n", kv.second - was, kv.first.c_str());
    }
}

int main(int argc, char** argv) {
    const int want = argc > 1 ? atoi(argv[1]) : 64;
    hipDeviceProp_t p;
    if (hipSetDevice(0) != hipSuccess || hipGetDeviceProperties(&p, 0) != hipSuccess) {
        printf("no usable device\n");
        return 1;
    }
    printf("=== %s (%s): descriptor accounting for %d VMM handles ===\n", p.name, p.gcnArchName,
           want);

    auto base = fd_targets();
    printf("  at startup: %d descriptors open\n", total(base));

    hipMemAllocationProp prop{};
    prop.type = hipMemAllocationTypePinned;
    prop.location.type = hipMemLocationTypeDevice;
    prop.location.id = 0;
    size_t gran = 0;
    hipMemGetAllocationGranularity(&gran, &prop, hipMemAllocationGranularityRecommended);

    // Phase 1: a plain hipMalloc, as a control. If VMM is special, this should
    // not scale descriptors with allocation count.
    std::vector<void*> plain;
    auto pre_plain = fd_targets();
    for (int i = 0; i < want; ++i) {
        void* q = nullptr;
        if (hipMalloc(&q, gran) != hipSuccess) break;
        plain.push_back(q);
    }
    auto post_plain = fd_targets();
    printf("\nphase 1: %zu x hipMalloc(%zu)\n", plain.size(), gran);
    report_delta("after hipMalloc", pre_plain, post_plain);
    for (void* q : plain) hipFree(q);

    // Phase 2: the same count via hipMemCreate only (no map, no SetAccess), to
    // isolate handle creation from mapping.
    auto pre_create = fd_targets();
    std::vector<hipMemGenericAllocationHandle_t> handles;
    for (int i = 0; i < want; ++i) {
        hipMemGenericAllocationHandle_t h;
        if (hipMemCreate(&h, gran, &prop, 0) != hipSuccess) break;
        handles.push_back(h);
    }
    auto post_create = fd_targets();
    printf("\nphase 2: %zu x hipMemCreate(%zu)  <-- the phase under test\n", handles.size(), gran);
    report_delta("after hipMemCreate", pre_create, post_create);

    // Phase 3: release them and confirm the descriptors come back.
    for (auto& h : handles) hipMemRelease(h);
    auto post_release = fd_targets();
    printf("\nphase 3: released %zu handles\n", handles.size());
    report_delta("after hipMemRelease", post_create, post_release);

    int per_handle_num = total(post_create) - total(pre_create);
    printf("\n=== descriptors per hipMemCreate handle: %.3f ===\n",
           handles.empty() ? 0.0 : (double)per_handle_num / (double)handles.size());
    printf("=== hipMalloc control: %.3f per allocation ===\n",
           plain.empty() ? 0.0
                         : (double)(total(post_plain) - total(pre_plain)) / (double)plain.size());
    return 0;
}
