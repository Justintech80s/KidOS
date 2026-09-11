# KidOS Production UI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the approved KidOS mockup into the actual installed Windows child interface while preserving the current Guardian, Safe Browser, Parent PIN, classifier, recovery, and release-validation boundaries.

**Architecture:** Keep `KidOSHomeShell.tsx` as the coordinator for destination state and trusted actions, but move each destination into a focused presentational screen component. Reuse the existing `KidOSApi`, `prepareProtectedNavigation`, system-status adapter, sidebar, top bar, dock, grid, and safety components. Styling stays local to `kidos-shell-2026.css`; no new backend privilege or arbitrary executable-launch API is added.

**Tech Stack:** React 19, TypeScript 5.9, Vite 7, Vitest 3, Testing Library 16, Tauri 2, existing KidOS Rust/Windows services, pnpm 10.

**Spec:** `docs/superpowers/specs/2026-09-10-kidos-production-ui-design.md`

## Global Constraints

- KidOS Guardian remains authoritative; React is presentation only.
- Safe Browser must continue through `prepareProtectedNavigation(...)` and `api.evaluateNavigation(...)` before `api.openProtectedBrowser(...)`.
- Missing telemetry must never render a green/active safety state.
- Parent controls remain behind `api.verifyParentPin(...)`.
- My Apps must not accept or execute arbitrary executable paths.
- Classifier credentials remain outside browser JavaScript.
- No remote image dependency is required for the shell to render.
- Preserve the eight primary child destinations exactly: Learn, Play, Create, Watch, Safe Browser, KidOS AI, Wellbeing, My Apps.
- Preserve the standard 4x2 home grid, two-column compact fallback, 1366x768 compression rule, 1920x1080 layout, and visible focus treatment.
- Existing Task 12 E2E, Windows Lockdown, Clean Windows VM, Windows Soak, Shadow Validation, Free Windows Hardening, Windows Release, and other triggered release/security checks remain blockers.

---

## File Structure

### Existing files to modify

- `apps/shell/src/features/home/KidOSHomeShell.tsx` — coordinator only; selects the active destination and passes safe callbacks/state into screens.
- `apps/shell/src/features/home/KidOSSidebar.tsx` — add explicit Parent navigation state/callback behavior while preserving accessibility.
- `apps/shell/src/features/home/KidOSHomeGrid.tsx` — retain exactly eight tiles and approved labels/copy.
- `apps/shell/src/features/home/KidOSSafetyStatus.tsx` — retain truthful active/degraded/offline labels.
- `apps/shell/src/features/home/KidOSGreeting.tsx` — visual refinement only.
- `apps/shell/src/features/home/KidOSProfileCard.tsx` — visual refinement only.
- `apps/shell/src/features/home/KidOSTopBar.tsx` — preserve safe global search; visual refinement only.
- `apps/shell/src/features/home/KidOSDock.tsx` — preserve safe destination navigation.
- `apps/shell/src/features/home/kidos-shell-2026.css` — production design system and responsive layout.
- `apps/shell/src/features/home/KidOSHomeShell.test.tsx` — component behavior coverage.
- `apps/shell/src/features/home/kidos-shell-2026-layout.test.ts` — layout contract coverage.
- `tests/e2e/kidos-home-shell.spec.tsx` — end-to-end vertical slice coverage.
- `apps/shell/src/App.test.tsx` — keep Guardian-before-shell boundary intact.

### New focused screen files

- `apps/shell/src/features/home/KidOSHomeScreen.tsx`
- `apps/shell/src/features/home/KidOSSafeBrowserScreen.tsx`
- `apps/shell/src/features/home/KidOSParentAccess.tsx`
- `apps/shell/src/features/home/KidOSLearnScreen.tsx`
- `apps/shell/src/features/home/KidOSPlayScreen.tsx`
- `apps/shell/src/features/home/KidOSCreateScreen.tsx`
- `apps/shell/src/features/home/KidOSWatchScreen.tsx`
- `apps/shell/src/features/home/KidOSAiScreen.tsx`
- `apps/shell/src/features/home/KidOSWellbeingScreen.tsx`
- `apps/shell/src/features/home/KidOSMyAppsScreen.tsx`
- `apps/shell/src/features/home/KidOSModulePrimitives.tsx` — small reusable module header/card/status primitives only.

---

### Task 1: Extract the home and destination screen boundaries without changing behavior

