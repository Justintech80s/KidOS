import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';

const iconPath = new URL('../../apps/shell/src-tauri/icons/icon.ico', import.meta.url);
const icon = await readFile(iconPath);

assert.ok(icon.length >= 22, 'KidOS icon must contain an ICO header and at least one directory entry');
assert.equal(icon.readUInt16LE(0), 0, 'ICO reserved field must be zero');
assert.equal(icon.readUInt16LE(2), 1, 'KidOS application icon must be an ICO image');

const count = icon.readUInt16LE(4);
assert.ok(count > 0, 'KidOS icon must contain at least one image');
assert.ok(icon.length >= 6 + count * 16, 'ICO directory must fit inside the file');

for (let index = 0; index < count; index += 1) {
  const entryOffset = 6 + index * 16;
  const bytesInResource = icon.readUInt32LE(entryOffset + 8);
  const imageOffset = icon.readUInt32LE(entryOffset + 12);

  assert.ok(bytesInResource > 0, `ICO image ${index + 1} must not be empty`);
  assert.ok(imageOffset >= 6 + count * 16, `ICO image ${index + 1} must start after the directory`);
  assert.ok(
    imageOffset + bytesInResource <= icon.length,
    `ICO image ${index + 1} declares ${bytesInResource} bytes at offset ${imageOffset}, beyond the ${icon.length}-byte file`,
  );
}

console.log(`KidOS icon container is structurally valid (${count} image${count === 1 ? '' : 's'}, ${icon.length} bytes).`);
