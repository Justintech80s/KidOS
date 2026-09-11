import { KidOSModuleHeader, KidOSPromptChips } from './KidOSModulePrimitives';

export interface KidOSAiScreenProps {
  value: string;
  answer: string;
  onValueChange(value: string): void;
  onSubmit(): void;
  onSuggestion(prompt: string): void;
}

const suggestions = [
  'Tell me about space',
  'Help me with math',
  'Explain why the sky is blue',
  'Give me a science question',
] as const;

export default function KidOSAiScreen({ value, answer, onValueChange, onSubmit, onSuggestion }: KidOSAiScreenProps) {
  return (
    <section className="kidos-screen kidos-ai-screen" data-testid="kidos-ai-screen">
      <KidOSModuleHeader
        icon="🤖"
        eyebrow="Ask safely"
        title="KidOS AI"
        copy="Ask school-safe questions and explore ideas inside the active KidOS safety rules."
      />
      <div className="kidos-ai-layout">
        <article className="kidos-ai-answer" aria-live="polite">
          <div className="kidos-ai-avatar" aria-hidden="true">✦</div>
          <div><strong>KidOS AI</strong><p>{answer}</p></div>
        </article>
        <KidOSPromptChips label="Suggested KidOS AI questions" prompts={suggestions} onSelect={onSuggestion} />
        <form className="kidos-ai-form" onSubmit={(event) => { event.preventDefault(); onSubmit(); }}>
          <input aria-label="Ask KidOS AI" value={value} onChange={(event) => onValueChange(event.target.value)} placeholder="Why is the sky blue?" />
          <button className="kidos-primary-action" type="submit">Ask safely</button>
        </form>
      </div>
      <p className="kidos-safety-note">🛡 KidOS AI stays inside this profile's active safety rules.</p>
    </section>
  );
}
