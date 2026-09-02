import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { ParentPolicyConfig } from '@kidos/contracts';
import ParentDashboard from './ParentDashboard';

afterEach(cleanup);

describe('KidOS parent dashboard', () => {
  it('does not expose parent settings or saving in child mode', () => {
    const savePolicy = vi.fn(async () => undefined);

    render(<ParentDashboard authorized={false} savePolicy={savePolicy} />);

    expect(screen.getByText('Parent authorization required')).toBeTruthy();
    expect(screen.queryByLabelText('Child age')).toBeNull();
    expect(screen.queryByRole('button', { name: 'Save parent settings' })).toBeNull();
    expect(savePolicy).not.toHaveBeenCalled();
  });

  it('submits typed policy settings and the parent PIN only from authorized mode', async () => {
    const savePolicy = vi.fn(async (_pin: string, _policy: ParentPolicyConfig) => undefined);

    render(<ParentDashboard authorized savePolicy={savePolicy} />);

    fireEvent.change(screen.getByLabelText('Child age'), { target: { value: '15' } });
    fireEvent.change(screen.getByLabelText('Allowed domains'), {
      target: { value: 'khanacademy.org\nscience.org' },
    });
    fireEvent.change(screen.getByLabelText('Blocked domains'), {
      target: { value: 'unsafe.example' },
    });
    fireEvent.click(screen.getByLabelText('Allow unknown websites for teen profile'));
    fireEvent.change(screen.getByLabelText('Social service'), { target: { value: 'youtube' } });
    fireEvent.change(screen.getByLabelText('Social access'), { target: { value: 'time_limited' } });
    fireEvent.change(screen.getByLabelText('Social start'), { target: { value: '08:00' } });
    fireEvent.change(screen.getByLabelText('Social end'), { target: { value: '20:00' } });
    fireEvent.change(screen.getByLabelText('Download protection'), {
      target: { value: 'block_high_risk' },
    });
    fireEvent.change(screen.getByLabelText('Parent PIN'), { target: { value: '2468' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save parent settings' }));

    expect(savePolicy).toHaveBeenCalledTimes(1);
    const [pin, policy] = savePolicy.mock.calls[0];
    expect(pin).toBe('2468');
    expect(policy).toEqual({
      childAge: 15,
      allowDomains: ['khanacademy.org', 'science.org'],
      blockDomains: ['unsafe.example'],
      teenUnknownWebEnabled: true,
      socialAccess: [
        {
          service: 'youtube',
          mode: 'time_limited',
          startMinutes: 480,
          endMinutes: 1200,
        },
      ],
      downloadMode: 'block_high_risk',
    });
  });
});
