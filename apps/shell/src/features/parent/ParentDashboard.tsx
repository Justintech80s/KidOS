import { type FormEvent, useState } from 'react';
import type { DownloadMode, ParentPolicyConfig, SocialAccessMode } from '@kidos/contracts';
import type { KidOSApi } from '../../lib/kidos-api';
import LockdownSettings from './LockdownSettings';
import SafetySummary, { type SafetySummaryData } from './SafetySummary';

type ParentDashboardProps = {
  authorized: boolean;
  savePolicy: (pin: string, policy: ParentPolicyConfig) => Promise<void>;
  safetySummary?: SafetySummaryData;
  clearSafetyEvents?: () => Promise<void>;
  lockdownApi?: Pick<KidOSApi, 'lockdownStatus' | 'configureWindowsLockdown' | 'requestParentMaintenanceUnlock' | 'removeWindowsLockdown'>;
};

function splitDomains(value: string): string[] {
  return value.split(/\r?\n/).map((domain) => domain.trim().toLowerCase()).filter(Boolean);
}
function timeToMinutes(value: string): number | undefined {
  if (!value) return undefined;
  const [hours, minutes] = value.split(':').map(Number);
  if (!Number.isInteger(hours) || !Number.isInteger(minutes)) return undefined;
  return hours * 60 + minutes;
}

export default function ParentDashboard({ authorized, savePolicy, safetySummary, clearSafetyEvents, lockdownApi }: ParentDashboardProps) {
  const [childAge, setChildAge] = useState(10);
  const [allowDomains, setAllowDomains] = useState('');
  const [blockDomains, setBlockDomains] = useState('');
  const [teenUnknownWebEnabled, setTeenUnknownWebEnabled] = useState(false);
  const [socialService, setSocialService] = useState('');
  const [socialMode, setSocialMode] = useState<SocialAccessMode>('blocked');
  const [socialStart, setSocialStart] = useState('');
  const [socialEnd, setSocialEnd] = useState('');
  const [downloadMode, setDownloadMode] = useState<DownloadMode>('require_parent_high_risk');
  const [pin, setPin] = useState('');
  const [status, setStatus] = useState<string | null>(null);

  if (!authorized) return <section aria-label="Parent controls"><h1>Parent authorization required</h1><p>KidOS keeps parent safety settings locked while the child environment is active.</p></section>;

  async function submit(event: FormEvent) {
    event.preventDefault();
    const socialAccess = socialService.trim() ? [{ service: socialService.trim().toLowerCase(), mode: socialMode, ...(socialMode === 'time_limited' ? { startMinutes: timeToMinutes(socialStart), endMinutes: timeToMinutes(socialEnd) } : {}) }] : [];
    const policy: ParentPolicyConfig = { childAge, allowDomains: splitDomains(allowDomains), blockDomains: splitDomains(blockDomains), teenUnknownWebEnabled: childAge >= 13 && teenUnknownWebEnabled, socialAccess, downloadMode };
    try { await savePolicy(pin, policy); setPin(''); setStatus('Parent settings saved'); }
    catch { setStatus('Parent settings were not saved'); }
  }

  return <section aria-label="Parent controls">
    <h1>Parent safety controls</h1>
    <form onSubmit={submit}>
      <label htmlFor="parent-child-age">Child age</label><input id="parent-child-age" type="number" min={3} max={17} value={childAge} onChange={(e) => { const age=Number(e.target.value); setChildAge(age); if(age<13)setTeenUnknownWebEnabled(false); }} />
      <label htmlFor="parent-allow-domains">Allowed domains</label><textarea id="parent-allow-domains" value={allowDomains} onChange={(e)=>setAllowDomains(e.target.value)} placeholder="khanacademy.org" />
      <label htmlFor="parent-block-domains">Blocked domains</label><textarea id="parent-block-domains" value={blockDomains} onChange={(e)=>setBlockDomains(e.target.value)} placeholder="unsafe.example" />
      <label htmlFor="parent-unknown-web"><input id="parent-unknown-web" type="checkbox" checked={teenUnknownWebEnabled} disabled={childAge<13} onChange={(e)=>setTeenUnknownWebEnabled(e.target.checked)} />Allow unknown websites for teen profile</label>
      <label htmlFor="parent-social-service">Social service</label><input id="parent-social-service" value={socialService} onChange={(e)=>setSocialService(e.target.value)} placeholder="youtube" />
      <label htmlFor="parent-social-access">Social access</label><select id="parent-social-access" value={socialMode} onChange={(e)=>setSocialMode(e.target.value as SocialAccessMode)}><option value="blocked">Blocked</option><option value="allowed">Allowed</option><option value="time_limited">Time limited</option></select>
      {socialMode==='time_limited'?<><label htmlFor="parent-social-start">Social start</label><input id="parent-social-start" type="time" value={socialStart} onChange={(e)=>setSocialStart(e.target.value)} /><label htmlFor="parent-social-end">Social end</label><input id="parent-social-end" type="time" value={socialEnd} onChange={(e)=>setSocialEnd(e.target.value)} /></>:null}
      <label htmlFor="parent-download-mode">Download protection</label><select id="parent-download-mode" value={downloadMode} onChange={(e)=>setDownloadMode(e.target.value as DownloadMode)}><option value="require_parent_high_risk">Require parent for high-risk downloads</option><option value="block_high_risk">Block high-risk downloads</option></select>
      <label htmlFor="parent-pin">Parent PIN</label><input id="parent-pin" type="password" inputMode="numeric" minLength={4} maxLength={8} value={pin} onChange={(e)=>setPin(e.target.value)} autoComplete="off" />
      <button type="submit">Save parent settings</button>
    </form>
    {status?<p role="status">{status}</p>:null}
    {lockdownApi?<LockdownSettings authorized={authorized} api={lockdownApi}/>:null}
    {safetySummary&&clearSafetyEvents?<SafetySummary authorized={authorized} summary={safetySummary} clearEvents={clearSafetyEvents}/>:null}
  </section>;
}
