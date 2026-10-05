param(
  [Parameter(Mandatory = $true)]
  [string]$VMName,

  [Parameter(Mandatory = $true)]
  [ValidateSet("prepare", "start", "reset", "stop")]
  [string]$Operation,

  [string]$SnapshotName = "KidOS-Clean"
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

function Get-KidOSVm {
  $vm = Get-VM -Name $VMName -ErrorAction SilentlyContinue
  if ($null -eq $vm) {
    throw "Hyper-V VM not found: $VMName"
  }
  return $vm
}

function Get-RequiredSnapshot {
  $snapshot = Get-VMSnapshot -VMName $VMName -Name $SnapshotName -ErrorAction SilentlyContinue
  if ($null -eq $snapshot) {
    throw "Required Hyper-V snapshot not found: $VMName/$SnapshotName"
  }
  return $snapshot
}

$null = Get-Command Get-VM -ErrorAction Stop
$vm = Get-KidOSVm
$restoredSnapshot = $null

switch ($Operation) {
  "prepare" {
    # Preparation is intentionally non-destructive: prove Hyper-V and the named VM
    # are available before later test stages attempt lifecycle mutations.
    $vm = Get-KidOSVm
  }

  "start" {
    if ($vm.State -ne "Running") {
      Start-VM -Name $VMName -ErrorAction Stop | Out-Null
    }
    $vm = Get-KidOSVm
  }

  "reset" {
    $snapshot = Get-RequiredSnapshot
    if ($vm.State -ne "Off") {
      Stop-VM -Name $VMName -TurnOff -Force -ErrorAction Stop | Out-Null
    }
    Restore-VMSnapshot -VMName $VMName -Name $SnapshotName -Confirm:$false -ErrorAction Stop | Out-Null
    Start-VM -Name $VMName -ErrorAction Stop | Out-Null
    $restoredSnapshot = $SnapshotName
    $vm = Get-KidOSVm
  }

  "stop" {
    if ($vm.State -ne "Off") {
      Stop-VM -Name $VMName -TurnOff -Force -ErrorAction Stop | Out-Null
    }
    $vm = Get-KidOSVm
  }
}

$result = [ordered]@{
  passed = $true
  vmName = $VMName
  operation = $Operation
  state = [string]$vm.State
  snapshot = $restoredSnapshot
  checkedAt = (Get-Date).ToString("o")
}

$result | ConvertTo-Json -Depth 4
