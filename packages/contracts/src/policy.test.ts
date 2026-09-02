import { describe, expect, it } from 'vitest';
import {
  ChildProfileSchema,
  DownloadRequestSchema,
  ParentPolicyConfigSchema,
  PolicyDecisionSchema,
} from './policy';

describe('policy contracts', () => {
  it('rejects policy outcomes outside the three supported decisions', () => {
    expect(PolicyDecisionSchema.safeParse('skip').success).toBe(false);
  });

  it('requires a child age from 3 through 17', () => {
    expect(ChildProfileSchema.safeParse({ id: 'kid-1', displayName: 'Ari', age: 10 }).success).toBe(true);
    expect(ChildProfileSchema.safeParse({ id: 'kid-2', displayName: 'Ari', age: 18 }).success).toBe(false);
  });

  it('accepts a parent policy with domains, social controls, and high-risk parent gating', () => {
    const result = ParentPolicyConfigSchema.safeParse({
      childAge: 14,
      allowDomains: ['khanacademy.org'],
      blockDomains: ['unsafe.example'],
      teenUnknownWebEnabled: true,
      socialAccess: [
        {
          service: 'youtube',
          mode: 'time_limited',
          startMinutes: 480,
          endMinutes: 1200,
        },
      ],
      downloadMode: 'require_parent_high_risk',
    });

    expect(result.success).toBe(true);
  });

  it('accepts a parent policy that blocks high-risk downloads', () => {
    const result = ParentPolicyConfigSchema.safeParse({
      childAge: 10,
      allowDomains: [],
      blockDomains: [],
      teenUnknownWebEnabled: false,
      socialAccess: [],
      downloadMode: 'block_high_risk',
    });

    expect(result.success).toBe(true);
  });

  it('rejects download modes outside the two approved high-risk policies', () => {
    const result = ParentPolicyConfigSchema.safeParse({
      childAge: 14,
      allowDomains: [],
      blockDomains: [],
      teenUnknownWebEnabled: false,
      socialAccess: [],
      downloadMode: 'allow_safe',
    });

    expect(result.success).toBe(false);
  });

  it('rejects unknown-web enablement for children under 13', () => {
    const result = ParentPolicyConfigSchema.safeParse({
      childAge: 10,
      allowDomains: [],
      blockDomains: [],
      teenUnknownWebEnabled: true,
      socialAccess: [],
      downloadMode: 'block_high_risk',
    });

    expect(result.success).toBe(false);
  });

  it('rejects invalid social time windows', () => {
    const result = ParentPolicyConfigSchema.safeParse({
      childAge: 15,
      allowDomains: [],
      blockDomains: [],
      teenUnknownWebEnabled: false,
      socialAccess: [
        {
          service: 'social.example',
          mode: 'time_limited',
          startMinutes: 1200,
          endMinutes: 480,
        },
      ],
      downloadMode: 'require_parent_high_risk',
    });

    expect(result.success).toBe(false);
  });

  it('carries archive inspection risk metadata with download requests', () => {
    const result = DownloadRequestSchema.safeParse({
      profile: { id: 'kid-1', displayName: 'Ari', age: 14 },
      url: 'https://downloads.example/game.exe.zip',
      fileName: 'game.exe.zip',
      mimeType: 'application/zip',
      archiveContainsHighRisk: true,
    });

    expect(result.success).toBe(true);
    if (result.success) {
      expect(result.data.archiveContainsHighRisk).toBe(true);
    }
  });
});
