param(
  [Parameter(Mandatory=$true)][string]$Path
)

$ErrorActionPreference = "Stop"

$pfx = $env:KIDOS_WINDOWS_PFX_BASE64
$password = $env:KIDOS_WINDOWS_PFX_PASSWORD

if ([string]::IsNullOrWhiteSpace($pfx) -or [string]::IsNullOrWhiteSpace($password)) {
  throw "KidOS release signing secrets are not configured."
}

& "$PSScriptRoot\sign-release-artifacts.ps1" -Files @($Path) -PfxBase64 $pfx -PfxPassword $password
if ($LASTEXITCODE -ne 0) {
  throw "KidOS Tauri signing command failed for $Path."
}