**Files:**
- Create: `apps/shell/src/features/home/KidOSHomeScreen.tsx`
- Create: `apps/shell/src/features/home/KidOSLearnScreen.tsx`
- Create: `apps/shell/src/features/home/KidOSPlayScreen.tsx`
- Create: `apps/shell/src/features/home/KidOSWatchScreen.tsx`
- Create: `apps/shell/src/features/home/KidOSWellbeingScreen.tsx`
- Create: `apps/shell/src/features/home/KidOSMyAppsScreen.tsx`
- Modify: `apps/shell/src/features/home/KidOSHomeShell.tsx`
- Test: `apps/shell/src/features/home/KidOSHomeShell.test.tsx`

**Interfaces:**
- Consumes: `KidOSSystemStatus`, `KidOSDestination`, `onNavigate(destination)`.
- Produces:

```ts
export interface KidOSHomeScreenProps {
  status: KidOSSystemStatus;
  onNavigate(destination: KidOSDestination): void;
}
```

- [ ] **Step 1: Add failing navigation tests for all eight destinations**

Add this pattern to `KidOSHomeShell.test.tsx`:

```ts
it.each([
  ['Learn', 'Learn'],
  ['Play', 'Play'],
  ['Create', 'Create'],
  ['Watch', 'Watch'],
  ['Safe Browser', 'Safe Browser'],
  ['KidOS AI', 'KidOS AI'],
  ['Wellbeing', 'Wellbeing'],
  ['My Apps', 'My Apps'],
])('opens the %s production screen', async (buttonName, heading) => {
  render(<KidOSHomeShell api={api} onOpenParentWorkspace={() => undefined} />);
  fireEvent.click(screen.getByRole('button', { name: new RegExp(buttonName) }));
  expect(await screen.findByRole('heading', { name: heading })).toBeTruthy();
});
```

- [ ] **Step 2: Run focused test and verify RED**

```bash
pnpm --filter @kidos/shell test --run src/features/home/KidOSHomeShell.test.tsx
```

Expected: at least one destination assertion fails while the shell still uses inline generic modules.

- [ ] **Step 3: Create `KidOSHomeScreen.tsx`**

```tsx
import type { KidOSDestination } from './KidOSSidebar';
import type { KidOSSystemStatus } from './system-status';
import KidOSGreeting from './KidOSGreeting';
import KidOSHomeGrid from './KidOSHomeGrid';
import KidOSSafetyStatus from './KidOSSafetyStatus';

export default function KidOSHomeScreen({ status, onNavigate }: {
  status: KidOSSystemStatus;
  onNavigate(destination: KidOSDestination): void;
}) {
  return (
    <section className="kidos-screen kidos-home-screen" aria-label="KidOS Home">
      <KidOSGreeting />
      <KidOSHomeGrid onNavigate={onNavigate} />
      <KidOSSafetyStatus status={status} />
    </section>
  );
}
```

- [ ] **Step 4: Create the safe static category screens**

Use focused files with no privileged actions. Example `KidOSLearnScreen.tsx`:

```tsx
export default function KidOSLearnScreen() {
  return (
    <section className="kidos-screen kidos-category-screen" data-testid="kidos-learn-screen">
      <header className="kidos-screen-header">
        <span className="kidos-screen-icon" aria-hidden="true">📘</span>
        <div><p className="eyebrow">Explore safely</p><h1>Learn</h1><p>Choose a subject and keep growing.</p></div>
      </header>
      <div className="kidos-category-grid">
        {['Math', 'Science', 'Reading', 'Homework'].map((label) => (
          <article className="kidos-category-card" key={label}><strong>{label}</strong><span>Parent-approved learning tools</span></article>
        ))}
      </div>
    </section>
  );
}
```

For `Play`, `Watch`, `Wellbeing`, and `My Apps`, use the same structural classes but copy specific to each module. `My Apps` must state that only parent-approved apps are available and must contain no path input, file picker, or process-launch control.

- [ ] **Step 5: Replace the inline destination JSX in `KidOSHomeShell.tsx` with component selection**

Use explicit branches or a `switch` and preserve coordinator-owned state/actions. Do not move Guardian polling or policy decisions into child components.

- [ ] **Step 6: Run focused shell tests and build**

```bash
pnpm --filter @kidos/shell test --run src/features/home/KidOSHomeShell.test.tsx
pnpm --filter @kidos/shell build
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add apps/shell/src/features/home
 git commit -m "refactor: split KidOS production destination screens"
```

---

### Task 2: Build the dedicated Safe Browser production screen

