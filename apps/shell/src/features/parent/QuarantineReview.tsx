import { useState } from 'react';
import type { KidOSApi, QuarantineAction, QuarantineItem } from '../../lib/kidos-api';

type Props = {
  authorized: boolean;
  api: Pick<KidOSApi, 'listQuarantineMedia' | 'previewQuarantineMedia' | 'reviewQuarantineMedia'>;
};

function sizeLabel(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export default function QuarantineReview({ authorized, api }: Props) {
  const [pin, setPin] = useState('');
  const [items, setItems] = useState<QuarantineItem[]>([]);
  const [message, setMessage] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [preview, setPreview] = useState<{ itemId: string; src: string } | null>(null);

  if (!authorized || !api.listQuarantineMedia || !api.reviewQuarantineMedia) return null;

  async function load() {
    if (!pin.trim()) {
      setMessage('Enter the parent PIN to review quarantined media.');
      return;
    }
    setBusy(true);
    try {
      setItems(await api.listQuarantineMedia!(pin.trim()));
      setMessage('Protected quarantine loaded.');
    } catch {
      setItems([]);
      setMessage('KidOS could not unlock the quarantine with that PIN.');
    } finally {
      setBusy(false);
    }
  }


  async function showPreview(item: QuarantineItem) {
    if (!api.previewQuarantineMedia) {
      setMessage('Secure preview is unavailable in this build.');
      return;
    }
    if (!pin.trim()) {
      setMessage('Enter the parent PIN before previewing quarantined media.');
      return;
    }
    setBusy(true);
    try {
      const result = await api.previewQuarantineMedia(pin.trim(), item.id);
      setPreview({
        itemId: item.id,
        src: `data:${result.mimeType};base64,${result.dataBase64}`,
      });
      setMessage('Parent-only preview loaded.');
    } catch {
      setPreview(null);
      setMessage('KidOS could not generate a secure preview for this item.');
    } finally {
      setBusy(false);
    }
  }

  async function act(item: QuarantineItem, action: QuarantineAction) {
    if (!pin.trim()) {
      setMessage('Enter the parent PIN again before changing quarantined media.');
      return;
    }
    setBusy(true);
    try {
      await api.reviewQuarantineMedia!(pin.trim(), item.id, action);
      setItems((current) => current.filter((entry) => entry.id !== item.id));
      setMessage(
        action === 'approve'
          ? `${item.fileName} approved by parent.`
          : action === 'delete'
            ? `${item.fileName} permanently deleted.`
            : `${item.fileName} kept blocked.`,
      );
    } catch {
      setMessage('KidOS did not apply that quarantine action.');
    } finally {
      setBusy(false);
    }
  }

  return (
    <section aria-label="Media quarantine review">
      <h2>Media quarantine</h2>
      <p>Unsafe or uncertain images and videos stay protected until a parent decides what to do.</p>
      <label htmlFor="quarantine-parent-pin">Parent PIN</label>
      <input
        id="quarantine-parent-pin"
        type="password"
        inputMode="numeric"
        minLength={4}
        maxLength={8}
        value={pin}
        onChange={(event) => setPin(event.target.value.replace(/\D/g, '').slice(0, 8))}
        autoComplete="off"
      />
      <button type="button" disabled={busy} onClick={load}>Review quarantined media</button>

      {items.length === 0 ? <p>No quarantine items are currently displayed.</p> : (
        <ul aria-label="Quarantined media">
          {items.map((item) => (
            <li key={item.id}>
              <strong>{item.fileName}</strong>
              <span> {sizeLabel(item.sizeBytes)}</span>
              {item.category ? <p>Category: <strong>{item.category.replaceAll('_', ' ')}</strong></p> : null}
              {item.risk ? <p>Risk: <strong>{item.risk}</strong></p> : null}
              {typeof item.confidence === 'number' ? <p>AI confidence: <strong>{Math.round(item.confidence * 100)}%</strong></p> : null}
              {item.reason ? <p>{item.reason}</p> : null}
              {preview?.itemId === item.id ? (
                <figure>
                  <img src={preview.src} alt={`Parent-only preview of ${item.fileName}`} />
                  <figcaption>Preview is generated inside KidOS and requires parent PIN authorization.</figcaption>
                </figure>
              ) : null}
              <div>
                <button type="button" disabled={busy} onClick={() => showPreview(item)}>Preview</button>
                <button type="button" disabled={busy} onClick={() => act(item, 'approve')}>Approve</button>
                <button type="button" disabled={busy} onClick={() => act(item, 'keep_blocked')}>Keep blocked</button>
                <button type="button" disabled={busy} onClick={() => act(item, 'delete')}>Delete</button>
              </div>
            </li>
          ))}
        </ul>
      )}
      {message ? <p role="status">{message}</p> : null}
    </section>
  );
}
