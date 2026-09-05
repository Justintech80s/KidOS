$ErrorActionPreference = "Stop"
$serviceName = "KidOSGuardian"
$installDir = "$env:ProgramFiles\KidOS\Guardian"

$principal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
  throw "Run this uninstaller from an elevated PowerShell window."
}

if (Get-Service -Name $serviceName -ErrorAction SilentlyContinue) {
  Stop-Service -Name $serviceName -Force -ErrorAction SilentlyContinue
  sc.exe delete $serviceName | Out-Null
  Start-Sleep -Seconds 1
}

if (Test-Path $installDir) {
  Remove-Item -Recurse -Force $installDir
}

Write-Host "KidOS Guardian service removed."
