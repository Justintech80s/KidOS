# KidOS Windows v1 Threat Model

## Purpose

KidOS is a child-focused Windows computing environment that places an unprivileged Tauri/React shell in front of protected creation, browsing, download, parent-control, and Windows Lockdown flows. Windows remains the underlying operating system. KidOS is designed for defense in depth and fail-closed handling of high-risk or unknown actions; it does not claim perfect filtering or immunity from every Windows bypass.

## Protected assets

KidOS protects parent authorization secrets, child/profile policy, Guardian state and last-known-valid state, privileged policy mutations, protected navigation/download/media decisions, privacy-minimized safety records, and the shell-to-Guardian trust boundary. It does not retain a long-term full browsing/search history by default.

## Trust boundaries

### Child-facing shell

The Tauri/React renderer is untrusted relative to privileged operations. It must not run as administrator, execute arbitrary OS commands, submit raw Assigned Access XML, directly mutate Guardian policy, configure administrative-tool allowlisting, or access parent secrets.

### Tauri command boundary

The shell uses named, typed KidOS commands only. Generic command dispatch, PowerShell/CMD bridges, raw XML submission, and renderer-controlled elevation are prohibited.

### Guardian and Windows Lockdown boundary

Guardian is the authority for privileged lockdown state. Windows Lockdown uses Assigned Access restricted-user profiles for a validated standard child account. KidOS generates the allowlist from structured application records and explicitly rejects common administrative executables. The managed account must exist before configuration and must not be an administrator.

Assigned Access enforcement is treated as external security state that Guardian must inspect rather than assume. Configuration/application mismatch, adapter failure, or inability to confirm protection causes Restricted Safe Mode. Parent maintenance access is temporary and authorization-gated. Removal is parent-authorized.

Some Assigned Access restricted-user policies are device-scoped and can affect administrator accounts too. KidOS therefore keeps the parent/admin account outside the child profile and treats provisioning/recovery as a privileged operation requiring explicit verification.

The Windows production adapter currently fails closed until the privileged Assigned Access host binding is implemented. KidOS must not report a device as locked merely because XML was generated.

### Secure-store boundary

Parent secret material uses the secure-store abstraction and OS-protected persistence. There is no plaintext fallback.

### Network/content boundary

Websites, redirects, downloads, media, remote moderation services, and social platforms are untrusted. KidOS policy remains authoritative when external classifiers or reputation services fail.

## Primary threats and mitigations

### Child-user Windows escape

Threats include closing KidOS, launching unrestricted programs, reaching Settings/File Explorer/Run/terminal surfaces, or manipulating local files. Mitigations include Assigned Access restricted-user policy, explicit desktop-app allowlisting, generated AppLocker restrictions, hidden/minimized Windows surfaces, Guardian-owned state, and fail-closed status verification. KidOS does not disable Windows security mechanisms or claim to replace the Windows kernel.

### Administrative tool allowlisting

Threat: a renderer or malformed profile attempts to permit command shells, scripting hosts, registry tools, Windows Terminal, or MMC. Mitigation: structured allowlist validation rejects known administrative executables before Assigned Access XML generation, and the renderer has no raw XML or generic command surface.

### Renderer compromise

Threat: malicious content or a renderer exploit attempts to cross into privileged capabilities. Mitigations include an unprivileged renderer, restrictive CSP, named Tauri commands, typed validation, no generic OS execution, no raw Assigned Access XML, and Guardian enforcement outside renderer authority.

### Local configuration tampering or lockdown drift

Threat: configuration is edited, removed, fails to apply, or no longer matches Guardian expectations. Mitigations include typed inspection, last-known-valid configuration, explicit lifecycle states, rollback/recovery behavior, and Restricted Safe Mode on mismatch or uncertainty.

### Malicious websites, redirects, downloads, and media

Mitigations include URL normalization, scheme/domain policy, SafeSearch rewriting, redirect re-evaluation, download classification, disguised-file detection, hybrid media classification, and `allow | block | require_parent` decisions. High-risk or uncertain actions fail closed or require a parent.

### Parent PIN brute force and leakage

Parent PINs are protected by the secure-store design and Argon2id verification. Plaintext browser storage is prohibited. Five failed attempts trigger the current 60-second lockout design.

### IPC spoofing and replay

Versioned typed IPC, actor authorization, session/nonces, and replay tracking reject malformed, replayed, or child-forged parent operations.

### Guardian failure

Guardian unavailability must never broaden access. KidOS enters Restricted Safe Mode and prefers current valid state, then last-known-valid state, then a strict baseline.

### Safety-event privacy leakage

Safety records are minimized. Full URLs, search queries, page text, parent credentials, child prompts, and viewed media are not persisted by the safety-event store. Parent-authorized clearing is supported.

## Installer and update security

The current NSIS bundle installs the KidOS shell for the current user. Guardian service registration/elevation is not yet shipped as a production privileged service installer and must not be represented as complete. Future service installation must occur in an explicitly reviewed installer/service boundary, never through renderer self-elevation.

Automatic updater trust remains disabled until code signing, update-signature verification, release provenance, rollback policy, and update-channel security are designed and tested.

## Explicit non-goals

KidOS is not a replacement Windows kernel, a guarantee that inappropriate content can never appear, a social-platform moderation replacement, a keylogger/private-message surveillance system, an unrestricted remote-administration system, a general arbitrary-command launcher, or a full searchable child-browsing-history database.

## Release gates

A Windows Lockdown release is acceptable only when JavaScript tests/builds pass; Rust tests/checks pass; the lockdown source-safety scanner passes; packaging metadata passes; the Tauri NSIS installer builds; the installer artifact is produced by release CI; renderer code contains no generic OS command bridge/raw Assigned Access XML/plaintext PIN persistence/child-configurable admin-tool allowlisting; and exact-head end-to-end Lockdown verification passes. Production marketing must not claim the privileged Guardian service or real Assigned Access application is complete until those bindings are implemented and verified on Windows hardware.
