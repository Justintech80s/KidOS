$ErrorActionPreference = 'SilentlyContinue'

function Remove-ServiceIfPresent([string]$Name) {
    $svc = Get-Service -Name $Name -ErrorAction SilentlyContinue
    if ($null -ne $svc) {
        if ($svc.Status -ne 'Stopped') {
            Stop-Service -Name $Name -Force -ErrorAction SilentlyContinue
            Start-Sleep -Milliseconds 300
        }
        & "$env:SystemRoot\System32\sc.exe" delete $Name | Out-Null
    }
}

Remove-ServiceIfPresent 'KidOSMediaClassifier'
Remove-ServiceIfPresent 'KidOSGuardian'

& "$env:SystemRoot\System32\schtasks.exe" /Delete /TN 'KidOS Guardian Recovery' /F | Out-Null
& "$env:SystemRoot\System32\schtasks.exe" /Delete /TN 'KidOS Restore Windows Account' /F | Out-Null

# Best-effort removal of KidOS Assigned Access state. This is intentionally scoped
# to the MDM AssignedAccess singleton and never touches user-profile files.
try {
    $namespace = 'root\cimv2\mdm\dmmap'
    $instances = @(Get-CimInstance -Namespace $namespace -ClassName MDM_AssignedAccess -ErrorAction SilentlyContinue)
    foreach ($instance in $instances) {
        try {
            Remove-CimInstance -InputObject $instance -ErrorAction Stop
        } catch {
            try {
                $instance.Configuration = ''
                Set-CimInstance -InputObject $instance -ErrorAction Stop | Out-Null
            } catch {}
        }
    }
} catch {}

Remove-Item -LiteralPath (Join-Path $env:ProgramData 'KidOS\Guardian\recovery-required.flag') -Force -ErrorAction SilentlyContinue
Remove-Item -LiteralPath (Join-Path $env:ProgramData 'KidOS\Guardian\lockdown-profile.json') -Force -ErrorAction SilentlyContinue

exit 0
