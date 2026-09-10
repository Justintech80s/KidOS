import type { KidOSDestination } from './KidOSSidebar';

const tiles: Array<{ destination: KidOSDestination; title: string; copy: string; icon: string; tone: string }> = [
  { destination: 'learn', title: 'Learn', copy: 'Math • Science • Reading', icon: '📘', tone: 'blue' },
  { destination: 'play', title: 'Play', copy: 'Fun & safe games', icon: '🎮', tone: 'purple' },
  { destination: 'create', title: 'Create', copy: 'Draw • Music • Build', icon: '🎨', tone: 'orange' },
  { destination: 'watch', title: 'Watch', copy: 'Videos picked for you', icon: '▶️', tone: 'red' },
  { destination: 'browser', title: 'Safe Browser', copy: 'Explore the web safely', icon: '🔎', tone: 'green' },
  { destination: 'ai', title: 'KidOS AI', copy: 'Ask and create safely', icon: '🤖', tone: 'sky' },
  { destination: 'wellbeing', title: 'Wellbeing', copy: 'Screen time & balance', icon: '☀️', tone: 'mint' },
  { destination: 'apps', title: 'My Apps', copy: 'Parent-approved apps', icon: '▦', tone: 'yellow' },
];

export default function KidOSHomeGrid({ onNavigate }: { onNavigate(destination: KidOSDestination): void }) {
  return (
    <section className="kidos-home-grid" data-testid="kidos-home-grid" aria-label="KidOS home destinations">
      {tiles.map((tile) => (
        <button type="button" key={tile.destination} className={`kidos-tile kidos-tile-${tile.tone}`} onClick={() => onNavigate(tile.destination)}>
          <span className="kidos-tile-icon">{tile.icon}</span><strong>{tile.title}</strong><small>{tile.copy}</small>
        </button>
      ))}
    </section>
  );
}
