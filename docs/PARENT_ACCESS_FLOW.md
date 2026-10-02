# KidOS Parent Access Flow

This document records the intended Parent Access behavior for KidOS Desktop and the protected KidOS edition.

## First-time setup

1. Parent opens Parent Access.
2. If no parent credential exists, KidOS shows **Set Up Parent Access** instead of a normal unlock form.
3. Parent creates a 4–8 digit PIN and confirms it.
4. KidOS stores only a protected credential representation, never the plain PIN.
5. Parent selects or creates the child profile the controls apply to.
6. Microsoft Family Safety may be offered as an optional external Windows-family-management service; it does not replace KidOS Parent Access.

## Normal locked state

- Parent controls remain locked during ordinary child use.
- A child may see that Parent Access exists but cannot view or change protected settings.
- Protected controls include Approved Apps, Activity, Time Limits, and Protection Settings.

## Unlock flow

1. Parent enters the PIN.
2. KidOS verifies it.
3. A successful verification opens the Parent Workspace for a bounded session.
4. The session expires automatically after a timeout, app close, explicit lock, or protected-edition security event.
5. Repeated invalid PIN attempts are rate-limited and can temporarily lock PIN entry.

## Parent Workspace

### Approved Apps
- Choose which apps and approved online services the child may use.
- Examples: Kiddle, Khan Academy Kids, PBS KIDS, YouTube Kids, and other parent-approved services.

### Activity
- Show KidOS activity such as apps opened, searches attempted, blocked destinations, and screen usage.
- Avoid exposing unrelated Windows activity the KidOS edition does not actually collect.

### Time Limits
- Configure daily usage limits, schedules, bedtime, and category-specific limits such as learning or play time.

### Protection Settings
- Configure Safe Browser rules, download policy, approved websites, content restrictions, media safety, and age/profile settings.

## Child profiles

- The current child profile appears in Parent Access.
- If multiple child profiles exist, the parent can switch which profile is being managed.

## Microsoft Family Safety

- Expose Microsoft Family Safety as an optional external tool.
- Do not imply KidOS is Microsoft Family Safety or that the services are integrated unless a supported Microsoft integration is implemented.
- Microsoft Family Safety retains its own account, permissions, and policy behavior.

## Electron Desktop behavior

KidOS Desktop Electron is a conventional Windows desktop preview/application edition.

- It may expose Parent Access UI and parent-managed KidOS settings that are implemented locally.
- It must visibly indicate that **Guardian enforcement, Assigned Access, Restricted Safe Mode, and Windows child-account lockdown are not active**.
- It must not claim Windows-level protection is enabled when Guardian is unavailable.

## Protected KidOS behavior

The Tauri + Rust Guardian edition remains the target for deeper Windows enforcement. Parent Access should use Guardian-backed verification and policy controls when that protected runtime is active.
