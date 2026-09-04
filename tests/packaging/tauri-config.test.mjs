import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';

const config = JSON.parse(
  await readFile(new URL('../../apps/shell/src-tauri/tauri.conf.json', import.meta.url), 'utf8'),
);

assert.equal(config.productName, 'KidOS');
assert.equal(config.identifier, 'com.justintech80s.kidos');
assert.match(config.version, /^\d+\.\d+\.\d+$/);
assert.deepEqual(config.bundle?.targets, ['nsis']);
assert.deepEqual(config.bundle?.icon, ['icons/icon.ico']);
assert.equal(config.bundle?.active, true);

// The child-facing shell must feel like its own environment instead of a normal
// resizable Windows app. Guardian/Assigned Access remains the security boundary;
// these window settings are presentation hardening only.
const shellWindow = config.app?.windows?.[0];
assert.equal(shellWindow?.fullscreen, true);
assert.equal(shellWindow?.maximized, true);
assert.equal(shellWindow?.decorations, false);
assert.equal(shellWindow?.resizable, false);
assert.equal(shellWindow?.skipTaskbar, true);
assert.equal(shellWindow?.center, true);

assert.equal(
  config.plugins?.updater,
  undefined,
  'updater must remain disabled until signing/update infrastructure exists',
);

console.log('KidOS Windows packaging configuration is immersive-shell and release-bundle ready.');
