import { z } from 'zod';

export const MediaSafetyCategorySchema = z.enum([
  'safe',
  'adult_nudity',
  'sexualized_content',
  'graphic_violence',
  'self_harm',
  'drugs',
  'extremist_content',
  'scam',
  'uncertain',
]);
export type MediaSafetyCategory = z.infer<typeof MediaSafetyCategorySchema>;

export const ClassificationSourceSchema = z.enum(['local', 'remote']);
export type ClassificationSource = z.infer<typeof ClassificationSourceSchema>;

export const MediaRiskSchema = z.enum(['low', 'medium', 'high']);
export type MediaRisk = z.infer<typeof MediaRiskSchema>;

export const MediaClassificationSchema = z.object({
  category: MediaSafetyCategorySchema,
  confidence: z.number().min(0).max(1),
  source: ClassificationSourceSchema,
  risk: MediaRiskSchema,
});
export type MediaClassification = z.infer<typeof MediaClassificationSchema>;
