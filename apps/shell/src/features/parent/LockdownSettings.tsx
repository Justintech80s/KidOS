import { useState } from 'react';
import type { LockdownStatus, ManagedAccountRole, ParentUnlockGrant } from '@kidos/contracts';
import type { KidOSApi } from '../../lib/kidos-api';

type Props = {
  authorized: boolean;
  api: Pick<KidOSApi, 'lockdownStatus' | 'configureWindowsLockdown' | 'requestParentMaintenanceUnlock' | 'removeWindowsLockdown'>;
  initialStatus?: LockdownStatus;
};

const windowsCapability = { platform: 'windows', supported: true, mechanism: 'assigned_access' } as const;

export default function LockdownSettings({
  authorized,
  api,
  initialStatus = { state: 'unmanaged', capability: windowsCapability },
}: Props) {
  const [accountName, setAccountName] = useState('');
  const [accountRole, setAccountRole] = useState<ManagedAccountRole>('standard');
  const [status, setStatus] = useState<LockdownStatus>(initialStatus);
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [parentPin, setParentPin] = useState('');

  if (!authorized) return null;

  async function configure() {
    setError(null);
    if (!accountName.trim() || accountRole !== 'standard') {
      setError('Windows Lockdown Mode requires a validated standard child account.');
      return;
    }
    const account = { id: accountName.trim(), displayName: accountName.trim(), role: accountRole } as const;
    const next = await api.configureWindowsLockdown({
      account,
      approvedApps: [{ id: 'kidos', displayName: 'KidOS', executablePath: 'KidOS.exe' }],
    });
    setStatus(next);
    setMessage("Lockdown is prepared. Windows applies Assigned Access at the child's next sign-in.");
  }

  async function unlock() {
    if (!parentPin.trim()) {
      setError('Enter the parent PIN to unlock maintenance access.');
      return;
    }
    const grant: ParentUnlockGrant = await api.requestParentMaintenanceUnlock(parentPin.trim(), 15);
    setParentPin('');
    setStatus((current) => ({ ...current, state: 'parent_unlocked', parentUnlock: grant }));
    setMessage(`Parent maintenance access expires at ${grant.expiresAt}.`);
  }

  async function remove() {
    if (!parentPin.trim()) {
      setError('Enter the parent PIN to remove Windows lockdown.');
      return;
    }
    const next = await api.removeWindowsLockdown(parentPin.trim());
    setParentPin('');
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
      <select id="lockdown-role" value={accountRole} onChange={(event) => setAccountRole(event.target.value as ManagedAccountRole)}>
        <option value="standard">Standard child account</option>
        <option value="administrator">Administrator</option>
        <option value="unknown">Unknown</option>
      </select>

      <label htmlFor="lockdown-parent-pin">Parent PIN for sensitive changes</label>
      <input id="lockdown-parent-pin" type="password" inputMode="numeric" minLength={4} maxLength={8} value={parentPin} onChange={(event) => setParentPin(event.target.value.replace(/\D/g, '').slice(0, 8))} autoComplete="off" />

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
