// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
#pragma once

// Force-included before an unmodified hipEngine HIP harness. It consumes the
// harness timing header, then redirects `HipSequenceTimer` tokens to a faithful
// HIP-graph implementation: capture the SAME launch closure once with
// hipStreamBeginCapture, instantiate it, and time hipGraphLaunch. This is
// exactly the hip_graph strategy from ROCm#6409's own harness applied to the
// matrix kernels, so hipGraph is measured on the same kernels, the same oracle,
// and the same GPU-timestamp domain (hipEvent) as the hip, vulkan, and redline
// arms.
#include "micro_timing_hip.hpp"

#include <chrono>
#include <stdexcept>
#include <unordered_map>
#include <vector>

namespace hipengine::micro {

// hipGraph is the retained-submission counterpart to Redline's retained PM4 IB.
// Capturing a fork/join graph faithfully for independent_throughput is a
// separate exercise; ROCm#6409 is a serial dispatch-latency complaint, so this
// shim implements serial_latency and refuses independent rather than silently
// timing a serialized single-stream graph as if it were independent.
class HipGraphSequenceTimer {
 public:
  HipGraphSequenceTimer(TimingMode mode, uint32_t independent_streams)
      : mode_(mode) {
    (void)independent_streams;
    if (mode_ != TimingMode::SerialLatency) {
      throw std::runtime_error(
          "hipgraph shim implements serial_latency only (independent_throughput "
          "needs a fork/join capture)");
    }
    timing_hip_check(
        hipStreamCreateWithFlags(&stream_, hipStreamNonBlocking),
        "hipStreamCreateWithFlags hipgraph");
    timing_hip_check(hipEventCreate(&start_), "hipEventCreate start");
    timing_hip_check(hipEventCreate(&stop_), "hipEventCreate stop");
  }

  HipGraphSequenceTimer(const HipGraphSequenceTimer&) = delete;
  HipGraphSequenceTimer& operator=(const HipGraphSequenceTimer&) = delete;

  ~HipGraphSequenceTimer() {
    for (auto& [_, exec] : graphs_) {
      (void)hipGraphExecDestroy(exec);
    }
    if (start_) (void)hipEventDestroy(start_);
    if (stop_) (void)hipEventDestroy(stop_);
    if (stream_) (void)hipStreamDestroy(stream_);
  }

  uint32_t stream_count() const { return 1u; }

  template <typename Launch>
  TimingSamples measure(uint32_t logical_iterations, uint32_t samples, Launch&& launch) {
    hipGraphExec_t exec = graph(logical_iterations, launch);
    TimingSamples result;
    result.gpu_sequence_us.reserve(samples);
    result.host_sequence_us.reserve(samples);
    for (uint32_t sample = 0; sample < samples; ++sample) {
      timing_hip_check(hipStreamSynchronize(stream_), "pre-sample sync");
      const auto host_start = std::chrono::steady_clock::now();
      timing_hip_check(hipEventRecord(start_, stream_), "hipEventRecord start");
      timing_hip_check(hipGraphLaunch(exec, stream_), "hipGraphLaunch");
      timing_hip_check(hipEventRecord(stop_, stream_), "hipEventRecord stop");
      timing_hip_check(hipEventSynchronize(stop_), "hipEventSynchronize stop");
      const auto host_stop = std::chrono::steady_clock::now();
      float elapsed_ms = 0.0f;
      timing_hip_check(hipEventElapsedTime(&elapsed_ms, start_, stop_), "hipEventElapsedTime");
      result.gpu_sequence_us.push_back(static_cast<double>(elapsed_ms) * 1000.0);
      result.host_sequence_us.push_back(
          std::chrono::duration<double, std::micro>(host_stop - host_start).count());
    }
    return result;
  }

  template <typename Launch>
  void run_and_wait(uint32_t logical_iterations, Launch&& launch) {
    hipGraphExec_t exec = graph(logical_iterations, launch);
    timing_hip_check(hipGraphLaunch(exec, stream_), "hipGraphLaunch run_and_wait");
    timing_hip_check(hipStreamSynchronize(stream_), "run_and_wait sync");
  }

 private:
  template <typename Launch>
  hipGraphExec_t graph(uint32_t logical_iterations, Launch&& launch) {
    if (auto found = graphs_.find(logical_iterations); found != graphs_.end()) {
      return found->second;
    }
    timing_hip_check(
        hipStreamBeginCapture(stream_, hipStreamCaptureModeGlobal),
        "hipStreamBeginCapture hipgraph");
    for (uint32_t rep = 0; rep < logical_iterations; ++rep) {
      launch(rep, stream_);
    }
    hipGraph_t captured = nullptr;
    timing_hip_check(hipStreamEndCapture(stream_, &captured), "hipStreamEndCapture hipgraph");
    hipGraphExec_t exec = nullptr;
    timing_hip_check(
        hipGraphInstantiate(&exec, captured, nullptr, nullptr, 0), "hipGraphInstantiate");
    (void)hipGraphDestroy(captured);
    graphs_.emplace(logical_iterations, exec);
    return exec;
  }

  TimingMode mode_;
  hipStream_t stream_ = nullptr;
  hipEvent_t start_ = nullptr;
  hipEvent_t stop_ = nullptr;
  std::unordered_map<uint32_t, hipGraphExec_t> graphs_;
};

}  // namespace hipengine::micro

#define HipSequenceTimer HipGraphSequenceTimer
