!define KIDOS_HOOK_DIR "${__FILEDIR__}"

!macro NSIS_HOOK_POSTINSTALL
  DetailPrint "Installing KidOS Guardian and local media classifier..."

  SetOutPath "$PROGRAMFILES64\KidOS\Guardian"
  File /oname=kidos-guardian-host.exe "${KIDOS_HOOK_DIR}\..\..\..\..\target\release\kidos-guardian-host.exe"

  SetOutPath "$PROGRAMFILES64\KidOS\MediaClassifier"
  File /oname=kidos-media-classifier.exe "${KIDOS_HOOK_DIR}\..\..\..\..\services\media-classifier\dist\kidos-media-classifier.exe"
  SetOutPath "$PROGRAMFILES64\KidOS\MediaClassifier\model"
  File /r "${KIDOS_HOOK_DIR}\..\..\..\..\services\media-classifier\dist\model\*.*"

  ; Replace older KidOS services during upgrades.
  nsExec::ExecToLog '"$SYSDIR\sc.exe" stop KidOSMediaClassifier'
  nsExec::ExecToLog '"$SYSDIR\sc.exe" delete KidOSMediaClassifier'
  nsExec::ExecToLog '"$SYSDIR\sc.exe" stop KidOSGuardian'
  nsExec::ExecToLog '"$SYSDIR\sc.exe" delete KidOSGuardian'
  Sleep 1000

  ; Protect service binaries and model files so a standard child account cannot replace them.
  nsExec::ExecToStack '"$SYSDIR\icacls.exe" "$PROGRAMFILES64\KidOS" /inheritance:r /grant:r "SYSTEM:(OI)(CI)(F)" "Administrators:(OI)(CI)(F)" "Users:(OI)(CI)(RX)"'
  Pop $0
  Pop $1
  ${If} $0 != 0
    MessageBox MB_ICONSTOP|MB_OK "KidOS could not secure the Guardian service directory. Installation will stop."
    Abort
  ${EndIf}

  ; Generate a private classifier token, pin the trusted installer publisher when signed,
  ; and protect Guardian state under ProgramData.
  nsExec::ExecToStack '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command "$d=$env:ProgramData+''\KidOS\Guardian''; New-Item -ItemType Directory -Force -Path $d | Out-Null; $b=New-Object byte[] 32; [System.Security.Cryptography.RandomNumberGenerator]::Create().GetBytes($b); $t=([Convert]::ToHexString($b)).ToLowerInvariant(); Set-Content -Path ($d+''\media-classifier.token'') -Value $t -NoNewline -Encoding ASCII; $s=Get-AuthenticodeSignature -LiteralPath ''$EXEPATH''; if($s.Status -eq ''Valid'' -and $s.SignerCertificate){ Set-Content -Path ($d+''\publisher-thumbprint.txt'') -Value $s.SignerCertificate.Thumbprint -NoNewline -Encoding ASCII }; icacls $d /inheritance:r /grant:r ''SYSTEM:(OI)(CI)(F)'' ''Administrators:(OI)(CI)(F)'' | Out-Null"'
  Pop $0
  Pop $1
  ${If} $0 != 0
    MessageBox MB_ICONSTOP|MB_OK "KidOS could not provision its protected Guardian credentials. Installation will stop."
    Abort
  ${EndIf}

  nsExec::ExecToStack '"$SYSDIR\sc.exe" create KidOSMediaClassifier binPath= "$\"$PROGRAMFILES64\KidOS\MediaClassifier\kidos-media-classifier.exe$\"" start= auto obj= LocalSystem DisplayName= "KidOS Media Classifier"'
  Pop $0
  Pop $1
  ${If} $0 != 0
    MessageBox MB_ICONSTOP|MB_OK "KidOS could not register the local media-classifier service. Installation will stop."
    Abort
  ${EndIf}
  nsExec::ExecToLog '"$SYSDIR\sc.exe" description KidOSMediaClassifier "Local KidOS image and video safety classification service."'
  nsExec::ExecToLog '"$SYSDIR\sc.exe" failure KidOSMediaClassifier reset= 86400 actions= restart/5000/restart/5000/restart/5000'
  nsExec::ExecToLog '"$SYSDIR\sc.exe" failureflag KidOSMediaClassifier 1'

  nsExec::ExecToStack '"$SYSDIR\sc.exe" create KidOSGuardian binPath= "$\"$PROGRAMFILES64\KidOS\Guardian\kidos-guardian-host.exe$\"" start= auto obj= LocalSystem DisplayName= "KidOS Guardian"'
  Pop $0
  Pop $1
  ${If} $0 != 0
    MessageBox MB_ICONSTOP|MB_OK "KidOS could not register the Guardian Windows service. Installation will stop."
    Abort
  ${EndIf}

  nsExec::ExecToLog '"$SYSDIR\sc.exe" description KidOSGuardian "Privileged KidOS Guardian service for Windows safety and Assigned Access enforcement."'
  nsExec::ExecToLog '"$SYSDIR\sc.exe" failure KidOSGuardian reset= 86400 actions= restart/5000/restart/5000/restart/5000'
  nsExec::ExecToLog '"$SYSDIR\sc.exe" failureflag KidOSGuardian 1'

  nsExec::ExecToStack '"$SYSDIR\sc.exe" start KidOSMediaClassifier'
  Pop $0
  Pop $1
  ${If} $0 != 0
    MessageBox MB_ICONSTOP|MB_OK "KidOS Media Classifier could not start. Installation will stop."
    Abort
  ${EndIf}

  nsExec::ExecToStack '"$SYSDIR\sc.exe" start KidOSGuardian'
  Pop $0
  Pop $1
  ${If} $0 != 0
    MessageBox MB_ICONSTOP|MB_OK "KidOS Guardian could not start. Installation will stop so KidOS is not left without protection."
    Abort
  ${EndIf}

  Sleep 2500
  nsExec::ExecToStack '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command "if ((Get-Service -Name KidOSGuardian -ErrorAction Stop).Status -ne [System.ServiceProcess.ServiceControllerStatus]::Running) { exit 20 }; if ((Get-Service -Name KidOSMediaClassifier -ErrorAction Stop).Status -ne [System.ServiceProcess.ServiceControllerStatus]::Running) { exit 21 }; $token=Get-Content ($env:ProgramData+''\KidOS\Guardian\media-classifier.token'') -Raw; $ok=$false; for($i=0;$i -lt 45;$i++){ try { $r=Invoke-RestMethod -Uri ''http://127.0.0.1:8765/health'' -Headers @{''x-kidos-classifier-token''=$token} -TimeoutSec 3; if($r.status -eq ''healthy''){ $ok=$true; break } } catch {}; Start-Sleep -Seconds 2 }; if(-not $ok){ exit 22 }"'
  Pop $0
  Pop $1
  ${If} $0 != 0
    MessageBox MB_ICONSTOP|MB_OK "KidOS Guardian did not pass its startup health check. Installation will stop."
    Abort
  ${EndIf}

  SetOutPath "$PROGRAMFILES64\KidOS\Recovery"
  File /oname=kidos-recovery.ps1 "${KIDOS_HOOK_DIR}\..\..\..\..\scripts\windows\kidos-recovery.ps1"
  nsExec::ExecToLog '"$SYSDIR\schtasks.exe" /Delete /TN "KidOS Guardian Recovery" /F'
  nsExec::ExecToStack '"$SYSDIR\schtasks.exe" /Create /TN "KidOS Guardian Recovery" /SC ONSTART /RU SYSTEM /RL HIGHEST /TR "$\"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe$\" -NoProfile -NonInteractive -ExecutionPolicy Bypass -File $\"$PROGRAMFILES64\KidOS\Recovery\kidos-recovery.ps1$\"" /F'
  Pop $0
  Pop $1
  ${If} $0 != 0
    MessageBox MB_ICONSTOP|MB_OK "KidOS could not install its recovery health check. Installation will stop."
    Abort
  ${EndIf}

  DetailPrint "KidOS Guardian and local media classifier are installed and running."
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  DetailPrint "Stopping KidOS protection services..."
  nsExec::ExecToLog '"$SYSDIR\schtasks.exe" /Delete /TN "KidOS Guardian Recovery" /F'
  nsExec::ExecToLog '"$SYSDIR\sc.exe" stop KidOSMediaClassifier'
  nsExec::ExecToLog '"$SYSDIR\sc.exe" delete KidOSMediaClassifier'
  nsExec::ExecToLog '"$SYSDIR\sc.exe" stop KidOSGuardian'
  nsExec::ExecToLog '"$SYSDIR\sc.exe" delete KidOSGuardian'
  Sleep 1000
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  RMDir /r "$PROGRAMFILES64\KidOS\Guardian"
  RMDir /r "$PROGRAMFILES64\KidOS\MediaClassifier"
  RMDir /r "$PROGRAMFILES64\KidOS\Recovery"
  RMDir "$PROGRAMFILES64\KidOS"
  RMDir /r "$PROGRAMDATA\KidOS"
!macroend
