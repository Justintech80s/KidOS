param(
  [int]$TimeoutSeconds = 45
)

$ErrorActionPreference = 'SilentlyContinue'

function Wait-ServiceGone([string]$Name, [int]$Seconds) {
  $deadline = (Get-Date).AddSeconds($Seconds)
  do {
    if (-not (Get-Service -Name $Name -ErrorAction SilentlyContinue)) { return $true }
    Start-Sleep -Milliseconds 500
  } while ((Get-Date) -lt $deadline)
  return $false
}

function Remove-ServiceIfPresent([string]$Name) {
  $svc = Get-Service -Name $Name -ErrorAction SilentlyContinue
  if ($null -eq $svc) { return }

  if ($svc.Status -ne 'Stopped') {
    Stop-Service -Name $Name -Force -ErrorAction SilentlyContinue
    try { $svc.WaitForStatus('Stopped', [TimeSpan]::FromSeconds(20)) } catch {}
  }
  & "$env:SystemRoot\System32\sc.exe" delete $Name | Out-Null
  [void](Wait-ServiceGone -Name $Name -Seconds $TimeoutSeconds)
}

# Disable recovery before service cleanup so it cannot race rollback by
# restarting Guardian while the installer is trying to remove partial state.
& "$env:SystemRoot\System32\schtasks.exe" /Delete /TN 'KidOS Guardian Recovery' /F | Out-Null
& "$env:SystemRoot\System32\schtasks.exe" /Delete /TN 'KidOS Restore Windows Account' /F | Out-Null

Remove-ServiceIfPresent 'KidOSMediaClassifier'
Remove-ServiceIfPresent 'KidOSGuardian'

foreach ($processName in @('kidos-media-classifier','kidos-guardian-host')) {
  $deadline = (Get-Date).AddSeconds(15)
  do {
    $running = @(Get-Process -Name $processName -ErrorAction SilentlyContinue)
    if ($running.Count -eq 0) { break }
    Start-Sleep -Milliseconds 500
  } while ((Get-Date) -lt $deadline)
  Get-Process -Name $processName -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
}

# Best-effort removal of KidOS Assigned Access state. This is intentionally scoped
# to the MDM AssignedAccess singleton and never touches user-profile files.
try {
    $namespace = 'root\cimv2\mdm\dmmap'
    $instances = @(Get-CimInstance -Namespace $namespace -ClassName MDM_AssignedAccess -ErrorAction SilentlyContinue)
    foreach ($instance in $instances) {
        try {
            Remove-CimInstance -InputObject $instance -ErrorAction Stop
        } catch {
            try {
                $instance.Configuration = ''
                Set-CimInstance -InputObject $instance -ErrorAction Stop | Out-Null
            } catch {}
        }
    }
} catch {}

Remove-Item -LiteralPath (Join-Path $env:ProgramData 'KidOS\Guardian\recovery-required.flag') -Force -ErrorAction SilentlyContinue
Remove-Item -LiteralPath (Join-Path $env:ProgramData 'KidOS\Guardian\lockdown-profile.json') -Force -ErrorAction SilentlyContinue

exit 0
