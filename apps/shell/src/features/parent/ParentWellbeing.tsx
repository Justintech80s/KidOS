import { useEffect, useState } from 'react';
import type { KidOSApi, UsageStatus, WellbeingSettings } from '../../lib/kidos-api';

const DEFAULTS: WellbeingSettings = {
  dailyMinutes: 120,
  breakEveryMinutes: 30,
  windDownHour: 20,
  largeText: false,
  reducedMotion: false,
};

export default function ParentWellbeing({
  api,
}: {
  api: Pick<KidOSApi, 'getWellbeing' | 'getUsageStatus' | 'saveParentWellbeing' | 'createDataBackup'>;
}) {
  const [settings, setSettings] = useState<WellbeingSettings>(DEFAULTS);
  const [usage, setUsage] = useState<UsageStatus>();
  const [pin, setPin] = useState('');
  const [status, setStatus] = useState('Loading screen-time controls...');

  async function refresh() {
    try {
      const [next, nextUsage] = await Promise.all([
        api.getWellbeing ? api.getWellbeing() : Promise.resolve(DEFAULTS),
        api.getUsageStatus ? api.getUsageStatus() : Promise.resolve(undefined),
      ]);
      setSettings(next);
      setUsage(nextUsage);
      setStatus('Screen-time controls loaded.');
    } catch {
      setStatus('Screen-time controls are unavailable.');
    }
  }

  useEffect(() => { void refresh(); }, []);

  async function save() {
    if (!api.saveParentWellbeing || !pin.trim()) {
      setStatus('Enter the parent PIN to change KidOS limits.');
      return;
    }
    setStatus('Saving parent-controlled limits...');
    try {
      const saved = await api.saveParentWellbeing(pin.trim(), settings);
      setSettings(saved);
      setPin('');
      setStatus('KidOS screen-time limits saved.');
      await refresh();
    } catch {
      setStatus('KidOS did not save the limits. Check the parent PIN.');
    }
  }

  async function backup() {
    if (!api.createDataBackup) {
      setStatus('Manual data backup is unavailable in this build.');
      return;
    }
    setStatus('Creating KidOS data backup...');
    try {
      const result = await api.createDataBackup();
      setStatus(result ? 'KidOS data backup created.' : 'KidOS data is already protected; no backup was needed.');
    } catch {
      setStatus('KidOS could not create a data backup.');
    }
  }

  return (
    <section aria-label="Parent wellbeing controls">
      <h2>Screen time & wellbeing</h2>
      <p>Today: {usage?.usedMinutes ?? 0} minutes used · {usage?.remainingMinutes ?? settings.dailyMinutes} minutes remaining.</p>

      <label htmlFor="parent-daily-minutes">Daily KidOS limit</label>
      <input id="parent-daily-minutes" type="number" min={15} max={600} value={settings.dailyMinutes}
        onChange={(event) => setSettings({ ...settings, dailyMinutes: Number(event.target.value) })} />

      <label htmlFor="parent-break-minutes">Break reminder interval</label>
      <input id="parent-break-minutes" type="number" min={10} max={120} value={settings.breakEveryMinutes}
        onChange={(event) => setSettings({ ...settings, breakEveryMinutes: Number(event.target.value) })} />

      <label htmlFor="parent-wind-down">Wind-down hour</label>
      <input id="parent-wind-down" type="number" min={0} max={23} value={settings.windDownHour}
        onChange={(event) => setSettings({ ...settings, windDownHour: Number(event.target.value) })} />

      <label htmlFor="parent-wellbeing-pin">Parent PIN</label>
      <input id="parent-wellbeing-pin" type="password" inputMode="numeric" minLength={4} maxLength={8}
        value={pin} onChange={(event) => setPin(event.target.value)} autoComplete="off" />

      <div>
        <button type="button" onClick={() => { void save(); }}>Save screen-time limits</button>
        <button type="button" onClick={() => { void backup(); }}>Create KidOS data backup</button>
      </div>
      <p role="status">{status}</p>
    </section>
  );
}
