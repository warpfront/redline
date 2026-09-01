// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
//
// Discriminator: hipEngine q8 packed-dot, ACO ISA vs LLVM ISA, same HIP
// dispatch, same hipMalloc buffers. Build one binary per (arch, wave):
//
//   hipcc --offload-arch=gfx1151 -O3 -std=c++17 \
//     microwave_dotpath_probe.hip -o probe_gfx1151_w32 -ldl
//   hipcc --offload-arch=gfx1151 -O3 -std=c++17 -mwavefrontsize64 -DWAVE64=1 \
//     microwave_dotpath_probe.hip -o probe_gfx1151_w64 -ldl
//
// Arms: llvm32/llvm64 (hipEngine kernel verbatim), aco32/aco64 (wrapped
// Microwave body), llvm128 (uint4 loads of base+chunk). Select with --arm.

#ifndef HIPENGINE_DOT_MODE
#define HIPENGINE_DOT_MODE 0
#endif
#ifndef HIPENGINE_DOT_GROUPS
#define HIPENGINE_DOT_GROUPS 16
#endif
#ifndef HIPENGINE_DOT_FIXED_BLOCK
#define HIPENGINE_DOT_FIXED_BLOCK 0
#endif
#ifndef HIPENGINE_BLOCK_SIZE
#define HIPENGINE_BLOCK_SIZE 64
#endif

#include <hip/hip_runtime.h>

#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif
#include <dlfcn.h>

#include <algorithm>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <string>
#include <vector>

#include "microwave_dotpath_gen.h"

using u32 = uint32_t;

#if HIPENGINE_DOT_FIXED_BLOCK
#define HIPENGINE_DOT_LAUNCH_BOUNDS __launch_bounds__(HIPENGINE_BLOCK_SIZE)
#else
#define HIPENGINE_DOT_LAUNCH_BOUNDS
#endif

// --- verbatim from hipEngine hip_dot_path.hip @ f2c3ad6 (mode 0 / groups 16) ---

__host__ __device__ uint32_t hash_u32(uint32_t value) {
  value ^= value >> 16;
  value *= 0x7feb352du;
  value ^= value >> 15;
  value *= 0x846ca68bu;
  value ^= value >> 16;
  return value;
}

__host__ __device__ uint8_t activation_byte(uint32_t i, uint32_t lane) {
  uint32_t bits = hash_u32(i * 1664525u + lane * 1013904223u + 0x9e3779b9u);
  return static_cast<uint8_t>(bits & 0xffu);
}

__host__ __device__ uint8_t weight_byte(uint32_t i, uint32_t lane) {
  uint32_t bits = hash_u32(i * 747796405u + lane * 2891336453u + 0x85ebca6bu);
#if HIPENGINE_DOT_MODE == 0
  return static_cast<uint8_t>(bits & 0xffu);
#elif HIPENGINE_DOT_MODE == 1 || HIPENGINE_DOT_MODE == 3
  return static_cast<uint8_t>(bits & 0x0fu);
#elif HIPENGINE_DOT_MODE == 2
  return static_cast<uint8_t>(bits & 0x3fu);
#else
  return static_cast<uint8_t>(bits & 0xffu);
#endif
}

__host__ __device__ uint32_t pack_weight_word(uint32_t i) {
  uint32_t packed = 0;
#pragma unroll
  for (uint32_t lane = 0; lane < 4; ++lane) {
    packed |= static_cast<uint32_t>(weight_byte(i, lane)) << (lane * 8);
  }
  return packed;
}

__host__ __device__ uint32_t pack_activation_word(uint32_t i) {
  uint32_t packed = 0;
#pragma unroll
  for (uint32_t lane = 0; lane < 4; ++lane) {
    packed |= static_cast<uint32_t>(activation_byte(i, lane)) << (lane * 8);
  }
  return packed;
}

uint32_t next_power_of_two(uint64_t value) {
  if (value <= 1) {
    return 1;
  }
  --value;
  value |= value >> 1;
  value |= value >> 2;
  value |= value >> 4;
  value |= value >> 8;
  value |= value >> 16;
  ++value;
  return static_cast<uint32_t>(value);
}

uint32_t required_data_elems(uint32_t n, uint32_t body_iters) {
  uint64_t elems = static_cast<uint64_t>(n) * body_iters * HIPENGINE_DOT_GROUPS;
  return next_power_of_two(elems < 1024 ? 1024 : elems);
}

