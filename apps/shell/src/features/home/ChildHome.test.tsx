import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import type { KidOSApi } from '../../lib/kidos-api';
import ChildHome from './ChildHome';

afterEach(cleanup);

const capability = { platform: 'windows', supported: true, mechanism: 'assigned_access' } as const;

const api: KidOSApi = {
  async planWorkspace(prompt) {
    return {
      kind: 'story',
      title: prompt.includes('space') ? 'Space Story' : 'Story',
      capabilities: ['story', 'export_project'],
    };
  },
  async evaluateNavigation() {
    return 'require_parent';
  },
  async evaluateDownload() {
    return 'require_parent';
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

const allowApi: KidOSApi = {
  ...api,
  async evaluateNavigation() {
    return 'allow';
  },
};

describe('KidOS protected child flows', () => {
  it('turns a creation request into the returned safe workspace', async () => {
    render(<ChildHome api={api} />);

    fireEvent.change(screen.getByLabelText('Ask KidOS'), {
      target: { value: 'make a story about space' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Create' }));

    expect(await screen.findByRole('heading', { name: 'Space Story' })).toBeTruthy();
    expect(screen.getByText('Story workspace')).toBeTruthy();
  });

  it('shows a parent approval gate instead of opening require-parent navigation', async () => {
    render(<ChildHome api={api} />);
    fireEvent.click(screen.getByRole('button', { name: /^Search$/ }));

    fireEvent.change(screen.getByLabelText('Protected web address'), {
      target: { value: 'https://unknown.example' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Check site' }));

    expect(await screen.findByText('Parent approval required')).toBeTruthy();
    expect(screen.queryByText('Opening https://unknown.example')).toBeNull();
  });

  it('shows the enforced Google SafeSearch URL before an allowed load', async () => {
    render(<ChildHome api={allowApi} />);
    fireEvent.click(screen.getByRole('button', { name: /^Search$/ }));

    fireEvent.change(screen.getByLabelText('Protected web address'), {
      target: { value: 'https://www.google.com/search?q=planets' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Check site' }));

    const status = await screen.findByRole('status');
    expect(status.textContent).toContain('Opening https://www.google.com/search?');
    expect(status.textContent).toContain('q=planets');
    expect(status.textContent).toContain('safe=active');
  });
});
