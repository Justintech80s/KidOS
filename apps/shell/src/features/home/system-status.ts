export type ProtectionState = 'active' | 'degraded' | 'offline';

export interface RawKidOSSystemStatus {
  guardianReachable: boolean;
  guardianEnforcing: boolean;
  classifierReachable: boolean;
  classifierReady: boolean;
  filterEnforcing: boolean;
  recoveryAvailable: boolean;
  observedAt: number;
}

export interface KidOSSystemStatus {
  guardianRunning: boolean;
  classifierRunning: boolean;
  safeMode: ProtectionState;
  internetFilter: ProtectionState;
  classifierReady: boolean;
  recoveryAvailable: boolean;
  observedAt: number;
}

export const OFFLINE_KIDOS_STATUS: KidOSSystemStatus = {
  guardianRunning: false,
  classifierRunning: false,
  safeMode: 'offline',
  internetFilter: 'offline',
  classifierReady: false,
  recoveryAvailable: false,
  observedAt: 0,
};

const STALE_AFTER_MS = 15_000;

export function normalizeKidOSSystemStatus(
  raw: RawKidOSSystemStatus,
  now = Date.now(),
): KidOSSystemStatus {
  const stale = now - raw.observedAt > STALE_AFTER_MS;
  const guardianRunning = raw.guardianReachable && !stale;
  const classifierRunning = raw.classifierReachable && !stale;

  const safeMode: ProtectionState = !guardianRunning
    ? 'offline'
    : raw.guardianEnforcing
      ? 'active'
      : 'degraded';

  const internetFilter: ProtectionState = !guardianRunning
    ? 'offline'
    : raw.filterEnforcing
      ? 'active'
      : 'degraded';

  return {
    guardianRunning,
    classifierRunning,
    safeMode,
    internetFilter,
    classifierReady: classifierRunning && raw.classifierReady,
    recoveryAvailable: raw.recoveryAvailable,
    observedAt: raw.observedAt,
  };
}
