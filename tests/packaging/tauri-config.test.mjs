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
assert.equal(config.bundle?.windows?.nsis?.installMode, 'perMachine');
assert.equal(config.bundle?.windows?.nsis?.installerHooks, './windows/hooks.nsh');

// Validate the ICO container before Tauri's Rust build script reads it. A
// truncated icon previously caused every Windows pipeline to fail deep inside
// tauri-build with an opaque image decoder panic.
const icon = await readFile(new URL('../../apps/shell/src-tauri/icons/icon.ico', import.meta.url));
assert.ok(icon.length >= 22, 'KidOS icon must contain an ICO header and directory entry');
assert.equal(icon.readUInt16LE(0), 0, 'ICO reserved field must be zero');
assert.equal(icon.readUInt16LE(2), 1, 'KidOS application icon must be an ICO image');
const iconCount = icon.readUInt16LE(4);
assert.ok(iconCount > 0, 'KidOS icon must contain at least one image');
assert.ok(icon.length >= 6 + iconCount * 16, 'ICO directory must fit inside the file');
for (let index = 0; index < iconCount; index += 1) {
  const entryOffset = 6 + index * 16;
  const bytesInResource = icon.readUInt32LE(entryOffset + 8);
  const imageOffset = icon.readUInt32LE(entryOffset + 12);
  assert.ok(bytesInResource > 0, `ICO image ${index + 1} must not be empty`);
  assert.ok(imageOffset >= 6 + iconCount * 16, `ICO image ${index + 1} must start after the directory`);
  assert.ok(
    imageOffset + bytesInResource <= icon.length,
    `ICO image ${index + 1} exceeds the ${icon.length}-byte file`,
  );
}

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

console.log(`KidOS Windows packaging configuration is valid; ICO contains ${iconCount} image(s).`);
