param(
  [Parameter(Mandatory=$true)][string]$ResourceRoot
)

$ErrorActionPreference='Stop'
$programRoot = Join-Path $env:ProgramFiles 'KidOS'
$recoveryScript = Join-Path $programRoot 'Recovery\restore-windows-account.ps1'
$verifyRestore = Join-Path $programRoot 'Recovery\verify-restore-result.ps1'
$stopServices = Join-Path $programRoot 'Guardian\stop-kidos-services.ps1'
$resultPath = Join-Path $env:ProgramData 'KidOS\Recovery\restore-windows.result'

if (Test-Path -LiteralPath $recoveryScript) {
  Remove-Item -LiteralPath $resultPath -Force -ErrorAction SilentlyContinue
  $taskName = 'KidOS Restore Windows Account'
  & "$env:SystemRoot\System32\schtasks.exe" /Delete /TN $taskName /F 2>$null | Out-Null
  $command = ('"{0}" -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "{1}"' -f "$env:SystemRoot\System32\WindowsPowerShell\v1.0\powershell.exe",$recoveryScript)
  & "$env:SystemRoot\System32\schtasks.exe" /Create /TN $taskName /SC ONSTART /RU SYSTEM /RL HIGHEST /TR $command /F | Out-Null
  if ($LASTEXITCODE -ne 0) { throw 'KidOS could not create the Windows recovery task.' }
  & "$env:SystemRoot\System32\schtasks.exe" /Run /TN $taskName | Out-Null
  if ($LASTEXITCODE -ne 0) { throw 'KidOS could not start the Windows recovery task.' }
  Start-Sleep -Seconds 5
  & "$env:SystemRoot\System32\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -ExecutionPolicy Bypass -File $verifyRestore
  if ($LASTEXITCODE -ne 0) { throw 'KidOS could not verify that Windows lockdown was removed.' }
  & "$env:SystemRoot\System32\schtasks.exe" /Delete /TN $taskName /F | Out-Null
}

if (Test-Path -LiteralPath $stopServices) {
  & "$env:SystemRoot\System32\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -ExecutionPolicy Bypass -File $stopServices -TimeoutSeconds 45
  if ($LASTEXITCODE -ne 0) { throw 'KidOS could not stop its protection services.' }
}

Remove-Item -LiteralPath $programRoot -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item -LiteralPath (Join-Path $env:ProgramData 'KidOS') -Recurse -Force -ErrorAction SilentlyContinue
Write-Host 'KidOS backend removed after Windows account recovery.'
