# KidOS Windows v1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first working Windows vertical slice of KidOS: a Tauri/React child shell with parent setup, age-based policy decisions, creation workspaces, protected navigation decisions, secure parent authorization, and a Rust Guardian Service boundary.

**Architecture:** The UI is an unprivileged Tauri 2 + React shell. Security-sensitive decisions live in a pure Rust `policy-core` crate, while privileged Windows operations are isolated in `guardian-service`; TypeScript contracts and the creation engine remain independently testable. The first milestone uses deterministic local policy and mockable adapters before adding external reputation/classification services.

**Tech Stack:** Tauri 2, React, TypeScript, Vite, Rust, Cargo workspace, Zod, SQLite, Vitest, React Testing Library, Windows DPAPI/Credential Manager abstraction, WebView2/Tauri webview APIs.

**Spec:** `docs/superpowers/specs/2026-09-02-kidos-windows-v1-design.md`

## Global Constraints

- Windows-first downloadable application.
- Child-facing renderer processes never receive administrator privileges.
- Policy decisions are exactly `allow`, `block`, or `require_parent`.
- Unknown high-risk actions fail closed or require parent approval.
- Parent secrets are never stored in plaintext configuration.
- Full child search history is not retained as a long-term browsing database by default.
- AI/creation orchestration cannot grant capabilities outside the active child policy.
- Guardian Service failure results in restricted safe mode, not privilege relaxation.
- No unsupported claim that KidOS can guarantee perfect filtering or legal compliance.

---

## File Structure

```text
KidOS/
  Cargo.toml
  package.json
  pnpm-workspace.yaml
  apps/
    shell/
      package.json
      vite.config.ts
      src/
        main.tsx
        App.tsx
        app.css
        lib/kidos-api.ts
        features/onboarding/ParentSetup.tsx
        features/home/ChildHome.tsx
        features/home/CreationCommand.tsx
        features/workspaces/WorkspaceView.tsx
        features/browser/ProtectedBrowser.tsx
        features/parent/ParentDashboard.tsx
      src-tauri/
        Cargo.toml
        tauri.conf.json
        src/lib.rs
        src/main.rs
        src/commands.rs
  packages/
    contracts/
      package.json
      src/index.ts
      src/policy.ts
      src/workspaces.ts
    creation-engine/
      package.json
      src/index.ts
      src/planWorkspace.ts
      src/planWorkspace.test.ts
  crates/
    policy-core/
      Cargo.toml
      src/lib.rs
      src/model.rs
      src/evaluate.rs
      tests/policy_decisions.rs
    secure-store/
      Cargo.toml
      src/lib.rs
      src/memory.rs
      src/windows.rs
      tests/secure_store.rs
    guardian-service/
      Cargo.toml
      src/lib.rs
      src/ipc.rs
      src/policy_store.rs
      src/service_state.rs
      tests/guardian_state.rs
  docs/
    safety/threat-model.md
  tests/
    e2e/
```

## Task 1: Bootstrap the KidOS monorepo and desktop shell

**Files:**
- Create: `package.json`
- Create: `pnpm-workspace.yaml`
- Create: `Cargo.toml`
- Create: `apps/shell/package.json`
- Create: `apps/shell/vite.config.ts`
- Create: `apps/shell/src/main.tsx`
- Create: `apps/shell/src/App.tsx`
- Create: `apps/shell/src/app.css`
- Create: `apps/shell/src-tauri/Cargo.toml`
- Create: `apps/shell/src-tauri/tauri.conf.json`
- Create: `apps/shell/src-tauri/src/main.rs`
- Create: `apps/shell/src-tauri/src/lib.rs`

**Interfaces:**
- Produces: runnable `@kidos/shell` React/Tauri application.
- Produces: Cargo workspace members for later Rust crates.

- [ ] **Step 1: Add a failing shell smoke test**

Create `apps/shell/src/App.test.tsx`:

```tsx
import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import App from './App';

describe('KidOS shell', () => {
  it('shows the creation-first prompt', () => {
    render(<App />);
    expect(screen.getByText('What do you want to create?')).toBeTruthy();
  });
});
```

- [ ] **Step 2: Run the test and confirm it fails before implementation**

Run:

```bash
pnpm --filter @kidos/shell test --run
```

Expected: FAIL because the shell package/App does not exist yet.

- [ ] **Step 3: Implement the minimal React/Tauri shell**

`apps/shell/src/App.tsx`:

