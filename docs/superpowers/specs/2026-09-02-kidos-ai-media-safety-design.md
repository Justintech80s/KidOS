# KidOS Hybrid AI Media Safety Design

## Goal
Add an AI-assisted cybersecurity layer that detects unsafe images and videos inside KidOS protected browsing and social-media access, while keeping the deterministic KidOS policy engine as the final security authority.

## Architecture
KidOS uses a hybrid classifier pipeline. Fast local/on-device classification handles common media first. Media that is uncertain or high-risk may be escalated to a vetted remote moderation provider under strict data-minimization rules. The classifier never directly grants access; it emits structured risk evidence consumed by the Rust policy engine.

## Media safety categories
The initial classifier contract supports `safe`, `adult_nudity`, `sexualized_content`, `graphic_violence`, `self_harm`, `drugs`, `extremist_content`, `scam`, and `uncertain`. Categories are safety signals, not diagnoses or permanent labels for people.

## Classification result
Every media classification returns a category, confidence score from 0.0 through 1.0, source (`local` or `remote`), and a risk level. The policy engine converts this evidence into `allow`, `block`, or `require_parent` according to the active child profile and parent policy.

## Image flow
Protected browsing keeps untrusted images obscured until the local classifier has evaluated them when classification is required by policy. Clearly safe media can be revealed. Clearly prohibited media stays blocked. Uncertain high-risk media remains obscured while KidOS either requests parent approval or, when configured and privacy rules permit, escalates classification remotely.

## Video flow
Before playback KidOS evaluates available thumbnail, title, surrounding page context, source reputation, and metadata. During playback it samples frames at bounded intervals rather than attempting to process every frame. Suspicious signals increase sampling frequency. A high-risk result pauses and obscures playback while policy evaluates the result. Redirects and newly loaded media are evaluated independently.

## Site and account reputation
Repeated confirmed unsafe classifications can raise the risk score for a domain, channel, or public account identifier. Reputation is an input to policy, not an automatic permanent ban. Parent explicit allow/block rules retain precedence, except KidOS never lets an allow rule silently bypass content classified into a non-overridable prohibited category for a young-child profile.

KidOS does not inspect or retain a child's private social-media password. v1 does not promise to read every private social message or control a third-party platform's internal moderation.

## Hybrid escalation
Local classification is the default. Remote moderation is used only when local evidence is insufficient and the active policy permits escalation. Requests send the minimum media-derived data needed for classification, avoid full browsing history, use short-lived request identifiers, and do not create an advertising or behavioral profile.

Parents can disable remote classification. When remote classification is disabled or unavailable, KidOS follows fail-closed rules rather than silently treating unknown high-risk media as safe.

## Fail-closed behavior
For ages 3–12, uncertain high-risk media is blocked or requires parent approval. For teen profiles, parent policy may permit lower-risk uncertain content, but high-risk uncertainty remains parent-gated. Classifier outages, timeouts, malformed responses, or unsupported media never become implicit `allow` decisions.

## Policy boundary
The AI media classifier is not a security authority. It cannot modify child profiles, parent rules, allowlists, blocklists, Guardian state, or authorization tokens. Only the Rust policy engine can convert media evidence into an access decision, and only the Guardian Service can broker privileged policy changes.

## Privacy and retention
Raw image/video samples used for local classification are processed ephemerally and are not written to the safety-event database. Remote samples are not retained locally after the classification transaction. KidOS safety events store only minimal metadata needed to explain a decision: timestamp, normalized destination/account identifier when needed, media category, coarse confidence/risk band, policy decision, and reason. Full frames, full videos, full search queries, page text, private messages, and child passwords are excluded from safety logs.

## Parent controls
Parents can configure media-safety sensitivity, remote-classification permission, approved services/accounts, explicit blocked services/accounts, temporary approval behavior, and age-profile rules. KidOS explains blocked content in age-appropriate language without showing the unsafe media.

## Interfaces
A new shared `MediaClassification` contract will contain `category`, `confidence`, `source`, and `risk`. The media-safety service exposes classification operations for images and sampled video frames. `policy-core` receives only validated classification evidence plus profile/policy context. Browser code never receives privileged policy mutation capability.

## Proposed repository additions
```text
packages/contracts/src/media-safety.ts
packages/media-safety/
  src/classifier.ts
  src/local-classifier.ts
  src/remote-classifier.ts
  src/video-sampler.ts
  src/reputation.ts
crates/policy-core/src/media.rs
docs/safety/media-safety.md
tests/e2e/media-safety.spec.ts
```

## Testing
Development is test-first. Contract tests reject unknown categories, invalid confidence values, and malformed remote responses. Media-service tests cover local-safe, local-block, uncertain escalation, remote-disabled behavior, remote outage behavior, bounded video sampling, risk-triggered increased sampling, and reputation updates. Rust policy tests prove AI evidence cannot override explicit policy precedence and that classifier failures never result in implicit allow. End-to-end tests verify unsafe image obscuring, video pause/obscure behavior, parent approval, and restricted safe-mode behavior.

Tests use synthetic fixtures specifically created for safety testing; the KidOS repository does not need to contain explicit sexual imagery or graphic media fixtures.

## Security constraints
- No arbitrary model-generated shell commands.
- No classifier-controlled policy mutation.
- No plaintext parent secrets or service credentials.
- No permanent storage of raw classified child-viewed media.
- No full browsing-history database.
- Remote moderation endpoints and credentials are Guardian-controlled configuration.
- Malformed or unsigned/untrusted remote responses are rejected.
- High-risk classifier failure is fail-closed.

## Roadmap integration
This subsystem is implemented after the core Rust policy engine is stable and before the protected-browser task is considered complete. The protected browser consumes media-safety decisions; the parent dashboard later exposes the corresponding controls and minimal safety-event summaries.

## Success criteria
KidOS can encounter an untrusted image or video in protected browsing, classify it locally, escalate an uncertain case only when policy permits, convert classification evidence through the Rust policy engine, obscure/block/parent-gate unsafe or uncertain high-risk media, avoid retaining raw viewed media, and continue safely when either local or remote classification is unavailable.
