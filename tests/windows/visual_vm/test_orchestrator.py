import importlib.util
import json
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("orchestrator.py")
HYPERV_PATH = Path(__file__).with_name("hyperv.ps1")
GUEST_VALIDATE_PATH = Path(__file__).with_name("guest-validate.ps1")
CAPTURE_PATH = Path(__file__).with_name("capture.ps1")


def load_orchestrator():
    if not MODULE_PATH.exists():
        raise AssertionError("orchestrator.py is missing")
    spec = importlib.util.spec_from_file_location("kidos_visual_vm_orchestrator", MODULE_PATH)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


def read_hyperv_adapter():
    if not HYPERV_PATH.exists():
        raise AssertionError("hyperv.ps1 is missing")
    return HYPERV_PATH.read_text(encoding="utf-8")


def read_guest_validator():
    if not GUEST_VALIDATE_PATH.exists():
        raise AssertionError("guest-validate.ps1 is missing")
    return GUEST_VALIDATE_PATH.read_text(encoding="utf-8")


def read_capture_adapter():
    if not CAPTURE_PATH.exists():
        raise AssertionError("capture.ps1 is missing")
    return CAPTURE_PATH.read_text(encoding="utf-8")


class VisualVmOrchestratorContractTests(unittest.TestCase):
    def setUp(self):
        self.healthy_phase = {
            "guardian_service_running": True,
            "guardian_ipc_healthy": True,
            "policy_valid": True,
            "ui_state": "normal",
        }
        self.fail_closed_phase = {
            "guardian_service_running": False,
            "guardian_ipc_healthy": False,
            "policy_valid": False,
            "ui_state": "restricted",
        }

    def test_healthy_then_fail_closed_sequence_passes(self):
        orchestrator = load_orchestrator()
        result = orchestrator.evaluate_run(self.healthy_phase, self.fail_closed_phase)
        self.assertTrue(result["guardian"]["healthy"])
        self.assertTrue(result["policy"]["valid"])
        self.assertTrue(result["normal_ui"])
        self.assertTrue(result["fail_closed"])
        self.assertTrue(result["overall_pass"])
        self.assertEqual([], result["contradictions"])

    def test_restricted_ui_while_guardian_is_healthy_fails(self):
        orchestrator = load_orchestrator()
        contradictory = dict(self.healthy_phase, ui_state="restricted")
        result = orchestrator.evaluate_run(contradictory, self.fail_closed_phase)
        self.assertFalse(result["overall_pass"])
        self.assertIn("healthy_guardian_showed_restricted_ui", result["contradictions"])

    def test_normal_ui_while_guardian_is_unavailable_fails(self):
        orchestrator = load_orchestrator()
        contradictory = dict(self.fail_closed_phase, ui_state="normal")
        result = orchestrator.evaluate_run(self.healthy_phase, contradictory)
        self.assertFalse(result["overall_pass"])
        self.assertIn("unhealthy_guardian_showed_normal_ui", result["contradictions"])

    def test_manifest_contains_visual_evidence_fields_and_writes_json(self):
        orchestrator = load_orchestrator()
        result = orchestrator.evaluate_run(self.healthy_phase, self.fail_closed_phase)
        self.assertEqual([], result["screenshots"])
        self.assertIsNone(result["video"])
        with tempfile.TemporaryDirectory() as tmp:
            output = orchestrator.write_result_manifest(Path(tmp), result)
            self.assertEqual(Path(tmp) / "visual-vm-result.json", output)
            stored = json.loads(output.read_text(encoding="utf-8"))
            self.assertEqual(result, stored)

    def test_visual_evidence_requires_both_screenshots(self):
        orchestrator = load_orchestrator()
        result = orchestrator.attach_visual_evidence(
            orchestrator.evaluate_run(self.healthy_phase, self.fail_closed_phase),
            ["healthy-home.png", "restricted-safe-mode.png"],
            None,
        )
        self.assertEqual(
            ["healthy-home.png", "restricted-safe-mode.png"], result["screenshots"]
        )
        self.assertTrue(result["visual_evidence_complete"])
        self.assertTrue(result["overall_pass"])

    def test_missing_required_screenshot_fails_visual_evidence(self):
        orchestrator = load_orchestrator()
        result = orchestrator.attach_visual_evidence(
            orchestrator.evaluate_run(self.healthy_phase, self.fail_closed_phase),
            ["healthy-home.png"],
            None,
        )
        self.assertFalse(result["visual_evidence_complete"])
        self.assertFalse(result["overall_pass"])

    def test_video_is_optional_but_reported(self):
        orchestrator = load_orchestrator()
        result = orchestrator.attach_visual_evidence(
            orchestrator.evaluate_run(self.healthy_phase, self.fail_closed_phase),
            ["healthy-home.png", "restricted-safe-mode.png"],
            "KidOS-Visual-VM.mp4",
        )
        self.assertEqual("KidOS-Visual-VM.mp4", result["video"])
        self.assertTrue(result["video_available"])


