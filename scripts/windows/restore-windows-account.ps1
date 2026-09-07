param(
    [string]$ResultPath = (Join-Path $env:ProgramData 'KidOS\Recovery\restore-windows.result')
)

$ErrorActionPreference = 'Stop'

function Write-Result([string]$Value) {
    $dir = Split-Path -Parent $ResultPath
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    Set-Content -LiteralPath $ResultPath -Value $Value -NoNewline -Encoding ASCII
}

try {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    if (-not $identity.IsSystem) {
        throw 'KidOS Windows recovery must run as LocalSystem.'
    }

    $namespace = 'root\cimv2\mdm\dmmap'
    $instances = @(Get-CimInstance -Namespace $namespace -ClassName MDM_AssignedAccess -ErrorAction SilentlyContinue)

    foreach ($instance in $instances) {
        try {
            Remove-CimInstance -InputObject $instance -ErrorAction Stop
        } catch {
            # Some Windows builds expose the singleton but reject deletion. Clearing
            # Configuration removes the KidOS Assigned Access payload without touching files.
            $instance.Configuration = ''
            Set-CimInstance -InputObject $instance -ErrorAction Stop | Out-Null
        }
    }

    $guardianData = Join-Path $env:ProgramData 'KidOS\Guardian'
    Remove-Item -LiteralPath (Join-Path $guardianData 'recovery-required.flag') -Force -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath (Join-Path $guardianData 'lockdown-profile.json') -Force -ErrorAction SilentlyContinue

    Write-Result 'restored'
    exit 0
} catch {
    Write-Result ('failed:' + $_.Exception.Message)
    Write-Error $_
    exit 1
}
