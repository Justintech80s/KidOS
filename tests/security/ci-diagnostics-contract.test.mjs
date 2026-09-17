import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

function read(path) {
  return readFileSync(new URL(`../../${path}`, import.meta.url), 'utf8');
}

const collector = read('scripts/windows/collect-ci-diagnostics.ps1');
const shadow = read('.github/workflows/shadow-validation-ci.yml');

for (const field of ['GITHUB_SHA', 'GITHUB_RUN_ID', 'GITHUB_RUN_ATTEMPT', 'GITHUB_JOB', 'GITHUB_WORKFLOW']) {
  assert.match(collector, new RegExp(`\\$env:${field}`), `diagnostics must record ${field}`);
}
for (const service of ['KidOSGuardian', 'KidOSMediaClassifier']) {
  assert.match(collector, new RegExp(service), `diagnostics must inspect ${service}`);
}
for (const file of ['metadata.json', 'services.txt', 'processes.txt', 'recovery-task.txt', 'service-control-manager-events.txt', 'application-events.txt', 'installed-files.txt', 'summary.json']) {
  assert.match(collector, new RegExp(file.replace('.', '\\.')), `diagnostics must create ${file}`);
}
assert.match(collector, /authorization|cookie|password|token/i, 'diagnostics must define secret redaction patterns');
assert.match(collector, /ProgramData.*KidOS.*Diagnostics/s, 'diagnostics must collect KidOS ProgramData evidence when present');
assert.match(shadow, /collect-ci-diagnostics\.ps1/, 'Shadow validation must use the shared diagnostics collector');
assert.match(shadow, /if:\s*always\(\)/, 'Shadow diagnostic collection must run even after failure');

console.log('KidOS CI diagnostics contract checks passed.');
