param(
  [int]$Minutes = 20,
  [int]$IntervalSeconds = 15
)

$ErrorActionPreference = "Stop"

function Assert-True([bool]$Condition, [string]$Message) {
  if (-not $Condition) { throw $Message }
}

function Service-Healthy([string]$Name) {
  $svc = Get-Service -Name $Name -ErrorAction SilentlyContinue
  return $svc -and $svc.Status -eq "Running"
}

$guardian = "KidOSGuardian"
$classifier = "KidOSMediaClassifier"
$tokenPath = Join-Path $env:ProgramData "KidOS\Guardian\media-classifier.token"
$deadline = (Get-Date).AddMinutes($Minutes)
$iterations = 0
$failures = @()

Write-Host "Starting KidOS protection soak test for $Minutes minutes."

while ((Get-Date) -lt $deadline) {
  $iterations++

  if (-not (Service-Healthy $guardian)) { $failures += "Guardian service not running at iteration $iterations." }
  if (-not (Service-Healthy $classifier)) { $failures += "Classifier service not running at iteration $iterations." }

  if (Test-Path $tokenPath) {
    try {
      $token = (Get-Content $tokenPath -Raw).Trim()
      if ($token.Length -lt 64) {
        $failures += "Classifier token invalid at iteration $iterations."
      } else {
        try {
          $health = Invoke-RestMethod -Uri "http://127.0.0.1:8765/health" -Headers @{ "x-kidos-classifier-token" = $token } -TimeoutSec 3
          if ($health.status -ne "healthy") { $failures += "Classifier health not healthy at iteration $iterations." }
        } catch {
          $failures += "Classifier health endpoint failed at iteration $iterations."
        }
      }
    } catch {
      $failures += "Classifier token could not be read at iteration $iterations."
    }
  } else {
    $failures += "Classifier token missing at iteration $iterations."
  }

  Start-Sleep -Seconds $IntervalSeconds
}

Assert-True ($failures.Count -eq 0) ("KidOS soak test found failures: " + ($failures -join "; "))

[pscustomobject]@{
  passed = $true
  durationMinutes = $Minutes
  iterations = $iterations
  guardianRunning = $true
  classifierRunning = $true
  completedAt = (Get-Date).ToString("o")
} | ConvertTo-Json
