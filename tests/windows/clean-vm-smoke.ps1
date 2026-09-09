param(
  [Parameter(Mandatory=$true)]
  [string]$InstallerPath
)

$ErrorActionPreference = "Stop"

function Invoke-ProcessWithTimeout {
  param(
    [Parameter(Mandatory=$true)][string]$FilePath,
    [string]$Arguments = "",
    [int]$TimeoutSeconds = 300,
    [string]$Label = "process"
  )

  Write-Host ("Starting {0} with a {1}s timeout..." -f $Label, $TimeoutSeconds)
  $p = Start-Process -FilePath $FilePath -ArgumentList $Arguments -PassThru
  try {
    if (-not $p.WaitForExit($TimeoutSeconds * 1000)) {
      Write-Warning ("{0} timed out after {1} seconds. Capturing process diagnostics." -f $Label, $TimeoutSeconds)
      Get-Process | Sort-Object ProcessName | Format-Table Id,ProcessName,CPU,StartTime -AutoSize | Out-String | Write-Host
      try { Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue } catch {}
      throw ("{0} timed out after {1} seconds." -f $Label, $TimeoutSeconds)
    }
    return $p.ExitCode
  } finally {
    try { $p.Dispose() } catch {}
  }
}

function Write-KidOSDiagnostics {
  param([string]$Stage)
  Write-Host ("=== Diagnostics: {0} ===" -f $Stage)
  Get-Service KidOSGuardian,KidOSMediaClassifier -ErrorAction SilentlyContinue |
    Format-Table Name,Status,StartType -AutoSize | Out-String | Write-Host
  Get-ScheduledTask -TaskName "KidOS Guardian Recovery" -ErrorAction SilentlyContinue |
    Format-List TaskName,State,TaskPath | Out-String | Write-Host
  Get-Process | Where-Object { $_.ProcessName -match "KidOS|kidos|powershell|nsis|uninstall" } |
    Format-Table Id,ProcessName,CPU,StartTime -AutoSize | Out-String | Write-Host
}
function Assert-True([bool]$Condition, [string]$Message) {
  if (-not $Condition) { throw $Message }
}

function Wait-ServiceRunning([string]$Name, [int]$Seconds = 45) {
  $deadline = (Get-Date).AddSeconds($Seconds)
  do {
    $svc = Get-Service -Name $Name -ErrorAction SilentlyContinue
    if ($svc -and $svc.Status -eq "Running") { return $svc }
    Start-Sleep -Seconds 2
  } while ((Get-Date) -lt $deadline)
  throw "Service '$Name' did not reach Running state."
}

Write-Host "=== KidOS clean Windows VM smoke test ==="
Assert-True (Test-Path $InstallerPath) "Installer was not found: $InstallerPath"

