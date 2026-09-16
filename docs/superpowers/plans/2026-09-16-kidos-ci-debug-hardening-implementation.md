# KidOS CI/CD Debug Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add cloud-runnable KidOS CI diagnostics, fail-closed fault injection, bounded flake detection, post-merge smoke validation, and exact-SHA evidence without making the offline Hyper-V runner block ordinary PR validation.

**Architecture:** Reuse the existing Windows workflows and test conventions instead of replacing them. Add small reusable scripts with static contract tests, then compose them from focused GitHub Actions workflows; the exact-SHA evidence layer treats Hyper-V visual validation as an explicit `not_run`/`passed`/`failed` state and never converts an offline runner into a successful release gate.

**Tech Stack:** GitHub Actions, Windows PowerShell 5.1/PowerShell 7, Node.js 22, Vitest, pnpm 10.15.1, Rust/Cargo, existing KidOS Guardian/Media Classifier services.

**Spec:** `docs/superpowers/specs/2026-09-16-kidos-ci-debug-hardening-design.md`

## Global Constraints

- Extend existing KidOS CodeQL, dependency security, installer upgrade, Windows hardening, Guardian, media safety, Shadow Validation, clean Windows VM, and visual Windows VM coverage; do not replace them.
- Do not add new programming languages or CI vendors merely for complexity.
- Do not make the offline self-hosted Hyper-V workflow a prerequisite for ordinary cloud PR validation.
- Shadow and test harnesses must never control a production child environment.
- Restricted Safe Mode remains fail-closed; tests must not weaken it to pass.
- No production secrets, registration tokens, credentials, cookies, or user data may be written to CI artifacts.
- Fault injection runs only in disposable CI/test environments and restores changed services/files in cleanup logic.
- `cloud_ci_ready` may be true without Hyper-V visual validation; `release_ready` may be true only when the exact SHA has all required cloud gates plus `hyperv_visual_validation: "passed"`.
- Missing Hyper-V evidence is `not_run`, never success.
- Production/release automation must never silently substitute a different SHA.
- Mutation testing is deferred to a later scheduled/nightly milestone.

---

## File Map

### New files

- `scripts/windows/collect-ci-diagnostics.ps1` — best-effort, redacted Windows/KidOS diagnostic bundle collector.
- `tests/security/ci-diagnostics-contract.test.mjs` — static contract for diagnostics content, redaction, and Shadow workflow integration.
- `tests/windows/fault-injection.ps1` — controlled Guardian/classifier failure scenarios with guaranteed service restoration and machine-readable output.
- `tests/security/fault-injection-contract.test.mjs` — static safety contract for fault injection and its workflow.
- `.github/workflows/fault-injection-ci.yml` — GitHub-hosted Windows install + fault-injection validation.
- `scripts/ci/repeat-command.mjs` — bounded deterministic repetition runner that records failing iteration and command.
- `tests/security/repeat-command.test.mjs` — executable tests for repeated success and first-failure capture.
- `.github/workflows/flake-check-ci.yml` — ten-run lightweight Guardian/shell contract repetition job.
- `.github/workflows/post-merge-smoke.yml` — exact-main-SHA post-merge Windows smoke validation and evidence upload.
- `scripts/ci/build-release-evidence.mjs` — converts GitHub check-runs into a strict exact-SHA readiness manifest.
- `tests/security/release-evidence.test.mjs` — executable readiness/SHA/Hype-V-status tests.
- `.github/workflows/release-evidence.yml` — queries checks for one exact SHA and uploads the canonical evidence manifest.

### Existing files modified

- `.github/workflows/shadow-validation-ci.yml` — replace duplicated inline Windows evidence collection with the reusable diagnostics collector.
- `.github/workflows/ci.yml` — add the new static CI-hardening contract tests to the existing production-hardening verification step.
- `README.md` — document CI debugging artifacts, readiness meanings, and the intentionally separate Hyper-V release-confidence layer.

### Deliberately separate follow-up

The full PR #37 visual-Hyper-V workflow is still on `feat/visual-windows-vm-validation` and depends on physical host/guest runners. Do not copy or reimplement that unmerged subsystem in this branch. After PR #37 is validated and merged, create a small follow-up plan to call `collect-ci-diagnostics.ps1` from that workflow and feed its result into the same evidence convention. This keeps two active feature branches from silently diverging on the visual VM implementation.

---

### Task 1: Reusable Windows CI diagnostics

**Files:**
- Create: `scripts/windows/collect-ci-diagnostics.ps1`
- Create: `tests/security/ci-diagnostics-contract.test.mjs`
- Modify: `.github/workflows/shadow-validation-ci.yml`
- Modify: `.github/workflows/ci.yml`

**Interfaces:**
- Consumes: GitHub Actions environment variables plus installed KidOS service/file state.
- Produces: `metadata.json`, `services.txt`, `processes.txt`, `recovery-task.txt`, `service-control-manager-events.txt`, `application-events.txt`, `installed-files.txt`, copied sanitized ProgramData diagnostics, and `summary.json` under a caller-selected directory.
- PowerShell entry point: `collect-ci-diagnostics.ps1 -ArtifactDir <path> -Stage <name> -FailureClass <name>`.

