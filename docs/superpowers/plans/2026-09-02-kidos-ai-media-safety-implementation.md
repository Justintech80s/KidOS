# KidOS AI Media Safety Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a hybrid local-first AI media safety subsystem that classifies images and sampled video frames, escalates uncertain cases only when policy permits, and feeds validated evidence into KidOS's Rust policy engine.

**Architecture:** Media classification is a separate unprivileged subsystem. Local classification runs first; uncertain/high-risk cases may be escalated through a narrow remote adapter. The classifier never returns access decisions. `policy-core` remains the only component that converts media evidence into `allow`, `block`, or `require_parent`.

**Tech Stack:** TypeScript, Zod, Vitest, Rust, Cargo tests, Tauri/React integration points, provider-agnostic remote moderation adapter.

**Spec:** `docs/superpowers/specs/2026-09-02-kidos-ai-media-safety-design.md`

## Global Constraints

- Local classification is the default path.
- Remote classification is optional and parent/policy controlled.
- The classifier cannot mutate policy, profiles, Guardian state, or parent authorization.
- Classifier failure never produces implicit `allow` for high-risk media.
- Raw viewed image/video samples are not written to long-term safety logs.
- Full browsing history, private messages, child passwords, and full search queries are excluded from safety logs.
- Tests use synthetic fixtures; explicit sexual or graphic media is not required in the repository.
- Final security outcomes remain exactly `allow`, `block`, or `require_parent`.

---

## File Structure

```text
packages/contracts/src/media-safety.ts
packages/contracts/src/media-safety.test.ts
packages/contracts/src/index.ts
packages/media-safety/package.json
packages/media-safety/tsconfig.json
packages/media-safety/src/index.ts
packages/media-safety/src/classifier.ts
packages/media-safety/src/local-classifier.ts
packages/media-safety/src/remote-classifier.ts
packages/media-safety/src/video-sampler.ts
packages/media-safety/src/reputation.ts
packages/media-safety/src/*.test.ts
crates/policy-core/src/media.rs
crates/policy-core/src/lib.rs
crates/policy-core/tests/media_policy.rs
apps/shell/src/features/browser/media-safety.ts
docs/safety/media-safety.md
```

## Task 1: Add shared media-safety contracts

**Files:**
- Create: `packages/contracts/src/media-safety.ts`
- Create: `packages/contracts/src/media-safety.test.ts`
- Modify: `packages/contracts/src/index.ts`

**Interfaces:**
- Produces `MediaSafetyCategory`, `MediaClassification`, `MediaRisk`, and `ClassificationSource` schemas/types.
- Categories: `safe`, `adult_nudity`, `sexualized_content`, `graphic_violence`, `self_harm`, `drugs`, `extremist_content`, `scam`, `uncertain`.

- [ ] **Step 1: Write failing contract tests**

```ts
import { describe, expect, it } from 'vitest';
import { MediaClassificationSchema } from './media-safety';

describe('media safety contracts', () => {
  it('accepts a valid local safe result', () => {
    expect(MediaClassificationSchema.safeParse({
      category: 'safe', confidence: 0.98, source: 'local', risk: 'low'
    }).success).toBe(true);
  });

  it('rejects invalid confidence and unknown categories', () => {
    expect(MediaClassificationSchema.safeParse({
      category: 'unknown-new-category', confidence: 1.2, source: 'local', risk: 'high'
    }).success).toBe(false);
  });
});
```

- [ ] **Step 2: Run and verify failure**

Run: `pnpm --filter @kidos/contracts test --run`
Expected: FAIL because `./media-safety` does not exist.

- [ ] **Step 3: Implement the contracts**

```ts
import { z } from 'zod';

export const MediaSafetyCategorySchema = z.enum([
  'safe', 'adult_nudity', 'sexualized_content', 'graphic_violence',
  'self_harm', 'drugs', 'extremist_content', 'scam', 'uncertain',
]);
export type MediaSafetyCategory = z.infer<typeof MediaSafetyCategorySchema>;

export const ClassificationSourceSchema = z.enum(['local', 'remote']);
export type ClassificationSource = z.infer<typeof ClassificationSourceSchema>;

export const MediaRiskSchema = z.enum(['low', 'medium', 'high']);
export type MediaRisk = z.infer<typeof MediaRiskSchema>;

export const MediaClassificationSchema = z.object({
  category: MediaSafetyCategorySchema,
  confidence: z.number().min(0).max(1),
  source: ClassificationSourceSchema,
  risk: MediaRiskSchema,
});
export type MediaClassification = z.infer<typeof MediaClassificationSchema>;
```

Export these from `packages/contracts/src/index.ts`.

- [ ] **Step 4: Verify**

