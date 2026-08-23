// Does a FAILED hipMemSetAccess on a sub-granularity range damage access that
// was already working? This is the untested half of the #2516 measurement and a
// candidate mechanism for ROCm/ROCm#6603's silent zeros.
//
// Rationale: #6603 needs BOTH max_split_size_mb (which makes the caching
// allocator split blocks into sub-granularity pieces) AND expandable_segments
// (which uses VMM). We measured that hipMemSetAccess rejects any range whose
// offset or length is not a multiple of the mapped handles' granularity. If a
// rejected call is *partially applied* -- revoking access it failed to set --
// then a live, correctly-written mapping would afterwards read back as zeros
// instead of faulting, which is exactly the reported symptom.
#include <hip/hip_runtime.h>
#include <cstdio>
#include <vector>

#define OK(x) do { hipError_t _e=(x); if(_e!=hipSuccess){ printf("  [fatal] %s -> %s\n",#x,hipGetErrorString(_e)); return 2; } } while(0)
static const unsigned PAT = 0xA5A5A5A5u;

static int verify(void* base, size_t bytes, const char* when) {
    size_t words = bytes/sizeof(unsigned);
    std::vector<unsigned> h(words, 0xDEADBEEF);
    hipError_t rc = hipMemcpy(h.data(), base, bytes, hipMemcpyDeviceToHost);
    if (rc != hipSuccess) { printf("  %-34s readback ERROR: %s\n", when, hipGetErrorString(rc)); return -1; }
    size_t bad=0, zeros=0, firstbad=0;
    for (size_t i=0;i<words;++i) if (h[i]!=PAT) { if(!bad) firstbad=i; ++bad; if(h[i]==0) ++zeros; }
    if (bad) printf("  %-34s CORRUPT: %zu/%zu words wrong (%zu zero), first word %zu\n", when, bad, words, zeros, firstbad);
    else     printf("  %-34s intact\n", when);
    return (int)bad;
}

int main(int argc, char** argv) {
    int dev = argc>1?atoi(argv[1]):0;
    OK(hipSetDevice(dev));
    hipDeviceProp_t p; OK(hipGetDeviceProperties(&p,dev));

    hipMemAllocationProp prop{};
    prop.type=hipMemAllocationTypePinned;
    prop.location.type=hipMemLocationTypeDevice;
    prop.location.id=dev;
    hipMemAccessDesc desc{};
    desc.location.type=hipMemLocationTypeDevice;
    desc.location.id=dev;
    desc.flags=hipMemAccessFlagsProtReadWrite;

    size_t C=0; OK(hipMemGetAllocationGranularity(&C,&prop,hipMemAllocationGranularityRecommended));
    const int N=4; size_t total=C*N;
    printf("=== %s (%s): %d handles x %zu B ===\n", p.name, p.gcnArchName, N, C);

    void* base=nullptr;
    OK(hipMemAddressReserve(&base,total,C,nullptr,0));
    std::vector<hipMemGenericAllocationHandle_t> hs;
    for(int i=0;i<N;++i){
        hipMemGenericAllocationHandle_t h;
        OK(hipMemCreate(&h,C,&prop,0));
        OK(hipMemMap((char*)base+(size_t)i*C,C,0,h,0));
        hs.push_back(h);
    }
    OK(hipMemSetAccess(base,total,&desc,1));

    size_t words=total/sizeof(unsigned);
    std::vector<unsigned> host(words,PAT);
    OK(hipMemcpy(base,host.data(),total,hipMemcpyHostToDevice));
    OK(hipDeviceSynchronize());
    if (verify(base,total,"after write, before any failure")!=0) return 3;

    // Now issue SetAccess calls that we KNOW are rejected, and re-verify after each.
    struct Bad { size_t off, len; const char* what; } bads[] = {
        { C/2,   C,     "unaligned off, straddling"   },
        { C/2,   C/2,   "unaligned off, inside handle"},
        { 0,     C/2,   "aligned off, short length"   },
        { C,     C/2,   "aligned off, half length"    },
        { 4096,  4096,  "4KiB slice inside handle"    },
    };
    int total_bad = 0;
    for (auto& b : bads) {
        hipError_t rc = hipMemSetAccess((char*)base+b.off,b.len,&desc,1);
        printf("\n  SetAccess(off=%zu len=%zu) [%s] -> %s\n", b.off,b.len,b.what,
               rc==hipSuccess?"OK (unexpected)":hipGetErrorString(rc));
        int bad = verify(base,total,"after the rejected call");
        if (bad>0) total_bad += bad;
        if (bad<0) return 4;
    }

    // Also: does a rejected call poison a subsequent *valid* whole-range call?
    hipError_t rc = hipMemSetAccess(base,total,&desc,1);
    printf("\n  re-issue valid whole-range SetAccess -> %s\n", rc==hipSuccess?"OK":hipGetErrorString(rc));
    verify(base,total,"after valid re-issue");

    printf("\n=== verdict: %s ===\n", total_bad? "REJECTED CALLS DAMAGED LIVE DATA":
           "rejected calls left live data intact (no side effect)");
    for(int i=0;i<N;++i) hipMemUnmap((char*)base+(size_t)i*C,C);
    hipMemAddressFree(base,total);
    for(auto&h:hs) hipMemRelease(h);
    return 0;
}
