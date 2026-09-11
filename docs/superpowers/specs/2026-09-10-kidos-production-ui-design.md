# KidOS Production UI Design

Date: 2026-09-10
Status: Approved design, pending implementation-plan review

## Goal

Turn the approved KidOS visual mockup into the real installed Windows KidOS experience without weakening the existing Guardian, filtering, recovery, Parent PIN, Safe Browser, or approved-app security boundaries.

The production UI should look and behave like a polished child-first operating environment while remaining a React/Tauri presentation layer on top of the existing Windows protection architecture.

## Non-negotiable safety rules

1. KidOS Guardian remains authoritative. React never becomes the source of truth for protection state or policy decisions.
2. Safe Browser navigation remains fail-closed and continues to pass every destination through the existing protected-navigation and policy-evaluation flow.
3. Green safety indicators may only render when the live backend reports the corresponding protection as active/ready.
4. Parent controls remain behind Parent PIN verification. No visual redesign may bypass or weaken authorization.
5. My Apps continues to launch only trusted parent-approved applications. The UI must not accept arbitrary executable paths.
6. Classifier credentials remain outside browser JavaScript.
7. Existing Windows recovery, installer, service, lockdown, soak, Shadow Validation, hardening, and release gates remain release blockers.

## Visual direction

The production shell will follow the approved KidOS mockup:

- Dark navy left sidebar with KidOS branding and clear destination navigation.
- Bright scenic background using layered CSS gradients and optional bundled illustration assets.
- Large friendly greeting area with child profile/avatar treatment.
- Four-by-two home tile grid on standard desktop widths.
- Rounded, colorful destination cards with large icons, short descriptions, hover/focus feedback, and accessible labels.
- Prominent live safety strip showing Safe Mode, Internet Filter, and Media Safety.
- Kid-friendly typography, generous spacing, high contrast, and soft glass/rounded panel styling.
- Responsive layout for 1366x768 and 1920x1080, including 125% and 150% Windows scaling.
- Two-column home fallback for narrower windows.

The UI should feel like a complete KidOS environment rather than a collection of generic web forms.

## Architecture

### Shell coordinator

`KidOSHomeShell.tsx` remains the top-level coordinator. It will own:

- active destination state
- live system status polling
- protected search orchestration
- Parent PIN orchestration
- safe workspace requests
- KidOS AI interaction state

It should no longer contain the full markup for every destination. Each major destination becomes an isolated screen component.

### New screen/component boundaries

Planned components under `apps/shell/src/features/home/`:

- `KidOSHomeScreen.tsx`
- `KidOSSafeBrowserScreen.tsx`
- `KidOSParentAccess.tsx`
- `KidOSLearnScreen.tsx`
- `KidOSPlayScreen.tsx`
- `KidOSCreateScreen.tsx`
- `KidOSWatchScreen.tsx`
- `KidOSAiScreen.tsx`
- `KidOSWellbeingScreen.tsx`
- `KidOSMyAppsScreen.tsx`
- shared visual primitives for module headers, category cards, status banners, and safe-action buttons when useful

Existing components such as `KidOSSidebar`, `KidOSTopBar`, `KidOSGreeting`, `KidOSHomeGrid`, `KidOSSafetyStatus`, `KidOSProfileCard`, and `KidOSDock` should be reused or refined rather than duplicated.

## Home screen

The Home screen will closely match the approved visual reference.

It includes exactly eight primary destinations:

1. Learn
2. Play
3. Create
4. Watch
5. Safe Browser
6. KidOS AI
7. Wellbeing
8. My Apps

The desktop layout is a four-column, two-row grid. Each tile includes an icon, title, and short child-friendly description.

The safety strip must remain visible and truthful. Active, degraded, and offline states must remain visually distinct and must never default to green when telemetry is missing.

## Safe Browser

The Safe Browser becomes a full-screen KidOS module rather than a generic form.

It includes:

- large title and protection explanation
- prominent protected search/address field
- Search button
- child-friendly topic shortcuts such as Learning, Science, Animals, Space, Arts & Crafts, and Math
- live protection strip
- clear result/status banner for opened, blocked, parent-gated, or unavailable destinations

All search/address submissions continue through `prepareProtectedNavigation(...)` and `api.evaluateNavigation` before `api.openProtectedBrowser(...)` is invoked.

No unrestricted fallback browser is introduced.

## Parent Access

Parent Access becomes a dedicated locked screen/card rather than appearing as a persistent generic module below every child screen.

It includes:

- lock icon and Parent Access title
- Parent PIN input
- Unlock action
- clear locked/rejected/unavailable feedback
- concise explanation of parent capabilities such as approved apps, activity, time limits, and protection settings

The existing focus behavior is preserved: selecting Parent from the sidebar moves keyboard focus into the Parent PIN field.

Successful PIN authorization calls the existing `onOpenParentWorkspace()` flow.

