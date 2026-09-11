import { useEffect, useRef, useState } from 'react';
import type { KidOSApi } from '../../lib/kidos-api';
import { prepareProtectedNavigation } from '../browser/protected-navigation';
import KidOSAiScreen from './KidOSAiScreen';
import KidOSCreateScreen from './KidOSCreateScreen';
import KidOSDock from './KidOSDock';
import KidOSHomeScreen from './KidOSHomeScreen';
import KidOSLearnScreen from './KidOSLearnScreen';
import KidOSMyAppsScreen from './KidOSMyAppsScreen';
import KidOSParentAccess from './KidOSParentAccess';
import KidOSPlayScreen from './KidOSPlayScreen';
import KidOSSafeBrowserScreen from './KidOSSafeBrowserScreen';
import KidOSSidebar, { type KidOSDestination } from './KidOSSidebar';
import KidOSTopBar from './KidOSTopBar';
import KidOSWatchScreen from './KidOSWatchScreen';
import KidOSWellbeingScreen from './KidOSWellbeingScreen';
import { OFFLINE_KIDOS_STATUS, normalizeKidOSSystemStatus, type KidOSSystemStatus } from './system-status';
import './kidos-shell-2026.css';

export default function KidOSHomeShell({ api, onOpenParentWorkspace }: { api: KidOSApi; onOpenParentWorkspace(): void }) {
  const [active, setActive] = useState<KidOSDestination>('home');
  const [status, setStatus] = useState<KidOSSystemStatus>(OFFLINE_KIDOS_STATUS);
  const [searchValue, setSearchValue] = useState('');
  const [searchStatus, setSearchStatus] = useState('');
  const [createValue, setCreateValue] = useState('');
  const [workspaceStatus, setWorkspaceStatus] = useState('');
  const [aiValue, setAiValue] = useState('');
  const [aiAnswer, setAiAnswer] = useState('Ask a school-safe question and KidOS AI will help you think it through.');
  const [parentPin, setParentPin] = useState('');
  const [parentStatus, setParentStatus] = useState('');
  const parentPinRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    let mounted = true;
    async function refresh() {
      try {
        const guardian = await api.guardianStatus();
        let recovery;
        let recoveryAvailable = false;
        if (api.getRecoveryStatus) {
          recovery = await api.getRecoveryStatus();
          recoveryAvailable = true;
        }
        if (!mounted) return;
        const guardianHealthy = guardian === 'healthy';
        setStatus(normalizeKidOSSystemStatus({
          guardianReachable: true,
          guardianEnforcing: guardianHealthy && (recovery?.policyValid ?? true),
          classifierReachable: recovery?.classifierHealthy ?? false,
          classifierReady: recovery?.classifierHealthy ?? false,
          filterEnforcing: guardianHealthy && (recovery?.policyValid ?? true),
          recoveryAvailable,
          observedAt: Date.now(),
        }));
      } catch {
        if (mounted) setStatus(OFFLINE_KIDOS_STATUS);
      }
    }
    void refresh();
    const id = window.setInterval(refresh, 10_000);
    return () => { mounted = false; window.clearInterval(id); };
  }, [api]);

  async function runSafeSearch(query: string) {
    setActive('browser');
    setSearchValue(query);
    setSearchStatus('Checking with KidOS...');
    const candidate = /^https?:\/\//i.test(query) ? query : `https://www.google.com/search?q=${encodeURIComponent(query)}`;
    try {
      const result = await prepareProtectedNavigation(candidate, api.evaluateNavigation);
      if (result.state === 'load') {
        if (!api.openProtectedBrowser) {
          setSearchStatus('Safe browser is unavailable. KidOS did not open the destination.');
          return;
        }
        await api.openProtectedBrowser(result.url);
        setSearchStatus('Opened through KidOS Safe Browser.');
      } else if (result.state === 'parent_gate') {
        setSearchStatus('Parent approval required.');
      } else {
        setSearchStatus('Blocked by KidOS safety policy.');
      }
    } catch {
      setSearchStatus('Safe browsing is unavailable. KidOS did not open the destination.');
    }
  }

  function openParentAccess() {
    setActive('parent');
    setParentStatus('');
    window.requestAnimationFrame(() => parentPinRef.current?.focus());
  }

  async function submitSearch() {
    const value = searchValue.trim();
    if (value) await runSafeSearch(value);
  }

  async function createWorkspace(prompt = createValue) {
    const value = prompt.trim();
    if (!value) return;
    setCreateValue(value);
    setWorkspaceStatus('Preparing a safe workspace...');
    try {
      const plan = await api.planWorkspace(value);
      setWorkspaceStatus(`Safe workspace ready: ${plan.title}`);
    } catch {
      setWorkspaceStatus('KidOS could not prepare the workspace safely.');
    }
  }

  function answerAi(query: string) {
    const q = query.trim();
    if (!q) return;
    setAiValue(q);
    setAiAnswer(/space|planet/i.test(q)
      ? 'Earth is one of eight planets orbiting our Sun. I can explain each planet in simple steps.'
      : /math|\d/.test(q)
        ? 'Break the problem into small steps, solve one step at a time, then check your answer.'
        : /sky/i.test(q)
          ? 'The sky looks blue because sunlight is scattered by gases in Earth’s atmosphere, and blue light scatters strongly.'
          : 'KidOS AI keeps answers age-appropriate and inside the active safety rules.');
  }

  function askAi() {
    answerAi(aiValue);
  }

  async function requestParent() {
    const pin = parentPin.trim();
    if (!api.verifyParentPin) {
      setParentStatus('Parent verification is available in the installed Windows build.');
      return;
    }
    if (!pin) return;
    try {
      const result = await api.verifyParentPin(pin);
      if (result.authorized) onOpenParentWorkspace();
      else setParentStatus(result.locked ? 'Parent PIN is temporarily locked.' : 'Parent PIN was not accepted.');
    } catch {
      setParentStatus('Parent verification is unavailable.');
    }
  }

  function renderActiveScreen() {
    switch (active) {
      case 'home':
        return <KidOSHomeScreen status={status} onNavigate={setActive} />;
      case 'learn':
        return <KidOSLearnScreen />;
      case 'play':
        return <KidOSPlayScreen />;
      case 'watch':
        return <KidOSWatchScreen />;
      case 'wellbeing':
        return <KidOSWellbeingScreen />;
      case 'apps':
        return <KidOSMyAppsScreen />;
      case 'browser':
        return (
          <KidOSSafeBrowserScreen
            value={searchValue}
            statusMessage={searchStatus}
            protectionStatus={status}
            onValueChange={setSearchValue}
            onSubmit={() => { void submitSearch(); }}
            onShortcut={(query) => { void runSafeSearch(query); }}
          />
        );
      case 'create':
        return (
          <KidOSCreateScreen
            value={createValue}
            statusMessage={workspaceStatus}
            onValueChange={setCreateValue}
            onSubmit={() => { void createWorkspace(); }}
            onPrompt={(prompt) => { void createWorkspace(prompt); }}
          />
        );
      case 'ai':
        return (
          <KidOSAiScreen
            value={aiValue}
            answer={aiAnswer}
            onValueChange={setAiValue}
            onSubmit={askAi}
            onSuggestion={answerAi}
          />
        );
      case 'parent':
        return (
          <KidOSParentAccess
            pin={parentPin}
            statusMessage={parentStatus}
            inputRef={parentPinRef}
            onPinChange={setParentPin}
            onUnlock={() => { void requestParent(); }}
          />
        );
      case 'music':
        return <section className="kidos-module"><h1>Music</h1><p>Parent-approved music and creative audio tools.</p></section>;
      default:
        return null;
    }
  }

  return (
    <main className="kidos-shell-2026" data-testid="kidos-shell">
      <KidOSSidebar active={active} onNavigate={setActive} onParentRequested={openParentAccess} />
      <section className="kidos-stage">
        <KidOSTopBar onSafeSearch={runSafeSearch} />
        <div className="kidos-main">{renderActiveScreen()}</div>
        <KidOSDock onNavigate={setActive} />
      </section>
    </main>
  );
}