Run:
`pnpm --filter @kidos/contracts test --run`
`pnpm --filter @kidos/contracts typecheck`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add packages/contracts/src/media-safety.ts packages/contracts/src/media-safety.test.ts packages/contracts/src/index.ts
git commit -m "feat: add KidOS media safety contracts"
```

## Task 2: Build the local-first classification orchestrator

**Files:**
- Create: `packages/media-safety/package.json`
- Create: `packages/media-safety/tsconfig.json`
- Create: `packages/media-safety/src/classifier.ts`
- Create: `packages/media-safety/src/local-classifier.ts`
- Create: `packages/media-safety/src/remote-classifier.ts`
- Create: `packages/media-safety/src/index.ts`
- Create: `packages/media-safety/src/classifier.test.ts`

**Interfaces:**
- Consumes `MediaClassification` contracts.
- Produces `classifyMedia(input, options): Promise<MediaClassification>`.
- `options.remoteEnabled: boolean` controls escalation.

- [ ] **Step 1: Write failing orchestration tests**

```ts
it('returns a confident local classification without remote escalation', async () => {
  const result = await classifyMedia(fakeInput, {
    remoteEnabled: true,
    localClassifier: async () => ({ category: 'safe', confidence: 0.95, source: 'local', risk: 'low' }),
    remoteClassifier: async () => { throw new Error('remote should not run'); },
  });
  expect(result.source).toBe('local');
});

it('fails closed when local is uncertain and remote is disabled', async () => {
  const result = await classifyMedia(fakeInput, {
    remoteEnabled: false,
    localClassifier: async () => ({ category: 'uncertain', confidence: 0.4, source: 'local', risk: 'high' }),
  });
  expect(result).toEqual({ category: 'uncertain', confidence: 0.4, source: 'local', risk: 'high' });
});
```

- [ ] **Step 2: Run and verify failure**

Run: `pnpm --filter @kidos/media-safety test --run`
Expected: FAIL because the package/classifier does not exist.

- [ ] **Step 3: Implement minimal orchestrator**

Define `MediaInput = { kind: 'image' | 'video_frame'; bytes: Uint8Array; context?: { title?: string; domain?: string; accountId?: string } }`.

Rules:
1. call local classifier first;
2. return local result if category is not `uncertain` and confidence >= 0.80;
3. if uncertain/high-risk and remote is enabled, call remote adapter;
4. validate remote response with `MediaClassificationSchema`;
5. malformed/error remote responses return the original local uncertain/high-risk result;
6. never convert errors into `safe`.

- [ ] **Step 4: Verify**

Run: `pnpm --filter @kidos/media-safety test --run`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add packages/media-safety
git commit -m "feat: add hybrid KidOS media classifier"
```

## Task 3: Add bounded video sampling and reputation signals

**Files:**
- Create: `packages/media-safety/src/video-sampler.ts`
- Create: `packages/media-safety/src/video-sampler.test.ts`
- Create: `packages/media-safety/src/reputation.ts`
- Create: `packages/media-safety/src/reputation.test.ts`

**Interfaces:**
- Produces `nextSampleIntervalMs(currentRisk): number`.
- Produces `updateReputation(currentScore, classification): number` with score clamped from 0 through 100.

- [ ] **Step 1: Write failing tests**

```ts
it('samples high-risk video more frequently than low-risk video', () => {
  expect(nextSampleIntervalMs('high')).toBeLessThan(nextSampleIntervalMs('low'));
});

it('raises reputation risk after confirmed unsafe classifications', () => {
  expect(updateReputation(20, { category: 'adult_nudity', confidence: 0.98, source: 'local', risk: 'high' })).toBeGreaterThan(20);
});
```

- [ ] **Step 2: Verify failure**

Run: `pnpm --filter @kidos/media-safety test --run`
Expected: FAIL for missing sampler/reputation functions.

- [ ] **Step 3: Implement bounded logic**

Use fixed intervals: low = 10000 ms, medium = 5000 ms, high = 2000 ms. Never sample faster than 2000 ms in v1.

Reputation rules: safe high-confidence result subtracts 2; medium-risk adds 5; high-risk adds 15; clamp 0..100. `uncertain` adds 5 rather than creating a permanent ban.

- [ ] **Step 4: Verify**

Run: `pnpm --filter @kidos/media-safety test --run`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add packages/media-safety/src/video-sampler.ts packages/media-safety/src/video-sampler.test.ts packages/media-safety/src/reputation.ts packages/media-safety/src/reputation.test.ts
git commit -m "feat: add KidOS video sampling and reputation signals"
```

## Task 4: Integrate media evidence into Rust policy-core

**Files:**
- Create: `crates/policy-core/src/media.rs`
- Modify: `crates/policy-core/src/lib.rs`
- Create: `crates/policy-core/tests/media_policy.rs`

**Interfaces:**
- Produces `evaluate_media(&MediaContext) -> PolicyDecision`.
- `MediaContext` includes age, category, risk, confidence band, parent override, and classifier availability.

- [ ] **Step 1: Write failing Rust tests**

```rust
#[test]
fn blocks_high_confidence_adult_media_for_young_child() {
    let ctx = MediaContext::new(10, MediaCategory::AdultNudity, MediaRisk::High, 98);
    assert_eq!(evaluate_media(&ctx), PolicyDecision::Block);
}

#[test]
fn uncertain_high_risk_media_requires_parent() {
    let ctx = MediaContext::new(11, MediaCategory::Uncertain, MediaRisk::High, 40);
    assert_eq!(evaluate_media(&ctx), PolicyDecision::RequireParent);
}

