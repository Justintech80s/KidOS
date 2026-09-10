import { type FormEvent, useEffect, useRef, useState } from 'react';
import type { KidOSApi } from '../../lib/kidos-api';
import { prepareProtectedNavigation } from '../browser/protected-navigation';
import KidOSDock from './KidOSDock';
import KidOSGreeting from './KidOSGreeting';
import KidOSHomeGrid from './KidOSHomeGrid';
import KidOSProfileCard from './KidOSProfileCard';
import KidOSSafetyStatus from './KidOSSafetyStatus';
import KidOSSidebar, { type KidOSDestination } from './KidOSSidebar';
import KidOSTopBar from './KidOSTopBar';
import { OFFLINE_KIDOS_STATUS, normalizeKidOSSystemStatus, type KidOSSystemStatus } from './system-status';
import './kidos-shell-2026.css';

export default function KidOSHomeShell({ api, onOpenParentWorkspace }: { api: KidOSApi; onOpenParentWorkspace(): void }) {
  const [active, setActive] = useState<KidOSDestination>('home');
  const [status, setStatus] = useState<KidOSSystemStatus>(OFFLINE_KIDOS_STATUS);
  const [searchValue, setSearchValue] = useState('');
  const [searchStatus, setSearchStatus] = useState('');
  const [createValue, setCreateValue] = useState('');
  const [workspaceTitle, setWorkspaceTitle] = useState('');
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
    setParentStatus('');
    window.requestAnimationFrame(() => parentPinRef.current?.focus());
  }

  async function submitSearch(event: FormEvent) {
    event.preventDefault();
    const value = searchValue.trim();
    if (value) await runSafeSearch(value);
  }

  async function createWorkspace(event: FormEvent) {
    event.preventDefault();
    const value = createValue.trim();
    if (!value) return;
    try {
      const plan = await api.planWorkspace(value);
      setWorkspaceTitle(plan.title);
    } catch {
      setWorkspaceTitle('KidOS could not prepare the workspace safely.');
    }
  }

  function askAi(event: FormEvent) {
    event.preventDefault();
    const q = aiValue.trim();
    if (!q) return;
    setAiAnswer(/space|planet/i.test(q)
      ? 'Earth is one of eight planets orbiting our Sun. I can explain each planet in simple steps.'
      : /math|\d/.test(q)
        ? 'Break the problem into small steps, solve one step at a time, then check your answer.'
        : 'KidOS AI keeps answers age-appropriate and inside the active safety rules.');
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

  return (
    <main className="kidos-shell-2026" data-testid="kidos-shell">
      <KidOSSidebar active={active} onNavigate={setActive} onParentRequested={openParentAccess} />
      <section className="kidos-stage">
        <KidOSTopBar onSafeSearch={runSafeSearch} />
        <div className="kidos-main">
          {active === 'home' ? <><KidOSGreeting /><KidOSHomeGrid onNavigate={setActive} /><KidOSSafetyStatus status={status} /></> :
           active === 'browser' ? <section className="kidos-module"><h1>Safe Browser</h1><p>Every destination is checked by KidOS before it can open.</p><form onSubmit={submitSearch}><input aria-label="Protected web address" value={searchValue} onChange={(e) => setSearchValue(e.target.value)} placeholder="Search or enter a website"/><button type="submit">Check site</button></form>{searchStatus && <div className="kidos-module-status" role="status">{searchStatus}</div>}</section> :
           active === 'create' ? <section className="kidos-module"><h1>Create</h1><p>Stories, drawings, presentations, and beginner coding begin in a protected workspace.</p><form onSubmit={createWorkspace}><input aria-label="Ask KidOS" value={createValue} onChange={(e) => setCreateValue(e.target.value)} placeholder="Make a space story..."/><button type="submit">Create</button></form>{workspaceTitle && <div className="kidos-module-status">Safe workspace ready: <strong>{workspaceTitle}</strong></div>}</section> :
           active === 'ai' ? <section className="kidos-module"><h1>KidOS AI</h1><p>{aiAnswer}</p><form onSubmit={askAi}><input aria-label="Ask KidOS AI" value={aiValue} onChange={(e) => setAiValue(e.target.value)} placeholder="Why is the sky blue?"/><button type="submit">Ask</button></form></section> :
           active === 'learn' ? <section className="kidos-module"><h1>Learn</h1><p>Math, science, reading, and approved learning tools live here.</p></section> :
           active === 'play' ? <section className="kidos-module"><h1>Play</h1><p>Only games approved for this KidOS profile are available.</p></section> :
           active === 'watch' ? <section className="kidos-module"><h1>Watch</h1><p>Videos appear only after KidOS media-safety checks.</p></section> :
           active === 'apps' ? <section className="kidos-module"><h1>My Apps</h1><p>KidOS launches only apps already approved by a parent. Arbitrary executable paths are never accepted here.</p></section> :
           active === 'wellbeing' ? <section className="kidos-module"><h1>Wellbeing</h1><p>Screen-time balance, accessibility, and healthy-break tools.</p></section> :
           active === 'music' ? <section className="kidos-module"><h1>Music</h1><p>Parent-approved music and creative audio tools.</p></section> : null}
          <section className="kidos-module" style={{ marginTop: 18 }} aria-label="Parent access"><h2>Parent access</h2><p>Guardian controls stay behind parent verification.</p><input ref={parentPinRef} aria-label="Parent PIN" type="password" inputMode="numeric" value={parentPin} onChange={(e) => setParentPin(e.target.value.replace(/\D/g,'').slice(0,8))} placeholder="Parent PIN"/><button type="button" onClick={requestParent}>Unlock Parent Workspace</button>{parentStatus && <div className="kidos-module-status" role="status">{parentStatus}</div>}<KidOSProfileCard profile={{ displayName: 'Alex', levelLabel: 'Explorer • Level 12' }} /></section>
        </div>
        <KidOSDock onNavigate={setActive} />
      </section>
    </main>
  );
}