```tsx
export default function App() {
  return (
    <main className="kidos-shell">
      <section className="hero">
        <p className="eyebrow">KidOS</p>
        <h1>What do you want to create?</h1>
        <p>Tell KidOS what you want to make and it will build a safe workspace for you.</p>
      </section>
    </main>
  );
}
```

`apps/shell/src/main.tsx`:

```tsx
import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import './app.css';

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode><App /></React.StrictMode>,
);
```

Use Tauri 2 with `apps/shell` as the frontend distribution source and keep all shell renderer code unprivileged.

- [ ] **Step 4: Run unit/build checks**

```bash
pnpm --filter @kidos/shell test --run
pnpm --filter @kidos/shell build
cargo check --workspace
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add package.json pnpm-workspace.yaml Cargo.toml apps/shell
git commit -m "feat: bootstrap KidOS desktop shell"
```

## Task 2: Define shared policy and workspace contracts

**Files:**
- Create: `packages/contracts/package.json`
- Create: `packages/contracts/src/policy.ts`
- Create: `packages/contracts/src/workspaces.ts`
- Create: `packages/contracts/src/index.ts`
- Test: `packages/contracts/src/policy.test.ts`

**Interfaces:**
- Produces: `PolicyDecision = 'allow' | 'block' | 'require_parent'`.
- Produces: `ChildProfile`, `NavigationRequest`, `DownloadRequest`, `WorkspacePlan`, `CapabilityId`.

- [ ] **Step 1: Write failing contract tests**

```ts
import { describe, expect, it } from 'vitest';
import { ChildProfileSchema, PolicyDecisionSchema } from './policy';

describe('policy contracts', () => {
  it('rejects policy outcomes outside the three supported decisions', () => {
    expect(PolicyDecisionSchema.safeParse('skip').success).toBe(false);
  });

  it('requires a child age from 3 through 17', () => {
    expect(ChildProfileSchema.safeParse({ id: 'kid-1', displayName: 'Ari', age: 10 }).success).toBe(true);
    expect(ChildProfileSchema.safeParse({ id: 'kid-2', displayName: 'Ari', age: 18 }).success).toBe(false);
  });
});
```

- [ ] **Step 2: Verify failure**

```bash
pnpm --filter @kidos/contracts test --run
```

Expected: FAIL because schemas are undefined.

- [ ] **Step 3: Implement schemas and exported TypeScript types**

Use Zod and define:

```ts
export const PolicyDecisionSchema = z.enum(['allow', 'block', 'require_parent']);
export type PolicyDecision = z.infer<typeof PolicyDecisionSchema>;

export const ChildProfileSchema = z.object({
  id: z.string().min(1),
  displayName: z.string().min(1).max(40),
  age: z.number().int().min(3).max(17),
});
```

Also define navigation/download/workspace schemas with explicit capability IDs: `story`, `drawing_presentation`, `beginner_coding`, `protected_web`, `audio_recording`, and `export_project`.

- [ ] **Step 4: Run tests and typecheck**

```bash
pnpm --filter @kidos/contracts test --run
pnpm --filter @kidos/contracts typecheck
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add packages/contracts
git commit -m "feat: define KidOS shared contracts"
```

## Task 3: Build the Rust policy engine first

**Files:**
- Create: `crates/policy-core/Cargo.toml`
- Create: `crates/policy-core/src/lib.rs`
- Create: `crates/policy-core/src/model.rs`
- Create: `crates/policy-core/src/evaluate.rs`
- Test: `crates/policy-core/tests/policy_decisions.rs`

**Interfaces:**
- Consumes: domain/action category, child age/profile flags, parent overrides.
- Produces: `evaluate_navigation(&NavigationContext) -> PolicyDecision`.
- Produces: `evaluate_download(&DownloadContext) -> PolicyDecision`.

- [ ] **Step 1: Write failing Rust policy tests**

```rust
#[test]
fn blocks_explicit_parent_denied_domain() {
    let ctx = NavigationContext::new("example-bad.test", 10)
        .with_parent_blocked(true);
    assert_eq!(evaluate_navigation(&ctx), PolicyDecision::Block);
}

#[test]
fn unknown_high_risk_navigation_requires_parent() {
    let ctx = NavigationContext::new("unknown.test", 10)
        .with_category(SiteCategory::Unknown)
        .with_risk(RiskLevel::High);
    assert_eq!(evaluate_navigation(&ctx), PolicyDecision::RequireParent);
}

#[test]
fn executable_download_is_parent_gated() {
    let ctx = DownloadContext::new("setup.exe", "application/vnd.microsoft.portable-executable", 12);
    assert_eq!(evaluate_download(&ctx), PolicyDecision::RequireParent);
}
```

