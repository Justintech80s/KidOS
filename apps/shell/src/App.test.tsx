import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import App from './App';
import type { KidOSApi } from './lib/kidos-api';

const healthyApi: KidOSApi = {
  async planWorkspace() {
    return {
      kind: 'story',
      title: 'Story',
      capabilities: ['story'],
    };
  },
  async evaluateNavigation() {
    return 'allow';
  },
  async evaluateDownload() {
    return 'allow';
  },
  async guardianStatus() {
    return 'healthy';
  },
};

describe('KidOS shell', () => {
  it('shows the creation-first prompt only after Guardian is healthy', async () => {
    render(<App api={healthyApi} />);
    expect(screen.getByText('Checking protection...')).toBeTruthy();
    expect(await screen.findByText('What do you want to create?')).toBeTruthy();
  });
});
