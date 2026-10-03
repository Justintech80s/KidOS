import { useEffect, useState } from 'react';
import type { LockdownStatus, ParentPolicyConfig } from '@kidos/contracts';
import type { KidOSApi } from '../../lib/kidos-api';
import ParentDashboard from './ParentDashboard';

const DEFAULT_POLICY: ParentPolicyConfig = {
  childAge: 10,
  allowDomains: [],
  blockDomains: [],
  teenUnknownWebEnabled: false,
  socialAccess: [],
  downloadMode: 'require_parent_high_risk',
};

export default function KidOSParentWorkspace({ api, onClose }: { api: KidOSApi; onClose(): void }) {
  const [policy, setPolicy] = useState<ParentPolicyConfig>(DEFAULT_POLICY);
  const [lockdown, setLockdown] = useState<LockdownStatus>();
  const [message, setMessage] = useState('Loading protected parent settings...');

  useEffect(() => {
    let active = true;
    Promise.all([
      api.getParentPolicy ? api.getParentPolicy().catch(() => DEFAULT_POLICY) : Promise.resolve(DEFAULT_POLICY),
      api.lockdownStatus().catch(() => undefined),
    ]).then(([nextPolicy, nextLockdown]) => {
      if (!active) return;
      setPolicy(nextPolicy);
      setLockdown(nextLockdown);
      setMessage('Parent controls are connected to KidOS Guardian.');
    });
    return () => { active = false; };
  }, [api]);

  async function savePolicy(pin: string, next: ParentPolicyConfig) {
    if (!api.saveParentPolicy) throw new Error('Parent policy saving is unavailable.');
    await api.saveParentPolicy(pin, next);
    setPolicy(next);
  }

  return (
    <main className="kidos-shell-2026" data-testid="kidos-parent-workspace">
      <section className="kidos-stage">
        <header className="kidos-screen-header">
          <div>
            <p className="eyebrow">Parent protected</p>
            <h1>KidOS Parent Dashboard</h1>
            <p>{message}</p>
          </div>
          <button type="button" className="kidos-primary-action" onClick={onClose}>Return to child mode</button>
        </header>
        <div className="kidos-main">
          <ParentDashboard
            authorized
            savePolicy={savePolicy}
            initialPolicy={policy}
            initialLockdownStatus={lockdown}
            lockdownApi={api}
          />
        </div>
      </section>
    </main>
  );
}
