import { describe, expect, it } from 'vitest';
import type { MediaClassification } from '@kidos/contracts';
import { updateReputation } from './reputation';

const classification = (
  overrides: Partial<MediaClassification>,
): MediaClassification => ({
  category: 'safe',
  confidence: 0.95,
  source: 'local',
  risk: 'low',
  ...overrides,
});

describe('updateReputation', () => {
  it('reduces risk score for high-confidence safe media', () => {
    expect(updateReputation(50, classification({}))).toBe(48);
  });

  it('adds 5 points for medium-risk media', () => {
    expect(updateReputation(50, classification({ risk: 'medium' }))).toBe(55);
  });

  it('adds 15 points for high-risk media', () => {
    expect(updateReputation(50, classification({ category: 'adult_nudity', risk: 'high' }))).toBe(65);
  });

  it('adds 5 points for uncertain media without double-counting its risk', () => {
    expect(updateReputation(50, classification({ category: 'uncertain', confidence: 0.4, risk: 'high' }))).toBe(55);
  });

  it('clamps reputation score between 0 and 100', () => {
    expect(updateReputation(1, classification({}))).toBe(0);
    expect(updateReputation(95, classification({ category: 'adult_nudity', risk: 'high' }))).toBe(100);
  });
});
