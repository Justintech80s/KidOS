import { describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import LockdownSettings from './LockdownSettings';

const capability = { platform: 'windows', supported: true, mechanism: 'assigned_access' } as const;
const api = {
  lockdownStatus: vi.fn(async () => ({ state: 'unmanaged' as const, capability })),
  configureWindowsLockdown: vi.fn(async () => ({ state: 'preparing' as const, capability })),
  requestParentMaintenanceUnlock: vi.fn(async () => ({ grantedAt: '2026-09-03T00:45:00Z', expiresAt: '2026-09-03T01:00:00Z' })),
  removeWindowsLockdown: vi.fn(async () => ({ state: 'unmanaged' as const, capability })),
};

describe('LockdownSettings', () => {
  it('hides lockdown controls from unauthorized users', () => {
    render(<LockdownSettings authorized={false} api={api} />);
    expect(screen.queryByRole('button', { name: /configure lockdown/i })).toBeNull();
  });

  it('rejects administrator accounts before configuration', async () => {
    api.configureWindowsLockdown.mockClear();
    render(<LockdownSettings authorized api={api} />);
    fireEvent.change(screen.getByLabelText(/account name/i), { target: { value: 'Admin' } });
    fireEvent.change(screen.getByLabelText(/account role/i), { target: { value: 'administrator' } });
    fireEvent.click(screen.getByRole('button', { name: /configure lockdown/i }));
    expect(await screen.findByRole('alert')).toHaveTextContent(/standard child account/i);
    expect(api.configureWindowsLockdown).not.toHaveBeenCalled();
  });

  it('explains that configuration takes effect at the next child sign-in', async () => {
    render(<LockdownSettings authorized api={api} />);
    fireEvent.change(screen.getByLabelText(/account name/i), { target: { value: 'Kid' } });
    fireEvent.click(screen.getByRole('button', { name: /configure lockdown/i }));
    expect(await screen.findByRole('status')).toHaveTextContent(/next sign-in/i);
  });

  it('shows restricted safe mode prominently', () => {
    render(<LockdownSettings authorized api={api} initialStatus={{ state: 'restricted_safe_mode', capability }} />);
    expect(screen.getByRole('alert')).toHaveTextContent(/restricted safe mode/i);
  });

  it('shows maintenance unlock expiry', async () => {
    render(<LockdownSettings authorized api={api} initialStatus={{ state: 'locked', capability }} />);
    fireEvent.click(screen.getByRole('button', { name: /maintenance unlock/i }));
    expect(await screen.findByRole('status')).toHaveTextContent(/expires/i);
  });
});