**Files:**
- Create: `apps/shell/src/features/home/KidOSSafeBrowserScreen.tsx`
- Modify: `apps/shell/src/features/home/KidOSHomeShell.tsx`
- Test: `apps/shell/src/features/home/KidOSHomeShell.test.tsx`
- Test: `tests/e2e/kidos-home-shell.spec.tsx`

**Interfaces:**

```ts
export interface KidOSSafeBrowserScreenProps {
  value: string;
  statusMessage: string;
  protectionStatus: KidOSSystemStatus;
  onValueChange(value: string): void;
  onSubmit(): void;
  onShortcut(query: string): void;
}
```

- [ ] **Step 1: Add failing tests for browser visual behavior and fail-closed states**

Add component tests covering `allow`, `block`, `require_parent`, and missing `openProtectedBrowser`. Preserve the existing Google `safe=active` assertion.

Example unavailable case:

```ts
it('does not open anything when the protected browser bridge is unavailable', async () => {
  const allowWithoutBrowser = { ...api, evaluateNavigation: async () => 'allow' as const };
  render(<KidOSHomeShell api={allowWithoutBrowser} onOpenParentWorkspace={() => undefined} />);
  fireEvent.change(screen.getByLabelText('Search KidOS safely'), { target: { value: 'planets' } });
  fireEvent.submit(screen.getByLabelText('Search KidOS safely').closest('form')!);
  expect(await screen.findByText(/Safe browser is unavailable/)).toBeTruthy();
});
```

- [ ] **Step 2: Run tests and verify RED**

```bash
pnpm --filter @kidos/shell test --run src/features/home/KidOSHomeShell.test.tsx
```

- [ ] **Step 3: Create `KidOSSafeBrowserScreen.tsx`**

```tsx
const shortcuts = [
  ['Learning', 'learning for kids'],
  ['Science', 'science for kids'],
  ['Animals', 'animals for kids'],
  ['Space', 'space for kids'],
  ['Arts & Crafts', 'arts and crafts for kids'],
  ['Math', 'math for kids'],
] as const;

export default function KidOSSafeBrowserScreen(props: KidOSSafeBrowserScreenProps) {
  return (
    <section className="kidos-screen kidos-browser-screen" data-testid="kidos-browser-screen">
      <header className="kidos-screen-header">
        <span className="kidos-screen-icon" aria-hidden="true">🔎</span>
        <div><p className="eyebrow">Protected browsing</p><h1>Safe Browser</h1><p>Every destination is checked by KidOS before it can open.</p></div>
      </header>
      <form className="kidos-browser-search" onSubmit={(event) => { event.preventDefault(); props.onSubmit(); }}>
        <input aria-label="Protected web address" value={props.value} onChange={(e) => props.onValueChange(e.target.value)} placeholder="Search or enter a website" />
        <button type="submit">Search safely</button>
      </form>
      <div className="kidos-shortcut-grid" aria-label="Safe search topics">
        {shortcuts.map(([label, query]) => <button type="button" key={label} onClick={() => props.onShortcut(query)}>{label}</button>)}
      </div>
      <KidOSSafetyStatus status={props.protectionStatus} />
      {props.statusMessage && <div className="kidos-action-status" role="status">{props.statusMessage}</div>}
    </section>
  );
}
```

- [ ] **Step 4: Wire browser callbacks to existing `runSafeSearch`**

Keep all protected-navigation calls in `KidOSHomeShell.tsx`. The screen receives callbacks only; it must not call `api.evaluateNavigation` directly.

- [ ] **Step 5: Extend E2E test**

Assert clicking the Safe Browser tile shows the dedicated screen, and submitting `solar system` still records exactly one protected URL with `safe=active`.

- [ ] **Step 6: Run shell + E2E tests**

```bash
pnpm --filter @kidos/shell test --run src/features/home/KidOSHomeShell.test.tsx
pnpm --filter @kidos/e2e test --run tests/e2e/kidos-home-shell.spec.tsx
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add apps/shell/src/features/home tests/e2e/kidos-home-shell.spec.tsx
git commit -m "feat: add KidOS Safe Browser production screen"
```

---

### Task 3: Build the dedicated Parent Access production screen

**Files:**
- Create: `apps/shell/src/features/home/KidOSParentAccess.tsx`
- Modify: `apps/shell/src/features/home/KidOSHomeShell.tsx`
- Modify: `apps/shell/src/features/home/KidOSSidebar.tsx`
- Test: `apps/shell/src/features/home/KidOSHomeShell.test.tsx`
- Test: `tests/e2e/kidos-home-shell.spec.tsx`