- [ ] **Step 2: Run and verify failure**

```bash
cargo test -p policy-core
```

Expected: FAIL because `policy-core` is not implemented.

- [ ] **Step 3: Implement deterministic precedence**

Implement this order exactly:

```text
1. explicit parent block -> block
2. explicit parent allow + not intrinsically high-risk action -> allow
3. known prohibited category for profile -> block
4. executable/script/installer download -> require_parent
5. high-risk unknown -> require_parent
6. ordinary unknown navigation -> require_parent for ages 3-12, allow only when teen profile explicitly enables unknown-web access
7. known approved/educational destination -> allow
8. fallback -> require_parent
```

Keep this crate pure: no network, filesystem, UI, or Windows API calls.

- [ ] **Step 4: Run policy tests**

```bash
cargo test -p policy-core
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/policy-core Cargo.toml
git commit -m "feat: add fail-closed KidOS policy engine"
```

## Task 4: Build the creation orchestrator and three starter workspaces

**Files:**
- Create: `packages/creation-engine/package.json`
- Create: `packages/creation-engine/src/index.ts`
- Create: `packages/creation-engine/src/planWorkspace.ts`
- Test: `packages/creation-engine/src/planWorkspace.test.ts`

**Interfaces:**
- Consumes: `planWorkspace(prompt: string, profile: ChildProfile, allowedCapabilities: CapabilityId[]): WorkspacePlan`.
- Produces: workspace type plus only allowed capabilities.

- [ ] **Step 1: Write failing tests**

```ts
it('maps a story request to the story workspace', () => {
  const plan = planWorkspace('write a superhero story', profile, ['story', 'export_project']);
  expect(plan.kind).toBe('story');
});

it('never adds a capability excluded by policy', () => {
  const plan = planWorkspace('make a coding game', profile, ['story']);
  expect(plan.capabilities).toEqual(['story']);
});
```

- [ ] **Step 2: Verify failure**

```bash
pnpm --filter @kidos/creation-engine test --run
```

- [ ] **Step 3: Implement deterministic local intent mapping**

For v1, use a local deterministic classifier before any remote AI integration:

```ts
const intents = [
  { kind: 'beginner_coding', terms: ['code', 'coding', 'game', 'program'] },
  { kind: 'drawing_presentation', terms: ['draw', 'picture', 'poster', 'presentation', 'slides', 'cartoon'] },
  { kind: 'story', terms: ['story', 'write', 'book', 'poem', 'script'] },
] as const;
```

Select the first matching workspace and intersect requested capabilities with the provided allowlist. Default to `story` with no elevated capability.

- [ ] **Step 4: Run tests**

```bash
pnpm --filter @kidos/creation-engine test --run
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add packages/creation-engine
git commit -m "feat: add safe creation workspace planner"
```

## Task 5: Add parent setup and secure parent authorization abstraction

**Files:**
- Create: `crates/secure-store/Cargo.toml`
- Create: `crates/secure-store/src/lib.rs`
- Create: `crates/secure-store/src/memory.rs`
- Create: `crates/secure-store/src/windows.rs`
- Test: `crates/secure-store/tests/secure_store.rs`
- Create: `apps/shell/src/features/onboarding/ParentSetup.tsx`

**Interfaces:**
- Produces Rust trait: `SecretStore { put_secret, verify_secret, delete_secret }`.
- Produces parent PIN record as salted password verifier or OS-protected secret; never plaintext.

- [ ] **Step 1: Write a failing secure-store contract test**

```rust
#[test]
fn parent_pin_is_verified_without_plaintext_round_trip() {
    let store = MemorySecretStore::default();
    store.put_secret("parent-pin", "4821").unwrap();
    assert!(store.verify_secret("parent-pin", "4821").unwrap());
    assert!(!store.verify_secret("parent-pin", "1111").unwrap());
}
```

- [ ] **Step 2: Verify failure**

```bash
cargo test -p secure-store
```

- [ ] **Step 3: Implement the trait, test adapter, and Windows adapter boundary**

The memory adapter stores only a salted Argon2id verifier for tests. The Windows adapter exposes the same interface and protects persisted material with DPAPI/Credential Manager. Do not create a plaintext fallback. Add rate-limit state for PIN verification at the command/service layer: 5 failed attempts -> 60 second lockout.

- [ ] **Step 4: Add parent setup UI**

