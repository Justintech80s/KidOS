param(
    [Parameter(Mandatory = $true)]
    [string]$InstallerPath,

    [string]$GuardianDataDir = (Join-Path $env:ProgramData 'KidOS\Guardian'),

    [switch]$SkipAcl
)

$ErrorActionPreference = 'Stop'
$diagnosticsDir = Join-Path (Split-Path $GuardianDataDir -Parent) 'Diagnostics'
$diagnosticsLog = Join-Path $diagnosticsDir 'credential-provisioning.log'
New-Item -ItemType Directory -Force -Path $diagnosticsDir | Out-Null

function Write-ProvisioningDiagnostic([string]$Message) {
    $line = ('{0:u} {1}' -f (Get-Date), $Message)
    Add-Content -LiteralPath $diagnosticsLog -Value $line -Encoding UTF8
    Write-Host $line
}

try {
    Write-ProvisioningDiagnostic ("Starting Guardian credential provisioning. DataDir={0}" -f $GuardianDataDir)
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
    Write-ProvisioningDiagnostic ("Created classifier token at {0}." -f $tokenPath)

    if (Test-Path -LiteralPath $InstallerPath) {
        $signature = Get-AuthenticodeSignature -LiteralPath $InstallerPath
        if ($signature.Status -eq 'Valid' -and $signature.SignerCertificate) {
            $thumbprintPath = Join-Path $GuardianDataDir 'publisher-thumbprint.txt'
            Set-Content -LiteralPath $thumbprintPath -Value $signature.SignerCertificate.Thumbprint -NoNewline -Encoding ASCII
        }
    }

    if (-not $SkipAcl) {
        # Use well-known SIDs instead of localized account names. This remains
        # stable on GitHub-hosted Windows images and non-English Windows installs.
        Write-ProvisioningDiagnostic 'Applying Guardian ACL with SYSTEM and Builtin Administrators SIDs.'
        & "$env:SystemRoot\System32\icacls.exe" $GuardianDataDir /inheritance:r /grant:r '*S-1-5-18:(OI)(CI)(F)' '*S-1-5-32-544:(OI)(CI)(F)' | Out-Null
        $aclExit = $LASTEXITCODE
        Write-ProvisioningDiagnostic ("icacls exited with code {0}." -f $aclExit)
        if ($aclExit -ne 0) {
            throw "icacls failed with exit code $aclExit."
        }
    }

    Write-ProvisioningDiagnostic 'Guardian credential provisioning completed successfully.'
    exit 0
} catch {
    try { Write-ProvisioningDiagnostic ("Guardian credential provisioning failed: {0}" -f $_.Exception.Message) } catch {}
    Write-Error $_
    exit 1
}