## Learn

Learn will present curated child-facing categories such as Math, Science, Reading, Homework, and approved educational tools.

The first implementation is visual/navigation structure only unless an existing trusted learning-provider flow is already available. No new unreviewed web-navigation bypass is added.

## Play

Play will present parent-approved games and safe play categories.

If no approved-app launch API is available for a specific game, the UI remains fail-closed and shows that the item requires parent approval or is unavailable rather than accepting an arbitrary executable path.

## Create

Create keeps the existing `api.planWorkspace(...)` path and redesigns it as a guided creation screen.

It should support child-friendly prompts such as stories, drawings, presentations, beginner coding, and project ideas while preserving the existing protected-workspace contract.

## Watch

Watch presents kid-safe media categories and clearly communicates that media is subject to KidOS safety checks.

This design does not invent a new media source or bypass the existing classifier. Content integration beyond existing approved providers is a later product feature.

## KidOS AI

KidOS AI receives a conversational child-friendly interface with:

- friendly assistant identity
- question input
- safe-answer panel
- suggested learning prompts
- clear indication that responses remain inside KidOS safety rules

The visual upgrade does not change the existing trust boundary: no browser-side classifier secrets and no hidden bypass of Guardian policy.

## Wellbeing

Wellbeing presents screen-time balance, healthy breaks, accessibility, and simple wellness guidance.

The first production pass focuses on safe presentation and existing available controls. New OS-level time enforcement is outside this UI-only phase unless already exposed by the KidOS API.

## My Apps

My Apps presents only trusted, approved applications exposed through existing KidOS capabilities.

If no trusted launch API is present, the screen shows approved-app status and remains non-launching rather than introducing generic process execution.

## Styling

`kidos-shell-2026.css` will be refactored into a clearer production design system while preserving current responsive behavior.

Key design tokens:

- navy sidebar and dark glass surfaces
- bright sky/scenic stage background
- large rounded card radii
- accessible focus ring
- child-friendly tile gradients
- elevated white/glass cards for Safe Browser and Parent Access
- consistent spacing scale and typography scale
- explicit active/degraded/offline status treatments

No remote image dependency is required for the shell to render. Any final illustration assets should be bundled with the app so KidOS remains reliable offline.

## Accessibility

The production UI must preserve or improve:

- keyboard navigation
- visible `:focus-visible` treatment
- semantic button/input labels
- sufficient text contrast
- large touch/click targets
- responsive scaling
- no safety information conveyed only by color

## Error handling

Every protected action must fail closed.

Examples:

- Guardian telemetry unavailable -> show offline/degraded state, never active.
- Safe Browser API unavailable -> do not open destination.
- Parent verification unavailable -> do not open Parent Workspace.
- workspace planner failure -> show safe error state.
- approved-app launcher unavailable -> do not execute arbitrary paths.

## Testing

### Component tests

Add or extend Vitest/Testing Library coverage for:

- exactly eight Home destinations
- navigation between all destination screens
- truthful safety-state rendering
- Safe Browser allow/block/parent-gate/unavailable states
- SafeSearch enforcement on allowed Google queries
- Parent PIN focus and authorization behavior
- My Apps fail-closed behavior
- Create workspace success/failure presentation

### Layout contract tests

Maintain explicit assertions for:

- four columns on standard desktop
- two columns below compact breakpoint
- 1366x768 fit/compression behavior
- 1920x1080 layout
- visible focus treatment

### Windows release gates

The UI change is not complete until the relevant exact-main Windows workflows pass, including:

- Task 12 E2E
- Windows Lockdown
- Clean Windows VM
- Windows Soak
- Shadow Validation
- Free Windows Hardening
- Windows Release
- other required security/release checks triggered by the change

The generated installer must still pass install, service startup, recovery, and uninstall verification.

## Implementation order

1. Extract destination screens from `KidOSHomeShell.tsx` without changing behavior.
2. Build/refine the shared production visual system.
3. Match the Home screen to the approved mockup.
4. Build the dedicated Safe Browser screen while preserving protected navigation.
5. Build the dedicated Parent Access screen while preserving PIN authorization.
6. Upgrade Learn, Play, Create, Watch, KidOS AI, Wellbeing, and My Apps using the same design system.
7. Add/upgrade component and layout tests.
8. Run branch CI and repair regressions.
9. Merge only after required branch checks pass.
10. Verify all required exact-main Windows release gates and installer artifacts before calling the production UI complete.

## Success criteria

The work is complete when:

- the installed KidOS shell visually matches the approved mockup closely enough to be recognizably the same product design
- all eight destinations are real interactive screens
- Safe Browser and Parent Access match the approved visual direction
- no security boundary is weakened
- live safety indicators remain truthful
- the shell works at target Windows resolutions/scaling
- automated UI tests pass
- required Windows post-merge validation passes on the exact main commit
- a verified Windows installer artifact is produced
