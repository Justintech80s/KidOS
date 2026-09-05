import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import { KidOSDesktopHarness } from './desktop-harness';

afterEach(cleanup);

describe('KidOS Windows Lockdown vertical slice', () => {
  it('lets an authorized parent prepare a standard child account and explains next sign-in', async () => {
    render(<KidOSDesktopHarness />);
    fireEvent.change(screen.getByLabelText('Child account name'), { target: { value: 'Kid' } });
    fireEvent.click(screen.getByRole('button', { name: 'Configure lockdown' }));
    expect(await screen.findByRole('status')).toHaveTextContent(/next sign-in/i);
  });

  it('rejects administrator accounts', async () => {
    render(<KidOSDesktopHarness />);
    fireEvent.change(screen.getByLabelText('Child account name'), { target: { value: 'Admin' } });
    fireEvent.change(screen.getByLabelText('Account role'), { target: { value: 'administrator' } });
    fireEvent.click(screen.getByRole('button', { name: 'Configure lockdown' }));
    expect(await screen.findByRole('alert')).toHaveTextContent(/standard child account/i);
  });

  it('does not expose parent lockdown controls in child mode', () => {
    render(<KidOSDesktopHarness startInChildMode />);
    expect(screen.queryByRole('button', { name: 'Configure lockdown' })).toBeNull();
  });

  it('shows restricted safe mode when Guardian cannot confirm lockdown', () => {
    render(<KidOSDesktopHarness initialLockdownState="restricted_safe_mode" />);
    expect(screen.getByRole('alert')).toHaveTextContent(/restricted safe mode/i);
  });

  it('expires a temporary parent maintenance unlock', async () => {
    render(<KidOSDesktopHarness initialLockdownState="locked" />);
    fireEvent.change(screen.getByLabelText('Parent PIN for sensitive changes'), { target: { value: '2468' } });
    fireEvent.click(screen.getByRole('button', { name: /maintenance unlock/i }));
    expect(await screen.findByRole('status')).toHaveTextContent(/expires/i);
    fireEvent.click(screen.getByRole('button', { name: 'Expire maintenance unlock' }));
    expect(screen.getByText(/current state:/i).parentElement).toHaveTextContent(/locked/i);
  });
});
