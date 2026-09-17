import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';

const root = new URL('../../', import.meta.url);
const dir = mkdtempSync(join(tmpdir(), 'kidos-repeat-'));
const successArtifact = join(dir, 'success.json');
const success = spawnSync(process.execPath, [
  'scripts/ci/repeat-command.mjs', '--count', '3', '--artifact', successArtifact,
  process.execPath, '-e', 'process.exit(0)',
], { cwd: root, encoding: 'utf8' });
assert.equal(success.status, 0, success.stderr);
const successJson = JSON.parse(readFileSync(successArtifact, 'utf8'));
assert.equal(successJson.passed, true);
assert.equal(successJson.completed, 3);

const failArtifact = join(dir, 'fail.json');
const fail = spawnSync(process.execPath, [
  'scripts/ci/repeat-command.mjs', '--count', '4', '--artifact', failArtifact,
  process.execPath, '-e', 'process.exit(7)',
], { cwd: root, encoding: 'utf8' });
assert.equal(fail.status, 7);
const failJson = JSON.parse(readFileSync(failArtifact, 'utf8'));
assert.equal(failJson.passed, false);
assert.equal(failJson.failed_iteration, 1);
assert.equal(failJson.exit_code, 7);

console.log('KidOS repeat-command tests passed.');
