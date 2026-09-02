import { describe, expect, it } from 'vitest';
import {
  evaluateImageForDisplay,
  evaluateVideoFrame,
  mapMediaDecisionToDisplay,
} from './media-safety';

const classifierOptions = {
  remoteEnabled: false,
  localClassifier: async () => ({
    category: 'adult_nudity' as const,
    confidence: 0.99,
    source: 'local' as const,
    risk: 'high' as const,
  }),
};

describe('mapMediaDecisionToDisplay', () => {
  it('shows allowed images', () => {
    expect(mapMediaDecisionToDisplay('image', 'allow')).toEqual({
      state: 'show',
      decision: 'allow',
      reason: 'media_allowed',
    });
  });

  it('obscures parent-gated images', () => {
    expect(mapMediaDecisionToDisplay('image', 'require_parent')).toEqual({
      state: 'obscure',
      decision: 'require_parent',
      reason: 'parent_approval_required',
    });
  });

  it('obscures blocked images', () => {
    expect(mapMediaDecisionToDisplay('image', 'block')).toEqual({
      state: 'obscure',
      decision: 'block',
      reason: 'media_blocked',
    });
  });

  it('pauses blocked and parent-gated video frames', () => {
    expect(mapMediaDecisionToDisplay('video_frame', 'block').state).toBe('pause');
    expect(mapMediaDecisionToDisplay('video_frame', 'require_parent').state).toBe('pause');
  });
});

describe('browser media evaluation', () => {
  it('classifies an image before applying the policy decision', async () => {
    const result = await evaluateImageForDisplay(
      new Uint8Array([1, 2, 3]),
      classifierOptions,
      (classification) => classification.category === 'adult_nudity' ? 'block' : 'allow',
    );
    expect(result).toEqual({ state: 'obscure', decision: 'block', reason: 'media_blocked' });
  });

  it('pauses video when policy requires a parent', async () => {
    const result = await evaluateVideoFrame(
      new Uint8Array([4, 5, 6]),
      classifierOptions,
      () => 'require_parent',
    );
    expect(result).toEqual({
      state: 'pause',
      decision: 'require_parent',
      reason: 'parent_approval_required',
    });
  });
});