- [ ] **Step 1: Write the failing static contract test**

Create `tests/security/ci-diagnostics-contract.test.mjs`:

```js
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
```

- [ ] **Step 2: Run the contract test and verify RED**

Run:

```bash
node tests/security/ci-diagnostics-contract.test.mjs
```

Expected: FAIL because `scripts/windows/collect-ci-diagnostics.ps1` does not exist yet and Shadow still contains inline collection logic.

- [ ] **Step 3: Implement the diagnostics collector**

Create `scripts/windows/collect-ci-diagnostics.ps1` with this structure:

```powershell
[CmdletBinding()]
param(
  [string]$ArtifactDir = "target/ci-diagnostics",
  [string]$Stage = "unknown",
  [string]$FailureClass = "unknown"
)

$ErrorActionPreference = "Continue"
New-Item -ItemType Directory -Force -Path $ArtifactDir | Out-Null

function Redact-Text([string]$Text) {
  if ($null -eq $Text) { return "" }
  $patterns = @(
    '(?i)(authorization\s*[:=]\s*)([^\s]+)',
    '(?i)(cookie\s*[:=]\s*)([^\r\n]+)',
    '(?i)(password\s*[:=]\s*)([^\s]+)',
    '(?i)(token\s*[:=]\s*)([^\s]+)'
  )
  $redacted = $Text
  foreach ($pattern in $patterns) {
    $redacted = [regex]::Replace($redacted, $pattern, '$1[REDACTED]')
  }
  return $redacted
}

function Write-RedactedFile([string]$Path, [object]$Value) {
  $text = if ($Value -is [string]) { $Value } else { $Value | Out-String }
  Redact-Text $text | Set-Content -Path $Path -Encoding UTF8
}

$os = Get-CimInstance Win32_OperatingSystem -ErrorAction SilentlyContinue
$metadata = [ordered]@{
  commit_sha = $env:GITHUB_SHA
  ref = $env:GITHUB_REF
  workflow = $env:GITHUB_WORKFLOW
  run_id = $env:GITHUB_RUN_ID
  run_attempt = $env:GITHUB_RUN_ATTEMPT
  job = $env:GITHUB_JOB
  stage = $Stage
  failure_class = $FailureClass
  windows_caption = $os.Caption
  windows_version = $os.Version
  generated_at = (Get-Date).ToUniversalTime().ToString('o')
}
$metadata | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $ArtifactDir 'metadata.json') -Encoding UTF8

$services = Get-CimInstance Win32_Service -ErrorAction SilentlyContinue |
  Where-Object { $_.Name -in @('KidOSGuardian', 'KidOSMediaClassifier') } |
  Select-Object Name, State, StartMode, StartName, ProcessId
Write-RedactedFile (Join-Path $ArtifactDir 'services.txt') $services

$processes = Get-Process -ErrorAction SilentlyContinue |
  Where-Object { $_.ProcessName -match 'KidOS|Guardian|Classifier' } |
  Select-Object ProcessName, Id, StartTime
Write-RedactedFile (Join-Path $ArtifactDir 'processes.txt') $processes

$task = Get-ScheduledTask -TaskName 'KidOS Guardian Recovery' -ErrorAction SilentlyContinue |
  Select-Object TaskName, State, TaskPath
Write-RedactedFile (Join-Path $ArtifactDir 'recovery-task.txt') $task

$since = (Get-Date).AddMinutes(-30)
$scm = Get-WinEvent -FilterHashtable @{ LogName='System'; ProviderName='Service Control Manager'; StartTime=$since } -ErrorAction SilentlyContinue |
  Where-Object { $_.Message -match 'KidOSGuardian|KidOSMediaClassifier' } |
  Select-Object TimeCreated, Id, LevelDisplayName, Message
Write-RedactedFile (Join-Path $ArtifactDir 'service-control-manager-events.txt') $scm

$app = Get-WinEvent -FilterHashtable @{ LogName='Application'; StartTime=$since } -ErrorAction SilentlyContinue |
  Where-Object { $_.Message -match 'KidOS|guardian|classifier' } |
  Select-Object TimeCreated, ProviderName, Id, LevelDisplayName, Message
Write-RedactedFile (Join-Path $ArtifactDir 'application-events.txt') $app

$installed = Get-ChildItem "$env:ProgramFiles\KidOS" -Recurse -ErrorAction SilentlyContinue |
  Select-Object FullName, Length, LastWriteTime
Write-RedactedFile (Join-Path $ArtifactDir 'installed-files.txt') $installed

$sourceDiagnostics = Join-Path $env:ProgramData 'KidOS\Diagnostics'
$copied = @()
if (Test-Path $sourceDiagnostics) {
  $dest = Join-Path $ArtifactDir 'programdata-diagnostics'
  New-Item -ItemType Directory -Force -Path $dest | Out-Null
  Get-ChildItem $sourceDiagnostics -File -ErrorAction SilentlyContinue | ForEach-Object {
    $target = Join-Path $dest $_.Name
    Write-RedactedFile $target (Get-Content $_.FullName -Raw -ErrorAction SilentlyContinue)
    $copied += $_.Name
  }
}

$summary = [ordered]@{
  metadata_present = Test-Path (Join-Path $ArtifactDir 'metadata.json')
  guardian_present = [bool](Get-Service KidOSGuardian -ErrorAction SilentlyContinue)
  classifier_present = [bool](Get-Service KidOSMediaClassifier -ErrorAction SilentlyContinue)
  programdata_files = $copied
  generated_at = (Get-Date).ToUniversalTime().ToString('o')
}
$summary | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $ArtifactDir 'summary.json') -Encoding UTF8
```

