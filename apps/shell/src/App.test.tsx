import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import App from './App';

describe('KidOS shell', () => {
  it('shows the creation-first prompt', () => {
    render(<App />);
    expect(screen.getByText('What do you want to create?')).toBeTruthy();
  });
});
