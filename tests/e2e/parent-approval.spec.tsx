import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import { KidOSDesktopHarness } from './desktop-harness';

afterEach(cleanup);

describe('KidOS one-time parent approval', () => {
  it('allows one parent-gated navigation only after parent approval', async () => {
    render(<KidOSDesktopHarness startInChildMode />);

    fireEvent.change(screen.getByLabelText('Protected web address'), {
      target: { value: 'https://unknown.example/learn' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Check site' }));

    expect(await screen.findByText('Parent approval required')).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Approve once' }));
    fireEvent.click(screen.getByRole('button', { name: 'Check site' }));

    expect(await screen.findByText(/Opening https:\/\/unknown\.example\/learn/)).toBeTruthy();

    fireEvent.click(screen.getByRole('button', { name: 'Check site' }));
    expect(await screen.findByText('Parent approval required')).toBeTruthy();
  });
});