- [ ] **Step 4: Replace Shadow’s duplicate evidence step with the collector**

In `.github/workflows/shadow-validation-ci.yml`, replace the current `Capture Shadow install diagnostics` PowerShell body with:

```yaml
      - name: Capture Shadow install diagnostics
        if: always()
        shell: pwsh
        run: ./scripts/windows/collect-ci-diagnostics.ps1 -ArtifactDir target/shadow-results/install-diagnostics -Stage shadow-validation -FailureClass runtime-or-install
```

Keep the existing `Upload Shadow report` step unchanged so `target/shadow-results/**` still captures the collector output.

- [ ] **Step 5: Add the new contract test to the existing hardening verification**

In `.github/workflows/ci.yml`, extend `Verify production hardening invariants` to:

```yaml
      - name: Verify production hardening invariants
        run: |
          node tests/security/release-hardening.test.mjs
          node tests/security/windows-code-signing-gate.test.mjs
          node tests/security/ci-diagnostics-contract.test.mjs
```

- [ ] **Step 6: Verify GREEN**

Run:

```bash
node tests/security/ci-diagnostics-contract.test.mjs
```

Expected: `KidOS CI diagnostics contract checks passed.` and exit code 0.

Then run the existing static hardening suite:

```bash
node tests/security/release-hardening.test.mjs
node tests/security/windows-code-signing-gate.test.mjs
```

Expected: all exit 0.

- [ ] **Step 7: Commit Task 1**

```bash
git add scripts/windows/collect-ci-diagnostics.ps1 tests/security/ci-diagnostics-contract.test.mjs .github/workflows/shadow-validation-ci.yml .github/workflows/ci.yml
git commit -m "ci: add reusable Windows diagnostics collection"
```

---

### Task 2: Fail-closed Windows fault injection

**Files:**
- Create: `tests/windows/fault-injection.ps1`
- Create: `tests/security/fault-injection-contract.test.mjs`
- Create: `.github/workflows/fault-injection-ci.yml`
- Modify: `.github/workflows/ci.yml`

**Interfaces:**
- Consumes: installed `KidOSGuardian` and `KidOSMediaClassifier`, the existing shell `system-status` tests, and the Task 1 diagnostics collector.
- Produces: `target/fault-injection/fault-injection.json` with per-scenario `name`, `passed`, `observed`, and `completed_at` fields.
- PowerShell entry point: `fault-injection.ps1 -ArtifactDir <path>`.

- [ ] **Step 1: Write the failing fault-injection contract test**

Create `tests/security/fault-injection-contract.test.mjs`:

```js
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
```

- [ ] **Step 2: Run and verify RED**

```bash
node tests/security/fault-injection-contract.test.mjs
```

Expected: FAIL because the harness/workflow do not exist.

- [ ] **Step 3: Implement the Windows fault harness**

Create `tests/windows/fault-injection.ps1`:

