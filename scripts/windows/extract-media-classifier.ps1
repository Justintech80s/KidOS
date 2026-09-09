param(
    [Parameter(Mandatory = $true)][string]$ArchivePath,
    [Parameter(Mandatory = $true)][string]$DestinationPath
)

$ErrorActionPreference = 'Stop'

try {
    if (-not (Test-Path -LiteralPath $ArchivePath -PathType Leaf)) {
        throw "Classifier payload archive is missing: $ArchivePath"
    }

    New-Item -ItemType Directory -Force -Path $DestinationPath | Out-Null
    Expand-Archive -LiteralPath $ArchivePath -DestinationPath $DestinationPath -Force

    $hostPath = Join-Path $DestinationPath 'kidos-media-classifier.exe'
    if (-not (Test-Path -LiteralPath $hostPath -PathType Leaf)) {
        throw "Classifier host is missing after payload extraction: $hostPath"
    }

    exit 0
} catch {
    Write-Error $_
    exit 1
}