$childUser = "KidOSTestChild"
if (-not (Get-LocalUser -Name $childUser -ErrorAction SilentlyContinue)) {
  $password = ConvertTo-SecureString ("K!dOS-" + [Guid]::NewGuid().ToString("N") + "9") -AsPlainText -Force
  New-LocalUser -Name $childUser -Password $password -PasswordNeverExpires -UserMayNotChangePassword | Out-Null
}
$admins = Get-LocalGroupMember -Group "Administrators" -ErrorAction Stop | ForEach-Object { $_.Name.Split("\")[-1] }
Assert-True (-not ($admins -contains $childUser)) "Test child account unexpectedly belongs to Administrators."

Write-Host "Installing KidOS silently..."
$installExit = Invoke-ProcessWithTimeout -FilePath $InstallerPath -Arguments "/S" -TimeoutSeconds 420 -Label "KidOS installer"
Assert-True ($installExit -eq 0) "KidOS installer exited with code $installExit."
Write-KidOSDiagnostics -Stage "after install"

$guardian = Wait-ServiceRunning "KidOSGuardian"
$classifier = Wait-ServiceRunning "KidOSMediaClassifier"

$guardianWmi = Get-CimInstance Win32_Service -Filter "Name='KidOSGuardian'"
$classifierWmi = Get-CimInstance Win32_Service -Filter "Name='KidOSMediaClassifier'"
Assert-True ($guardianWmi.StartName -eq "LocalSystem") "Guardian is not running as LocalSystem."
Assert-True ($classifierWmi.StartName -eq "LocalSystem") "Classifier is not running as LocalSystem."
Assert-True ($guardianWmi.StartMode -eq "Auto") "Guardian is not configured for automatic startup."
Assert-True ($classifierWmi.StartMode -eq "Auto") "Classifier is not configured for automatic startup."

$guardianExe = Join-Path $env:ProgramFiles "KidOS\Guardian\kidos-guardian-host.exe"
$classifierExe = Join-Path $env:ProgramFiles "KidOS\MediaClassifier\kidos-media-classifier.exe"
$recoveryScript = Join-Path $env:ProgramFiles "KidOS\Recovery\kidos-recovery.ps1"
$tokenPath = Join-Path $env:ProgramData "KidOS\Guardian\media-classifier.token"

Assert-True (Test-Path $guardianExe) "Guardian executable was not installed."
Assert-True (Test-Path $classifierExe) "Classifier executable was not installed."
Assert-True (Test-Path $recoveryScript) "Recovery script was not installed."
Assert-True (Test-Path $tokenPath) "Classifier credential was not provisioned."

$token = (Get-Content $tokenPath -Raw).Trim()
Assert-True ($token.Length -ge 64) "Classifier token is too short."

Write-Host "Checking local classifier health..."
$health = $null
for ($i=0; $i -lt 30; $i++) {
  try {
    $health = Invoke-RestMethod -Uri "http://127.0.0.1:8765/health" -Headers @{ "x-kidos-classifier-token" = $token } -TimeoutSec 3
    if ($health.status -eq "healthy") { break }
  } catch {}
  Start-Sleep -Seconds 2
}
Assert-True ($health.status -eq "healthy") "Classifier health endpoint did not become healthy."

Write-Host "Checking deep AI model readiness..."
$modelHealth = $null
$modelDeadline = (Get-Date).AddMinutes(5)
do {
  try {
    $modelHealth = Invoke-RestMethod -Uri "http://127.0.0.1:8765/model-health" -Headers @{ "x-kidos-classifier-token" = $token } -TimeoutSec 15
    if ($modelHealth.status -eq "healthy" -and $modelHealth.classifier_loaded -eq $true) { break }
  } catch {}
  Start-Sleep -Seconds 5
} while ((Get-Date) -lt $modelDeadline)
Assert-True ($modelHealth.status -eq "healthy" -and $modelHealth.classifier_loaded -eq $true) "Classifier AI model did not become ready within 5 minutes."

Write-Host "Checking classifier rejects unauthenticated health access..."
$unauthorized = $false
try {
  Invoke-WebRequest -Uri "http://127.0.0.1:8765/health" -TimeoutSec 3 | Out-Null
} catch {
  if ($_.Exception.Response.StatusCode.value__ -eq 401) { $unauthorized = $true }
}
Assert-True $unauthorized "Classifier health endpoint accepted a request without the KidOS token."

Write-Host "Checking recovery scheduled task..."
$task = Get-ScheduledTask -TaskName "KidOS Guardian Recovery" -ErrorAction Stop
Assert-True ($task.Principal.UserId -eq "SYSTEM") "Recovery task does not run as SYSTEM."

Write-Host "Simulating Guardian service failure and recovery..."
Stop-Service KidOSGuardian -Force
Start-Sleep -Seconds 2
& powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File $recoveryScript
Wait-ServiceRunning "KidOSGuardian" | Out-Null

Write-Host "Simulating classifier service failure and recovery..."
Stop-Service KidOSMediaClassifier -Force
Start-Sleep -Seconds 2
& powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File $recoveryScript
Wait-ServiceRunning "KidOSMediaClassifier" | Out-Null

Write-Host "Checking recovery status script..."
$statusJson = & powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File $recoveryScript -StatusOnly | Out-String
Assert-True ($statusJson -match "Running") "Recovery status did not report running protection services."

Write-Host "Checking protected directories..."
$guardianAcl = (Get-Acl (Split-Path $guardianExe -Parent)).Access
$usersWrite = $guardianAcl | Where-Object {
  $_.IdentityReference -match "Users$" -and
  $_.AccessControlType -eq "Allow" -and
  ($_.FileSystemRights.ToString() -match "Write|Modify|FullControl")
}
Assert-True (-not $usersWrite) "Standard Users have write access to the Guardian directory."

Write-Host "Locating registered KidOS uninstaller..."
$uninstallEntry = Get-ChildItem "HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall" -ErrorAction SilentlyContinue |
  ForEach-Object { Get-ItemProperty $_.PSPath } |
  Where-Object { $_.DisplayName -eq "KidOS" } |
  Select-Object -First 1

if (-not $uninstallEntry) {
  $uninstallEntry = Get-ChildItem "HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall" -ErrorAction SilentlyContinue |
    ForEach-Object { Get-ItemProperty $_.PSPath } |
    Where-Object { $_.DisplayName -eq "KidOS" } |
    Select-Object -First 1
}
Assert-True ($null -ne $uninstallEntry) "KidOS uninstall registration was not found."

$uninstallString = $uninstallEntry.UninstallString
Assert-True (-not [string]::IsNullOrWhiteSpace($uninstallString)) "KidOS uninstaller command is missing."
if ($uninstallString -match '^"([^"]+)"') { $uninstaller = $matches[1] } else { $uninstaller = $uninstallString.Split(" ")[0] }
Assert-True (Test-Path $uninstaller) "KidOS uninstaller executable was not found."

Write-Host "Uninstalling KidOS silently..."
$uninstallExit = Invoke-ProcessWithTimeout -FilePath $uninstaller -Arguments "/S" -TimeoutSeconds 300 -Label "KidOS uninstaller"
Assert-True ($uninstallExit -eq 0) "KidOS uninstaller exited with code $uninstallExit."
Write-KidOSDiagnostics -Stage "after uninstall"
Start-Sleep -Seconds 3

Assert-True (-not (Get-Service KidOSGuardian -ErrorAction SilentlyContinue)) "Guardian service remains after uninstall."
Assert-True (-not (Get-Service KidOSMediaClassifier -ErrorAction SilentlyContinue)) "Classifier service remains after uninstall."
Assert-True (-not (Get-ScheduledTask -TaskName "KidOS Guardian Recovery" -ErrorAction SilentlyContinue)) "Recovery task remains after uninstall."

Remove-LocalUser -Name $childUser -ErrorAction SilentlyContinue

Write-Host "KidOS clean Windows VM smoke test passed."
