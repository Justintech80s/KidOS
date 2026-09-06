# KidOS trusted Windows release

KidOS production Windows releases use Authenticode signing and a signed-update manifest.

## Required GitHub repository secrets

- `KIDOS_WINDOWS_PFX_BASE64`: Base64-encoded Windows code-signing PFX certificate.
- `KIDOS_WINDOWS_PFX_PASSWORD`: Password protecting that PFX.

The trusted release workflow refuses to publish if either secret is missing.

## What gets signed

The release pipeline signs and verifies:

- the KidOS desktop executable through Tauri's Windows `signCommand`
- `kidos-guardian-host.exe`
- `kidos-media-classifier.exe`
- the final NSIS installer

The installer is timestamped and then checked with Windows Authenticode verification before release publication.

## Trusted update package

Every release also produces:

- `KidOS-SHA256SUMS.txt`
- `KidOS-update-manifest.json`

The update manifest contains the release version, Windows installer filename, GitHub Release URL, SHA-256 digest, file size, and Authenticode signer metadata. The manifest generator refuses to produce update metadata for an installer whose Windows signature is not valid.

## Publishing

1. Update the version in both `apps/shell/src-tauri/tauri.conf.json` and `apps/shell/src-tauri/Cargo.toml`.
2. Ensure normal release, lockdown, E2E, clean-Windows, and interactive child-session validation have passed.
3. Create and push a tag such as `v0.2.0`.
4. The **KidOS Trusted Windows Release** workflow builds, signs, verifies, hashes, packages, and publishes the GitHub Release.

The workflow can also be started manually for signing validation, but only a `v*` tag publishes a GitHub Release.

## Certificate note

A real certificate must be issued by a trusted Windows code-signing provider. Repository code can prepare and enforce signing, but it cannot create a publicly trusted publisher identity by itself.
