import { invoke } from '@tauri-apps/api/core';
import type {
  LockdownStatus,
  ManagedAccount,
  ApprovedDesktopApp,
  ParentUnlockGrant,
  ParentPolicyConfig,
  WorkspacePlan,
} from '@kidos/contracts';

export type PolicyDecision = 'allow' | 'block' | 'require_parent';
export type GuardianStatus = 'healthy' | 'restricted_safe_mode';

export interface ConfigureWindowsLockdownRequest {
  account: ManagedAccount;
  approvedApps: readonly ApprovedDesktopApp[];
  parentPin: string;
}

export interface QuarantineItem {
  id: string;
  fileName: string;
  sizeBytes: number;
  modifiedSeconds: number;
  category?: string;
  risk?: string;
  confidence?: number;
  reason?: string;
}

export interface QuarantinePreview {
  mimeType: string;
  dataBase64: string;
}

export type QuarantineAction = 'approve' | 'delete' | 'keep_blocked';

export interface RecoveryStatus {
  guardianHealthy: boolean;
  classifierHealthy: boolean;
  recoveryRequired: boolean;
  recoveryReason?: string;
  policyValid: boolean;
  lockdownState: string;
}

export type RecoveryAction = 'reset_policy_defaults' | 'remove_lockdown' | 'clear_recovery_marker';

export interface ParentVerification {
  authorized: boolean;
  locked: boolean;
}

export interface ApprovedAppSummary {
  id: string;
  displayName: string;
}

export interface KidOSAiResponse {
  available: boolean;
  answer: string;
}

export interface WellbeingSettings {
  dailyMinutes: number;
  breakEveryMinutes: number;
  windDownHour: number;
  largeText: boolean;
  reducedMotion: boolean;
}

export interface KidOSApi {
  planWorkspace(prompt: string): Promise<WorkspacePlan>;
  evaluateNavigation(url: string): Promise<PolicyDecision>;
  evaluateDownload(fileName: string, mimeType: string): Promise<PolicyDecision>;
  openProtectedBrowser?(url: string): Promise<void>;
  guardianStatus(): Promise<GuardianStatus>;
  parentSetupStatus?(): Promise<boolean>;
  configureParentPin?(pin: string, currentPin?: string): Promise<void>;
  verifyParentPin?(pin: string): Promise<ParentVerification>;
  getParentPolicy?(): Promise<ParentPolicyConfig>;
  saveParentPolicy?(pin: string, policy: ParentPolicyConfig): Promise<{ saved: boolean }>;
  lockdownStatus(): Promise<LockdownStatus>;
  configureWindowsLockdown(request: ConfigureWindowsLockdownRequest): Promise<LockdownStatus>;
  requestParentMaintenanceUnlock(pin: string, durationMinutes: number): Promise<ParentUnlockGrant>;
  removeWindowsLockdown(pin: string): Promise<LockdownStatus>;
  listApprovedApps?(): Promise<ApprovedAppSummary[]>;
  launchApprovedApp?(appId: string): Promise<boolean>;
  listQuarantineMedia?(pin: string): Promise<QuarantineItem[]>;
  previewQuarantineMedia?(pin: string, itemId: string): Promise<QuarantinePreview>;
  reviewQuarantineMedia?(pin: string, itemId: string, action: QuarantineAction): Promise<void>;
  getRecoveryStatus?(): Promise<RecoveryStatus>;
  runParentRecovery?(pin: string, action: RecoveryAction): Promise<string>;
  askAi?(query: string): Promise<KidOSAiResponse>;
  getWellbeing?(): Promise<WellbeingSettings>;
  saveWellbeing?(settings: WellbeingSettings): Promise<WellbeingSettings>;
}

export const tauriKidOSApi: KidOSApi = {
  planWorkspace(prompt) { return invoke<WorkspacePlan>('plan_workspace', { prompt }); },
  evaluateNavigation(url) { return invoke<PolicyDecision>('evaluate_navigation_with_parent_policy', { url }); },
  evaluateDownload(fileName, mimeType) { return invoke<PolicyDecision>('evaluate_download_with_parent_policy', { fileName, mimeType }); },
  openProtectedBrowser(url) { return invoke<void>('open_protected_browser', { url }); },
  guardianStatus() { return invoke<GuardianStatus>('get_guardian_status'); },
  parentSetupStatus() { return invoke<boolean>('parent_setup_status'); },
  configureParentPin(pin, currentPin) { return invoke<void>('configure_parent_pin', { pin, currentPin: currentPin ?? null }); },
  verifyParentPin(pin) { return invoke<ParentVerification>('verify_parent_pin', { pin }); },
  saveParentPolicy(pin, policy) { return invoke<{ saved: boolean }>('save_parent_policy', { pin, policy }); },
  lockdownStatus() { return invoke<LockdownStatus>('lockdown_status'); },
  configureWindowsLockdown(request) { return invoke<LockdownStatus>('configure_windows_lockdown', { request }); },
  requestParentMaintenanceUnlock(pin, durationMinutes) { return invoke<ParentUnlockGrant>('request_parent_maintenance_unlock', { pin, durationMinutes }); },
  removeWindowsLockdown(pin) { return invoke<LockdownStatus>('remove_windows_lockdown', { pin }); },
  listQuarantineMedia(pin) { return invoke<QuarantineItem[]>('list_quarantine_media', { pin }); },
  previewQuarantineMedia(pin, itemId) { return invoke<QuarantinePreview>('preview_quarantine_media', { pin, itemId }); },
  reviewQuarantineMedia(pin, itemId, action) { return invoke<void>('review_quarantine_media', { pin, itemId, action }); },
  getRecoveryStatus() { return invoke<RecoveryStatus>('get_recovery_status'); },
  runParentRecovery(pin, action) { return invoke<string>('run_parent_recovery', { pin, action }); },
};

