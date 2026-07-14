#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Summarize matched HIP/Vulkan/Redline #6409 artifacts."""

from __future__ import annotations

import argparse
import json
import statistics
from pathlib import Path
from typing import Any


KEYS = {
    "geometry": ("k", "rows", "workgroup_size", "body_repeats"),
    "reduction": ("variant", "k", "rows", "workgroup_size", "body_repeats"),
    "memory-waitcnt": ("mode", "param", "n", "block_size", "body_iters"),
    "packed-dot": ("mode", "groups", "n", "block_size", "body_iters"),
    "vopd": ("mode", "accums", "n", "block_size", "body_iters"),
    "sampler": ("rows", "vocab", "workgroup_size", "top_k"),
    "two-stage-reduction": (
        "variant",
        "k",
        "rows",
        "workgroup_size",
        "split_count",
        "body_repeats",
    ),
    "q4-selected-dual": (
        "operation",
        "x_rows",
        "rows",
        "experts",
        "in_features",
        "out_features",
        "local_size",
    ),
    "q6-x8-selected-down": (
        "operation",
        "rows",
        "experts",
        "in_features",
        "out_features",
        "local_size",
        "row_tile",
    ),
    "dense-q8": (
        "operation",
        "rows",
        "in_features",
        "out_features",
        "local_size",
        "row_tile",
    ),
}
BACKENDS = ("hip", "vulkan", "redline")
MODES = ("serial_latency", "independent_throughput")


def artifact_rows(data: dict[str, Any]) -> list[dict[str, Any]]:
    measurements = data.get("measurements")
    if isinstance(measurements, dict) and isinstance(measurements.get("rows"), list):
        return measurements["rows"]
    return data.get("rows", []) if isinstance(data.get("rows"), list) else []


def metric(row: dict[str, Any]) -> float:
    return float(row["timing"]["burst"]["gpu_elapsed"]["per_iteration_us"]["median"])


def row_key(family: str, fields: tuple[str, ...], row: dict[str, Any]) -> tuple[Any, ...]:
    values = [row.get(field) for field in fields]
    if family == "reduction" and values[0] == "subgroup":
        values[0] = "wave_shuffle"
    return tuple(values)


def describe(values: list[float]) -> dict[str, Any]:
    wins = sum(value < 1.0 for value in values)
    ties = sum(value == 1.0 for value in values)
    losses = sum(value > 1.0 for value in values)
    return {
        "count": len(values),
        "min": min(values),
        "median": statistics.median(values),
        "max": max(values),
        "wins": wins,
        "win_percent": 100.0 * wins / len(values),
        "ties": ties,
        "losses": losses,
    }


def index_result(root: Path) -> dict[tuple[str, str, tuple[Any, ...]], dict[str, float]]:
    result: dict[tuple[str, str, tuple[Any, ...]], dict[str, float]] = {}
    for mode in MODES:
        for family, fields in KEYS.items():
            indexed: dict[str, dict[tuple[Any, ...], dict[str, Any]]] = {}
            for backend in BACKENDS:
                path = root / mode / f"{backend}-{family}.json"
                data = json.loads(path.read_text())
                indexed[backend] = {
                    row_key(family, fields, row): row
                    for row in artifact_rows(data)
                    if row.get("correctness_pass") is True
                }
            common = set.intersection(*(set(indexed[backend]) for backend in BACKENDS))
            for key in common:
                result[(mode, family, key)] = {
                    backend: metric(indexed[backend][key]) for backend in BACKENDS
                }
    return result


def comparisons(
    indexed: dict[tuple[str, str, tuple[Any, ...]], dict[str, float]]
) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    rows: list[dict[str, Any]] = []
    groups: list[dict[str, Any]] = []
    for mode in MODES:
        for family in KEYS:
            group_rows = []
            for (row_mode, row_family, key), times in indexed.items():
                if (row_mode, row_family) != (mode, family):
                    continue
                ordered = sorted(times, key=times.get)
                winner = ordered[0]
                place = ordered.index("redline") + 1
                row = {
                    "mode": mode,
                    "family": family,
                    "shape": dict(zip(KEYS[family], key, strict=True)),
                    "times_us": times,
                    "winner": winner,
                    "redline_place": place,
                    "redline_over_hip": times["redline"] / times["hip"],
                    "redline_over_vulkan": times["redline"] / times["vulkan"],
                    "redline_over_winner": times["redline"] / times[winner],
                    "redline_deficit_percent": 100.0 * (times["redline"] / times[winner] - 1.0),
                }
                rows.append(row)
                group_rows.append(row)
            ratios_hip = [row["redline_over_hip"] for row in group_rows]
            ratios_vulkan = [row["redline_over_vulkan"] for row in group_rows]
            groups.append(
                {
                    "family": family,
                    "mode": mode,
                    "matched_rows": len(group_rows),
                    "redline_placements": {
                        str(place): sum(row["redline_place"] == place for row in group_rows)
                        for place in (1, 2, 3)
                    },
                    "redline_over_hip": describe(ratios_hip) if ratios_hip else None,
                    "redline_over_vulkan": describe(ratios_vulkan) if ratios_vulkan else None,
                }
            )
    return rows, groups


