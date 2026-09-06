param(
  [int]$Minutes = 20
)

$ErrorActionPreference = "Stop"

function Assert-True([bool]$Condition, [string]$Message) {
  if (-not $Condition) { throw $Message }
}

$tokenPath = Join-Path $env:ProgramData "KidOS\Guardian\media-classifier.token"
Assert-True (Test-Path $tokenPath) "KidOS classifier token is missing."
$token = (Get-Content $tokenPath -Raw).Trim()
$deadline = (Get-Date).AddMinutes($Minutes)
$iteration = 0

while ((Get-Date) -lt $deadline) {
  $iteration++
  $guardian = Get-Service KidOSGuardian -ErrorAction Stop
  $classifier = Get-Service KidOSMediaClassifier -ErrorAction Stop
  Assert-True ($guardian.Status -eq "Running") "Guardian stopped during soak iteration $iteration."
  Assert-True ($classifier.Status -eq "Running") "Media classifier stopped during soak iteration $iteration."

  $health = Invoke-RestMethod -Uri "http://127.0.0.1:8765/health" -Headers @{ "x-kidos-classifier-token" = $token } -TimeoutSec 5
  Assert-True ($health.status -eq "healthy") "Classifier health check failed during soak iteration $iteration."

  $guardianProcess = Get-CimInstance Win32_Service -Filter "Name='KidOSGuardian'" | ForEach-Object { Get-Process -Id $_.ProcessId -ErrorAction SilentlyContinue }
  $classifierProcess = Get-CimInstance Win32_Service -Filter "Name='KidOSMediaClassifier'" | ForEach-Object { Get-Process -Id $_.ProcessId -ErrorAction SilentlyContinue }
  if ($guardianProcess) {
    Assert-True ($guardianProcess.WorkingSet64 -lt 2GB) "Guardian memory exceeded 2 GB during soak test."
  }
  if ($classifierProcess) {
    Assert-True ($classifierProcess.WorkingSet64 -lt 6GB) "Classifier memory exceeded 6 GB during soak test."
  }

  if (($iteration % 5) -eq 0) {
    $recovery = Join-Path $env:ProgramFiles "KidOS\Recovery\kidos-recovery.ps1"
    Assert-True (Test-Path $recovery) "Recovery script disappeared during soak test."
    & powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File $recovery -StatusOnly | Out-Null
    Assert-True ($LASTEXITCODE -eq 0) "Recovery status check failed during soak test."
  }

  Write-Host "KidOS soak iteration $iteration healthy at $(Get-Date -Format o)."
  Start-Sleep -Seconds 30
}

Write-Host "KidOS service soak test passed for $Minutes minute(s)."
