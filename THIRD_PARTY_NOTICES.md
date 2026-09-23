# Third-Party Notices

KidOS is proprietary software, but it uses third-party software packages that
remain governed by their own licenses. The KidOS proprietary LICENSE does not
replace, narrow, or override those upstream licenses.

This file records the current direct dependency inventory from the repository's
package manifests. Exact release notices must be regenerated from locked,
resolved dependency versions before a commercial/OEM release.

## JavaScript / TypeScript direct dependencies

Application/runtime dependencies currently include:

- @tauri-apps/api
- react
- react-dom
- zod

Development/test/build dependencies currently include:

- @tauri-apps/cli
- @testing-library/jest-dom
- @testing-library/react
- @types/react
- @types/react-dom
- @vitejs/plugin-react
- jsdom
- playwright
- typescript
- vite
- vitest

Workspace packages under the @kidos namespace are first-party project
components and are governed by the KidOS proprietary license unless separately
stated.

## Rust direct dependencies

Current external Rust dependencies include:

- tauri
- tauri-build
- serde
- serde_json
- reqwest
- base64
- sha2
- semver
- windows-service
- windows-sys
- rusqlite
- wmi
- argon2
- rand_core
- keyring

## Python direct dependencies

The media-classifier service currently declares:

- fastapi
- uvicorn
- pillow
- transformers
- torch
- opencv-python-headless
- pywin32
- pytest
- httpx
- pyinstaller
- huggingface-hub
- setuptools
- wheel

## Release requirement

Before KidOS is shipped under an OEM or other commercial agreement:

1. dependency versions must be locked or otherwise made reproducible;
2. a license scanner must evaluate the exact resolved direct and transitive
   dependency graph;
3. required upstream copyright notices and license texts must be included in the
   release package;
4. copyleft, source-available, non-commercial, custom, or unknown licenses must
   be reviewed before distribution; and
5. the resulting third-party notice bundle must be retained as release
   evidence.

See docs/IP_COMPLIANCE.md for the current audit status and policy.
