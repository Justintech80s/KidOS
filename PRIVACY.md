# KidOS Privacy Principles

KidOS is intended for child-focused computing, so the default architecture should minimize collection and keep safety processing local whenever practical.

## Current design principles

- Parent policy, recovery state, quarantine metadata, update state, and service credentials are stored locally on the Windows device.
- Image and video safety classification is designed to run through the local KidOS media-classifier service on `127.0.0.1`.
- Quarantined media stays on the device unless a future parent-authorized feature explicitly says otherwise.
- The child-facing shell does not need access to the Guardian service's secret storage.
- KidOS does not require advertising identifiers for its safety architecture.
- Production telemetry should be opt-in or strictly minimized, documented, and separated from child-created content.
- Secrets such as the parent PIN and classifier token must never be written to ordinary application logs.

## Uninstall behavior

The Windows uninstall path removes KidOS program files, Guardian services, recovery tasks, classifier credentials, protected update staging data, and KidOS data stored under ProgramData.

## Future network services

Any future cloud AI, account sync, messaging, analytics, or remote-parent feature must receive a separate privacy review before being enabled by default. That review should define exactly what leaves the device, why it is needed, retention, deletion behavior, parent controls, and age-appropriate consent requirements.

## Children and production release

Before public family distribution, KidOS should receive a legal/privacy review appropriate to the countries and age groups where it will be offered. Repository architecture and technical controls are not a substitute for that review.
