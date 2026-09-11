const appStates = [
  ['✅', 'Approved apps', 'Only apps already approved by a parent appear here.'],
  ['🛡️', 'Protected launch', 'KidOS never accepts an arbitrary executable path from this screen.'],
  ['🔒', 'Parent controls', 'Changes to approved apps stay behind Parent Access.'],
] as const;

export default function KidOSMyAppsScreen() {
  return (
    <section className="kidos-screen kidos-category-screen" data-testid="kidos-apps-screen">
      <header className="kidos-screen-header">
        <span className="kidos-screen-icon" aria-hidden="true">▦</span>
        <div><p className="eyebrow">Parent-approved</p><h1>My Apps</h1><p>Your safe app collection lives here.</p></div>
      </header>
      <div className="kidos-category-grid kidos-app-status-grid">
        {appStates.map(([icon, label, copy]) => (
          <article className="kidos-category-card" key={label}>
            <span aria-hidden="true">{icon}</span><strong>{label}</strong><small>{copy}</small>
          </article>
        ))}
      </div>
      <div className="kidos-action-status" role="status">Approved app launching stays locked until KidOS exposes a trusted approved-app launch capability.</div>
    </section>
  );
}
