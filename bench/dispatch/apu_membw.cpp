// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
//
// Why this probe is shaped this way: hipEngine rows on gfx1151 (Strix Halo,
// ROCm 10.0) show every memory-streaming HIP kernel capped near ~76 GB/s while
// Vulkan reaches 928 GB/s on the same chip and the same ISA profile
// (2026-08-29-gfx1151-packed-dot-codegen.md). This measures raw read bandwidth
// from a plain hipMalloc buffer with a trivial kernel, so the answer cannot
// hide behind kernel structure. Run it as matched pairs (each ROCm release
// built AND run with its own toolchain) to discriminate a 10.0 regression from
// a platform property. It prints its own runtime provenance first, because an
// earlier A/B in this repo silently loaded one runtime for both arms.

#include <hip/hip_runtime.h>

#include <chrono>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <dlfcn.h>

#define CHECK(x)                                                       \
  do {                                                                 \
    hipError_t e_ = (x);                                               \
    if (e_ != hipSuccess) {                                            \
      std::fprintf(stderr, "%s failed: %s\n", #x, hipGetErrorString(e_)); \
      std::exit(1);                                                    \
    }                                                                  \
  } while (0)

__global__ void read_sum(const float4* __restrict__ src, float* __restrict__ out,
                         size_t n4) {
  float acc = 0.f;
  size_t stride = (size_t)gridDim.x * blockDim.x;
  for (size_t i = (size_t)blockIdx.x * blockDim.x + threadIdx.x; i < n4;
       i += stride) {
    float4 v = src[i];
    acc += v.x + v.y + v.z + v.w;
  }
  if (acc == -1.f) out[0] = acc;  // defeat DCE; never taken for zero input
}

int main(int argc, char** argv) {
  int runtime = 0;
  CHECK(hipRuntimeGetVersion(&runtime));
  Dl_info info{};
  dladdr((void*)(hipError_t(*)(void**, size_t)) & hipMalloc, &info);
  std::printf("provenance runtime=%d libamdhip64=%s\n", runtime,
              info.dli_fname ? info.dli_fname : "?");
  hipDeviceProp_t prop{};
  CHECK(hipGetDeviceProperties(&prop, 0));
  std::printf("device=%s arch=%s\n", prop.name, prop.gcnArchName);

  size_t mib = argc > 1 ? std::strtoull(argv[1], nullptr, 10) : 256;
  size_t bytes = mib << 20;
  size_t n4 = bytes / sizeof(float4);
  float4* src = nullptr;
  float* out = nullptr;
  CHECK(hipMalloc(&src, bytes));
  CHECK(hipMalloc(&out, sizeof(float)));
  CHECK(hipMemset(src, 0, bytes));

  const int reps = 20, warm = 3;
  dim3 grid(1024), block(256);
  for (int i = 0; i < warm; ++i)
    hipLaunchKernelGGL(read_sum, grid, block, 0, nullptr, src, out, n4);
  CHECK(hipDeviceSynchronize());
  auto t0 = std::chrono::steady_clock::now();
  for (int i = 0; i < reps; ++i)
    hipLaunchKernelGGL(read_sum, grid, block, 0, nullptr, src, out, n4);
  CHECK(hipDeviceSynchronize());
  auto t1 = std::chrono::steady_clock::now();
  double s = std::chrono::duration<double>(t1 - t0).count();
  double gbps = (double)bytes * reps / s / 1e9;
  std::printf("read %zu MiB x%d: %.3f s -> %.1f GB/s\n", mib, reps, s, gbps);

  // Same measurement from a hipHostMalloc (fine-grained host) buffer for
  // contrast: if device reads of host memory match the hipMalloc number, the
  // hipMalloc buffer is behaving like host memory.
  float4* hsrc = nullptr;
  CHECK(hipHostMalloc(&hsrc, bytes));
  std::memset(hsrc, 0, bytes);
  for (int i = 0; i < warm; ++i)
    hipLaunchKernelGGL(read_sum, grid, block, 0, nullptr, hsrc, out, n4);
  CHECK(hipDeviceSynchronize());
  t0 = std::chrono::steady_clock::now();
  for (int i = 0; i < reps; ++i)
    hipLaunchKernelGGL(read_sum, grid, block, 0, nullptr, hsrc, out, n4);
  CHECK(hipDeviceSynchronize());
  t1 = std::chrono::steady_clock::now();
  s = std::chrono::duration<double>(t1 - t0).count();
  std::printf("read hostMalloc %zu MiB x%d: %.3f s -> %.1f GB/s\n", mib, reps,
              s, (double)bytes * reps / s / 1e9);
  CHECK(hipFree(src));
  CHECK(hipFree(out));
  CHECK(hipHostFree(hsrc));
  return 0;
}
