#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0

import unittest

from normalize_redline import _retained_lane_count, normalize
from redline_hip_timing import _lane_plan


class LanePlanTests(unittest.TestCase):
    def test_gfx11_q4_stripes_complete_iterations(self) -> None:
        self.assertEqual(
            _lane_plan("independent_throughput", queue_cap=4, logical_iterations=10),
            [0, 1, 2, 3, 0, 1, 2, 3, 0, 1],
        )

    def test_gfx12_q2_stripes_complete_iterations(self) -> None:
        self.assertEqual(
            _lane_plan("independent_throughput", queue_cap=2, logical_iterations=5),
            [0, 1, 0, 1, 0],
        )

    def test_serial_and_single_iteration_remain_q1(self) -> None:
        self.assertEqual(_lane_plan("serial_latency", 4, 5), [0, 0, 0, 0, 0])
        self.assertEqual(_lane_plan("independent_throughput", 4, 1), [0])


class NormalizationTests(unittest.TestCase):
    def test_multiqueue_provenance_preserves_observed_lane_count(self) -> None:
        artifact = {
            "backend": "hip",
            "submission": {"queue_or_stream_count": 4},
            "dependency_contract": {},
        }

        normalize(artifact, "independent_throughput")

        self.assertEqual(artifact["backend"], "redline")
        self.assertEqual(artifact["submission"]["queue_or_stream_count"], 4)
        self.assertEqual(
            artifact["dependency_contract"]["inter_dispatch_ordering"],
            "redline_multi_queue_disjoint_outputs",
        )

    def test_control_provenance_distinguishes_q1_from_q2_cap(self) -> None:
        artifact = {
            "submission": {"queue_or_stream_count": 2},
            "timing": {
                "single": {"logical_iterations": 1},
                "burst": {"logical_iterations": 5},
            },
        }

        normalize(artifact, "independent_throughput")

        self.assertEqual(artifact["timing"]["single"]["retained_lane_count"], 1)
        self.assertEqual(artifact["timing"]["burst"]["retained_lane_count"], 2)

    def test_gfx12_provenance_prefers_observed_q2_over_requested_q4(self) -> None:
        artifact = {
            "parameters": {"actual_independent_lanes": 4},
            "measurements": {
                "rows": [{"submission": {"queue_or_stream_count": 2}}]
            },
        }

        self.assertEqual(_retained_lane_count(artifact), 2)

    def test_gfx11_provenance_preserves_observed_q4(self) -> None:
        artifact = {
            "measurements": {
                "rows": [{"submission": {"queue_or_stream_count": 4}}]
            }
        }

        self.assertEqual(_retained_lane_count(artifact), 4)


if __name__ == "__main__":
    unittest.main()
