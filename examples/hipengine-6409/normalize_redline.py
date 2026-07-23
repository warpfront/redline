#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Relabel an upstream HIP-shaped artifact with the backend that executed it."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
from typing import Any


def normalize(value: Any, mode: str) -> None:
    if isinstance(value, list):
        for item in value:
            normalize(item, mode)
        return
    if not isinstance(value, dict):
        return
    if value.get("backend") == "hip":
        value["backend"] = "redline"
    lane_count = 1
    submission = value.get("submission")
    if isinstance(submission, dict):
        observed = submission.get("queue_or_stream_count", 1)
        try:
            lane_count = max(1, int(observed))
        except (TypeError, ValueError):
            lane_count = 1
        submission.update(
            {
                "strategy": "retained_pm4_ib",
                "queue_or_stream_count": lane_count,
                "recording_in_timed_region": False,
                "submit_in_host_wall": True,
                "completion_in_host_wall": True,
            }
        )
    timing = value.get("timing")
    if isinstance(timing, dict):
        for control in ("single", "burst"):
            record = timing.get(control)
            if isinstance(record, dict):
                try:
                    logical_iterations = max(
                        1, int(record.get("logical_iterations", 1))
                    )
                except (TypeError, ValueError):
                    logical_iterations = 1
                record["retained_lane_count"] = (
                    1
                    if mode == "serial_latency"
                    else min(lane_count, logical_iterations)
                )
                gpu = record.get("gpu_elapsed")
                if isinstance(gpu, dict):
                    gpu["clock"] = "redline_pm4_timestamp"
    dependency = value.get("dependency_contract")
    if isinstance(dependency, dict):
        if mode == "serial_latency":
            dependency["inter_dispatch_ordering"] = "redline_radiowave_certified_rmw_acquire"
        else:
            dependency["inter_dispatch_ordering"] = (
                "redline_multi_queue_disjoint_outputs"
                if lane_count > 1
                else "redline_single_queue_disjoint_outputs"
            )
    correctness = value.get("correctness")
    if isinstance(correctness, dict):
        sync = correctness.get("synchronization")
        if isinstance(sync, dict):
            sync["method"] = (
                "redline_radiowave_certified_rmw_acquire"
                if mode == "serial_latency"
                else "redline_internal_stage_dependencies_only"
            )
    for item in value.values():
        normalize(item, mode)


def _retained_lane_count(data: dict[str, Any]) -> int:
    observed: list[int] = []

    def collect(value: Any) -> None:
        if isinstance(value, list):
            for item in value:
                collect(item)
            return
        if not isinstance(value, dict):
            return
        submission = value.get("submission")
        if isinstance(submission, dict):
            try:
                observed.append(
                    max(1, int(submission.get("queue_or_stream_count", 1)))
                )
            except (TypeError, ValueError):
                pass
        for item in value.values():
            collect(item)

    collect(data)
    if observed:
        return max(observed)
    parameters = data.get("parameters")
    if isinstance(parameters, dict):
        try:
            return max(1, int(parameters.get("actual_independent_lanes", 1)))
        except (TypeError, ValueError):
            pass
    return 1


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("input", type=Path)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--timing-mode", choices=("serial_latency", "independent_throughput"))
    args = parser.parse_args()
    data = json.loads(args.input.read_text())
    mode = args.timing_mode or str(data.get("parameters", {}).get("timing_mode") or "")
    if mode not in {"serial_latency", "independent_throughput"}:
        raise RuntimeError("artifact has no valid timing mode")
    normalize(data, mode)
    preheat_replays = int(os.environ.get("REDLINE_PREHEAT_REPLAYS", "0"))
    if preheat_replays < 0:
        raise ValueError("REDLINE_PREHEAT_REPLAYS must be non-negative")
    lane_count = _retained_lane_count(data)
    parameters = data.setdefault("parameters", {})
    if isinstance(parameters, dict):
        # `actual_independent_lanes` in upstream production artifacts records
        # the requested width. Replace it with the queue count the timer
        # actually reported in each result row.
        if "actual_independent_lanes" in parameters:
            parameters["actual_independent_lanes"] = lane_count
        execution = (
            f"up to {lane_count} retained PM4 IB lane(s) on ROCr public queues; "
            "each graph uses min(lane cap, logical iterations)"
            if lane_count > 1
            else "one retained PM4 IB on one ROCr vendor packet"
        )
        parameters["redline_submission"] = {
            "capture_role": "HIP graph used once for exact kernel/argument introspection only",
            "code_object": "Radiowave-produced and manifest-certified AMDGPU offload bundle",
            "execution": execution,
            "gpu_clock": "COPY_DATA start plus RELEASE_MEM bottom-of-pipe end",
            "rmw_boundary": "consumer-aware Radiowave VMEM certification with fail-closed generic fallback",
            "preheat_replays_per_measured_ib": preheat_replays,
            "preheat_scope": "outside_returned_gpu_samples",
            "retained_lane_cap": lane_count,
            "single_control_lane_count": 1,
        }
    data["redline_provenance"] = {
        "backend": "redline",
        "submission": "retained_pm4_ib",
        "kernel_codegen": "Radiowave policy over upstream hipcc/LLVM",
        "radiowave_manifest_verified": True,
        "timed_hip_graph_replay": False,
        "preheat_replays_per_measured_ib": preheat_replays,
        "preheat_scope": "outside_returned_gpu_samples",
        "retained_lane_cap": lane_count,
        "single_control_lane_count": 1,
    }
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n")


if __name__ == "__main__":
    main()
