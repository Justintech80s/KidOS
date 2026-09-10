# KidOS Shell UI + Live System State Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rebuild the KidOS Windows home shell to closely match the approved 2026 mockup while keeping Guardian, classifier, parent authorization, safe navigation, and approved-app enforcement authoritative and intact.

**Architecture:** Split the current large `ChildHome.tsx` into focused presentational components plus a small frontend system-status adapter. The shell consumes normalized read-only state and existing safe actions; it never reads privileged credentials or decides security policy. Styling lives in a dedicated 2026 shell stylesheet with responsive CSS Grid/Flexbox rules for 1366x768, 1920x1080, and Windows 125%/150% scaling.

**Tech Stack:** React, TypeScript, Vite, Vitest, Testing Library, Tauri, Rust where a narrow read-only bridge is required, existing KidOS Guardian/Media Classifier services, PowerShell/Windows CI release gates.

**Spec:** `docs/superpowers/specs/2026-09-10-kidos-shell-ui-api-design.md`

## Global Constraints

- Primary visual source of truth is the second approved KidOS mockup; selective scenic richness may be borrowed from the first.
- The shell must not display `Safe Mode: ON` unless authoritative KidOS protection state reports active enforcement.
- The shell must not display `Internet Filter: Active` unless the authoritative filtering path is healthy.
- Guardian remains privileged and authoritative; React is presentation only.
- Browser-facing JavaScript must never read the Media Classifier credential.
- Fast service readiness and deep classifier-model readiness remain separate concepts.
- Safe search must never fall back to unrestricted browsing.
- Parent functions retain their existing authorization boundary.
- Approved-app launching must never accept an arbitrary frontend-provided executable path.
- Existing valid Windows `apps/shell/src-tauri/icons/icon.ico` remains untouched during this UI phase.
- Supported primary desktop targets: 1366x768 and 1920x1080, including common Windows 125% and 150% scaling.
- Existing Clean Windows VM, Windows Soak, Shadow Validation, Windows Lockdown, Windows hardening, and Windows Release workflows remain release gates.

---

## File Structure

Create or modify these focused units:

- `apps/shell/src/features/home/system-status.ts` — pure normalization from raw trusted status into the UI model.
- `apps/shell/src/features/home/system-status.test.ts` — truthfulness contract tests.
- `apps/shell/src/features/home/KidOSHomeShell.tsx` — layout composition only.
- `apps/shell/src/features/home/KidOSSidebar.tsx` — child-safe navigation and Parent entry callback.
- `apps/shell/src/features/home/KidOSTopBar.tsx` — safe-search input, clock/date, profile surface.
- `apps/shell/src/features/home/KidOSGreeting.tsx` — child greeting and positive message.
- `apps/shell/src/features/home/KidOSHomeGrid.tsx` — eight destination tiles.
- `apps/shell/src/features/home/KidOSDock.tsx` — approved actions/apps only.
- `apps/shell/src/features/home/KidOSSafetyStatus.tsx` — authoritative active/degraded/offline indicators.
- `apps/shell/src/features/home/KidOSProfileCard.tsx` — profile rendering with safe fallbacks.
- `apps/shell/src/features/home/kidos-shell-2026.css` — dedicated shell tokens, responsive layout, focus states, scenic background fallback.
- `apps/shell/src/features/home/ChildHome.tsx` — retain existing state/actions and delegate home presentation to the new shell.
- `apps/shell/src/features/home/ChildHome.test.tsx` — preserve old behavior and cover redesigned integration.
- `apps/shell/src/App.test.tsx` — preserve app-level child/parent boundary tests.
- `tests/e2e/child-home-shell.spec.tsx` — redesigned-home E2E coverage.
- `tests/e2e/child-ai-create.spec.tsx` — preserve AI approval behavior.
- `apps/shell/src-tauri/src/main.rs` or the existing command module — only if the current Tauri bridge lacks a safe read-only status command.

---

### Task 1: Establish the truthful system-status contract

