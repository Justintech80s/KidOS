param(
  [Parameter(Mandatory=$true)][string[]]$Files,
  [Parameter(Mandatory=$true)][string]$PfxBase64,
  [Parameter(Mandatory=$true)][string]$PfxPassword,
  [string]$TimestampUrl = "http://timestamp.digicert.com"
)

$ErrorActionPreference = "Stop"

function Find-SignTool {
  $roots = @(
    "${env:ProgramFiles(x86)}\Windows Kits\10\bin",
    "${env:ProgramFiles}\Windows Kits\10\bin"
  )
  foreach ($root in $roots) {
    if (-not (Test-Path $root)) { continue }
    $candidate = Get-ChildItem $root -Recurse -Filter signtool.exe -ErrorAction SilentlyContinue |
      Where-Object { $_.FullName -match "\\x64\\signtool\.exe$" } |
      Sort-Object FullName -Descending |
      Select-Object -First 1
    if ($candidate) { return $candidate.FullName }
  }
  throw "Windows SDK signtool.exe was not found."
}

if ([string]::IsNullOrWhiteSpace($PfxBase64)) { throw "KidOS signing certificate is missing." }
if ([string]::IsNullOrWhiteSpace($PfxPassword)) { throw "KidOS signing certificate password is missing." }

$tempPfx = Join-Path $env:RUNNER_TEMP ("kidos-signing-" + [Guid]::NewGuid().ToString("N") + ".pfx")
try {
  [IO.File]::WriteAllBytes($tempPfx, [Convert]::FromBase64String($PfxBase64))
  $signTool = Find-SignTool

  foreach ($file in $Files) {
    $resolved = (Resolve-Path $file -ErrorAction Stop).Path
    Write-Host "Signing $resolved"
    & $signTool sign /fd SHA256 /f $tempPfx /p $PfxPassword /tr $TimestampUrl /td SHA256 $resolved
    if ($LASTEXITCODE -ne 0) { throw "Authenticode signing failed for $resolved." }

    & $signTool verify /pa /all $resolved
    if ($LASTEXITCODE -ne 0) { throw "Authenticode verification failed for $resolved." }
  }
} finally {
  if (Test-Path $tempPfx) {
    Remove-Item $tempPfx -Force
  }
}
