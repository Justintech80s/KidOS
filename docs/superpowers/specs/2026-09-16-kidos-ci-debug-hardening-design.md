# KidOS CI/CD Debug Hardening Design

Date: 2026-09-16
Branch: `feat/ci-debug-hardening-2026`

## Purpose

Extend the existing KidOS CI/CD system so failures are easier to diagnose, fail-closed safety behavior is exercised deliberately, flaky Windows behavior is surfaced early, and releases are tied to the exact commit that passed validation.

This design extends the current workflows instead of replacing them. Existing CodeQL, dependency security, installer upgrade, Windows hardening, Guardian, media safety, Shadow Validation, clean Windows VM, and visual Windows VM coverage remain authoritative in their areas.

## Goals

1. Produce useful diagnostic evidence automatically on every relevant failure.
2. Deliberately inject Guardian, classifier, policy, IPC, and filtering failures and verify fail-closed behavior.
3. Detect intermittent Windows/service/installer failures with bounded repeated runs.
4. Preserve exact commit identity through validation and release evidence.
5. Re-run a focused smoke suite after merge to `main`.
6. Keep cloud-runnable validation independent from the currently offline self-hosted Hyper-V runner.
7. Preserve the real Hyper-V visual validation as a separate release-confidence gate once the self-hosted host and guest runners are online.

## Non-goals

- Do not add new programming languages or CI vendors merely for complexity.
- Do not make the self-hosted Hyper-V workflow a prerequisite for every ordinary PR while the runner is offline.
- Do not let Shadow or test harnesses control the production child environment.
- Do not weaken Restricted Safe Mode or bypass Guardian fail-closed behavior to make tests pass.
- Do not automatically deploy a commit different from the one that passed the release gate.

## Architecture

The validation pipeline is split into four layers.

### Layer 1: Fast PR validation

Runs on every relevant pull request and is expected to finish quickly.

- TypeScript / shell unit tests
- Rust workspace tests
- Guardian contract tests
- system-status/fail-closed unit tests
- CodeQL
- dependency security
- existing focused package tests

Failure output must include commit SHA, workflow/run/job identity, operating system, test command, and a concise machine-readable failure summary when possible.

### Layer 2: Windows runtime hardening

Runs on GitHub-hosted Windows where possible.

- build Guardian
- package classifier
- build installer
- silent install
- service-state verification
- installer upgrade/reinstall coverage
- fault-injection tests
- bounded flake/repetition checks
- automatic evidence capture

### Layer 3: Post-merge validation

Runs against the exact new `main` SHA after merge.

- focused unit/shell/Guardian smoke tests
- installer smoke test
- service health and fail-closed smoke test
- exact-SHA evidence manifest

A post-merge failure does not rewrite history; it marks the merged SHA as not release-ready and produces diagnostics for follow-up.

### Layer 4: Real Hyper-V visual validation

Runs only when the two self-hosted runners are online:

- host label: `kidos-interactive-test`
- guest label: `kidos-visual-vm-guest`

It restores `KidOS-Visual-Test` to checkpoint `KidOS-Clean`, validates healthy Guardian state, captures the real KidOS desktop, stops Guardian, verifies Restricted Safe Mode, captures the restricted state, writes the canonical visual result manifest, and uploads evidence.

This layer is required for a fully validated Windows visual release, but it must not leave all ordinary cloud CI permanently queued when the physical runner is offline.

## Automatic Diagnostic Bundle

Create a reusable PowerShell diagnostics collector under `scripts/windows/` that can be called by Windows workflows with `if: always()`.

The bundle should contain, where available:

- `metadata.json`
  - commit SHA
  - branch/ref
  - workflow name
  - run ID / attempt
  - job name
  - Windows version
  - timestamp
- `services.txt`
  - `KidOSGuardian`
  - `KidOSMediaClassifier`
  - service startup types and current states
- `processes.txt`
  - KidOS-related processes and PIDs
- `recovery-task.txt`
  - `KidOS Guardian Recovery` scheduled task state
- `service-control-manager-events.txt`
- `application-events.txt`
- `installed-files.txt`
- copied `%ProgramData%\KidOS\Diagnostics\*` when present
- installed KidOS version/build metadata when available
- a compact `summary.json` indicating which evidence sources were present or unavailable

Diagnostics collection must be best-effort and must not hide the original test failure.

## Fault-Injection Matrix

Add Windows fault-injection coverage that verifies observable fail-closed behavior rather than merely checking process exit codes.

### Guardian stopped

Action: stop `KidOSGuardian`.

Expected:
- Guardian health becomes unavailable/unhealthy.
- shell/API state does not report protection as active.
- Restricted Safe Mode or equivalent fail-closed state is selected.

### Classifier stopped

Action: stop `KidOSMediaClassifier` where installed.

Expected:
- classifier status is not reported healthy/ready.
- media safety paths do not report a false healthy state.
- Guardian/shell state remains conservative.

### Invalid policy

Action: use the existing test-mode policy fixture/mechanism to provide an invalid or rejected policy. Do not modify a real production child policy.

Expected:
- Guardian does not advertise a valid enforcing state.
- privileged/creation/protected browsing paths remain blocked or restricted according to the current safety contract.

