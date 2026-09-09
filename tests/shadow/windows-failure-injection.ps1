$ErrorActionPreference = "Stop"

if ($env:KIDOS_SHADOW_TEST_MODE -ne "1" -or $env:CI -ne "true") {
    throw "Shadow failure injection is restricted to explicit CI/test environments."
}

function Assert-Running([string]$Name) {
    $service = Get-Service -Name $Name -ErrorAction Stop
    if ($service.Status -ne "Running") {
        throw "$Name is not running."
    }
}

Write-Host "SHADOW: Guardian failure injection"
Stop-Service KidOSGuardian -Force
Start-Sleep -Seconds 2
& "$env:ProgramFiles\KidOS\Recovery\kidos-recovery.ps1"
Start-Sleep -Seconds 3
Assert-Running "KidOSGuardian"

Write-Host "SHADOW: Classifier failure injection"
Stop-Service KidOSMediaClassifier -Force
Start-Sleep -Seconds 2
& "$env:ProgramFiles\KidOS\Recovery\kidos-recovery.ps1"
Start-Sleep -Seconds 3
Assert-Running "KidOSMediaClassifier"

Write-Host "Shadow recovery tests PASSED."
