import { describe, expect, it } from 'vitest';
import { normalizeKidOSSystemStatus } from './system-status';

const NOW = 1_000_000;
const raw = (overrides = {}) => ({ guardianReachable: true, guardianEnforcing: true, classifierReachable: true, classifierReady: true, filterEnforcing: true, recoveryAvailable: true, observedAt: NOW, ...overrides });

describe('normalizeKidOSSystemStatus', () => {
  it('reports active only when enforcement is healthy', () => expect(normalizeKidOSSystemStatus(raw(), NOW)).toMatchObject({ safeMode: 'active', internetFilter: 'active' }));
  it('fails closed when Guardian is unreachable', () => expect(normalizeKidOSSystemStatus(raw({ guardianReachable: false }), NOW).safeMode).toBe('offline'));
  it('degrades when Guardian is not enforcing', () => expect(normalizeKidOSSystemStatus(raw({ guardianEnforcing: false }), NOW).safeMode).toBe('degraded'));
  it('does not show an active filter without enforcement', () => expect(normalizeKidOSSystemStatus(raw({ filterEnforcing: false }), NOW).internetFilter).toBe('degraded'));
  it('invalidates stale green state', () => expect(normalizeKidOSSystemStatus(raw({ observedAt: NOW - 16_000 }), NOW).safeMode).not.toBe('active'));
});
