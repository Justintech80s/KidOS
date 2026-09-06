param(
  [Parameter(Mandatory=$true)][string]$Version,
  [Parameter(Mandatory=$true)][string]$InstallerPath,
  [Parameter(Mandatory=$true)][string]$OutputPath,
  [string]$Repository = "Justintech80s/KidOS"
)

$ErrorActionPreference = "Stop"
$installer = Get-Item $InstallerPath -ErrorAction Stop
$hash = Get-FileHash $installer.FullName -Algorithm SHA256
$sig = Get-AuthenticodeSignature $installer.FullName

if ($sig.Status -ne "Valid") {
  throw "Refusing to create a trusted update manifest for an unsigned or invalid installer."
}

$tag = "v$Version"
$downloadUrl = "https://github.com/$Repository/releases/download/$tag/$($installer.Name)"

$payload = [ordered]@{
  schemaVersion = 1
  product = "KidOS"
  version = $Version
  publishedAt = (Get-Date).ToUniversalTime().ToString("o")
  platform = "windows-x64"
  installer = [ordered]@{
    fileName = $installer.Name
    url = $downloadUrl
    sha256 = $hash.Hash.ToLowerInvariant()
    sizeBytes = $installer.Length
    authenticode = [ordered]@{
      status = $sig.Status.ToString()
      signerSubject = $sig.SignerCertificate.Subject
      signerThumbprint = $sig.SignerCertificate.Thumbprint
    }
  }
}

$payload | ConvertTo-Json -Depth 8 | Set-Content $OutputPath -Encoding UTF8
Get-Content $OutputPath
