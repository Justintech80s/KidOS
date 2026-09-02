import { describe, expect, it } from 'vitest';
import { MediaClassificationSchema } from './media-safety';

describe('media safety contracts', () => {
  it('accepts a valid local safe result', () => {
    expect(
      MediaClassificationSchema.safeParse({
        category: 'safe',
        confidence: 0.98,
        source: 'local',
        risk: 'low',
      }).success,
    ).toBe(true);
  });

  it('accepts every supported unsafe or uncertain category', () => {
    const categories = [
      'adult_nudity',
      'sexualized_content',
      'graphic_violence',
      'self_harm',
      'drugs',
      'extremist_content',
      'scam',
      'uncertain',
    ] as const;

    for (const category of categories) {
      expect(
        MediaClassificationSchema.safeParse({
          category,
          confidence: 0.85,
          source: 'local',
          risk: 'high',
        }).success,
      ).toBe(true);
    }
  });

  it('rejects invalid confidence values', () => {
    expect(
      MediaClassificationSchema.safeParse({
        category: 'safe',
        confidence: 1.2,
        source: 'local',
        risk: 'low',
      }).success,
    ).toBe(false);
  });

  it('rejects unknown categories and sources', () => {
    expect(
      MediaClassificationSchema.safeParse({
        category: 'unknown-new-category',
        confidence: 0.8,
        source: 'device-magic',
        risk: 'high',
      }).success,
    ).toBe(false);
  });
});
