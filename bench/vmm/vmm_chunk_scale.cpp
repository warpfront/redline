// Does a larger per-handle chunk sidestep the descriptor ceiling?
//
// The ceiling is (descriptor budget) x (bytes per handle). If each hipMemCreate
// still costs exactly one dmabuf FD regardless of size, then allocating fewer,
// larger physical chunks raises the reachable total with NO change to
// RLIMIT_NOFILE -- which matters because a container may not let you raise it.
#include <hip/hip_runtime.h>
#include <sys/resource.h>
#include <cstdio>
#include <cstdlib>
#include <vector>

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
    printf("%-12s %10s %12s %14s %s\n", "chunk", "handles", "reachable", "vs 2MiB", "stopped by");

    const size_t MiB = 1ull << 20;
    size_t chunks[] = {2 * MiB, 8 * MiB, 32 * MiB, 128 * MiB, 512 * MiB};
    double base = 0;

    for (size_t c : chunks) {
        std::vector<hipMemGenericAllocationHandle_t> hs;
        hipError_t last = hipSuccess;
        // Cap so we never try to exceed free VRAM; we want the descriptor limit
        // to be what bites, or to report honestly that memory was the limit.
        size_t avail = free_b;
        if (budget_gib > 0) {
            size_t b = (size_t)(budget_gib * 1073741824.0);
            if (b < avail) avail = b;
        }
        size_t cap = avail / c;
        for (size_t i = 0; i < cap; ++i) {
            hipMemGenericAllocationHandle_t h;
            last = hipMemCreate(&h, c, &prop, 0);
            if (last != hipSuccess) break;
            hs.push_back(h);
        }
        double gib = (double)hs.size() * (double)c / 1073741824.0;
        if (base == 0) base = gib;
        const char* why = (hs.size() >= cap) ? "budget/VRAM" : "descriptors";
        printf("%-12s %10zu %9.2f GiB %13.1fx %s\n",
               c == 2 * MiB ? "2 MiB" : c == 8 * MiB ? "8 MiB" : c == 32 * MiB ? "32 MiB"
                   : c == 128 * MiB ? "128 MiB" : "512 MiB",
               hs.size(), gib, gib / base, why);
        for (auto& h : hs) hipMemRelease(h);
    }
    printf("\nEach handle costs one dmabuf FD regardless of its size, so reachable\n"
           "bytes scale with chunk size at a fixed descriptor budget.\n");
    return 0;
}
