param(
  [Parameter(Mandatory=$true)][string[]]$Files
)

$ErrorActionPreference = "Stop"

foreach ($file in $Files) {
  $resolved = (Resolve-Path $file -ErrorAction Stop).Path
  $signature = Get-AuthenticodeSignature $resolved
  if ($signature.Status -ne "Valid") {
    throw "KidOS release artifact is not trusted-signed: $resolved (status: $($signature.Status))"
  }
  if (-not $signature.SignerCertificate) {
    throw "KidOS release artifact has no signer certificate: $resolved"
  }

  [pscustomobject]@{
    File = $resolved
    Status = $signature.Status
    Subject = $signature.SignerCertificate.Subject
    Thumbprint = $signature.SignerCertificate.Thumbprint
    NotAfter = $signature.SignerCertificate.NotAfter
  } | Format-List
}
