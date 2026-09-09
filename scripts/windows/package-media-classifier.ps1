param(
  [string]$Python = "python"
)

$ErrorActionPreference = "Stop"

$serviceDir = Join-Path $PSScriptRoot "..\..\services\media-classifier"
$serviceDir = (Resolve-Path $serviceDir).Path
$modelDir = Join-Path $serviceDir "dist\model"
$bundleDir = Join-Path $serviceDir "dist\kidos-media-classifier"
$exePath = Join-Path $bundleDir "kidos-media-classifier.exe"
$payloadZip = Join-Path $serviceDir "dist\kidos-media-classifier-bundle.zip"

& $Python -m pip install --upgrade pip
if ($LASTEXITCODE -ne 0) { throw "pip upgrade failed." }

& $Python -m pip install -e "$serviceDir[packaging]"
if ($LASTEXITCODE -ne 0) { throw "KidOS media classifier dependency installation failed." }

& $Python -c "import app, windows_service, PyInstaller, huggingface_hub; print('KidOS classifier packaging imports verified')"
if ($LASTEXITCODE -ne 0) { throw "KidOS media classifier packaging imports are incomplete." }

& $Python -c "from huggingface_hub import snapshot_download; snapshot_download(repo_id='openai/clip-vit-base-patch32', local_dir=r'$($modelDir.Replace("'", "''"))')"
if ($LASTEXITCODE -ne 0) { throw "KidOS classifier model download failed." }

Push-Location $serviceDir
try {
  & $Python -m PyInstaller --clean --noconfirm kidos-media-classifier.spec
  if ($LASTEXITCODE -ne 0) { throw "PyInstaller failed to package the KidOS media classifier." }
} finally {
  Pop-Location
}

if (-not (Test-Path $exePath)) { throw "Classifier executable was not produced: $exePath" }
if (-not (Test-Path $bundleDir -PathType Container)) { throw "Classifier service bundle was not produced: $bundleDir" }
if (-not (Test-Path (Join-Path $modelDir "config.json"))) { throw "Classifier model bundle was not produced." }

$bundleFiles = @(Get-ChildItem $bundleDir -Recurse -File)
if ($bundleFiles.Count -lt 2) { throw "Classifier on-disk bundle is unexpectedly incomplete." }

if (Test-Path -LiteralPath $payloadZip) { Remove-Item -LiteralPath $payloadZip -Force }
Write-Host ("Compressing KidOS classifier service payload ({0} files)..." -f $bundleFiles.Count)
Compress-Archive -Path (Join-Path $bundleDir '*') -DestinationPath $payloadZip -CompressionLevel Optimal -Force
if (-not (Test-Path -LiteralPath $payloadZip)) { throw "Classifier service payload archive was not produced." }

$zipSize = (Get-Item -LiteralPath $payloadZip).Length
if ($zipSize -ge 1900000000) {
  throw ("Classifier service payload is still too large for reliable NSIS packaging: {0:N0} bytes." -f $zipSize)
}
Write-Host ("KidOS media classifier payload verified: {0} files; host={1}; compressed={2:N0} bytes" -f $bundleFiles.Count, $exePath, $zipSize)
