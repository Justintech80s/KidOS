param()

$ErrorActionPreference = "Stop"

function Assert-True([bool]$Condition, [string]$Message) {
  if (-not $Condition) { throw $Message }
}

$programFilesKidOS = Join-Path $env:ProgramFiles "KidOS"
$guardianDir = Join-Path $programFilesKidOS "Guardian"
$classifierDir = Join-Path $programFilesKidOS "MediaClassifier"
$guardianData = Join-Path $env:ProgramData "KidOS\Guardian"
$tokenPath = Join-Path $guardianData "media-classifier.token"

Assert-True (Test-Path $guardianDir) "Guardian directory is missing."
Assert-True (Test-Path $classifierDir) "Classifier directory is missing."
Assert-True (Test-Path $tokenPath) "Classifier token is missing."

foreach ($path in @($guardianDir, $classifierDir, $guardianData)) {
  $acl = Get-Acl $path
  $dangerous = $acl.Access | Where-Object {
    $_.AccessControlType -eq "Allow" -and
    $_.IdentityReference -match "(^|\\)(Users|Everyone|Authenticated Users)$" -and
    ($_.FileSystemRights.ToString() -match "Write|Modify|FullControl|Delete|ChangePermissions|TakeOwnership")
  }
  Assert-True (-not $dangerous) "Unprivileged users have write-like rights on protected KidOS path: $path"
}

$tokenAcl = Get-Acl $tokenPath
$tokenDanger = $tokenAcl.Access | Where-Object {
  $_.AccessControlType -eq "Allow" -and
  $_.IdentityReference -match "(^|\\)(Users|Everyone|Authenticated Users)$" -and
  ($_.FileSystemRights.ToString() -match "Read|Write|Modify|FullControl")
}
Assert-True (-not $tokenDanger) "Unprivileged users can access the media-classifier credential."

$guardianSvc = Get-CimInstance Win32_Service -Filter "Name='KidOSGuardian'"
$classifierSvc = Get-CimInstance Win32_Service -Filter "Name='KidOSMediaClassifier'"
Assert-True ($guardianSvc.StartName -eq "LocalSystem") "Guardian service account changed."
Assert-True ($classifierSvc.StartName -eq "LocalSystem") "Classifier service account changed."
Assert-True ($guardianSvc.StartMode -eq "Auto") "Guardian startup mode changed."
Assert-True ($classifierSvc.StartMode -eq "Auto") "Classifier startup mode changed."

$task = Get-ScheduledTask -TaskName "KidOS Guardian Recovery" -ErrorAction Stop
Assert-True ($task.Principal.UserId -eq "SYSTEM") "KidOS recovery task no longer runs as SYSTEM."

$unauthenticatedAccepted = $false
try {
  Invoke-WebRequest -Uri "http://127.0.0.1:8765/health" -TimeoutSec 3 | Out-Null
  $unauthenticatedAccepted = $true
} catch {
  if ($_.Exception.Response -and $_.Exception.Response.StatusCode.value__ -eq 401) {
    $unauthenticatedAccepted = $false
  }
}
Assert-True (-not $unauthenticatedAccepted) "Classifier accepted an unauthenticated request."

[pscustomobject]@{
  passed = $true
  guardianServiceAccount = $guardianSvc.StartName
  classifierServiceAccount = $classifierSvc.StartName
  recoveryTaskAccount = $task.Principal.UserId
  completedAt = (Get-Date).ToString("o")
} | ConvertTo-Json -Depth 4
