import { describe, expect, it } from 'vitest';
import type {
  ApprovedDesktopApp,
  LockdownState,
  ManagedAccount,
  ParentUnlockGrant,
  PlatformLockdownCapability,
} from './lockdown';

describe('lockdown contracts', () => {
  it('supports only the KidOS lockdown lifecycle states', () => {
    const states: LockdownState[] = [
      'unmanaged',
      'preparing',
      'locked',
      'parent_unlocked',
      'restricted_safe_mode',
    ];
    expect(states).toHaveLength(5);
  });

  it('uses structured account, app, capability and unlock values', () => {
    const account: ManagedAccount = { id: 'child-1', displayName: 'Child', role: 'standard' };
    const app: ApprovedDesktopApp = { id: 'kidos', displayName: 'KidOS', executablePath: 'C:\\Program Files\\KidOS\\KidOS.exe' };
    const capability: PlatformLockdownCapability = { platform: 'windows', supported: true, mechanism: 'assigned_access' };
    const grant: ParentUnlockGrant = { grantedAt: '2026-09-02T23:00:00Z', expiresAt: '2026-09-02T23:15:00Z' };

    expect(account.role).toBe('standard');
    expect(app.executablePath).toContain('KidOS.exe');
    expect(capability.mechanism).toBe('assigned_access');
    expect(grant.expiresAt).toBeTruthy();
  });
});
