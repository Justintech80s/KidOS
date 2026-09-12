# KidOS Visual Windows VM Validation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Python-orchestrated Hyper-V validation harness that proves Guardian/UI behavior on real Windows and retains visual evidence.

**Architecture:** Python owns the test state machine and result manifest; PowerShell owns Hyper-V, Windows service/session, and capture operations. GitHub Actions invokes the harness on the existing self-hosted interactive Windows runner and uploads screenshots, video, logs, and JSON.

**Tech Stack:** Python 3, pytest/unittest, PowerShell 7/Windows PowerShell, Hyper-V, GitHub Actions, KidOS Windows installer.

**Spec:** `docs/superpowers/specs/2026-09-11-visual-windows-vm-validation-design.md`

## Global Constraints
- Restricted Safe Mode must remain fail-closed and must never be hidden to make a test pass.
- Healthy Guardian + valid policy must lead to normal KidOS UI.
- Unavailable/invalid Guardian must lead to Restricted Safe Mode.
- Visual evidence must be retained as CI artifacts.
- VM execution occurs only on the labeled self-hosted Windows test runner.

---

### Task 1: Python VM orchestration contract

**Files:**
- Create: `tests/windows/visual_vm/orchestrator.py`
- Create: `tests/windows/visual_vm/test_orchestrator.py`

**Interfaces:**
- Consumes: command-line VM name, installer path, artifact directory.
- Produces: `visual-vm-result.json` with Guardian, policy, normal-UI, fail-closed, screenshots, video and overall-pass fields.

- [ ] Write failing tests for healthy/fail-closed state transitions and contradictory Guardian/UI states.
- [ ] Run `python -m unittest tests.windows.visual_vm.test_orchestrator -v` and verify RED.
- [ ] Implement the minimal orchestration/state-validation code.
- [ ] Run the tests and verify GREEN.
- [ ] Commit the task.

### Task 2: Hyper-V lifecycle adapter

**Files:**
- Create: `tests/windows/visual_vm/hyperv.ps1`
- Extend: `tests/windows/visual_vm/test_orchestrator.py`

**Interfaces:**
- Consumes: VM name and operation (`prepare`, `start`, `reset`, `stop`).
- Produces: nonzero exit on lifecycle failure and structured status text for the orchestrator.

- [ ] Add failing command-contract tests.
- [ ] Verify tests fail before the adapter exists.
- [ ] Implement fail-closed Hyper-V lifecycle commands with explicit VM existence checks.
- [ ] Run tests and PowerShell syntax validation.
- [ ] Commit the task.

### Task 3: Guardian and KidOS guest validation

**Files:**
- Create: `tests/windows/visual_vm/guest-validate.ps1`
- Extend: `tests/windows/visual_vm/test_orchestrator.py`

**Interfaces:**
- Consumes: KidOS installer path and expected shell executable.
- Produces: structured JSON containing installer status, `KidOSGuardian` service state, Guardian IPC/policy health, and detected UI state.

- [ ] Write failing fixtures/tests for healthy Guardian, missing Guardian, invalid policy, and UI contradiction.
- [ ] Verify RED.
- [ ] Implement installation, service check, existing Guardian IPC/recovery probe reuse, and UI-state reporting.
- [ ] Verify GREEN without weakening fail-closed checks.
- [ ] Commit the task.

### Task 4: Visual capture

**Files:**
- Create: `tests/windows/visual_vm/capture.ps1`
- Extend: `tests/windows/visual_vm/orchestrator.py`
- Extend: `tests/windows/visual_vm/test_orchestrator.py`

**Interfaces:**
- Consumes: artifact directory and capture label.
- Produces: timestamped PNG screenshots and, where recorder support exists, an MP4 walkthrough.

- [ ] Write failing artifact-manifest tests requiring healthy-home and restricted-mode screenshots.
- [ ] Verify RED.
- [ ] Implement interactive-session screenshot capture and optional FFmpeg recording integration.
- [ ] Verify screenshots are mandatory and video absence is explicitly reported rather than fabricated.
- [ ] Commit the task.

### Task 5: GitHub Actions integration

**Files:**
- Modify: `.github/workflows/interactive-windows-session.yml`

**Interfaces:**
- Consumes: workflow-dispatch VM name, child user, installer path and expected KidOS executable.
- Produces: `KidOS-Visual-Windows-VM-Validation` artifact bundle.

- [ ] Add a regression check that the workflow invokes the Python orchestrator and uploads PNG/MP4/JSON/log evidence.
- [ ] Verify the regression check fails against the old workflow.
- [ ] Wire the harness into the self-hosted `kidos-interactive-test` runner.
- [ ] Upload `artifacts/visual-vm/**` with `if-no-files-found: error` for required evidence.
- [ ] Run repository security/Windows tests and open a PR.

### Task 6: Exact-run visual proof

**Files:** No source changes expected.

**Interfaces:**
- Consumes: merged/PR head SHA and configured Hyper-V self-hosted runner.
- Produces: exact-SHA workflow result plus visual artifact bundle.

- [ ] Dispatch the visual Windows VM validation for the exact SHA.
- [ ] Confirm Guardian healthy + valid policy is reported.
- [ ] Inspect the captured normal KidOS screenshot.
- [ ] Confirm the intentional Guardian failure produces the Restricted Safe Mode screenshot.
- [ ] Confirm the run is green and report the artifact names/SHA to the user.