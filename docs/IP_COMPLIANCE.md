# KidOS IP and License Compliance

_Last reviewed: 2026-09-23_

This document records the repository-level controls used to support proprietary
commercial and OEM licensing of KidOS. It is an engineering/IP hygiene record,
not a legal opinion.

## Current ownership/provenance review

A GitHub commit-history review performed on 2026-09-23 covered 423 commits on
the repository's default branch history. All 423 commits were associated with
the GitHub account `Justintech80s`, and no `Co-authored-by:` trailers were
found in those commit messages.

That result materially reduces the current contributor-ownership risk, but it
does not by itself prove that every line, asset, model, snippet, or design was
created without third-party rights. Code copied from another source, contractor
work, generated assets, external design files, and embedded third-party
materials must still have documented provenance.

## Repository license posture

KidOS uses a proprietary root LICENSE. Public visibility of the repository does
not grant a general software license. Commercial/OEM rights require a separate
written agreement.

Third-party components are excluded from the proprietary grant and remain under
their upstream licenses.

## Contributor control

Future external contributions require acceptance of
`CONTRIBUTOR_LICENSE_AGREEMENT.md`.

The agreement gives the KidOS owner broad rights to use, modify, sublicense,
relicense, distribute, and commercialize accepted contributions while allowing
contributors to retain copyright in their own contribution.

The pull-request template contains an explicit contributor-agreement
confirmation so acceptance is recorded with the contribution.

## Dependency audit status

### Direct dependency inventory

The repository currently contains these package manifests:

- `apps/shell/package.json`
- `apps/shell/src-tauri/Cargo.toml`
- `crates/guardian-host/Cargo.toml`
- `crates/guardian-service/Cargo.toml`
- `crates/policy-core/Cargo.toml`
- `crates/secure-store/Cargo.toml`
- `crates/shadow-core/Cargo.toml`
- `packages/contracts/package.json`
- `packages/creation-engine/package.json`
- `packages/media-safety/package.json`
- `services/media-classifier/pyproject.toml`
- `tests/e2e/package.json`

The direct dependency names are recorded in `THIRD_PARTY_NOTICES.md`.

### Reproducibility gap

At the time of this review, the repository tree did not contain a committed
`pnpm-lock.yaml`, `Cargo.lock`, or Python lock file. Several manifests use
version ranges. Therefore the exact transitive dependency set and exact license
obligations cannot yet be treated as frozen release evidence.

This is a release blocker for OEM/commercial distribution, not a blocker for
continued development.

## License policy for commercial releases

The release process should generally permit commonly used permissive licenses
when their notice/attribution obligations are satisfied, including licenses such
as MIT, BSD-2-Clause, BSD-3-Clause, Apache-2.0, ISC, Zlib, and similarly
permissive terms.

The following require explicit review before commercial distribution:

- GPL, AGPL, SSPL, or other strong/network copyleft licenses;
- LGPL or weak-copyleft components where linking/distribution obligations must
  be evaluated;
- source-available or field-of-use-restricted licenses;
- non-commercial or research-only licenses;
- packages with missing, custom, or ambiguous license metadata; and
- binary/model/data assets with separate terms.

A dependency is not approved merely because it is available through npm,
crates.io, PyPI, GitHub, Hugging Face, or another package registry.

## Release gate

A commercial/OEM build must not be labeled IP-compliance-ready until all of the
following are true:

- exact dependency versions are reproducibly resolved;
- direct and transitive licenses have been scanned;
- required notices and source-offer obligations, if any, are satisfied;
- third-party models/assets are inventoried;
- all external contributors have documented contribution rights;
- the release has a retained software bill of materials or equivalent
  dependency inventory; and
- any exception has written approval from the project owner and, for material
  commercial risk, qualified counsel.

## Asset and model provenance

For each image, video, font, icon, model, dataset, or other non-code asset
included in KidOS, retain:

- source/origin;
- creator or provider;
- applicable license or written permission;
- date acquired;
- whether modification/redistribution is permitted; and
- any attribution requirement.

## Commercial contracts

The repository LICENSE is not the OEM contract. A separate commercial agreement
should define at minimum:

- licensed products/devices and platform;
- permitted distribution and installation rights;
- fees and payment schedule;
- per-device or subscription terms;
- source-code/API access, if any;
- support and update obligations;
- confidentiality;
- security and privacy responsibilities;
- trademark/co-branding rights;
- warranties, indemnities, and liability allocation;
- term and termination; and
- whether any exclusivity exists.

Unless separately negotiated in writing, an OEM customer should receive a
limited license rather than ownership of the KidOS intellectual property.
