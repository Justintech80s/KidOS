import { describe, expect, it } from 'vitest';
import { nextSampleIntervalMs } from './video-sampler';

describe('nextSampleIntervalMs', () => {
  it('samples low-risk video every 10 seconds', () => {
    expect(nextSampleIntervalMs('low')).toBe(10_000);
  });

  it('samples medium-risk video every 5 seconds', () => {
    expect(nextSampleIntervalMs('medium')).toBe(5_000);
  });

  it('samples high-risk video every 2 seconds', () => {
    expect(nextSampleIntervalMs('high')).toBe(2_000);
  });
});
