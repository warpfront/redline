// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
// GPU contention load for scripts/6529-contention-ab.sh (ROCm/ROCm#6529).
// hipcc --offload-arch=$ARCH bench/contention_load.hip -o contention_load
#include <hip/hip_runtime.h>
#include <chrono>
#include <cstdio>
#include <cstdlib>
#include <thread>

__global__ void burn(unsigned long long* c) {
  unsigned long long v = 0;
  for (int i = 0; i < 256; ++i) v += atomicAdd(c, 1ULL);
  if (v == ~0ULL) c[0] = v;
}

int main(int argc, char** argv) {
  int sec = (argc > 1) ? std::atoi(argv[1]) : 120;
  if (sec <= 0) sec = 120;
  unsigned long long* d = nullptr;
  if (hipMalloc(&d, sizeof(*d)) != hipSuccess) return 2;
  hipMemset(d, 0, sizeof(*d));
  auto end = std::chrono::steady_clock::now() + std::chrono::seconds(sec);
  while (std::chrono::steady_clock::now() < end) {
    burn<<<1024, 256>>>(d);
    if (hipGetLastError() != hipSuccess) { hipFree(d); return 3; }
    std::this_thread::sleep_for(std::chrono::microseconds(50));
  }
  hipDeviceSynchronize();
  hipFree(d);
  return 0;
}
