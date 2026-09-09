param(
    [int]$SoakMinutes = 5
)

$ErrorActionPreference = "Stop"

$resultsDir = Join-Path (Get-Location) "target\shadow-results"
New-Item -ItemType Directory -Force -Path $resultsDir | Out-Null

$results = @()

function Invoke-ShadowTest {
    param(
        [Parameter(Mandatory=$true)][string]$Name,
        [Parameter(Mandatory=$true)][scriptblock]$Action,
        [string]$Severity = "Critical"
    )

    $watch = [System.Diagnostics.Stopwatch]::StartNew()
    try {
        & $Action
        $watch.Stop()
        $script:results += [pscustomobject]@{
            name = $Name
            status = "PASS"
            severity = "Info"
            latency_ms = $watch.ElapsedMilliseconds
            reason = "Passed"
        }
    } catch {
        $watch.Stop()
        $script:results += [pscustomobject]@{
            name = $Name
            status = "FAIL"
            severity = $Severity
            latency_ms = $watch.ElapsedMilliseconds
            reason = $_.Exception.Message
        }
    }
}

Invoke-ShadowTest "startup_performance" {
    & "$PSScriptRoot\startup-performance.ps1"
}

Invoke-ShadowTest "tamper_protection" {
    & "$PSScriptRoot\tamper-test.ps1"
}

Invoke-ShadowTest "service_recovery" {
    & "$PSScriptRoot\windows-failure-injection.ps1"
}

Invoke-ShadowTest "service_soak" {
    & "$PSScriptRoot\shadow-soak.ps1" -Minutes $SoakMinutes
}

$total = $results.Count
$passed = @($results | Where-Object status -eq "PASS").Count
$warnings = @($results | Where-Object severity -eq "Warning").Count
$critical = @($results | Where-Object { $_.status -eq "FAIL" -and $_.severity -eq "Critical" }).Count
$score = if ($total -eq 0) { 0.0 } else { [math]::Round(($passed / $total) * 100.0, 2) }
$releaseReady = ($critical -eq 0 -and $score -ge 98.0)

$tauriConfig = Get-Content "apps/shell/src-tauri/tauri.conf.json" -Raw | ConvertFrom-Json
$report = [ordered]@{
    build = $tauriConfig.version
    overall_score = $score
    release_ready = $releaseReady
    critical_failures = $critical
    warnings = $warnings
    tests = $results
}

$report | ConvertTo-Json -Depth 8 | Set-Content (Join-Path $resultsDir "shadow-report.json") -Encoding UTF8

@"
SHADOW REPORT

Overall: $score%
Critical failures: $critical
Warnings: $warnings
Release recommendation: $(if ($releaseReady) { "APPROVED" } else { "BLOCKED" })
"@ | Set-Content (Join-Path $resultsDir "shadow-report.txt") -Encoding UTF8

Get-Content (Join-Path $resultsDir "shadow-report.txt")

if (-not $releaseReady) {
    throw "KidOS Shadow release gate blocked this build."
}

Write-Host "KIDOS SHADOW VALIDATION PASSED"
