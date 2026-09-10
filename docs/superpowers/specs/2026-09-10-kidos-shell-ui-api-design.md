# KidOS Shell UI + Live System State Design

Date: 2026-09-10
Status: Approved design, pending implementation plan

## Goal

Rebuild the KidOS Windows shell so it visually matches the approved KidOS mockup while keeping the existing Guardian, Media Classifier, recovery, and Windows hardening architecture intact. The interface must not display protection states that are not backed by live system data.

## Visual Source of Truth

Use the second approved mockup as the primary layout reference, with selective visual richness from the first mockup.

Primary characteristics:

- Full-screen scenic mountain/lake background.
- Dark translucent left navigation rail.
- KidOS logo and tagline at the top-left.
- Large centered greeting and safe-search field.
- Eight large rounded destination tiles arranged in a 4x2 grid.
- Top-right profile card with current time/date and system indicators.
- Right-side positive-message card.
- Bottom centered dock.
- Bottom-right live safety indicators.
- Strong rounded corners, soft shadows, glass/translucent panels, large readable typography, and child-friendly iconography.

The target is not a static screenshot recreation. The shell should maintain this composition across supported Windows resolutions and scaling settings.

## Shell Structure

The main shell should be broken into focused components instead of putting the entire screen into one large component.

Recommended structure:

- `KidOSHomeShell`
  - owns layout composition only.
- `KidOSSidebar`
  - Home, Apps, Learn, Create, Play, Watch, Music, Settings, Parent.
- `KidOSTopBar`
  - safe search, profile, date/time, Wi-Fi/battery when available.
- `KidOSGreeting`
  - child display name and positive message.
- `KidOSHomeGrid`
  - Learn, Play, Create, Watch, Safe Browser, KidOS AI, Wellbeing, My Apps.
- `KidOSDock`
  - selected safe local applications/actions.
- `KidOSSafetyStatus`
  - live Safe Mode and Internet Filter state.
- `KidOSProfileCard`
  - child display name, avatar, age/level.

Each component should have a narrow interface and remain testable without requiring the entire shell.

## Styling Architecture

Create a dedicated KidOS 2026 shell stylesheet rather than layering more rules into legacy styles.

Design tokens should cover:

- panel transparency and blur,
- tile radius,
- tile spacing,
- shadow depth,
- typography scale,
- sidebar width,
- dock height,
- responsive breakpoints,
- status indicator sizes,
- focus/keyboard state.

Use CSS grid/flex layout instead of pixel-positioning the entire screen. Pixel-level tuning is appropriate for local spacing, but the global layout must remain responsive.

The existing valid Windows `.ico` file should remain untouched until the approved 2026 KidOS shield is regenerated as a structurally valid multi-resolution ICO and verified by packaging tests.

## Background Artwork

The scenic background is a presentation asset. It must not be coupled to system safety logic.

Requirements:

- cover the full viewport,
- preserve center composition at common 16:9 resolutions,
- degrade gracefully when the image is unavailable,
- provide enough overlay contrast for navigation and text,
- not block keyboard or accessibility navigation.

The final production asset should be an approved KidOS-owned or properly licensed image stored in the application assets.

## Live System State Adapter

Introduce a thin frontend adapter that converts backend/service state into a small stable UI model.

Example conceptual model:

```ts
interface KidOSSystemStatus {
  guardianRunning: boolean;
  classifierRunning: boolean;
  safeMode: 'active' | 'degraded' | 'offline';
  internetFilter: 'active' | 'degraded' | 'offline';
  classifierReady: boolean;
  recoveryAvailable: boolean;
}
```

The shell should consume this model rather than directly knowing Windows service details.

## Safety-State Truthfulness

The UI must never infer safety from its own local state.

Examples:

- `Safe Mode: ON` may be shown only when the authoritative KidOS protection state reports active enforcement.
- `Internet Filter: Active` may be shown only when the filtering path is available and healthy according to the authoritative backend contract.
- If Guardian cannot be reached, the UI should show a degraded/offline state instead of leaving the previous green label visible.
- If the classifier is booting, the UI may show a transitional status without implying full protection.

Security remains controlled by Guardian/service policy. The shell is a client and cannot grant itself safer status.

## Guardian Integration

Do not weaken or replace Guardian's privileged role.

The frontend should consume only the minimum read-only status needed for presentation. If no existing read-only status contract exposes the required information, add a narrow read-only command/endpoint rather than exposing privileged service operations to the shell.

No UI route should provide direct unrestricted stop, disable, bypass, ACL modification, or credential access for Guardian.

## Media Classifier Integration

Keep the existing authenticated classifier service contract.

The shell may consume summarized classifier availability through the trusted backend bridge. It should not read the classifier credential directly from browser-facing JavaScript.

Continue separating:

- fast service readiness, and
- deep model readiness.

The home screen should not block for model loading. Features that require deep classification can show a loading/unavailable state until the model is ready.

## Profile Data

The visible child profile should come from KidOS profile state rather than hard-coded values.

UI model should support:

- display name,
- avatar,
- age or age band,
- level if KidOS retains that concept,
- approved apps.

The design should work with missing optional profile fields.

## Navigation Behavior

Home tiles and left-nav entries should resolve to KidOS routes or approved application launch actions.

Primary home tiles:

1. Learn
2. Play
3. Create
4. Watch
5. Safe Browser
6. KidOS AI
7. Wellbeing
8. My Apps

Navigation must remain compatible with child-mode restrictions. A visual route must not become a bypass around Guardian policy.

