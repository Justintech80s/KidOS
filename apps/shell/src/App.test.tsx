import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import App from './App';
import type { KidOSApi } from './lib/kidos-api';

const capability = { platform: 'windows', supported: true, mechanism: 'assigned_access' } as const;

const healthyApi: KidOSApi = {
  async planWorkspace() {
    return {
      kind: 'story',
      title: 'Story',
      capabilities: ['story'],
    };
  },
  async evaluateNavigation() {
    return 'allow';
  },
  async evaluateDownload() {
    return 'allow';
  },
  async guardianStatus() {
    return 'healthy';
  },
  async lockdownStatus() {
    return { state: 'unmanaged', capability };
  },
  async configureWindowsLockdown(request) {
    return { state: 'preparing', capability, managedAccount: request.account };
  },
  async requestParentMaintenanceUnlock() {
    return { grantedAt: '2026-09-03T00:45:00Z', expiresAt: '2026-09-03T01:00:00Z' };
  },
  async removeWindowsLockdown() {
    return { state: 'unmanaged', capability };
  },
};

describe('KidOS shell', () => {
  it('shows the creation-first prompt only after Guardian is healthy', async () => {
    render(<App api={healthyApi} />);
    expect(screen.getByText('Checking protection...')).toBeTruthy();
    expect(await screen.findByText('What do you want to create?')).toBeTruthy();
  });
});
