import importlib.util
import json
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("orchestrator.py")


def load_orchestrator():
    if not MODULE_PATH.exists():
        raise AssertionError("orchestrator.py is missing")
    spec = importlib.util.spec_from_file_location("kidos_visual_vm_orchestrator", MODULE_PATH)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


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


if __name__ == "__main__":
    unittest.main()