**Files:**
- Create: `apps/shell/src/features/home/system-status.ts`
- Create: `apps/shell/src/features/home/system-status.test.ts`

**Interfaces:**
- Consumes: trusted raw status values already available to the shell or returned by a read-only Tauri command.
- Produces:

```ts
export type ProtectionState = 'active' | 'degraded' | 'offline';

export interface RawKidOSSystemStatus {
  guardianReachable: boolean;
  guardianEnforcing: boolean;
  classifierReachable: boolean;
  classifierReady: boolean;
  filterEnforcing: boolean;
  recoveryAvailable: boolean;
  observedAt: number;
}

export interface KidOSSystemStatus {
  guardianRunning: boolean;
  classifierRunning: boolean;
  safeMode: ProtectionState;
  internetFilter: ProtectionState;
  classifierReady: boolean;
  recoveryAvailable: boolean;
  observedAt: number;
}

export function normalizeKidOSSystemStatus(
  raw: RawKidOSSystemStatus,
  now?: number,
): KidOSSystemStatus;
```

- [ ] **Step 1: Write the failing truthfulness tests**

```ts
import { describe, expect, it } from 'vitest';
import { normalizeKidOSSystemStatus } from './system-status';

const NOW = 1_000_000;

function raw(overrides = {}) {
  return {
    guardianReachable: true,
    guardianEnforcing: true,
    classifierReachable: true,
    classifierReady: true,
    filterEnforcing: true,
    recoveryAvailable: true,
    observedAt: NOW,
    ...overrides,
  };
}

describe('normalizeKidOSSystemStatus', () => {
  it('reports active only when authoritative enforcement is healthy', () => {
    expect(normalizeKidOSSystemStatus(raw(), NOW)).toMatchObject({
      safeMode: 'active',
      internetFilter: 'active',
    });
  });

  it('never reports safe mode active when Guardian is unreachable', () => {
    expect(
      normalizeKidOSSystemStatus(raw({ guardianReachable: false }), NOW).safeMode,
    ).toBe('offline');
  });

  it('reports degraded rather than active when Guardian is reachable but not enforcing', () => {
    expect(
      normalizeKidOSSystemStatus(raw({ guardianEnforcing: false }), NOW).safeMode,
    ).toBe('degraded');
  });

  it('never reports filtering active when filter enforcement is false', () => {
    expect(
      normalizeKidOSSystemStatus(raw({ filterEnforcing: false }), NOW).internetFilter,
    ).toBe('degraded');
  });

  it('invalidates stale green state after 15 seconds', () => {
    const status = normalizeKidOSSystemStatus(
      raw({ observedAt: NOW - 16_000 }),
      NOW,
    );
    expect(status.safeMode).not.toBe('active');
    expect(status.internetFilter).not.toBe('active');
  });
});
```

- [ ] **Step 2: Run the focused test and verify RED**

Run:

```bash
pnpm --filter @kidos/shell test --run src/features/home/system-status.test.ts
```

Expected: FAIL because `system-status.ts` does not exist.

- [ ] **Step 3: Implement the minimum pure normalizer**

```ts
export type ProtectionState = 'active' | 'degraded' | 'offline';

export interface RawKidOSSystemStatus {
  guardianReachable: boolean;
  guardianEnforcing: boolean;
  classifierReachable: boolean;
  classifierReady: boolean;
  filterEnforcing: boolean;
  recoveryAvailable: boolean;
  observedAt: number;
}

export interface KidOSSystemStatus {
  guardianRunning: boolean;
  classifierRunning: boolean;
  safeMode: ProtectionState;
  internetFilter: ProtectionState;
  classifierReady: boolean;
  recoveryAvailable: boolean;
  observedAt: number;
}

const STALE_AFTER_MS = 15_000;

export function normalizeKidOSSystemStatus(
  raw: RawKidOSSystemStatus,
  now = Date.now(),
): KidOSSystemStatus {
  const stale = now - raw.observedAt > STALE_AFTER_MS;
  const guardianRunning = raw.guardianReachable && !stale;
  const classifierRunning = raw.classifierReachable && !stale;

  const safeMode: ProtectionState = !guardianRunning
    ? 'offline'
    : raw.guardianEnforcing
      ? 'active'
      : 'degraded';

  const internetFilter: ProtectionState = !guardianRunning
    ? 'offline'
    : raw.filterEnforcing
      ? 'active'
      : 'degraded';

  return {
    guardianRunning,
    classifierRunning,
    safeMode,
    internetFilter,
    classifierReady: classifierRunning && raw.classifierReady,
    recoveryAvailable: raw.recoveryAvailable,
    observedAt: raw.observedAt,
  };
}
```

