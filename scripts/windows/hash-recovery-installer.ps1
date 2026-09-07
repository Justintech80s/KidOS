param(
 [Parameter(Mandatory=$true)][string]$InstallerPath,
 [Parameter(Mandatory=$true)][string]$HashPath
)
$ErrorActionPreference='Stop'
try {
 if (-not (Test-Path -LiteralPath $InstallerPath)) { throw "Installer missing: $InstallerPath" }
 $hash=(Get-FileHash -LiteralPath $InstallerPath -Algorithm SHA256).Hash.ToLowerInvariant()
 $dir=Split-Path -Parent $HashPath
 New-Item -ItemType Directory -Force -Path $dir | Out-Null
 Set-Content -LiteralPath $HashPath -Value $hash -NoNewline -Encoding ASCII
 exit 0
} catch { Write-Error $_; exit 1 }
