const categories = [
  ['🌎', 'Explore', 'Kid-safe discovery videos', 'https://kids.youtube.com/'],
  ['🧪', 'Science', 'Visual science learning', 'https://kids.youtube.com/search?q=science'],
  ['🐾', 'Animals', 'Nature and wildlife', 'https://kids.youtube.com/search?q=animals'],
  ['🎨', 'Create', 'Art, craft, and maker videos', 'https://kids.youtube.com/search?q=art%20crafts'],
] as const;

export default function KidOSWatchScreen({
  statusMessage,
  onOpen,
}: {
  statusMessage: string;
  onOpen(label: string, url: string): void;
}) {
  return (
    <section className="kidos-screen kidos-category-screen" data-testid="kidos-watch-screen">
      <header className="kidos-screen-header">
        <span className="kidos-screen-icon" aria-hidden="true">▶️</span>
        <div><p className="eyebrow">Protected destination</p><h1>Watch</h1><p>KidOS Guardian approves the destination before video services open in the protected browser.</p></div>
      </header>
      <div className="kidos-category-grid">
        {categories.map(([icon, label, copy, url]) => (
          <button className="kidos-category-card" type="button" key={label} onClick={() => onOpen(label, url)}>
            <span aria-hidden="true">{icon}</span><strong>{label}</strong><small>{copy}</small>
          </button>
        ))}
      </div>
      <div className="kidos-action-status" role="status">{statusMessage}</div>
      <p className="kidos-third-party-note">Streaming content remains subject to the provider's own child-safety and parent-control policies. KidOS local media classification applies to supported local/downloaded media.</p>
    </section>
  );
}
