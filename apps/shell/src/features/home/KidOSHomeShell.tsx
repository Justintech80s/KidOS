import { useEffect, useRef, useState } from 'react';
import type { ApprovedAppSummary, KidOSApi, WellbeingSettings } from '../../lib/kidos-api';
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

const DEFAULT_WELLBEING: WellbeingSettings = {
  dailyMinutes: 120,
  breakEveryMinutes: 30,
  windDownHour: 20,
  largeText: false,
  reducedMotion: false,
};

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
  const [approvedApps, setApprovedApps] = useState<ApprovedAppSummary[]>([]);
  const [appsStatus, setAppsStatus] = useState('Loading Guardian-approved apps...');
  const [wellbeing, setWellbeing] = useState<WellbeingSettings>(DEFAULT_WELLBEING);
  const [wellbeingStatus, setWellbeingStatus] = useState('Loading wellbeing settings...');
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

  useEffect(() => {
    let active = true;
    async function loadChildServices() {
      if (api.listApprovedApps) {
        try {
          const apps = await api.listApprovedApps();
          if (active) {
            setApprovedApps(apps);
            setAppsStatus(apps.length ? 'Approved apps are ready.' : 'No extra apps are approved yet.');
          }
        } catch {
          if (active) setAppsStatus('Guardian approved-app profile is unavailable.');
        }
      } else if (active) {
        setAppsStatus('Approved app launching is unavailable in this build.');
      }

      if (api.getWellbeing) {
        try {
          const next = await api.getWellbeing();
          if (active) {
            setWellbeing(next);
            setWellbeingStatus('Wellbeing settings loaded.');
          }
        } catch {
          if (active) setWellbeingStatus('Using default wellbeing settings.');
        }
      } else if (active) {
        setWellbeingStatus('Wellbeing settings are local to the Electron build.');
      }
    }
    void loadChildServices();
    return () => { active = false; };
  }, [api]);

  async function refreshApprovedApps() {
    if (!api.listApprovedApps) {
      setAppsStatus('Approved app launching is unavailable in this build.');
      return;
    }
    setAppsStatus('Checking Guardian-approved apps...');
    try {
      const apps = await api.listApprovedApps();
      setApprovedApps(apps);
      setAppsStatus(apps.length ? 'Approved apps refreshed.' : 'No extra apps are approved yet.');
    } catch {
      setAppsStatus('Guardian approved-app profile could not be read.');
    }
  }

  async function launchApprovedApp(appId: string) {
    if (!api.launchApprovedApp) {
      setAppsStatus('Approved app launching is unavailable in this build.');
      return;
    }
    setAppsStatus('Launching approved app...');
    try {
      await api.launchApprovedApp(appId);
      setAppsStatus('Approved app launched.');
    } catch {
      setAppsStatus('KidOS blocked or could not launch that app.');
    }
  }

  async function saveWellbeing() {
    if (!api.saveWellbeing) {
      setWellbeingStatus('Wellbeing saving is unavailable in this build.');
      return;
    }
    setWellbeingStatus('Saving wellbeing settings...');
    try {
      const saved = await api.saveWellbeing(wellbeing);
      setWellbeing(saved);
      setWellbeingStatus('Wellbeing settings saved on this device.');
    } catch {
      setWellbeingStatus('KidOS could not save wellbeing settings.');
    }
  }

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

  async function answerAi(query: string) {
    const q = query.trim();
    if (!q) return;
    setAiValue(q);
    setAiAnswer('KidOS AI is checking the protected learning service...');
    if (!api.askAi) {
      setAiAnswer('KidOS AI is not connected in this build.');
      return;
    }
    try {
      const result = await api.askAi(q);
      setAiAnswer(result.answer);
    } catch {
      setAiAnswer('KidOS AI could not answer safely right now. Try again later or ask a parent or teacher.');
    }
  }

  function askAi() {
    void answerAi(aiValue);
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
        return <KidOSWellbeingScreen settings={wellbeing} statusMessage={wellbeingStatus} onChange={setWellbeing} onSave={() => { void saveWellbeing(); }} />;
      case 'apps':
        return <KidOSMyAppsScreen apps={approvedApps} statusMessage={appsStatus} onRefresh={() => { void refreshApprovedApps(); }} onLaunch={(appId) => { void launchApprovedApp(appId); }} />;
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
            onSuggestion={(prompt) => { void answerAi(prompt); }}
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
