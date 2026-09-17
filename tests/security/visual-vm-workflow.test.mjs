import assert from 'node:assert/strict';
import fs from 'node:fs';

const workflowPath = '.github/workflows/interactive-windows-session.yml';
const workflow = fs.readFileSync(workflowPath, 'utf8');

assert.match(workflow, /push:\s*\n\s*branches:\s*\n\s*- feat\/visual-windows-vm-validation/, 'workflow must self-trigger only on the visual validation branch');
assert.match(workflow, /inputs\.vm_name \|\| 'KidOS-Visual-Test'/, 'push runs must fall back to the default VM name');
assert.match(workflow, /inputs\.snapshot_name \|\| 'KidOS-Clean'/, 'push runs must fall back to the clean snapshot');
assert.match(workflow, /inputs\.expected_kidos_exe \|\| 'KidOS.exe'/, 'push runs must fall back to the KidOS shell executable');
assert.match(workflow, /vm_name:/, 'workflow must accept a VM name');
assert.match(workflow, /installer_path:/, 'workflow must accept an installer path');
assert.match(workflow, /prepare-hyperv-vm:/, 'workflow must have a Hyper-V host preparation job');
assert.match(workflow, /kidos-interactive-test/, 'host preparation must use the Hyper-V test host runner');
assert.match(workflow, /visual-guest-validation:/, 'workflow must have a dedicated VM guest validation job');
assert.match(workflow, /kidos-visual-vm-guest/, 'visual validation must execute inside the Windows VM runner');
assert.match(workflow, /needs:\s*prepare-hyperv-vm/, 'guest validation must wait for the Hyper-V host job');
assert.match(workflow, /tests\/windows\/visual_vm\/orchestrator\.py/, 'workflow must invoke the visual VM Python orchestrator');
assert.match(workflow, /tests\/windows\/visual_vm\/hyperv\.ps1/, 'workflow must invoke the Hyper-V adapter');
assert.match(workflow, /tests\/windows\/visual_vm\/guest-validate\.ps1/, 'workflow must invoke the Guardian guest validator');
assert.match(workflow, /tests\/windows\/visual_vm\/capture\.ps1/, 'workflow must invoke visual capture');
assert.match(workflow, /Stop-Service\s+KidOSGuardian/, 'workflow must intentionally exercise fail-closed mode in the disposable VM');
assert.match(workflow, /Start-Service\s+KidOSGuardian/, 'workflow must restore Guardian after the fail-closed test');
assert.match(workflow, /healthy-home\.png/, 'workflow must require the healthy KidOS screenshot');
assert.match(workflow, /restricted-safe-mode\.png/, 'workflow must require the Restricted Safe Mode screenshot');
assert.match(workflow, /visual-vm-result\.json/, 'workflow must emit the canonical result manifest');
assert.match(workflow, /KidOS-Visual-Windows-VM-Validation/, 'workflow must upload the visual VM artifact bundle');
assert.match(workflow, /if-no-files-found:\s*error/, 'workflow must fail when required evidence is missing');

console.log('visual VM workflow contract: PASS');