void fill_inputs(std::vector<uint32_t>& weights, std::vector<uint32_t>& activations, uint32_t n,
                 uint32_t body_iters) {
  uint32_t data_elems = required_data_elems(n, body_iters);
  weights.resize(data_elems);
  activations.resize(data_elems);
  for (uint32_t i = 0; i < data_elems; ++i) {
    weights[i] = pack_weight_word(i);
    activations[i] = pack_activation_word(i);
  }
}

__host__ __device__ int dot_q8_s8_s8(uint32_t a, uint32_t b, int c) {
#if defined(__HIP_DEVICE_COMPILE__)
  return __builtin_amdgcn_sudot4(true, static_cast<int>(a), true, static_cast<int>(b), c, false);
#else
  int acc = c;
#pragma unroll
  for (uint32_t lane = 0; lane < 4; ++lane) {
    int av = static_cast<int>(static_cast<int8_t>((a >> (lane * 8)) & 0xffu));
    int bv = static_cast<int>(static_cast<int8_t>((b >> (lane * 8)) & 0xffu));
    acc += av * bv;
  }
  return acc;
#endif
}

__host__ __device__ int run_value(const uint32_t* __restrict__ weights,
                                 const uint32_t* __restrict__ activations, uint32_t idx, uint32_t n,
                                 uint32_t body_iters, uint32_t data_mask) {
  int sum = 0;
  for (uint32_t iter = 0; iter < body_iters; ++iter) {
    uint32_t base = ((iter * n + idx) * HIPENGINE_DOT_GROUPS) & data_mask;
#pragma unroll
    for (uint32_t group = 0; group < HIPENGINE_DOT_GROUPS; ++group) {
      uint32_t offset = (base + group) & data_mask;
      uint32_t a = weights[offset];
      uint32_t b = activations[offset];
      sum = dot_q8_s8_s8(a, b, sum);
    }
  }
  return sum;
}

HIPENGINE_DOT_LAUNCH_BOUNDS
__global__ void dot_path_kernel(const uint32_t* __restrict__ weights,
                               const uint32_t* __restrict__ activations, int32_t* __restrict__ out,
                               uint32_t n, uint32_t body_iters, uint32_t data_mask,
                               uint32_t sequence_id) {
  uint32_t idx =
#if HIPENGINE_DOT_FIXED_BLOCK
      blockIdx.x * HIPENGINE_BLOCK_SIZE + threadIdx.x;
#else
      blockIdx.x * blockDim.x + threadIdx.x;
#endif
  if (idx >= n) {
    return;
  }
  out[idx] = run_value(weights, activations, idx, n, body_iters, data_mask) +
             static_cast<int32_t>(sequence_id);
}

// llvm128: same semantics, uint4 loads of base+chunk. Legal because base is a
// multiple of 16 and mask+1 >= 16, so (base+chunk)&mask stays 4-word aligned.
HIPENGINE_DOT_LAUNCH_BOUNDS
__global__ void dot_path_kernel_u4(const uint32_t* __restrict__ weights,
                                  const uint32_t* __restrict__ activations,
                                  int32_t* __restrict__ out, uint32_t n, uint32_t body_iters,
                                  uint32_t data_mask, uint32_t sequence_id) {
  uint32_t idx = blockIdx.x * blockDim.x + threadIdx.x;
  if (idx >= n) {
    return;
  }
  int sum = 0;
  for (uint32_t iter = 0; iter < body_iters; ++iter) {
    uint32_t base = ((iter * n + idx) * HIPENGINE_DOT_GROUPS) & data_mask;
#pragma unroll
    for (uint32_t chunk = 0; chunk < HIPENGINE_DOT_GROUPS; chunk += 4) {
      uint32_t offset = (base + chunk) & data_mask;
      uint4 a = *reinterpret_cast<const uint4*>(weights + offset);
      uint4 b = *reinterpret_cast<const uint4*>(activations + offset);
      sum = dot_q8_s8_s8(a.x, b.x, sum);
      sum = dot_q8_s8_s8(a.y, b.y, sum);
      sum = dot_q8_s8_s8(a.z, b.z, sum);
      sum = dot_q8_s8_s8(a.w, b.w, sum);
    }
  }
  out[idx] = sum + static_cast<int32_t>(sequence_id);
}

