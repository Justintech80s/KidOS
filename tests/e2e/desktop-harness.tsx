import { useMemo, useRef, useState } from 'react';
import type { ParentPolicyConfig } from '@kidos/contracts';
import { planWorkspace } from '@kidos/creation-engine';
import ChildHome from '../../apps/shell/src/features/home/ChildHome';
import ParentDashboard from '../../apps/shell/src/features/parent/ParentDashboard';
import type { KidOSApi, PolicyDecision } from '../../apps/shell/src/lib/kidos-api';

const defaultPolicy: ParentPolicyConfig = {
  childAge: 10,
  allowDomains: [],
  blockDomains: ['blocked.example'],
  teenUnknownWebEnabled: false,
  socialAccess: [],
  downloadMode: 'block_high_risk',
};

function isHighRiskDownload(fileName: string) {
  const lower = fileName.toLowerCase();
  return /\.(exe|msi|bat|cmd|ps1|scr|com|js|vbs)$/.test(lower) ||
    /\.(exe|msi|bat|cmd|ps1|scr|com|js|vbs)\.(zip|rar|7z)$/.test(lower);
}

export function KidOSDesktopHarness({ startInChildMode = false }: { startInChildMode?: boolean }) {
  const [policy, setPolicy] = useState<ParentPolicyConfig>(defaultPolicy);
  const [saved, setSaved] = useState(startInChildMode);
  const [childMode, setChildMode] = useState(startInChildMode);
  const [pendingUrl, setPendingUrl] = useState<string | null>(null);
  const oneTimeApprovals = useRef(new Set<string>());
  const [downloadFile, setDownloadFile] = useState('');
  const [downloadState, setDownloadState] = useState<string | null>(null);

  const api = useMemo<KidOSApi>(() => ({
    async planWorkspace(prompt) {
      return planWorkspace(
        prompt,
        { id: 'kid-e2e', displayName: 'Kid', age: policy.childAge },
        ['story', 'drawing_presentation', 'beginner_coding', 'protected_web', 'export_project'],
      );
    },
    async evaluateNavigation(url) {
      const parsed = new URL(url);
      const hostname = parsed.hostname.toLowerCase();
      if (policy.blockDomains.some((domain) => hostname === domain || hostname.endsWith(`.${domain}`))) {
        return 'block';
      }
      if (oneTimeApprovals.current.delete(url)) {
        return 'allow';
      }
      if (hostname === 'unknown.example') {
        setPendingUrl(url);
        return 'require_parent';
      }
      return 'allow';
    },
    async evaluateDownload(fileName) {
      if (!isHighRiskDownload(fileName)) return 'allow';
      return policy.downloadMode === 'block_high_risk' ? 'block' : 'require_parent';
    },
    async guardianStatus() {
      return 'healthy';
    },
  }), [policy]);

  async function savePolicy(pin: string, nextPolicy: ParentPolicyConfig) {
    if (pin !== '2468') throw new Error('authorization denied');
    setPolicy(nextPolicy);
    setSaved(true);
  }

  async function testDownload() {
    const decision: PolicyDecision = await api.evaluateDownload(
      downloadFile,
      'application/octet-stream',
    );
    setDownloadState(
      decision === 'block'
        ? 'Download blocked by KidOS'
        : decision === 'require_parent'
          ? 'Parent approval required for download'
          : 'Download allowed',
    );
  }

  if (!childMode) {
    return (
      <main>
        <ParentDashboard authorized savePolicy={savePolicy} />
        {saved ? (
          <button type="button" onClick={() => setChildMode(true)}>
            Enter child mode
          </button>
        ) : null}
      </main>
    );
  }

  return (
    <>
      <ChildHome api={api} />
      {pendingUrl ? (
        <button
          type="button"
          onClick={() => {
            oneTimeApprovals.current.add(pendingUrl);
            setPendingUrl(null);
          }}
        >
          Approve once
        </button>
      ) : null}
      <section aria-label="Protected download test">
        <label htmlFor="e2e-download-file">Test download file</label>
        <input
          id="e2e-download-file"
          value={downloadFile}
          onChange={(event) => setDownloadFile(event.target.value)}
        />
        <button type="button" onClick={testDownload}>Test download</button>
        {downloadState ? <p role="status">{downloadState}</p> : null}
      </section>
    </>
  );
}
