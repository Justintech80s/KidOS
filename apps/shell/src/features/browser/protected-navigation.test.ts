import { describe, expect, it } from 'vitest';
import {
  prepareProtectedNavigation,
  prepareProtectedRedirect,
  type NavigationEvaluator,
} from './protected-navigation';

const allow: NavigationEvaluator = async () => 'allow';

function decision(value: 'allow' | 'block' | 'require_parent'): NavigationEvaluator {
  return async () => value;
}

describe('KidOS protected navigation', () => {
  it('forces Google SafeSearch while preserving the search query', async () => {
    const result = await prepareProtectedNavigation(
      'https://www.google.com/search?q=solar+system',
      allow,
    );

    expect(result.state).toBe('load');
    if (result.state !== 'load') throw new Error('expected loadable navigation');
    expect(result.url).toContain('q=solar+system');
    expect(result.url).toContain('safe=active');
  });

  it('forces Bing strict adult filtering', async () => {
    const result = await prepareProtectedNavigation(
      'https://www.bing.com/search?q=dinosaurs',
      allow,
    );

    expect(result.state).toBe('load');
    if (result.state !== 'load') throw new Error('expected loadable navigation');
    expect(result.url).toContain('adlt=strict');
  });

  it('requires strict YouTube restriction metadata for YouTube destinations', async () => {
    const result = await prepareProtectedNavigation('https://www.youtube.com/watch?v=abc', allow);

    expect(result.state).toBe('load');
    if (result.state !== 'load') throw new Error('expected loadable navigation');
    expect(result.requestHeaders).toEqual({ 'YouTube-Restrict': 'Strict' });
  });

  it('never loads a blocked destination', async () => {
    const result = await prepareProtectedNavigation('https://blocked.example', decision('block'));

    expect(result).toMatchObject({ state: 'blocked', decision: 'block' });
    expect(result.url).toBeUndefined();
  });

  it('never loads a destination requiring parent approval', async () => {
    const result = await prepareProtectedNavigation(
      'https://unknown.example',
      decision('require_parent'),
    );

    expect(result).toMatchObject({ state: 'parent_gate', decision: 'require_parent' });
    expect(result.url).toBeUndefined();
  });

  it('re-evaluates every redirect before allowing the redirected destination', async () => {
    const seen: string[] = [];
    const evaluator: NavigationEvaluator = async (url) => {
      seen.push(url);
      return url.includes('blocked.example') ? 'block' : 'allow';
    };

    const initial = await prepareProtectedNavigation('https://safe.example', evaluator);
    expect(initial.state).toBe('load');

    const redirected = await prepareProtectedRedirect('https://blocked.example/landing', evaluator);
    expect(redirected.state).toBe('blocked');
    expect(seen).toEqual(['https://safe.example/', 'https://blocked.example/landing']);
  });

  it('fails closed for malformed or unsupported URLs', async () => {
    const result = await prepareProtectedNavigation('javascript:alert(1)', allow);

    expect(result).toMatchObject({ state: 'blocked', decision: 'block' });
  });
});
