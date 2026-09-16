import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';

const root = new URL('../../', import.meta.url);
const dir = mkdtempSync(join(tmpdir(), 'kidos-evidence-'));
const sha = 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
const checksPath = join(dir, 'checks.json');
const out = join(dir, 'evidence.json');
writeFileSync(checksPath, JSON.stringify({ check_runs: [
  { name: 'CodeQL', status: 'completed', conclusion: 'success', head_sha: sha },
  { name: 'dependency-security', status: 'completed', conclusion: 'success', head_sha: sha },
  { name: 'windows-release', status: 'completed', conclusion: 'success', head_sha: sha },
  { name: 'installer-upgrade', status: 'completed', conclusion: 'success', head_sha: sha },
  { name: 'fault-injection', status: 'completed', conclusion: 'success', head_sha: sha },
  { name: 'critical-contract-repeat', status: 'completed', conclusion: 'success', head_sha: sha },
  { name: 'exact-main-sha-smoke', status: 'completed', conclusion: 'success', head_sha: sha },
] }));

const result = spawnSync(process.execPath, [
  'scripts/ci/build-release-evidence.mjs', '--sha', sha, '--checks', checksPath, '--output', out,
], { cwd: root, encoding: 'utf8' });
assert.equal(result.status, 0, result.stderr);
const manifest = JSON.parse(readFileSync(out, 'utf8'));
assert.equal(manifest.commit_sha, sha);
assert.equal(manifest.cloud_ci_ready, true);
assert.equal(manifest.hyperv_visual_validation, 'not_run');
assert.equal(manifest.release_ready, false, 'release cannot be ready without Hyper-V visual pass');

const wrongShaChecks = join(dir, 'wrong-sha.json');
writeFileSync(wrongShaChecks, JSON.stringify({ check_runs: [
  { name: 'CodeQL', status: 'completed', conclusion: 'success', head_sha: 'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb' },
] }));
const wrong = spawnSync(process.execPath, [
  'scripts/ci/build-release-evidence.mjs', '--sha', sha, '--checks', wrongShaChecks, '--output', join(dir, 'wrong.json'),
], { cwd: root, encoding: 'utf8' });
assert.notEqual(wrong.status, 0, 'evidence builder must reject check-runs for another SHA');

console.log('KidOS release evidence tests passed.');
