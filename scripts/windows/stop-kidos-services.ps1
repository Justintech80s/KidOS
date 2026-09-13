param(
  [int]$TimeoutSeconds = 45
)

$ErrorActionPreference = 'Stop'

function Remove-KidOSRecoveryTask {
  # Query the scheduler through the PowerShell ScheduledTasks provider so a
  # missing KidOS task is a normal clean-install state instead of native stderr.
  # A scheduler query failure still throws and keeps upgrade cleanup fail-closed.
  $tasks = @(Get-ScheduledTask -ErrorAction Stop | Where-Object {
    $_.TaskName -eq 'KidOS Guardian Recovery' -and $_.TaskPath -eq '\'
  })

  foreach ($task in $tasks) {
    Unregister-ScheduledTask -InputObject $task -Confirm:$false -ErrorAction Stop
  }

  $remaining = @(Get-ScheduledTask -ErrorAction Stop | Where-Object {
    $_.TaskName -eq 'KidOS Guardian Recovery' -and $_.TaskPath -eq '\'
  })
  if ($remaining.Count -ne 0) {
    throw 'KidOS Guardian Recovery task is still registered after cleanup.'
  }
}

function Wait-KidOSServiceGone([string]$Name, [int]$Seconds) {
  $deadline = (Get-Date).AddSeconds($Seconds)
  do {
    if (-not (Get-Service -Name $Name -ErrorAction SilentlyContinue)) { return }
    Start-Sleep -Milliseconds 500
  } while ((Get-Date) -lt $deadline)
  throw "KidOS service '$Name' is still registered after $Seconds seconds."
}

function Stop-AndDeleteKidOSService([string]$Name, [int]$Seconds) {
  $svc = Get-Service -Name $Name -ErrorAction SilentlyContinue
  if ($null -eq $svc) { return }

  if ($svc.Status -ne 'Stopped') {
    Stop-Service -Name $Name -Force -ErrorAction SilentlyContinue
    try { $svc.WaitForStatus('Stopped', [TimeSpan]::FromSeconds([Math]::Min($Seconds, 30))) } catch {}
  }

  $stillRunning = Get-Service -Name $Name -ErrorAction SilentlyContinue
  if ($stillRunning -and $stillRunning.Status -ne 'Stopped') {
    throw "KidOS service '$Name' could not be stopped."
  }

  & "$env:SystemRoot\System32\sc.exe" delete $Name | Out-Null
  $deleteExit = $LASTEXITCODE
  if ($deleteExit -ne 0 -and (Get-Service -Name $Name -ErrorAction SilentlyContinue)) {
    throw "KidOS service '$Name' could not be deleted (sc.exe exit $deleteExit)."
  }
  Wait-KidOSServiceGone -Name $Name -Seconds $Seconds
}

function Wait-KidOSProcessGone([string]$ProcessName, [int]$Seconds) {
  $deadline = (Get-Date).AddSeconds($Seconds)
  do {
    $processes = @(Get-Process -Name $ProcessName -ErrorAction SilentlyContinue)
    if ($processes.Count -eq 0) { return }
    Start-Sleep -Milliseconds 500
  } while ((Get-Date) -lt $deadline)

  # A deleted Windows service can briefly leave its process alive. Kill only the
  # exact KidOS executable name, then verify the file lock is actually gone.
  Get-Process -Name $ProcessName -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
  $forceDeadline = (Get-Date).AddSeconds(10)
  do {
    if (@(Get-Process -Name $ProcessName -ErrorAction SilentlyContinue).Count -eq 0) { return }
    Start-Sleep -Milliseconds 500
  } while ((Get-Date) -lt $forceDeadline)

  throw "KidOS process '$ProcessName' is still running after service shutdown."
}

try {
  # Disable recovery first so it cannot restart Guardian while an upgrade or
  # uninstall deliberately stops the protection services.
  Remove-KidOSRecoveryTask

  Stop-AndDeleteKidOSService -Name 'KidOSMediaClassifier' -Seconds $TimeoutSeconds
  Stop-AndDeleteKidOSService -Name 'KidOSGuardian' -Seconds $TimeoutSeconds

  Wait-KidOSProcessGone -ProcessName 'kidos-media-classifier' -Seconds 15
  Wait-KidOSProcessGone -ProcessName 'kidos-guardian-host' -Seconds 15

  exit 0
} catch {
  Write-Error $_
  exit 1
}