`ParentSetup` collects parent PIN twice, validates 4-8 digits for v1, calls a Tauri command `configure_parent_pin`, and never writes the PIN into localStorage/sessionStorage.

- [ ] **Step 5: Run checks**

```bash
cargo test -p secure-store
pnpm --filter @kidos/shell test --run
```

- [ ] **Step 6: Commit**

```bash
git add crates/secure-store apps/shell/src/features/onboarding
git commit -m "feat: add protected parent authorization setup"
```

## Task 6: Add Guardian Service state boundary and safe-mode behavior

**Files:**
- Create: `crates/guardian-service/Cargo.toml`
- Create: `crates/guardian-service/src/lib.rs`
- Create: `crates/guardian-service/src/ipc.rs`
- Create: `crates/guardian-service/src/policy_store.rs`
- Create: `crates/guardian-service/src/service_state.rs`
- Test: `crates/guardian-service/tests/guardian_state.rs`

**Interfaces:**
- Produces: `GuardianState::Healthy | RestrictedSafeMode`.
- Produces: schema-validated request enum for policy evaluation and parent-authorized privileged operations.
- Consumes: `policy-core`.

- [ ] **Step 1: Write failing service tests**

```rust
#[test]
fn missing_valid_policy_enters_restricted_safe_mode() {
    let state = load_service_state(None, None);
    assert_eq!(state.mode, GuardianMode::RestrictedSafeMode);
}

#[test]
fn malformed_ipc_is_rejected() {
    assert!(decode_request(br#"{\"type\":\"shell\",\"command\":123}"#).is_err());
}
```

- [ ] **Step 2: Verify failure**

```bash
cargo test -p guardian-service
```

- [ ] **Step 3: Implement safe-mode and authenticated-message structure**

Use a versioned request envelope:

```rust
struct RequestEnvelope {
    version: u16,
    session_id: String,
    nonce: String,
    request: GuardianRequest,
}
```

Reject unknown versions, duplicate nonces within the active session, malformed schemas, and requests unavailable in child mode. The first milestone may use an in-process transport adapter for tests, but the interface must not expose arbitrary command execution.

- [ ] **Step 4: Implement last-known-valid policy loading**

Load current integrity-validated policy; otherwise last-known-valid policy; otherwise strict built-in baseline and `RestrictedSafeMode`.

- [ ] **Step 5: Run tests**

```bash
cargo test -p guardian-service
cargo test --workspace
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/guardian-service
git commit -m "feat: add Guardian Service safety boundary"
```

## Task 7: Connect shell commands to policy and creation flows

**Files:**
- Create: `apps/shell/src-tauri/src/commands.rs`
- Modify: `apps/shell/src-tauri/src/lib.rs`
- Create: `apps/shell/src/lib/kidos-api.ts`
- Create: `apps/shell/src/features/home/CreationCommand.tsx`
- Create: `apps/shell/src/features/home/ChildHome.tsx`
- Create: `apps/shell/src/features/workspaces/WorkspaceView.tsx`
- Modify: `apps/shell/src/App.tsx`

**Interfaces:**
- Produces Tauri commands: `plan_workspace`, `evaluate_navigation`, `evaluate_download`, `get_guardian_status`.
- Child UI receives typed results only; no arbitrary Rust command passthrough.

- [ ] **Step 1: Write failing UI integration tests**

Test that entering `make a story about space` renders the Story workspace, and that a `require_parent` navigation result shows a parent approval screen rather than navigating.

- [ ] **Step 2: Verify tests fail**

```bash
pnpm --filter @kidos/shell test --run
```

- [ ] **Step 3: Implement a narrow `kidos-api.ts` adapter**

```ts
export interface KidOSApi {
  planWorkspace(prompt: string): Promise<WorkspacePlan>;
  evaluateNavigation(url: string): Promise<PolicyDecision>;
  guardianStatus(): Promise<'healthy' | 'restricted_safe_mode'>;
}
```

The production implementation invokes named Tauri commands. Tests inject a fake implementation.

- [ ] **Step 4: Implement child home and starter workspace view**

Render only capabilities returned in `WorkspacePlan`. Do not expose a generic “run command” or unrestricted filesystem UI.

- [ ] **Step 5: Run tests/build**

```bash
pnpm --filter @kidos/shell test --run
pnpm --filter @kidos/shell build
cargo test --workspace
```

- [ ] **Step 6: Commit**

```bash
git add apps/shell
git commit -m "feat: connect KidOS shell to protected core flows"
```

