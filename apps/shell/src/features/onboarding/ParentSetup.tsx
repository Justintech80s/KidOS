import { FormEvent, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

type ConfigureParentPin = (pin: string) => Promise<void>;

async function configureThroughTauri(pin: string): Promise<void> {
  await invoke('configure_parent_pin', { pin });
}

interface ParentSetupProps {
  configureParentPin?: ConfigureParentPin;
  onConfigured?: () => void;
}

export default function ParentSetup({
  configureParentPin = configureThroughTauri,
  onConfigured,
}: ParentSetupProps) {
  const [pin, setPin] = useState('');
  const [confirmation, setConfirmation] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setError(null);

    if (!/^\d{4,8}$/.test(pin)) {
      setError('Use 4 to 8 digits for the parent PIN.');
      return;
    }

    if (pin !== confirmation) {
      setError('The PIN entries must match.');
      return;
    }

    setSaving(true);
    try {
      await configureParentPin(pin);
      setPin('');
      setConfirmation('');
      onConfigured?.();
    } catch {
      setError('KidOS could not protect the parent PIN. Try again.');
    } finally {
      setSaving(false);
    }
  }

  return (
    <section aria-labelledby="parent-setup-title">
      <p>Parent setup</p>
      <h1 id="parent-setup-title">Create your KidOS parent PIN</h1>
      <p>This PIN protects parent-only settings and approvals.</p>

      <form onSubmit={handleSubmit}>
        <label>
          Parent PIN
          <input
            aria-label="Parent PIN"
            autoComplete="new-password"
            inputMode="numeric"
            maxLength={8}
            type="password"
            value={pin}
            onChange={(event) => setPin(event.target.value)}
          />
        </label>

        <label>
          Confirm parent PIN
          <input
            aria-label="Confirm parent PIN"
            autoComplete="new-password"
            inputMode="numeric"
            maxLength={8}
            type="password"
            value={confirmation}
            onChange={(event) => setConfirmation(event.target.value)}
          />
        </label>

        {error ? <p role="alert">{error}</p> : null}

        <button type="submit" disabled={saving}>
          {saving ? 'Protecting…' : 'Protect KidOS'}
        </button>
      </form>
    </section>
  );
}
