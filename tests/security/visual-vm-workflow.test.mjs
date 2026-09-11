import assert from 'node:assert/strict';
import fs from 'node:fs';

const workflowPath = '.github/workflows/interactive-windows-session.yml';
const workflow = fs.readFileSync(workflowPath, 'utf8');

assert.match(workflow, /vm_name:/, 'workflow must accept a VM name');
assert.match(workflow, /installer_path:/, 'workflow must accept an installer path');
assert.match(workflow, /tests\/windows\/visual_vm\/orchestrator\.py/, 'workflow must invoke the visual VM Python orchestrator');
assert.match(workflow, /tests\/windows\/visual_vm\/hyperv\.ps1/, 'workflow must invoke the Hyper-V adapter');
assert.match(workflow, /tests\/windows\/visual_vm\/guest-validate\.ps1/, 'workflow must invoke the Guardian guest validator');
assert.match(workflow, /tests\/windows\/visual_vm\/capture\.ps1/, 'workflow must invoke visual capture');
assert.match(workflow, /healthy-home\.png/, 'workflow must require the healthy KidOS screenshot');
assert.match(workflow, /restricted-safe-mode\.png/, 'workflow must require the Restricted Safe Mode screenshot');
assert.match(workflow, /visual-vm-result\.json/, 'workflow must emit the canonical result manifest');
assert.match(workflow, /KidOS-Visual-Windows-VM-Validation/, 'workflow must upload the visual VM artifact bundle');
assert.match(workflow, /if-no-files-found:\s*error/, 'workflow must fail when required evidence is missing');

console.log('visual VM workflow contract: PASS');
