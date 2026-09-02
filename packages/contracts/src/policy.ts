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
  archiveContainsHighRisk: z.boolean().default(false),
});
export type DownloadRequest = z.infer<typeof DownloadRequestSchema>;

export const DownloadResultSchema = z.object({
  decision: PolicyDecisionSchema,
  reason: z.string().min(1),
});
export type DownloadResult = z.infer<typeof DownloadResultSchema>;

export const DownloadModeSchema = z.enum(['block_high_risk', 'require_parent_high_risk']);
export type DownloadMode = z.infer<typeof DownloadModeSchema>;

export const SocialAccessModeSchema = z.enum(['blocked', 'allowed', 'time_limited']);
export type SocialAccessMode = z.infer<typeof SocialAccessModeSchema>;

export const SocialAccessRuleSchema = z
  .object({
    service: z.string().trim().min(1).max(80),
    mode: SocialAccessModeSchema,
    startMinutes: z.number().int().min(0).max(1439).optional(),
    endMinutes: z.number().int().min(1).max(1440).optional(),
  })
  .superRefine((rule, context) => {
    if (rule.mode !== 'time_limited') return;

    if (rule.startMinutes === undefined || rule.endMinutes === undefined) {
      context.addIssue({
        code: 'custom',
        message: 'Time-limited social access requires a start and end time.',
      });
      return;
    }

    if (rule.startMinutes >= rule.endMinutes) {
      context.addIssue({
        code: 'custom',
        message: 'Social access start time must be before the end time.',
      });
    }
  });
export type SocialAccessRule = z.infer<typeof SocialAccessRuleSchema>;

const DomainRuleSchema = z
  .string()
  .trim()
  .min(1)
  .max(253)
  .regex(/^(?:[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.)+[a-z]{2,63}$/i);

export const ParentPolicyConfigSchema = z
  .object({
    childAge: z.number().int().min(3).max(17),
    allowDomains: z.array(DomainRuleSchema).max(100),
    blockDomains: z.array(DomainRuleSchema).max(100),
    teenUnknownWebEnabled: z.boolean(),
    socialAccess: z.array(SocialAccessRuleSchema).max(50),
    downloadMode: DownloadModeSchema,
  })
  .superRefine((policy, context) => {
    if (policy.childAge < 13 && policy.teenUnknownWebEnabled) {
      context.addIssue({
        code: 'custom',
        path: ['teenUnknownWebEnabled'],
        message: 'Unknown-web access is available only for teen profiles.',
      });
    }
  });
export type ParentPolicyConfig = z.infer<typeof ParentPolicyConfigSchema>;
