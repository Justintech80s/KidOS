import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import KidOSHomeShell from '../../apps/shell/src/features/home/KidOSHomeShell';
import type { KidOSApi } from '../../apps/shell/src/lib/kidos-api';

const capability = { platform: 'windows', supported: true, mechanism: 'assigned_access' } as const;

function makeApi(opened: string[]) : KidOSApi {
  return {
    async planWorkspace(prompt) { return { kind: 'story', title: prompt, capabilities: ['story'] }; },
    async evaluateNavigation() { return 'allow'; },
    async evaluateDownload() { return 'block'; },
    async openProtectedBrowser(url) { opened.push(url); },
    async guardianStatus() { return 'healthy'; },
    async getRecoveryStatus() {
      return {
        guardianHealthy: true,
        classifierHealthy: true,
        recoveryRequired: false,
        policyValid: true,
        lockdownState: 'locked',
      };
    },
    async lockdownStatus() { return { state: 'locked', capability }; },
    async configureWindowsLockdown(request) { return { state: 'preparing', capability, managedAccount: request.account }; },
    async requestParentMaintenanceUnlock() { return { grantedAt: '2026-09-10T20:00:00Z', expiresAt: '2026-09-10T20:15:00Z' }; },
    async removeWindowsLockdown() { return { state: 'unmanaged', capability }; },
  };
}

afterEach(cleanup);

describe('KidOS 2026 protected child shell', () => {
  it('shows the approved eight-destination home and truthful protection state', async () => {
    render(<KidOSHomeShell api={makeApi([])} onOpenParentWorkspace={() => undefined} />);
    const grid = screen.getByTestId('kidos-home-grid');
    expect(within(grid).getAllByRole('button')).toHaveLength(8);
    expect(await screen.findByText(/Safe Mode: Active/)).toBeTruthy();
    expect(screen.getByText(/Internet Filter: Active/)).toBeTruthy();
    expect(screen.getByText(/Media Safety: Ready/)).toBeTruthy();
  });

  it('opens the dedicated Safe Browser and keeps navigation protected', async () => {
    const opened: string[] = [];
    render(<KidOSHomeShell api={makeApi(opened)} onOpenParentWorkspace={() => undefined} />);
    const grid = screen.getByTestId('kidos-home-grid');
    fireEvent.click(within(grid).getByRole('button', { name: /Safe Browser/ }));
    expect(screen.getByTestId('kidos-browser-screen')).toBeTruthy();
    const input = screen.getByLabelText('Protected web address');
    fireEvent.change(input, { target: { value: 'solar system' } });
    fireEvent.submit(input.closest('form')!);
    expect(await screen.findByText('Opened through KidOS Safe Browser.')).toBeTruthy();
    expect(opened).toHaveLength(1);
    expect(new URL(opened[0]).searchParams.get('safe')).toBe('active');
  });

  it('enforces SafeSearch before an allowed Google search opens', async () => {
    const opened: string[] = [];
    render(<KidOSHomeShell api={makeApi(opened)} onOpenParentWorkspace={() => undefined} />);
    const input = screen.getByLabelText('Search KidOS safely');
    fireEvent.change(input, { target: { value: 'solar system' } });
    fireEvent.submit(input.closest('form')!);
    expect(await screen.findByText('Opened through KidOS Safe Browser.')).toBeTruthy();
    expect(opened).toHaveLength(1);
    expect(new URL(opened[0]).searchParams.get('safe')).toBe('active');
  });

  it('routes Parent to the protected PIN entry point', async () => {
    render(<KidOSHomeShell api={makeApi([])} onOpenParentWorkspace={() => undefined} />);
    const parentButton = screen.getByTestId('kidos-sidebar').querySelector<HTMLButtonElement>('.kidos-parent-entry');
    expect(parentButton).toBeTruthy();
    fireEvent.click(parentButton!);
    await waitFor(() => {
      expect(document.activeElement).toBe(screen.getByLabelText('Parent PIN'));
    });
  });
});
