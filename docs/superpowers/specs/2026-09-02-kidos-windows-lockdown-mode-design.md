# KidOS Windows Lockdown Mode Design

## Goal
Make KidOS the managed child-facing Windows environment while preserving a normal parent/admin Windows account. KidOS uses Microsoft Assigned Access restricted-user mode plus the KidOS Guardian boundary rather than replacing the Windows kernel or using unsupported shell-hiding tricks.

## Platform scope
This design targets Windows 11 Pro, Enterprise, Education, and IoT Enterprise editions that support Assigned Access. Windows is the first lockdown implementation. KidOS remains architected for later downloadable macOS, Linux, Android, and iOS/iPadOS editions, each using its own supported OS security mechanisms rather than pretending Windows Assigned Access is portable.

## Architecture

### Managed child account
Lockdown applies only to an explicitly selected standard child Windows account. The parent/admin account remains outside the KidOS restricted profile. Guardian refuses provisioning when the target resolves to an administrator account or when account identity cannot be validated.

### Assigned Access restricted-user profile
Guardian generates a deterministic Assigned Access configuration for the child account. The profile exposes KidOS and only explicitly approved applications required by policy. The normal unrestricted application surface is not the KidOS security model. Assigned Access/AppLocker restrictions provide the Windows enforcement layer while KidOS policy remains responsible for KidOS web, download, social, AI, and creation decisions.

### Lockdown state machine
Guardian owns these states:
- `unmanaged`: no KidOS Assigned Access profile is active for the child account.
- `preparing`: Guardian is validating prerequisites and preparing a candidate configuration.
- `locked`: a validated configuration is installed and the child must sign in again for it to take effect.
- `parent_unlocked`: a time-bounded parent-authorized maintenance state.
- `restricted_safe_mode`: Guardian cannot establish or validate the expected protection state; child-facing privileged actions fail closed.

The renderer can read state and request typed operations, but it cannot write raw Assigned Access configuration or invoke generic operating-system commands.

### Provisioning boundary
A dedicated Rust Windows lockdown module owns configuration generation, validation, apply/remove adapters, and status inspection. Production Windows integration uses a narrow privileged adapter. Tests use an in-memory/fake adapter. No child-controlled string becomes a PowerShell, CMD, shell, registry, WMI, or CIM command.

### Application allowlist
The default restricted profile includes KidOS plus the minimum Windows components required for the supported Assigned Access experience. Parent-approved desktop applications are represented as validated structured entries, not arbitrary command lines. Dangerous administrative tools such as Command Prompt, PowerShell, Registry Editor, Task Manager, MMC, Windows Terminal, scripting hosts, and package-management shells are never implicitly approved.

### KidOS startup
KidOS is pinned/available as the primary child application inside the restricted profile. Windows Assigned Access applies the restriction on the targeted user's next sign-in. KidOS may additionally use a supported startup registration mechanism, but startup registration is not treated as the security boundary. If KidOS exits unexpectedly, Guardian records the unhealthy session and the next shell launch remains restricted; KidOS does not weaken Assigned Access to recover.

### Parent unlock
Temporary maintenance/unlock requires an already authenticated parent authorization from the Guardian parent-auth boundary. Unlock is time-bounded and represented by an expiry timestamp. Expired authorization automatically returns the state to the managed/locked policy. The child renderer never receives the parent PIN or a reusable privileged token.

### Recovery and rollback
Guardian validates the candidate configuration before applying it and keeps the last known valid KidOS lockdown configuration metadata. If provisioning fails before commit, the existing known-good state remains authoritative. If status inspection indicates a missing, malformed, or unexpected Assigned Access state, Guardian enters `restricted_safe_mode` rather than declaring the device unlocked. Removing KidOS lockdown is a parent-authorized maintenance operation and must not silently alter the parent/admin account.

### Cross-platform boundary
Shared KidOS code will consume a platform-neutral session-control contract. Windows implements that contract using Assigned Access. Future macOS, Linux, Android, and iOS/iPadOS implementations must use supported controls for those platforms and may expose different enforcement capabilities. Shared UI must not assume every platform can provide Windows-level app allowlisting.

## Security invariants
- Never target an administrator account for child lockdown.
- Never modify the parent/admin account's Assigned Access identity.
- Never expose a generic shell/command execution bridge.
- Never accept raw Assigned Access XML from the child renderer.
- Never store a parent PIN in browser/local-storage/plaintext configuration.
- Never claim Assigned Access is impossible to bypass or that KidOS replaces Windows.
- Guardian or status-validation failure fails closed into restricted safe mode.
- Parent unlock expires automatically.
- Only validated allowlist entries can reach the Windows configuration generator.

## Microsoft behavior relied upon
KidOS relies on Microsoft's documented Assigned Access restricted-user experience: Windows limits the targeted user to a defined list of applications and applies policy/AppLocker restrictions. Assigned Access is supported on Windows Pro and higher supported editions. Configuration is device-scoped and takes effect for the associated profile on the targeted user's next sign-in. Because some Assigned Access policy settings are device-level, KidOS keeps its generated configuration minimal and documents any setting that can affect users outside the child profile.

## Testing
Test-first coverage includes:
1. standard child account accepted and administrator target rejected;
2. deterministic Assigned Access configuration generation;
3. KidOS included and dangerous administrative tools excluded;
4. parent/admin identity never placed in the restricted config;
5. malformed/unexpected platform status enters restricted safe mode;
6. parent unlock requires parent authorization and expires;
7. apply failure preserves last known valid state;
8. remove/rollback requires parent authorization;
9. renderer API contains named lockdown commands only;
10. source-safety scan rejects generic shell/PowerShell/CMD execution bridges and raw renderer-supplied Assigned Access XML;
11. Windows CI compiles and tests the platform adapter boundary.

## Non-goals for this increment
- Replacing `Explorer.exe` with Shell Launcher.
- Replacing the Windows kernel.
- Disabling secure attention/Ctrl+Alt+Del through unsupported hooks.
- Creating or managing the child's Microsoft account credentials.
- Shipping the macOS/Linux/Android/iOS lockdown implementations in this Windows increment.
- Silently enabling automatic Windows logon.

## Success criteria
A parent can select a validated standard child Windows account, authorize KidOS lockdown, generate/apply a minimal Assigned Access restricted-user configuration, and receive a clear next-sign-in status. The parent/admin account remains normal. The child renderer cannot alter the Windows configuration directly. Invalid Guardian/platform state fails closed, and a parent can perform a time-bounded authorized maintenance unlock or removal through Guardian.