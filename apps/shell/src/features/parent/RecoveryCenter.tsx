import { useEffect, useState } from 'react';
import type { KidOSApi, RecoveryAction, RecoveryStatus } from '../../lib/kidos-api';

type Props = {
  authorized: boolean;
  api: Pick<KidOSApi, 'getRecoveryStatus' | 'runParentRecovery'>;
};

export default function RecoveryCenter({ authorized, api }: Props) {
  const [status, setStatus] = useState<RecoveryStatus | null>(null);
  const [pin, setPin] = useState('');
  const [message, setMessage] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function refresh() {
    if (!api.getRecoveryStatus) return;
    setBusy(true);
    try {
      setStatus(await api.getRecoveryStatus());
      setMessage(null);
    } catch {
      setStatus({
        guardianHealthy: false,
        classifierHealthy: false,
        recoveryRequired: true,
        recoveryReason: 'KidOS Guardian could not be reached from the parent interface.',
        policyValid: false,
        lockdownState: 'unknown',
      });
      setMessage('Guardian is unavailable. Use the installed Windows recovery health check from an administrator account.');
    } finally {
      setBusy(false);
    }
  }

  useEffect(() => {
    if (authorized) void refresh();
  }, [authorized]);

  if (!authorized || !api.getRecoveryStatus || !api.runParentRecovery) return null;

  async function recover(action: RecoveryAction) {
    const runRecovery = api.runParentRecovery;
    if (!runRecovery) {
      setMessage('Recovery controls are unavailable in this build.');
      return;
    }
    if (!pin.trim()) {
      setMessage('Enter the parent PIN before using recovery controls.');
      return;
    }
    setBusy(true);
    try {
      const result = await runRecovery(pin.trim(), action);
      setPin('');
      setMessage(result);
      await refresh();
    } catch {
      setMessage('KidOS did not complete that recovery action. Check the parent PIN and system status.');
    } finally {
      setBusy(false);
    }
  }

  return (
    <section aria-label="KidOS Recovery Center" className="recovery-center">
      <h2>Recovery Center</h2>
      <p>Diagnose KidOS protection without exposing recovery controls to the child session.</p>

      {status ? (
        <div className="recovery-grid">
          <div><span>Guardian service</span><strong>{status.guardianHealthy ? 'Healthy' : 'Unavailable'}</strong></div>
          <div><span>Media classifier</span><strong>{status.classifierHealthy ? 'Healthy' : 'Fail-closed'}</strong></div>
          <div><span>Parent policy</span><strong>{status.policyValid ? 'Valid' : 'Needs repair'}</strong></div>
          <div><span>Windows lockdown</span><strong>{status.lockdownState.replaceAll('_', ' ')}</strong></div>
        </div>
      ) : null}

      {status?.recoveryRequired ? (
        <p role="alert" className="recovery-alert">
          Recovery is required{status.recoveryReason ? `: ${status.recoveryReason.replaceAll('-', ' ')}` : '.'}
        </p>
      ) : (
        <p className="recovery-ok">No active recovery warning.</p>
      )}

      <label htmlFor="recovery-parent-pin">Parent PIN</label>
      <input
        id="recovery-parent-pin"
        type="password"
        inputMode="numeric"
        minLength={4}
        maxLength={8}
        value={pin}
        onChange={(event) => setPin(event.target.value.replace(/\D/g, '').slice(0, 8))}
        autoComplete="off"
      />

      <div className="guardian-button-row">
        <button type="button" className="secondary-button" disabled={busy} onClick={refresh}>Recheck health</button>
        <button type="button" className="secondary-button" disabled={busy} onClick={() => recover('reset_policy_defaults')}>Reset safety policy</button>
        <button type="button" className="secondary-button" disabled={busy} onClick={() => recover('clear_recovery_marker')}>Clear resolved warning</button>
        <button type="button" className="secondary-button" disabled={busy} onClick={() => recover('remove_lockdown')}>Emergency remove lockdown</button>
      </div>

      <p className="recovery-note">
        Forgotten PIN recovery is intentionally not available to the child-facing app. A parent with Windows administrator access must use the installed KidOS recovery path.
      </p>
      {message ? <p role="status">{message}</p> : null}
    </section>
  );
}
