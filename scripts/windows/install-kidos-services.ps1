param(
  [Parameter(Mandatory=$true)][string]$GuardianExe,
  [Parameter(Mandatory=$true)][string]$ClassifierExe
)

$ErrorActionPreference='Stop'
$diagDir = Join-Path $env:ProgramData 'KidOS\Diagnostics'
$diagLog = Join-Path $diagDir 'service-install.log'
New-Item -ItemType Directory -Force -Path $diagDir | Out-Null

function Write-Diag([string]$Message) {
  $line = ('{0:u} {1}' -f (Get-Date), $Message)
  Add-Content -LiteralPath $diagLog -Value $line -Encoding UTF8
  Write-Host $line
}

function Capture-ServiceState([string]$Name) {
  try {
    $svc = Get-CimInstance Win32_Service -Filter ("Name='{0}'" -f $Name) -ErrorAction Stop
    Write-Diag ("Service {0}: State={1}; StartMode={2}; StartName={3}; PathName={4}; ExitCode={5}; ServiceSpecificExitCode={6}" -f
      $Name,$svc.State,$svc.StartMode,$svc.StartName,$svc.PathName,$svc.ExitCode,$svc.ServiceSpecificExitCode)
  } catch {
    Write-Diag ("Service {0}: not present or unreadable: {1}" -f $Name,$_.Exception.Message)
  }
}

function Capture-ServiceEvents([string]$Name) {
  try {
    $since = (Get-Date).AddMinutes(-10)
    $events = Get-WinEvent -FilterHashtable @{LogName='System'; ProviderName='Service Control Manager'; StartTime=$since} -ErrorAction SilentlyContinue |
      Where-Object { $_.Message -match [regex]::Escape($Name) } |
      Select-Object -First 20 TimeCreated,Id,LevelDisplayName,Message
    foreach ($event in $events) {
      $message = ($event.Message -replace '\r?\n',' ')
      Write-Diag ("SCM event for {0}: Id={1}; Level={2}; Message={3}" -f $Name,$event.Id,$event.LevelDisplayName,$message)
    }
  } catch {
    Write-Diag ("Could not capture SCM events for {0}: {1}" -f $Name,$_.Exception.Message)
  }
}

function Remove-ServiceIfPresent([string]$Name) {
  $svc=Get-Service -Name $Name -ErrorAction SilentlyContinue
  if($null -ne $svc){
    Write-Diag ("Removing existing service {0} (status {1})." -f $Name,$svc.Status)
    if($svc.Status -ne 'Stopped'){ Stop-Service -Name $Name -Force -ErrorAction SilentlyContinue }
    & "$env:SystemRoot\System32\sc.exe" delete $Name | Out-Null
    Start-Sleep -Milliseconds 750
  }
}

function Start-KidOSService([string]$Name) {
  Write-Diag ("Starting service {0}..." -f $Name)
  try {
    Start-Service -Name $Name -ErrorAction Stop
    (Get-Service $Name).WaitForStatus('Running',[TimeSpan]::FromSeconds(30))
    Capture-ServiceState $Name
    Write-Diag ("Service {0} reached Running." -f $Name)
  } catch {
    Write-Diag ("Service {0} failed to start: {1}" -f $Name,$_.Exception.Message)
    Capture-ServiceState $Name
    Capture-ServiceEvents $Name
    throw
  }
}

try {
  Write-Diag ("KidOS service installation started. GuardianExe={0}; ClassifierExe={1}" -f $GuardianExe,$ClassifierExe)
  if(-not (Test-Path -LiteralPath $GuardianExe)){ throw "Guardian executable is missing: $GuardianExe" }
  if(-not (Test-Path -LiteralPath $ClassifierExe)){ throw "Media classifier executable is missing: $ClassifierExe" }

  Remove-ServiceIfPresent 'KidOSMediaClassifier'
  Remove-ServiceIfPresent 'KidOSGuardian'

  Write-Diag 'Creating KidOS Windows services.'
  New-Service -Name 'KidOSMediaClassifier' -BinaryPathName ('"' + $ClassifierExe + '"') -DisplayName 'KidOS Media Classifier' -StartupType Automatic | Out-Null
  New-Service -Name 'KidOSGuardian' -BinaryPathName ('"' + $GuardianExe + '"') -DisplayName 'KidOS Guardian' -StartupType Automatic | Out-Null

  & "$env:SystemRoot\System32\sc.exe" config KidOSMediaClassifier obj= LocalSystem | Out-Null
  if($LASTEXITCODE -ne 0){ throw "Could not configure KidOSMediaClassifier as LocalSystem (exit $LASTEXITCODE)." }
  & "$env:SystemRoot\System32\sc.exe" config KidOSGuardian obj= LocalSystem | Out-Null
  if($LASTEXITCODE -ne 0){ throw "Could not configure KidOSGuardian as LocalSystem (exit $LASTEXITCODE)." }

  & "$env:SystemRoot\System32\sc.exe" description KidOSMediaClassifier "Local KidOS image and video safety classification service." | Out-Null
  & "$env:SystemRoot\System32\sc.exe" failure KidOSMediaClassifier reset= 86400 actions= restart/5000/restart/5000/restart/5000 | Out-Null
  & "$env:SystemRoot\System32\sc.exe" failureflag KidOSMediaClassifier 1 | Out-Null
  & "$env:SystemRoot\System32\sc.exe" description KidOSGuardian "Privileged KidOS Guardian service for Windows safety and Assigned Access enforcement." | Out-Null
  & "$env:SystemRoot\System32\sc.exe" failure KidOSGuardian reset= 86400 actions= restart/5000/restart/5000/restart/5000 | Out-Null
  & "$env:SystemRoot\System32\sc.exe" failureflag KidOSGuardian 1 | Out-Null

  Capture-ServiceState 'KidOSMediaClassifier'
  Capture-ServiceState 'KidOSGuardian'

  Start-KidOSService 'KidOSMediaClassifier'
  Start-KidOSService 'KidOSGuardian'

  Write-Diag 'KidOS service installation completed successfully.'
  exit 0
} catch {
  Write-Diag ("KidOS service installation failed: {0}" -f $_.Exception.Message)
  Capture-ServiceState 'KidOSMediaClassifier'
  Capture-ServiceState 'KidOSGuardian'
  Capture-ServiceEvents 'KidOSMediaClassifier'
  Capture-ServiceEvents 'KidOSGuardian'
  Write-Error $_
  Remove-ServiceIfPresent 'KidOSMediaClassifier'
  Remove-ServiceIfPresent 'KidOSGuardian'
  exit 1
}
