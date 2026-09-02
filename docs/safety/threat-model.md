# KidOS Windows v1 Threat Model

## Purpose

KidOS is a child-focused Windows computing environment that places an unprivileged Tauri/React shell in front of protected creation, browsing, download, and parent-control flows. Windows remains the underlying operating system. KidOS is designed to reduce a child's exposure to inappropriate or unsafe content and to prevent the child-facing renderer from becoming a privileged security boundary.

KidOS does not claim perfect content detection, perfect website blocking, or immunity from all Windows bypass techniques. The security goal is defense in depth with fail-closed behavior for high-risk or unknown actions.

## Protected assets

KidOS protects:

- parent authorization secrets and PIN verifiers;
- active child profile and parent policy configuration;
- Guardian Service policy state and last-known-valid policy state;
- privileged policy-change operations;
- protected navigation and download decisions;
- safety-event records containing only minimal metadata;
- child-facing creation projects from unauthorized privileged access;
- the integrity of the KidOS shell-to-Guardian trust boundary.

KidOS intentionally does not retain a long-term full browsing/search history by default.

## Trust boundaries

### Child-facing shell

The Tauri/React renderer is treated as untrusted relative to privileged operations. It must not run with administrator privileges, execute arbitrary operating-system commands, write Guardian policy directly, or gain access to parent secrets.

### Tauri command boundary

The shell communicates through a narrow allowlisted command surface. Generic command dispatchers and arbitrary shell bridges are prohibited. Inputs must be typed and validated before reaching policy or privileged code.

### Guardian Service boundary

The Guardian Service is the authority for privileged policy mutation and protected system actions. The intended Windows deployment model is a locally installed Guardian component running under a service identity with only the privileges required for KidOS enforcement. The child shell does not inherit that identity or its privileges.

The first vertical slice packages the KidOS shell as a current-user NSIS application and defines the Guardian installation/security boundary. Service registration/elevation must be performed only by a future explicitly designed and signed installer step; the renderer itself must never self-elevate.

### Secure-store boundary

Parent secret material is stored through the secure-store abstraction. Windows persistence uses operating-system protected storage. There is no plaintext fallback.

### Network/content boundary

Websites, redirects, downloads, media, remote moderation services, and social platforms are untrusted external inputs. KidOS policy remains authoritative even when external classifiers or reputation sources are unavailable or incorrect.

## Threats and mitigations

### Child-user bypass attempts

Threats include closing the shell, launching unrestricted apps, navigating around protected browser controls, modifying local files, or attempting to reach normal Windows surfaces.

Mitigations in the v1 architecture include a restricted child-facing command surface, Guardian-owned privileged actions, parent authorization for policy changes, fail-closed navigation/download policy, and the planned Windows shell/Assigned Access lockdown layer. KidOS does not yet claim that all Windows escape paths are eliminated until the later lockdown/installer integration is fully deployed and verified.

### Malicious websites and redirects

Threats include unsafe content, deceptive domains, redirect chains, script URLs, phishing pages, and attempts to bypass SafeSearch parameters.

Mitigations include URL normalization, scheme restrictions, domain policy, provider-specific SafeSearch rewriting, redirect re-evaluation, media-safety classification, and `allow | block | require_parent` policy decisions. Unsupported or malformed high-risk destinations fail closed.

### Hostile downloads

Threats include executable/script payloads, misleading double extensions, archive-wrapped executables, MIME/extension mismatches, and files that appear benign by display name.

Mitigations include policy-core download classification, disguised-file detection, parent download mode, and Guardian-controlled decisions. High-risk downloads are blocked or parent-gated according to the active parent policy.

### Renderer compromise

Threat: a malicious page or renderer exploit attempts to cross into privileged KidOS capabilities.

Mitigations include keeping the renderer unprivileged, a restrictive content security policy, narrow Tauri commands, no generic OS command execution bridge, typed request validation, and Guardian enforcement outside renderer authority.

A renderer compromise must not automatically become an administrator or Guardian compromise.

### Local configuration tampering

Threat: a child or local process edits ordinary application files to weaken policy.

Mitigations include Guardian ownership of privileged policy changes, integrity-checked policy snapshots, last-known-valid fallback, strict baseline fallback, and secure parent authorization. Corrupt or unavailable policy must not result in unrestricted mode.

### Parent PIN brute force

Threat: repeated guesses attempt to unlock parent controls.

Mitigations include salted Argon2id verification, no plaintext PIN persistence, and command/service rate limiting. Five failed attempts trigger a 60-second lockout in the current design.

### IPC spoofing and replay

Threats include malformed requests, unsupported protocol versions, replayed nonces, forged child requests for parent operations, and session confusion.

Mitigations include versioned typed IPC, non-empty session IDs/nonces, replay tracking per session, strict actor authorization, and rejection of child attempts to mutate parent policy.

### Guardian failure

Threat: Guardian is unavailable, crashes, or cannot load a valid policy.

Mitigation: KidOS enters restricted safe mode. Guardian unavailability must never broaden permissions. The state loader prefers current valid policy, then last-known-valid policy, then strict baseline.

### AI/media-classifier error or outage

Threats include false negatives, false positives, classifier crashes, malformed model output, or unavailable remote moderation.

Mitigations include local-first classification, schema validation, Rust policy authority, optional remote escalation only for uncertain/high-risk cases, and fail-closed parent gating where confidence is inadequate. AI never overrides explicit parent/Guardian policy.

### Safety-event privacy leakage

Threat: safety logging becomes a surveillance database or retains sensitive child content.

Mitigations include minimal SQLite records: timestamp, action class, normalized domain when necessary, decision, reason, and coarse media safety metadata. Full URLs, search queries, page text, parent credentials, child prompts, and viewed media are not persisted by the safety-event store. Parent-authorized clearing is supported.

## Installer and update security

KidOS v1 uses a Windows NSIS bundle configured for current-user installation of the shell. The child renderer is not automatically elevated.

Automatic updater configuration is deliberately absent. KidOS must not enable silent update trust until code signing, update-signature verification, release provenance, rollback policy, and update-channel security are intentionally designed and tested.

A future Guardian Service installer may require elevation to register or update a Windows service. That elevation must occur in an installer/service-management boundary, not inside the child-facing renderer.

## Service identity principle

The Guardian Service should run with the least-privileged Windows service identity that can enforce the required controls. It must expose only KidOS-specific IPC operations. It must not become a general-purpose privileged broker.

Any future capability requiring additional Windows privileges must be reviewed as a new trust-boundary change and added to this threat model before implementation.

## Explicit non-goals

KidOS v1 is not:

- a replacement Windows kernel;
- a guarantee that no inappropriate content can ever be displayed;
- a replacement for platform-side social-media moderation;
- a keylogger or private-message surveillance tool;
- an unrestricted remote-administration system;
- a general arbitrary-command launcher;
- a system that stores a full searchable history of a child's browsing;
- a promise that a current-user shell installer alone can prevent every route back into ordinary Windows.

## Release gates

A KidOS Windows release is acceptable only when:

1. JavaScript tests/builds pass.
2. Rust tests/checks pass on Windows.
3. packaging metadata passes the least-privilege configuration test.
4. the Tauri NSIS build succeeds.
5. the installer artifact is produced by CI.
6. source-safety scans continue to reject plaintext browser PIN storage, generic OS command execution bridges, and viewed-media persistence identifiers.
7. no updater is enabled without a separately reviewed signing/update design.
