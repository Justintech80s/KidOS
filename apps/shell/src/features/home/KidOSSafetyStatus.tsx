import type { KidOSSystemStatus, ProtectionState } from './system-status';

function label(prefix: string, state: ProtectionState) {
  if (state === 'active') return `${prefix}: Active`;
  if (state === 'degraded') return `${prefix}: Degraded`;
  return `${prefix}: Offline`;
}

export default function KidOSSafetyStatus({ status }: { status: KidOSSystemStatus }) {
  const classifierState: ProtectionState = status.classifierReady
    ? 'active'
    : status.classifierRunning
      ? 'degraded'
      : 'offline';

  return (
    <div className="kidos-safety-status" data-testid="kidos-safety-status" aria-label="KidOS protection status">
      <span data-state={status.safeMode}>🛡 {label('Safe Mode', status.safeMode)}</span>
      <span data-state={status.internetFilter}>🌐 {label('Internet Filter', status.internetFilter)}</span>
      <span data-state={classifierState}>
        🤖 Media Safety: {status.classifierReady ? 'Ready' : status.classifierRunning ? 'Starting' : 'Offline'}
      </span>
    </div>
  );
}
