import assert from 'node:assert/strict';
import fs from 'node:fs';

const bootstrapPath = 'scripts/windows/bootstrap-visual-vm-runner.ps1';

assert.ok(fs.existsSync(bootstrapPath), 'visual VM runner bootstrap must exist');

const bootstrap = fs.readFileSync(bootstrapPath, 'utf8');

assert.match(bootstrap, /ValidateSet\(['"]Host['"],\s*['"]Guest['"]\)/, 'bootstrap must support explicit Host and Guest roles');
assert.match(bootstrap, /Parameter\(Mandatory\s*=\s*\$true\)[\s\S]*\$RegistrationToken/, 'bootstrap must require a temporary GitHub runner registration token');
assert.match(bootstrap, /kidos-interactive-test/, 'host runner must register the kidos-interactive-test label');
assert.match(bootstrap, /kidos-visual-vm-guest/, 'guest runner must register the kidos-visual-vm-guest label');
assert.match(bootstrap, /Get-VM\s+-Name\s+\$VMName/, 'host bootstrap must verify the disposable Hyper-V VM exists');
assert.match(bootstrap, /Get-VMSnapshot[\s\S]*\$SnapshotName/, 'host bootstrap must verify the clean Hyper-V snapshot exists');
assert.match(bootstrap, /SessionId[\s\S]*-eq\s+0/, 'guest bootstrap must reject Windows Session 0');
assert.match(bootstrap, /Get-Service\s+-Name\s+['"]KidOSGuardian['"]/, 'guest bootstrap must verify the KidOSGuardian service exists');
assert.match(bootstrap, /if\s*\(\$Role\s+-eq\s+['"]Host['"]\)[\s\S]*--runasservice/, 'only the Hyper-V host role should opt into Windows service mode');
assert.match(bootstrap, /if\s*\(\$Role\s+-eq\s+['"]Guest['"]\)[\s\S]*run\.cmd/, 'guest role must launch the runner interactively');
assert.match(bootstrap, /api\.github\.com\/repos\/actions\/runner\/releases\/latest/, 'bootstrap must resolve the official GitHub Actions runner release');
assert.doesNotMatch(bootstrap, /ghp_[A-Za-z0-9]+|github_pat_[A-Za-z0-9_]+/, 'bootstrap must not contain a hard-coded GitHub credential');

console.log('visual VM runner bootstrap contract: PASS');