```powershell
[CmdletBinding()]
param([string]$ArtifactDir = 'target/fault-injection')

$ErrorActionPreference = 'Stop'
New-Item -ItemType Directory -Force -Path $ArtifactDir | Out-Null

function Assert-True([bool]$Condition, [string]$Message) {
  if (-not $Condition) { throw $Message }
}

$results = @()

function Add-Result([string]$Name, [bool]$Passed, [hashtable]$Observed) {
  $script:results += [ordered]@{
    name = $Name
    passed = $Passed
    observed = $Observed
    completed_at = (Get-Date).ToUniversalTime().ToString('o')
  }
}

try {
  $guardianBefore = Get-Service KidOSGuardian -ErrorAction Stop
  Stop-Service KidOSGuardian -Force -ErrorAction Stop
  Start-Sleep -Seconds 2
  $guardianAfter = Get-Service KidOSGuardian -ErrorAction Stop
  Assert-True ($guardianAfter.Status -ne 'Running') 'Guardian remained running after injected stop.'
  Add-Result 'guardian_stopped' $true @{ service_status = $guardianAfter.Status.ToString(); fail_closed_expected = $true }
}
finally {
  Start-Service KidOSGuardian -ErrorAction SilentlyContinue
  (Get-Service KidOSGuardian -ErrorAction Stop).WaitForStatus('Running', [TimeSpan]::FromSeconds(20))
}

try {
  $classifierBefore = Get-Service KidOSMediaClassifier -ErrorAction Stop
  Stop-Service KidOSMediaClassifier -Force -ErrorAction Stop
  Start-Sleep -Seconds 2
  $classifierAfter = Get-Service KidOSMediaClassifier -ErrorAction Stop
  Assert-True ($classifierAfter.Status -ne 'Running') 'Classifier remained running after injected stop.'
  Add-Result 'classifier_stopped' $true @{ service_status = $classifierAfter.Status.ToString(); media_ready_expected = $false }
}
finally {
  Start-Service KidOSMediaClassifier -ErrorAction SilentlyContinue
  (Get-Service KidOSMediaClassifier -ErrorAction Stop).WaitForStatus('Running', [TimeSpan]::FromSeconds(20))
}

$finalGuardian = Get-Service KidOSGuardian -ErrorAction Stop
$finalClassifier = Get-Service KidOSMediaClassifier -ErrorAction Stop
Assert-True ($finalGuardian.Status -eq 'Running') 'Guardian was not restored after fault injection.'
Assert-True ($finalClassifier.Status -eq 'Running') 'Classifier was not restored after fault injection.'

$output = [ordered]@{
  passed = ($results | Where-Object { -not $_.passed }).Count -eq 0
  scenarios = $results
  restored = @{ guardian = $finalGuardian.Status.ToString(); classifier = $finalClassifier.Status.ToString() }
  commit_sha = $env:GITHUB_SHA
}
$output | ConvertTo-Json -Depth 8 | Set-Content (Join-Path $ArtifactDir 'fault-injection.json') -Encoding UTF8
$output | ConvertTo-Json -Depth 8
```

This runtime harness deliberately verifies the real service failure/restoration. The authoritative UI safety mapping remains covered by the existing `apps/shell/src/features/home/system-status.test.ts`, which is run in the same workflow so a stopped/unreachable Guardian can never be mapped to `active`.

- [ ] **Step 4: Create the cloud Windows fault-injection workflow**

Create `.github/workflows/fault-injection-ci.yml`:

```yaml
name: KidOS Fault Injection CI

on:
  pull_request:
    paths:
      - 'apps/shell/**'
      - 'crates/**'
      - 'services/**'
      - 'scripts/windows/**'
      - 'tests/windows/**'
      - 'tests/security/**'
      - '.github/workflows/fault-injection-ci.yml'
  push:
    branches: [main]
    paths:
      - 'apps/shell/**'
      - 'crates/**'
      - 'services/**'
      - 'scripts/windows/**'
      - 'tests/windows/**'
      - 'tests/security/**'
      - '.github/workflows/fault-injection-ci.yml'
  workflow_dispatch:

jobs:
  fault-injection:
    runs-on: windows-latest
    timeout-minutes: 90
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 22
      - uses: dtolnay/rust-toolchain@stable
      - uses: actions/setup-python@v5
        with:
          python-version: '3.11'
      - name: Install dependencies
        run: pnpm install --no-frozen-lockfile
      - name: Verify fail-closed shell mapping
        run: pnpm --filter @kidos/shell test --run src/features/home/system-status.test.ts
      - name: Package classifier
        shell: pwsh
        run: ./scripts/windows/package-media-classifier.ps1
      - name: Build Guardian
        run: cargo build -p kidos-guardian-host --release
      - name: Build installer
        run: pnpm --filter @kidos/shell tauri build --bundles nsis
      - name: Install KidOS
        shell: pwsh
        run: |
          $installer = (Get-ChildItem target/release/bundle/nsis/*-setup.exe | Select-Object -First 1).FullName
          $p = Start-Process $installer -ArgumentList '/S' -PassThru -Wait
          if ($p.ExitCode -ne 0) { throw "KidOS installer failed with exit code $($p.ExitCode)." }
      - name: Run controlled fault injection
        shell: pwsh
        run: ./tests/windows/fault-injection.ps1 -ArtifactDir target/fault-injection
      - name: Capture diagnostics
        if: always()
        shell: pwsh
        run: ./scripts/windows/collect-ci-diagnostics.ps1 -ArtifactDir target/fault-injection/diagnostics -Stage fault-injection -FailureClass safety-runtime
      - name: Upload fault evidence
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: KidOS-Fault-Injection-${{ github.sha }}
          path: target/fault-injection/**
          if-no-files-found: error
```

- [ ] **Step 5: Register the static contract in release CI**

Append to `.github/workflows/ci.yml` hardening command block:

```yaml
          node tests/security/fault-injection-contract.test.mjs
```

- [ ] **Step 6: Verify GREEN locally for static + shell contracts**

```bash
node tests/security/fault-injection-contract.test.mjs
pnpm --filter @kidos/shell test --run src/features/home/system-status.test.ts
```

Expected: both exit 0.

The actual PowerShell service injection is verified by the GitHub-hosted Windows workflow after push.