**Interfaces:**

```ts
export interface KidOSParentAccessProps {
  pin: string;
  statusMessage: string;
  inputRef: React.RefObject<HTMLInputElement | null>;
  onPinChange(pin: string): void;
  onUnlock(): void;
}
```

- [ ] **Step 1: Add failing Parent-screen tests**

Add assertions that clicking `.kidos-parent-entry`:
1. displays a `Parent Access` heading,
2. moves focus to `Parent PIN`,
3. never opens the parent workspace without successful `verifyParentPin`.

Also add successful authorization and locked/rejected cases.

- [ ] **Step 2: Run focused tests and verify RED**

```bash
pnpm --filter @kidos/shell test --run src/features/home/KidOSHomeShell.test.tsx
```

- [ ] **Step 3: Create `KidOSParentAccess.tsx`**

Render a dedicated centered lock card, PIN field, unlock button, and four short capability chips: Approved Apps, Activity, Time Limits, Protection Settings. Keep the input numeric-only and limited to eight digits through the coordinator callback.

- [ ] **Step 4: Change `openParentAccess()` to select the parent screen before focusing**

`KidOSDestination` may gain `'parent'` if needed. Parent remains outside the eight Home destination tiles. Use `requestAnimationFrame(() => parentPinRef.current?.focus())` exactly as the current tested behavior requires.

- [ ] **Step 5: Keep `requestParent()` unchanged semantically**

- missing `verifyParentPin` -> unavailable status, no navigation
- `authorized: true` -> `onOpenParentWorkspace()`
- `locked: true` -> locked status
- otherwise -> rejected status
- thrown error -> unavailable status

- [ ] **Step 6: Run component + E2E tests**

```bash
pnpm --filter @kidos/shell test --run src/features/home/KidOSHomeShell.test.tsx
pnpm --filter @kidos/e2e test --run tests/e2e/kidos-home-shell.spec.tsx
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add apps/shell/src/features/home tests/e2e/kidos-home-shell.spec.tsx
git commit -m "feat: add protected Parent Access screen"
```

---

### Task 4: Upgrade Create and KidOS AI into real production modules

**Files:**
- Create: `apps/shell/src/features/home/KidOSCreateScreen.tsx`
- Create: `apps/shell/src/features/home/KidOSAiScreen.tsx`
- Create: `apps/shell/src/features/home/KidOSModulePrimitives.tsx`
- Modify: `apps/shell/src/features/home/KidOSHomeShell.tsx`
- Test: `apps/shell/src/features/home/KidOSHomeShell.test.tsx`

**Interfaces:**

```ts
export interface KidOSCreateScreenProps {
  value: string;
  workspaceTitle: string;
  onValueChange(value: string): void;
  onCreate(): void;
}

export interface KidOSAiScreenProps {
  value: string;
  answer: string;
  onValueChange(value: string): void;
  onAsk(): void;
  onSuggestion(question: string): void;
}
```

- [ ] **Step 1: Add Create success/failure tests**

Success must show `Safe workspace ready: <title>`. Rejected planner call must show `KidOS could not prepare the workspace safely.`

- [ ] **Step 2: Add AI presentation tests**

Verify suggested prompts populate/submit through the existing coordinator logic and that the screen visibly states responses remain inside KidOS safety rules.

- [ ] **Step 3: Run tests and verify RED**

```bash
pnpm --filter @kidos/shell test --run src/features/home/KidOSHomeShell.test.tsx
```

- [ ] **Step 4: Create shared primitives**

Keep them presentational:

```tsx
export function KidOSModuleHeader({ icon, eyebrow, title, description }: { icon: string; eyebrow: string; title: string; description: string }) {
  return <header className="kidos-screen-header"><span className="kidos-screen-icon" aria-hidden="true">{icon}</span><div><p className="eyebrow">{eyebrow}</p><h1>{title}</h1><p>{description}</p></div></header>;
}

export function KidOSStatusBanner({ children }: { children: React.ReactNode }) {
  return <div className="kidos-action-status" role="status">{children}</div>;
}
```

- [ ] **Step 5: Build `KidOSCreateScreen` around existing `api.planWorkspace(...)` coordinator state**

Offer visible prompt chips for Story, Drawing, Presentation, Beginner Coding, and Project Idea. Chips only set the prompt; they do not create new capabilities.

- [ ] **Step 6: Build `KidOSAiScreen` around existing coordinator answer logic**

Use a friendly assistant card, answer panel, question field, and safe suggestion chips. Do not add network calls or credentials.

