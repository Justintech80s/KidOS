import { describe, expect, it, vi } from 'vitest';
import { classifyMedia } from './classifier';

const fakeInput = {
  kind: 'image' as const,
  bytes: new Uint8Array([1, 2, 3]),
  context: { domain: 'example.test' },
};

describe('classifyMedia', () => {
  it('returns a confident local classification without remote escalation', async () => {
    const remoteClassifier = vi.fn(async () => ({
      category: 'adult_nudity' as const,
      confidence: 0.99,
      source: 'remote' as const,
      risk: 'high' as const,
    }));
    const result = await classifyMedia(fakeInput, {
      remoteEnabled: true,
      localClassifier: async () => ({ category: 'safe', confidence: 0.95, source: 'local', risk: 'low' }),
      remoteClassifier,
    });
    expect(result.source).toBe('local');
    expect(remoteClassifier).not.toHaveBeenCalled();
  });

  it('fails closed when local is uncertain and remote is disabled', async () => {
    const result = await classifyMedia(fakeInput, {
      remoteEnabled: false,
      localClassifier: async () => ({ category: 'uncertain', confidence: 0.4, source: 'local', risk: 'high' }),
    });
    expect(result).toEqual({ category: 'uncertain', confidence: 0.4, source: 'local', risk: 'high' });
  });

  it('escalates an uncertain local result when remote moderation is enabled', async () => {
    const result = await classifyMedia(fakeInput, {
      remoteEnabled: true,
      localClassifier: async () => ({ category: 'uncertain', confidence: 0.45, source: 'local', risk: 'high' }),
      remoteClassifier: async () => ({ category: 'adult_nudity', confidence: 0.97, source: 'remote', risk: 'high' }),
    });
    expect(result.category).toBe('adult_nudity');
    expect(result.source).toBe('remote');
  });

  it('keeps the uncertain local result when remote moderation fails', async () => {
    const localResult = { category: 'uncertain' as const, confidence: 0.35, source: 'local' as const, risk: 'high' as const };
    const result = await classifyMedia(fakeInput, {
      remoteEnabled: true,
      localClassifier: async () => localResult,
      remoteClassifier: async () => { throw new Error('moderation unavailable'); },
    });
    expect(result).toEqual(localResult);
  });

  it('rejects a malformed remote result and keeps the local risk evidence', async () => {
    const localResult = { category: 'uncertain' as const, confidence: 0.3, source: 'local' as const, risk: 'high' as const };
    const result = await classifyMedia(fakeInput, {
      remoteEnabled: true,
      localClassifier: async () => localResult,
      remoteClassifier: async () => ({ category: 'safe', confidence: 9, source: 'remote', risk: 'low' } as never),
    });
    expect(result).toEqual(localResult);
  });
});
