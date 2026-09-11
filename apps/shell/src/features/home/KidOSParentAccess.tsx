import type { RefObject } from 'react';
import KidOSProfileCard from './KidOSProfileCard';

export interface KidOSParentAccessProps {
  pin: string;
  statusMessage: string;
  inputRef: RefObject<HTMLInputElement | null>;
  onPinChange(pin: string): void;
  onUnlock(): void;
}

const capabilities = ['Approved Apps', 'Activity', 'Time Limits', 'Protection Settings'] as const;

export default function KidOSParentAccess({ pin, statusMessage, inputRef, onPinChange, onUnlock }: KidOSParentAccessProps) {
  return (
    <section className="kidos-screen kidos-parent-screen" data-testid="kidos-parent-screen">
      <div className="kidos-parent-card">
        <span className="kidos-parent-lock" aria-hidden="true">🔒</span>
        <p className="eyebrow">Adults only</p>
        <h1>Parent Access</h1>
        <p>Guardian controls stay locked until a parent is verified.</p>
        <div className="kidos-parent-capabilities" aria-label="Parent controls">
          {capabilities.map((label) => <span key={label}>{label}</span>)}
        </div>
        <label className="kidos-parent-pin-label">
          Parent PIN
          <input
            ref={inputRef}
            aria-label="Parent PIN"
            type="password"
            inputMode="numeric"
            autoComplete="off"
            value={pin}
            onChange={(event) => onPinChange(event.target.value.replace(/\D/g, '').slice(0, 8))}
            placeholder="Enter PIN"
          />
        </label>
        <button className="kidos-primary-action" type="button" onClick={onUnlock}>Unlock Parent Workspace</button>
        {statusMessage && <div className="kidos-action-status" role="status">{statusMessage}</div>}
        <KidOSProfileCard profile={{ displayName: 'Alex', levelLabel: 'Explorer • Level 12' }} />
      </div>
    </section>
  );
}
