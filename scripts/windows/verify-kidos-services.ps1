$ErrorActionPreference = 'Stop'
try {
  if ((Get-Service -Name KidOSGuardian -ErrorAction Stop).Status -ne 'Running') { exit 20 }
  if ((Get-Service -Name KidOSMediaClassifier -ErrorAction Stop).Status -ne 'Running') { exit 21 }
  $tokenPath = Join-Path $env:ProgramData 'KidOS\Guardian\media-classifier.token'
  $token = (Get-Content -LiteralPath $tokenPath -Raw).Trim()
  if ([string]::IsNullOrWhiteSpace($token)) { exit 22 }
  $ok = $false
  for ($i=0; $i -lt 45; $i++) {
    try {
      $r = Invoke-RestMethod -Uri 'http://127.0.0.1:8765/health' -Headers @{'x-kidos-classifier-token'=$token} -TimeoutSec 3
      if ($r.status -eq 'healthy') { $ok=$true; break }
    } catch {}
    Start-Sleep -Seconds 2
  }
  if (-not $ok) { exit 22 }
  exit 0
} catch { Write-Error $_; exit 23 }