### IPC unavailable

Action: simulate or exercise the existing test harness path for Guardian IPC unavailability.

Expected:
- clients do not infer healthy protection from stale state.
- UI/system status becomes offline/restricted after the allowed freshness window.

### Filter unavailable

Action: use a test fixture or controlled runtime harness to represent filtering enforcement as unavailable.

Expected:
- internet filter status is degraded/offline, never active.

Every injected fault must restore the test environment in `finally`/cleanup logic so subsequent tests start from a known state.

## Flake Detection

Add a bounded repetition job for critical deterministic tests. It must not loop the entire two-hour Windows pipeline.

Initial scope:

- Guardian health/fail-closed contract tests
- shell system-status tests
- installer/service startup smoke test if runtime permits

Default repetitions: 10 in PR CI for the lightweight contract tests. A longer scheduled run can use 25-50 repetitions later.

The job fails on any intermittent failure and records the repetition number and test command in its artifact.

## Exact-SHA Release Evidence

Create a machine-readable release evidence manifest for validated commits.

Minimum fields:

- `commit_sha`
- `ref`
- `generated_at`
- `codeql`
- `dependency_security`
- `unit_tests`
- `guardian_contract`
- `installer_upgrade`
- `fault_injection`
- `flake_check`
- `post_merge_smoke` when applicable
- `hyperv_visual_validation` with `passed`, `failed`, or `not_run`
- `release_ready`

Rules:

- `release_ready` may be true only for the SHA represented by the manifest.
- Hyper-V visual status must be explicit; absence is not equivalent to success.
- Production/release automation must never silently substitute another SHA.

## Post-Merge Smoke Validation

Add a workflow that triggers on pushes to `main` affecting runtime/security/installer code.

It should:

1. Check out the exact pushed SHA.
2. Run the focused shell/Guardian contract tests.
3. Build/install KidOS on GitHub-hosted Windows when feasible.
4. Verify Guardian/service state.
5. Exercise one fail-closed smoke case.
6. Capture diagnostics on failure.
7. Upload `post-merge-evidence.json` tied to `github.sha`.

This workflow is not a substitute for the deeper existing Windows hardening/Shadow workflows; it is an immediate merged-SHA confidence check.

## CI Failure Triage Output

Each new hardening workflow should end with a compact JSON/text summary suitable for human or agent review. The summary should include:

- first failing stage
- exact SHA
- test/fault scenario
- whether failure reproduced in a repeated attempt
- diagnostic artifact name
- whether the failure is code, test infrastructure, or unknown when determinable from direct evidence

Do not classify a failure as infrastructure merely because a rerun passes; label it intermittent unless evidence identifies the infrastructure cause.

## Workflow Integration

Prefer adding reusable scripts and narrowly scoped workflows over duplicating logic across many existing YAML files.

Likely new/updated areas:

- `scripts/windows/collect-ci-diagnostics.ps1`
- `tests/security/` or `tests/windows/` regression tests for diagnostics/fault-injection contracts
- a dedicated fault-injection PowerShell harness under `tests/windows/`
- a focused flake-check script/harness
- `.github/workflows/` for fault injection / post-merge / release evidence
- existing Windows workflows updated only where necessary to call the reusable diagnostics collector

The implementation plan should determine the smallest exact file set after inspecting current test helpers and avoiding duplicate mechanisms already present in Shadow Validation.

## Security Constraints

- No production secrets in artifacts.
- Redact or omit tokens, credentials, registration tokens, cookies, or user data.
- Self-hosted runners remain dedicated to KidOS validation and must not execute untrusted fork code.
- Fault injection uses disposable CI/test environments only.
- Shadow remains non-authoritative and never changes the child production environment.
- Restricted Safe Mode remains fail-closed.

## Rollout

### Phase 1

Implement reusable diagnostics collection with regression tests and wire it into one Windows workflow first.

### Phase 2

Implement the fault-injection harness and CI job.

### Phase 3

Implement bounded flake detection for critical tests.

### Phase 4

Implement exact-SHA evidence manifest and post-merge smoke workflow.

### Phase 5

Wire the same diagnostics/evidence conventions into the Hyper-V visual workflow after the physical host and guest runners are online.

Mutation testing is intentionally deferred to a later scheduled/nightly milestone after these layers are stable.

## Acceptance Criteria

The upgrade is complete when all of the following are true:

1. A deliberately failing Windows test uploads a useful diagnostic bundle without masking the original failure.
2. Guardian-stop fault injection proves the shell/runtime does not report active protection.
3. Classifier/policy/IPC/filter fault scenarios have explicit fail-closed assertions.
4. Critical contract tests run repeatedly and any intermittent failure identifies its repetition.
5. A post-merge `main` workflow validates and records the exact merged SHA.
6. A release evidence manifest cannot report another SHA as validated.
7. Hyper-V visual status is explicitly recorded as `not_run` while the runner is offline, never as passed.
8. Existing CodeQL, dependency, installer, Shadow, hardening, Guardian, and media-safety workflows continue to run without being replaced.
9. No new workflow requires the offline Hyper-V runner for ordinary cloud PR validation.
10. All new tests and workflows are documented and reproducible.
