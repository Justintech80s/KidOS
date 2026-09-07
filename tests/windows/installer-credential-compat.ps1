$ErrorActionPreference = 'Stop'

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot '..\..')
$scriptPath = Join-Path $repoRoot 'scripts\windows\provision-guardian-credentials.ps1'
$hooksPath = Join-Path $repoRoot 'apps\shell\src-tauri\windows\hooks.nsh'

if (-not (Test-Path -LiteralPath $scriptPath)) {
    throw "Guardian provisioning script is missing: $scriptPath"
}

$tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("kidos-guardian-test-" + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Force -Path $tempRoot | Out-Null

try {
    # Run the exact script shipped by the installer under Windows PowerShell 5.1.
    & $scriptPath -InstallerPath $scriptPath -GuardianDataDir $tempRoot -SkipAcl
    if ($LASTEXITCODE -ne 0) {
        throw "Guardian provisioning script failed with exit code $LASTEXITCODE."
    }

    $tokenPath = Join-Path $tempRoot 'media-classifier.token'
    if (-not (Test-Path -LiteralPath $tokenPath)) {
        throw 'Guardian provisioning did not create media-classifier.token.'
    }

    $token = Get-Content -LiteralPath $tokenPath -Raw
    if ($token.Length -ne 64) {
        throw "Guardian classifier token must be 64 lowercase hex characters; got length $($token.Length)."
    }
    if ($token -cnotmatch '^[0-9a-f]{64}$') {
        throw 'Guardian classifier token contains characters outside lowercase hexadecimal.'
    }

    $hooks = Get-Content -LiteralPath $hooksPath -Raw
    if ($hooks -match '\[Convert\]::ToHexString') {
        throw 'Installer still contains Convert.ToHexString, which is not compatible with Windows PowerShell 5.1/.NET Framework.'
    }
    if ($hooks -notmatch 'provision-guardian-credentials\.ps1') {
        throw 'Installer does not ship and invoke the standalone Guardian provisioning script.'
    }
    if ($hooks -notmatch '-File .*provision-guardian-credentials\.ps1') {
        throw 'Installer must invoke Guardian credential provisioning with PowerShell -File instead of inline -Command.'
    }

    Write-Host 'KidOS Guardian credential provisioning passed under Windows PowerShell 5.1.'
} finally {
    Remove-Item -LiteralPath $tempRoot -Recurse -Force -ErrorAction SilentlyContinue
}
