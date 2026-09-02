import { describe, expect, it } from 'vitest';
import { mapMediaDecisionToDisplay } from './media-safety';

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
