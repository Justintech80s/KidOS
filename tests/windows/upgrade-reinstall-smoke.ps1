param(
  [Parameter(Mandatory=$true)][string]$InstallerPath
)

$ErrorActionPreference = 'Stop'

# This real-machine smoke test intentionally exercises the packaged installer twice.
# Installer-stage diagnostics are enforced separately by the packaging contract test.
function Assert-True([bool]$Condition, [string]$Message) {
  if (-not $Condition) { throw $Message }
}

function Invoke-ProcessWithTimeout {
  param(
    [Parameter(Mandatory=$true)][string]$FilePath,
    [string]$Arguments = '',
    [int]$TimeoutSeconds = 900,
    [string]$Label = 'process'
  )
  $p = Start-Process -FilePath $FilePath -ArgumentList $Arguments -PassThru
  try {
    if (-not $p.WaitForExit($TimeoutSeconds * 1000)) {
      try { Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue } catch {}
      throw "$Label timed out after $TimeoutSeconds seconds."
    }
    return $p.ExitCode
  } finally {
    try { $p.Dispose() } catch {}
  }
}

function Wait-ServiceRunning([string]$Name, [int]$Seconds = 60) {
  $deadline = (Get-Date).AddSeconds($Seconds)
  do {
    $svc = Get-Service -Name $Name -ErrorAction SilentlyContinue
    if ($svc -and $svc.Status -eq 'Running') { return }
    Start-Sleep -Seconds 2
  } while ((Get-Date) -lt $deadline)
  throw "Service '$Name' did not reach Running state."
}

function Find-KidOSUninstaller {
  foreach ($root in @(
    'HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall',
    'HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall'
  )) {
    $entry = Get-ChildItem $root -ErrorAction SilentlyContinue |
      ForEach-Object { Get-ItemProperty $_.PSPath } |
      Where-Object { $_.DisplayName -eq 'KidOS' } |
      Select-Object -First 1
    if ($entry) {
      $value = $entry.UninstallString
      if ($value -match '^"([^"]+)"') { return $matches[1] }
      return $value.Split(' ')[0]
    }
  }
  return $null
}

function Wait-Uninstalled([int]$Seconds = 150) {
  $deadline = (Get-Date).AddSeconds($Seconds)
  do {
    $guardian = Get-Service KidOSGuardian -ErrorAction SilentlyContinue
    $classifier = Get-Service KidOSMediaClassifier -ErrorAction SilentlyContinue
    $task = Get-ScheduledTask -TaskName 'KidOS Guardian Recovery' -ErrorAction SilentlyContinue
    if (-not $guardian -and -not $classifier -and -not $task) { return }
    Start-Sleep -Seconds 2
  } while ((Get-Date) -lt $deadline)
  throw 'KidOS uninstall did not complete service/task cleanup.'
}

Assert-True (Test-Path $InstallerPath) "Installer missing: $InstallerPath"

Write-Host '=== First KidOS install ==='
$first = Invoke-ProcessWithTimeout -FilePath $InstallerPath -Arguments '/S' -Label 'first KidOS install'
Assert-True ($first -eq 0) "First install exited with code $first."
Wait-ServiceRunning 'KidOSGuardian'
Wait-ServiceRunning 'KidOSMediaClassifier'

$guardianPath = Join-Path $env:ProgramFiles 'KidOS\Guardian\kidos-guardian-host.exe'
Assert-True (Test-Path $guardianPath) 'Guardian executable missing after first install.'
$firstGuardianHash = (Get-FileHash $guardianPath -Algorithm SHA256).Hash

Write-Host '=== Reinstall/upgrade while services are running ==='
$second = Invoke-ProcessWithTimeout -FilePath $InstallerPath -Arguments '/S' -Label 'KidOS reinstall over running services'
Assert-True ($second -eq 0) "Reinstall exited with code $second."
Wait-ServiceRunning 'KidOSGuardian'
Wait-ServiceRunning 'KidOSMediaClassifier'
Assert-True (Test-Path $guardianPath) 'Guardian executable missing after reinstall.'
$secondGuardianHash = (Get-FileHash $guardianPath -Algorithm SHA256).Hash
Assert-True ($firstGuardianHash -eq $secondGuardianHash) 'Reinstall produced an unexpected Guardian binary.'

Write-Host '=== Uninstall after upgrade ==='
$uninstaller = Find-KidOSUninstaller
Assert-True (-not [string]::IsNullOrWhiteSpace($uninstaller)) 'KidOS uninstaller registration was not found.'
Assert-True (Test-Path $uninstaller) 'KidOS uninstaller executable was not found.'
$uninstall = Invoke-ProcessWithTimeout -FilePath $uninstaller -Arguments '/S' -TimeoutSeconds 300 -Label 'KidOS uninstall after upgrade'
Assert-True ($uninstall -eq 0) "Uninstall exited with code $uninstall."
Wait-Uninstalled

Assert-True (-not (Get-Service KidOSGuardian -ErrorAction SilentlyContinue)) 'Guardian service remains after upgrade uninstall.'
Assert-True (-not (Get-Service KidOSMediaClassifier -ErrorAction SilentlyContinue)) 'Classifier service remains after upgrade uninstall.'
Assert-True (-not (Get-ScheduledTask -TaskName 'KidOS Guardian Recovery' -ErrorAction SilentlyContinue)) 'Recovery task remains after upgrade uninstall.'
Assert-True (-not (Test-Path (Join-Path $env:ProgramFiles 'KidOS\Guardian'))) 'Guardian directory remains after upgrade uninstall.'
Assert-True (-not (Test-Path (Join-Path $env:ProgramFiles 'KidOS\MediaClassifier'))) 'Classifier directory remains after upgrade uninstall.'

Write-Host 'KidOS upgrade/reinstall lifecycle smoke test passed.'
