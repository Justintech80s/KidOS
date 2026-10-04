const categories = [
  ['🧩', 'Puzzles', 'Think, match, and solve'],
  ['🏎️', 'Adventure', 'Explore parent-approved games'],
  ['♟️', 'Strategy', 'Plan, build, and practice'],
  ['👥', 'Together', 'Family-friendly shared play'],
] as const;

const PBS_GAMES = 'https://pbskids.org/games/';

export default function KidOSPlayScreen({
  statusMessage,
  onOpen,
}: {
  statusMessage: string;
  onOpen(label: string, url: string): void;
}) {
  return (
    <section className="kidos-screen kidos-category-screen" data-testid="kidos-play-screen">
      <header className="kidos-screen-header">
        <span className="kidos-screen-icon" aria-hidden="true">🎮</span>
        <div><p className="eyebrow">Play safely</p><h1>Play</h1><p>Game destinations are checked by Guardian and open inside the protected KidOS browser.</p></div>
      </header>
      <div className="kidos-category-grid">
        {categories.map(([icon, label, copy]) => (
          <button className="kidos-category-card" type="button" key={label} onClick={() => onOpen(label, PBS_GAMES)}>
            <span aria-hidden="true">{icon}</span><strong>{label}</strong><small>{copy}</small>
          </button>
        ))}
      </div>
      <div className="kidos-action-status" role="status">{statusMessage}</div>
      <p className="kidos-third-party-note">KidOS currently routes Play to a parent-approved PBS KIDS game destination; PBS maintains its own content and policies.</p>
    </section>
  );
}
