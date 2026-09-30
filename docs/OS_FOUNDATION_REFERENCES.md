# KidOS OS Foundation Reference Review

KidOS reviewed six attached operating-system projects as architecture references. The implementation in `crates/os-foundation` is original KidOS code and intentionally avoids combining unrelated kernels or copying third-party source.

## What was adopted as design direction

| Reference | Relevant source characteristic | KidOS implementation |
| --- | --- | --- |
| BRUTAL | Capability-based microkernel | `Capability`, `CapabilitySet`, deny-by-default authorization |
| Theseus | Rust type/memory safety; independently recoverable components | `RecoveryMachine` and verified recovery before returning healthy |
| SerenityOS | Aggressive process sandboxing and web-content isolation | `SandboxProfile` and explicit app identities/capabilities |
| Redox | Rust microkernel and separated OS services | Small explicit service boundaries and routes |
| ToaruOS | Complete kernel/userspace/compositor/session structure | `ProcessRole` and child-session topology |
| blog_os | Stepwise Rust boot/kernel learning model | `BootSequence` with Guardian verification before child session |

## Security rule

KidOS keeps one rule above all reference designs: **failure must not silently reduce child protections**. If Guardian health, policy integrity, or child-session containment is not valid, `CoreHealth::mode()` returns `RestrictedSafeMode`.

## Licensing boundary

The attached references use several permissive licenses (including MIT, BSD-2-Clause, and NCSA), but this change does not vendor their source code. Concepts were studied and then implemented independently in KidOS. Any future direct code import must receive a separate provenance and license review before merge.

## Current scope

This is an architectural foundation, not a replacement kernel. The current KidOS Windows build continues to use Windows for hardware drivers and low-level OS facilities. The new crate gives Guardian and future Android/Linux/native targets a shared model for capabilities, isolation, boot gating, process topology, and fault recovery.
