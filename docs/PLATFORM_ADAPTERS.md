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

The generic contract already includes `PlatformKind::Linux`, but there is intentionally no production Linux enforcer in this change. The next implementation should map the adapter contract to a dedicated child session, system service supervision, application sandboxing, and network controls on the chosen Linux base.

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
