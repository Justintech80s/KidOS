# KidOS Windows Lockdown Mode Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Guardian-owned Windows Lockdown Mode using Microsoft Assigned Access restricted-user profiles for a validated standard child account without restricting the parent/admin account.

**Architecture:** A platform-neutral lockdown contract is implemented by a Windows-specific Rust module that generates and validates Assigned Access configuration and calls a narrow privileged adapter. Guardian owns state, authorization, rollback, and fail-closed behavior; the Tauri renderer receives named commands only and never raw XML or OS-command execution.

**Tech Stack:** Rust, TypeScript, React, Tauri 2, Windows Assigned Access/CSP boundary, Vitest, cargo test, GitHub Actions Windows runner.

**Spec:** `docs/superpowers/specs/2026-09-02-kidos-windows-lockdown-mode-design.md`

## Global Constraints
- Windows lockdown targets Assigned Access supported Windows editions, with Windows Pro as the broad consumer baseline.
- Lockdown applies only to a validated standard child account; administrator targets are rejected.
- Parent/admin account remains outside the restricted profile.
- No generic PowerShell, CMD, shell, registry, WMI, or CIM execution bridge is exposed to the renderer.
- Raw Assigned Access XML is never accepted from the child renderer.
- Guardian/platform validation failure enters `restricted_safe_mode`.
- Parent maintenance unlock is authorized and time-bounded.
- Shared KidOS architecture must remain compatible with later macOS, Linux, Android, and iOS/iPadOS platform implementations.

---

### Task 1: Define platform-neutral lockdown contracts

**Files:**
- Create: `packages/contracts/src/lockdown.ts`
- Modify: `packages/contracts/src/index.ts`
- Create: `packages/contracts/src/lockdown.test.ts`

**Interfaces:**
- Produces: `LockdownState`, `PlatformLockdownCapability`, `ManagedAccount`, `ApprovedDesktopApp`, `LockdownStatus`, `ParentUnlockGrant`.

- [ ] **Step 1: Write failing contract tests**

Test that states are exactly `unmanaged | preparing | locked | parent_unlocked | restricted_safe_mode`; account role is `standard | administrator | unknown`; approved apps contain structured `id`, `displayName`, and `executablePath` fields rather than command lines; unlock grants contain `expiresAt`.

- [ ] **Step 2: Run the contract test**

Run: `pnpm --filter @kidos/contracts test --run`
Expected: FAIL because `lockdown.ts` does not exist.

- [ ] **Step 3: Implement the minimal exported contracts**

Use discriminated string unions and readonly structured interfaces. Do not add raw XML or command fields.

- [ ] **Step 4: Run tests**

Run: `pnpm --filter @kidos/contracts test --run`
Expected: PASS.

- [ ] **Step 5: Commit**

`git commit -am "feat: define KidOS lockdown contracts"`

---

### Task 2: Build deterministic Assigned Access configuration generation

**Files:**
- Create: `crates/guardian-service/src/windows_lockdown/config.rs`
- Create: `crates/guardian-service/src/windows_lockdown/mod.rs`
- Modify: `crates/guardian-service/src/lib.rs`
- Create: `crates/guardian-service/tests/windows_lockdown_config.rs`

**Interfaces:**
- Consumes: validated managed-account and approved-app values.
- Produces: `LockdownProfile`, `LockdownConfigError`, `build_assigned_access_config(profile: &LockdownProfile) -> Result<String, LockdownConfigError>`.

- [ ] **Step 1: Write failing Rust tests**

Cover standard account success, administrator/unknown rejection, KidOS mandatory inclusion, deterministic output, XML escaping, and rejection of administrative executable names including `cmd.exe`, `powershell.exe`, `pwsh.exe`, `regedit.exe`, `wt.exe`, `wscript.exe`, `cscript.exe`, and `mmc.exe`.

- [ ] **Step 2: Run RED**

Run: `cargo test -p guardian-service --test windows_lockdown_config`
Expected: FAIL because the module/API does not exist.

- [ ] **Step 3: Implement minimal generator**

Generate the Assigned Access restricted-user configuration from structured data only. No command-line interpolation. Validate account role and app entries before XML generation.

- [ ] **Step 4: Run GREEN**

Run: `cargo test -p guardian-service --test windows_lockdown_config`
Expected: PASS.

- [ ] **Step 5: Commit**

