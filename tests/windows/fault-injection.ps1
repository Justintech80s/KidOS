[CmdletBinding()]
param([string]$ArtifactDir = 'target/fault-injection')

$ErrorActionPreference = 'Stop'
New-Item -ItemType Directory -Force -Path $ArtifactDir | Out-Null

function Assert-True([bool]$Condition, [string]$Message) {
  if (-not $Condition) { throw $Message }
}

$results = @()

function Add-Result([string]$Name, [bool]$Passed, [hashtable]$Observed) {
  $script:results += [ordered]@{
    name = $Name
    passed = $Passed
    observed = $Observed
    completed_at = (Get-Date).ToUniversalTime().ToString('o')
  }
}

try {
  $guardianBefore = Get-Service KidOSGuardian -ErrorAction Stop
  Stop-Service KidOSGuardian -Force -ErrorAction Stop
  Start-Sleep -Seconds 2
  $guardianAfter = Get-Service KidOSGuardian -ErrorAction Stop
  Assert-True ($guardianAfter.Status -ne 'Running') 'Guardian remained running after injected stop.'
  Add-Result 'guardian_stopped' $true @{ service_status = $guardianAfter.Status.ToString(); fail_closed_expected = $true }
}
finally {
  Start-Service KidOSGuardian -ErrorAction SilentlyContinue
  (Get-Service KidOSGuardian -ErrorAction Stop).WaitForStatus('Running', [TimeSpan]::FromSeconds(20))
}

try {
  $classifierBefore = Get-Service KidOSMediaClassifier -ErrorAction Stop
  Stop-Service KidOSMediaClassifier -Force -ErrorAction Stop
  Start-Sleep -Seconds 2
  $classifierAfter = Get-Service KidOSMediaClassifier -ErrorAction Stop
  Assert-True ($classifierAfter.Status -ne 'Running') 'Classifier remained running after injected stop.'
  Add-Result 'classifier_stopped' $true @{ service_status = $classifierAfter.Status.ToString(); media_ready_expected = $false }
}
finally {
  Start-Service KidOSMediaClassifier -ErrorAction SilentlyContinue
  (Get-Service KidOSMediaClassifier -ErrorAction Stop).WaitForStatus('Running', [TimeSpan]::FromSeconds(20))
}

$finalGuardian = Get-Service KidOSGuardian -ErrorAction Stop
$finalClassifier = Get-Service KidOSMediaClassifier -ErrorAction Stop
Assert-True ($finalGuardian.Status -eq 'Running') 'Guardian was not restored after fault injection.'
Assert-True ($finalClassifier.Status -eq 'Running') 'Classifier was not restored after fault injection.'

$output = [ordered]@{
  passed = ($results | Where-Object { -not $_.passed }).Count -eq 0
  scenarios = $results
  restored = @{ guardian = $finalGuardian.Status.ToString(); classifier = $finalClassifier.Status.ToString() }
  commit_sha = $env:GITHUB_SHA
}
$output | ConvertTo-Json -Depth 8 | Set-Content (Join-Path $ArtifactDir 'fault-injection.json') -Encoding UTF8
$output | ConvertTo-Json -Depth 8
