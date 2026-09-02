import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import SafetySummary from './SafetySummary';

afterEach(cleanup);

describe('KidOS safety summary', () => {
  it('shows aggregate decisions without a browsing timeline', () => {
    render(
      <SafetySummary
        authorized
        summary={{ total: 12, allowed: 7, blocked: 3, parentGated: 2 }}
        clearEvents={vi.fn(async () => undefined)}
      />,
    );

    expect(screen.getByText('12 safety decisions')).toBeTruthy();
    expect(screen.getByText('3 blocked')).toBeTruthy();
    expect(screen.queryByText(/history/i)).toBeNull();
  });

  it('does not expose summary or clearing in child mode', () => {
    const clearEvents = vi.fn(async () => undefined);
    render(
      <SafetySummary
        authorized={false}
        summary={{ total: 12, allowed: 7, blocked: 3, parentGated: 2 }}
        clearEvents={clearEvents}
      />,
    );

    expect(screen.queryByRole('button', { name: 'Clear safety events' })).toBeNull();
    expect(screen.queryByText('12 safety decisions')).toBeNull();
  });

  it('lets an authorized parent clear the minimal safety records', async () => {
    const clearEvents = vi.fn(async () => undefined);
    render(
      <SafetySummary
        authorized
        summary={{ total: 12, allowed: 7, blocked: 3, parentGated: 2 }}
        clearEvents={clearEvents}
      />,
    );

    fireEvent.click(screen.getByRole('button', { name: 'Clear safety events' }));
    expect(clearEvents).toHaveBeenCalledTimes(1);
  });
});
