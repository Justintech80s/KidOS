param(
  [string]$Python = "python"
)

$ErrorActionPreference = "Stop"

$serviceDir = Join-Path $PSScriptRoot "..\..\services\media-classifier"
$serviceDir = (Resolve-Path $serviceDir).Path
$modelDir = Join-Path $serviceDir "dist\model"
$exePath = Join-Path $serviceDir "dist\kidos-media-classifier.exe"

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
if (-not (Test-Path (Join-Path $modelDir "config.json"))) { throw "Classifier model bundle was not produced." }

Write-Host "KidOS media classifier package verified: $exePath"