type DesktopBridge = {
  guardianStatus(): Promise<GuardianStatus>;
  parentSetupStatus(): Promise<boolean>;
  planWorkspace(prompt: string): Promise<WorkspacePlan>;
  evaluateNavigation(url: string): Promise<PolicyDecision>;
  evaluateDownload(fileName: string, mimeType: string): Promise<PolicyDecision>;
  openProtectedBrowser(url: string): Promise<void>;
  configureParentPin(pin: string, currentPin?: string): Promise<void>;
  verifyParentPin(pin: string): Promise<ParentVerification>;
  getParentPolicy(): Promise<ParentPolicyConfig>;
  saveParentPolicy(pin: string, policy: ParentPolicyConfig): Promise<{ saved: boolean }>;
  lockdownStatus(): Promise<LockdownStatus>;
  configureWindowsLockdown(request: ConfigureWindowsLockdownRequest): Promise<LockdownStatus>;
  requestParentMaintenanceUnlock(pin: string, durationMinutes: number): Promise<ParentUnlockGrant>;
  removeWindowsLockdown(pin: string): Promise<LockdownStatus>;
  listApprovedApps(): Promise<ApprovedAppSummary[]>;
  launchApprovedApp(appId: string): Promise<boolean>;
  listQuarantineMedia(pin: string): Promise<QuarantineItem[]>;
  previewQuarantineMedia(pin: string, itemId: string): Promise<QuarantinePreview>;
  reviewQuarantineMedia(pin: string, itemId: string, action: QuarantineAction): Promise<void>;
  getRecoveryStatus(): Promise<RecoveryStatus>;
  runParentRecovery(pin: string, action: RecoveryAction): Promise<string>;
  askAi(query: string): Promise<KidOSAiResponse>;
  getWellbeing(): Promise<WellbeingSettings>;
  saveWellbeing(settings: WellbeingSettings): Promise<WellbeingSettings>;
};

function desktopBridge(): DesktopBridge | undefined {
  if (typeof window === 'undefined') return undefined;
  return (window as Window & { kidosDesktop?: DesktopBridge }).kidosDesktop;
}

export function isDesktopPreviewRuntime(): boolean {
  return Boolean(desktopBridge());
}

export const electronKidOSApi: KidOSApi = {
  planWorkspace(prompt) { return desktopBridge()!.planWorkspace(prompt); },
  evaluateNavigation(url) { return desktopBridge()!.evaluateNavigation(url); },
  evaluateDownload(fileName, mimeType) { return desktopBridge()!.evaluateDownload(fileName, mimeType); },
  openProtectedBrowser(url) { return desktopBridge()!.openProtectedBrowser(url); },
  guardianStatus() { return desktopBridge()!.guardianStatus(); },
  parentSetupStatus() { return desktopBridge()!.parentSetupStatus(); },
  configureParentPin(pin, currentPin) { return desktopBridge()!.configureParentPin(pin, currentPin); },
  verifyParentPin(pin) { return desktopBridge()!.verifyParentPin(pin); },
  getParentPolicy() { return desktopBridge()!.getParentPolicy(); },
  saveParentPolicy(pin, policy) { return desktopBridge()!.saveParentPolicy(pin, policy); },
  lockdownStatus() { return desktopBridge()!.lockdownStatus(); },
  configureWindowsLockdown(request) { return desktopBridge()!.configureWindowsLockdown(request); },
  requestParentMaintenanceUnlock(pin, durationMinutes) { return desktopBridge()!.requestParentMaintenanceUnlock(pin, durationMinutes); },
  removeWindowsLockdown(pin) { return desktopBridge()!.removeWindowsLockdown(pin); },
  listApprovedApps() { return desktopBridge()!.listApprovedApps(); },
  launchApprovedApp(appId) { return desktopBridge()!.launchApprovedApp(appId); },
  listQuarantineMedia(pin) { return desktopBridge()!.listQuarantineMedia(pin); },
  previewQuarantineMedia(pin, itemId) { return desktopBridge()!.previewQuarantineMedia(pin, itemId); },
  reviewQuarantineMedia(pin, itemId, action) { return desktopBridge()!.reviewQuarantineMedia(pin, itemId, action); },
  getRecoveryStatus() { return desktopBridge()!.getRecoveryStatus(); },
  runParentRecovery(pin, action) { return desktopBridge()!.runParentRecovery(pin, action); },
  askAi(query) { return desktopBridge()!.askAi(query); },
  getWellbeing() { return desktopBridge()!.getWellbeing(); },
  saveWellbeing(settings) { return desktopBridge()!.saveWellbeing(settings); },
};

export function runtimeKidOSApi(): KidOSApi {
  return desktopBridge() ? electronKidOSApi : tauriKidOSApi;
}
