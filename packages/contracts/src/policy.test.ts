import { describe, expect, it } from 'vitest';
import { ChildProfileSchema, PolicyDecisionSchema } from './policy';

describe('policy contracts', () => {
  it('rejects policy outcomes outside the three supported decisions', () => {
    expect(PolicyDecisionSchema.safeParse('skip').success).toBe(false);
  });

  it('requires a child age from 3 through 17', () => {
    expect(ChildProfileSchema.safeParse({ id: 'kid-1', displayName: 'Ari', age: 10 }).success).toBe(true);
    expect(ChildProfileSchema.safeParse({ id: 'kid-2', displayName: 'Ari', age: 18 }).success).toBe(false);
  });
});
