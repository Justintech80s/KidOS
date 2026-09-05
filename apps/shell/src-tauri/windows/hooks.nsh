!define KIDOS_HOOK_DIR "${__FILEDIR__}"

!macro NSIS_HOOK_POSTINSTALL
  DetailPrint "Installing KidOS Guardian protection service..."

  SetOutPath "$PROGRAMFILES64\KidOS\Guardian"
  File /oname=kidos-guardian-host.exe "${KIDOS_HOOK_DIR}\..\..\..\..\target\release\kidos-guardian-host.exe"

  ; Replace an older Guardian service during upgrades.
  nsExec::ExecToLog '"$SYSDIR\sc.exe" stop KidOSGuardian'
  nsExec::ExecToLog '"$SYSDIR\sc.exe" delete KidOSGuardian'
  Sleep 1000

  ; Protect the service directory so a standard child account cannot replace the binary.
  nsExec::ExecToStack '"$SYSDIR\icacls.exe" "$PROGRAMFILES64\KidOS\Guardian" /inheritance:r /grant:r "SYSTEM:(OI)(CI)(F)" "Administrators:(OI)(CI)(F)" "Users:(OI)(CI)(RX)"'
  Pop $0
  Pop $1
  ${If} $0 != 0
    MessageBox MB_ICONSTOP|MB_OK "KidOS could not secure the Guardian service directory. Installation will stop."
    Abort
  ${EndIf}

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

  nsExec::ExecToStack '"$SYSDIR\sc.exe" start KidOSGuardian'
  Pop $0
  Pop $1
  ${If} $0 != 0
    MessageBox MB_ICONSTOP|MB_OK "KidOS Guardian could not start. Installation will stop so KidOS is not left without protection."
    Abort
  ${EndIf}

  Sleep 1500
  nsExec::ExecToStack '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command "if ((Get-Service -Name KidOSGuardian -ErrorAction Stop).Status -ne [System.ServiceProcess.ServiceControllerStatus]::Running) { exit 20 }"'
  Pop $0
  Pop $1
  ${If} $0 != 0
    MessageBox MB_ICONSTOP|MB_OK "KidOS Guardian did not pass its startup health check. Installation will stop."
    Abort
  ${EndIf}

  DetailPrint "KidOS Guardian is installed and running."
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  DetailPrint "Stopping KidOS Guardian protection service..."
  nsExec::ExecToLog '"$SYSDIR\sc.exe" stop KidOSGuardian'
  nsExec::ExecToLog '"$SYSDIR\sc.exe" delete KidOSGuardian'
  Sleep 1000
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  RMDir /r "$PROGRAMFILES64\KidOS\Guardian"
  RMDir "$PROGRAMFILES64\KidOS"
!macroend
