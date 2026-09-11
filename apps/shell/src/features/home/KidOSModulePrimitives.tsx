import type { ReactNode } from 'react';

export function KidOSModuleHeader({ icon, eyebrow, title, copy }: { icon: string; eyebrow: string; title: string; copy: string }) {
  return (
    <header className="kidos-screen-header">
      <span className="kidos-screen-icon" aria-hidden="true">{icon}</span>
      <div><p className="eyebrow">{eyebrow}</p><h1>{title}</h1><p>{copy}</p></div>
    </header>
  );
}

export function KidOSActionStatus({ children }: { children?: ReactNode }) {
  if (!children) return null;
  return <div className="kidos-action-status" role="status">{children}</div>;
}

export function KidOSPromptChips({ label, prompts, onSelect }: { label: string; prompts: readonly string[]; onSelect(prompt: string): void }) {
  return (
    <div className="kidos-prompt-chips" aria-label={label}>
      {prompts.map((prompt) => <button type="button" key={prompt} onClick={() => onSelect(prompt)}>{prompt}</button>)}
    </div>
  );
}
