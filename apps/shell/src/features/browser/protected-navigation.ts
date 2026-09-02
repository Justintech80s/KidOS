import type { PolicyDecision } from '../../lib/kidos-api';

export type NavigationEvaluator = (url: string) => Promise<PolicyDecision>;

export type ProtectedNavigationResult =
  | {
      state: 'load';
      decision: 'allow';
      url: string;
      requestHeaders?: Record<string, string>;
    }
  | {
      state: 'blocked';
      decision: 'block';
      reason: string;
      url?: undefined;
    }
  | {
      state: 'parent_gate';
      decision: 'require_parent';
      reason: string;
      url?: undefined;
    };

function isDomain(hostname: string, domain: string) {
  return hostname === domain || hostname.endsWith(`.${domain}`);
}

function normalizeDestination(destination: string): URL | null {
  try {
    const url = new URL(destination);
    if (url.protocol !== 'https:' && url.protocol !== 'http:') return null;
    return url;
  } catch {
    return null;
  }
}

function applySaferServiceSettings(url: URL) {
  if (isDomain(url.hostname, 'google.com')) {
    url.searchParams.set('safe', 'active');
  }

  if (isDomain(url.hostname, 'bing.com')) {
    url.searchParams.set('adlt', 'strict');
  }

  const requestHeaders =
    isDomain(url.hostname, 'youtube.com') || isDomain(url.hostname, 'youtu.be')
      ? { 'YouTube-Restrict': 'Strict' }
      : undefined;

  return { url: url.toString(), requestHeaders };
}

export async function prepareProtectedNavigation(
  destination: string,
  evaluateNavigation: NavigationEvaluator,
): Promise<ProtectedNavigationResult> {
  const normalized = normalizeDestination(destination);
  if (!normalized) {
    return {
      state: 'blocked',
      decision: 'block',
      reason: 'Unsupported or malformed web address',
    };
  }

  const checkedUrl = normalized.toString();
  const decision = await evaluateNavigation(checkedUrl);

  if (decision === 'block') {
    return { state: 'blocked', decision, reason: 'Blocked by KidOS policy' };
  }

  if (decision === 'require_parent') {
    return {
      state: 'parent_gate',
      decision,
      reason: 'Parent approval required',
    };
  }

  const safer = applySaferServiceSettings(normalized);
  return {
    state: 'load',
    decision: 'allow',
    url: safer.url,
    requestHeaders: safer.requestHeaders,
  };
}

export function prepareProtectedRedirect(
  destination: string,
  evaluateNavigation: NavigationEvaluator,
) {
  return prepareProtectedNavigation(destination, evaluateNavigation);
}