## Task 8: Add protected navigation and SafeSearch URL rewriting

**Files:**
- Create: `apps/shell/src/features/browser/ProtectedBrowser.tsx`
- Create: `packages/contracts/src/browser.ts`
- Add test: `apps/shell/src/features/browser/ProtectedBrowser.test.tsx`
- Modify: `crates/policy-core/src/evaluate.rs`

**Interfaces:**
- Consumes: typed navigation request.
- Produces: sanitized destination or blocked/parent-required state.
- Produces: `apply_safe_search(url) -> sanitized_url` for explicitly supported providers.

- [ ] **Step 1: Write failing tests**

Cover:

```text
Google search -> ensure safe=active
Bing search -> ensure adlt=strict
YouTube search/watch -> restricted handling adapter enabled where platform supports it
blocked domain -> never load destination
require_parent -> display approval gate
```

- [ ] **Step 2: Verify failure**

```bash
pnpm --filter @kidos/shell test --run
cargo test -p policy-core
```

- [ ] **Step 3: Implement provider-specific URL sanitizers and navigation gate**

Only rewrite providers explicitly recognized by KidOS. Do not claim a provider control exists where it does not. Resolve redirects through the same navigation gate before final display.

- [ ] **Step 4: Run tests**

```bash
pnpm --filter @kidos/shell test --run
cargo test -p policy-core
```

- [ ] **Step 5: Commit**

```bash
git add apps/shell/src/features/browser packages/contracts crates/policy-core
git commit -m "feat: add protected browsing gate"
```

## Task 9: Add download policy and parent dashboard controls

**Files:**
- Create: `apps/shell/src/features/parent/ParentDashboard.tsx`
- Modify: `packages/contracts/src/policy.ts`
- Modify: `crates/policy-core/src/evaluate.rs`
- Add tests: `apps/shell/src/features/parent/ParentDashboard.test.tsx`
- Add tests: `crates/policy-core/tests/policy_decisions.rs`

**Interfaces:**
- Parent config fields: age profile, blocked domains, allowed domains, unknown-web toggle for teen profiles, social platform status/time windows, download mode.
- Download mode: `block_high_risk | require_parent_high_risk`.

- [ ] **Step 1: Write failing tests for override precedence and file risk**

Include disguised high-risk filenames such as `game.exe.zip` and double extensions such as `photo.jpg.exe`; classify using extension + MIME + archive inspection metadata rather than filename display text alone.

- [ ] **Step 2: Write failing parent dashboard tests**

Verify child mode cannot open/save parent settings without successful parent authorization.

- [ ] **Step 3: Implement parent policy editor and download rules**

Save typed policy through Guardian Service only after PIN authorization. Shell UI never writes privileged policy directly.

- [ ] **Step 4: Run all checks**

```bash
pnpm -r test --run
cargo test --workspace
```

- [ ] **Step 5: Commit**

```bash
git add apps/shell/src/features/parent packages/contracts crates/policy-core
git commit -m "feat: add parent policy and download protection"
```

## Task 10: Add minimal safety-event storage without browsing-history retention

**Files:**
- Create: `crates/guardian-service/src/safety_events.rs`
- Modify: `crates/guardian-service/src/lib.rs`
- Modify: `apps/shell/src/features/parent/ParentDashboard.tsx`
- Test: `crates/guardian-service/tests/safety_events.rs`

**Interfaces:**
- Stores only: timestamp, action class, normalized domain when needed, decision, reason code.
- Does not store full search query, page text, parent PIN, auth token, or arbitrary child prompt content.

- [ ] **Step 1: Write failing redaction/storage tests**

```rust
#[test]
fn event_record_excludes_query_text_and_secrets() {
    let event = SafetyEvent::blocked_navigation("https://example.test/search?q=private words");
    let row = event.to_record();
    assert_eq!(row.domain.as_deref(), Some("example.test"));
    assert!(!format!("{row:?}").contains("private words"));
}
```

- [ ] **Step 2: Implement SQLite-backed event repository**

Use parameterized SQL and a bounded retention configuration. Add `clear_events()` callable only from parent-authorized flow.

- [ ] **Step 3: Add parent summary UI**

Display reason-level summaries such as “3 blocked unsafe downloads” rather than a surveillance-style full browsing timeline.

- [ ] **Step 4: Run tests**

```bash
cargo test -p guardian-service
pnpm --filter @kidos/shell test --run
```

- [ ] **Step 5: Commit**

```bash
git add crates/guardian-service apps/shell/src/features/parent
git commit -m "feat: add privacy-minimized safety events"
```

