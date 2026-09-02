import type {
  CapabilityId,
  ChildProfile,
  WorkspaceKind,
  WorkspacePlan,
} from '@kidos/contracts';

const intents: ReadonlyArray<{
  kind: WorkspaceKind;
  terms: readonly string[];
}> = [
  { kind: 'beginner_coding', terms: ['code', 'coding', 'game', 'program'] },
  {
    kind: 'drawing_presentation',
    terms: ['draw', 'picture', 'poster', 'presentation', 'slides', 'cartoon'],
  },
  { kind: 'story', terms: ['story', 'write', 'book', 'poem', 'script'] },
];

const titles: Record<WorkspaceKind, string> = {
  story: 'Story Workspace',
  drawing_presentation: 'Drawing & Presentation Workspace',
  beginner_coding: 'Beginner Coding Workspace',
};

function detectWorkspaceKind(prompt: string): WorkspaceKind | null {
  const normalizedPrompt = prompt.toLowerCase();

  for (const intent of intents) {
    if (intent.terms.some((term) => normalizedPrompt.includes(term))) {
      return intent.kind;
    }
  }

  return null;
}

function uniqueCapabilities(capabilities: CapabilityId[]): CapabilityId[] {
  return [...new Set(capabilities)];
}

export function planWorkspace(
  prompt: string,
  profile: ChildProfile,
  allowedCapabilities: CapabilityId[],
): WorkspacePlan {
  void profile;

  const matchedKind = detectWorkspaceKind(prompt);
  const kind = matchedKind ?? 'story';

  return {
    kind,
    title: titles[kind],
    capabilities: matchedKind ? uniqueCapabilities(allowedCapabilities) : [],
  };
}
