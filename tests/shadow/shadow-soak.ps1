param(
    [int]$Minutes = 30,
    [int]$IntervalSeconds = 10
)

$ErrorActionPreference = "Stop"
$deadline = (Get-Date).AddMinutes($Minutes)

while ((Get-Date) -lt $deadline) {
    foreach ($serviceName in @(
        "KidOSGuardian",
        "KidOSMediaClassifier"
    )) {
        $service = Get-Service $serviceName -ErrorAction SilentlyContinue

        if (-not $service) {
            throw "SHADOW CRITICAL: $serviceName disappeared."
        }

        if ($service.Status -ne "Running") {
            throw "SHADOW CRITICAL: $serviceName stopped."
        }
    }

    Start-Sleep -Seconds $IntervalSeconds
}

Write-Host "Shadow soak test PASSED."
