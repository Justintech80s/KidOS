$ErrorActionPreference = "Stop"

$guardian = "$env:ProgramFiles\KidOS\Guardian\kidos-guardian-host.exe"

if (-not (Test-Path -LiteralPath $guardian -PathType Leaf)) {
    throw "SHADOW CRITICAL: Guardian executable is missing."
}

$acl = Get-Acl $guardian

$dangerous = $acl.Access | Where-Object {
    $_.IdentityReference -match "Users$" -and
    $_.AccessControlType -eq "Allow" -and
    $_.FileSystemRights.ToString() -match "Write|Modify|FullControl"
}

if ($dangerous) {
    throw "SHADOW CRITICAL: Standard Users can modify Guardian."
}

Write-Host "Shadow Guardian tamper protection PASSED."