## Task 11: Add Windows installer/service packaging and threat-model documentation

**Files:**
- Modify: `apps/shell/src-tauri/tauri.conf.json`
- Create: `docs/safety/threat-model.md`
- Create: `.github/workflows/ci.yml`

**Interfaces:**
- Produces Windows installer bundle for shell.
- Documents Guardian Service installation boundary and service identity.
- CI runs TypeScript and Rust test suites.

- [ ] **Step 1: Add CI that initially fails on the incomplete packaging configuration**

CI commands:

```yaml
- run: corepack enable
- run: pnpm install --frozen-lockfile
- run: pnpm -r test --run
- run: pnpm -r build
- run: cargo test --workspace
- run: cargo check --workspace
```

- [ ] **Step 2: Configure Tauri Windows bundle metadata**

Set product name `KidOS`, unique Windows identifier, version, icon paths, updater disabled until signing/update infrastructure is intentionally designed, and least-privilege defaults. Do not auto-elevate the renderer.

- [ ] **Step 3: Document the threat model**

`docs/safety/threat-model.md` must cover assets, trust boundaries, child-user bypass attempts, malicious websites, hostile downloads, renderer compromise, local config tampering, PIN brute force, IPC spoofing/replay, Guardian failure, and explicit non-goals.

- [ ] **Step 4: Run release verification**

```bash
pnpm -r test --run
pnpm -r build
cargo test --workspace
cargo check --workspace
pnpm --filter @kidos/shell tauri build
```

Expected: installer bundle created on a Windows runner/developer machine; all tests PASS.

- [ ] **Step 5: Commit**

```bash
git add .github apps/shell/src-tauri/tauri.conf.json docs/safety/threat-model.md
git commit -m "build: package and verify KidOS Windows vertical slice"
```

## Task 12: End-to-end vertical-slice verification

**Files:**
- Create: `tests/e2e/child-creation.spec.ts`
- Create: `tests/e2e/parent-approval.spec.ts`
- Create: `tests/e2e/protected-browser.spec.ts`
- Create: `tests/e2e/restricted-safe-mode.spec.ts`

**Interfaces:**
- Verifies the spec's first household workflow from setup through creation and protected browsing.

- [ ] **Step 1: Add end-to-end tests for the four critical flows**

Required cases:

```text
1. Parent configures profile -> child launches -> creates Story workspace.
2. Child attempts parent-gated unknown/high-risk navigation -> parent approves -> approved action proceeds once.
3. Child attempts blocked domain/high-risk download -> action remains blocked.
4. Guardian unavailable -> UI enters restricted safe mode and does not relax web/privileged policy.
```

- [ ] **Step 2: Run them against the packaged/test desktop harness**

```bash
pnpm test:e2e
```

Expected: PASS for all four critical flows.

- [ ] **Step 3: Run full verification suite**

```bash
pnpm -r test --run
pnpm -r build
cargo test --workspace
cargo check --workspace
pnpm test:e2e
```

- [ ] **Step 4: Confirm prohibited behaviors are absent**

Repository scan must show no child renderer `Command::new`/shell execution bridge, no plaintext PIN persistence, no generic arbitrary Tauri command dispatcher, and no full search-history storage table.

- [ ] **Step 5: Commit**

```bash
git add tests/e2e
git commit -m "test: verify KidOS Windows vertical slice"
```

## Self-review

### Spec coverage

- Windows shell/install path: Tasks 1 and 11.
- Child creation-first UI: Tasks 1, 4, and 7.
- Parent PIN/controls: Tasks 5 and 9.
- Age/policy engine: Tasks 2 and 3.
- Protected browser/SafeSearch: Task 8.
- Download controls: Task 9.
- Guardian boundary/safe mode: Task 6.
- Minimal safety-event storage/privacy: Task 10.
- Three starter workspaces: Tasks 4 and 7.
- Security/threat testing: Tasks 3, 5, 6, 8-12.

### Placeholder scan

No implementation step relies on `TBD`, `TODO`, “implement later,” or undefined generic error-handling instructions. External reputation/classification services are deliberately outside this first vertical slice; adapters can be added after deterministic local policy is verified.

### Type consistency

- Policy decision: `allow | block | require_parent` across Rust/TypeScript contracts.
- Guardian state: `healthy | restricted_safe_mode` at the shell boundary.
- Creation workspace kinds: `story | drawing_presentation | beginner_coding`.
- Parent authorization remains separate from child content and policy evaluation.
