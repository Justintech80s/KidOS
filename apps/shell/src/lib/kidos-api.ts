import { invoke } from '@tauri-apps/api/core';
import type { LockdownStatus, ManagedAccount, ApprovedDesktopApp, ParentUnlockGrant, ParentPolicyConfig, WorkspacePlan } from '@kidos/contracts';

export type PolicyDecision = 'allow' | 'block' | 'require_parent';
export type GuardianStatus = 'healthy' | 'restricted_safe_mode';

export interface ConfigureWindowsLockdownRequest {
  account: ManagedAccount;
  approvedApps: readonly ApprovedDesktopApp[];
}

export interface ParentVerification {
  authorized: boolean;
  locked: boolean;
}

export interface KidOSApi {
  planWorkspace(prompt: string): Promise<WorkspacePlan>;
  evaluateNavigation(url: string): Promise<PolicyDecision>;
  evaluateDownload(fileName: string, mimeType: string): Promise<PolicyDecision>;
  openProtectedBrowser?(url: string): Promise<void>;
  guardianStatus(): Promise<GuardianStatus>;
  configureParentPin?(pin: string, currentPin?: string): Promise<void>;
  verifyParentPin?(pin: string): Promise<ParentVerification>;
  saveParentPolicy?(pin: string, policy: ParentPolicyConfig): Promise<{ saved: boolean }>;
  lockdownStatus(): Promise<LockdownStatus>;
  configureWindowsLockdown(request: ConfigureWindowsLockdownRequest): Promise<LockdownStatus>;
  requestParentMaintenanceUnlock(pin: string, durationMinutes: number): Promise<ParentUnlockGrant>;
  removeWindowsLockdown(pin: string): Promise<LockdownStatus>;
}

export const tauriKidOSApi: KidOSApi = {
  planWorkspace(prompt) { return invoke<WorkspacePlan>('plan_workspace', { prompt }); },
  evaluateNavigation(url) { return invoke<PolicyDecision>('evaluate_navigation_with_parent_policy', { url }); },
  evaluateDownload(fileName, mimeType) { return invoke<PolicyDecision>('evaluate_download_with_parent_policy', { fileName, mimeType }); },
  openProtectedBrowser(url) { return invoke<void>('open_protected_browser', { url }); },
  guardianStatus() { return invoke<GuardianStatus>('get_guardian_status'); },
  configureParentPin(pin, currentPin) { return invoke<void>('configure_parent_pin', { pin, currentPin: currentPin ?? null }); },
  verifyParentPin(pin) { return invoke<ParentVerification>('verify_parent_pin', { pin }); },
  saveParentPolicy(pin, policy) { return invoke<{ saved: boolean }>('save_parent_policy', { pin, policy }); },
  lockdownStatus() { return invoke<LockdownStatus>('lockdown_status'); },
  configureWindowsLockdown(request) { return invoke<LockdownStatus>('configure_windows_lockdown', { request }); },
  requestParentMaintenanceUnlock(pin, durationMinutes) { return invoke<ParentUnlockGrant>('request_parent_maintenance_unlock', { pin, durationMinutes }); },
  removeWindowsLockdown(pin) { return invoke<LockdownStatus>('remove_windows_lockdown', { pin }); },
};