- [ ] **Step 7: Run tests + build**

```bash
pnpm --filter @kidos/shell test --run src/features/home/KidOSHomeShell.test.tsx
pnpm --filter @kidos/shell build
```

Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add apps/shell/src/features/home
git commit -m "feat: upgrade KidOS Create and AI modules"
```

---

### Task 5: Apply the production visual system to match the approved mockup

**Files:**
- Modify: `apps/shell/src/features/home/kidos-shell-2026.css`
- Modify: `apps/shell/src/features/home/KidOSSidebar.tsx`
- Modify: `apps/shell/src/features/home/KidOSTopBar.tsx`
- Modify: `apps/shell/src/features/home/KidOSGreeting.tsx`
- Modify: `apps/shell/src/features/home/KidOSHomeGrid.tsx`
- Modify: `apps/shell/src/features/home/KidOSSafetyStatus.tsx`
- Modify: `apps/shell/src/features/home/KidOSProfileCard.tsx`
- Modify: `apps/shell/src/features/home/KidOSDock.tsx`
- Test: `apps/shell/src/features/home/kidos-shell-2026-layout.test.ts`

**Interfaces:**
- Consumes semantic production screen classes.
- Produces the approved scenic KidOS shell with no remote asset requirement.

- [ ] **Step 1: Expand layout contract tests before CSS changes**

Add assertions for the production classes and target rules:

```ts
it('keeps production screens constrained and readable on wide desktops', () => {
  expect(css).toContain('.kidos-screen{');
  expect(css).toContain('.kidos-category-grid{');
});

