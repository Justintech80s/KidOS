import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import type { KidOSApi } from '../../lib/kidos-api';
import KidOSHomeShell from './KidOSHomeShell';

const capability = { platform: 'windows', supported: true, mechanism: 'assigned_access' } as const;
const api: KidOSApi = {
  async planWorkspace(prompt) { return { kind: 'story', title: prompt, capabilities: ['story'] }; },
  async evaluateNavigation() { return 'require_parent'; },
  async evaluateDownload() { return 'require_parent'; },
  async guardianStatus() { return 'healthy'; },
  async askAi(query) {
    if (/planet|space/i.test(query)) return { available: true, answer: 'Earth is one of eight planets orbiting our Sun.' };
    if (/math/i.test(query)) return { available: true, answer: 'Break the problem into small steps, solve one step at a time, then check your answer.' };
    return { available: true, answer: 'KidOS AI safe test answer.' };
  },
  async lockdownStatus() { return { state: 'unmanaged', capability }; },
  async configureWindowsLockdown(request) { return { state: 'preparing', capability, managedAccount: request.account }; },
  async requestParentMaintenanceUnlock() { return { grantedAt: '2026-09-03T00:45:00Z', expiresAt: '2026-09-03T01:00:00Z' }; },
  async removeWindowsLockdown() { return { state: 'unmanaged', capability }; },
};

function openParent() {
  const parentButton = screen.getByTestId('kidos-sidebar').querySelector<HTMLButtonElement>('.kidos-parent-entry');
  expect(parentButton).toBeTruthy();
  fireEvent.click(parentButton!);
}

function openHomeDestination(name: string) {
  const grid = screen.getByTestId('kidos-home-grid');
  fireEvent.click(within(grid).getByRole('button', { name: new RegExp(name) }));
}

afterEach(cleanup);

