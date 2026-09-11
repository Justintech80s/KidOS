import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import type { KidOSApi } from '../../lib/kidos-api';
import KidOSHomeShell from './KidOSHomeShell';

const capability = { platform: 'windows', supported: true, mechanism: 'assigned_access' } as const;
const api: KidOSApi = {
  async planWorkspace(prompt) { return { kind: 'story', title: prompt, capabilities: ['story'] }; },
  async evaluateNavigation() { return 'require_parent'; },
  async evaluateDownload() { return 'require_parent'; },
  async guardianStatus() { return 'healthy'; },
  async lockdownStatus() { return { state: 'unmanaged', capability }; },
  async configureWindowsLockdown(request) { return { state: 'preparing', capability, managedAccount: request.account }; },
  async requestParentMaintenanceUnlock() { return { grantedAt: '2026-09-03T00:45:00Z', expiresAt: '2026-09-03T01:00:00Z' }; },
  async removeWindowsLockdown() { return { state: 'unmanaged', capability }; },
};

afterEach(cleanup);

describe('KidOSHomeShell', () => {
  it('renders the eight protected home destinations', () => {
    render(<KidOSHomeShell api={api} onOpenParentWorkspace={() => undefined} />);
    const grid = screen.getByTestId('kidos-home-grid');
    expect(grid.querySelectorAll('button')).toHaveLength(8);
  });

  it.each([
    ['Learn', 'kidos-learn-screen'],
    ['Play', 'kidos-play-screen'],
    ['Create', 'kidos-create-screen'],
    ['Watch', 'kidos-watch-screen'],
    ['Safe Browser', 'kidos-browser-screen'],
    ['KidOS AI', 'kidos-ai-screen'],
    ['Wellbeing', 'kidos-wellbeing-screen'],
    ['My Apps', 'kidos-apps-screen'],
  ])('opens the dedicated %s production screen', (buttonName, testId) => {
    render(<KidOSHomeShell api={api} onOpenParentWorkspace={() => undefined} />);
    const grid = screen.getByTestId('kidos-home-grid');
    fireEvent.click(within(grid).getByRole('button', { name: new RegExp(buttonName) }));
    expect(screen.getByTestId(testId)).toBeTruthy();
  });

  it('routes safe search through policy evaluation and keeps require-parent closed', async () => {
    render(<KidOSHomeShell api={api} onOpenParentWorkspace={() => undefined} />);
    const input = screen.getByLabelText('Search KidOS safely');
    fireEvent.change(input, { target: { value: 'planets' } });
    fireEvent.submit(input.closest('form')!);
    expect(await screen.findByText('Parent approval required.')).toBeTruthy();
  });

  it('blocks a destination when KidOS policy returns block', async () => {
    const blockedApi: KidOSApi = { ...api, async evaluateNavigation() { return 'block'; } };
    render(<KidOSHomeShell api={blockedApi} onOpenParentWorkspace={() => undefined} />);
    const input = screen.getByLabelText('Search KidOS safely');
    fireEvent.change(input, { target: { value: 'unsafe.example' } });
    fireEvent.submit(input.closest('form')!);
    expect(await screen.findByText('Blocked by KidOS safety policy.')).toBeTruthy();
  });

  it('does not open anything when the protected browser bridge is unavailable', async () => {
    const allowWithoutBrowser: KidOSApi = { ...api, async evaluateNavigation() { return 'allow'; } };
    render(<KidOSHomeShell api={allowWithoutBrowser} onOpenParentWorkspace={() => undefined} />);
    const input = screen.getByLabelText('Search KidOS safely');
    fireEvent.change(input, { target: { value: 'planets' } });
    fireEvent.submit(input.closest('form')!);
    expect(await screen.findByText(/Safe browser is unavailable/)).toBeTruthy();
  });

  it('enforces provider safe-search settings before opening an allowed search', async () => {
    let opened = '';
    const allowApi: KidOSApi = {
      ...api,
      async evaluateNavigation() { return 'allow'; },
      async openProtectedBrowser(url) { opened = url; },
    };
    render(<KidOSHomeShell api={allowApi} onOpenParentWorkspace={() => undefined} />);
    const input = screen.getByLabelText('Search KidOS safely');
    fireEvent.change(input, { target: { value: 'planets' } });
    fireEvent.submit(input.closest('form')!);
    expect(await screen.findByText('Opened through KidOS Safe Browser.')).toBeTruthy();
    expect(new URL(opened).searchParams.get('safe')).toBe('active');
  });

  it('moves keyboard focus to parent verification when Parent is selected', async () => {
    render(<KidOSHomeShell api={api} onOpenParentWorkspace={() => undefined} />);
    const parentButton = screen.getByTestId('kidos-sidebar').querySelector<HTMLButtonElement>('.kidos-parent-entry');
    expect(parentButton).toBeTruthy();
    fireEvent.click(parentButton!);
    await waitFor(() => expect(screen.getByLabelText('Parent PIN')).toBe(document.activeElement));
  });

  it('does not claim classifier readiness without classifier telemetry', async () => {
    render(<KidOSHomeShell api={api} onOpenParentWorkspace={() => undefined} />);
    expect(await screen.findByText(/Media Safety: Offline/)).toBeTruthy();
  });

  it('shows restricted safe mode as degraded rather than unreachable', async () => {
    const restrictedApi = { ...api, guardianStatus: async () => 'restricted_safe_mode' as const } as KidOSApi;
    render(<KidOSHomeShell api={restrictedApi} onOpenParentWorkspace={() => undefined} />);
    expect(await screen.findByText(/Safe Mode: Degraded/)).toBeTruthy();
  });

  it('does not claim active protection when live status cannot be read', async () => {
    const offlineApi = { ...api, guardianStatus: async () => { throw new Error('offline'); } } as KidOSApi;
    render(<KidOSHomeShell api={offlineApi} onOpenParentWorkspace={() => undefined} />);
    expect(await screen.findByText(/Safe Mode: Offline/)).toBeTruthy();
    expect(screen.queryByText('Safe Mode: ON')).toBeNull();
  });
});
