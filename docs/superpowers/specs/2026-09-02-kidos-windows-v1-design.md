# KidOS Windows v1 Design

## Vision
KidOS is a downloadable, child-focused Windows computing environment built around creating instead of opening traditional applications. A child can ask to make a story, drawing, presentation, beginner program, or research project, and KidOS assembles an age-appropriate workspace.

KidOS uses defense-in-depth rather than promising perfect internet filtering. Safety combines a protected browser, search restrictions, age profiles, parent controls, download rules, network protections, and policy-based AI behavior. Unknown high-risk actions fail closed or require parent approval.

## Release target
- Windows-first downloadable installer.
- Full-screen KidOS desktop shell with parent-controlled exit.
- Child profiles with age-appropriate policy.
- Parent/guardian controls protected by a PIN and Windows-backed secure storage.
- Architecture designed to support macOS later.

## Architecture

### KidOS Shell
The child-facing desktop provides a large “What do you want to create?” command bar, recent projects, approved creation spaces, protected web access, and a clear safety status. The shell presents decisions but does not control privileged security policy.

### Creation Engine
The creation engine converts requests into safe workspace plans. For example, “make a cartoon about dinosaurs” can assemble story planning, drawing, narration, scene arrangement, and export capabilities. Every request receives the active age and safety policy, and the engine cannot grant capabilities outside that policy.

### Safety Policy Engine
The policy engine is the authority for protected navigation, downloads, external-app launches, social-platform access, and permission escalation. Decisions are `allow`, `block`, or `require_parent`.

Initial categories include adult content, gambling, malware/phishing/scams, inappropriate dangerous content, extremist recruitment/propaganda, controlled-substance sales, risky file hosting/executable downloads, unknown destinations, and parent-defined allow/block rules.

### Protected Browser and Search
KidOS web access runs through a protected browser surface that enforces supported SafeSearch/Restricted Mode settings, checks navigation against policy, restricts high-risk downloads, applies parent domain rules, and prevents casual external-browser bypass from the KidOS shell.

KidOS minimizes collection of children's search data. Full search history is not retained as a long-term browsing database by default. Safety logs retain only information necessary to explain blocked or approval-required events.

### Windows Guardian Service
A Windows background service keeps critical protections active for managed child profiles. It starts protections at sign-in, stores integrity-protected policy, brokers privileged parent-authorized changes, monitors the protected session, and prevents child-facing processes from directly modifying privileged policy.

Child-facing renderer processes never receive administrator privileges.

## Parent controls
Parents can configure a PIN, child age/profile, web categories, domain allow/block lists, approved social platforms, schedules/time windows, download permissions, permission prompts, safety-event summaries, and temporary exit/unlock behavior. Parent secrets are never stored in plaintext configuration.

## Social-media safeguards
KidOS controls access to third-party social platforms rather than claiming to control those platforms' internal moderation. Parents can block a service, allow it during selected times, require protected-browser access, restrict external links/downloads/uploads, or require parent approval before first access. KidOS does not store the child's social-media password.

## Filtering model
Filtering combines parent rules, domain reputation/category data, URL/navigation policy, provider SafeSearch controls, local content classification when needed, and fail-closed handling for high-risk unknown actions. Optional network/DNS filtering adds another layer but is not the only defense.

## Download safety
Executables, scripts, installers, suspicious archives, and other high-risk downloads are parent-gated or blocked according to policy. Child-created images, documents, project files, and approved media remain exportable. Downloads pass through policy before being exposed to the child session.

## Privacy
KidOS follows data minimization: no advertising profile, no sale of child activity data, no full search-history retention by default, minimal safety-event logging, parent controls for clearing local records, and documentation of local versus remote processing. Commercial release requires a dedicated child-privacy/legal review; the repository will not make unsupported compliance claims.

## AI boundaries
The AI receives age/policy context on every request, cannot reveal parent secrets, cannot execute arbitrary shell commands from child prompts, uses allowlisted creation capabilities, requires parent authorization for privileged actions, and cannot override the policy engine. AI is an orchestrator/tutor, not the system security authority.

## Security boundary
The main trust boundary is between the child-facing UI and the Windows Guardian Service. UI requests actions; policy decides; privileged operations execute only through the Guardian Service after validation. IPC is authenticated and schema-validated, policy integrity is protected, parent authentication is rate-limited, and logs exclude PINs/tokens/unnecessary child content.

## Proposed stack
- TypeScript + React for the KidOS shell.
- Tauri 2 for the Windows desktop shell.
- Rust for policy-core, Guardian Service, and privileged Windows integration.
- WebView2/Tauri webview for protected browsing where feasible.
- SQLite for local projects and minimal safety-event metadata.
- Windows DPAPI/Credential Manager-backed secret storage.
- Vitest for TypeScript tests.
- `cargo test` for Rust policy/service tests.
- Playwright/equivalent for end-to-end flows once the desktop harness exists.

## Repository structure
```text
KidOS/
  apps/shell/
  crates/policy-core/
  crates/guardian-service/
  crates/secure-store/
  packages/contracts/
  packages/creation-engine/
  docs/architecture/
  docs/safety/
  docs/superpowers/specs/
  docs/superpowers/plans/
  tests/e2e/
```

## Child session flow
1. Windows signs into a managed child account.
2. Guardian Service loads valid KidOS policy.
3. KidOS Shell launches in protected mode.
4. Child enters a creation or browsing request.
5. Web/download/external-app/permission actions go through policy.
6. Policy returns allow, block, or require_parent.
7. Parent-required actions use a protected authorization flow.
8. Approved privileged operations are brokered by Guardian Service.
9. KidOS gives the child a simple explanation of the result.

## Failure behavior
If Guardian Service is unavailable, KidOS enters restricted safe mode. If policy is corrupt, it uses the last valid policy or a strict built-in baseline. If classification is unavailable, known approved destinations may continue while unknown/high-risk navigation is blocked or parent-gated. If AI is unavailable, non-AI creation tools continue working. Secure-store failure never falls back to plaintext secrets.

## Testing
Security-sensitive code is developed test-first. Automated coverage includes age-profile decisions, parent rule precedence, unknown-site behavior, SafeSearch configuration, executable blocking, authorization expiry, malformed IPC rejection, privileged-policy protection, Guardian restart/recovery, creation-engine capability limits, and clearing local safety-event records. Manual tests cover closing the shell, launching external browsers, editing local configuration, redirect bypasses, disguised executable downloads, and parent-PIN brute-force resistance.

## v1 scope
The first usable milestone includes the Windows installer, child shell, parent setup/PIN, age profiles, policy engine, protected navigation, SafeSearch, domain/category rules, high-risk download restrictions, basic social-platform policies, Guardian Service, creation command bar, Story workspace, Drawing/Presentation workspace, Beginner Coding workspace, recent projects, and parent safety-event summary.

## Non-goals
KidOS v1 will not replace the Windows kernel, promise perfect filtering, read every private social message, secretly record children, keylog, bypass third-party security, permit unrestricted AI shell execution, provide a full app store, or ship macOS/Linux installers.

## Success criteria
A Windows household can install KidOS, create a protected child profile, launch into the KidOS shell, ask it to assemble a starter creative workspace, browse through protected web access, see unsafe or unknown actions blocked/parent-gated, and let a parent manage rules without exposing administrative controls to the child.
