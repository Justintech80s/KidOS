import fs from 'node:fs';
import assert from 'node:assert/strict';

const hooks = fs.readFileSync('apps/shell/src-tauri/windows/hooks.nsh', 'utf8');
const stopScript = fs.readFileSync('scripts/windows/stop-kidos-services.ps1', 'utf8');
const extractor = fs.readFileSync('scripts/windows/extract-media-classifier.ps1', 'utf8');

const postStart = hooks.indexOf('!macro NSIS_HOOK_POSTINSTALL');
const guardianCopy = hooks.indexOf('File /oname=kidos-guardian-host.exe', postStart);
const preflightStop = hooks.indexOf('stop-kidos-services.ps1', postStart);
assert(postStart >= 0 && guardianCopy >= 0 && preflightStop >= 0, 'installer hook markers are missing');
assert(preflightStop < guardianCopy, 'KidOS must stop/remove existing services before replacing the Guardian binary');

assert.match(stopScript, /Wait-ServiceGone|Wait-KidOSServiceGone/, 'service cleanup must wait for SCM removal');
assert.match(stopScript, /Get-Process|Win32_Process/, 'service cleanup must verify KidOS processes are gone');
assert.match(stopScript, /timeout|deadline/i, 'service cleanup must have a bounded timeout');

assert.match(extractor, /StagingPath|staging/i, 'classifier extraction must use a staging directory');
assert.match(extractor, /Move-Item|Rename-Item/, 'classifier staging must be promoted only after validation');
assert.match(extractor, /kidos-media-classifier\.exe/, 'classifier host must be validated before promotion');

const preUninstall = hooks.indexOf('!macro NSIS_HOOK_PREUNINSTALL');
const postUninstall = hooks.indexOf('!macro NSIS_HOOK_POSTUNINSTALL');
assert(preUninstall >= 0 && postUninstall > preUninstall, 'uninstall hooks are missing');
const uninstallBody = hooks.slice(preUninstall, postUninstall);
assert.match(uninstallBody, /stop-kidos-services\.ps1/, 'uninstall must use bounded service/process cleanup');

console.log('KidOS installer lifecycle contract passed.');
