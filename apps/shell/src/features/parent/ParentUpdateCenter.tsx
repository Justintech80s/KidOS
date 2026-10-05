import { useEffect, useState } from 'react';
import type { KidOSApi, UpdateStatus } from '../../lib/kidos-api';

const INITIAL: UpdateStatus = { status: 'idle' };

export default function ParentUpdateCenter({
  api,
}: {
  api: Pick<KidOSApi, 'getUpdateStatus' | 'checkUpdates' | 'downloadUpdate' | 'installUpdate'>;
}) {
  const [update, setUpdate] = useState<UpdateStatus>(INITIAL);
  const [pin, setPin] = useState('');
  const [message, setMessage] = useState('Loading KidOS update status...');

  async function refresh() {
    if (!api.getUpdateStatus) {
      setMessage('Automatic updates are unavailable in this build.');
      return;
    }
    try {
      const next = await api.getUpdateStatus();
      setUpdate(next);
      setMessage(next.error ? `Update error: ${next.error}` : 'KidOS update status loaded.');
    } catch {
      setMessage('KidOS update status could not be read.');
    }
  }

  useEffect(() => { void refresh(); }, []);

  async function check() {
    if (!api.checkUpdates) return;
    setMessage('Checking for KidOS updates...');
    try {
      const next = await api.checkUpdates();
      setUpdate(next);
      setMessage(next.status === 'available' ? `KidOS ${next.availableVersion ?? 'update'} is available.` : 'Update check completed.');
    } catch {
      setMessage('KidOS could not check for updates.');
    }
  }

  async function download() {
    if (!api.downloadUpdate || !pin.trim()) {
      setMessage('Enter the parent PIN to download a KidOS update.');
      return;
    }
    setMessage('Downloading KidOS update...');
    try {
      const next = await api.downloadUpdate(pin.trim());
      setUpdate(next);
      setMessage('KidOS update download started.');
    } catch {
      setMessage('KidOS did not start the update download. Check the parent PIN.');
    }
  }

  async function install() {
    if (!api.installUpdate || !pin.trim()) {
      setMessage('Enter the parent PIN to install a KidOS update.');
      return;
    }
    setMessage('Restarting KidOS to install the update...');
    try {
      await api.installUpdate(pin.trim());
    } catch {
      setMessage('KidOS could not install the update.');
    }
  }

  return (
    <section aria-label="KidOS update center">
      <h2>KidOS Updates</h2>
      <p>Current version: {update.currentVersion ?? 'unknown'} · Status: {update.status}</p>
      {update.availableVersion ? <p>Available version: {update.availableVersion}</p> : null}
      {typeof update.percent === 'number' ? <p>Download: {update.percent}%</p> : null}

      <label htmlFor="parent-update-pin">Parent PIN</label>
      <input id="parent-update-pin" type="password" inputMode="numeric" minLength={4} maxLength={8}
        value={pin} onChange={(event) => setPin(event.target.value)} autoComplete="off" />

      <div>
        <button type="button" onClick={() => { void check(); }}>Check for updates</button>
        <button type="button" disabled={update.status !== 'available'} onClick={() => { void download(); }}>Download update</button>
        <button type="button" disabled={update.status !== 'ready'} onClick={() => { void install(); }}>Install & restart</button>
      </div>
      <p role="status">{message}</p>
    </section>
  );
}
