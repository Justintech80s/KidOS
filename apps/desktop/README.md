# KidOS Desktop

KidOS Desktop is the fast-install Electron edition of the KidOS interface.

It packages the existing KidOS React shell into a normal Windows x64 NSIS installer. This edition is intentionally separate from the Tauri + Guardian protected Windows edition.

## Security boundary

KidOS Desktop does **not** claim that the Guardian Windows service, Assigned Access, Restricted Safe Mode, or child-account lockdown is active. The renderer is sandboxed, Node integration is disabled, context isolation is enabled, permission requests are denied by default, and the preload bridge exposes only a bounded runtime-status method.

The protected KidOS Windows edition remains the Tauri + Rust Guardian build.

## Build

From the repository root:

    pnpm install --no-frozen-lockfile
    pnpm --filter @kidos/desktop test
    pnpm --filter @kidos/desktop build

The Windows installer is written to:

    apps/desktop/dist-installer/KidOS-Desktop-Windows-x64-0.1.0-Setup.exe

## Development

    pnpm --filter @kidos/desktop dev
