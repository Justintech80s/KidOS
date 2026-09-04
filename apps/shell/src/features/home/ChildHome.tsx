import { type FormEvent, useEffect, useMemo, useState } from 'react';
import type { WorkspacePlan } from '@kidos/contracts';
import type { KidOSApi } from '../../lib/kidos-api';
import { prepareProtectedNavigation } from '../browser/protected-navigation';

type Section =
  | 'Home'
  | 'Learn'
  | 'Play'
  | 'Create'
  | 'Watch'
  | 'Search'
  | 'My Apps'
  | 'KidOS AI'
  | 'Messages'
  | 'Settings';

const navItems: Array<{ label: Section; icon: string }> = [
  { label: 'Home', icon: '⌂' },
  { label: 'Learn', icon: '📘' },
  { label: 'Play', icon: '🎮' },
  { label: 'Create', icon: '🎨' },
  { label: 'Watch', icon: '▶' },
  { label: 'Search', icon: '⌕' },
  { label: 'My Apps', icon: '▦' },
  { label: 'KidOS AI', icon: '✦' },
  { label: 'Messages', icon: '✉' },
  { label: 'Settings', icon: '⚙' },
];

const cards: Array<{ section: Section; icon: string; title: string; copy: string; tone: string }> = [
  { section: 'Learn', icon: '📗', title: 'Learn', copy: 'Math • Science • Reading', tone: 'blue' },
  { section: 'Play', icon: '🎮', title: 'Play', copy: 'Fun & safe games', tone: 'purple' },
  { section: 'Create', icon: '🖌️', title: 'Create', copy: 'Draw • Music • Build', tone: 'orange' },
  { section: 'Watch', icon: '▶️', title: 'Watch', copy: 'Videos picked for you', tone: 'red' },
  { section: 'Search', icon: '🔎', title: 'Safe Search', copy: 'Explore the web safely', tone: 'green' },
  { section: 'My Apps', icon: '▦', title: 'My Apps', copy: 'Your approved apps', tone: 'yellow' },
  { section: 'KidOS AI', icon: '🤖', title: 'KidOS AI', copy: 'Ask me anything', tone: 'sky' },
  { section: 'Settings', icon: '🛡️', title: 'Wellbeing', copy: 'Screen time & balance', tone: 'mint' },
];

const sectionDescriptions: Record<Exclude<Section, 'Home' | 'Create' | 'Search'>, string> = {
  Learn: 'Your approved learning tools and school-friendly activities will live here.',
  Play: 'Only games approved for your KidOS profile will appear here.',
  Watch: 'KidOS will show videos that pass your profile and media-safety rules.',
  'My Apps': 'This is your personal shelf of approved KidOS apps.',
  'KidOS AI': 'Your protected AI helper for questions, learning, and creative ideas.',
  Messages: 'Safe family and approved-contact messages will appear here.',
  Settings: 'Profile, accessibility, sound, display, and wellbeing controls will live here.',
};

function workspaceLabel(kind: WorkspacePlan['kind']) {
  switch (kind) {
    case 'story':
      return 'Story workspace';
    case 'drawing_presentation':
      return 'Draw & Present workspace';
    case 'beginner_coding':
      return 'Beginner Coding workspace';
  }
}

