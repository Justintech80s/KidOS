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

function Invoke-Schtasks([string[]]$Arguments, [int]$TimeoutSeconds = 20) {
  $exe = Join-Path $env:SystemRoot 'System32\schtasks.exe'
  $psi = New-Object System.Diagnostics.ProcessStartInfo
  $psi.FileName = $exe
  $psi.UseShellExecute = $false
  $psi.CreateNoWindow = $true
  $psi.RedirectStandardOutput = $true
  $psi.RedirectStandardError = $true

  # Quote each argument ourselves so the /TR command remains one argument.
  $escaped = foreach ($arg in $Arguments) {
    if ($arg -match '[\s"]') {
      '"' + ($arg -replace '(\\*)"', '$1$1\"' -replace '(\\+)$', '$1$1') + '"'
    } else {
      $arg
    }
  }
  $psi.Arguments = ($escaped -join ' ')

  $p = New-Object System.Diagnostics.Process
  $p.StartInfo = $psi
  if (-not $p.Start()) { throw 'Could not start schtasks.exe.' }

  if (-not $p.WaitForExit($TimeoutSeconds * 1000)) {
    try { $p.Kill() } catch {}
    throw "schtasks.exe timed out after $TimeoutSeconds seconds."
  }

  $stdout = $p.StandardOutput.ReadToEnd().Trim()
  $stderr = $p.StandardError.ReadToEnd().Trim()
  Write-RecoveryTaskLog ("schtasks exit={0}; stdout={1}; stderr={2}" -f $p.ExitCode,$stdout,$stderr)

  if ($p.ExitCode -ne 0) {
    throw "schtasks.exe failed with exit code $($p.ExitCode)."
  }
}

try {
  if (-not (Test-Path -LiteralPath $RecoveryScript -PathType Leaf)) {
    throw "Recovery script is missing: $RecoveryScript"
  }

  $taskName = 'KidOS Guardian Recovery'
  $powershell = Join-Path $env:SystemRoot 'System32\WindowsPowerShell\v1.0\powershell.exe'
  $taskCommand = ('"{0}" -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "{1}"' -f $powershell,$RecoveryScript)

  Write-RecoveryTaskLog ("Installing recovery task with command: {0}" -f $taskCommand)

  # Delete is best effort: the task does not exist on a clean install.
  try {
    Invoke-Schtasks @('/Delete','/TN',$taskName,'/F') 10
  } catch {
    Write-RecoveryTaskLog ("Existing task delete was non-fatal: {0}" -f $_.Exception.Message)
  }

  Invoke-Schtasks @(
    '/Create',
    '/TN', $taskName,
    '/SC', 'ONSTART',
    '/RU', 'SYSTEM',
    '/RL', 'HIGHEST',
    '/TR', $taskCommand,
    '/F'
  ) 20

  # Verify with schtasks rather than importing the ScheduledTasks module.
  Invoke-Schtasks @('/Query','/TN',$taskName) 10

  Write-RecoveryTaskLog 'Recovery task installed and verified successfully.'
  exit 0
} catch {
  Write-RecoveryTaskLog ("Recovery task installation failed: {0}" -f $_.Exception.Message)
  Write-Error $_
  exit 1
}