- [ ] **Step 4: Run focused tests and verify GREEN**

```bash
pnpm --filter @kidos/shell test --run src/features/home/system-status.test.ts
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add apps/shell/src/features/home/system-status.ts apps/shell/src/features/home/system-status.test.ts
git commit -m "Add truthful KidOS system status adapter"
```

---

### Task 2: Add or reuse a narrow read-only platform status bridge

**Files:**
- Inspect/Modify only if required: existing Tauri command module under `apps/shell/src-tauri/src/`
- Modify: `apps/shell/src/features/home/ChildHome.tsx`
- Test: `apps/shell/src/features/home/ChildHome.test.tsx`

**Interfaces:**
- Consumes: existing Guardian/service status APIs without exposing credentials.
- Produces frontend raw status matching `RawKidOSSystemStatus`.

- [ ] **Step 1: Search current shell/Tauri status commands before changing Rust**

Run code search for Guardian/service status and Tauri `invoke` calls. Reuse an existing read-only command if it already provides enforcement, filter, classifier, and recovery status.

- [ ] **Step 2: Write a failing shell test proving unsafe defaults**

Add a test that renders `ChildHome` with the status provider unavailable and asserts that neither `Safe Mode: ON` nor `Internet Filter: Active` is rendered.

- [ ] **Step 3: If the existing bridge is sufficient, map it into `RawKidOSSystemStatus`; otherwise add exactly one read-only command**

Required returned shape:

```rust
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct KidOsReadOnlyStatus {
    guardian_reachable: bool,
    guardian_enforcing: bool,
    classifier_reachable: bool,
    classifier_ready: bool,
    filter_enforcing: bool,
    recovery_available: bool,
    observed_at: u64,
}
```

The command must not accept service stop/start parameters, file paths, ACL controls, or credentials.

- [ ] **Step 4: Run shell and Rust tests**

```bash
pnpm --filter @kidos/shell test --run src/features/home/ChildHome.test.tsx
cargo test --workspace
cargo check --workspace
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add apps/shell/src/features/home/ChildHome.tsx apps/shell/src/features/home/ChildHome.test.tsx apps/shell/src-tauri/src
git commit -m "Wire read-only protection state into KidOS shell"
```

---

### Task 3: Split the large home UI into focused components

**Files:**
- Create: `apps/shell/src/features/home/KidOSHomeShell.tsx`
- Create: `apps/shell/src/features/home/KidOSSidebar.tsx`
- Create: `apps/shell/src/features/home/KidOSTopBar.tsx`
- Create: `apps/shell/src/features/home/KidOSGreeting.tsx`
- Create: `apps/shell/src/features/home/KidOSHomeGrid.tsx`
- Create: `apps/shell/src/features/home/KidOSDock.tsx`
- Create: `apps/shell/src/features/home/KidOSSafetyStatus.tsx`
- Create: `apps/shell/src/features/home/KidOSProfileCard.tsx`
- Modify: `apps/shell/src/features/home/ChildHome.tsx`
- Test: `apps/shell/src/features/home/ChildHome.test.tsx`

**Interfaces:**

