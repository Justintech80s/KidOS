import { invoke } from '@tauri-apps/api/core';
import type { LockdownStatus, ManagedAccount, ApprovedDesktopApp, ParentUnlockGrant, WorkspacePlan } from '@kidos/contracts';

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
  guardianStatus(): Promise<GuardianStatus>;
  configureParentPin?(pin: string): Promise<void>;
  verifyParentPin?(pin: string): Promise<ParentVerification>;
  lockdownStatus(): Promise<LockdownStatus>;
  configureWindowsLockdown(request: ConfigureWindowsLockdownRequest): Promise<LockdownStatus>;
  requestParentMaintenanceUnlock(durationMinutes: number): Promise<ParentUnlockGrant>;
  removeWindowsLockdown(): Promise<LockdownStatus>;
}

export const tauriKidOSApi: KidOSApi = {
  planWorkspace(prompt) { return invoke<WorkspacePlan>('plan_workspace', { prompt }); },
  evaluateNavigation(url) { return invoke<PolicyDecision>('evaluate_navigation', { url }); },
  evaluateDownload(fileName, mimeType) { return invoke<PolicyDecision>('evaluate_download', { fileName, mimeType }); },
  guardianStatus() { return invoke<GuardianStatus>('get_guardian_status'); },
  configureParentPin(pin) { return invoke<void>('configure_parent_pin', { pin }); },
  verifyParentPin(pin) { return invoke<ParentVerification>('verify_parent_pin', { pin }); },
  lockdownStatus() { return invoke<LockdownStatus>('lockdown_status'); },
  configureWindowsLockdown(request) { return invoke<LockdownStatus>('configure_windows_lockdown', { request }); },
  requestParentMaintenanceUnlock(durationMinutes) { return invoke<ParentUnlockGrant>('request_parent_maintenance_unlock', { durationMinutes }); },
  removeWindowsLockdown() { return invoke<LockdownStatus>('remove_windows_lockdown'); },
};
