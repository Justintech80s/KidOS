import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const workflow = readFileSync(new URL('../../.github/workflows/post-merge-smoke.yml', import.meta.url), 'utf8');
assert.match(workflow, /push:[\s\S]*branches:\s*\[?main\]?/s, 'post-merge workflow must trigger from main pushes');
assert.match(workflow, /ref:\s*\$\{\{\s*github\.sha\s*\}\}/, 'checkout must pin the exact pushed SHA');
assert.match(workflow, /system-status\.test\.ts/, 'must run fail-closed shell contract');
assert.match(workflow, /security-regression\.ps1/, 'must run installed Windows security smoke checks');
assert.match(workflow, /fault-injection\.ps1/, 'must exercise a fail-closed runtime smoke case');
assert.match(workflow, /collect-ci-diagnostics\.ps1/, 'must collect diagnostics on every run');
assert.match(workflow, /post-merge-evidence\.json/, 'must emit exact-SHA evidence');
assert.match(workflow, /github\.sha/, 'evidence must include github.sha');
console.log('KidOS post-merge workflow contract checks passed.');