```ts
export interface KidOSProfileViewModel {
  displayName: string;
  avatarUrl?: string;
  ageLabel?: string;
  levelLabel?: string;
}

export type KidOSDestination =
  | 'home' | 'apps' | 'learn' | 'create' | 'play'
  | 'watch' | 'music' | 'browser' | 'ai' | 'wellbeing';

export interface KidOSHomeActionHandlers {
  onNavigate(destination: KidOSDestination): void;
  onParentRequested(): void;
  onSafeSearch(query: string): void;
  onLaunchApprovedApp(appId: string): void;
}
```

- [ ] **Step 1: Add failing component/integration assertions**

Assert the home view exposes these eight accessible buttons exactly once: `Learn`, `Play`, `Create`, `Watch`, `Safe Browser`, `KidOS AI`, `Wellbeing`, `My Apps`. Also assert the Parent navigation action invokes the existing authorization flow instead of rendering parent content directly.

- [ ] **Step 2: Run test and verify RED**

```bash
pnpm --filter @kidos/shell test --run src/features/home/ChildHome.test.tsx
```

- [ ] **Step 3: Extract layout components without changing existing behavior**

`KidOSHomeShell` owns composition only and receives `profile`, `status`, approved dock items, and `actions` as props. Keep business/security state in `ChildHome.tsx` during this extraction.

- [ ] **Step 4: Run test and verify GREEN**

```bash
pnpm --filter @kidos/shell test --run src/features/home/ChildHome.test.tsx
```

- [ ] **Step 5: Commit**

```bash
git add apps/shell/src/features/home
git commit -m "Split KidOS home shell into focused components"
```

---

### Task 4: Implement the approved 2026 visual system

**Files:**
- Create: `apps/shell/src/features/home/kidos-shell-2026.css`
- Modify: `apps/shell/src/features/home/KidOSHomeShell.tsx`
- Modify: all new home presentation components as needed for semantic class names
- Modify: `apps/shell/src/main.tsx` only if stylesheet import belongs at the application root

**Interfaces:**
- Consumes semantic component class names.
- Produces responsive scenic KidOS shell presentation.

- [ ] **Step 1: Add structural assertions before styling**

In `ChildHome.test.tsx`, assert stable layout hooks exist: `data-testid="kidos-shell"`, `kidos-sidebar`, `kidos-home-grid`, `kidos-dock`, and `kidos-safety-status`.

- [ ] **Step 2: Add design tokens**

Use CSS custom properties at `.kidos-shell-2026`:

```css
.kidos-shell-2026 {
  --kidos-sidebar-width: clamp(184px, 14vw, 230px);
  --kidos-panel-bg: rgba(8, 31, 55, 0.72);
  --kidos-glass-bg: rgba(255, 255, 255, 0.20);
  --kidos-tile-radius: clamp(22px, 1.8vw, 30px);
  --kidos-gap: clamp(12px, 1.2vw, 22px);
  --kidos-focus: 0 0 0 4px rgba(255, 255, 255, 0.95), 0 0 0 7px rgba(31, 117, 255, 0.8);
  min-height: 100vh;
}
```

- [ ] **Step 3: Build responsive composition**

Use a fixed-width responsive sidebar and a main content grid. Keep the eight tiles in 4x2 at standard desktop widths and reduce gaps/typography before changing structure. Use `background-size: cover`, `background-position: center`, and a gradient fallback.

- [ ] **Step 4: Add keyboard focus and contrast rules**

Every interactive tile/nav/dock button must have a visible `:focus-visible` ring. No status meaning may depend only on color; include text labels for active/degraded/offline.

- [ ] **Step 5: Verify tests/build**

```bash
pnpm --filter @kidos/shell test --run
pnpm --filter @kidos/shell build
```

- [ ] **Step 6: Commit**

```bash
git add apps/shell/src/features/home/kidos-shell-2026.css apps/shell/src/features/home/*.tsx apps/shell/src/main.tsx
git commit -m "Match KidOS shell to approved 2026 visual design"
```

---

### Task 5: Wire profile, safe search, destinations, AI, Parent, and approved apps

**Files:**
- Modify: `apps/shell/src/features/home/ChildHome.tsx`
- Modify: new home components
- Modify: `apps/shell/src/features/home/ChildHome.test.tsx`
- Preserve: `tests/e2e/child-ai-create.spec.tsx`

