// Pure-HIP model of ROCm/ROCm#6603: a LIVE VMM mapping that reads back as zeros.
//
// The upstream report needs PyTorch with BOTH max_split_size_mb AND
// expandable_segments:True, and dies around cycle ~115-144 on ROCm 7.14 while
// staying clean on 7.2. This removes PyTorch entirely and models the mechanism
// the allocator actually exercises:
//   * an expandable segment grown by mapping handles into one reserved VA range
//   * a canary written ONCE into an early mapped chunk, never rewritten
//   * churn that maps/unmaps/releases OTHER handles around the live canary
//   * VA reuse: after unmap, a DIFFERENT handle is mapped at the same offset
//   * near-exhaustion retry (hipMemCreate fails -> release cached -> retry),
//     (the original report tied corruption to the first num_alloc_retries; the
//     reporter WITHDREW that on 2026-08-20 and now records failure at cycle
//     113-114 with num_alloc_retries still 0, so the retry path is modelled here
//     for completeness, not because it is the trigger)
// After each cycle the canary is re-read. If a live mapping ever reads back
// zeros (or anything but the pattern) we have reproduced #6603 without a
// framework in the loop.
//
// Bounded on purpose: caps its own footprint and cycle count so it can run on a
// shared host without taking the machine down.
#include <hip/hip_runtime.h>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <vector>

#define OK(x) do { hipError_t _e = (x); if (_e != hipSuccess) { \
    printf("  [fatal] %s -> %s\n", #x, hipGetErrorString(_e)); return 2; } } while (0)

static const unsigned PATTERN = 0x5A5A5A5Au;

struct Chunk {
    hipMemGenericAllocationHandle_t h{};
    size_t off = 0;
    bool mapped = false;
};

int main(int argc, char** argv) {
    int dev        = argc > 1 ? atoi(argv[1]) : 0;
    int cycles     = argc > 2 ? atoi(argv[2]) : 200;
    double fill    = argc > 3 ? atof(argv[3]) : 0.80;   // fraction of free VRAM to chase
    OK(hipSetDevice(dev));

    hipDeviceProp_t p;
    OK(hipGetDeviceProperties(&p, dev));
    size_t freeB = 0, totalB = 0;
    OK(hipMemGetInfo(&freeB, &totalB));

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

    // Model an expandable segment: one big VA reservation, handles mapped into it.
    const size_t CHUNK = gran;                                   // 2 MiB on RDNA
    size_t budget = (size_t)(freeB * fill);
    size_t max_chunks = budget / CHUNK;
    if (max_chunks < 16) { printf("not enough free VRAM\n"); return 1; }
    size_t reserve = max_chunks * CHUNK;

    printf("=== %s (%s) dev %d ===\n", p.name, p.gcnArchName, dev);
    printf("VRAM free %.2f GiB / total %.2f GiB | granularity %zu | chunks %zu | reserve %.2f GiB\n",
           freeB / 1073741824.0, totalB / 1073741824.0, CHUNK, max_chunks, reserve / 1073741824.0);
    printf("cycles %d\n\n", cycles);

    void* base = nullptr;
    OK(hipMemAddressReserve(&base, reserve, CHUNK, nullptr, 0));

    std::vector<Chunk> chunks(max_chunks);
    for (size_t i = 0; i < max_chunks; ++i) chunks[i].off = i * CHUNK;

    auto map_chunk = [&](size_t i) -> bool {
        if (chunks[i].mapped) return true;
        hipMemGenericAllocationHandle_t h;
        if (hipMemCreate(&h, CHUNK, &prop, 0) != hipSuccess) return false;
        void* addr = (char*)base + chunks[i].off;
        if (hipMemMap(addr, CHUNK, 0, h, 0) != hipSuccess) { hipMemRelease(h); return false; }
        if (hipMemSetAccess(addr, CHUNK, &desc, 1) != hipSuccess) {
            hipMemUnmap(addr, CHUNK); hipMemRelease(h); return false;
        }
        chunks[i].h = h; chunks[i].mapped = true;
        return true;
    };
    auto unmap_chunk = [&](size_t i) {
        if (!chunks[i].mapped) return;
        void* addr = (char*)base + chunks[i].off;
        hipMemUnmap(addr, CHUNK);
        hipMemRelease(chunks[i].h);
        chunks[i].mapped = false;
    };

    // --- the canary: chunk 0, written once, never touched again by us ---
    if (!map_chunk(0)) { printf("could not map canary chunk\n"); return 2; }
    size_t words = CHUNK / sizeof(unsigned);
    std::vector<unsigned> host(words, PATTERN);
    OK(hipMemcpy(base, host.data(), CHUNK, hipMemcpyHostToDevice));
    OK(hipDeviceSynchronize());
    printf("canary written at %p (%zu words of 0x%08X)\n\n", base, words, PATTERN);

    std::vector<unsigned> check(words);
    unsigned long long retries = 0;
    size_t high_water = 1;

    for (int c = 1; c <= cycles; ++c) {
        // grow toward exhaustion; count retries the way the allocator would
        size_t grew = 0;
        for (size_t i = 1; i < max_chunks; ++i) {
            if (map_chunk(i)) { ++grew; if (i + 1 > high_water) high_water = i + 1; }
            else {
                ++retries;
                // allocator behaviour on failure: release cached blocks, retry once
                for (size_t j = max_chunks - 1; j > max_chunks * 3 / 4 && j > 1; --j) unmap_chunk(j);
                if (map_chunk(i)) ++grew;
                else break;
            }
        }
        // shrink: drop a strided subset, then remap DIFFERENT handles at those VAs
        for (size_t i = 1; i < max_chunks; i += 3) unmap_chunk(i);
        for (size_t i = 1; i < max_chunks; i += 3) map_chunk(i);

        // read the live, never-rewritten canary back
        memset(check.data(), 0xFF, CHUNK);
        hipError_t rc = hipMemcpy(check.data(), base, CHUNK, hipMemcpyDeviceToHost);
        if (rc != hipSuccess) { printf("cycle %d: canary readback FAILED: %s\n", c, hipGetErrorString(rc)); return 3; }

        size_t bad = 0, zeros = 0, first_bad = 0;
        for (size_t w = 0; w < words; ++w) {
            if (check[w] != PATTERN) {
                if (!bad) first_bad = w;
                ++bad;
                if (check[w] == 0) ++zeros;
            }
        }
        if (bad) {
            printf("\n*** REPRODUCED at cycle %d ***\n", c);
            printf("    %zu/%zu words corrupt (%zu of them zero), first at word %zu (byte %zu)\n",
                   bad, words, zeros, first_bad, first_bad * sizeof(unsigned));
            printf("    retries so far: %llu, high-water chunks: %zu\n", retries, high_water);
            return 4;
        }
        if (c % 10 == 0 || c == 1)
            printf("cycle %3d: canary intact | grew %zu | retries %llu | high-water %zu chunks (%.2f GiB)\n",
                   c, grew, retries, high_water, high_water * CHUNK / 1073741824.0);
    }

    printf("\n=== canary intact through %d cycles (retries %llu, high-water %zu chunks) ===\n",
           cycles, retries, high_water);
    for (size_t i = 0; i < max_chunks; ++i) unmap_chunk(i);
    hipMemAddressFree(base, reserve);
    return 0;
}
