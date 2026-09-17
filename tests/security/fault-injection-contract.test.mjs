import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

function read(path) {
  return readFileSync(new URL(`../../${path}`, import.meta.url), 'utf8');
}

const harness = read('tests/windows/fault-injection.ps1');
const workflow = read('.github/workflows/fault-injection-ci.yml');

assert.match(harness, /Stop-Service\s+KidOSGuardian/i, 'must inject Guardian service failure');
assert.match(harness, /Stop-Service\s+KidOSMediaClassifier/i, 'must inject classifier service failure');
assert.match(harness, /finally[\s\S]*Start-Service/i, 'must restore services in finally cleanup');
assert.match(harness, /fault-injection\.json/, 'must emit machine-readable fault evidence');
assert.match(harness, /guardian_stopped/, 'must name the Guardian stopped scenario');
assert.match(harness, /classifier_stopped/, 'must name the classifier stopped scenario');
assert.match(workflow, /windows-latest/, 'fault injection must run on disposable GitHub-hosted Windows');
assert.match(workflow, /collect-ci-diagnostics\.ps1/, 'fault workflow must capture diagnostics');
assert.match(workflow, /system-status\.test\.ts/, 'fault workflow must run shell fail-closed contract tests');

console.log('KidOS fault-injection contract checks passed.');