- [ ] **Step 7: Commit Task 2**

```bash
git add tests/windows/fault-injection.ps1 tests/security/fault-injection-contract.test.mjs .github/workflows/fault-injection-ci.yml .github/workflows/ci.yml
git commit -m "ci: add fail-closed Windows fault injection"
```

---

### Task 3: Bounded flake detection

**Files:**
- Create: `scripts/ci/repeat-command.mjs`
- Create: `tests/security/repeat-command.test.mjs`
- Create: `.github/workflows/flake-check-ci.yml`
- Modify: `.github/workflows/ci.yml`

**Interfaces:**
- Consumes: `--count`, `--artifact`, followed by a command and its arguments.
- Produces: JSON artifact `{ passed, count, completed, failed_iteration, command, exit_code }`.
- Fails immediately on the first non-zero child exit code.

- [ ] **Step 1: Write executable tests first**

Create `tests/security/repeat-command.test.mjs`:

```js
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
```

- [ ] **Step 2: Run and verify RED**

```bash
node tests/security/repeat-command.test.mjs
```

Expected: FAIL because `scripts/ci/repeat-command.mjs` does not exist.

- [ ] **Step 3: Implement the minimal repetition runner**

Create `scripts/ci/repeat-command.mjs`:

```js
import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname } from 'node:path';
import { spawnSync } from 'node:child_process';

const args = process.argv.slice(2);
function take(flag) {
  const i = args.indexOf(flag);
  if (i < 0 || i + 1 >= args.length) throw new Error(`Missing ${flag}`);
  const value = args[i + 1];
  args.splice(i, 2);
  return value;
}

const count = Number.parseInt(take('--count'), 10);
const artifact = take('--artifact');
if (!Number.isInteger(count) || count < 1 || count > 100) throw new Error('--count must be 1..100');
if (args.length === 0) throw new Error('A command is required');
const [command, ...commandArgs] = args;

let completed = 0;
let failedIteration = null;
let exitCode = 0;
for (let iteration = 1; iteration <= count; iteration += 1) {
  const result = spawnSync(command, commandArgs, { stdio: 'inherit', shell: false });
  completed = iteration;
  const status = result.status ?? 1;
  if (status !== 0) {
    failedIteration = iteration;
    exitCode = status;
    break;
  }
}

const output = {
  passed: failedIteration === null,
  count,
  completed,
  failed_iteration: failedIteration,
  command: [command, ...commandArgs],
  exit_code: exitCode,
};
mkdirSync(dirname(artifact), { recursive: true });
writeFileSync(artifact, `${JSON.stringify(output, null, 2)}\n`, 'utf8');
process.exit(exitCode);
```

- [ ] **Step 4: Verify GREEN**

```bash
node tests/security/repeat-command.test.mjs
```

Expected: `KidOS repeat-command tests passed.`

- [ ] **Step 5: Add the ten-run contract flake workflow**

Create `.github/workflows/flake-check-ci.yml`:

```yaml
name: KidOS Critical Flake Check

on:
  pull_request:
    paths:
      - 'apps/shell/src/features/home/**'
      - 'crates/guardian-host/**'
      - 'scripts/ci/**'
      - 'tests/security/**'
      - '.github/workflows/flake-check-ci.yml'
  workflow_dispatch:

jobs:
  critical-contract-repeat:
    runs-on: ubuntu-latest
    timeout-minutes: 30
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 22
      - name: Install dependencies
        run: pnpm install --no-frozen-lockfile
      - name: Repeat shell fail-closed contract 10 times
        run: >-
          node scripts/ci/repeat-command.mjs
          --count 10
          --artifact target/flake-results/system-status.json
          pnpm --filter @kidos/shell test --run src/features/home/system-status.test.ts
      - name: Upload flake evidence
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: KidOS-Flake-Check-${{ github.sha }}
          path: target/flake-results/**
          if-no-files-found: error
```

Do not repeat the full installer/Windows pipeline in this PR workflow; that remains a later scheduled extension after the lightweight repetition job is stable.

- [ ] **Step 6: Register repeat-runner tests in release CI**

Append to `.github/workflows/ci.yml` hardening command block:

```yaml
          node tests/security/repeat-command.test.mjs
```

- [ ] **Step 7: Commit Task 3**

```bash
git add scripts/ci/repeat-command.mjs tests/security/repeat-command.test.mjs .github/workflows/flake-check-ci.yml .github/workflows/ci.yml
git commit -m "ci: add bounded critical flake detection"
```

---

### Task 4: Exact-main-SHA post-merge smoke validation

**Files:**
- Create: `.github/workflows/post-merge-smoke.yml`
- Create: `tests/security/post-merge-workflow-contract.test.mjs`
- Modify: `.github/workflows/ci.yml`

**Interfaces:**
- Consumes: a `push` to `main` and the exact `${{ github.sha }}` checked out by Actions.
- Produces: `target/post-merge/post-merge-evidence.json` and Task 1 diagnostics on failure.

- [ ] **Step 1: Write the failing workflow contract test**

