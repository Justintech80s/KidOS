param([string]$ResultPath = (Join-Path $env:ProgramData 'KidOS\Recovery\restore-windows.result'))
$ErrorActionPreference='Stop'
try {
 if (-not (Test-Path -LiteralPath $ResultPath)) { exit 40 }
 $result=(Get-Content -LiteralPath $ResultPath -Raw).Trim()
 if ($result -ne 'restored') { exit 40 }
 exit 0
} catch { exit 40 }
