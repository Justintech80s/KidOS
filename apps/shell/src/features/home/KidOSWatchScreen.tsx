const categories = [
  ['🌎', 'Explore', 'Kid-safe discovery videos'],
  ['🧪', 'Science', 'Learn with visual experiments'],
  ['🐾', 'Animals', 'Nature and wildlife'],
  ['🎨', 'Create', 'Art, craft, and maker videos'],
] as const;

export default function KidOSWatchScreen() {
  return (
    <section className="kidos-screen kidos-category-screen" data-testid="kidos-watch-screen">
      <header className="kidos-screen-header">
        <span className="kidos-screen-icon" aria-hidden="true">▶️</span>
        <div><p className="eyebrow">Media safety on</p><h1>Watch</h1><p>Videos appear only after KidOS safety checks.</p></div>
      </header>
      <div className="kidos-category-grid">
        {categories.map(([icon, label, copy]) => (
          <article className="kidos-category-card" key={label}>
            <span aria-hidden="true">{icon}</span><strong>{label}</strong><small>{copy}</small>
          </article>
        ))}
      </div>
    </section>
  );
}