Create `tests/security/post-merge-workflow-contract.test.mjs`:

```js
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
```

- [ ] **Step 2: Run and verify RED**

```bash
node tests/security/post-merge-workflow-contract.test.mjs
```

Expected: FAIL because the workflow does not exist.

- [ ] **Step 3: Create the post-merge workflow**

Create `.github/workflows/post-merge-smoke.yml`:

```yaml
name: KidOS Post-Merge Smoke

on:
  push:
    branches: [main]
    paths:
      - 'apps/shell/**'
      - 'crates/**'
      - 'services/**'
      - 'scripts/windows/**'
      - 'tests/windows/**'
      - '.github/workflows/**'

permissions:
  contents: read

jobs:
  exact-main-sha-smoke:
    runs-on: windows-latest
    timeout-minutes: 90
    steps:
      - uses: actions/checkout@v4
        with:
          ref: ${{ github.sha }}
      - uses: pnpm/action-setup@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 22
      - uses: dtolnay/rust-toolchain@stable
      - uses: actions/setup-python@v5
        with:
          python-version: '3.11'
      - name: Install dependencies
        run: pnpm install --no-frozen-lockfile
      - name: Run exact-SHA contract smoke
        run: |
          pnpm --filter @kidos/shell test --run src/features/home/system-status.test.ts
          node tests/security/release-hardening.test.mjs
      - name: Package classifier
        shell: pwsh
        run: ./scripts/windows/package-media-classifier.ps1
      - name: Build Guardian
        run: cargo build -p kidos-guardian-host --release
      - name: Build installer
        run: pnpm --filter @kidos/shell tauri build --bundles nsis
      - name: Install KidOS
        shell: pwsh
        run: |
          $installer = (Get-ChildItem target/release/bundle/nsis/*-setup.exe | Select-Object -First 1).FullName
          $p = Start-Process $installer -ArgumentList '/S' -PassThru -Wait
          if ($p.ExitCode -ne 0) { throw "KidOS installer failed with exit code $($p.ExitCode)." }
      - name: Verify installed security state
        shell: pwsh
        run: ./tests/windows/security-regression.ps1 | Tee-Object -FilePath target/post-merge-security.txt
      - name: Exercise fail-closed runtime smoke
        shell: pwsh
        run: ./tests/windows/fault-injection.ps1 -ArtifactDir target/post-merge/fault-injection
      - name: Write exact-SHA post-merge evidence
        if: success()
        shell: pwsh
        run: |
          New-Item -ItemType Directory -Force -Path target/post-merge | Out-Null
          [ordered]@{
            commit_sha = '${{ github.sha }}'
            ref = '${{ github.ref }}'
            post_merge_smoke = 'passed'
            generated_at = (Get-Date).ToUniversalTime().ToString('o')
          } | ConvertTo-Json | Set-Content target/post-merge/post-merge-evidence.json -Encoding UTF8
      - name: Capture post-merge diagnostics
        if: always()
        shell: pwsh
        run: ./scripts/windows/collect-ci-diagnostics.ps1 -ArtifactDir target/post-merge/diagnostics -Stage post-merge-smoke -FailureClass runtime-or-install
      - name: Upload post-merge evidence
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: KidOS-Post-Merge-${{ github.sha }}
          path: target/post-merge/**
          if-no-files-found: warn
```

- [ ] **Step 4: Verify the static workflow contract GREEN**

```bash
node tests/security/post-merge-workflow-contract.test.mjs
```

Expected: pass.

- [ ] **Step 5: Register the contract test in release CI**

Append:

```yaml
          node tests/security/post-merge-workflow-contract.test.mjs
```

to the existing `Verify production hardening invariants` block in `.github/workflows/ci.yml`.

- [ ] **Step 6: Commit Task 4**

```bash
git add .github/workflows/post-merge-smoke.yml tests/security/post-merge-workflow-contract.test.mjs .github/workflows/ci.yml
git commit -m "ci: add exact-SHA post-merge smoke validation"
```

---

### Task 5: Exact-SHA cloud/release evidence manifest

**Files:**
- Create: `scripts/ci/build-release-evidence.mjs`
- Create: `tests/security/release-evidence.test.mjs`
- Create: `.github/workflows/release-evidence.yml`
- Modify: `.github/workflows/ci.yml`

**Interfaces:**
- `build-release-evidence.mjs --sha <sha> --checks <check-runs.json> --output <manifest.json>`.
- Output fields: `commit_sha`, `ref`, `generated_at`, `codeql`, `dependency_security`, `unit_tests`, `guardian_contract`, `installer_upgrade`, `fault_injection`, `flake_check`, `post_merge_smoke`, `hyperv_visual_validation`, `cloud_ci_ready`, `release_ready`.
- Status values are only `passed`, `failed`, `pending`, or `not_run`.
- `release_ready = cloud_ci_ready && hyperv_visual_validation === 'passed'`.

- [ ] **Step 1: Write executable evidence tests first**

Create `tests/security/release-evidence.test.mjs`:

