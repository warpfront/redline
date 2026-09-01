// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
//
// hipModuleLoad host for clause-surgery arms. Same fill/oracle/timing as
// microwave_dotpath_probe.hip (n=32768, body_iters=64, mask=0x1ffffff,
// block 64, 20 chained launches, 3 warmups, 7 samples, 3 repeats).

#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif
#include <dlfcn.h>
#include <hip/hip_runtime.h>
#include <cstddef>

#include <algorithm>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <string>
#include <vector>

using u32 = uint32_t;

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
static const uint32_t kDotGroups = 16;
static const size_t kKernargSize = 296;

static const char* kKernelSym = "_Z15dot_path_kernelPKjS0_Pijjjj";

__host__ uint32_t hash_u32(uint32_t value) {
  value ^= value >> 16;
  value *= 0x7feb352du;
  value ^= value >> 15;
  value *= 0x846ca68bu;
  value ^= value >> 16;
  return value;
}

__host__ uint8_t activation_byte(uint32_t i, uint32_t lane) {
  uint32_t bits = hash_u32(i * 1664525u + lane * 1013904223u + 0x9e3779b9u);
  return static_cast<uint8_t>(bits & 0xffu);
}

__host__ uint8_t weight_byte(uint32_t i, uint32_t lane) {
  uint32_t bits = hash_u32(i * 747796405u + lane * 2891336453u + 0x85ebca6bu);
  return static_cast<uint8_t>(bits & 0xffu);
}

__host__ uint32_t pack_weight_word(uint32_t i) {
  uint32_t packed = 0;
  for (uint32_t lane = 0; lane < 4; ++lane) {
    packed |= static_cast<uint32_t>(weight_byte(i, lane)) << (lane * 8);
  }
  return packed;
}

