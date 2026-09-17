[CmdletBinding()]
param(
  [string]$ArtifactDir = "target/ci-diagnostics",
  [string]$Stage = "unknown",
  [string]$FailureClass = "unknown"
)

$ErrorActionPreference = "Continue"
New-Item -ItemType Directory -Force -Path $ArtifactDir | Out-Null

function Redact-Text([string]$Text) {
  if ($null -eq $Text) { return "" }
  $patterns = @(
    '(?i)(authorization\s*[:=]\s*)([^\s]+)',
    '(?i)(cookie\s*[:=]\s*)([^\r\n]+)',
    '(?i)(password\s*[:=]\s*)([^\s]+)',
    '(?i)(token\s*[:=]\s*)([^\s]+)'
  )
  $redacted = $Text
  foreach ($pattern in $patterns) {
    $redacted = [regex]::Replace($redacted, $pattern, '$1[REDACTED]')
  }
  return $redacted
}

function Write-RedactedFile([string]$Path, [object]$Value) {
  $text = if ($Value -is [string]) { $Value } else { $Value | Out-String }
  Redact-Text $text | Set-Content -Path $Path -Encoding UTF8
}

$os = Get-CimInstance Win32_OperatingSystem -ErrorAction SilentlyContinue
$metadata = [ordered]@{
  commit_sha = $env:GITHUB_SHA
  ref = $env:GITHUB_REF
  workflow = $env:GITHUB_WORKFLOW
  run_id = $env:GITHUB_RUN_ID
  run_attempt = $env:GITHUB_RUN_ATTEMPT
  job = $env:GITHUB_JOB
  stage = $Stage
  failure_class = $FailureClass
  windows_caption = $os.Caption
  windows_version = $os.Version
  generated_at = (Get-Date).ToUniversalTime().ToString('o')
}
$metadata | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $ArtifactDir 'metadata.json') -Encoding UTF8

$services = Get-CimInstance Win32_Service -ErrorAction SilentlyContinue |
  Where-Object { $_.Name -in @('KidOSGuardian', 'KidOSMediaClassifier') } |
  Select-Object Name, State, StartMode, StartName, ProcessId
Write-RedactedFile (Join-Path $ArtifactDir 'services.txt') $services

$processes = Get-Process -ErrorAction SilentlyContinue |
  Where-Object { $_.ProcessName -match 'KidOS|Guardian|Classifier' } |
  Select-Object ProcessName, Id, StartTime
Write-RedactedFile (Join-Path $ArtifactDir 'processes.txt') $processes

$task = Get-ScheduledTask -TaskName 'KidOS Guardian Recovery' -ErrorAction SilentlyContinue |
  Select-Object TaskName, State, TaskPath
Write-RedactedFile (Join-Path $ArtifactDir 'recovery-task.txt') $task

$since = (Get-Date).AddMinutes(-30)
$scm = Get-WinEvent -FilterHashtable @{ LogName='System'; ProviderName='Service Control Manager'; StartTime=$since } -ErrorAction SilentlyContinue |
  Where-Object { $_.Message -match 'KidOSGuardian|KidOSMediaClassifier' } |
  Select-Object TimeCreated, Id, LevelDisplayName, Message
Write-RedactedFile (Join-Path $ArtifactDir 'service-control-manager-events.txt') $scm

$app = Get-WinEvent -FilterHashtable @{ LogName='Application'; StartTime=$since } -ErrorAction SilentlyContinue |
  Where-Object { $_.Message -match 'KidOS|guardian|classifier' } |
  Select-Object TimeCreated, ProviderName, Id, LevelDisplayName, Message
Write-RedactedFile (Join-Path $ArtifactDir 'application-events.txt') $app

$installed = Get-ChildItem "$env:ProgramFiles\KidOS" -Recurse -ErrorAction SilentlyContinue |
  Select-Object FullName, Length, LastWriteTime
Write-RedactedFile (Join-Path $ArtifactDir 'installed-files.txt') $installed

$sourceDiagnostics = Join-Path $env:ProgramData 'KidOS\Diagnostics'
$copied = @()
if (Test-Path $sourceDiagnostics) {
  $dest = Join-Path $ArtifactDir 'programdata-diagnostics'
  New-Item -ItemType Directory -Force -Path $dest | Out-Null
  Get-ChildItem $sourceDiagnostics -File -ErrorAction SilentlyContinue | ForEach-Object {
    $target = Join-Path $dest $_.Name
    Write-RedactedFile $target (Get-Content $_.FullName -Raw -ErrorAction SilentlyContinue)
    $copied += $_.Name
  }
}

$summary = [ordered]@{
  metadata_present = Test-Path (Join-Path $ArtifactDir 'metadata.json')
  guardian_present = [bool](Get-Service KidOSGuardian -ErrorAction SilentlyContinue)
  classifier_present = [bool](Get-Service KidOSMediaClassifier -ErrorAction SilentlyContinue)
  programdata_files = $copied
  generated_at = (Get-Date).ToUniversalTime().ToString('o')
}
$summary | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $ArtifactDir 'summary.json') -Encoding UTF8