```js
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
```

- [ ] **Step 2: Run and verify RED**

```bash
node tests/security/release-evidence.test.mjs
```

Expected: FAIL because the evidence builder does not exist.

- [ ] **Step 3: Implement exact-SHA evidence generation**

Create `scripts/ci/build-release-evidence.mjs`:

```js
import { readFileSync, mkdirSync, writeFileSync } from 'node:fs';
import { dirname } from 'node:path';

const args = process.argv.slice(2);
function take(flag) {
  const i = args.indexOf(flag);
  if (i < 0 || i + 1 >= args.length) throw new Error(`Missing ${flag}`);
  return args[i + 1];
}
const sha = take('--sha');
const checksPath = take('--checks');
const outputPath = take('--output');
const payload = JSON.parse(readFileSync(checksPath, 'utf8'));
const checks = payload.check_runs ?? [];

for (const check of checks) {
  if (check.head_sha && check.head_sha !== sha) {
    throw new Error(`Check ${check.name} belongs to ${check.head_sha}, not ${sha}`);
  }
}

function statusFor(pattern) {
  const matching = checks.filter((check) => pattern.test(check.name));
  if (matching.length === 0) return 'not_run';
  if (matching.some((check) => check.status !== 'completed')) return 'pending';
  if (matching.some((check) => check.conclusion !== 'success' && check.conclusion !== 'skipped')) return 'failed';
  return matching.some((check) => check.conclusion === 'success') ? 'passed' : 'not_run';
}

const manifest = {
  commit_sha: sha,
  ref: process.env.GITHUB_REF ?? '',
  generated_at: new Date().toISOString(),
  codeql: statusFor(/CodeQL/i),
  dependency_security: statusFor(/dependency-security|Dependency Security/i),
  unit_tests: statusFor(/windows-release|verify-shell|Run JavaScript tests/i),
  guardian_contract: statusFor(/windows-release|guardian/i),
  installer_upgrade: statusFor(/installer-upgrade|Installer Upgrade/i),
  fault_injection: statusFor(/fault-injection/i),
  flake_check: statusFor(/critical-contract-repeat|Flake Check/i),
  post_merge_smoke: statusFor(/exact-main-sha-smoke|Post-Merge Smoke/i),
  hyperv_visual_validation: statusFor(/visual-guest-validation|Interactive Windows Session/i),
};

const cloudRequired = [
  manifest.codeql,
  manifest.dependency_security,
  manifest.unit_tests,
  manifest.guardian_contract,
  manifest.installer_upgrade,
  manifest.fault_injection,
  manifest.flake_check,
  manifest.post_merge_smoke,
];
manifest.cloud_ci_ready = cloudRequired.every((status) => status === 'passed');
manifest.release_ready = manifest.cloud_ci_ready && manifest.hyperv_visual_validation === 'passed';

mkdirSync(dirname(outputPath), { recursive: true });
writeFileSync(outputPath, `${JSON.stringify(manifest, null, 2)}\n`, 'utf8');
```

- [ ] **Step 4: Verify GREEN**

```bash
node tests/security/release-evidence.test.mjs
```

Expected: `KidOS release evidence tests passed.`

- [ ] **Step 5: Create the evidence workflow**

Create `.github/workflows/release-evidence.yml`:

```yaml
name: KidOS Exact-SHA Release Evidence

on:
  workflow_run:
    workflows: ['KidOS Post-Merge Smoke']
    types: [completed]
  workflow_dispatch:
    inputs:
      commit_sha:
        description: 'Exact KidOS commit SHA to evaluate'
        required: true

permissions:
  actions: read
  checks: read
  contents: read

jobs:
  build-evidence:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          ref: ${{ github.event.workflow_run.head_sha || inputs.commit_sha }}
      - uses: actions/setup-node@v4
        with:
          node-version: 22
      - name: Resolve exact SHA
        id: sha
        shell: bash
        run: echo "value=${{ github.event.workflow_run.head_sha || inputs.commit_sha }}" >> "$GITHUB_OUTPUT"
      - name: Fetch checks for exact SHA
        env:
          GH_TOKEN: ${{ github.token }}
          SHA: ${{ steps.sha.outputs.value }}
        shell: bash
        run: |
          mkdir -p target/release-evidence
          gh api -H 'Accept: application/vnd.github+json' \
            "/repos/${{ github.repository }}/commits/$SHA/check-runs?per_page=100" \
            > target/release-evidence/check-runs.json
      - name: Build exact-SHA evidence
        env:
          SHA: ${{ steps.sha.outputs.value }}
        run: >-
          node scripts/ci/build-release-evidence.mjs
          --sha "$SHA"
          --checks target/release-evidence/check-runs.json
          --output target/release-evidence/release-evidence.json
      - name: Upload release evidence
        uses: actions/upload-artifact@v4
        with:
          name: KidOS-Release-Evidence-${{ steps.sha.outputs.value }}
          path: target/release-evidence/**
          if-no-files-found: error
```

- [ ] **Step 6: Register the evidence tests in release CI**

