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
assert.equal(config.app?.windows?.[0]?.fullscreen, false);
assert.equal(config.app?.windows?.[0]?.decorations ?? true, true);
assert.equal(config.plugins?.updater, undefined, 'updater must remain disabled until signing/update infrastructure exists');

console.log('KidOS Windows packaging configuration is least-privilege and release-bundle ready.');
