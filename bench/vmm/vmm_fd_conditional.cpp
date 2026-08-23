// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
//
// Does the per-handle dmabuf descriptor depend on the caller asking for an
// exportable handle?
//
// Why the probe is shaped this way:
//   hipMemCreate takes a hipMemAllocationProp whose requestedHandleTypes field
//   declares whether the caller intends to export the handle for sharing. A
//   caller that never shares memory has no reason to need an OS-visible
//   descriptor. This probe creates the same number of handles twice -- once with
//   requestedHandleTypes left at hipMemHandleTypeNone, once with it set to
//   hipMemHandleTypePosixFileDescriptor -- and measures the open-descriptor
//   delta for each arm.
//
//   Interpretation is deliberately narrow. If both arms cost the same number of
//   descriptors, then the descriptor is being spent regardless of the caller's
//   declared intent, and that is a cost a non-sharing caller cannot opt out of
//   through the public API. If the None arm costs zero, the ceiling is
//   self-inflicted by whatever requested export. This probe does NOT show where
//   in the runtime the descriptor is created, and does not claim it could be
//   made lazy -- only whether the public API's declared intent changes the cost.
//
// Build: hipcc --offload-arch=gfx1201 -O2 vmm_fd_conditional.cpp -o vmm_fd_conditional
// Run:   ./vmm_fd_conditional [handles_per_arm]

#include <hip/hip_runtime.h>
#include <dirent.h>
#include <cerrno>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <unistd.h>
#include <vector>

static int count_open_fds() {
    DIR* d = opendir("/proc/self/fd");
    if (!d) return errno == EMFILE ? -2 : -1;
    int n = 0;
    struct dirent* e;
    while ((e = readdir(d)) != nullptr) {
        if (e->d_name[0] == '.') continue;
        ++n;
    }
    closedir(d);
    return n;
}

// Count how many of this process's open descriptors are dmabufs.
static int count_dmabuf_fds() {
    DIR* d = opendir("/proc/self/fd");
    if (!d) return -1;
    int n = 0;
    struct dirent* e;
    char path[64], target[256];
    while ((e = readdir(d)) != nullptr) {
        if (e->d_name[0] == '.') continue;
        snprintf(path, sizeof(path), "/proc/self/fd/%s", e->d_name);
        ssize_t k = readlink(path, target, sizeof(target) - 1);
        if (k <= 0) continue;
        target[k] = '\0';
        if (strstr(target, "dmabuf")) ++n;
    }
    closedir(d);
    return n;
}

struct ArmResult {
    int fd_delta = 0;
    int dmabuf_delta = 0;
    size_t created = 0;
    hipError_t err = hipSuccess;
};

static ArmResult run_arm(size_t count, size_t chunk, unsigned long long handle_type,
                         const char* label) {
    ArmResult r;
    const int fd_before = count_open_fds();
    const int db_before = count_dmabuf_fds();

    hipMemAllocationProp prop{};
    prop.type = hipMemAllocationTypePinned;
    prop.location.type = hipMemLocationTypeDevice;
    prop.location.id = 0;
    prop.requestedHandleTypes = (hipMemAllocationHandleType)handle_type;

    std::vector<hipMemGenericAllocationHandle_t> hs;
    hs.reserve(count);
    for (size_t i = 0; i < count; ++i) {
        hipMemGenericAllocationHandle_t h;
        hipError_t e = hipMemCreate(&h, chunk, &prop, 0);
        if (e != hipSuccess) { r.err = e; break; }
        hs.push_back(h);
    }
    r.created = hs.size();

    const int fd_after = count_open_fds();
    const int db_after = count_dmabuf_fds();
    r.fd_delta = (fd_after >= 0 && fd_before >= 0) ? fd_after - fd_before : -1;
    r.dmabuf_delta = (db_after >= 0 && db_before >= 0) ? db_after - db_before : -1;

    printf("%-34s created %3zu | fd delta %+4d | dmabuf delta %+4d | %s\n",
           label, r.created, r.fd_delta, r.dmabuf_delta,
           r.err == hipSuccess ? "no error" : hipGetErrorString(r.err));

    for (auto& h : hs) hipMemRelease(h);

    const int fd_released = count_open_fds();
    if (fd_released >= 0 && fd_before >= 0)
        printf("%-34s after release, fd delta vs start %+d\n", "", fd_released - fd_before);
    return r;
}

int main(int argc, char** argv) {
    const size_t n = argc > 1 ? (size_t)atoll(argv[1]) : 64;
    const size_t chunk = 2ull << 20;
    if (hipSetDevice(0) != hipSuccess) { printf("no device\n"); return 1; }

    hipDeviceProp_t p;
    hipGetDeviceProperties(&p, 0);
    printf("=== %s (%s) === %zu handles per arm, %zu MiB each\n\n",
           p.name, p.gcnArchName, n, chunk >> 20);

    // Arm A: caller declares no intent to share. hipMemHandleTypeNone == 0.
    ArmResult none = run_arm(n, chunk, 0, "requestedHandleTypes=None");
    printf("\n");
    // Arm B: caller explicitly asks for an exportable POSIX fd.
    ArmResult posix = run_arm(n, chunk, (unsigned long long)hipMemHandleTypePosixFileDescriptor,
                              "requestedHandleTypes=PosixFd");

    printf("\n--- reading ---\n");
    if (none.fd_delta == (int)none.created && posix.fd_delta == (int)posix.created) {
        printf("Both arms spend one descriptor per handle. Declaring no intent to\n"
               "export does not avoid the cost through the public API.\n");
    } else if (none.fd_delta == 0) {
        printf("The None arm spends no descriptors; the ceiling is only reached by\n"
               "callers that requested an exportable handle type.\n");
    } else {
        printf("Arms differ but neither matches its handle count exactly:\n"
               "  None  delta %+d over %zu handles\n"
               "  PosixFd delta %+d over %zu handles\n"
               "Reported without attribution.\n",
               none.fd_delta, none.created, posix.fd_delta, posix.created);
    }
    return 0;
}