extern "C" __global__ __launch_bounds__(64) void aco_dot_path(
    const uint32_t* __restrict__ weights, const uint32_t* __restrict__ activations,
    int32_t* __restrict__ out, uint32_t n, uint32_t body_iters, uint32_t data_mask,
    uint32_t sequence_id) {
  (void)weights;
  (void)activations;
  (void)out;
  (void)n;
  (void)body_iters;
  (void)data_mask;
  (void)sequence_id;
#if defined(__HIP_DEVICE_COMPILE__)
#if defined(__gfx1151__)
#ifdef WAVE64
  MICROWAVE_DOTPATH_INVOKE_GFX1151_W64;
#else
  MICROWAVE_DOTPATH_INVOKE_GFX1151_W32;
#endif
#elif defined(__gfx1100__)
#ifdef WAVE64
  MICROWAVE_DOTPATH_INVOKE_GFX1100_W64;
#else
  MICROWAVE_DOTPATH_INVOKE_GFX1100_W32;
#endif
#elif defined(__gfx1201__)
#ifdef WAVE64
  MICROWAVE_DOTPATH_INVOKE_GFX1201_W64;
#else
  MICROWAVE_DOTPATH_INVOKE_GFX1201_W32;
#endif
#else
#error "aco_dot_path: unsupported arch"
#endif
#endif
}

// --- host ---

