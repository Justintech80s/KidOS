import { openApprovedResource } from '../../lib/approved-online-resources';

const categories = [
  ['🧩', 'Puzzles', 'Think, match, and solve'],
  ['🏎️', 'Adventure', 'Parent-approved games only'],
  ['♟️', 'Strategy', 'Plan, build, and practice'],
  ['👥', 'Together', 'Safe local play experiences'],
] as const;

export default function KidOSPlayScreen() {
  return (
    <section className="kidos-screen kidos-category-screen" data-testid="kidos-play-screen">
      <header className="kidos-screen-header">
        <span className="kidos-screen-icon" aria-hidden="true">🎮</span>
        <div><p className="eyebrow">Play safely</p><h1>Play</h1><p>Only games approved for this KidOS profile are available.</p></div>
      </header>
      <div className="kidos-category-grid">
        {categories.map(([icon, label, copy]) => (
          <article className="kidos-category-card" key={label}>
            <span aria-hidden="true">{icon}</span><strong>{label}</strong><small>{copy}</small>
          </article>
        ))}
      </div>
      <button className="kidos-approved-resource" type="button" onClick={() => { void openApprovedResource('pbs-kids'); }}>
        🎮 Open PBS KIDS Games
      </button>
      <p className="kidos-third-party-note">Third-party game service. KidOS is not affiliated with PBS.</p>
    </section>
  );
}
