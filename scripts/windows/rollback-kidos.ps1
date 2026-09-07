param(
    [string]$RecoveryRoot = (Join-Path $env:ProgramData 'KidOS\Recovery')
)

$ErrorActionPreference = 'Stop'

$previous = Join-Path $RecoveryRoot 'Previous\KidOS-previous.exe'
$hashFile = Join-Path $RecoveryRoot 'Previous\KidOS-previous.sha256'

if (-not (Test-Path -LiteralPath $previous) -or -not (Test-Path -LiteralPath $hashFile)) {
    throw 'No verified previous KidOS installer is available for rollback.'
}

$expected = (Get-Content -LiteralPath $hashFile -Raw).Trim().ToLowerInvariant()
if ($expected -notmatch '^[0-9a-f]{64}$') {
    throw 'Previous KidOS installer hash record is invalid.'
}

$actual = (Get-FileHash -LiteralPath $previous -Algorithm SHA256).Hash.ToLowerInvariant()
if ($actual -ne $expected) {
    throw 'KidOS blocked rollback because the previous installer failed SHA-256 verification.'
}

$signature = Get-AuthenticodeSignature -LiteralPath $previous
if ($signature.Status -notin @('Valid', 'NotSigned')) {
    throw "KidOS blocked rollback because the previous installer signature state is $($signature.Status)."
}

# Experimental builds may be unsigned. Production builds are expected to be
# Authenticode-signed and are additionally protected by the Guardian publisher pin.
$process = Start-Process -FilePath $previous -ArgumentList '/S' -Wait -PassThru
if ($process.ExitCode -ne 0) {
    throw "Previous KidOS installer exited with code $($process.ExitCode)."
}

Write-Host 'KidOS rollback installer completed.'
