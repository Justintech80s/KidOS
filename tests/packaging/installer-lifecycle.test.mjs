import fs from 'node:fs';
import assert from 'node:assert/strict';

const hooks = fs.readFileSync('apps/shell/src-tauri/windows/hooks.nsh', 'utf8');
const stopScript = fs.readFileSync('scripts/windows/stop-kidos-services.ps1', 'utf8');
const extractor = fs.readFileSync('scripts/windows/extract-media-classifier.ps1', 'utf8');
const upgradeWorkflow = fs.readFileSync('.github/workflows/installer-upgrade-ci.yml', 'utf8');

const preStart = hooks.indexOf('!macro NSIS_HOOK_PREINSTALL');
const postStart = hooks.indexOf('!macro NSIS_HOOK_POSTINSTALL');
const guardianCopy = hooks.indexOf('File /oname=kidos-guardian-host.exe', postStart);
assert(preStart >= 0 && postStart > preStart && guardianCopy > postStart, 'installer hook markers are missing or out of order');

const preinstallBody = hooks.slice(preStart, postStart);
assert.match(preinstallBody, /stop-kidos-services\.ps1/, 'preinstall must stage the bounded service cleanup script');
assert.match(preinstallBody, /ExecToStack[\s\S]*stop-kidos-services\.ps1/, 'preinstall must execute service cleanup before postinstall file replacement');
assert.match(preinstallBody, /Abort|Quit/, 'preinstall must fail closed if existing protection cannot be stopped');

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
assert.match(uninstallBody, /ExecToStack[\s\S]*stop-kidos-services\.ps1/, 'uninstall must wait for service/process cleanup before file deletion');

const restoreVerifyIndex = uninstallBody.indexOf('verify-restore-result.ps1');
const stopCleanupIndex = uninstallBody.lastIndexOf('stop-kidos-services.ps1');
const classifierDeleteIndex = uninstallBody.indexOf('RMDir /r "$PROGRAMFILES64\\KidOS\\MediaClassifier"');
const recoveryDeleteIndex = uninstallBody.indexOf('RMDir /r "$PROGRAMFILES64\\KidOS\\Recovery"');
assert(restoreVerifyIndex >= 0, 'uninstall must verify Windows account recovery before deleting recovery payloads');
assert(stopCleanupIndex > restoreVerifyIndex, 'uninstall must stop KidOS services only after Windows account recovery is verified');
assert(classifierDeleteIndex > stopCleanupIndex, 'uninstall must delete the hook-created MediaClassifier payload after bounded service cleanup');
assert(recoveryDeleteIndex > stopCleanupIndex, 'uninstall must delete the hook-created Recovery payload after bounded service cleanup');

assert.match(hooks, /KidOSInstallerFailure\.txt/, 'fail-closed installer exits must persist a stage-specific diagnostic before rollback');
assert.match(hooks, /FileWrite/, 'installer failure diagnostics must write the failing stage/message to disk');
assert.match(upgradeWorkflow, /KidOSInstallerFailure\.txt/, 'installer upgrade CI must capture the persisted installer failure diagnostic');

assert.match(upgradeWorkflow, /Clean-host PowerShell 5\.1 service cleanup preflight/, 'installer upgrade CI must exercise service cleanup on a clean Windows host before packaging');
assert.match(upgradeWorkflow, /WindowsPowerShell[\\/]v1\.0[\\/]powershell\.exe[\s\S]*stop-kidos-services\.ps1/, 'clean-host cleanup preflight must use Windows PowerShell 5.1, matching the NSIS runtime');

console.log('KidOS installer lifecycle contract passed.');
