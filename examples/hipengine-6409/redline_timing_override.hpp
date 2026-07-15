// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
#pragma once

// This header is force-included before an unmodified hipEngine HIP harness.
// It first consumes the harness timing header, then redirects only subsequent
// `HipSequenceTimer` tokens to the compatible Redline implementation below.
#include "micro_timing_hip.hpp"
#include "redline_dispatch.h"

#include <chrono>
#include <cctype>
#include <cstdio>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <fstream>
#include <map>
#include <memory>
#include <sstream>
#include <stdexcept>
#include <string>
#include <unordered_map>
#include <utility>
#include <vector>

namespace hipengine::micro {

#ifndef REDLINE_DEFAULT_HSACO
#define REDLINE_DEFAULT_HSACO ""
#endif
#ifndef REDLINE_DEFAULT_HSACO_MANIFEST
#define REDLINE_DEFAULT_HSACO_MANIFEST ""
#endif
#ifndef REDLINE_DEFAULT_RADIOWAVE_MANIFEST
#define REDLINE_DEFAULT_RADIOWAVE_MANIFEST ""
#endif

namespace redline_detail {

struct ArgSpec {
  size_t offset = 0;
  size_t size = 0;
  std::string kind;
};

struct KernelSpec {
  std::string name;
  std::string symbol;
  size_t kernarg_size = 0;
  std::vector<ArgSpec> args;
};

// The generated manifest is deliberately tiny JSON. This parser accepts only
// its own stable output rather than becoming a general JSON implementation.
inline std::string read_file(const char* path) {
  std::ifstream input(path, std::ios::binary);
  if (!input) throw std::runtime_error(std::string("cannot open ") + path);
  return std::string(std::istreambuf_iterator<char>(input), {});
}

inline std::string string_field(const std::string& object, const char* key) {
  const std::string needle = std::string("\"") + key + "\"";
  size_t pos = object.find(needle);
  if (pos == std::string::npos) throw std::runtime_error("manifest string field missing");
  pos = object.find('"', object.find(':', pos) + 1) + 1;
  const size_t end = object.find('"', pos);
  return object.substr(pos, end - pos);
}

inline size_t integer_field(const std::string& object, const char* key) {
  const std::string needle = std::string("\"") + key + "\"";
  size_t pos = object.find(needle);
  if (pos == std::string::npos) throw std::runtime_error("manifest integer field missing");
  pos = object.find(':', pos) + 1;
  while (pos < object.size() && std::isspace(static_cast<unsigned char>(object[pos]))) ++pos;
  return std::stoull(object.substr(pos));
}

inline std::vector<std::string> array_objects(const std::string& text, const char* key) {
  const std::string needle = std::string("\"") + key + "\"";
  size_t pos = text.find(needle);
  if (pos == std::string::npos) return {};
  pos = text.find('[', pos);
  std::vector<std::string> result;
  int depth = 0;
  size_t begin = std::string::npos;
  for (++pos; pos < text.size(); ++pos) {
    if (text[pos] == '{') {
      if (depth++ == 0) begin = pos;
    } else if (text[pos] == '}') {
      if (--depth == 0) result.push_back(text.substr(begin, pos - begin + 1));
    } else if (text[pos] == ']' && depth == 0) {
      break;
    }
  }
  return result;
}

inline std::map<std::string, KernelSpec> load_manifest(const char* path) {
  const std::string text = read_file(path);
  std::map<std::string, KernelSpec> result;
  for (const std::string& object : array_objects(text, "kernels")) {
    KernelSpec spec;
    spec.name = string_field(object, "name");
    spec.symbol = string_field(object, "symbol");
    spec.kernarg_size = integer_field(object, "kernarg_size");
    for (const std::string& arg : array_objects(object, "args")) {
      spec.args.push_back(
          {integer_field(arg, "offset"), integer_field(arg, "size"), string_field(arg, "value_kind")});
    }
    result.emplace(spec.name, spec);
    result.emplace(spec.symbol, spec);
    if (spec.symbol.size() > 3 &&
        spec.symbol.compare(spec.symbol.size() - 3, 3, ".kd") == 0) {
      result.emplace(spec.symbol.substr(0, spec.symbol.size() - 3), spec);
    }
  }
  return result;
}

struct Context {
  RlGpu* gpu = nullptr;
  RlModule* module = nullptr;
  std::map<std::string, KernelSpec> specs;