`git commit -am "feat: generate validated Assigned Access profiles"`

---

### Task 3: Add the privileged Windows lockdown adapter boundary

**Files:**
- Create: `crates/guardian-service/src/windows_lockdown/adapter.rs`
- Create: `crates/guardian-service/tests/windows_lockdown_adapter.rs`

**Interfaces:**
- Produces: trait `WindowsLockdownAdapter` with typed `inspect`, `apply`, and `remove` methods; `InMemoryWindowsLockdownAdapter` for tests; Windows production adapter behind `cfg(target_os = "windows")`.

- [ ] **Step 1: Write failing adapter tests**

Verify apply receives only a validated configuration object, inspect returns typed status, remove has no arbitrary command parameter, and adapter failures are typed errors.

- [ ] **Step 2: Run RED**

Run: `cargo test -p guardian-service --test windows_lockdown_adapter`
Expected: FAIL because adapter types do not exist.

- [ ] **Step 3: Implement the narrow adapter**

Keep OS integration isolated. The renderer cannot construct or call this adapter. Windows implementation applies the Assigned Access configuration through the supported Windows management boundary; non-Windows builds expose an unsupported capability rather than pretending to enforce lockdown.

- [ ] **Step 4: Run GREEN plus cross-platform compile**

Run: `cargo test -p guardian-service --test windows_lockdown_adapter && cargo check --workspace`
Expected: PASS.

- [ ] **Step 5: Commit**

`git commit -am "feat: add Windows lockdown adapter boundary"`

---

### Task 4: Add Guardian lockdown state, rollback, and parent unlock

**Files:**
- Create: `crates/guardian-service/src/windows_lockdown/service.rs`
- Create: `crates/guardian-service/tests/windows_lockdown_service.rs`
- Modify: `crates/guardian-service/src/windows_lockdown/mod.rs`

**Interfaces:**
- Produces: `WindowsLockdownService<A: WindowsLockdownAdapter>`; methods `status`, `prepare_and_apply`, `begin_parent_unlock`, `remove_lockdown`.
- Consumes: existing Guardian parent authorization state rather than a PIN string.

- [ ] **Step 1: Write failing state-machine tests**

Cover unmanaged→preparing→locked, apply failure preserving last-known-valid metadata, malformed/missing inspected state→restricted safe mode, unauthorized unlock rejection, authorized unlock expiry, and unauthorized removal rejection.

- [ ] **Step 2: Run RED**

Run: `cargo test -p guardian-service --test windows_lockdown_service`
Expected: FAIL because the service does not exist.

- [ ] **Step 3: Implement minimal state machine**

Use injected clock/test time for deterministic unlock expiry. Never interpret failure as permission to disable lockdown.

- [ ] **Step 4: Run GREEN**

Run: `cargo test -p guardian-service --test windows_lockdown_service`
Expected: PASS.

- [ ] **Step 5: Commit**

`git commit -am "feat: enforce Guardian lockdown lifecycle"`

---

### Task 5: Expose named Tauri lockdown commands only

**Files:**
- Modify: `apps/shell/src/lib/kidos-api.ts`
- Modify: `apps/shell/src-tauri/src/lib.rs`
- Create: `apps/shell/src/lib/kidos-api-lockdown.test.ts`

**Interfaces:**
- Produces UI API methods: `lockdownStatus()`, `configureWindowsLockdown(request)`, `requestParentMaintenanceUnlock(durationMinutes)`, `removeWindowsLockdown()`.
- No generic dispatcher and no raw XML/string command API.

- [ ] **Step 1: Write failing API tests**

Assert the named methods exist and source contains no `executeCommand`, `runShell`, `powershell`, `cmd.exe`, or raw Assigned Access XML parameter.

- [ ] **Step 2: Run RED**

Run: `pnpm --filter @kidos/shell test --run`
Expected: FAIL because lockdown API methods do not exist.

- [ ] **Step 3: Implement named bridge methods**

Map each method to one Tauri command with structured arguments. Guardian validates parent authorization before privileged changes.

- [ ] **Step 4: Run GREEN**

Run: `pnpm --filter @kidos/shell test --run && cargo check --workspace`
Expected: PASS.

- [ ] **Step 5: Commit**

`git commit -am "feat: expose typed KidOS lockdown commands"`

---

### Task 6: Add parent Lockdown Mode setup/status UI

