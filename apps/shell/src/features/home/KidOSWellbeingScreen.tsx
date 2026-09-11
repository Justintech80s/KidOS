const wellbeing = [
  ['⏱️', 'Screen Balance', 'See healthy time and break guidance'],
  ['🧘', 'Take a Break', 'Pause, stretch, and reset'],
  ['👁️', 'Comfort', 'Simple display and accessibility guidance'],
  ['🌙', 'Wind Down', 'Gentle reminders for quieter time'],
] as const;

export default function KidOSWellbeingScreen() {
  return (
    <section className="kidos-screen kidos-category-screen" data-testid="kidos-wellbeing-screen">
      <header className="kidos-screen-header">
        <span className="kidos-screen-icon" aria-hidden="true">☀️</span>
        <div><p className="eyebrow">Healthy habits</p><h1>Wellbeing</h1><p>Balance screen time with breaks, comfort, and healthy routines.</p></div>
      </header>
      <div className="kidos-category-grid">
        {wellbeing.map(([icon, label, copy]) => (
          <article className="kidos-category-card" key={label}>
            <span aria-hidden="true">{icon}</span><strong>{label}</strong><small>{copy}</small>
          </article>
        ))}
      </div>
    </section>
  );
}
