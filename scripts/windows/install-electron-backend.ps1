param(
  [Parameter(Mandatory=$true)][string]$ResourceRoot,
  [Parameter(Mandatory=$true)][string]$InstallerPath
)

$ErrorActionPreference='Stop'
$programRoot = Join-Path $env:ProgramFiles 'KidOS'
$guardianDir = Join-Path $programRoot 'Guardian'
$classifierDir = Join-Path $programRoot 'MediaClassifier'
$recoveryDir = Join-Path $programRoot 'Recovery'
$scriptsDir = Join-Path $ResourceRoot 'scripts'

function Run-Script([string]$Path, [string[]]$Arguments = @()) {
  & "$env:SystemRoot\System32\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -ExecutionPolicy Bypass -File $Path @Arguments
  if ($LASTEXITCODE -ne 0) { throw "KidOS backend script failed ($LASTEXITCODE): $Path" }
}

$principal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
  throw 'KidOS backend installation requires administrator rights.'
}

$guardianSource = Join-Path $ResourceRoot 'kidos-guardian-host.exe'
$classifierArchive = Join-Path $ResourceRoot 'kidos-media-classifier-bundle.zip'
$modelSource = Join-Path $ResourceRoot 'model'
if (-not (Test-Path -LiteralPath $guardianSource)) { throw "Missing Guardian payload: $guardianSource" }
if (-not (Test-Path -LiteralPath $classifierArchive)) { throw "Missing classifier payload: $classifierArchive" }
if (-not (Test-Path -LiteralPath $modelSource -PathType Container)) { throw "Missing classifier model: $modelSource" }

Run-Script (Join-Path $scriptsDir 'stop-kidos-services.ps1') @('-TimeoutSeconds','45')

New-Item -ItemType Directory -Force -Path $guardianDir,$classifierDir,$recoveryDir | Out-Null
Copy-Item -LiteralPath $guardianSource -Destination (Join-Path $guardianDir 'kidos-guardian-host.exe') -Force

$guardianScripts = @(
  'provision-guardian-credentials.ps1',
  'install-kidos-services.ps1',
  'stop-kidos-services.ps1',
  'verify-kidos-services.ps1',
  'install-recovery-task.ps1',
  'hash-recovery-installer.ps1',
  'rollback-partial-install.ps1'
)
foreach ($name in $guardianScripts) {
  Copy-Item -LiteralPath (Join-Path $scriptsDir $name) -Destination (Join-Path $guardianDir $name) -Force
}

$recoveryScripts = @('kidos-recovery.ps1','restore-windows-account.ps1','verify-restore-result.ps1','rollback-kidos.ps1')
foreach ($name in $recoveryScripts) {
  Copy-Item -LiteralPath (Join-Path $scriptsDir $name) -Destination (Join-Path $recoveryDir $name) -Force
}

Run-Script (Join-Path $scriptsDir 'extract-media-classifier.ps1') @(
  '-ArchivePath', $classifierArchive,
  '-DestinationPath', $classifierDir
)
$modelDestination = Join-Path $classifierDir 'model'
Remove-Item -LiteralPath $modelDestination -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $modelDestination | Out-Null
Copy-Item -Path (Join-Path $modelSource '*') -Destination $modelDestination -Recurse -Force

& "$env:SystemRoot\System32\icacls.exe" $programRoot /inheritance:r /grant:r '*S-1-5-18:(OI)(CI)(F)' '*S-1-5-32-544:(OI)(CI)(F)' '*S-1-5-32-545:(OI)(CI)(RX)' | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'KidOS could not secure its backend installation directory.' }

Run-Script (Join-Path $guardianDir 'provision-guardian-credentials.ps1') @('-InstallerPath',$InstallerPath)
Run-Script (Join-Path $guardianDir 'install-kidos-services.ps1') @(
  '-GuardianExe',(Join-Path $guardianDir 'kidos-guardian-host.exe'),
  '-ClassifierExe',(Join-Path $classifierDir 'kidos-media-classifier.exe')
)
Start-Sleep -Seconds 3
Run-Script (Join-Path $guardianDir 'verify-kidos-services.ps1')
Run-Script (Join-Path $guardianDir 'install-recovery-task.ps1') @('-RecoveryScript',(Join-Path $recoveryDir 'kidos-recovery.ps1'))

Write-Host 'KidOS Electron backend installation completed.'
