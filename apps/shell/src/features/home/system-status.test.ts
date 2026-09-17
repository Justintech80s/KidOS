import { describe, expect, it } from 'vitest';
import { normalizeKidOSSystemStatus } from './system-status';

const NOW = 1_000_000;
const raw = (overrides = {}) => ({
  guardianReachable: true,
  guardianEnforcing: true,
  classifierReachable: true,
  classifierReady: true,
  filterEnforcing: true,
  recoveryAvailable: true,
  observedAt: NOW,
  ...overrides,
});

describe('normalizeKidOSSystemStatus', () => {
  it('reports active only when authoritative enforcement is healthy', () => {
    expect(normalizeKidOSSystemStatus(raw(), NOW)).toMatchObject({
      safeMode: 'active',
      internetFilter: 'active',
      classifierReady: true,
    });
  });

  it('fails closed when Guardian IPC is unavailable', () => {
    expect(normalizeKidOSSystemStatus(raw({ guardianReachable: false }), NOW)).toMatchObject({
      guardianRunning: false,
      safeMode: 'offline',
      internetFilter: 'offline',
    });
  });

  it('fails closed when policy is invalid and Guardian cannot enforce it', () => {
    expect(normalizeKidOSSystemStatus(raw({ guardianEnforcing: false, filterEnforcing: false }), NOW)).toMatchObject({
      safeMode: 'degraded',
      internetFilter: 'degraded',
    });
  });

  it('does not report media classification ready when the classifier is unavailable', () => {
    expect(normalizeKidOSSystemStatus(raw({ classifierReachable: false, classifierReady: false }), NOW)).toMatchObject({
      classifierRunning: false,
      classifierReady: false,
    });
  });

  it('does not show an active internet filter when filter enforcement is unavailable', () => {
    expect(normalizeKidOSSystemStatus(raw({ filterEnforcing: false }), NOW).internetFilter).toBe('degraded');
  });

  it('invalidates stale IPC state instead of preserving a stale green status', () => {
    expect(normalizeKidOSSystemStatus(raw({ observedAt: NOW - 16_000 }), NOW)).toMatchObject({
      guardianRunning: false,
      classifierRunning: false,
      safeMode: 'offline',
      internetFilter: 'offline',
      classifierReady: false,
    });
  });
});
