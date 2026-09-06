param(
  [Parameter(Mandatory=$true)][string]$OutputPath
)

$ErrorActionPreference = "Stop"

$cargo = cargo metadata --format-version 1 --locked 2>$null | ConvertFrom-Json
$components = @()

foreach ($pkg in $cargo.packages) {
  $components += [ordered]@{
    type = "library"
    name = $pkg.name
    version = $pkg.version
    purl = "pkg:cargo/$($pkg.name)@$($pkg.version)"
  }
}

$pythonProject = "services/media-classifier/pyproject.toml"
if (Test-Path $pythonProject) {
  $lines = Get-Content $pythonProject
  $inDependencies = $false
  foreach ($line in $lines) {
    if ($line -match '^dependencies\s*=\s*\[') { $inDependencies = $true; continue }
    if ($inDependencies -and $line -match '^\]') { $inDependencies = $false; continue }
    if ($inDependencies -and $line -match '"([A-Za-z0-9_.-]+)([^"]*)"') {
      $name = $matches[1]
      $constraint = $matches[2]
      $components += [ordered]@{
        type = "library"
        name = $name
        version = $constraint.Trim()
        purl = "pkg:pypi/$name"
      }
    }
  }
}

$unique = $components | Sort-Object name,version -Unique
$sbom = [ordered]@{
  bomFormat = "CycloneDX"
  specVersion = "1.5"
  serialNumber = "urn:uuid:$([Guid]::NewGuid())"
  version = 1
  metadata = [ordered]@{
    timestamp = (Get-Date).ToUniversalTime().ToString("o")
    component = [ordered]@{
      type = "application"
      name = "KidOS"
    }
  }
  components = @($unique)
}

$directory = Split-Path $OutputPath -Parent
if ($directory) { New-Item -ItemType Directory -Force -Path $directory | Out-Null }
$sbom | ConvertTo-Json -Depth 12 | Set-Content $OutputPath -Encoding UTF8
Write-Host "Created CycloneDX SBOM with $($unique.Count) components at $OutputPath"
