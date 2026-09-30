# KidOS Platform Adapter Layer

The platform adapter layer separates **KidOS security policy** from **operating-system-specific enforcement**.

## Boundary

Guardian and the OS foundation decide what KidOS requires:

- child session containment
- fail-closed state
- parent-authorized maintenance windows
- approved application access
- recovery before returning to normal mode

A platform adapter is responsible only for translating those requirements into the host platform's supported mechanism.

```text
KidOS policy + OS foundation
            |
            v
PlatformLockdownService
            |
      Platform adapter
       /      |      \
 Windows   Linux   Android
```

## Current implementation

### Windows

The Windows adapter is active now.

`ProductionWindowsPlatformAdapter` wraps the existing Windows Assigned Access enforcement. Guardian no longer needs its core lockdown state machine to be Windows-specific. The Windows profile is converted into a `PlatformLockdownConfig`, then the generic `PlatformLockdownService` applies it through the Windows adapter.

### Linux

The Linux adapter is now implemented as a production-targeted systemd adapter. It validates a dedicated child account and KidOS shell path, writes the protected child-session service and target, enables/starts the KidOS child target, verifies that containment remains active, and rolls back a partial apply if startup fails.

This is the enforcement adapter layer only. A native Linux Guardian host, Linux IPC, graphical session startup, and reproducible boot image are separate milestones before KidOS can be called a standalone Linux-based OS distribution.

### Android

The generic contract also includes `PlatformKind::Android`. A later Android adapter should map it to Device Owner / managed-device APIs, allowlisted packages, lock-task/kiosk policy, and parent-controlled maintenance.

## Fail-closed behavior

The generic service never treats an adapter error as success.

- wrong platform config -> Restricted Safe Mode
- apply failure -> Restricted Safe Mode
- a configured session that later reports missing/unsupported containment -> Restricted Safe Mode
- parent unlock expires -> locked state resumes
- removal requires parent authorization

## Why this exists

KidOS can now keep one security model while changing the low-level enforcement mechanism by platform. Windows, Linux, and Android can evolve independently without copying platform-specific code into the policy engine.
