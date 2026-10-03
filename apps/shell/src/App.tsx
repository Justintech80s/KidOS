import { useEffect, useState } from 'react';
import KidOSParentWorkspace from './features/parent/KidOSParentWorkspace';
import KidOSHomeShell from './features/home/KidOSHomeShell';
import {
  isDesktopPreviewRuntime,
  runtimeKidOSApi,
  type GuardianStatus,
  type KidOSApi,
} from './lib/kidos-api';

type ProtectionState = 'checking' | GuardianStatus;

interface AppProps {
  api?: KidOSApi;
  desktopPreview?: boolean;
}

export default function App({ api, desktopPreview = isDesktopPreviewRuntime() }: AppProps) {
  const activeApi = api ?? runtimeKidOSApi();
  const [protectionState, setProtectionState] = useState<ProtectionState>('checking');
  const [parentWorkspace, setParentWorkspace] = useState(false);

  useEffect(() => {
    let active = true;

    activeApi.guardianStatus()
      .then((status) => {
        if (active) setProtectionState(status);
      })
      .catch(() => {
        if (active) setProtectionState('restricted_safe_mode');
      });

    return () => {
      active = false;
    };
  }, [activeApi]);

  if (protectionState === 'checking') {
    return (
      <main className="kidos-shell">
        <section className="hero" aria-live="polite">
          <p className="eyebrow">KidOS</p>
          <h1>Checking protection...</h1>
          <p>Creation, web access, and privileged actions stay locked until Guardian is healthy.</p>
        </section>
      </main>
    );
  }

  if (protectionState === 'restricted_safe_mode' && !desktopPreview) {
    return (
      <main className="kidos-shell">
        <nav className="topbar" aria-label="KidOS status">
          <strong>KidOS</strong>
          <span className="safety-pill">Restricted</span>
        </nav>
        <section className="hero" aria-live="assertive">
          <p className="eyebrow">Guardian protection required</p>
          <h1>Restricted safe mode</h1>
          <p>
            KidOS has locked creation, protected web, downloads, and privileged actions because
            Guardian is unavailable or does not have a valid safety policy.
          </p>
        </section>
      </main>
    );
  }

  if (parentWorkspace) {
    return <KidOSParentWorkspace api={activeApi} onClose={() => setParentWorkspace(false)} />;
  }

  return <KidOSHomeShell api={activeApi} onOpenParentWorkspace={() => setParentWorkspace(true)} />;
}
