import { z } from 'zod';

export const CapabilityIdSchema = z.enum([
  'story',
  'drawing_presentation',
  'beginner_coding',
  'protected_web',
  'audio_recording',
  'export_project',
]);
export type CapabilityId = z.infer<typeof CapabilityIdSchema>;

export const WorkspaceKindSchema = z.enum([
  'story',
  'drawing_presentation',
  'beginner_coding',
]);
export type WorkspaceKind = z.infer<typeof WorkspaceKindSchema>;

export const WorkspacePlanSchema = z.object({
  kind: WorkspaceKindSchema,
  title: z.string().min(1).max(80),
  capabilities: z.array(CapabilityIdSchema),
});
export type WorkspacePlan = z.infer<typeof WorkspacePlanSchema>;
