const categories = [
  ['➗', 'Math', 'Numbers, puzzles, and problem solving', 'https://www.khanacademy.org/math'],
  ['🔬', 'Science', 'Discover how the world works', 'https://www.khanacademy.org/science'],
  ['📚', 'Reading', 'Stories, vocabulary, and comprehension', 'https://www.khanacademy.org/ela'],
  ['✏️', 'Homework', 'Parent-approved learning tools', 'https://www.khanacademy.org/'],
] as const;

export default function KidOSLearnScreen({
  statusMessage,
  onOpen,
}: {
  statusMessage: string;
  onOpen(label: string, url: string): void;
}) {
  return (
    <section className="kidos-screen kidos-category-screen" data-testid="kidos-learn-screen">
      <header className="kidos-screen-header">
        <span className="kidos-screen-icon" aria-hidden="true">📘</span>
        <div><p className="eyebrow">Explore safely</p><h1>Learn</h1><p>Choose a subject. KidOS checks the destination with Guardian before it opens.</p></div>
      </header>
      <div className="kidos-category-grid">
        {categories.map(([icon, label, copy, url]) => (
          <button className="kidos-category-card" type="button" key={label} onClick={() => onOpen(label, url)}>
            <span aria-hidden="true">{icon}</span><strong>{label}</strong><small>{copy}</small>
          </button>
        ))}
      </div>
      <div className="kidos-action-status" role="status">{statusMessage}</div>
      <p className="kidos-third-party-note">Learning pages open in KidOS Safe Browser. Third-party services keep their own content and privacy policies.</p>
    </section>
  );
}
