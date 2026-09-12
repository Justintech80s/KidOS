"""KidOS visual Windows VM validation state contract.

This module defines the fail-closed Guardian/UI rules and the machine-readable
visual evidence manifest consumed by the Windows VM validation workflow.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any, Mapping, Sequence


NORMAL_UI = "normal"
RESTRICTED_UI = "restricted"
REQUIRED_SCREENSHOTS = {"healthy-home.png", "restricted-safe-mode.png"}


def _guardian_healthy(phase: Mapping[str, Any]) -> bool:
    return bool(
        phase.get("guardian_service_running")
        and phase.get("guardian_ipc_healthy")
        and phase.get("policy_valid")
    )


def evaluate_run(
    healthy_phase: Mapping[str, Any],
    fail_closed_phase: Mapping[str, Any],
) -> dict[str, Any]:
    """Evaluate the two required KidOS safety states without hiding conflicts."""
    healthy_guardian = _guardian_healthy(healthy_phase)
    healthy_ui = healthy_phase.get("ui_state")
    fail_guardian_healthy = _guardian_healthy(fail_closed_phase)
    fail_ui = fail_closed_phase.get("ui_state")

    contradictions: list[str] = []
    if healthy_guardian and healthy_ui == RESTRICTED_UI:
        contradictions.append("healthy_guardian_showed_restricted_ui")
    if not fail_guardian_healthy and fail_ui == NORMAL_UI:
        contradictions.append("unhealthy_guardian_showed_normal_ui")

    normal_ui = healthy_guardian and healthy_ui == NORMAL_UI
    fail_closed = (not fail_guardian_healthy) and fail_ui == RESTRICTED_UI
    state_pass = normal_ui and fail_closed and not contradictions

    return {
        "guardian": {
            "healthy": healthy_guardian,
            "service_running": bool(healthy_phase.get("guardian_service_running")),
            "ipc_healthy": bool(healthy_phase.get("guardian_ipc_healthy")),
        },
        "policy": {"valid": bool(healthy_phase.get("policy_valid"))},
        "normal_ui": normal_ui,
        "fail_closed": fail_closed,
        "screenshots": [],
        "video": None,
        "video_available": False,
        "visual_evidence_complete": False,
        "contradictions": contradictions,
        "state_pass": state_pass,
        "overall_pass": state_pass,
    }


def attach_visual_evidence(
    result: Mapping[str, Any],
    screenshots: Sequence[str | Path],
    video: str | Path | None,
) -> dict[str, Any]:
    """Attach evidence and fail closed unless both required screenshots exist."""
    updated = dict(result)
    screenshot_values = [str(path) for path in screenshots]
    screenshot_names = {Path(path).name for path in screenshot_values}
    visual_evidence_complete = REQUIRED_SCREENSHOTS.issubset(screenshot_names)
    video_value = None if video is None else str(video)

    updated["screenshots"] = screenshot_values
    updated["video"] = video_value
    updated["video_available"] = video_value is not None
    updated["visual_evidence_complete"] = visual_evidence_complete
    updated["overall_pass"] = bool(updated.get("state_pass")) and visual_evidence_complete
    return updated


def write_result_manifest(artifact_dir: Path, result: Mapping[str, Any]) -> Path:
    """Persist the canonical visual VM result manifest."""
    artifact_dir = Path(artifact_dir)
    artifact_dir.mkdir(parents=True, exist_ok=True)
    output = artifact_dir / "visual-vm-result.json"
    output.write_text(json.dumps(dict(result), indent=2) + "\n", encoding="utf-8")
    return output


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="KidOS visual Windows VM validator")
    parser.add_argument("--vm-name", required=True)
    parser.add_argument("--installer", required=True, type=Path)
    parser.add_argument("--artifact-dir", required=True, type=Path)
    return parser


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    """Parse the stable command-line inputs used by the VM adapters."""
    return build_parser().parse_args(argv)


if __name__ == "__main__":
    parse_args()
