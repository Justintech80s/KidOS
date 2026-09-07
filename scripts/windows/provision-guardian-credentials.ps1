param(
    [Parameter(Mandatory = $true)]
    [string]$InstallerPath,

    [string]$GuardianDataDir = (Join-Path $env:ProgramData 'KidOS\Guardian'),

    [switch]$SkipAcl
)

$ErrorActionPreference = 'Stop'

try {
    New-Item -ItemType Directory -Force -Path $GuardianDataDir | Out-Null

    $bytes = New-Object byte[] 32
    $rng = [System.Security.Cryptography.RandomNumberGenerator]::Create()
    try {
        $rng.GetBytes($bytes)
    } finally {
        $rng.Dispose()
    }

    $token = ([System.BitConverter]::ToString($bytes)).Replace('-', '').ToLowerInvariant()
    if ($token.Length -ne 64 -or $token -cnotmatch '^[0-9a-f]{64}$') {
        throw 'Generated Guardian token failed validation.'
    }

    $tokenPath = Join-Path $GuardianDataDir 'media-classifier.token'
    Set-Content -LiteralPath $tokenPath -Value $token -NoNewline -Encoding ASCII

    if (Test-Path -LiteralPath $InstallerPath) {
        $signature = Get-AuthenticodeSignature -LiteralPath $InstallerPath
        if ($signature.Status -eq 'Valid' -and $signature.SignerCertificate) {
            $thumbprintPath = Join-Path $GuardianDataDir 'publisher-thumbprint.txt'
            Set-Content -LiteralPath $thumbprintPath -Value $signature.SignerCertificate.Thumbprint -NoNewline -Encoding ASCII
        }
    }

    if (-not $SkipAcl) {
        & "$env:SystemRoot\System32\icacls.exe" $GuardianDataDir /inheritance:r /grant:r 'SYSTEM:(OI)(CI)(F)' 'Administrators:(OI)(CI)(F)' | Out-Null
        if ($LASTEXITCODE -ne 0) {
            throw "icacls failed with exit code $LASTEXITCODE."
        }
    }

    exit 0
} catch {
    Write-Error $_
    exit 1
}