it('preserves explicit active degraded and offline styling', () => {
  expect(css).toContain('[data-state="active"]');
  expect(css).toContain('[data-state="degraded"]');
  expect(css).toContain('[data-state="offline"]');
});
```

- [ ] **Step 2: Run layout test and verify RED**

```bash
pnpm --filter @kidos/shell test --run src/features/home/kidos-shell-2026-layout.test.ts
```

- [ ] **Step 3: Refactor CSS into readable production sections**

Keep one file but organize in this order: tokens/shell, sidebar, top bar, home, destination modules, safety states, dock, focus/accessibility, responsive rules.

Required visual behavior:
- navy translucent sidebar
- bright layered sky/green scenic background using CSS gradients
- white/glass production cards
- large rounded tile radii
- strong but non-flashing hover lift
- readable light/dark text contrast by surface
- safety states differentiated by both label and outline/background
- Parent card visually distinct from child modules
- compact 1366x768 behavior without changing the standard four-column desktop grid

- [ ] **Step 4: Preserve exact layout-contract substrings or update tests intentionally**

The final CSS must retain:

```css
.kidos-home-grid{display:grid;grid-template-columns:repeat(4,minmax(0,1fr))
@media(max-width:1100px){.kidos-home-grid{grid-template-columns:repeat(2,minmax(0,1fr))}
@media(max-height:780px) and (min-width:1101px)
```

If formatting changes, update the contract test to assert equivalent semantics without weakening the requirement.

- [ ] **Step 5: Run layout, component, and build checks**

```bash
pnpm --filter @kidos/shell test --run src/features/home/kidos-shell-2026-layout.test.ts src/features/home/KidOSHomeShell.test.tsx
pnpm --filter @kidos/shell build
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add apps/shell/src/features/home
git commit -m "style: match KidOS production shell to approved design"
```

---

### Task 6: Strengthen app-level and E2E safety regression coverage

**Files:**
- Modify: `apps/shell/src/App.test.tsx`
- Modify: `tests/e2e/kidos-home-shell.spec.tsx`
- Modify: `apps/shell/src/features/home/KidOSHomeShell.test.tsx`

**Interfaces:**
- No new runtime interface.
- Produces release-blocking proof that the UI refactor did not weaken existing boundaries.

- [ ] **Step 1: Add an app-level restricted-mode regression**

```ts
it('never renders the production child shell when Guardian reports restricted safe mode', async () => {
  const restrictedApi = { ...healthyApi, guardianStatus: async () => 'restricted_safe_mode' as const };
  render(<App api={restrictedApi} />);
  expect(await screen.findByText('Restricted safe mode')).toBeTruthy();
  expect(screen.queryByTestId('kidos-shell')).toBeNull();
});
```

- [ ] **Step 2: Add E2E navigation coverage for all eight destinations**

Use `within(screen.getByTestId('kidos-home-grid'))` to click each tile and assert the screen heading. Return Home using the sidebar between cases.

- [ ] **Step 3: Add My Apps fail-closed assertion**

Verify the My Apps screen contains parent-approved language and does not contain `input[type="file"]`, path-entry fields, or any button named `Run`, `Execute`, or `Browse executable`.

- [ ] **Step 4: Run all JS/TS test suites and shell build**

```bash
pnpm test
pnpm test:e2e
pnpm build
```

Expected: PASS.

- [ ] **Step 5: Run source-policy scans locally where supported**

```bash
git grep -n -E '(localStorage|sessionStorage).*(pin|PIN)|((pin|PIN).*(localStorage|sessionStorage))' -- apps packages crates && exit 1 || true
git grep -n -E 'search_history|searchHistoryTable|full_search_history' -- apps packages 'crates/*/src' && exit 1 || true
```

Expected: no prohibited matches.

- [ ] **Step 6: Commit**

```bash
git add apps/shell/src/App.test.tsx apps/shell/src/features/home/KidOSHomeShell.test.tsx tests/e2e/kidos-home-shell.spec.tsx
git commit -m "test: lock KidOS production UI safety regressions"
```

---

### Task 7: Run full branch validation and open the implementation PR

**Files:**
- No production file changes unless a test exposes a real defect.

- [ ] **Step 1: Run the repository verification set**

```bash
pnpm test
pnpm test:e2e
pnpm build
cargo test --workspace
cargo check --workspace
```

Expected: PASS.

- [ ] **Step 2: Verify the branch diff is limited to the production UI scope**

Confirm no classifier credentials, arbitrary command bridge, installer weakening, Guardian policy bypass, or unrelated refactor was introduced.

- [ ] **Step 3: Push/open a PR against `main`**

PR title:

```text
Complete KidOS production UI
```

PR body must explicitly state:
- visual shell upgraded to approved KidOS design
- eight destination screens implemented
- Safe Browser remains policy-gated
- Parent Access remains PIN-gated
- My Apps remains fail-closed without a trusted launcher
- no security boundary was weakened
- branch tests/build passed

- [ ] **Step 4: Wait for and inspect every PR-triggered workflow**

Do not merge while any required workflow is queued, running, or failed.

- [ ] **Step 5: Merge only after required branch validation is green**

Use the PR head SHA as the merge guard.

---

### Task 8: Verify exact-main Windows release validation and installer artifacts

**Files:**
- No source change unless exact-main validation finds a real defect.

- [ ] **Step 1: Read the new exact `main` SHA after merge**

Use that SHA for every post-merge query; do not reuse a prior main commit's green status.

- [ ] **Step 2: Verify every workflow triggered for that exact main SHA**

Required minimum set:
- KidOS Task 12 E2E CI
- KidOS Windows Lockdown CI
- KidOS Clean Windows VM CI
- KidOS Windows Soak CI
- KidOS Shadow Validation CI
- KidOS Free Windows Hardening
- KidOS Windows Release CI
- any additional security/release workflow triggered for the commit

- [ ] **Step 3: Inspect Clean Windows VM and Windows Release artifacts**

Require successful install/service/recovery/uninstall verification and a produced Windows installer/integrity manifest artifact.

- [ ] **Step 4: Declare completion only with fresh evidence**

The production UI is 100% complete only when the exact-main required workflows are successful and the verified installer artifact exists. If any workflow is running or failed, report the real percentage/status instead of calling the task complete.

---

## Self-Review

### Spec coverage

- Shell decomposition: Tasks 1-4.
- Home mockup and eight destinations: Tasks 1 and 5.
- Safe Browser visual + security flow: Task 2.
- Parent Access visual + PIN boundary: Task 3.
- Learn/Play/Watch/Wellbeing/My Apps: Task 1.
- Create and KidOS AI: Task 4.
- Styling/responsive/accessibility: Task 5.
- Fail-closed behavior/regressions: Task 6.
- Branch CI/PR: Task 7.
- Exact-main Windows/installer validation: Task 8.

### Placeholder scan

No TBD/TODO/implement-later placeholders are present. Every task has concrete files, expected interfaces, commands, and pass conditions.

### Type consistency

- `KidOSSystemStatus` remains imported from existing `system-status.ts`.
- `KidOSDestination` remains the navigation type from `KidOSSidebar.tsx`; only optional `'parent'` extension is introduced if needed.
- Safe Browser callbacks remain coordinator-owned and use the existing `KidOSApi` methods.
- Parent Access continues to call the existing `verifyParentPin` contract.
- No new privileged `KidOSApi` method is required by this plan.
