$ErrorActionPreference = 'Stop'
foreach ($name in @('KidOSMediaClassifier','KidOSGuardian')) {
  $svc = Get-Service -Name $name -ErrorAction SilentlyContinue
  if ($null -ne $svc) {
    if ($svc.Status -ne 'Stopped') {
      Stop-Service -Name $name -Force -ErrorAction SilentlyContinue
      Start-Sleep -Milliseconds 300
    }
    & "$env:SystemRoot\System32\sc.exe" delete $name | Out-Null
  }
}
exit 0
