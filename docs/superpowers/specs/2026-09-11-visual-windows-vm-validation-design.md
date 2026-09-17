# KidOS Visual Windows VM Validation Design

## Goal
Add a real-Windows visual validation layer that proves KidOS recognizes a healthy Guardian, reaches the normal child interface, and still fails closed when Guardian is unavailable.

## Architecture
A Python orchestration script coordinates a disposable Hyper-V Windows VM on a self-hosted Windows test host. PowerShell remains responsible for Windows-native VM/service/session operations. The harness installs the current KidOS build, validates the KidOSGuardian service plus Guardian IPC/policy health, launches an interactive KidOS child session, captures screenshots and a short screen recording, and emits machine-readable results.

The existing `interactive-windows-session.yml` remains the integration point for a real Windows interactive session. The existing browser-only `kidos-demo-video.yml` remains a presentation/demo path and is not treated as proof that the Windows Guardian service is healthy.

## Visual Evidence
Every successful VM run produces screenshots for: Windows/KidOS startup, Guardian healthy state, normal KidOS child home, and the intentional Guardian-failure Restricted Safe Mode test. It also produces a short MP4 walkthrough when recording support is available on the self-hosted host. Screenshots, video, Guardian/service diagnostics, and a JSON summary are uploaded as GitHub Actions artifacts.

## Safety Invariant
Restricted Safe Mode is never suppressed. The test passes only when (1) healthy Guardian + valid policy leads to normal KidOS, and (2) intentionally unavailable Guardian leads to Restricted Safe Mode. A mismatch fails the run.

## Components
- `tests/windows/visual_vm/orchestrator.py`: Python state machine and result manifest.
- `tests/windows/visual_vm/hyperv.ps1`: Hyper-V lifecycle and snapshot/reset operations.
- `tests/windows/visual_vm/guest-validate.ps1`: installer, service, IPC/policy and UI-state checks inside Windows.
- `tests/windows/visual_vm/capture.ps1`: screenshot/video capture from the interactive session.
- `tests/windows/visual_vm/test_orchestrator.py`: deterministic Python unit tests using mocked VM/guest commands.
- `.github/workflows/interactive-windows-session.yml`: dispatch inputs, execution, assertions, and artifact upload.

## Success Criteria
1. A disposable Windows VM can be prepared/reset from the Python harness.
2. The current KidOS installer is installed in the VM.
3. `KidOSGuardian` is running and Guardian IPC/policy health is positively validated.
4. KidOS launches to the normal child interface while Guardian is healthy.
5. Screenshots visibly document the normal interface.
6. Stopping/invalidating Guardian causes Restricted Safe Mode, proving fail-closed behavior remains intact.
7. Screenshots, optional MP4, diagnostics, and JSON are retained as CI artifacts.
8. Any Guardian/UI contradiction fails CI instead of being hidden.