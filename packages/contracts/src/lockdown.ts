export type LockdownState =
  | 'unmanaged'
  | 'preparing'
  | 'locked'
  | 'parent_unlocked'
  | 'restricted_safe_mode';

export type ManagedAccountRole = 'standard' | 'administrator' | 'unknown';

export interface ManagedAccount {
  readonly id: string;
  readonly displayName: string;
  readonly role: ManagedAccountRole;
}

export interface ApprovedDesktopApp {
  readonly id: string;
  readonly displayName: string;
  readonly executablePath: string;
}

export interface PlatformLockdownCapability {
  readonly platform: 'windows' | 'macos' | 'linux' | 'android' | 'ios';
  readonly supported: boolean;
  readonly mechanism: 'assigned_access' | 'platform_specific' | 'unsupported';
  readonly reason?: string;
}

export interface ParentUnlockGrant {
  readonly grantedAt: string;
  readonly expiresAt: string;
}

export interface LockdownStatus {
  readonly state: LockdownState;
  readonly capability: PlatformLockdownCapability;
  readonly managedAccount?: ManagedAccount;
  readonly parentUnlock?: ParentUnlockGrant;
  readonly reason?: string;
}
