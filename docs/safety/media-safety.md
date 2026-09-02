# KidOS AI Media Safety

KidOS uses an AI-assisted media safety layer to reduce exposure to inappropriate images and video while keeping deterministic policy and parent controls in charge of the final decision.

## Security model

The media classifier is evidence, not authority. A classification describes a category, confidence, risk level, and whether the result came from the local or approved remote classifier. Only the KidOS policy layer may convert that evidence into one of three decisions: `allow`, `block`, or `require_parent`.

The child-facing browser must not override policy decisions. Allowed images may be shown. Blocked or parent-gated images are obscured. Blocked or parent-gated video frames cause playback to pause until policy permits playback.

## Classification categories

KidOS recognizes these media-safety categories:

- `safe`
- `adult_nudity`
- `sexualized_content`
- `graphic_violence`
- `self_harm`
- `drugs`
- `extremist_content`
- `scam`
- `uncertain`

Classification confidence is constrained to the range 0 through 1. Risk is represented as low, medium, or high.

## Local-first hybrid moderation

KidOS evaluates supported media locally first. A confident low-risk local result can avoid a remote request. Uncertain, high-risk, low-confidence, malformed, or failed local classifications may be escalated to an approved remote moderation service when the parent policy permits remote moderation.

Remote moderation is a fallback, not a bypass. If remote moderation is disabled, unavailable, throws an error, or returns malformed data, KidOS preserves the local risk evidence or uses a fail-closed `uncertain` / high-risk result.

A classifier failure must never become an implicit allow.

## Age-sensitive policy

Parent blocks take priority over classifier results. For younger child profiles, high-confidence adult nudity, sexualized content, and graphic violence are blocked. High-risk uncertain media requires parent authorization for every age profile. Lower-risk uncertain media for teens may only be allowed when an explicit parent-controlled policy enables that behavior.

The model cannot change the child's age, parent rules, allow/block lists, Guardian state, authorization tokens, or policy decisions.

## Browser behavior

The protected-browser media gate translates policy decisions into child-facing display states:

| Policy decision | Image | Video frame |
| --- | --- | --- |
| `allow` | show | continue/show |
| `block` | obscure | pause |
| `require_parent` | obscure | pause |

The browser gate classifies the media first and then requests a policy decision. It does not create a second policy authority inside the renderer.

## Video sampling

Video safety uses bounded frame sampling rather than attempting to retain or analyze an entire video as a permanent artifact. Sampling becomes more frequent as risk rises:

- low risk: approximately every 10 seconds
- medium risk: approximately every 5 seconds
- high risk: approximately every 2 seconds

A product integration may also inspect thumbnails, metadata, page context, and reputation signals before playback. High-risk uncertainty should keep playback paused while policy resolution is pending.

## Reputation signals

Repeated unsafe classifications may increase a normalized risk score for a destination, channel, account, or other policy subject. Reputation is only an input to policy. It is not permission for an AI model to permanently ban an account by itself.

## Privacy and data minimization

KidOS is designed so viewed raw image and video samples are ephemeral safety inputs, not browsing-history records. Raw media bytes must not be written to the safety-event database by the media-safety layer. Remote samples should not be retained locally after the moderation transaction.

Minimal safety events may contain only what is needed for parent-facing safety summaries, such as:

- timestamp
- normalized destination or account identifier when needed
- media category
- coarse confidence or risk band
- policy decision
- reason

Safety events must not contain full viewed frames, full videos, passwords, authentication tokens, complete search queries, private messages, arbitrary page text, or a full browsing-history timeline.

KidOS does not sell child activity or create an advertising profile from safety events.

## Remote-service boundaries

Remote classifier endpoints and credentials must be controlled by privileged KidOS configuration rather than arbitrary child-page input. Only the minimum media/context necessary for moderation should be sent. Responses are untrusted inputs and must be schema-validated before policy consumes them.

## Failure behavior

The media safety path fails closed:

- local classifier throws -> `uncertain`, high-risk fallback or approved remote escalation
- local classifier returns malformed data -> fail-closed fallback or approved remote escalation
- remote classifier throws -> preserve local/fail-closed evidence
- remote classifier returns malformed data -> preserve local/fail-closed evidence
- policy or Guardian cannot safely resolve high-risk media -> block or require parent according to the active safe-mode policy

No classifier or remote-service failure is an automatic permission to display high-risk media.

## Security constraints

The media-safety subsystem must not:

- execute arbitrary operating-system shell commands
- mutate parent policy directly
- persist parent secrets in plaintext
- persist raw viewed media as a safety log
- build a full child browsing-history database
- treat model output as trusted policy

Privileged changes belong behind the Guardian and authenticated parent controls as those components are implemented.

## Verification requirements

Changes to this subsystem are expected to pass the integrated KidOS Media Safety CI. The gate covers shared contracts, media-safety classifier/reputation/sampling tests and typechecking, Rust policy tests/checks, shell browser-gate tests/build, and the Windows Rust workspace check.

Synthetic fixtures and deterministic test classifiers are preferred for automated safety tests. Real inappropriate child-directed content is neither needed nor appropriate for repository fixtures.

## Product claims

KidOS uses multiple layers of filtering, AI-assisted classification, deterministic policy, and parent controls designed to restrict inappropriate content. It must not be marketed as guaranteeing perfect detection or as blocking every harmful item on the internet.