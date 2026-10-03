import type { WellbeingSettings } from '../../lib/kidos-api';

export default function KidOSWellbeingScreen({
  settings,
  statusMessage,
  onChange,
  onSave,
}: {
  settings: WellbeingSettings;
  statusMessage: string;
  onChange(next: WellbeingSettings): void;
  onSave(): void;
}) {
  return (
    <section className="kidos-screen kidos-category-screen" data-testid="kidos-wellbeing-screen">
      <header className="kidos-screen-header">
        <span className="kidos-screen-icon" aria-hidden="true">☀️</span>
        <div><p className="eyebrow">Healthy habits</p><h1>Wellbeing</h1><p>KidOS stores these comfort and screen-balance settings on this device.</p></div>
      </header>

      <div className="kidos-category-grid">
        <label className="kidos-category-card">
          <span aria-hidden="true">⏱️</span><strong>Daily screen target</strong>
          <input type="number" min={15} max={600} value={settings.dailyMinutes} onChange={(event) => onChange({ ...settings, dailyMinutes: Number(event.target.value) })} />
          <small>Minutes per day</small>
        </label>
        <label className="kidos-category-card">
          <span aria-hidden="true">🧘</span><strong>Break reminder</strong>
          <input type="number" min={10} max={120} value={settings.breakEveryMinutes} onChange={(event) => onChange({ ...settings, breakEveryMinutes: Number(event.target.value) })} />
          <small>Remind every N minutes</small>
        </label>
        <label className="kidos-category-card">
          <span aria-hidden="true">🌙</span><strong>Wind-down hour</strong>
          <input type="number" min={0} max={23} value={settings.windDownHour} onChange={(event) => onChange({ ...settings, windDownHour: Number(event.target.value) })} />
          <small>24-hour clock</small>
        </label>
        <article className="kidos-category-card">
          <span aria-hidden="true">👁️</span><strong>Comfort</strong>
          <label><input type="checkbox" checked={settings.largeText} onChange={(event) => onChange({ ...settings, largeText: event.target.checked })} /> Larger text</label>
          <label><input type="checkbox" checked={settings.reducedMotion} onChange={(event) => onChange({ ...settings, reducedMotion: event.target.checked })} /> Reduce motion</label>
        </article>
      </div>

      <button className="kidos-primary-action" type="button" onClick={onSave}>Save wellbeing settings</button>
      <div className="kidos-action-status" role="status">{statusMessage}</div>
    </section>
  );
}