#[test]
fn classifier_failure_never_implicitly_allows() {
    let ctx = MediaContext::unavailable(9);
    assert_eq!(evaluate_media(&ctx), PolicyDecision::RequireParent);
}
```

- [ ] **Step 2: Verify failure**

Run: `cargo test -p policy-core`
Expected: FAIL because media policy types/functions are undefined.

- [ ] **Step 3: Implement deterministic precedence**

Rules:
1. explicit parent block -> `Block`;
2. classifier unavailable/malformed -> `RequireParent` for media requiring classification;
3. ages 3–12 + high-confidence adult nudity/sexualized content/graphic violence -> `Block`;
4. any age + high-risk uncertain -> `RequireParent`;
5. safe + high confidence -> `Allow` unless parent block applies;
6. teen lower-risk uncertain -> `Allow` only if profile flag explicitly permits it;
7. fallback -> `RequireParent`.

Do not add network, filesystem, model, or UI calls to `policy-core`.

- [ ] **Step 4: Verify**

Run: `cargo test -p policy-core`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/policy-core/src/media.rs crates/policy-core/src/lib.rs crates/policy-core/tests/media_policy.rs
git commit -m "feat: add KidOS media policy decisions"
```

## Task 5: Add protected-browser media gate adapter

**Files:**
- Create: `apps/shell/src/features/browser/media-safety.ts`
- Create: `apps/shell/src/features/browser/media-safety.test.ts`

**Interfaces:**
- Consumes `classifyMedia()` and a typed policy-evaluation bridge.
- Produces `evaluateImageForDisplay()` and `evaluateVideoFrame()` returning `{ state: 'show' | 'obscure' | 'pause'; decision: PolicyDecision; reason: string }`.

- [ ] **Step 1: Write failing tests**

```ts
it('obscures an image while high-risk media requires parent approval', async () => {
  const result = await evaluateImageForDisplay(fakeImage, depsReturningRequireParent);
  expect(result.state).toBe('obscure');
  expect(result.decision).toBe('require_parent');
});

it('pauses video when a sampled frame is blocked', async () => {
  const result = await evaluateVideoFrame(fakeFrame, depsReturningBlock);
  expect(result.state).toBe('pause');
  expect(result.decision).toBe('block');
});
```

- [ ] **Step 2: Verify failure**

Run: `pnpm --filter @kidos/shell test --run`
Expected: FAIL because browser media-safety adapter does not exist.

- [ ] **Step 3: Implement narrow adapter**

Mapping:
- `allow` -> image `show`, video continues;
- `require_parent` -> image `obscure`, video `pause`;
- `block` -> image `obscure`, video `pause`.

Do not expose raw media bytes to logs or parent dashboard state.

- [ ] **Step 4: Verify**

Run: `pnpm --filter @kidos/shell test --run`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add apps/shell/src/features/browser/media-safety.ts apps/shell/src/features/browser/media-safety.test.ts
git commit -m "feat: gate browser media through KidOS safety policy"
```

## Task 6: Document privacy behavior and add integrated CI

**Files:**
- Create: `docs/safety/media-safety.md`
- Create: `.github/workflows/media-safety-ci.yml`

**Interfaces:**
- Produces repository-level verification for contracts, media-safety package, policy-core, and shell media-gate tests.

- [ ] **Step 1: Add privacy/security documentation**

Document that raw classified frames are ephemeral, remote escalation is optional, full browsing history is not stored, private social passwords/messages are not collected by this subsystem, and the classifier cannot change policy.

- [ ] **Step 2: Add CI workflow**

Run these commands:

```bash
pnpm install --no-frozen-lockfile
pnpm --filter @kidos/contracts test --run
pnpm --filter @kidos/contracts typecheck
pnpm --filter @kidos/media-safety test --run
pnpm --filter @kidos/shell test --run
cargo test -p policy-core
cargo check --workspace
```

- [ ] **Step 3: Run full verification**

Expected: every command exits 0.

- [ ] **Step 4: Repository safety scan**

Search the repository and confirm there is no plaintext parent secret, no raw-media persistence path in the media-safety package, no generic shell-command bridge, and no classifier method that mutates policy.

- [ ] **Step 5: Commit**

```bash
git add docs/safety/media-safety.md .github/workflows/media-safety-ci.yml
git commit -m "docs: verify KidOS AI media safety architecture"
```

## Integration Order

1. Finish and merge the existing core Rust policy-engine task.
2. Implement Tasks 1–4 above so media evidence has contracts, classification, sampling/reputation, and policy decisions.
3. Implement Task 5 together with the protected-browser milestone so media can actually be obscured/paused.
4. Implement Task 6 before the media-safety feature is considered complete.

## Completion Gate

AI Media Safety is complete only when all contract, TypeScript, Rust, shell, and workspace checks are green; malformed/failed classification never implicitly allows high-risk media; raw viewed media is not persisted; and the protected-browser adapter demonstrably obscures or pauses blocked/parent-gated media.
