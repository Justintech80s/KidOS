import { describe, expect, it } from 'vitest';
import { normalizeKidOSSystemStatus } from './system-status';

const NOW = 1_000_000;

function raw(overrides: Partial<Parameters<typeof normalizeKidOSSystemStatus>[0]> = {}) {
  return {
    guardianReachable: true,
    guardianEnforcing: true,
    classifierReachable: true,
    classifierReady: true,
    filterEnforcing: true,
    recoveryAvailable: true,
    observedAt: NOW,
    ...overrides,
  };
}

describe('normalizeKidOSSystemStatus', () => {
  it('reports active only when authoritative enforcement is healthy', () => {
    expect(normalizeKidOSSystemStatus(raw(), NOW)).toMatchObject({
      safeMode: 'active',
      internetFilter: 'active',
    });
  });

  it('never reports safe mode active when Guardian is unreachable', () => {
    expect(normalizeKidOSSystemStatus(raw({ guardianReachable: false }), NOW).safeMode).toBe('offline');
  });

  it('reports degraded rather than active when Guardian is reachable but not enforcing', () => {
    expect(normalizeKidOSSystemStatus(raw({ guardianEnforcing: false }), NOW).safeMode).toBe('degraded');
  });

  it('never reports filtering active when filter enforcement is false', () => {
    expect(normalizeKidOSSystemStatus(raw({ filterEnforcing: false }), NOW).internetFilter).toBe('degraded');
  });

  it('invalidates stale green state after 15 seconds', () => {
    const status = normalizeKidOSSystemStatus(raw({ observedAt: NOW - 16_000 }), NOW);
    expect(status.safeMode).not.toBe('active');
    expect(status.internetFilter).not.toBe('active');
  });
});
