import { type FormEvent, useState } from 'react';
import type { WorkspacePlan } from '@kidos/contracts';
import type { KidOSApi } from '../../lib/kidos-api';

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
  const [prompt, setPrompt] = useState('');
  const [plan, setPlan] = useState<WorkspacePlan | null>(null);
  const [url, setUrl] = useState('');
  const [navigationState, setNavigationState] = useState<string | null>(null);

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

    const decision = await api.evaluateNavigation(destination);
    if (decision === 'allow') {
      setNavigationState(`Opening ${destination}`);
    } else if (decision === 'block') {
      setNavigationState('Site blocked by KidOS');
    } else {
      setNavigationState('Parent approval required');
    }
  }

  if (plan) {
    return (
      <main className="kidos-shell">
        <nav className="topbar" aria-label="KidOS status">
          <strong>KidOS</strong>
          <span className="safety-pill">Protected</span>
        </nav>
        <section className="hero">
          <p className="eyebrow">Safe workspace ready</p>
          <h1>{plan.title}</h1>
          <p>{workspaceLabel(plan.kind)}</p>
          <button type="button" onClick={() => setPlan(null)}>
            Back home
          </button>
        </section>
      </main>
    );
  }

  return (
    <main className="kidos-shell">
      <nav className="topbar" aria-label="KidOS status">
        <strong>KidOS</strong>
        <span className="safety-pill">Protected</span>
      </nav>

      <section className="hero">
        <p className="eyebrow">Create safely with KidOS</p>
        <h1>What do you want to create?</h1>
        <p className="hero-copy">
          Tell KidOS what you want to make. It will open only the tools allowed for your profile.
        </p>
        <form className="creation-bar" onSubmit={createWorkspace}>
          <label htmlFor="creation-request">Ask KidOS</label>
          <input
            id="creation-request"
            value={prompt}
            onChange={(event) => setPrompt(event.target.value)}
            placeholder="Make a space story..."
          />
          <button type="submit">Create</button>
        </form>
      </section>

      <section className="starter-grid" aria-label="Starter workspaces">
        <article>
          <h2>Story</h2>
          <p>Write characters, scenes, and adventures.</p>
        </article>
        <article>
          <h2>Draw &amp; Present</h2>
          <p>Create pictures, posters, and presentations.</p>
        </article>
        <article>
          <h2>Beginner Coding</h2>
          <p>Build simple games and learn coding safely.</p>
        </article>
      </section>

      <section className="protected-web" aria-label="Protected web">
        <h2>Protected web</h2>
        <p>KidOS checks a site with the safety policy before it can open.</p>
        <form onSubmit={checkSite}>
          <label htmlFor="protected-web-address">Protected web address</label>
          <input
            id="protected-web-address"
            value={url}
            onChange={(event) => setUrl(event.target.value)}
            placeholder="https://example.com"
          />
          <button type="submit">Check site</button>
        </form>
        {navigationState ? <p role="status">{navigationState}</p> : null}
      </section>
    </main>
  );
}
