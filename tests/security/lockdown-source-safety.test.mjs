import assert from 'node:assert/strict';
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, extname, join, relative, resolve } from 'node:path';
import test from 'node:test';

const testDirectory = dirname(fileURLToPath(import.meta.url));
const root = resolve(testDirectory, '../..');
const roots = ['apps/shell/src', 'apps/shell/src-tauri/src', 'crates/guardian-service/src', 'packages/contracts/src'];
const extensions = new Set(['.ts', '.tsx', '.rs']);

function filesUnder(path) {
  const result = [];
  for (const entry of readdirSync(path)) {
    const full = join(path, entry);
    if (statSync(full).isDirectory()) result.push(...filesUnder(full));
    else if (extensions.has(extname(full))) result.push(full);
  }
  return result;
}

const files = roots.flatMap((path) => filesUnder(join(root, path)));
const source = files.map((file) => `\n// ${relative(root, file)}\n${readFileSync(file, 'utf8')}`).join('\n');

test('renderer does not expose generic operating-system command dispatch', () => {
  const forbidden = [
    /invoke\s*<[^>]*>\s*\(\s*['"](?:run|exec|shell|command|powershell|cmd)['"]/i,
    /Command::new\s*\(/,
    /std::process::Command/,
  ];
  for (const pattern of forbidden) assert.doesNotMatch(source, pattern);
});

test('renderer cannot submit raw Assigned Access XML', () => {
  const renderer = readFileSync(join(root, 'apps/shell/src/lib/kidos-api.ts'), 'utf8');
  assert.doesNotMatch(renderer, /AssignedAccessConfiguration|rawXml|assignedAccessXml/i);
});

test('lockdown code does not persist plaintext parent PINs', () => {
  assert.doesNotMatch(source, /localStorage\.(?:setItem|getItem)\([^\n]*(?:pin|password)/i);
  assert.doesNotMatch(source, /sessionStorage\.(?:setItem|getItem)\([^\n]*(?:pin|password)/i);
});

test('child-facing code cannot configure administrative-tool allowlisting', () => {
  const renderer = readFileSync(join(root, 'apps/shell/src/lib/kidos-api.ts'), 'utf8');
  for (const executable of ['cmd.exe', 'powershell.exe', 'pwsh.exe', 'regedit.exe', 'wt.exe', 'wscript.exe', 'cscript.exe', 'mmc.exe']) {
    assert.equal(renderer.toLowerCase().includes(executable), false, `${executable} must not be renderer-configurable`);
  }
});
