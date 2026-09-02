import { invoke } from '@tauri-apps/api/core';
import type { WorkspacePlan } from '@kidos/contracts';

export type PolicyDecision = 'allow' | 'block' | 'require_parent';
export type GuardianStatus = 'healthy' | 'restricted_safe_mode';

export interface KidOSApi {
  planWorkspace(prompt: string): Promise<WorkspacePlan>;
  evaluateNavigation(url: string): Promise<PolicyDecision>;
  evaluateDownload(fileName: string, mimeType: string): Promise<PolicyDecision>;
  guardianStatus(): Promise<GuardianStatus>;
}

export const tauriKidOSApi: KidOSApi = {
  planWorkspace(prompt) {
    return invoke<WorkspacePlan>('plan_workspace', { prompt });
  },
  evaluateNavigation(url) {
    return invoke<PolicyDecision>('evaluate_navigation', { url });
  },
  evaluateDownload(fileName, mimeType) {
    return invoke<PolicyDecision>('evaluate_download', { fileName, mimeType });
  },
  guardianStatus() {
    return invoke<GuardianStatus>('get_guardian_status');
  },
};
