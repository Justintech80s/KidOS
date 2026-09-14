# KidOS Installer Lifecycle Repair Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make KidOS upgrades, partial-install recovery, uninstall, and reinstall reliable on real Windows machines without weakening Guardian fail-closed protection.

**Architecture:** Treat installer lifecycle as a state machine: preflight/cleanup -> stage payload -> atomically install -> verify -> finalize. Critical binaries are never overwritten while services are running; classifier payload is extracted to staging and validated before promotion; uninstall waits for service/process/task removal before deleting files.

**Tech Stack:** NSIS/Tauri Windows installer hooks, PowerShell 5.1, Rust Guardian service, GitHub Actions Windows runners.

**Spec:** Real-machine failures captured 2026-09-11: Guardian binary locked during upgrade, media-classifier extraction rollback, uninstall residue, and subsequent Restricted Safe Mode.

## Global Constraints

- Guardian remains authoritative and fail-closed.
- Never continue after failing to replace a critical Guardian binary.
- Support Windows PowerShell 5.1.
- Preserve recovery guarantees during partial install/uninstall.
- Do not publish a replacement installer until exact-main Windows validation passes.

---

### Task 1: Add installer lifecycle regression contract
**Files:**
- Create: `tests/packaging/installer-lifecycle.test.mjs`
- Modify: `.github/workflows/ci.yml`

- [ ] Assert old KidOS services are stopped before Guardian binary extraction/copy.
- [ ] Assert installer waits for services/processes to disappear.
- [ ] Assert classifier extracts to staging before promotion.
- [ ] Assert uninstall waits for service/task removal.
- [ ] Run the new test and confirm RED on current installer.

### Task 2: Harden service shutdown/removal
**Files:**
- Modify: `scripts/windows/stop-kidos-services.ps1`
- Modify: `apps/shell/src-tauri/windows/hooks.nsh`

- [ ] Stop Guardian/classifier first during upgrades.
- [ ] Wait for service state and service registration removal.
- [ ] Wait for matching processes to terminate.
- [ ] Abort safely with diagnostics on timeout.

### Task 3: Stage and validate classifier payload
**Files:**
- Modify: `scripts/windows/extract-media-classifier.ps1`
- Modify: `apps/shell/src-tauri/windows/hooks.nsh`

- [ ] Validate archive before extraction.
- [ ] Extract to temporary staging directory.
- [ ] Verify classifier host exists.
- [ ] Promote staged directory to final path only after validation.
- [ ] Clean staging on failure.

### Task 4: Harden rollback and uninstall
**Files:**
- Modify: `scripts/windows/rollback-partial-install.ps1`
- Modify: `apps/shell/src-tauri/windows/hooks.nsh`
- Modify: `tests/windows/clean-vm-smoke.ps1`

- [ ] Remove stale services/tasks/processes during rollback.
- [ ] Wait for actual uninstall completion before file deletion.
- [ ] Verify Program Files/ProgramData cleanup.
- [ ] Verify reinstall after uninstall.

### Task 5: Add upgrade + damaged-install VM coverage
**Files:**
- Modify: `tests/windows/clean-vm-smoke.ps1`
- Modify: `.github/workflows/clean-windows-vm-ci.yml` if required.

- [ ] Install version once and leave services running.
- [ ] Run same installer again as an upgrade/reinstall.
- [ ] Verify no locked-file condition and services are healthy afterward.
- [ ] Simulate partial residue and verify repair/reinstall path.

### Task 6: Verify Guardian startup status after install
**Files:**
- Modify: `tests/windows/clean-vm-smoke.ps1`

- [ ] Confirm Guardian service running.
- [ ] Confirm recovery/status IPC reports healthy + valid policy.
- [ ] Confirm shell no longer enters Restricted Safe Mode on healthy install.

### Task 7: Full PR and exact-main validation
- [ ] Run all PR workflows and resolve failures without weakening assertions.
- [ ] Merge only after green.
- [ ] Verify exact-main Clean VM, Shadow, Hardening, Soak, Release, Lockdown, CodeQL, and E2E.
- [ ] Verify new installer artifact belongs to exact main SHA and publish only then.
