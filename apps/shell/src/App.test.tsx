import { cleanup, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import App from './App';
import type { KidOSApi } from './lib/kidos-api';

const capability = { platform: 'windows', supported: true, mechanism: 'assigned_access' } as const;

const healthyApi: KidOSApi = {
  async planWorkspace() { return { kind: 'story', title: 'Story', capabilities: ['story'] }; },
  async evaluateNavigation() { return 'allow'; },
  async evaluateDownload() { return 'allow'; },
  async guardianStatus() { return 'healthy'; },
  async lockdownStatus() { return { state: 'unmanaged', capability }; },
  async configureWindowsLockdown(request) { return { state: 'preparing', capability, managedAccount: request.account }; },
  async requestParentMaintenanceUnlock() { return { grantedAt: '2026-09-03T00:45:00Z', expiresAt: '2026-09-03T01:00:00Z' }; },
  async removeWindowsLockdown() { return { state: 'unmanaged', capability }; },
};

afterEach(() => {
  cleanup();
});

describe('KidOS shell', () => {
  it('shows the protected 2026 home only after Guardian is healthy', async () => {
    render(<App api={healthyApi} />);
    expect(screen.getByText('Checking protection...')).toBeTruthy();
    expect(await screen.findByTestId('kidos-shell')).toBeTruthy();
    expect(screen.getAllByRole('button', { name: /Safe Browser/ }).length).toBeGreaterThan(0);
    expect(screen.getAllByRole('button', { name: /KidOS AI/ }).length).toBeGreaterThan(0);
  });

  it('never renders the production child shell when Guardian reports restricted safe mode', async () => {
    const restrictedApi: KidOSApi = { ...healthyApi, async guardianStatus() { return 'restricted_safe_mode'; } };
    render(<App api={restrictedApi} />);
    expect(await screen.findByRole('heading', { name: 'Restricted safe mode' })).toBeTruthy();
    expect(screen.queryByTestId('kidos-shell')).toBeNull();
  });

  it('fails closed into restricted safe mode when Guardian status cannot be read', async () => {
    const unavailableApi: KidOSApi = { ...healthyApi, async guardianStatus() { throw new Error('guardian unavailable'); } };
    render(<App api={unavailableApi} />);
    expect(await screen.findByRole('heading', { name: 'Restricted safe mode' })).toBeTruthy();
    expect(screen.queryByTestId('kidos-shell')).toBeNull();
  });
});
