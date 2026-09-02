import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import { KidOSDesktopHarness } from './desktop-harness';

afterEach(cleanup);

describe('KidOS household creation flow', () => {
  it('lets a parent configure a profile and the child create a Story workspace', async () => {
    render(<KidOSDesktopHarness />);

    fireEvent.change(screen.getByLabelText('Child age'), { target: { value: '10' } });
    fireEvent.change(screen.getByLabelText('Parent PIN'), { target: { value: '2468' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save parent settings' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Enter child mode' }));

    fireEvent.change(screen.getByLabelText('Ask KidOS'), { target: { value: 'write a space story' } });
    fireEvent.click(screen.getByRole('button', { name: 'Create' }));

    expect(await screen.findByText('Story workspace')).toBeTruthy();
  });
});
