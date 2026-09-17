param(
  [string]$InstallerPath,
  [string]$ExpectedKidOSExe = "KidOS.exe",
  [ValidateSet("normal", "restricted", "unknown")]
  [string]$DetectedUiState = "unknown"
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

function Install-KidOSIfRequested {
  if ([string]::IsNullOrWhiteSpace($InstallerPath)) { return $false }
  if (-not (Test-Path -LiteralPath $InstallerPath)) {
    throw "KidOS installer not found: $InstallerPath"
  }

  $process = Start-Process -FilePath $InstallerPath -ArgumentList "/S" -Wait -PassThru
  if ($process.ExitCode -ne 0) {
    throw "KidOS installer failed with exit code $($process.ExitCode)."
  }
  return $true
}

function Invoke-GuardianRecoveryStatus {
  $pipe = New-Object System.IO.Pipes.NamedPipeClientStream(
    ".",
    "KidOSGuardian.v1",
    [System.IO.Pipes.PipeDirection]::InOut,
    [System.IO.Pipes.PipeOptions]::None
  )

  try {
    $pipe.Connect(5000)
    $payload = [ordered]@{
      version = 1
      session_id = "visual-vm-$PID-$([DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds())"
      nonce = [Guid]::NewGuid().ToString("N")
      request = [ordered]@{ type = "recovery_status" }
    } | ConvertTo-Json -Compress -Depth 5

    $bytes = [Text.Encoding]::UTF8.GetBytes($payload)
    $pipe.Write($bytes, 0, $bytes.Length)
    $pipe.Flush()

    $buffer = New-Object byte[] 65536
    $memory = New-Object System.IO.MemoryStream
    do {
      $read = $pipe.Read($buffer, 0, $buffer.Length)
      if ($read -gt 0) { $memory.Write($buffer, 0, $read) }
    } while ($read -gt 0)

    $responseText = [Text.Encoding]::UTF8.GetString($memory.ToArray())
    if ([string]::IsNullOrWhiteSpace($responseText)) {
      throw "Guardian IPC returned an empty recovery-status response."
    }
    return $responseText | ConvertFrom-Json
  }
  finally {
    $pipe.Dispose()
  }
}

$installed = Install-KidOSIfRequested
$guardianServiceRunning = $false
$guardianIpcHealthy = $false
$policyValid = $false
$guardianHealthy = $false
$recoveryRequired = $true
$recoveryReason = $null
$lockdownState = $null
$ipcError = $null

$guardian = Get-Service KidOSGuardian -ErrorAction SilentlyContinue
if ($null -ne $guardian -and $guardian.Status -eq "Running") {
  $guardianServiceRunning = $true
  try {
    $recovery = Invoke-GuardianRecoveryStatus
    if ($recovery.type -eq "recovery_status") {
      $guardianIpcHealthy = $true
      $guardianHealthy = [bool]$recovery.guardian_healthy
      $policyValid = [bool]$recovery.policy_valid
      $recoveryRequired = [bool]$recovery.recovery_required
      $recoveryReason = $recovery.recovery_reason
      $lockdownState = [string]$recovery.lockdown_state
    } else {
      throw "Guardian IPC returned an unexpected response type."
    }
  }
  catch {
    $ipcError = $_.Exception.Message
  }
}

$uiState = $DetectedUiState
if (-not $guardianServiceRunning) {
  $guardianServiceRunning = $false
  $uiState = "restricted"
}
if (-not $policyValid) {
  $uiState = "restricted"
}

$kidOSRunning = $false
$processName = [IO.Path]::GetFileNameWithoutExtension($ExpectedKidOSExe)
if (-not [string]::IsNullOrWhiteSpace($processName)) {
  $kidOSRunning = $null -ne (Get-Process -Name $processName -ErrorAction SilentlyContinue | Select-Object -First 1)
}

$contradiction = $null
if ($guardianHealthy -and $policyValid -and $uiState -eq "restricted") {
  $contradiction = "healthy_guardian_showed_restricted_ui"
}
if ((-not $guardianHealthy -or -not $policyValid) -and $uiState -eq "normal") {
  $contradiction = "unhealthy_guardian_showed_normal_ui"
}

$passed = $guardianServiceRunning -and $guardianIpcHealthy -and $guardianHealthy -and $policyValid -and ($null -eq $contradiction)

$result = [ordered]@{
  installedThisRun = $installed
  guardianServiceRunning = $guardianServiceRunning
  guardianIpcHealthy = $guardianIpcHealthy
  guardianHealthy = $guardianHealthy
  policyValid = $policyValid
  recoveryRequired = $recoveryRequired
  recoveryReason = $recoveryReason
  lockdownState = $lockdownState
  kidOSRunning = $kidOSRunning
  uiState = $uiState
  contradiction = $contradiction
  ipcError = $ipcError
  passed = $passed
  checkedAt = (Get-Date).ToString("o")
}

$result | ConvertTo-Json -Depth 6