**Interfaces:**
- `onSafeSearch(query)` must route through the existing KidOS safe-navigation path.
- `onParentRequested()` must invoke existing parent authorization.
- `onLaunchApprovedApp(appId)` must resolve from trusted approved-app state only.

- [ ] **Step 1: Write failing tests for safe action routing**

Add tests covering:

```ts
it('submits search through the safe-search action');
it('does not submit blank search');
it('requests parent authorization instead of directly opening parent tools');
it('opens KidOS AI through the existing approval-aware flow');
it('only launches an app id present in approved app state');
```

- [ ] **Step 2: Run tests and verify RED**

```bash
pnpm --filter @kidos/shell test --run src/features/home/ChildHome.test.tsx
```

- [ ] **Step 3: Wire new components to existing handlers rather than replacing security logic**

Use existing ChildHome/browser/parent functions as the implementation source. The new UI should call those functions; it must not create unrestricted URLs or executable paths.

- [ ] **Step 4: Verify child AI approval regression**

```bash
pnpm --filter @kidos/shell test --run
pnpm exec vitest --config tests/e2e/vitest.config.ts --run tests/e2e/child-ai-create.spec.tsx
```

- [ ] **Step 5: Commit**

```bash
git add apps/shell/src/features/home tests/e2e/child-ai-create.spec.tsx
git commit -m "Wire KidOS 2026 shell to safe child actions"
```

---

### Task 6: Add redesigned-home E2E coverage

**Files:**
- Create: `tests/e2e/child-home-shell.spec.tsx`
- Modify: `tests/e2e/vitest.config.ts` only if necessary for the new file pattern

**Interfaces:**
- Exercises user-visible behavior, not privileged service internals.

- [ ] **Step 1: Add E2E test cases**

Cover:

```ts
it('renders the approved eight-tile home layout');
it('navigates to each child-safe destination');
it('shows degraded protection when live status is unavailable');
it('submits safe search without opening unrestricted navigation');
it('keeps Parent behind authorization');
it('supports keyboard traversal through search, tiles, and navigation');
```

- [ ] **Step 2: Run E2E and verify failures before completing integration**

```bash
pnpm exec vitest --config tests/e2e/vitest.config.ts --run tests/e2e/child-home-shell.spec.tsx
```

- [ ] **Step 3: Make only minimal integration adjustments required by failing cases**

Do not change Guardian, classifier, or parent security semantics to satisfy UI tests.

- [ ] **Step 4: Run all E2E**

```bash
pnpm exec vitest --config tests/e2e/vitest.config.ts --run
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add tests/e2e apps/shell/src/features/home
git commit -m "Add KidOS redesigned home E2E coverage"
```

---

### Task 7: Add deterministic visual-layout regression checks

**Files:**
- Create: `tests/e2e/child-home-layout.spec.tsx` if current test stack is DOM-only.
- If an existing browser screenshot framework is already present, add the two target viewport cases there instead of introducing a new dependency.

**Interfaces:**
- Verifies layout invariants; does not replace functional tests.

- [ ] **Step 1: Check whether the repo already has Playwright/browser screenshot tooling**

Reuse existing tooling if present. Do not add a large visual testing dependency solely for this feature when deterministic DOM layout assertions are sufficient in the current CI stack.

- [ ] **Step 2: Add 1366x768 and 1920x1080 layout checks**

Verify sidebar remains visible, eight tiles remain reachable, dock does not overlap tiles, and safety status remains visible. If screenshot tooling exists, store only deterministic app-render snapshots without OS chrome.

- [ ] **Step 3: Verify both target viewport cases**

Run the repository's existing visual/browser test command, or the E2E Vitest command if using layout assertions.

- [ ] **Step 4: Commit**

```bash
git add tests/e2e
git commit -m "Guard KidOS desktop shell layout across target viewports"
```

---

### Task 8: Full JS/Rust/security regression

