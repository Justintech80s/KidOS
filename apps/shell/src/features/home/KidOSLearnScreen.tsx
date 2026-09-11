const categories = [
  ['➗', 'Math', 'Numbers, puzzles, and problem solving'],
  ['🔬', 'Science', 'Discover how the world works'],
  ['📚', 'Reading', 'Stories, vocabulary, and comprehension'],
  ['✏️', 'Homework', 'Parent-approved learning tools'],
] as const;

export default function KidOSLearnScreen() {
  return (
    <section className="kidos-screen kidos-category-screen" data-testid="kidos-learn-screen">
      <header className="kidos-screen-header">
        <span className="kidos-screen-icon" aria-hidden="true">📘</span>
        <div><p className="eyebrow">Explore safely</p><h1>Learn</h1><p>Choose a subject and keep growing.</p></div>
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
