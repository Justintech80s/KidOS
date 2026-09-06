# KidOS Security Policy

KidOS is designed as a child-safe Windows computing environment. Security-sensitive behavior is intentionally kept behind a privileged Windows Guardian service rather than trusting the child-facing interface.

## Security boundaries

- The KidOS React/Tauri shell is treated as an unprivileged presentation layer.
- `KidOSGuardian` runs as a Windows LocalSystem service and owns privileged policy decisions.
- Parent-sensitive actions require Guardian-side PIN verification.
- Windows Assigned Access is used to restrict the managed child account.
- Browser navigations, downloads, approved desktop applications, quarantine actions, and recovery operations are validated at privileged boundaries.
- The local media classifier binds to localhost and requires a protected random token.
- Automatic updates are restricted to the official KidOS GitHub Releases namespace, require SHA-256 integrity, valid Authenticode signing, a publisher certificate pinned during the trusted installation, and anti-rollback version checks.
- Recovery logic fails closed instead of silently disabling safety controls.

## Reporting a vulnerability

Do not publish a working bypass, parent PIN bypass, child-account escape path, unsafe-content bypass, update-signing bypass, or LocalSystem privilege issue before it can be fixed.

Open a private GitHub security advisory for this repository when available. Include:

- affected KidOS version or commit
- Windows version and edition
- reproduction steps
- whether the issue requires child, parent, administrator, or SYSTEM privileges
- expected and actual behavior
- logs or screenshots that do not expose private child data

## Release security gates

A production Windows build should not be considered release-ready unless all applicable automated tests pass, the clean Windows VM validation passes, the real rebooted child-session validation passes on a disposable Windows environment, and the release artifacts are signed by the trusted KidOS publisher certificate.

## Non-goals

KidOS is not a new Windows kernel and does not claim to protect an administrator who intentionally disables or removes KidOS. The trusted parent/administrator account remains the recovery and maintenance authority for the computer.