class HyperVLifecycleContractTests(unittest.TestCase):
    def test_adapter_exposes_only_expected_operations(self):
        script = read_hyperv_adapter()
        self.assertIn("ValidateSet(\"prepare\", \"start\", \"reset\", \"stop\")", script)
        self.assertIn("[string]$VMName", script)
        self.assertIn("[string]$Operation", script)

    def test_adapter_fails_closed_when_vm_does_not_exist(self):
        script = read_hyperv_adapter()
        self.assertIn("Get-VM -Name $VMName -ErrorAction SilentlyContinue", script)
        self.assertIn('throw "Hyper-V VM not found: $VMName"', script)

    def test_reset_requires_named_snapshot_before_restore(self):
        script = read_hyperv_adapter()
        self.assertIn("Get-VMSnapshot -VMName $VMName -Name $SnapshotName", script)
        self.assertIn('throw "Required Hyper-V snapshot not found:', script)
        self.assertIn("Restore-VMSnapshot", script)
        self.assertIn("-Confirm:$false", script)

    def test_adapter_emits_structured_status(self):
        script = read_hyperv_adapter()
        self.assertIn("ConvertTo-Json", script)
        self.assertIn("passed = $true", script)
        self.assertIn("operation = $Operation", script)
        self.assertIn("vmName = $VMName", script)


class GuestValidationContractTests(unittest.TestCase):
    def test_healthy_guardian_requires_service_ipc_and_policy(self):
        script = read_guest_validator()
        self.assertIn("Get-Service KidOSGuardian", script)
        self.assertIn("guardianIpcHealthy", script)
        self.assertIn("policyValid", script)
        self.assertIn("guardianHealthy", script)

    def test_missing_guardian_fails_closed(self):
        script = read_guest_validator()
        self.assertIn("guardianServiceRunning = $false", script)
        self.assertIn('uiState = "restricted"', script)

    def test_invalid_policy_fails_closed(self):
        script = read_guest_validator()
        self.assertIn("if (-not $policyValid)", script)
        self.assertIn('uiState = "restricted"', script)

    def test_ui_contradiction_is_reported(self):
        script = read_guest_validator()
        self.assertIn("contradiction", script)
        self.assertIn("healthy_guardian_showed_restricted_ui", script)
        self.assertIn("unhealthy_guardian_showed_normal_ui", script)


class VisualCaptureContractTests(unittest.TestCase):
    def test_capture_adapter_writes_png_for_named_state(self):
        script = read_capture_adapter()
        self.assertIn("System.Drawing.Bitmap", script)
        self.assertIn("CopyFromScreen", script)
        self.assertIn("healthy-home.png", script)
        self.assertIn("restricted-safe-mode.png", script)

    def test_capture_adapter_reports_optional_video(self):
        script = read_capture_adapter()
        self.assertIn("ffmpeg", script)
        self.assertIn("videoAvailable", script)
        self.assertIn("KidOS-Visual-VM.mp4", script)

    def test_capture_adapter_emits_json_manifest(self):
        script = read_capture_adapter()
        self.assertIn("screenshots = @(", script)
        self.assertIn("ConvertTo-Json", script)
        self.assertIn("visualCapture", script)


if __name__ == "__main__":
    unittest.main()
