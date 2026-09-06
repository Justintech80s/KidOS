param(
  [Parameter(Mandatory=$true)][string]$InstallerPath,
  [string]$Repository = "Justintech80s/KidOS",
  [string]$SbomPredicate = "https://cyclonedx.org/bom"
)

$ErrorActionPreference = "Stop"

if (-not (Get-Command gh -ErrorAction SilentlyContinue)) {
  throw "GitHub CLI is required to verify KidOS release provenance."
}

$installer = (Resolve-Path $InstallerPath -ErrorAction Stop).Path
Write-Host "Verifying KidOS build provenance for $installer"
& gh attestation verify $installer --repo $Repository
if ($LASTEXITCODE -ne 0) {
  throw "KidOS provenance attestation verification failed."
}

Write-Host "Verifying KidOS SBOM attestation for $installer"
& gh attestation verify $installer --repo $Repository --predicate-type $SbomPredicate
if ($LASTEXITCODE -ne 0) {
  throw "KidOS SBOM attestation verification failed."
}

Write-Host "KidOS provenance and SBOM attestations verified."