**Files:**
- Create: `apps/shell/src/features/parent/LockdownSettings.tsx`
- Create: `apps/shell/src/features/parent/LockdownSettings.test.tsx`
- Modify: `apps/shell/src/features/parent/ParentDashboard.tsx`

**Interfaces:**
- Consumes: named `KidOSApi` lockdown methods and existing authorized parent dashboard boundary.
- Produces: child-account selection/validation status, lockdown status, next-sign-in explanation, temporary maintenance unlock, parent-authorized removal.

- [ ] **Step 1: Write failing UI tests**

Verify controls are absent when parent dashboard is unauthorized, admin/unknown account cannot be submitted, successful setup explains that restrictions take effect on the child's next sign-in, restricted-safe-mode is clearly shown, and maintenance unlock shows expiry.

- [ ] **Step 2: Run RED**

Run: `pnpm --filter @kidos/shell test --run`
Expected: FAIL because `LockdownSettings` does not exist.

- [ ] **Step 3: Implement minimal parent UI**

Keep child copy simple and parent controls explicit. Do not claim perfect bypass prevention.

- [ ] **Step 4: Run GREEN**

Run: `pnpm --filter @kidos/shell test --run`
Expected: PASS.

- [ ] **Step 5: Commit**

`git commit -am "feat: add parent Windows Lockdown settings"`

---

### Task 7: Add security scanner and Windows CI verification

**Files:**
- Create: `tests/security/lockdown-source-safety.test.mjs`
- Create: `.github/workflows/windows-lockdown-ci.yml`
- Modify: `docs/safety/threat-model.md`

**Interfaces:**
- Consumes: all lockdown production source.
- Produces: CI gate rejecting unsafe bridges and documenting the new trust boundary.

- [ ] **Step 1: Write the source-safety test**

Scan production shell/Tauri/Guardian lockdown source. Reject generic OS-command dispatch APIs, renderer-supplied raw Assigned Access XML, plaintext PIN handling, and child-configurable admin-tool allowlisting.

- [ ] **Step 2: Run scanner**

Run: `node --test tests/security/lockdown-source-safety.test.mjs`
Expected: PASS only when production source satisfies the boundary.

- [ ] **Step 3: Extend threat model**

Document Assigned Access device-level policy effects, child-account targeting, parent-account separation, provisioning failure, unlock expiry, and platform-specific limitations.

- [ ] **Step 4: Add Windows CI**

Windows runner executes `pnpm install --no-frozen-lockfile`, JS tests, JS builds, Rust tests, `cargo check --workspace`, source-safety scanner, and existing Tauri installer build.

- [ ] **Step 5: Commit**

`git commit -am "ci: verify KidOS Windows Lockdown Mode"`

---

### Task 8: End-to-end Lockdown Mode verification and merge gate

**Files:**
- Create: `tests/e2e/windows-lockdown.spec.tsx`
- Modify: `tests/e2e/desktop-harness.tsx`
- Modify: `tests/e2e/package.json` if needed for workspace dependencies.

**Interfaces:**
- Consumes: parent UI, typed KidOS API, Guardian lockdown state machine through test adapter.
- Produces: household vertical-slice proof.

- [ ] **Step 1: Write failing E2E flows**

Cover: authorized parent selects standard child→configures lockdown→status says next sign-in; administrator target rejected; child cannot access parent lockdown controls; platform status mismatch→restricted safe mode; authorized temporary maintenance unlock expires back to locked.

- [ ] **Step 2: Run RED**

Run: `pnpm test:e2e`
Expected: FAIL until the full vertical slice is wired.

- [ ] **Step 3: Add only missing integration glue**

Use the real parent components/contracts and a fake Windows adapter. Do not fake production security claims.

- [ ] **Step 4: Run full verification**

Run: `pnpm install --no-frozen-lockfile && pnpm -r test --run && pnpm -r build && cargo test --workspace && cargo check --workspace && pnpm test:e2e && node --test tests/security/lockdown-source-safety.test.mjs`
Expected: PASS.

- [ ] **Step 5: Verify Windows workflow**

Require the Windows Lockdown CI and existing Windows installer workflow to pass on the exact feature-head SHA before merge.

- [ ] **Step 6: Merge gate**

Audit changed files, verify no generic shell bridge/plaintext PIN/raw renderer XML, confirm parent/admin account is never targeted, then squash merge using the exact verified head SHA.