def baseline_comparison(
    current: dict[tuple[str, str, tuple[Any, ...]], dict[str, float]],
    baseline: dict[tuple[str, str, tuple[Any, ...]], dict[str, float]],
    baseline_path: Path,
) -> dict[str, Any]:
    common = sorted(set(current) & set(baseline), key=str)
    ratios = [current[key]["redline"] / baseline[key]["redline"] for key in common]
    current_vulkan_wins = sum(
        current[key]["redline"] < current[key]["vulkan"] for key in common
    )
    baseline_vulkan_wins = sum(
        baseline[key]["redline"] < baseline[key]["vulkan"] for key in common
    )
    flipped_to_redline = sum(
        baseline[key]["redline"] >= baseline[key]["vulkan"]
        and current[key]["redline"] < current[key]["vulkan"]
        for key in common
    )
    flipped_to_vulkan = sum(
        baseline[key]["redline"] < baseline[key]["vulkan"]
        and current[key]["redline"] >= current[key]["vulkan"]
        for key in common
    )
    return {
        "baseline": str(baseline_path),
        "matched_rows": len(common),
        "current_redline_over_baseline_redline": describe(ratios),
        "redline_over_vulkan_wins": {
            "baseline": baseline_vulkan_wins,
            "current": current_vulkan_wins,
            "delta": current_vulkan_wins - baseline_vulkan_wins,
            "flipped_to_redline": flipped_to_redline,
            "flipped_to_vulkan": flipped_to_vulkan,
        },
    }


def dispatch_comparison(root: Path, dispatch: dict[str, Any]) -> dict[str, Any]:
    redline = {
        (row["mode"], row["sweep"], row["count"], row["grid_blocks"]): row["median_us"]
        for row in dispatch["redline"]["rows"]
        if row["correctness_pass"]
    }
    peers: dict[str, dict[tuple[Any, ...], float]] = {}
    for backend in ("hip", "vulkan"):
        peers[backend] = {}
        for mode in MODES:
            artifact = json.loads((root / mode / f"{backend}-dispatch.json").read_text())
            for row in artifact_rows(artifact):
                if row.get("burst_correctness_pass") is not True:
                    continue
                count = row.get("node_count", row.get("dispatch_count"))
                key = (mode, row["sweep"], count, row["grid_blocks"])
                peers[backend][key] = metric(row)
    common = sorted(set(redline) & set(peers["hip"]) & set(peers["vulkan"]), key=str)
    rows = []
    for key in common:
        times = {
            "redline": redline[key],
            "hip": peers["hip"][key],
            "vulkan": peers["vulkan"][key],
        }
        ordered = sorted(times, key=times.get)
        rows.append(
            {
                "mode": key[0],
                "sweep": key[1],
                "count": key[2],
                "grid_blocks": key[3],
                "times_us": times,
                "winner": ordered[0],
                "redline_place": ordered.index("redline") + 1,
                "redline_over_hip": times["redline"] / times["hip"],
                "redline_over_vulkan": times["redline"] / times["vulkan"],
            }
        )
    ratios_hip = [row["redline_over_hip"] for row in rows]
    ratios_vulkan = [row["redline_over_vulkan"] for row in rows]
    return {
        "valid_redline_rows": len(redline),
        "rejected_redline_rows": len(dispatch["redline"]["rows"]) - len(redline),
        "matched_rows": len(rows),
        "codegen": dispatch["redline"]["codegen"],
        "dependency_boundary": dispatch["redline"]["dependency_boundary"],
        "redline_placements": {
            str(place): sum(row["redline_place"] == place for row in rows)
            for place in (1, 2, 3)
        },
        "redline_over_hip": describe(ratios_hip),
        "redline_over_vulkan": describe(ratios_vulkan),
        "rows": rows,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("result_dir", type=Path)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--baseline", type=Path)
    args = parser.parse_args()
    root = args.result_dir.resolve()
    indexed = index_result(root)
    rows, groups = comparisons(indexed)
    ratios_hip = [row["redline_over_hip"] for row in rows]
    ratios_vulkan = [row["redline_over_vulkan"] for row in rows]
    placements = {
        str(place): sum(row["redline_place"] == place for row in rows)
        for place in (1, 2, 3)
    }
    losses = sorted(
        (row for row in rows if row["redline_place"] != 1),
        key=lambda row: row["redline_deficit_percent"],
    )
    payload: dict[str, Any] = {
        "kind": "radiowave_redline_hipengine_6409_matched_summary",
        "schema_version": 2,
        "ratio_convention": "Redline GPU time / comparison GPU time; below 1.0 favors Redline",
        "stack": {
            "codegen": "Radiowave manifest-bound upstream LLVM HIP code object",
            "submission": "Redline retained stateful PM4 IB",
            "dependency": "consumer-aware RMW: VMEM-only minimal boundary, fail-closed generic fallback",
        },
        "groups": groups,
        "overall": {
            "matched_rows": len(rows),
            "redline_placements": placements,
            "redline_first_percent": 100.0 * placements["1"] / len(rows),
            "redline_over_hip": describe(ratios_hip),
            "redline_over_vulkan": describe(ratios_vulkan),
        },
        "redline_non_first_rows": losses,
    }
    dispatch_path = root / "dispatch-matrix.json"
    if dispatch_path.exists():
        dispatch = json.loads(dispatch_path.read_text())
        payload["dispatch"] = dispatch_comparison(root, dispatch)
    if args.baseline:
        baseline_path = args.baseline.resolve()
        payload["baseline_comparison"] = baseline_comparison(
            indexed, index_result(baseline_path), baseline_path
        )
    args.out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")


if __name__ == "__main__":
    main()
