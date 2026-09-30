# KidOS OS Foundation

This crate is the platform-neutral security and lifecycle layer for KidOS.

It does **not** merge third-party operating systems into KidOS and it does not copy their kernels. Instead, it turns the most useful architectural ideas from the reviewed OS projects into KidOS-owned interfaces that can be used by the current Windows Guardian and future Android/Linux/native KidOS targets.

## Implemented primitives

- **Capabilities:** deny-by-default app privileges for camera, microphone, network, child media, approved app launch, parent settings, Guardian control, and system settings.
- **Sandbox profiles:** bind an app identity to the smallest explicit capability set it needs.
- **Fail-closed health:** Guardian health, policy integrity, and child-session containment must all be valid before the platform is considered healthy.
- **Recovery state machine:** faults move KidOS to restricted mode; normal mode returns only after recovery is explicitly verified.
- **Boot gating:** a child session cannot advance to ready state before Guardian verification.
- **Service topology:** only explicitly declared process-to-process routes are authorized.

## Reference mapping

- BRUTAL -> capability-based security model.
- Theseus -> fault recovery and safe-language isolation ideas.
- SerenityOS -> aggressive process isolation/sandboxing ideas.
- Redox -> small service boundaries and microkernel-inspired separation.
- ToaruOS -> complete OS/session/compositor structure reference.
- blog_os -> explicit boot-stage modeling and low-level Rust OS learning reference.

No third-party source code from these projects is copied into this crate.
