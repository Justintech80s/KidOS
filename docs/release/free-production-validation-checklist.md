# KidOS free production-validation checklist

This checklist tracks the release-hardening work that can be completed without purchasing a Windows code-signing certificate or paid testing service.

## Automated free gates

- [x] End-to-end application tests
- [x] Windows installer build
- [x] Clean Windows install/service/recovery/uninstall smoke test
- [x] Guardian service failure recovery test
- [x] Media-classifier failure recovery test
- [x] Protected directory ACL regression checks
- [x] Classifier credential ACL checks
- [x] Unauthenticated classifier rejection test
- [x] Scheduled protection soak test
- [x] Dependency vulnerability review
- [x] Rust dependency audit
- [x] Python dependency audit
- [x] CodeQL JavaScript/TypeScript analysis
- [x] Rust Clippy security gate
- [x] CycloneDX SBOM generation
- [x] Build provenance attestation
- [x] SBOM attestation
- [x] Attestation verification
- [x] SHA-256 release manifest
- [x] Trusted update manifest generation
- [x] Guardian updater anti-rollback and Authenticode verification

## Free checks requiring a real Windows machine

These cost nothing if a compatible Windows test PC/VM is already available, but cannot be proven by a GitHub-hosted runner alone.

- [ ] Reboot after KidOS installation
- [ ] Sign into the real standard child Windows account
- [ ] Confirm Assigned Access persists across reboot
- [ ] Confirm KidOS becomes the child session shell
- [ ] Confirm Guardian and classifier survive reboot
- [ ] Attempt normal escape paths from the child session
- [ ] Test sleep/wake and network loss
- [ ] Test Windows Update/reboot interactions
- [ ] Run multi-day real-use soak testing
- [ ] Verify parent recovery from an administrator account

Use the **KidOS Interactive Windows Session Validation** workflow with a disposable self-hosted Windows test machine to record these results.

## Paid/external gate

- [ ] Obtain a publicly trusted Windows code-signing certificate or compatible cloud-signing service.
- [ ] Configure trusted signing credentials in GitHub without committing private key material.
- [ ] Produce and verify the first trusted signed KidOS Windows release.

KidOS should not be described as fully production-validated for families until the real child-session checks and trusted signing gate are complete.