The Parent entry should retain its existing authorization boundary. Merely clicking the sidebar item must not expose parent-only functions without authentication/approval.

## Safe Search

The large top search field is a shell entry point into KidOS safe browsing/search, not a generic unrestricted browser address bar.

It should:

- pass searches through the existing KidOS safe-navigation path,
- support keyboard submit,
- provide clear disabled/degraded feedback if protection services are unavailable,
- never silently fall back to unrestricted browsing.

## KidOS AI Tile

KidOS AI should enter the existing approved AI creation/learning flow.

The shell redesign must preserve current parent-approval behavior for protected AI-session actions. Visual redesign is not permission to bypass the existing approval flow.

## Apps and Dock

`My Apps` and dock content must be based on approved applications/actions. The design may show friendly icons, but launch permission still comes from KidOS policy.

Do not create arbitrary executable launching from a frontend-provided path.

## System Indicators

Time/date can be read locally.

Battery and connectivity indicators should use platform-safe read-only APIs when available. If Windows/Tauri access is unavailable or unreliable, omit or show a neutral state rather than inventing values.

These indicators are decorative/system-information features and are separate from KidOS security state.

## Error and Degraded States

The scenic shell must remain usable when a backend service is unavailable.

Required behavior:

- Guardian unavailable -> prominent degraded protection indication.
- Classifier unavailable -> classifier-dependent features disabled/degraded.
- profile load failure -> safe fallback profile presentation.
- background asset failure -> solid/gradient fallback background.
- status polling failure -> stale green state must not persist indefinitely.

Errors should be child-readable on the home screen while detailed diagnostics remain available to guardian/admin tooling.

## Accessibility

The visual target must not sacrifice accessibility.

Requirements:

- keyboard navigation across sidebar, search, tiles, and dock,
- visible focus states,
- semantic buttons/links,
- usable screen-reader labels for icon-only controls,
- sufficient text contrast over scenic artwork,
- support for Windows display scaling,
- no essential information conveyed only by color.

## Responsive Behavior

Primary target: Windows desktop/laptop landscape screens.

The shell should support at minimum:

- 1366x768,
- 1920x1080,
- common Windows 125% and 150% scaling.

At constrained widths/heights, tiles may scale and gaps may tighten while preserving the eight-tile structure. The sidebar and safety indicators must remain reachable.

## Testing Strategy

### Component tests

Add tests for:

- sidebar route selection,
- home-tile actions,
- profile rendering,
- safe/degraded/offline protection status,
- search submission,
- parent authorization boundary,
- approved-app launch behavior.

### Contract tests

Test the system-status adapter independently from the visual shell. Ensure unsafe/stale inputs cannot produce `Safe Mode: ON` or `Internet Filter: Active` incorrectly.

### E2E tests

Preserve existing child-mode and AI approval tests and add coverage for:

- loading the redesigned home screen,
- navigating all child-safe destinations,
- rendering degraded service state,
- keyboard navigation,
- safe search route.

### Visual regression

Add deterministic screenshot/visual-regression coverage at the target desktop viewport sizes where the existing CI stack permits it. Do not use visual snapshots as a substitute for functional security tests.

### Windows release tests

The shell redesign must continue passing:

- Clean Windows VM,
- Windows Soak,
- Shadow Validation,
- Windows Lockdown,
- Windows hardening,
- Windows Release.

A visual improvement is not complete if it breaks installer, Guardian, classifier, recovery, or lockdown gates.

## Implementation Boundaries

This redesign should not:

- replace Guardian,
- move privileged security enforcement into React,
- expose classifier credentials to browser code,
- weaken parent authentication,
- add unrestricted executable launch,
- add unrestricted web navigation,
- alter recovery semantics merely to support the new appearance.

## Expected Repository Areas

Likely implementation areas include:

- `apps/shell/src/` for React components, routes, adapters, and styles,
- `apps/shell/src-tauri/` only where a narrow read-only platform bridge is required,
- existing Guardian/backend crates only if a missing read-only status contract must be added,
- `tests/e2e/` and shell tests for UI behavior,
- packaging/security tests only as needed to prevent regression.

Exact files should be selected after implementation planning reads the current shell structure.

## Acceptance Criteria

The redesign is complete when:

1. KidOS home screen closely matches the approved second mockup in composition, spacing, hierarchy, translucency, tile presentation, sidebar, search, profile, dock, and safety indicators.
2. The layout works at supported Windows viewport/scaling targets.
3. Visible protection labels are driven by authoritative live status, not hard-coded frontend values.
4. Existing parent authorization and child-mode restrictions remain enforced.
5. Safe search cannot fall back to unrestricted browsing.
6. Existing E2E/security behavior remains green.
7. Clean Windows VM, Windows Soak, Shadow Validation, Lockdown/Hardening, and Windows Release remain release gates.
8. No new critical/high security defect is introduced.

## Delivery Sequence

Implementation should proceed in this order:

1. Read current shell component/routing/status architecture.
2. Add tests for the new system-status truthfulness contract.
3. Implement the system-status adapter/read-only bridge where required.
4. Build the new shell component hierarchy.
5. Apply the approved visual design and responsive layout.
6. Wire profile, routes, approved apps, safe search, AI, and safety indicators.
7. Add component/E2E/visual-regression coverage.
8. Run JS/Rust/security tests.
9. Run Windows release gates.
10. Reintroduce the approved 2026 KidOS shield icon only after generating and validating a proper multi-resolution ICO.

