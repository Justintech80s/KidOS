import type { KidOSSystemStatus } from './system-status';
import KidOSSafetyStatus from './KidOSSafetyStatus';

export interface KidOSSafeBrowserScreenProps {
  value: string;
  statusMessage: string;
  protectionStatus: KidOSSystemStatus;
  onValueChange(value: string): void;
  onSubmit(): void;
  onShortcut(query: string): void;
}

const shortcuts = [
  ['📚', 'Learning', 'learning for kids'],
  ['🔬', 'Science', 'science for kids'],
  ['🐾', 'Animals', 'animals for kids'],
  ['🚀', 'Space', 'space for kids'],
  ['🎨', 'Arts & Crafts', 'arts and crafts for kids'],
  ['➗', 'Math', 'math for kids'],
] as const;

export default function KidOSSafeBrowserScreen({
  value,
  statusMessage,
  protectionStatus,
  onValueChange,
  onSubmit,
  onShortcut,
}: KidOSSafeBrowserScreenProps) {
  return (
    <section className="kidos-screen kidos-browser-screen" data-testid="kidos-browser-screen">
      <header className="kidos-screen-header">
        <span className="kidos-screen-icon" aria-hidden="true">🔎</span>
        <div>
          <p className="eyebrow">Protected browsing</p>
          <h1>Safe Browser</h1>
          <p>Every destination is checked by KidOS before it can open.</p>
        </div>
      </header>

      <div className="kidos-browser-card">
        <form className="kidos-browser-search" onSubmit={(event) => { event.preventDefault(); onSubmit(); }}>
          <span aria-hidden="true">⌕</span>
          <input
            aria-label="Protected web address"
            value={value}
            onChange={(event) => onValueChange(event.target.value)}
            placeholder="Search or enter a website"
          />
          <button type="submit">Search safely</button>
        </form>

        <div className="kidos-shortcut-grid" aria-label="Safe search topics">
          {shortcuts.map(([icon, label, query]) => (
            <button type="button" key={label} onClick={() => onShortcut(query)}>
              <span aria-hidden="true">{icon}</span>
              <strong>{label}</strong>
            </button>
          ))}
        </div>
      </div>

      <KidOSSafetyStatus status={protectionStatus} />
      {statusMessage && <div className="kidos-action-status" role="status">{statusMessage}</div>}
    </section>
  );
}
