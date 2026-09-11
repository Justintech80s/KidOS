import { KidOSActionStatus, KidOSModuleHeader, KidOSPromptChips } from './KidOSModulePrimitives';

export interface KidOSCreateScreenProps {
  value: string;
  statusMessage: string;
  onValueChange(value: string): void;
  onSubmit(): void;
  onPrompt(prompt: string): void;
}

const prompts = [
  'Write a space adventure',
  'Make a science presentation',
  'Plan a beginner coding project',
  'Create an animal story',
] as const;

export default function KidOSCreateScreen({ value, statusMessage, onValueChange, onSubmit, onPrompt }: KidOSCreateScreenProps) {
  return (
    <section className="kidos-screen kidos-create-screen" data-testid="kidos-create-screen">
      <KidOSModuleHeader
        icon="🎨"
        eyebrow="Protected creativity"
        title="Create"
        copy="Stories, presentations, drawings, and beginner coding start in a protected KidOS workspace."
      />
      <div className="kidos-creator-card">
        <KidOSPromptChips label="Creation ideas" prompts={prompts} onSelect={onPrompt} />
        <form className="kidos-creator-form" onSubmit={(event) => { event.preventDefault(); onSubmit(); }}>
          <label>
            What would you like to make?
            <textarea
              aria-label="Ask KidOS"
              value={value}
              onChange={(event) => onValueChange(event.target.value)}
              placeholder="Make a space story..."
              rows={4}
            />
          </label>
          <button className="kidos-primary-action" type="submit">Create safely</button>
        </form>
      </div>
      <KidOSActionStatus>{statusMessage}</KidOSActionStatus>
    </section>
  );
}
