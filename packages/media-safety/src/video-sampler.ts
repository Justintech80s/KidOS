import type { MediaRisk } from '@kidos/contracts';

export function nextSampleIntervalMs(currentRisk: MediaRisk): number {
  switch (currentRisk) {
    case 'high':
      return 2_000;
    case 'medium':
      return 5_000;
    case 'low':
      return 10_000;
  }
}