export default function ChildHome({ api }: { api: KidOSApi }) {
  const [activeSection, setActiveSection] = useState<Section>('Home');
  const [prompt, setPrompt] = useState('');
  const [plan, setPlan] = useState<WorkspacePlan | null>(null);
  const [url, setUrl] = useState('');
  const [navigationState, setNavigationState] = useState<string | null>(null);
  const [demoStep, setDemoStep] = useState(0);

  const demoMode = useMemo(
    () =>
      typeof window !== 'undefined' &&
      new URLSearchParams(window.location.search).get('demo') === '1',
    [],
  );

  const demoSequence = useMemo<Section[]>(
    () => ['Home', 'Learn', 'Play', 'Create', 'Search', 'KidOS AI', 'Settings', 'Home'],
    [],
  );

  const currentDate = useMemo(
    () =>
      new Intl.DateTimeFormat('en-US', {
        weekday: 'long',
        month: 'short',
        day: 'numeric',
      }).format(new Date()),
    [],
  );

  useEffect(() => {
    if (!demoMode) return;

    const applyDemoStep = (step: number) => {
      const section = demoSequence[step] ?? 'Home';
      setActiveSection(section);
      setPlan(null);

      if (section === 'Create') {
        setPrompt('Make a space story with friendly robots');
      } else {
        setPrompt('');
      }

      if (section === 'Search') {
        setUrl('https://blocked.example');
        setNavigationState('Site blocked by KidOS');
      } else {
        setUrl('');
        setNavigationState(null);
      }
    };

    applyDemoStep(0);
    const interval = window.setInterval(() => {
      setDemoStep((current) => {
        const next = (current + 1) % demoSequence.length;
        applyDemoStep(next);
        return next;
      });
    }, 2800);

    return () => window.clearInterval(interval);
  }, [demoMode, demoSequence]);

  async function createWorkspace(event: FormEvent) {
    event.preventDefault();
    const request = prompt.trim();
    if (!request) return;
    setPlan(await api.planWorkspace(request));
  }

  async function checkSite(event: FormEvent) {
    event.preventDefault();
    const destination = url.trim();
    if (!destination) return;

    const result = await prepareProtectedNavigation(destination, (checkedUrl) =>
      api.evaluateNavigation(checkedUrl),
    );

    if (result.state === 'load') {
      setNavigationState(`Opening ${result.url}`);
    } else if (result.state === 'blocked') {
      setNavigationState('Site blocked by KidOS');
    } else {
      setNavigationState('Parent approval required');
    }
  }

  function openSection(section: Section) {
    setActiveSection(section);
    setPlan(null);
    setNavigationState(null);
  }

  if (plan) {
    return (
      <main className="kidos-shell concept-shell">
      {demoMode ? (
        <div className="demo-badge" role="status">
          <span className="demo-dot" />
          KidOS Demo • {demoSequence[demoStep]}
        </div>
      ) : null}
        <section className="workspace-screen">
          <span className="workspace-badge">Safe workspace ready</span>
          <div className="workspace-icon">✨</div>
          <h1>{plan.title}</h1>
          <p>{workspaceLabel(plan.kind)}</p>
          <button className="primary-button" type="button" onClick={() => setPlan(null)}>
            Back home
          </button>
        </section>
      </main>
    );
  }

  return (
    <main className="kidos-shell concept-shell">
      {demoMode ? (
        <div className="demo-badge" role="status">
          <span className="demo-dot" />
          KidOS Demo • {demoSequence[demoStep]}
        </div>
      ) : null}
      <aside className="sidebar" aria-label="KidOS navigation">
        <div className="brand">
          <div className="brand-mark">🐻</div>
          <div>
            <strong>KidOS</strong>
            <span>A Safer, Brighter Tomorrow</span>
          </div>
        </div>

        <nav className="side-nav">
          {navItems.map((item) => (
            <button
              type="button"
              key={item.label}
              className={activeSection === item.label ? 'nav-item active' : 'nav-item'}
              onClick={() => openSection(item.label)}
            >
              <span>{item.icon}</span>
              {item.label}
            </button>
          ))}
        </nav>

        <div className="profile-card">
          <div className="avatar">🧑🏽</div>
          <div>
            <strong>Alex</strong>
            <span>Explorer • Level 12</span>
          </div>
          <span className="profile-more">•••</span>
        </div>
      </aside>

      <section className="desktop">
        <header className="desktop-topbar">
          <label className="global-search">
            <span>⌕</span>
            <input
              aria-label="Search KidOS"
              placeholder="Search KidOS..."
              onFocus={() => activeSection === 'Home' && setActiveSection('Search')}
            />
          </label>

          <div className="status-cluster" aria-label="KidOS status">
            <span>🔔</span>
            <span className="safe-status">🛡 Safe Mode: ON</span>
            <span>📶</span>
            <span>🔋 92%</span>
            <span className="date-block">{currentDate}</span>
          </div>
        </header>

        <div className="scenery" aria-hidden="true">
          <div className="sun" />
          <div className="mountain mountain-one" />
          <div className="mountain mountain-two" />
          <div className="hill hill-one" />
          <div className="hill hill-two" />
          <div className="lake" />
          <div className="treehouse">🏡</div>
        </div>

        <div className="desktop-content">
          {activeSection === 'Home' ? (
            <>
              <section className="welcome-row">
                <div>
                  <p className="eyebrow">Protected profile</p>
                  <h1>Good Morning, Alex! 👋</h1>
                  <p className="welcome-copy">What would you like to do today?</p>
                </div>
                <div className="guardian-chip">🛡 Guardian healthy</div>
              </section>

              <section className="app-grid" aria-label="KidOS apps">
                {cards.map((card) => (
                  <button
                    type="button"
                    key={card.title}
                    className={`app-card ${card.tone}`}
                    onClick={() => openSection(card.section)}
                  >
                    <span className="app-icon">{card.icon}</span>
                    <strong>{card.title}</strong>
                    <small>{card.copy}</small>
                  </button>
                ))}
              </section>

              <section className="creation-panel">
                <div>
                  <span className="mini-label">KidOS Create</span>
                  <h2>What do you want to create?</h2>
                  <p>KidOS opens only the tools allowed for your profile.</p>
                </div>
                <form className="creation-bar" onSubmit={createWorkspace}>
                  <label className="sr-only" htmlFor="creation-request">Ask KidOS</label>
                  <input
                    id="creation-request"
                    value={prompt}
                    onChange={(event) => setPrompt(event.target.value)}
                    placeholder="Make a space story..."
                    aria-label="Ask KidOS"
                  />
                  <button type="submit">Create</button>
                </form>
              </section>

              <section className="quote-card">
                <span>☀️</span>
                <div>
                  <h2>“The future belongs to curious minds.”</h2>
                  <p>Keep exploring, learning, and creating.</p>
                </div>
              </section>
            </>
          ) : activeSection === 'Create' ? (
            <section className="module-panel">
              <span className="module-icon">🎨</span>
              <p className="eyebrow">Create safely</p>
              <h1>What do you want to create?</h1>
              <p className="module-copy">Stories, drawings, presentations, and beginner coding start here.</p>
              <form className="creation-bar module-form" onSubmit={createWorkspace}>
                <label className="sr-only" htmlFor="creation-request-module">Ask KidOS</label>
                <input
                  id="creation-request-module"
                  value={prompt}
                  onChange={(event) => setPrompt(event.target.value)}
                  placeholder="Make a space story..."
                  aria-label="Ask KidOS"
                />
                <button type="submit">Create</button>
              </form>
            </section>
          ) : activeSection === 'Search' ? (
            <section className="module-panel">
              <span className="module-icon">🔎</span>
              <p className="eyebrow">Safe Search</p>
              <h1>Explore the web safely</h1>
              <p className="module-copy">KidOS checks destinations against the active safety policy before opening them.</p>
              <form className="protected-web-form" onSubmit={checkSite}>
                <label htmlFor="protected-web-address">Protected web address</label>
                <div>
                  <input
                    id="protected-web-address"
                    value={url}
                    onChange={(event) => setUrl(event.target.value)}
                    placeholder="https://example.com"
                  />
                  <button type="submit">Check site</button>
                </div>
              </form>
              {navigationState ? <p className="navigation-status" role="status">{navigationState}</p> : null}
            </section>
          ) : (
            <section className="module-panel">
              <span className="module-icon">
                {navItems.find((item) => item.label === activeSection)?.icon}
              </span>
              <p className="eyebrow">KidOS module</p>
              <h1>{activeSection}</h1>
              <p className="module-copy">
                {sectionDescriptions[activeSection as Exclude<Section, 'Home' | 'Create' | 'Search'>]}
              </p>
              <div className="coming-soon-card">
                <strong>Interface shell connected</strong>
                <span>This screen is ready for the next functional module.</span>
              </div>
            </section>
          )}
        </div>

        <footer className="dock" aria-label="KidOS dock">
          {[
            ['📁', 'Files'],
            ['🌐', 'Browser'],
            ['📝', 'Notes'],
            ['🧮', 'Calculator'],
            ['📷', 'Camera'],
            ['🎵', 'Music'],
            ['🗑️', 'Trash'],
          ].map(([icon, label]) => (
            <button type="button" key={label} title={label}>
              <span>{icon}</span>
              <small>{label}</small>
            </button>
          ))}
        </footer>
      </section>
    </main>
  );
}
