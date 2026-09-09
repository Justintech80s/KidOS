$ErrorActionPreference = "Stop"

if ($env:KIDOS_SHADOW_TEST_MODE -ne "1" -or $env:CI -ne "true") {
    throw "Shadow service startup benchmark is restricted to explicit CI/test environments."
}

$services = @(
    "KidOSGuardian",
    "KidOSMediaClassifier"
)

foreach ($name in $services) {
    Stop-Service $name -Force -ErrorAction SilentlyContinue
    Start-Sleep -Milliseconds 500

    $watch = [System.Diagnostics.Stopwatch]::StartNew()
    Start-Service $name
    (Get-Service $name).WaitForStatus(
        "Running",
        [TimeSpan]::FromSeconds(60)
    )
    $watch.Stop()

    Write-Host ("SHADOW STARTUP: {0} = {1} ms" -f $name, $watch.ElapsedMilliseconds)

    if ($watch.ElapsedMilliseconds -gt 30000) {
        throw "$name startup exceeded Shadow threshold."
    }
}