#define OK(x)                                                                                      \
  do {                                                                                             \
    hipError_t _e = (x);                                                                           \
    if (_e != hipSuccess) {                                                                        \
      fprintf(stderr, "FATAL %s -> %s\n", #x, hipGetErrorString(_e));                              \
      std::exit(1);                                                                                \
    }                                                                                              \
  } while (0)

static const uint32_t kN = 32768;
static const uint32_t kBodyIters = 64;
static const uint32_t kBlock = 64;
static const uint32_t kReps = 20;
static const uint32_t kWarmup = 3;
static const uint32_t kSamples = 7;
static const double kBytesPerDispatch = 268435456.0;

struct Arm {
  const char* name;
  const void* func;
  int want_wave;
};

static double median_of(std::vector<double> v) {
  std::sort(v.begin(), v.end());
  return v[v.size() / 2];
}

static void print_runtime() {
  int ver = 0;
  OK(hipRuntimeGetVersion(&ver));
  printf("hipRuntimeGetVersion=%d\n", ver);
  Dl_info info;
  std::memset(&info, 0, sizeof(info));
  void* hip_malloc_sym = dlsym(RTLD_DEFAULT, "hipMalloc");
  if (hip_malloc_sym && dladdr(hip_malloc_sym, &info) && info.dli_fname) {
    printf("libamdhip64=%s\n", info.dli_fname);
  } else {
    printf("libamdhip64=(dladdr failed)\n");
  }
}

static void print_func_attrs(const char* arm, const void* func) {
  hipFuncAttributes attr;
  std::memset(&attr, 0, sizeof(attr));
  hipError_t e = hipFuncGetAttributes(&attr, func);
  if (e != hipSuccess) {
    printf("arm=%s hipFuncGetAttributes failed: %s\n", arm, hipGetErrorString(e));
    return;
  }
  printf("arm=%s numRegs=%d localSizeBytes=%zu sharedSizeBytes=%zu constSizeBytes=%zu "
         "maxThreadsPerBlock=%d binaryVersion=%d ptxVersion=%d\n",
         arm, attr.numRegs, attr.localSizeBytes, attr.sharedSizeBytes, attr.constSizeBytes,
         attr.maxThreadsPerBlock, attr.binaryVersion, attr.ptxVersion);
}

using LaunchFn = void (*)(const uint32_t*, const uint32_t*, int32_t*, uint32_t, uint32_t, uint32_t,
                          uint32_t);

static int run_arm(const Arm& arm, uint32_t* d_w, uint32_t* d_a, int32_t* d_out,
                   const std::vector<uint32_t>& hw, const std::vector<uint32_t>& ha, uint32_t mask,
                   int repeats, const char* json_path) {
#ifdef WAVE64
  const int built_wave = 64;
#else
  const int built_wave = 32;
#endif
  if (arm.want_wave != built_wave) {
    printf("SKIP arm=%s want_wave=%d built_wave=%d\n", arm.name, arm.want_wave, built_wave);
    return 0;
  }

  print_func_attrs(arm.name, arm.func);

  dim3 block(kBlock);
  dim3 grid((kN + kBlock - 1) / kBlock);
  hipStream_t stream;
  OK(hipStreamCreate(&stream));
  hipEvent_t ev0, ev1;
  OK(hipEventCreate(&ev0));
  OK(hipEventCreate(&ev1));

  auto launch_chain = [&](uint32_t /*unused*/) {
    for (uint32_t rep = 0; rep < kReps; ++rep) {
      if (std::strcmp(arm.name, "llvm128") == 0) {
        hipLaunchKernelGGL(dot_path_kernel_u4, grid, block, 0, stream, d_w, d_a, d_out, kN,
                           kBodyIters, mask, rep);
      } else if (std::strncmp(arm.name, "aco", 3) == 0) {
        hipLaunchKernelGGL(aco_dot_path, grid, block, 0, stream, d_w, d_a, d_out, kN, kBodyIters,
                           mask, rep);
      } else {
        hipLaunchKernelGGL(dot_path_kernel, grid, block, 0, stream, d_w, d_a, d_out, kN, kBodyIters,
                           mask, rep);
      }
    }
  };

  OK(hipMemset(d_out, 0, sizeof(int32_t) * kN));
  OK(hipDeviceSynchronize());
  launch_chain(0);
  OK(hipGetLastError());
  OK(hipDeviceSynchronize());

  std::vector<int32_t> host_out(kN);
  OK(hipMemcpy(host_out.data(), d_out, sizeof(int32_t) * kN, hipMemcpyDeviceToHost));
  const uint32_t seq = kReps - 1;
  long mismatches = 0;
  auto check_idx = [&](uint32_t idx) {
    int expected =
        run_value(hw.data(), ha.data(), idx, kN, kBodyIters, mask) + static_cast<int32_t>(seq);
    if (host_out[idx] != expected) {
      if (mismatches < 8) {
        printf("mismatch arm=%s idx=%u got=%d expected=%d\n", arm.name, idx, host_out[idx],
               expected);
      }
      ++mismatches;
    }
  };
  check_idx(0);
  check_idx(kN - 1);
  const uint32_t stride = kN / 4096u;
  for (uint32_t i = 0; i < 4096u; ++i) {
    check_idx(i * stride);
  }
  const char* gate = mismatches ? "FAIL" : "PASS";
  printf("gate arm=%s mismatches=%ld -> %s\n", arm.name, mismatches, gate);

  std::vector<double> outer;
  for (int r = 0; r < repeats; ++r) {
    for (uint32_t w = 0; w < kWarmup; ++w) {
      launch_chain(0);
    }
    OK(hipStreamSynchronize(stream));
    std::vector<double> samples;
    for (uint32_t s = 0; s < kSamples; ++s) {
      OK(hipEventRecord(ev0, stream));
      launch_chain(0);
      OK(hipEventRecord(ev1, stream));
      OK(hipEventSynchronize(ev1));
      float ms = 0;
      OK(hipEventElapsedTime(&ms, ev0, ev1));
      double us = (static_cast<double>(ms) * 1000.0) / static_cast<double>(kReps);
      samples.push_back(us);
    }
    double med = median_of(samples);
    outer.push_back(med);
    printf("repeat arm=%s r=%d median_us=%.4f samples", arm.name, r, med);
    for (double u : samples) {
      printf(" %.4f", u);
    }
    printf("\n");
  }

  double med = median_of(outer);
  double mn = *std::min_element(outer.begin(), outer.end());
  double mx = *std::max_element(outer.begin(), outer.end());
  double gbps = kBytesPerDispatch / med / 1000.0;
  printf("RESULT arm=%s wave=%d median_us=%.4f min_us=%.4f max_us=%.4f gbps=%.4f gate=%s\n",
         arm.name, built_wave, med, mn, mx, gbps, gate);
  if (gbps > 256.0) {
    hipDeviceProp_t p;
    OK(hipGetDeviceProperties(&p, 0));
    const char* arch = p.gcnArchName;
    double peak = 256.0;
    if (std::strstr(arch, "gfx1100")) {
      peak = 960.0;
    } else if (std::strstr(arch, "gfx1201")) {
      peak = 640.0;
    } else if (std::strstr(arch, "gfx1151")) {
      peak = 256.0;
    }
    if (gbps > peak) {
      printf("FLAG measurement bug: gbps=%.4f exceeds DRAM peak %.1f for %s\n", gbps, peak, arch);
    }
  }

  if (json_path && json_path[0]) {
    FILE* f = fopen(json_path, "a");
    if (f) {
      fprintf(f,
              "{\"arm\":\"%s\",\"wave\":%d,\"median_us\":%.6f,\"min_us\":%.6f,\"max_us\":%.6f,"
              "\"gbps\":%.6f,\"gate\":\"%s\",\"mismatches\":%ld,\"repeats_us\":[",
              arm.name, built_wave, med, mn, mx, gbps, gate, mismatches);
      for (size_t i = 0; i < outer.size(); ++i) {
        fprintf(f, "%s%.6f", i ? "," : "", outer[i]);
      }
      fprintf(f, "]}\n");
      fclose(f);
    }
  }

  OK(hipEventDestroy(ev0));
  OK(hipEventDestroy(ev1));
  OK(hipStreamDestroy(stream));
  return mismatches ? 2 : 0;
}

int main(int argc, char** argv) {
  const char* want_arm = "all";
  int repeats = 3;
  const char* json_path = nullptr;
  for (int i = 1; i < argc; ++i) {
    if (std::strcmp(argv[i], "--arm") == 0 && i + 1 < argc) {
      want_arm = argv[++i];
    } else if (std::strcmp(argv[i], "--repeats") == 0 && i + 1 < argc) {
      repeats = std::atoi(argv[++i]);
    } else if (std::strcmp(argv[i], "--json") == 0 && i + 1 < argc) {
      json_path = argv[++i];
    } else {
      fprintf(stderr, "unknown arg %s\n", argv[i]);
      return 1;
    }
  }

  print_runtime();
  hipDeviceProp_t prop;
  OK(hipGetDeviceProperties(&prop, 0));
  printf("device=%s gcn=%s\n", prop.name, prop.gcnArchName);
#ifdef WAVE64
  printf("built_wave=64\n");
#else
  printf("built_wave=32\n");
#endif

  std::vector<uint32_t> hw, ha;
  fill_inputs(hw, ha, kN, kBodyIters);
  const uint32_t mask = static_cast<uint32_t>(hw.size() - 1);
  printf("n=%u body_iters=%u data_elems=%zu data_mask=0x%x bytes/dispatch=%.0f\n", kN, kBodyIters,
         hw.size(), mask, kBytesPerDispatch);

  uint32_t *d_w = nullptr, *d_a = nullptr;
  int32_t* d_out = nullptr;
  OK(hipMalloc(&d_w, hw.size() * 4));
  OK(hipMalloc(&d_a, ha.size() * 4));
  OK(hipMalloc(&d_out, kN * 4));
  OK(hipMemcpy(d_w, hw.data(), hw.size() * 4, hipMemcpyHostToDevice));
  OK(hipMemcpy(d_a, ha.data(), ha.size() * 4, hipMemcpyHostToDevice));

#ifdef WAVE64
  Arm arms[] = {
      {"llvm64", reinterpret_cast<const void*>(dot_path_kernel), 64},
      {"aco64", reinterpret_cast<const void*>(aco_dot_path), 64},
      {"llvm128", reinterpret_cast<const void*>(dot_path_kernel_u4), 64},
  };
#else
  Arm arms[] = {
      {"llvm32", reinterpret_cast<const void*>(dot_path_kernel), 32},
      {"aco32", reinterpret_cast<const void*>(aco_dot_path), 32},
      {"llvm128", reinterpret_cast<const void*>(dot_path_kernel_u4), 32},
  };
#endif

  int rc = 0;
  for (const Arm& arm : arms) {
    if (std::strcmp(want_arm, "all") != 0 && std::strcmp(want_arm, arm.name) != 0) {
      continue;
    }
    int arm_rc = run_arm(arm, d_w, d_a, d_out, hw, ha, mask, repeats, json_path);
    if (arm_rc) {
      rc = arm_rc;
    }
  }

  OK(hipFree(d_w));
  OK(hipFree(d_a));
  OK(hipFree(d_out));
  return rc;
}
