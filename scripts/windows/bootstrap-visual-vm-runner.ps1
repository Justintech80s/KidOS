[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('Host', 'Guest')]
    [string]$Role,

    [Parameter(Mandatory = $true)]
    [string]$RegistrationToken,

    [string]$RepositoryUrl = 'https://github.com/Justintech80s/KidOS',
    [string]$VMName = 'KidOS-Visual-Test',
    [string]$SnapshotName = 'KidOS-Clean',
    [string]$RunnerRoot = '',
    [string]$RunnerName = ''
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

function Assert-WindowsAdministrator {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($identity)
    $isAdministrator = $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    if (-not $isAdministrator) {
        throw 'Run this bootstrap from an elevated Administrator PowerShell session.'
    }
}

function Resolve-RunnerAsset {
    $headers = @{
        'Accept' = 'application/vnd.github+json'
        'User-Agent' = 'KidOS-Visual-VM-Runner-Bootstrap'
        'X-GitHub-Api-Version' = '2022-11-28'
    }

    $release = Invoke-RestMethod `
        -Uri 'https://api.github.com/repos/actions/runner/releases/latest' `
        -Headers $headers `
        -Method Get

    $asset = $release.assets |
        Where-Object { $_.name -like 'actions-runner-win-x64-*.zip' } |
        Select-Object -First 1

    if (-not $asset) {
        throw 'The latest official GitHub Actions runner release does not contain a Windows x64 ZIP asset.'
    }

    return $asset
}

function Install-RunnerFiles {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Destination
    )

    New-Item -ItemType Directory -Path $Destination -Force | Out-Null

    $configPath = Join-Path $Destination 'config.cmd'
    if (Test-Path -LiteralPath $configPath) {
        return
    }

    $asset = Resolve-RunnerAsset
    $archive = Join-Path $env:TEMP $asset.name

    try {
        Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $archive -UseBasicParsing
        Expand-Archive -LiteralPath $archive -DestinationPath $Destination -Force
    }
    finally {
        Remove-Item -LiteralPath $archive -Force -ErrorAction SilentlyContinue
    }

    if (-not (Test-Path -LiteralPath $configPath)) {
        throw "GitHub Actions runner extraction did not create $configPath."
    }
}

function Assert-HostPrerequisites {
    Import-Module Hyper-V -ErrorAction Stop

    $vm = Get-VM -Name $VMName -ErrorAction Stop
    if (-not $vm) {
        throw "Hyper-V VM '$VMName' was not found."
    }

    $snapshot = Get-VMSnapshot -VMName $VMName -Name $SnapshotName -ErrorAction Stop
    if (-not $snapshot) {
        throw "Hyper-V snapshot '$SnapshotName' was not found for VM '$VMName'."
    }

    Write-Host "Verified Hyper-V VM '$VMName' and snapshot '$SnapshotName'."
}

function Assert-GuestPrerequisites {
    $sessionId = [System.Diagnostics.Process]::GetCurrentProcess().SessionId
    if ($sessionId -eq 0) {
        throw 'Guest runner bootstrap cannot run in Windows Session 0. Sign in to the KidOS test VM desktop and run this command from an interactive Administrator PowerShell window.'
    }

    $guardian = Get-Service -Name 'KidOSGuardian' -ErrorAction Stop
    Write-Host "Verified KidOSGuardian service exists with status '$($guardian.Status)'."
    Write-Host "Verified interactive Windows session $sessionId."
}

Assert-WindowsAdministrator

if ([string]::IsNullOrWhiteSpace($RegistrationToken)) {
    throw 'RegistrationToken is required. Use a temporary GitHub self-hosted runner registration token.'
}

if ($Role -eq 'Host') {
    Assert-HostPrerequisites
    $runnerLabel = 'kidos-interactive-test'
    if ([string]::IsNullOrWhiteSpace($RunnerRoot)) {
        $RunnerRoot = 'C:\actions-runner-kidos-host'
    }
    if ([string]::IsNullOrWhiteSpace($RunnerName)) {
        $RunnerName = "$env:COMPUTERNAME-kidos-hyperv-host"
    }
}
else {
    Assert-GuestPrerequisites
    $runnerLabel = 'kidos-visual-vm-guest'
    if ([string]::IsNullOrWhiteSpace($RunnerRoot)) {
        $RunnerRoot = 'C:\actions-runner-kidos-visual-guest'
    }
    if ([string]::IsNullOrWhiteSpace($RunnerName)) {
        $RunnerName = "$env:COMPUTERNAME-kidos-visual-guest"
    }
}

if (Test-Path -LiteralPath (Join-Path $RunnerRoot '.runner')) {
    throw "A GitHub Actions runner is already configured at '$RunnerRoot'. Use a clean runner directory or remove the existing runner registration first."
}

Install-RunnerFiles -Destination $RunnerRoot

Push-Location $RunnerRoot
try {
    $configArguments = @(
        '--unattended',
        '--url', $RepositoryUrl,
        '--token', $RegistrationToken,
        '--name', $RunnerName,
        '--labels', $runnerLabel,
        '--work', '_work',
        '--replace'
    )

    if ($Role -eq 'Host') {
        $configArguments += '--runasservice'
    }

    & .\config.cmd @configArguments
    if ($LASTEXITCODE -ne 0) {
        throw "GitHub Actions runner configuration failed with exit code $LASTEXITCODE."
    }

    if ($Role -eq 'Host') {
        Write-Host "Host runner '$RunnerName' is registered with label 'kidos-interactive-test' as a Windows service."
        Write-Host "The queued prepare-hyperv-vm job can start once the runner service is online."
    }

    if ($Role -eq 'Guest') {
        Write-Host "Guest runner '$RunnerName' is registered with label 'kidos-visual-vm-guest'."
        Write-Host 'Starting the GitHub Actions runner in this interactive desktop session. Keep this window open while the visual VM validation runs.'
        & .\run.cmd
        if ($LASTEXITCODE -ne 0) {
            throw "Interactive GitHub Actions runner exited with code $LASTEXITCODE."
        }
    }
}
finally {
    Pop-Location
}
