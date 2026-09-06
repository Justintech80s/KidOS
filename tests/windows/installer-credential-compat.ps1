$ErrorActionPreference = 'Stop'

# This test intentionally uses only APIs available to Windows PowerShell 5.1.
$bytes = New-Object byte[] 32
$rng = [System.Security.Cryptography.RandomNumberGenerator]::Create()
try {
    $rng.GetBytes($bytes)
} finally {
    $rng.Dispose()
}

$token = ([System.BitConverter]::ToString($bytes)).Replace('-', '').ToLowerInvariant()

if ($token.Length -ne 64) {
    throw "Guardian classifier token must be 64 lowercase hex characters; got length $($token.Length)."
}
if ($token -cnotmatch '^[0-9a-f]{64}$') {
    throw 'Guardian classifier token contains characters outside lowercase hexadecimal.'
}

$hooksPath = Join-Path $PSScriptRoot '..\..\apps\shell\src-tauri\windows\hooks.nsh'
$hooks = Get-Content -LiteralPath $hooksPath -Raw
if ($hooks -match '\[Convert\]::ToHexString') {
    throw 'Installer uses Convert.ToHexString, which is not compatible with Windows PowerShell 5.1/.NET Framework.'
}
if ($hooks -notmatch '\[System\.BitConverter\]::ToString\(\$b\)') {
    throw 'Installer does not use the verified Windows PowerShell 5.1-compatible token conversion.'
}

Write-Host 'KidOS installer Guardian credential generation is compatible with Windows PowerShell 5.1.'