describe('KidOSHomeShell', () => {
  it('renders the eight protected home destinations', () => {
    render(<KidOSHomeShell api={api} onOpenParentWorkspace={() => undefined} />);
    const grid = screen.getByTestId('kidos-home-grid');
    expect(grid.querySelectorAll('button')).toHaveLength(8);
  });

  it.each([
    ['Learn', 'kidos-learn-screen'],
    ['Play', 'kidos-play-screen'],
    ['Create', 'kidos-create-screen'],
    ['Watch', 'kidos-watch-screen'],
    ['Safe Browser', 'kidos-browser-screen'],
    ['KidOS AI', 'kidos-ai-screen'],
    ['Wellbeing', 'kidos-wellbeing-screen'],
    ['My Apps', 'kidos-apps-screen'],
  ])('opens the dedicated %s production screen', (buttonName, testId) => {
    render(<KidOSHomeShell api={api} onOpenParentWorkspace={() => undefined} />);
    openHomeDestination(buttonName);
    expect(screen.getByTestId(testId)).toBeTruthy();
  });

  it.each([
    ['Learn', 'Math', 'khanacademy.org/math'],
    ['Play', 'Puzzles', 'pbskids.org/games'],
    ['Watch', 'Science', 'kids.youtube.com/search'],
  ])('routes %s category actions through the protected browser', async (destination, action, expectedUrl) => {
    const opened: string[] = [];
    const allowApi: KidOSApi = {
      ...api,
      async evaluateNavigation() { return 'allow'; },
      async openProtectedBrowser(url) { opened.push(url); },
    };
    render(<KidOSHomeShell api={allowApi} onOpenParentWorkspace={() => undefined} />);
    openHomeDestination(destination);
    fireEvent.click(screen.getByRole('button', { name: new RegExp(action) }));
    expect(await screen.findByText(new RegExp(`${action} opened through KidOS Safe Browser`))).toBeTruthy();
    expect(opened.some((url) => url.includes(expectedUrl))).toBe(true);
  });

  it('saves an editable Create project through the Electron project API contract', async () => {
    let savedContent = '';
    const createApi: KidOSApi = {
      ...api,
      async listWorkspaceDocuments() { return []; },
      async saveWorkspaceDocument(document) {
        savedContent = document.content;
        return {
          ...document,
          id: 'project-1',
          createdAt: '2026-10-04T12:00:00Z',
          updatedAt: '2026-10-04T12:00:00Z',
        };
      },
    };

    render(<KidOSHomeShell api={createApi} onOpenParentWorkspace={() => undefined} />);
    openHomeDestination('Create');
    const prompt = screen.getByLabelText('Ask KidOS');
    fireEvent.change(prompt, { target: { value: 'Write a moon story' } });
    fireEvent.submit(prompt.closest('form')!);

    expect(await screen.findByTestId('kidos-workspace-editor')).toBeTruthy();
    const content = screen.getByLabelText('Project content');
    fireEvent.change(content, { target: { value: 'Once upon a time on the Moon.' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save project' }));

    expect(await screen.findByText('Project saved safely on this computer.')).toBeTruthy();
    expect(savedContent).toBe('Once upon a time on the Moon.');
    expect(screen.getByText('Saved project')).toBeTruthy();
  });

  it('creates a protected workspace through the existing planner API', async () => {
    render(<KidOSHomeShell api={api} onOpenParentWorkspace={() => undefined} />);
    openHomeDestination('Create');
    const input = screen.getByLabelText('Ask KidOS');
    fireEvent.change(input, { target: { value: 'Make a moon story' } });
    fireEvent.submit(input.closest('form')!);
    expect(await screen.findByText('Safe workspace ready: Make a moon story')).toBeTruthy();
  });

  it('fails closed when the protected workspace planner fails', async () => {
    const failingApi: KidOSApi = { ...api, async planWorkspace() { throw new Error('planner offline'); } };
    render(<KidOSHomeShell api={failingApi} onOpenParentWorkspace={() => undefined} />);
    openHomeDestination('Create');
    const input = screen.getByLabelText('Ask KidOS');
    fireEvent.change(input, { target: { value: 'Make a moon story' } });
    fireEvent.submit(input.closest('form')!);
    expect(await screen.findByText('KidOS could not prepare the workspace safely.')).toBeTruthy();
  });

  it('renders KidOS AI answers from the protected backend inside the dedicated safe module', async () => {
    render(<KidOSHomeShell api={api} onOpenParentWorkspace={() => undefined} />);
    openHomeDestination('KidOS AI');
    const input = screen.getByLabelText('Ask KidOS AI');
    fireEvent.change(input, { target: { value: 'Tell me about planets' } });
    fireEvent.submit(input.closest('form')!);
    expect(await screen.findByText(/Earth is one of eight planets/)).toBeTruthy();
  });

  it('supports child-friendly KidOS AI suggestion prompts through the backend', async () => {
    render(<KidOSHomeShell api={api} onOpenParentWorkspace={() => undefined} />);
    openHomeDestination('KidOS AI');
    fireEvent.click(screen.getByRole('button', { name: 'Help me with math' }));
    expect(await screen.findByText(/Break the problem into small steps/)).toBeTruthy();
  });

  it('routes safe search through policy evaluation and keeps require-parent closed', async () => {
    render(<KidOSHomeShell api={api} onOpenParentWorkspace={() => undefined} />);
    const input = screen.getByLabelText('Search KidOS safely');
    fireEvent.change(input, { target: { value: 'planets' } });
    fireEvent.submit(input.closest('form')!);
    expect(await screen.findByText('Parent approval required.')).toBeTruthy();
  });

  it('blocks a destination when KidOS policy returns block', async () => {
    const blockedApi: KidOSApi = { ...api, async evaluateNavigation() { return 'block'; } };
    render(<KidOSHomeShell api={blockedApi} onOpenParentWorkspace={() => undefined} />);
    const input = screen.getByLabelText('Search KidOS safely');
    fireEvent.change(input, { target: { value: 'unsafe.example' } });
    fireEvent.submit(input.closest('form')!);
    expect(await screen.findByText('Blocked by KidOS safety policy.')).toBeTruthy();
  });

  it('does not open anything when the protected browser bridge is unavailable', async () => {
    const allowWithoutBrowser: KidOSApi = { ...api, async evaluateNavigation() { return 'allow'; } };
    render(<KidOSHomeShell api={allowWithoutBrowser} onOpenParentWorkspace={() => undefined} />);
    const input = screen.getByLabelText('Search KidOS safely');
    fireEvent.change(input, { target: { value: 'planets' } });
    fireEvent.submit(input.closest('form')!);
    expect(await screen.findByText(/Safe browser is unavailable/)).toBeTruthy();
  });

  it('enforces provider safe-search settings before opening an allowed search', async () => {
    let opened = '';
    const allowApi: KidOSApi = {
      ...api,
      async evaluateNavigation() { return 'allow'; },
      async openProtectedBrowser(url) { opened = url; },
    };
    render(<KidOSHomeShell api={allowApi} onOpenParentWorkspace={() => undefined} />);
    const input = screen.getByLabelText('Search KidOS safely');
    fireEvent.change(input, { target: { value: 'planets' } });
    fireEvent.submit(input.closest('form')!);
    expect(await screen.findByText('Opened through KidOS Safe Browser.')).toBeTruthy();
    expect(new URL(opened).searchParams.get('safe')).toBe('active');
  });

  it('opens the dedicated Parent Access screen and moves keyboard focus to the PIN', async () => {
    render(<KidOSHomeShell api={api} onOpenParentWorkspace={() => undefined} />);
    openParent();
    expect(screen.getByRole('heading', { name: 'Parent Access' })).toBeTruthy();
    expect(screen.getByTestId('kidos-parent-screen')).toBeTruthy();
    await waitFor(() => expect(screen.getByLabelText('Parent PIN')).toBe(document.activeElement));
  });

  it('does not open the parent workspace when verification is unavailable', async () => {
    let opened = false;
    render(<KidOSHomeShell api={api} onOpenParentWorkspace={() => { opened = true; }} />);
    openParent();
    fireEvent.change(screen.getByLabelText('Parent PIN'), { target: { value: '1234' } });
    fireEvent.click(screen.getByRole('button', { name: 'Unlock Parent Workspace' }));
    expect(await screen.findByText('Parent verification is available in the installed Windows build.')).toBeTruthy();
    expect(opened).toBe(false);
  });

  it('opens the parent workspace only after successful PIN verification', async () => {
    let opened = false;
    const parentApi: KidOSApi = { ...api, async verifyParentPin() { return { authorized: true, locked: false }; } };
    render(<KidOSHomeShell api={parentApi} onOpenParentWorkspace={() => { opened = true; }} />);
    openParent();
    fireEvent.change(screen.getByLabelText('Parent PIN'), { target: { value: '2468' } });
    fireEvent.click(screen.getByRole('button', { name: 'Unlock Parent Workspace' }));
    await waitFor(() => expect(opened).toBe(true));
  });

  it.each([
    [{ authorized: false, locked: true }, 'Parent PIN is temporarily locked.'],
    [{ authorized: false, locked: false }, 'Parent PIN was not accepted.'],
  ])('keeps Parent Access locked for rejected verification %#', async (verification, message) => {
    const parentApi: KidOSApi = { ...api, async verifyParentPin() { return verification; } };
    render(<KidOSHomeShell api={parentApi} onOpenParentWorkspace={() => undefined} />);
    openParent();
    fireEvent.change(screen.getByLabelText('Parent PIN'), { target: { value: '2468' } });
    fireEvent.click(screen.getByRole('button', { name: 'Unlock Parent Workspace' }));
    expect(await screen.findByText(message)).toBeTruthy();
  });

  it('does not claim classifier readiness without classifier telemetry', async () => {
    render(<KidOSHomeShell api={api} onOpenParentWorkspace={() => undefined} />);
    expect(await screen.findByText(/Media Safety: Offline/)).toBeTruthy();
  });

  it('shows restricted safe mode as degraded rather than unreachable', async () => {
    const restrictedApi = { ...api, guardianStatus: async () => 'restricted_safe_mode' as const } as KidOSApi;
    render(<KidOSHomeShell api={restrictedApi} onOpenParentWorkspace={() => undefined} />);
    expect(await screen.findByText(/Safe Mode: Degraded/)).toBeTruthy();
  });

  it('does not claim active protection when live status cannot be read', async () => {
    const offlineApi = { ...api, guardianStatus: async () => { throw new Error('offline'); } } as KidOSApi;
    render(<KidOSHomeShell api={offlineApi} onOpenParentWorkspace={() => undefined} />);
    expect(await screen.findByText(/Safe Mode: Offline/)).toBeTruthy();
    expect(screen.queryByText('Safe Mode: ON')).toBeNull();
  });
});