  Context() {
    const char* hsaco = std::getenv("REDLINE_HSACO");
    const char* manifest = std::getenv("REDLINE_HSACO_MANIFEST");
    const char* radiowave_manifest = std::getenv("REDLINE_RADIOWAVE_MANIFEST");
    if (!hsaco || !*hsaco) hsaco = REDLINE_DEFAULT_HSACO;
    if (!manifest || !*manifest) manifest = REDLINE_DEFAULT_HSACO_MANIFEST;
    if (!radiowave_manifest || !*radiowave_manifest) {
      radiowave_manifest = REDLINE_DEFAULT_RADIOWAVE_MANIFEST;
    }
    if (!hsaco || !*hsaco || !manifest || !*manifest || !radiowave_manifest ||
        !*radiowave_manifest) {
      throw std::runtime_error(
          "REDLINE_HSACO, REDLINE_HSACO_MANIFEST, and REDLINE_RADIOWAVE_MANIFEST are required");
    }
    std::string bytes = read_file(hsaco);
    std::string radiowave = read_file(radiowave_manifest);
    gpu = rl_gpu_new(0);
    if (!gpu) throw std::runtime_error("rl_gpu_new failed");
    if (rl_gpu_load_module_radiowave(
            gpu, reinterpret_cast<const uint8_t*>(bytes.data()), bytes.size(),
            reinterpret_cast<const uint8_t*>(radiowave.data()), radiowave.size(), &module) != RL_OK) {
      throw std::runtime_error("rl_gpu_load_module_radiowave failed");
    }
    if (!rl_module_radiowave_certified(module)) {
      throw std::runtime_error("Redline module did not retain Radiowave certification");
    }
    specs = load_manifest(manifest);
  }
};

inline Context& context() {
  static Context* value = new Context();
  return *value;
}

inline void copy_integer(std::vector<uint8_t>& bytes, const ArgSpec& arg, uint64_t value) {
  if (arg.offset + arg.size > bytes.size() || arg.size > sizeof(value)) {
    throw std::runtime_error("hidden kernarg is out of range");
  }
  std::memcpy(bytes.data() + arg.offset, &value, arg.size);
}

inline std::vector<hipGraphNode_t> topological_nodes(hipGraph_t graph) {
  size_t count = 0;
  timing_hip_check(hipGraphGetNodes(graph, nullptr, &count), "hipGraphGetNodes count");
  std::vector<hipGraphNode_t> nodes(count);
  timing_hip_check(hipGraphGetNodes(graph, nodes.data(), &count), "hipGraphGetNodes");
  std::vector<hipGraphNode_t> ordered;
  ordered.reserve(nodes.size());
  std::unordered_map<hipGraphNode_t, bool> emitted;
  while (emitted.size() != nodes.size()) {
    bool progress = false;
    for (hipGraphNode_t node : nodes) {
      if (emitted[node]) continue;
      size_t dep_count = 0;
      timing_hip_check(hipGraphNodeGetDependencies(node, nullptr, &dep_count), "graph dependency count");
      std::vector<hipGraphNode_t> deps(dep_count);
      if (dep_count) {
        timing_hip_check(hipGraphNodeGetDependencies(node, deps.data(), &dep_count), "graph dependencies");
      }
      bool ready = true;
      for (hipGraphNode_t dep : deps) ready = ready && emitted[dep];
      if (ready) {
        emitted[node] = true;
        hipGraphNodeType type{};
        timing_hip_check(hipGraphNodeGetType(node, &type), "hipGraphNodeGetType topology");
        if (type == hipGraphNodeTypeGraph) {
          hipGraph_t child = nullptr;
          timing_hip_check(
              hipGraphChildGraphNodeGetGraph(node, &child), "hipGraphChildGraphNodeGetGraph");
          auto child_nodes = topological_nodes(child);
          ordered.insert(ordered.end(), child_nodes.begin(), child_nodes.end());
        } else {
          ordered.push_back(node);
        }
        progress = true;
      }
    }
    if (!progress) throw std::runtime_error("captured HIP graph is cyclic");
  }
  return ordered;
}

}  // namespace redline_detail

class RedlineSequenceTimer {
 public:
  RedlineSequenceTimer(TimingMode mode, uint32_t independent_streams)
      : mode_(mode), reported_streams_(mode == TimingMode::SerialLatency ? 1u : independent_streams) {
    timing_hip_check(hipStreamCreateWithFlags(&capture_stream_, hipStreamNonBlocking), "capture stream create");
  }