**Files:**
- No planned source changes; fix only proven regressions using the systematic-debugging workflow.

- [ ] **Step 1: Run workspace tests**

```bash
pnpm -r test --run
```

- [ ] **Step 2: Run workspace builds**

```bash
pnpm -r build
```

- [ ] **Step 3: Run Rust tests/checks**

```bash
cargo test --workspace
cargo check --workspace
```

- [ ] **Step 4: Run existing security/packaging contract tests**

Run the repository's existing security, Guardian, installer-hook, and packaging test commands exactly as CI defines them.

- [ ] **Step 5: Commit only if a proven regression required a fix**

Use one commit per root cause.

---

### Task 9: Windows release-gate validation

**Files:**
- No expected source changes.

- [ ] **Step 1: Push the final UI commit set to `main` and capture its exact SHA**

- [ ] **Step 2: Verify KidOS Windows Release CI on that exact SHA**

Required outcome: success.

- [ ] **Step 3: Verify Clean Windows VM on that exact SHA**

Required lifecycle: fresh install, services running, authenticated classifier health, model readiness, Guardian recovery, classifier recovery, protected ACLs, registered uninstall, clean service/task removal.

- [ ] **Step 4: Verify Windows Soak on that exact SHA**

Required outcome: success with no service crash or unrecovered failure.

- [ ] **Step 5: Verify Shadow Validation on that exact SHA**

Required outcome: success; failure injection must not produce an unsafe green UI state.

- [ ] **Step 6: Verify Windows Lockdown and Windows hardening on that exact SHA**

Required outcome: success; standard child account cannot disable or modify protected components.

- [ ] **Step 7: Record any external/manual release blockers separately**

Production code-signing certificate and physical Windows hardware validation remain external requirements if no signing identity/hardware runner is connected.

---

### Task 10: Reintroduce the approved 2026 shield icon only after shell/release stability

**Files:**
- Modify later: `apps/shell/src-tauri/icons/icon.ico`
- Test: `tests/packaging/icon-validity.test.mjs`
- Test: `tests/packaging/tauri-config.test.mjs`

**Interfaces:**
- Produces a structurally valid multi-resolution Windows ICO accepted by Tauri/NSIS.

- [ ] **Step 1: Generate a real multi-resolution ICO from the approved shield artwork**

Include at minimum 16x16, 32x32, 48x48, 64x64, 128x128, and 256x256 images in one valid ICO container.

- [ ] **Step 2: Run ICO structural tests before updating the app icon**

```bash
node --test tests/packaging/icon-validity.test.mjs tests/packaging/tauri-config.test.mjs
```

Expected: PASS.

- [ ] **Step 3: Run a Windows Tauri NSIS build**

```bash
pnpm --filter @kidos/shell tauri build --bundles nsis
```

Expected: icon decodes successfully and installer is produced.

- [ ] **Step 4: Commit icon separately**

```bash
git add apps/shell/src-tauri/icons/icon.ico tests/packaging
git commit -m "Install validated KidOS 2026 Windows icon"
```

---

## Plan Self-Review

- **Spec coverage:** The plan covers shell decomposition, approved layout, live authoritative status, stale-status handling, Guardian/classifier boundaries, profile data, safe search, AI approval, approved apps, Parent authorization, accessibility, responsive desktop targets, component/E2E/layout tests, Windows release gates, and deferred validated ICO replacement.
- **Placeholder scan:** No `TBD`, `TODO`, `implement later`, or unspecified generic error-handling steps are used. Conditional bridge/visual-tooling steps explicitly require reuse of existing repository capability before adding anything.
- **Type consistency:** `RawKidOSSystemStatus`, `KidOSSystemStatus`, `ProtectionState`, `KidOSProfileViewModel`, `KidOSDestination`, and `KidOSHomeActionHandlers` are defined once and referenced consistently across tasks.
- **Security consistency:** No task grants React privileged authority, exposes classifier credentials, adds unrestricted navigation, adds arbitrary executable launch, or weakens Parent authorization.
