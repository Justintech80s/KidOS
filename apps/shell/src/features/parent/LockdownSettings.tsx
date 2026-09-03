import { useState } from 'react';
import type { KidOSApi, LockdownStatusView } from '../../lib/kidos-api';

type Props = {
  authorized: boolean;
  api: Pick<KidOSApi, 'lockdownStatus' | 'configureWindowsLockdown' | 'requestParentMaintenanceUnlock' | 'removeWindowsLockdown'>;
  initialStatus?: LockdownStatusView;
};

export default function LockdownSettings({ authorized, api, initialStatus = { state: 'unmanaged' } }: Props) {
  const [accountName, setAccountName] = useState('');
  const [accountRole, setAccountRole] = useState<'standard' | 'administrator' | 'unknown'>('standard');
  const [status, setStatus] = useState(initialStatus);
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  if (!authorized) return null;

  async function configure() {
    setError(null);
    if (!accountName.trim() || accountRole !== 'standard') {
      setError('Windows Lockdown Mode requires a validated standard child account.');
      return;
    }
    const next = await api.configureWindowsLockdown({ accountName: accountName.trim(), accountRole });
    setStatus(next);
    setMessage("Lockdown is prepared. Windows applies Assigned Access at the child's next sign-in.");
  }

  async function unlock() {
    const next = await api.requestParentMaintenanceUnlock(15);
    setStatus(next);
    setMessage(next.expiresAt ? `Parent maintenance access expires at ${next.expiresAt}.` : 'Parent maintenance access is temporary.');
  }

  async function remove() {
    const next = await api.removeWindowsLockdown();
    setStatus(next);
    setMessage('Windows Lockdown removal was requested by the authorized parent.');
  }

  return (
    <section aria-label="Windows Lockdown Mode">
      <h2>Windows Lockdown Mode</h2>
      <p>Limit the child Windows account to KidOS and parent-approved applications.</p>
      {status.state === 'restricted_safe_mode' ? (
        <p role="alert">Restricted Safe Mode is active because KidOS cannot confirm Windows lockdown protection.</p>
      ) : null}
      <p>Current state: <strong>{status.state.replaceAll('_', ' ')}</strong></p>

      <label htmlFor="lockdown-account">Child account name</label>
      <input id="lockdown-account" value={accountName} onChange={(event) => setAccountName(event.target.value)} />

      <label htmlFor="lockdown-role">Account role</label>
      <select id="lockdown-role" value={accountRole} onChange={(event) => setAccountRole(event.target.value as typeof accountRole)}>
        <option value="standard">Standard child account</option>
        <option value="administrator">Administrator</option>
        <option value="unknown">Unknown</option>
      </select>

      <button type="button" onClick={configure}>Configure lockdown</button>
      {status.state === 'locked' || status.state === 'parent_unlocked' ? (
        <button type="button" onClick={unlock}>15-minute maintenance unlock</button>
      ) : null}
      <button type="button" onClick={remove}>Remove lockdown</button>

      {error ? <p role="alert">{error}</p> : null}
      {message ? <p role="status">{message}</p> : null}
    </section>
  );
}
