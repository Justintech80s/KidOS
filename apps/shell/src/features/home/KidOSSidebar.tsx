export type KidOSDestination = 'home' | 'apps' | 'learn' | 'create' | 'play' | 'watch' | 'music' | 'browser' | 'ai' | 'wellbeing' | 'parent';

const items: Array<[KidOSDestination, string, string]> = [
  ['home', 'Home', '⌂'], ['learn', 'Learn', '📘'], ['play', 'Play', '🎮'], ['create', 'Create', '🎨'],
  ['watch', 'Watch', '▶'], ['browser', 'Safe Browser', '🔎'], ['apps', 'My Apps', '▦'], ['ai', 'KidOS AI', '✦'], ['wellbeing', 'Wellbeing', '☀'],
];

export default function KidOSSidebar({ active, onNavigate, onParentRequested }: { active: KidOSDestination; onNavigate(destination: KidOSDestination): void; onParentRequested(): void }) {
  return (
    <aside className="kidos-sidebar" data-testid="kidos-sidebar" aria-label="KidOS navigation">
      <div className="kidos-brand"><span className="kidos-brand-shield">🛡</span><div><strong>KidOS</strong><small>A Safer, Brighter Tomorrow</small></div></div>
      <nav>
        {items.map(([destination, label, icon]) => <button key={destination} type="button" className={active === destination ? 'is-active' : ''} aria-current={active === destination ? 'page' : undefined} onClick={() => onNavigate(destination)}><span>{icon}</span>{label}</button>)}
      </nav>
      <button type="button" className={`kidos-parent-entry${active === 'parent' ? ' is-active' : ''}`} aria-current={active === 'parent' ? 'page' : undefined} onClick={onParentRequested}>🔒 Parent</button>
    </aside>
  );
}