  ~RedlineSequenceTimer() {
    for (auto& [_, ib] : graphs_) rl_pm4_ib_free(ib);
    if (capture_stream_) (void)hipStreamDestroy(capture_stream_);
  }

  uint32_t stream_count() const { return reported_streams_; }

  template <typename Launch>
  TimingSamples measure(uint32_t logical_iterations, uint32_t samples, Launch&& launch) {
    RlPm4Ib* ib = graph(logical_iterations, launch);
    TimingSamples result;
    result.gpu_sequence_us.reserve(samples);
    result.host_sequence_us.reserve(samples);
    for (uint32_t sample = 0; sample < samples; ++sample) {
      double gpu_us = 0.0;
      const auto start = std::chrono::steady_clock::now();
      if (rl_pm4_replay_profiled(ib, &gpu_us) != RL_OK) throw std::runtime_error("profiled replay failed");
      const auto end = std::chrono::steady_clock::now();
      result.gpu_sequence_us.push_back(gpu_us);
      result.host_sequence_us.push_back(std::chrono::duration<double, std::micro>(end - start).count());
    }
    return result;
  }

  template <typename Launch>
  void run_and_wait(uint32_t logical_iterations, Launch&& launch) {
    double ignored = 0.0;
    if (rl_pm4_replay_profiled(graph(logical_iterations, launch), &ignored) != RL_OK) {
      throw std::runtime_error("Redline replay failed");
    }
  }

