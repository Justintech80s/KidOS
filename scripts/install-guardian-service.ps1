param(
  [string]$SourceExe = "$(Resolve-Path "$PSScriptRoot\..\target\release\kidos-guardian-host.exe")",
  [string]$InstallDir = "$env:ProgramFiles\KidOS\Guardian"
)

$ErrorActionPreference = "Stop"

$principal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
  throw "Run this installer from an elevated PowerShell window."
}

if (-not (Test-Path $SourceExe)) {
  throw "Guardian host executable not found: $SourceExe"
}

$serviceName = "KidOSGuardian"
$targetExe = Join-Path $InstallDir "kidos-guardian-host.exe"

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

if (Get-Service -Name $serviceName -ErrorAction SilentlyContinue) {
  Stop-Service -Name $serviceName -Force -ErrorAction SilentlyContinue
  sc.exe delete $serviceName | Out-Null
  Start-Sleep -Seconds 1
}

Copy-Item -Force $SourceExe $targetExe

# Keep the Guardian binary writable only by SYSTEM and Administrators.
icacls $InstallDir /inheritance:r | Out-Null
icacls $InstallDir /grant:r "SYSTEM:(OI)(CI)(F)" "Administrators:(OI)(CI)(F)" "Users:(OI)(CI)(RX)" | Out-Null

$binPath = '"' + $targetExe + '"'
sc.exe create $serviceName binPath= $binPath start= auto obj= LocalSystem DisplayName= "KidOS Guardian" | Out-Null
sc.exe description $serviceName "Privileged KidOS Guardian service for Windows safety and Assigned Access enforcement." | Out-Null
sc.exe failure $serviceName reset= 86400 actions= restart/5000/restart/5000/restart/5000 | Out-Null
sc.exe failureflag $serviceName 1 | Out-Null

Start-Service -Name $serviceName
$service = Get-Service -Name $serviceName
if ($service.Status -ne "Running") {
  throw "KidOS Guardian did not reach the Running state."
}

Write-Host "KidOS Guardian installed and running as LocalSystem."