__host__ uint32_t pack_activation_word(uint32_t i) {
  uint32_t packed = 0;
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
  uint64_t elems = static_cast<uint64_t>(n) * body_iters * kDotGroups;
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

__host__ int dot_q8_s8_s8(uint32_t a, uint32_t b, int c) {
  int acc = c;
  for (uint32_t lane = 0; lane < 4; ++lane) {
    int av = static_cast<int>(static_cast<int8_t>((a >> (lane * 8)) & 0xffu));
    int bv = static_cast<int>(static_cast<int8_t>((b >> (lane * 8)) & 0xffu));
    acc += av * bv;
  }
  return acc;
}

__host__ int run_value(const uint32_t* weights, const uint32_t* activations, uint32_t idx,
                       uint32_t n, uint32_t body_iters, uint32_t data_mask) {
  int sum = 0;
  for (uint32_t iter = 0; iter < body_iters; ++iter) {
    uint32_t base = ((iter * n + idx) * kDotGroups) & data_mask;
    for (uint32_t group = 0; group < kDotGroups; ++group) {
      uint32_t offset = (base + group) & data_mask;
      uint32_t a = weights[offset];
      uint32_t b = activations[offset];
      sum = dot_q8_s8_s8(a, b, sum);
    }
  }
  return sum;
}

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

// Explicit args (40 bytes) + hidden HIP kernarg. hipModuleLaunchKernel with
// extra[] does not append hidden fields; the kernel reads group_size_x at
// 0x34 (see s_load_b32 s3, s[0:1], 0x34). Pack the 296-byte segment.
struct Kernarg {
  const uint32_t* weights;
  const uint32_t* activations;
  int32_t* out;
  uint32_t n;
  uint32_t body_iters;
  uint32_t mask;
  uint32_t sequence_id;
  uint32_t block_count_x;
  uint32_t block_count_y;
  uint32_t block_count_z;
  uint16_t group_size_x;
  uint16_t group_size_y;
  uint16_t group_size_z;
  uint16_t remainder_x;
  uint16_t remainder_y;
  uint16_t remainder_z;
  uint8_t pad[kKernargSize - 64];
};

static_assert(offsetof(Kernarg, block_count_x) == 40, "hidden_block_count_x");
static_assert(offsetof(Kernarg, group_size_x) == 52, "hidden_group_size_x");
static_assert(sizeof(Kernarg) == kKernargSize, "kernarg 296");

static void launch_one(hipFunction_t fn, hipStream_t stream, const uint32_t* d_w,
                       const uint32_t* d_a, int32_t* d_out, uint32_t mask, uint32_t seq,
                       uint32_t grid_x) {
  Kernarg k;
  std::memset(&k, 0, sizeof(k));
  k.weights = d_w;
  k.activations = d_a;
  k.out = d_out;
  k.n = kN;
  k.body_iters = kBodyIters;
  k.mask = mask;
  k.sequence_id = seq;
  k.block_count_x = grid_x;
  k.block_count_y = 1;
  k.block_count_z = 1;
  k.group_size_x = static_cast<uint16_t>(kBlock);
  k.group_size_y = 1;
  k.group_size_z = 1;
  size_t sz = sizeof(k);
  void* extra[] = {HIP_LAUNCH_PARAM_BUFFER_POINTER, &k, HIP_LAUNCH_PARAM_BUFFER_SIZE, &sz,
                   HIP_LAUNCH_PARAM_END};
  OK(hipModuleLaunchKernel(fn, grid_x, 1, 1, kBlock, 1, 1, 0, stream, nullptr, extra));
}

static int run_arm(const char* arm, const char* hsaco_path, uint32_t* d_w, uint32_t* d_a,
                   int32_t* d_out, const std::vector<uint32_t>& hw,
                   const std::vector<uint32_t>& ha, uint32_t mask, int repeats,
                   const char* json_path) {
  hipModule_t mod = nullptr;
  OK(hipModuleLoad(&mod, hsaco_path));
  hipFunction_t fn = nullptr;
  OK(hipModuleGetFunction(&fn, mod, kKernelSym));

  int nregs = 0;
  OK(hipFuncGetAttribute(&nregs, HIP_FUNC_ATTRIBUTE_NUM_REGS, fn));
  printf("arm=%s numRegs=%d hsaco=%s\n", arm, nregs, hsaco_path);

  const uint32_t grid_x = (kN + kBlock - 1) / kBlock;
  hipStream_t stream;
  OK(hipStreamCreate(&stream));
  hipEvent_t ev0, ev1;
  OK(hipEventCreate(&ev0));
  OK(hipEventCreate(&ev1));

  auto launch_chain = [&]() {
    for (uint32_t rep = 0; rep < kReps; ++rep) {
      launch_one(fn, stream, d_w, d_a, d_out, mask, rep, grid_x);
    }
  };

  OK(hipMemset(d_out, 0, sizeof(int32_t) * kN));
  OK(hipDeviceSynchronize());
  launch_chain();
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
        printf("mismatch arm=%s idx=%u got=%d expected=%d\n", arm, idx, host_out[idx], expected);
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
  printf("gate arm=%s mismatches=%ld -> %s\n", arm, mismatches, gate);

  std::vector<double> outer;
  for (int r = 0; r < repeats; ++r) {
    for (uint32_t w = 0; w < kWarmup; ++w) {
      launch_chain();
    }
    OK(hipStreamSynchronize(stream));
    std::vector<double> samples;
    for (uint32_t s = 0; s < kSamples; ++s) {
      OK(hipEventRecord(ev0, stream));
      launch_chain();
      OK(hipEventRecord(ev1, stream));
      OK(hipEventSynchronize(ev1));
      float ms = 0;
      OK(hipEventElapsedTime(&ms, ev0, ev1));
      double us = (static_cast<double>(ms) * 1000.0) / static_cast<double>(kReps);
      samples.push_back(us);
    }
    double med = median_of(samples);
    outer.push_back(med);
    printf("repeat arm=%s r=%d median_us=%.4f samples", arm, r, med);
    for (double u : samples) {
      printf(" %.4f", u);
    }
    printf("\n");
  }

  double med = median_of(outer);
  double mn = *std::min_element(outer.begin(), outer.end());
  double mx = *std::max_element(outer.begin(), outer.end());
  double gbps = kBytesPerDispatch / med / 1000.0;
  printf("RESULT arm=%s wave=32 median_us=%.4f min_us=%.4f max_us=%.4f gbps=%.4f gate=%s\n", arm,
         med, mn, mx, gbps, gate);

  if (json_path && json_path[0]) {
    FILE* f = fopen(json_path, "a");
    if (f) {
      hipDeviceProp_t p;
      OK(hipGetDeviceProperties(&p, 0));
      fprintf(f,
              "{\"arm\":\"%s\",\"arch\":\"%s\",\"wave\":32,\"numRegs\":%d,\"median_us\":%.6f,"
              "\"min_us\":%.6f,\"max_us\":%.6f,\"gbps\":%.6f,\"gate\":\"%s\",\"mismatches\":%ld,"
              "\"repeats_us\":[",
              arm, p.gcnArchName, nregs, med, mn, mx, gbps, gate, mismatches);
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
  OK(hipModuleUnload(mod));
  return mismatches ? 2 : 0;
}

int main(int argc, char** argv) {
  const char* arm = "arm";
  const char* hsaco = nullptr;
  int repeats = 3;
  const char* json_path = nullptr;
  for (int i = 1; i < argc; ++i) {
    if (std::strcmp(argv[i], "--arm") == 0 && i + 1 < argc) {
      arm = argv[++i];
    } else if (std::strcmp(argv[i], "--hsaco") == 0 && i + 1 < argc) {
      hsaco = argv[++i];
    } else if (std::strcmp(argv[i], "--repeats") == 0 && i + 1 < argc) {
      repeats = std::atoi(argv[++i]);
    } else if (std::strcmp(argv[i], "--json") == 0 && i + 1 < argc) {
      json_path = argv[++i];
    } else {
      fprintf(stderr, "unknown arg %s\n", argv[i]);
      return 1;
    }
  }
  if (!hsaco) {
    fprintf(stderr, "usage: run_hsaco --hsaco FILE --arm NAME [--repeats N] [--json PATH]\n");
    return 1;
  }

  print_runtime();
  hipDeviceProp_t prop;
  OK(hipGetDeviceProperties(&prop, 0));
  printf("device=%s gcn=%s\n", prop.name, prop.gcnArchName);
  printf("built_wave=32\n");

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

  int rc = run_arm(arm, hsaco, d_w, d_a, d_out, hw, ha, mask, repeats, json_path);

  OK(hipFree(d_w));
  OK(hipFree(d_a));
  OK(hipFree(d_out));
  return rc;
}
