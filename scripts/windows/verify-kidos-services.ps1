$ErrorActionPreference = 'Stop'
$diagDir = Join-Path $env:ProgramData 'KidOS\Diagnostics'
$diagLog = Join-Path $diagDir 'service-verification.log'
New-Item -ItemType Directory -Force -Path $diagDir | Out-Null

function Write-VerifyLog([string]$Message) {
  $line = ('{0:u} {1}' -f (Get-Date), $Message)
  Add-Content -LiteralPath $diagLog -Value $line -Encoding UTF8
  Write-Host $line
}

try {
  $guardian = Get-Service -Name KidOSGuardian -ErrorAction Stop
  $classifierService = Get-Service -Name KidOSMediaClassifier -ErrorAction Stop
  Write-VerifyLog ("Guardian status={0}; Classifier status={1}" -f $guardian.Status,$classifierService.Status)

  if ($guardian.Status -ne 'Running') {
    Write-VerifyLog 'Guardian service is not Running.'
    exit 20
  }
  if ($classifierService.Status -ne 'Running') {
    Write-VerifyLog 'Media classifier service is not Running.'
    exit 21
  }

  $tokenPath = Join-Path $env:ProgramData 'KidOS\Guardian\media-classifier.token'
  if (-not (Test-Path -LiteralPath $tokenPath -PathType Leaf)) {
    Write-VerifyLog ("Classifier token is missing: {0}" -f $tokenPath)
    exit 22
  }

  $token = (Get-Content -LiteralPath $tokenPath -Raw).Trim()
  if ([string]::IsNullOrWhiteSpace($token)) {
    Write-VerifyLog 'Classifier token file is empty.'
    exit 22
  }

  $ok = $false
  for ($i=0; $i -lt 60; $i++) {
    try {
      $r = Invoke-RestMethod -Uri 'http://127.0.0.1:8765/health' -Headers @{'x-kidos-classifier-token'=$token} -TimeoutSec 2
      if ($r.status -eq 'healthy' -and $r.model_present -ne $false) {
        Write-VerifyLog ("Classifier service health passed on attempt {0}; model_present={1}; classifier_loaded={2}" -f ($i+1),$r.model_present,$r.classifier_loaded)
        $ok=$true
        break
      }
      Write-VerifyLog ("Classifier health returned unexpected payload on attempt {0}." -f ($i+1))
    } catch {
      if (($i % 5) -eq 0) {
        Write-VerifyLog ("Classifier health attempt {0} failed: {1}" -f ($i+1),$_.Exception.Message)
      }
    }
    Start-Sleep -Seconds 1
  }

  if (-not $ok) {
    Write-VerifyLog 'Classifier service did not become healthy within 60 seconds.'
    exit 22
  }

  Write-VerifyLog 'KidOS Windows services passed installation verification.'
  exit 0
} catch {
  Write-VerifyLog ("Service verification failed unexpectedly: {0}" -f $_.Exception.Message)
  Write-Error $_
  exit 23
}
