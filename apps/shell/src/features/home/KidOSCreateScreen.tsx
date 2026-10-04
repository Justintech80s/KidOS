import type { WorkspacePlan } from '@kidos/contracts';
import type { WorkspaceDocument } from '../../lib/kidos-api';
import { KidOSActionStatus, KidOSModuleHeader, KidOSPromptChips } from './KidOSModulePrimitives';

export interface KidOSCreateScreenProps {
  value: string;
  statusMessage: string;
  workspace?: WorkspacePlan;
  projectId?: string;
  projectTitle: string;
  projectContent: string;
  savedProjects: WorkspaceDocument[];
  onValueChange(value: string): void;
  onSubmit(): void;
  onPrompt(prompt: string): void;
  onProjectTitleChange(value: string): void;
  onProjectContentChange(value: string): void;
  onSaveProject(): void;
  onOpenProject(project: WorkspaceDocument): void;
}

const prompts = [
  'Write a space adventure',
  'Make a science presentation',
  'Plan a beginner coding project',
  'Create an animal story',
] as const;

const workspaceHelp: Record<WorkspacePlan['kind'], string> = {
  story: 'Write and revise your story in the protected KidOS editor.',
  drawing_presentation: 'Build a presentation or visual-project outline with one idea per section.',
  beginner_coding: 'Practice beginner code, pseudocode, and project notes without unrestricted system access.',
};

export default function KidOSCreateScreen({
  value,
  statusMessage,
  workspace,
  projectId,
  projectTitle,
  projectContent,
  savedProjects,
  onValueChange,
  onSubmit,
  onPrompt,
  onProjectTitleChange,
  onProjectContentChange,
  onSaveProject,
  onOpenProject,
}: KidOSCreateScreenProps) {
  return (
    <section className="kidos-screen kidos-create-screen" data-testid="kidos-create-screen">
      <KidOSModuleHeader
        icon="🎨"
        eyebrow="Protected creativity"
        title="Create"
        copy="Start a KidOS project, work inside the protected editor, save it locally, and reopen it later."
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
              rows={3}
            />
          </label>
          <button className="kidos-primary-action" type="submit">Create safely</button>
        </form>

        {workspace ? (
          <section className="kidos-workspace-editor" data-testid="kidos-workspace-editor">
            <div className="kidos-workspace-heading">
              <div>
                <p className="eyebrow">{workspace.kind.replaceAll('_', ' ')}</p>
                <h2>{workspace.title}</h2>
                <p>{workspaceHelp[workspace.kind]}</p>
              </div>
              {projectId ? <span className="kidos-project-saved-badge">Saved project</span> : null}
            </div>

            <label>
              Project title
              <input
                aria-label="Project title"
                value={projectTitle}
                maxLength={80}
                onChange={(event) => onProjectTitleChange(event.target.value)}
              />
            </label>

            <label>
              Project content
              <textarea
                aria-label="Project content"
                className={workspace.kind === 'beginner_coding' ? 'kidos-code-editor' : undefined}
                value={projectContent}
                onChange={(event) => onProjectContentChange(event.target.value)}
                rows={12}
              />
            </label>

            <button className="kidos-primary-action" type="button" onClick={onSaveProject}>Save project</button>
          </section>
        ) : null}

        <section className="kidos-recent-projects" aria-label="Recent KidOS projects">
          <h2>Recent Projects</h2>
          {savedProjects.length === 0 ? (
            <p>No saved projects yet. Create one above and KidOS will keep it on this computer.</p>
          ) : (
            <div className="kidos-recent-project-grid">
              {savedProjects.slice(0, 8).map((project) => (
                <button type="button" key={project.id ?? project.title} onClick={() => onOpenProject(project)}>
                  <strong>{project.title}</strong>
                  <small>{project.kind.replaceAll('_', ' ')}</small>
                </button>
              ))}
            </div>
          )}
        </section>
      </div>

      <KidOSActionStatus>{statusMessage}</KidOSActionStatus>
    </section>
  );
}
