# KidOS Linux Platform Adapter

This adapter is the first production-targeted Linux implementation of the shared KidOS platform lockdown contract. Policy and state transitions stay in `guardian-service`; privileged file writes and systemd execution live in the Guardian host.

## What it does

When given a validated `LinuxLockdownProfile`, the adapter:

1. Requires a privileged/root execution context.
2. Writes the validated KidOS Linux lockdown profile to `/var/lib/kidos/lockdown.json`.
3. Creates `kidos-child-session.service`.
4. Creates `kidos-child.target`.
5. Reloads systemd.
6. Enables the child-session service and target.
7. Starts the protected KidOS child target.
8. Reports the platform as configured only when the required files exist and the systemd child service is enabled and the child target is active.

The child session unit launches only the configured KidOS shell under the dedicated child account. It also enables several systemd hardening controls including `NoNewPrivileges`, kernel/control-group protection, an empty capability bounding set, and a restrictive umask.

The privileged host does not accept arbitrary systemctl argument arrays. `guardian-service` emits a closed `SystemdAction` enum and `crates/guardian-host/src/linux_platform.rs` maps that enum to a reviewed allowlist of exact systemctl arguments.

## Fail closed

A partial apply is rolled back. If systemd cannot start the protected child target, the adapter removes the partial KidOS unit/config files and disables the target/service. The generic `PlatformLockdownService` then moves KidOS into Restricted Safe Mode.

A configuration marked for Android or Windows is rejected by the Linux adapter.

## Current boundary

This is a real Linux enforcement adapter, but it is not yet a complete bootable KidOS Linux distribution.

The next Linux milestones are:

- run the KidOS Guardian host natively as a long-lived Linux systemd service;
- provide Linux IPC between the shell and Guardian;
- add the graphical session/compositor startup path;
- package the shell, Guardian, classifier, policy files, and systemd units into a reproducible image;
- boot that image in a VM before producing an ISO/USB image.

## Approved apps

The validated Linux profile records approved application paths so the KidOS policy layer can retain the same application model across platforms. The adapter itself starts only the KidOS shell; child-facing app launches remain subject to KidOS policy/capability enforcement rather than exposing a normal Linux desktop.

## Safety

Do not run this adapter against a primary Linux workstation yet. Its intended environment is a dedicated KidOS VM/image while the Linux path is still experimental.
