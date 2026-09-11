param(
    [Parameter(Mandatory = $true)][string]$ArchivePath,
    [Parameter(Mandatory = $true)][string]$DestinationPath
)

$ErrorActionPreference = 'Stop'
$StagingPath = "$DestinationPath.staging-$([Guid]::NewGuid().ToString('N'))"

try {
    if (-not (Test-Path -LiteralPath $ArchivePath -PathType Leaf)) {
        throw "Classifier payload archive is missing: $ArchivePath"
    }

    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $archive = $null
    try {
        $archive = [System.IO.Compression.ZipFile]::OpenRead($ArchivePath)
        if ($archive.Entries.Count -eq 0) { throw 'Classifier payload archive is empty.' }
    } finally {
        if ($null -ne $archive) { $archive.Dispose() }
    }

    $parent = Split-Path -Parent $DestinationPath
    New-Item -ItemType Directory -Force -Path $parent | Out-Null
    Remove-Item -LiteralPath $StagingPath -Recurse -Force -ErrorAction SilentlyContinue
    New-Item -ItemType Directory -Force -Path $StagingPath | Out-Null

    Expand-Archive -LiteralPath $ArchivePath -DestinationPath $StagingPath -Force

    $hostPath = Join-Path $StagingPath 'kidos-media-classifier.exe'
    if (-not (Test-Path -LiteralPath $hostPath -PathType Leaf)) {
        throw "Classifier host is missing after payload extraction: $hostPath"
    }

    if (Test-Path -LiteralPath $DestinationPath) {
        Remove-Item -LiteralPath $DestinationPath -Recurse -Force
    }
    Move-Item -LiteralPath $StagingPath -Destination $DestinationPath

    $finalHost = Join-Path $DestinationPath 'kidos-media-classifier.exe'
    if (-not (Test-Path -LiteralPath $finalHost -PathType Leaf)) {
        throw "Classifier host is missing after staging promotion: $finalHost"
    }

    exit 0
} catch {
    Remove-Item -LiteralPath $StagingPath -Recurse -Force -ErrorAction SilentlyContinue
    Write-Error $_
    exit 1
}
