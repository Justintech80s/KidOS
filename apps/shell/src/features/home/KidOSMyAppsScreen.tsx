import type { ApprovedAppSummary } from '../../lib/kidos-api';

export default function KidOSMyAppsScreen({
  apps,
  statusMessage,
  onRefresh,
  onLaunch,
}: {
  apps: ApprovedAppSummary[];
  statusMessage: string;
  onRefresh(): void;
  onLaunch(appId: string): void;
}) {
  return (
    <section className="kidos-screen kidos-category-screen" data-testid="kidos-apps-screen">
      <header className="kidos-screen-header">
        <span className="kidos-screen-icon" aria-hidden="true">▦</span>
        <div><p className="eyebrow">Parent-approved</p><h1>My Apps</h1><p>Only applications in the active Guardian profile appear here.</p></div>
      </header>

      <div className="kidos-category-grid kidos-app-status-grid">
        {apps.length === 0 ? (
          <article className="kidos-category-card">
            <span aria-hidden="true">🔒</span>
            <strong>No approved apps yet</strong>
            <small>A parent can add apps from Parent Dashboard → Windows Lockdown.</small>
          </article>
        ) : apps.map((app) => (
          <button className="kidos-category-card" type="button" key={app.id} onClick={() => onLaunch(app.id)}>
            <span aria-hidden="true">✅</span>
            <strong>{app.displayName}</strong>
            <small>Launch through the active Guardian-approved profile.</small>
          </button>
        ))}
      </div>

      <button className="kidos-approved-resource" type="button" onClick={onRefresh}>↻ Refresh approved apps</button>
      <div className="kidos-action-status" role="status">{statusMessage}</div>
    </section>
  );
}
