import type { MediaClassification, PolicyDecision } from '@kidos/contracts';
import {
  classifyMedia,
  type ClassifyMediaOptions,
  type MediaInput,
} from '@kidos/media-safety';

export type BrowserMediaState = 'show' | 'obscure' | 'pause';

export type BrowserMediaResult = {
  state: BrowserMediaState;
  decision: PolicyDecision;
  reason: 'media_allowed' | 'media_blocked' | 'parent_approval_required';
};

export type MediaPolicyEvaluator = (
  classification: MediaClassification,
) => PolicyDecision | Promise<PolicyDecision>;

export function mapMediaDecisionToDisplay(
  kind: MediaInput['kind'],
  decision: PolicyDecision,
): BrowserMediaResult {
  if (decision === 'allow') {
    return { state: 'show', decision, reason: 'media_allowed' };
  }

  const state: BrowserMediaState = kind === 'image' ? 'obscure' : 'pause';

  if (decision === 'block') {
    return { state, decision, reason: 'media_blocked' };
  }

  return { state, decision, reason: 'parent_approval_required' };
}

async function evaluateMediaForDisplay(
  kind: MediaInput['kind'],
  bytes: Uint8Array,
  classifierOptions: ClassifyMediaOptions,
  evaluatePolicy: MediaPolicyEvaluator,
): Promise<BrowserMediaResult> {
  const classification = await classifyMedia({ kind, bytes }, classifierOptions);
  const decision = await evaluatePolicy(classification);
  return mapMediaDecisionToDisplay(kind, decision);
}

export function evaluateImageForDisplay(
  bytes: Uint8Array,
  classifierOptions: ClassifyMediaOptions,
  evaluatePolicy: MediaPolicyEvaluator,
): Promise<BrowserMediaResult> {
  return evaluateMediaForDisplay('image', bytes, classifierOptions, evaluatePolicy);
}

export function evaluateVideoFrame(
  bytes: Uint8Array,
  classifierOptions: ClassifyMediaOptions,
  evaluatePolicy: MediaPolicyEvaluator,
): Promise<BrowserMediaResult> {
  return evaluateMediaForDisplay('video_frame', bytes, classifierOptions, evaluatePolicy);
}
