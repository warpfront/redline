// Does a larger per-handle chunk sidestep the descriptor ceiling?
//
// The ceiling is (available descriptor slots) x (bytes per physical handle). If
// each hipMemCreate still costs exactly one dmabuf FD regardless of size, then
// allocating fewer, larger physical chunks raises the total successfully created
// bytes with NO change to RLIMIT_NOFILE -- which matters because a container may
// not let you raise it.
//
// This probe only calls hipMemCreate to count successfully created physical
// handles; it never reserves, maps, sets access, or touches memory. The
// reported GiB is therefore created-handle bytes, not mapped-and-accessible
// memory.
//
#include <hip/hip_runtime.h>
#include <sys/resource.h>
#include <dirent.h>
#include <cerrno>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <vector>

// Returns the open-fd count, or a negative sentinel:
//   -2 = opendir itself failed with EMFILE, which is direct positive evidence
//        that this process has exhausted its descriptors
//   -1 = opendir failed for some other reason (genuinely unknown)
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

int main(int argc, char** argv) {
    if (hipSetDevice(0) != hipSuccess) { printf("no device\n"); return 1; }
    // Optional total-byte budget (GiB) so this is safe to run on a shared APU
    // whose carveout is far larger than we need to demonstrate the scaling.
    const double budget_gib = argc > 1 ? atof(argv[1]) : 0.0;
    hipDeviceProp_t p; hipGetDeviceProperties(&p, 0);
    rlimit rl{}; getrlimit(RLIMIT_NOFILE, &rl);
    size_t free_b = 0, total_b = 0; hipMemGetInfo(&free_b, &total_b);

    hipMemAllocationProp prop{};
    prop.type = hipMemAllocationTypePinned;
    prop.location.type = hipMemLocationTypeDevice;
    prop.location.id = 0;

    printf("=== %s (%s) ===\n", p.name, p.gcnArchName);
    printf("soft RLIMIT_NOFILE %llu | VRAM free %.2f GiB\n\n",
           (unsigned long long)rl.rlim_cur, free_b / 1073741824.0);
    printf("%-12s %10s %12s %14s %-32s %s\n", "chunk", "handles", "created", "vs 2MiB", "attribution", "detail");
    printf("%-12s %10s %12s %14s %-32s %s\n", "-----", "-------", "-------", "-------", "-----------", "------");

    const size_t MiB = 1ull << 20;
    size_t chunks[] = {2 * MiB, 8 * MiB, 32 * MiB, 128 * MiB, 512 * MiB};
    double base = 0;

    for (size_t c : chunks) {
        std::vector<hipMemGenericAllocationHandle_t> hs;
        hipError_t last = hipSuccess;
        size_t avail = free_b;
        bool budget_active = false;
        size_t budget_bytes = 0;
        if (budget_gib > 0) {
            budget_bytes = (size_t)(budget_gib * 1073741824.0);
            if (budget_bytes < avail) {
                avail = budget_bytes;
                budget_active = true;
            }
        }
        size_t cap = (c == 0) ? 0 : avail / c;
        // Attempt beyond the computed cap so a genuine device limit is observed
        // rather than inferred. When a user-supplied budget caps the run we
        // honour it as a safety limit and do not probe beyond it.
        size_t attempt = budget_active ? cap : cap + 64;
        for (size_t i = 0; i < attempt; ++i) {
            hipMemGenericAllocationHandle_t h;
            last = hipMemCreate(&h, c, &prop, 0);
            if (last != hipSuccess) break;
            hs.push_back(h);
        }
        double gib = (double)hs.size() * (double)c / 1073741824.0;
        if (base == 0) base = gib;

        // Sample FD headroom and free VRAM at stop for honest attribution.
        int open_fds = count_open_fds();
        rlimit rl_now{}; getrlimit(RLIMIT_NOFILE, &rl_now);
        long headroom = open_fds >= 0 ? (long)rl_now.rlim_cur - (long)open_fds : -1;
        size_t free_after = 0, total_after = 0;
        hipMemGetInfo(&free_after, &total_after);
        const char* err_str = hipGetErrorString(last);

        const char* why = nullptr;
        if (last != hipSuccess) {
            // We observed a real failure; attribute only from measured state.
            // open_fds == -2 means opendir("/proc/self/fd") itself hit EMFILE,
            // which is direct evidence that descriptors, not VRAM, ran out.
            const bool fd_exhausted = (open_fds == -2) || (headroom >= 0 && headroom <= 16);
            const bool mem_nearly_gone = free_after < 512ull * 1048576ull;
            if (fd_exhausted && !mem_nearly_gone) {
                why = "descriptor-bound";
            } else if (mem_nearly_gone && !fd_exhausted) {
                why = "memory-bound";
            } else if (fd_exhausted && mem_nearly_gone) {
                why = "coincident (both exhausted)";
            } else {
                why = "indeterminate";
            }
        } else {
            // No HIP error observed.
            if (budget_active && hs.size() >= cap) {
                why = "budget-cap (not a device limit)";
            } else if (hs.size() >= attempt) {
                why = "indeterminate (no failure observed)";
            } else {
                // Should be unreachable (no error but stopped early), treat as indeterminate.
                why = "indeterminate";
            }
        }

        const char* chunk_label = c == 2 * MiB ? "2 MiB" : c == 8 * MiB ? "8 MiB" : c == 32 * MiB ? "32 MiB"
                   : c == 128 * MiB ? "128 MiB" : "512 MiB";

        char detail[256];
        if (last != hipSuccess) {
            char fd_part[80];
            if (open_fds >= 0)
                snprintf(fd_part, sizeof(fd_part), "FDs %d/%llu headroom %ld",
                         open_fds, (unsigned long long)rl_now.rlim_cur, headroom);
            else if (open_fds == -2)
                snprintf(fd_part, sizeof(fd_part), "FDs exhausted (opendir /proc/self/fd: EMFILE, soft %llu)",
                         (unsigned long long)rl_now.rlim_cur);
            else
                snprintf(fd_part, sizeof(fd_part), "FDs unknown (opendir failed)");
            snprintf(detail, sizeof(detail), "%s | %s | free %.1f MiB",
                     err_str, fd_part, free_after / 1048576.0);
        } else {
            char fd_part[64];
            if (open_fds >= 0)
                snprintf(fd_part, sizeof(fd_part), "FDs %d/%llu headroom %ld",
                         open_fds, (unsigned long long)rl_now.rlim_cur, headroom);
            else
                snprintf(fd_part, sizeof(fd_part), "FDs unknown");
            if (budget_active)
                snprintf(detail, sizeof(detail), "no error (budget %.1f GiB) | %s | free %.1f MiB",
                         budget_gib, fd_part, free_after / 1048576.0);
            else
                snprintf(detail, sizeof(detail), "no error | %s | free %.1f MiB",
                         fd_part, free_after / 1048576.0);
        }

        printf("%-12s %10zu %9.2f GiB %13.1fx %-32s %s\n",
               chunk_label, hs.size(), gib, base > 0 ? gib / base : 0.0, why, detail);
        for (auto& h : hs) hipMemRelease(h);
    }
    printf("\nEach handle costs one dmabuf FD regardless of its size, so created\n"
           "bytes scale with chunk size at a fixed descriptor budget. This probe\n"
           "measures created handles only; it does not map or access memory.\n");
    return 0;
}