 private:
  template <typename Launch>
  RlPm4Ib* graph(uint32_t logical_iterations, Launch&& launch) {
    if (auto found = graphs_.find(logical_iterations); found != graphs_.end()) return found->second;
    timing_hip_check(
        hipStreamBeginCapture(capture_stream_, hipStreamCaptureModeGlobal), "hipStreamBeginCapture Redline");
    for (uint32_t rep = 0; rep < logical_iterations; ++rep) launch(rep, capture_stream_);
    hipGraph_t captured = nullptr;
    timing_hip_check(hipStreamEndCapture(capture_stream_, &captured), "hipStreamEndCapture Redline");

    auto nodes = redline_detail::topological_nodes(captured);
    if (nodes.empty() || nodes.size() % logical_iterations != 0) {
      throw std::runtime_error("captured graph does not have a fixed kernel count per iteration");
    }
    const size_t nodes_per_iteration = nodes.size() / logical_iterations;
    auto& ctx = redline_detail::context();
    RlPm4Builder* builder = rl_pm4_builder_new(ctx.gpu);
    if (!builder) throw std::runtime_error("rl_pm4_builder_new failed");

    for (size_t node_index = 0; node_index < nodes.size(); ++node_index) {
      hipGraphNodeType type{};
      timing_hip_check(hipGraphNodeGetType(nodes[node_index], &type), "hipGraphNodeGetType");
      if (type != hipGraphNodeTypeKernel) throw std::runtime_error("capture contains a non-kernel node");
      hipKernelNodeParams params{};
      timing_hip_check(hipGraphKernelNodeGetParams(nodes[node_index], &params), "kernel node params");
      const char* kernel_name = hipKernelNameRefByPtr(params.func, capture_stream_);
      if (!kernel_name) throw std::runtime_error("hipKernelNameRefByPtr failed");
      if (std::getenv("REDLINE_CAPTURE_TRACE")) {
        std::fprintf(
            stderr, "[redline capture] %s grid=%u,%u,%u block=%u,%u,%u\n", kernel_name,
            params.gridDim.x, params.gridDim.y, params.gridDim.z, params.blockDim.x,
            params.blockDim.y, params.blockDim.z);
      }
      auto found = ctx.specs.find(kernel_name);
      if (found == ctx.specs.end()) {
        throw std::runtime_error(std::string("kernel is absent from HSACO manifest: ") + kernel_name);
      }
      const auto& spec = found->second;
      const bool dependency_before =
          node_index > 0 &&
          (mode_ == TimingMode::SerialLatency || node_index % nodes_per_iteration != 0);
      if (dependency_before &&
          rl_pm4_wait_rmw(builder, ctx.module, spec.symbol.c_str()) != RL_OK) {
        throw std::runtime_error("rl_pm4_wait_rmw failed");
      }
      std::vector<uint8_t> kernarg(spec.kernarg_size, 0);
      size_t explicit_index = 0;
      for (const auto& arg : spec.args) {
        if (arg.kind.compare(0, 7, "hidden_") != 0) {
          if (!params.kernelParams || !params.kernelParams[explicit_index]) {
            throw std::runtime_error("captured explicit kernel argument is null");
          }
          std::memcpy(kernarg.data() + arg.offset, params.kernelParams[explicit_index++], arg.size);
        } else if (arg.kind == "hidden_block_count_x") {
          redline_detail::copy_integer(kernarg, arg, params.gridDim.x);
        } else if (arg.kind == "hidden_block_count_y") {
          redline_detail::copy_integer(kernarg, arg, params.gridDim.y);
        } else if (arg.kind == "hidden_block_count_z") {
          redline_detail::copy_integer(kernarg, arg, params.gridDim.z);
        } else if (arg.kind == "hidden_group_size_x") {
          redline_detail::copy_integer(kernarg, arg, params.blockDim.x);
        } else if (arg.kind == "hidden_group_size_y") {
          redline_detail::copy_integer(kernarg, arg, params.blockDim.y);
        } else if (arg.kind == "hidden_group_size_z") {
          redline_detail::copy_integer(kernarg, arg, params.blockDim.z);
        } else if (arg.kind == "hidden_remainder_x") {
          redline_detail::copy_integer(kernarg, arg, params.blockDim.x);
        } else if (arg.kind == "hidden_remainder_y") {
          redline_detail::copy_integer(kernarg, arg, params.blockDim.y);
        } else if (arg.kind == "hidden_remainder_z") {
          redline_detail::copy_integer(kernarg, arg, params.blockDim.z);
        } else if (arg.kind == "hidden_grid_dims") {
          const uint64_t dims = params.gridDim.z > 1 ? 3 : (params.gridDim.y > 1 ? 2 : 1);
          redline_detail::copy_integer(kernarg, arg, dims);
        } else if (arg.kind == "hidden_dynamic_lds_size") {
          redline_detail::copy_integer(kernarg, arg, params.sharedMemBytes);
        }
      }
      const int rc = rl_pm4_dispatch(
          builder, ctx.module, spec.symbol.c_str(), params.gridDim.x * params.blockDim.x,
          params.gridDim.y * params.blockDim.y, params.gridDim.z * params.blockDim.z,
          params.blockDim.x, params.blockDim.y, params.blockDim.z, params.sharedMemBytes,
          kernarg.data(), kernarg.size());
      if (rc != RL_OK) throw std::runtime_error("rl_pm4_dispatch failed");

    }
    RlPm4Ib* ib = nullptr;
    if (rl_pm4_finalize_profiled(ctx.gpu, builder, &ib) != RL_OK) {
      throw std::runtime_error("rl_pm4_finalize_profiled failed");
    }
    timing_hip_check(hipGraphDestroy(captured), "hipGraphDestroy Redline capture");
    graphs_.emplace(logical_iterations, ib);
    return ib;
  }

  TimingMode mode_;
  uint32_t reported_streams_;
  hipStream_t capture_stream_ = nullptr;
  std::map<uint32_t, RlPm4Ib*> graphs_;
};

}  // namespace hipengine::micro

#define HipSequenceTimer RedlineSequenceTimer
