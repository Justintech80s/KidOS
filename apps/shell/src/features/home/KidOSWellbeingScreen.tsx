import type { UsageStatus, WellbeingSettings } from '../../lib/kidos-api';

export default function KidOSWellbeingScreen({
  settings,
  usage,
  statusMessage,
  onChange,
  onSave,
}: {
  settings: WellbeingSettings;
  usage?: UsageStatus;
  statusMessage: string;
  onChange(next: WellbeingSettings): void;
  onSave(): void;
}) {
  const used = usage?.usedMinutes ?? 0;
  const limit = usage?.dailyLimitMinutes ?? settings.dailyMinutes;
  const percent = Math.min(100, Math.round((used / Math.max(1, limit)) * 100));

  return (
    <section className="kidos-screen kidos-category-screen" data-testid="kidos-wellbeing-screen">
      <header className="kidos-screen-header">
        <span className="kidos-screen-icon" aria-hidden="true">☀️</span>
        <div><p className="eyebrow">Healthy habits</p><h1>Wellbeing</h1><p>See today's balance. Parent-controlled limits cannot be changed from child mode.</p></div>
      </header>

      <div className="kidos-wellbeing-progress" aria-label="Daily screen use">
        <div><strong>{used} min used</strong><span>{usage?.remainingMinutes ?? Math.max(0, limit-used)} min remaining</span></div>
        <div className="kidos-progress-track"><span style={{ width: `${percent}%` }} /></div>
        {usage?.dailyLimitReached ? <p>Daily KidOS online/app limit reached. Local creation stays available.</p> : null}
        {usage?.windDownActive ? <p>Wind-down time is active.</p> : null}
        {usage?.breakDue ? <p>Good time for a short break.</p> : null}
      </div>

      <div className="kidos-category-grid">
        <article className="kidos-category-card">
          <span aria-hidden="true">⏱️</span><strong>Daily screen target</strong>
          <small>{settings.dailyMinutes} minutes per day · parent controlled</small>
        </article>
        <article className="kidos-category-card">
          <span aria-hidden="true">🧘</span><strong>Break reminder</strong>
          <small>Every {settings.breakEveryMinutes} minutes · parent controlled</small>
        </article>
        <article className="kidos-category-card">
          <span aria-hidden="true">🌙</span><strong>Wind-down</strong>
          <small>Starts at {String(settings.windDownHour).padStart(2,'0')}:00 · parent controlled</small>
        </article>
        <article className="kidos-category-card">
          <span aria-hidden="true">👁️</span><strong>Comfort</strong>
          <label><input type="checkbox" checked={settings.largeText} onChange={(event) => onChange({ ...settings, largeText: event.target.checked })} /> Larger text</label>
          <label><input type="checkbox" checked={settings.reducedMotion} onChange={(event) => onChange({ ...settings, reducedMotion: event.target.checked })} /> Reduce motion</label>
        </article>
      </div>

      <button className="kidos-primary-action" type="button" onClick={onSave}>Save comfort settings</button>
      <div className="kidos-action-status" role="status">{statusMessage}</div>
    </section>
  );
}
