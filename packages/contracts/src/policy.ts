import { z } from 'zod';

export const PolicyDecisionSchema = z.enum(['allow', 'block', 'require_parent']);
export type PolicyDecision = z.infer<typeof PolicyDecisionSchema>;

export const ChildProfileSchema = z.object({
  id: z.string().min(1),
  displayName: z.string().min(1).max(40),
  age: z.number().int().min(3).max(17),
});
export type ChildProfile = z.infer<typeof ChildProfileSchema>;

export const NavigationRequestSchema = z.object({
  profile: ChildProfileSchema,
  url: z.url(),
  isRedirect: z.boolean().default(false),
});
export type NavigationRequest = z.infer<typeof NavigationRequestSchema>;

export const NavigationResultSchema = z.object({
  decision: PolicyDecisionSchema,
  reason: z.string().min(1),
});
export type NavigationResult = z.infer<typeof NavigationResultSchema>;

export const DownloadRequestSchema = z.object({
  profile: ChildProfileSchema,
  url: z.url(),
  fileName: z.string().min(1),
  mimeType: z.string().min(1).optional(),
});
export type DownloadRequest = z.infer<typeof DownloadRequestSchema>;

export const DownloadResultSchema = z.object({
  decision: PolicyDecisionSchema,
  reason: z.string().min(1),
});
export type DownloadResult = z.infer<typeof DownloadResultSchema>;
