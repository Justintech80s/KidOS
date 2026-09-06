param(
  [string]$ChildUser = "KidOSInteractiveChild",
  [string]$ExpectedKidOSExe = "KidOS.exe"
)

$ErrorActionPreference = "Stop"

function Assert-True([bool]$Condition, [string]$Message) {
  if (-not $Condition) { throw $Message }
}

$guardian = Get-Service KidOSGuardian -ErrorAction Stop
$classifier = Get-Service KidOSMediaClassifier -ErrorAction Stop
Assert-True ($guardian.Status -eq "Running") "Guardian is not running after reboot."
Assert-True ($classifier.Status -eq "Running") "Media classifier is not running after reboot."

$assignedAccess = Get-CimInstance -Namespace "root\cimv2\mdm\dmmap" -ClassName "MDM_AssignedAccess" -ErrorAction SilentlyContinue
Assert-True ($null -ne $assignedAccess) "Assigned Access provider is unavailable."
Assert-True (-not [string]::IsNullOrWhiteSpace($assignedAccess.Configuration)) "Assigned Access has no active configuration."
Assert-True ($assignedAccess.Configuration -match [Regex]::Escape($ChildUser)) "Assigned Access configuration does not target the expected child account."

$interactiveLogonFound = $false
try {
  $events = Get-WinEvent -FilterHashtable @{ LogName='Security'; Id=4624; StartTime=(Get-Date).AddHours(-6) } -ErrorAction Stop
  foreach ($event in $events) {
    $xml = [xml]$event.ToXml()
    $data = @{}
    foreach ($node in $xml.Event.EventData.Data) { $data[$node.Name] = $node.'#text' }
    if ($data.TargetUserName -eq $ChildUser -and $data.LogonType -in @('2','10')) {
      $interactiveLogonFound = $true
      break
    }
  }
} catch {
  Write-Warning "Security log inspection unavailable: $($_.Exception.Message)"
}
Assert-True $interactiveLogonFound "No recent interactive child-account logon was found."

$processes = Get-CimInstance Win32_Process -Filter "Name='$ExpectedKidOSExe'" -ErrorAction SilentlyContinue
$childProcessFound = $false
foreach ($process in $processes) {
  try {
    $owner = Invoke-CimMethod -InputObject $process -MethodName GetOwner
    if ($owner.User -eq $ChildUser) {
      $childProcessFound = $true
      break
    }
  } catch {}
}
Assert-True $childProcessFound "KidOS is not running in the child account's interactive session."

$result = [ordered]@{
  passed = $true
  childUser = $ChildUser
  guardianRunning = $true
  classifierRunning = $true
  assignedAccessConfigured = $true
  interactiveChildLogonFound = $true
  kidOSRunningInChildSession = $true
  verifiedAt = (Get-Date).ToString("o")
}
$result | ConvertTo-Json
