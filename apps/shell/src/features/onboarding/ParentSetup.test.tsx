import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import ParentSetup from './ParentSetup';

afterEach(cleanup);

describe('ParentSetup', () => {
  it('requires a matching 4-8 digit PIN before configuration', async () => {
    const configure = vi.fn().mockResolvedValue(undefined);
    render(<ParentSetup configureParentPin={configure} />);

    fireEvent.change(screen.getByLabelText('Parent PIN'), { target: { value: '12ab' } });
    fireEvent.change(screen.getByLabelText('Confirm parent PIN'), { target: { value: '12ab' } });
    fireEvent.click(screen.getByRole('button', { name: 'Protect KidOS' }));

    expect(await screen.findByText('Use 4 to 8 digits for the parent PIN.')).toBeTruthy();
    expect(configure).not.toHaveBeenCalled();

    fireEvent.change(screen.getByLabelText('Parent PIN'), { target: { value: '2468' } });
    fireEvent.change(screen.getByLabelText('Confirm parent PIN'), { target: { value: '2469' } });
    fireEvent.click(screen.getByRole('button', { name: 'Protect KidOS' }));

    expect(await screen.findByText('The PIN entries must match.')).toBeTruthy();
    expect(configure).not.toHaveBeenCalled();
  });

  it('hands the PIN directly to secure configuration without browser persistence', async () => {
    const configure = vi.fn().mockResolvedValue(undefined);
    const storageWrite = vi.spyOn(Storage.prototype, 'setItem');
    render(<ParentSetup configureParentPin={configure} />);

    fireEvent.change(screen.getByLabelText('Parent PIN'), { target: { value: '2468' } });
    fireEvent.change(screen.getByLabelText('Confirm parent PIN'), { target: { value: '2468' } });
    fireEvent.click(screen.getByRole('button', { name: 'Protect KidOS' }));

    await waitFor(() => expect(configure).toHaveBeenCalledWith('2468'));
    expect(storageWrite).not.toHaveBeenCalled();
    storageWrite.mockRestore();
  });
});
