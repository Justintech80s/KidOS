import type { MediaClassification } from '@kidos/contracts';

function clampScore(score: number): number {
  return Math.min(100, Math.max(0, score));
}

export function updateReputation(
  currentScore: number,
  classification: MediaClassification,
): number {
  let delta = 0;

  if (classification.category === 'uncertain') {
    delta = 5;
  } else if (
    classification.category === 'safe' &&
    classification.confidence >= 0.8 &&
    classification.risk === 'low'
  ) {
    delta = -2;
  } else if (classification.risk === 'high') {
    delta = 15;
  } else if (classification.risk === 'medium') {
    delta = 5;
  }

  return clampScore(currentScore + delta);
}
