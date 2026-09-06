param(
  [switch]$StatusOnly
)

$ErrorActionPreference = "Stop"
$guardian = "KidOSGuardian"
$classifier = "KidOSMediaClassifier"
$data = Join-Path $env:ProgramData "KidOS\Guardian"
$flag = Join-Path $data "recovery-required.flag"
$log = Join-Path $data "recovery.log"

New-Item -ItemType Directory -Force -Path $data | Out-Null

function Write-RecoveryLog([string]$Message) {
  $line = "$(Get-Date -Format o) $Message"
  Add-Content -Path $log -Value $line -Encoding UTF8
}

$guardianSvc = Get-Service -Name $guardian -ErrorAction SilentlyContinue
$classifierSvc = Get-Service -Name $classifier -ErrorAction SilentlyContinue

if ($StatusOnly) {
  [pscustomobject]@{
    Guardian = if ($guardianSvc) { $guardianSvc.Status } else { "Missing" }
    Classifier = if ($classifierSvc) { $classifierSvc.Status } else { "Missing" }
    RecoveryRequired = Test-Path $flag
  }
  exit 0
}

if (-not $guardianSvc) {
  Set-Content -Path $flag -Value "guardian-service-missing" -Encoding ASCII
  Write-RecoveryLog "Guardian service missing; recovery required."
  exit 20
}

if ($guardianSvc.Status -ne "Running") {
  try {
    Start-Service $guardian
    Start-Sleep -Seconds 3
    $guardianSvc = Get-Service $guardian
  } catch {
    Set-Content -Path $flag -Value "guardian-service-failed" -Encoding ASCII
    Write-RecoveryLog "Guardian failed to start. KidOS lockdown must not be newly applied."
    exit 21
  }
}

if ($classifierSvc -and $classifierSvc.Status -ne "Running") {
  try {
    Start-Service $classifier
    Write-RecoveryLog "Restarted local classifier."
  } catch {
    Set-Content -Path $flag -Value "classifier-service-failed" -Encoding ASCII
    Write-RecoveryLog "Classifier unavailable. Media will fail closed to parent review."
  }
}

Write-RecoveryLog "Recovery health check completed."
exit 0
