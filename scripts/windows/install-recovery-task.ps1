param(
  [Parameter(Mandatory=$true)][string]$RecoveryScript
)

$ErrorActionPreference = 'Stop'
$diagDir = Join-Path $env:ProgramData 'KidOS\Diagnostics'
$diagLog = Join-Path $diagDir 'recovery-task-install.log'
New-Item -ItemType Directory -Force -Path $diagDir | Out-Null

function Write-RecoveryTaskLog([string]$Message) {
  $line = ('{0:u} {1}' -f (Get-Date), $Message)
  Add-Content -LiteralPath $diagLog -Value $line -Encoding UTF8
  Write-Host $line
}

try {
  if (-not (Test-Path -LiteralPath $RecoveryScript -PathType Leaf)) {
    throw "Recovery script is missing: $RecoveryScript"
  }

  $taskName = 'KidOS Guardian Recovery'
  Write-RecoveryTaskLog ("Installing scheduled recovery task for {0}" -f $RecoveryScript)

  Unregister-ScheduledTask -TaskName $taskName -Confirm:$false -ErrorAction SilentlyContinue

  $powershell = Join-Path $env:SystemRoot 'System32\WindowsPowerShell\v1.0\powershell.exe'
  $arguments = ('-NoProfile -NonInteractive -ExecutionPolicy Bypass -File "{0}"' -f $RecoveryScript)

  $action = New-ScheduledTaskAction -Execute $powershell -Argument $arguments
  $trigger = New-ScheduledTaskTrigger -AtStartup
  $principal = New-ScheduledTaskPrincipal -UserId 'SYSTEM' -LogonType ServiceAccount -RunLevel Highest
  $settings = New-ScheduledTaskSettingsSet -StartWhenAvailable -MultipleInstances IgnoreNew

  Register-ScheduledTask -TaskName $taskName -Action $action -Trigger $trigger -Principal $principal -Settings $settings -Force | Out-Null

  $task = Get-ScheduledTask -TaskName $taskName -ErrorAction Stop
  if ($task.Principal.UserId -ne 'SYSTEM') {
    throw "Recovery task principal is not SYSTEM."
  }
  if ($task.State -eq 'Disabled') {
    throw "Recovery task was created disabled."
  }

  Write-RecoveryTaskLog ("Recovery task installed successfully. State={0}; UserId={1}" -f $task.State,$task.Principal.UserId)
  exit 0
} catch {
  Write-RecoveryTaskLog ("Recovery task installation failed: {0}" -f $_.Exception.Message)
  Write-Error $_
  exit 1
}