Append:

```yaml
          node tests/security/release-evidence.test.mjs
```

to `.github/workflows/ci.yml`’s production-hardening verification block.

- [ ] **Step 7: Commit Task 5**

```bash
git add scripts/ci/build-release-evidence.mjs tests/security/release-evidence.test.mjs .github/workflows/release-evidence.yml .github/workflows/ci.yml
git commit -m "ci: add exact-SHA release evidence"
```

---

### Task 6: Documentation and full cloud verification

**Files:**
- Modify: `README.md`

**Interfaces:**
- Documents artifact names, `cloud_ci_ready` vs `release_ready`, local verification commands, and the separate Hyper-V layer.

- [ ] **Step 1: Add a CI debugging section to README**

Add a section containing these exact concepts:

```markdown
## CI debugging and release evidence

KidOS cloud CI produces machine-readable artifacts for Windows diagnostics, fail-closed fault injection, repeated contract checks, post-merge smoke validation, and exact-SHA release evidence.

- `cloud_ci_ready: true` means the required GitHub-hosted validation for that exact commit passed.
- `release_ready: true` additionally requires the real Hyper-V visual validation to report `passed` for that same commit.
- An unavailable Hyper-V host is recorded as `hyperv_visual_validation: "not_run"`; it is never treated as success.

Important artifacts:

- `KidOS-Fault-Injection-<sha>`
- `KidOS-Flake-Check-<sha>`
- `KidOS-Post-Merge-<sha>`
- `KidOS-Release-Evidence-<sha>`

The self-hosted visual VM validation remains separate from ordinary PR CI so an offline physical runner cannot leave all cloud validation permanently queued.
```

Also document local commands:

```bash
node tests/security/ci-diagnostics-contract.test.mjs
node tests/security/fault-injection-contract.test.mjs
node tests/security/repeat-command.test.mjs
node tests/security/post-merge-workflow-contract.test.mjs
node tests/security/release-evidence.test.mjs
pnpm --filter @kidos/shell test --run src/features/home/system-status.test.ts
```

- [ ] **Step 2: Run the complete new static/executable suite**

```bash
node tests/security/ci-diagnostics-contract.test.mjs
node tests/security/fault-injection-contract.test.mjs
node tests/security/repeat-command.test.mjs
node tests/security/post-merge-workflow-contract.test.mjs
node tests/security/release-evidence.test.mjs
node tests/security/release-hardening.test.mjs
node tests/security/windows-code-signing-gate.test.mjs
pnpm --filter @kidos/shell test --run src/features/home/system-status.test.ts
```

Expected: all commands exit 0.

- [ ] **Step 3: Run broader repository verification**

```bash
pnpm -r test --run
pnpm -r build
cargo test --workspace
cargo check --workspace
```

Expected: all commands exit 0.

- [ ] **Step 4: Commit Task 6**

```bash
git add README.md
git commit -m "docs: document KidOS CI diagnostics and readiness"
```

- [ ] **Step 5: Push branch and verify GitHub Actions on the exact head SHA**

After push, verify these new jobs on the branch head SHA:

- `KidOS Fault Injection CI` — success and artifact exists.
- `KidOS Critical Flake Check` — success and artifact exists.
- existing `KidOS Windows Release CI` — success with all new static contract tests.
- CodeQL and dependency security — no new failing alerts introduced by the CI scripts.

Do not claim the post-merge workflow has passed before merge; its trigger is intentionally `main` only.

- [ ] **Step 6: Open PR and require fresh exact-head verification before merge**

PR summary must state:

```markdown
Adds reusable redacted Windows diagnostics, GitHub-hosted fail-closed fault injection, bounded flake detection, exact-main-SHA post-merge smoke validation, and exact-SHA cloud/release evidence.

Hyper-V visual validation remains a separate release-confidence gate. While the physical runner is offline, evidence records it as `not_run`; this never blocks ordinary cloud PR CI and never counts as `release_ready`.
```

Before merge, inspect the exact head SHA and require all branch-applicable CI/security jobs to be green.

---

## Self-Review Results

- **Spec coverage:** Tasks 1-5 cover diagnostics, runtime fault injection, bounded flake detection, post-merge exact-SHA smoke validation, CI triage/evidence, and cloud/release readiness semantics. Existing CI layers are preserved. Hyper-V visual integration is intentionally split into a follow-up after PR #37 merges, because that subsystem currently lives on a separate unmerged branch and requires physical runners.
- **Placeholder scan:** No `TBD`, `TODO`, “implement later,” or unspecified code steps remain in this plan.
- **Type/name consistency:** `fault-injection.json`, `post-merge-evidence.json`, `release-evidence.json`, `cloud_ci_ready`, `release_ready`, and `hyperv_visual_validation` use the same names throughout the plan.
- **Safety:** Diagnostics redact credential-like text; service fault injection restores services in `finally`; ordinary cloud CI never requires the offline Hyper-V runner; full release readiness still requires a real visual Hyper-V pass for the same exact SHA.
