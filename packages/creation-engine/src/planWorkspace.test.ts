import { describe, expect, it } from 'vitest';
import type { ChildProfile } from '@kidos/contracts';
import { planWorkspace } from './planWorkspace';

const profile: ChildProfile = {
  id: 'kid-1',
  displayName: 'Ari',
  age: 10,
};

describe('planWorkspace', () => {
  it('maps a story request to the story workspace', () => {
    const plan = planWorkspace('write a superhero story', profile, ['story', 'export_project']);
    expect(plan.kind).toBe('story');
    expect(plan.capabilities).toEqual(['story', 'export_project']);
  });

  it('maps a coding request to beginner coding', () => {
    const plan = planWorkspace('make a coding game', profile, ['beginner_coding', 'export_project']);
    expect(plan.kind).toBe('beginner_coding');
  });

  it('maps a drawing request to drawing and presentation', () => {
    const plan = planWorkspace('draw a cartoon poster', profile, ['drawing_presentation']);
    expect(plan.kind).toBe('drawing_presentation');
  });

  it('never adds a capability excluded by policy', () => {
    const plan = planWorkspace('make a coding game', profile, ['story']);
    expect(plan.capabilities).toEqual(['story']);
  });

  it('defaults unknown requests to a story workspace without elevated capabilities', () => {
    const plan = planWorkspace('make something cool', profile, ['protected_web', 'audio_recording']);
    expect(plan.kind).toBe('story');
    expect(plan.capabilities).toEqual([]);
  });
});
