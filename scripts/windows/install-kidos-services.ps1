param(
  [Parameter(Mandatory=$true)][string]$GuardianExe,
  [Parameter(Mandatory=$true)][string]$ClassifierExe
)
$ErrorActionPreference='Stop'
function Remove-ServiceIfPresent([string]$Name) {
  $svc=Get-Service -Name $Name -ErrorAction SilentlyContinue
  if($null -ne $svc){
    if($svc.Status -ne 'Stopped'){ Stop-Service -Name $Name -Force -ErrorAction SilentlyContinue }
    & "$env:SystemRoot\System32\sc.exe" delete $Name | Out-Null
    Start-Sleep -Milliseconds 750
  }
}
try {
  if(-not (Test-Path -LiteralPath $GuardianExe)){ throw "Guardian executable is missing: $GuardianExe" }
  if(-not (Test-Path -LiteralPath $ClassifierExe)){ throw "Media classifier executable is missing: $ClassifierExe" }

  Remove-ServiceIfPresent 'KidOSMediaClassifier'
  Remove-ServiceIfPresent 'KidOSGuardian'

  New-Service -Name 'KidOSMediaClassifier' -BinaryPathName ('"' + $ClassifierExe + '"') -DisplayName 'KidOS Media Classifier' -StartupType Automatic | Out-Null
  New-Service -Name 'KidOSGuardian' -BinaryPathName ('"' + $GuardianExe + '"') -DisplayName 'KidOS Guardian' -StartupType Automatic | Out-Null

  & "$env:SystemRoot\System32\sc.exe" description KidOSMediaClassifier "Local KidOS image and video safety classification service." | Out-Null
  & "$env:SystemRoot\System32\sc.exe" failure KidOSMediaClassifier reset= 86400 actions= restart/5000/restart/5000/restart/5000 | Out-Null
  & "$env:SystemRoot\System32\sc.exe" failureflag KidOSMediaClassifier 1 | Out-Null
  & "$env:SystemRoot\System32\sc.exe" description KidOSGuardian "Privileged KidOS Guardian service for Windows safety and Assigned Access enforcement." | Out-Null
  & "$env:SystemRoot\System32\sc.exe" failure KidOSGuardian reset= 86400 actions= restart/5000/restart/5000/restart/5000 | Out-Null
  & "$env:SystemRoot\System32\sc.exe" failureflag KidOSGuardian 1 | Out-Null

  Start-Service -Name KidOSMediaClassifier
  Start-Service -Name KidOSGuardian
  (Get-Service KidOSMediaClassifier).WaitForStatus('Running',[TimeSpan]::FromSeconds(20))
  (Get-Service KidOSGuardian).WaitForStatus('Running',[TimeSpan]::FromSeconds(20))
  exit 0
} catch {
  Write-Error $_
  Remove-ServiceIfPresent 'KidOSMediaClassifier'
  Remove-ServiceIfPresent 'KidOSGuardian'
  exit 1
}
