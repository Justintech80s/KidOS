import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import { KidOSDesktopHarness } from './desktop-harness';

afterEach(cleanup);

describe('KidOS protected actions', () => {
  it('keeps blocked domains and high-risk downloads blocked', async () => {
    render(<KidOSDesktopHarness startInChildMode />);

    fireEvent.change(screen.getByLabelText('Protected web address'), {
      target: { value: 'https://blocked.example' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Check site' }));
    expect(await screen.findByText('Site blocked by KidOS')).toBeTruthy();

    fireEvent.change(screen.getByLabelText('Test download file'), {
      target: { value: 'photo.jpg.exe' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Test download' }));
    expect(await screen.findByText('Download blocked by KidOS')).toBeTruthy();
  });
});
