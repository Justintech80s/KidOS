!define KIDOS_HOOK_DIR "${__FILEDIR__}"

!macro KIDOS_ABORT_WITH_ROLLBACK MESSAGE
  nsExec::ExecToLog '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$PROGRAMFILES64\KidOS\Guardian\rollback-partial-install.ps1"'
  MessageBox MB_ICONSTOP|MB_OK "${MESSAGE}"
  Abort
!macroend

!macro NSIS_HOOK_POSTINSTALL
  DetailPrint "Installing KidOS Guardian and local media classifier..."

  SetOutPath "$PROGRAMFILES64\KidOS\Guardian"
  File /oname=kidos-guardian-host.exe "${KIDOS_HOOK_DIR}\..\..\..\..\target\release\kidos-guardian-host.exe"
  File /oname=provision-guardian-credentials.ps1 "${KIDOS_HOOK_DIR}\..\..\..\..\scripts\windows\provision-guardian-credentials.ps1"
  File /oname=install-kidos-services.ps1 "${KIDOS_HOOK_DIR}\..\..\..\..\scripts\windows\install-kidos-services.ps1"
  File /oname=stop-kidos-services.ps1 "${KIDOS_HOOK_DIR}\..\..\..\..\scripts\windows\stop-kidos-services.ps1"
  File /oname=verify-kidos-services.ps1 "${KIDOS_HOOK_DIR}\..\..\..\..\scripts\windows\verify-kidos-services.ps1"
  File /oname=hash-recovery-installer.ps1 "${KIDOS_HOOK_DIR}\..\..\..\..\scripts\windows\hash-recovery-installer.ps1"
  File /oname=verify-restore-result.ps1 "${KIDOS_HOOK_DIR}\..\..\..\..\scripts\windows\verify-restore-result.ps1"
  File /oname=rollback-partial-install.ps1 "${KIDOS_HOOK_DIR}\..\..\..\..\scripts\windows\rollback-partial-install.ps1"

  ; Stage recovery before any fallible setup step so partial installs remain recoverable.
  SetOutPath "$PROGRAMFILES64\KidOS\Recovery"
  File /oname=kidos-recovery.ps1 "${KIDOS_HOOK_DIR}\..\..\..\..\scripts\windows\kidos-recovery.ps1"
  File /oname=restore-windows-account.ps1 "${KIDOS_HOOK_DIR}\..\..\..\..\scripts\windows\restore-windows-account.ps1"
  File /oname=rollback-kidos.ps1 "${KIDOS_HOOK_DIR}\..\..\..\..\scripts\windows\rollback-kidos.ps1"

  SetOutPath "$PROGRAMFILES64\KidOS\MediaClassifier"
  File /oname=kidos-media-classifier.exe "${KIDOS_HOOK_DIR}\..\..\..\..\services\media-classifier\dist\kidos-media-classifier.exe"
  SetOutPath "$PROGRAMFILES64\KidOS\MediaClassifier\model"
  File /r "${KIDOS_HOOK_DIR}\..\..\..\..\services\media-classifier\dist\model\*.*"

  ; Replace older KidOS services during upgrades. On a first install there is
  ; nothing to remove, so do this quietly instead of showing harmless SC 1060 errors.
  nsExec::ExecToLog '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$PROGRAMFILES64\KidOS\Guardian\stop-kidos-services.ps1"'
  Sleep 1000

  ; Protect service binaries and model files so a standard child account cannot replace them.
  nsExec::ExecToStack '"$SYSDIR\icacls.exe" "$PROGRAMFILES64\KidOS" /inheritance:r /grant:r "SYSTEM:(OI)(CI)(F)" "Administrators:(OI)(CI)(F)" "Users:(OI)(CI)(RX)"'
  Pop $0
  Pop $1
  ${If} $0 != 0
    !insertmacro KIDOS_ABORT_WITH_ROLLBACK "KidOS could not secure the Guardian service directory. Installation was rolled back."
  ${EndIf}

  ; Provision protected Guardian credentials from a standalone PowerShell 5.1 script.
  nsExec::ExecToStack '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$PROGRAMFILES64\KidOS\Guardian\provision-guardian-credentials.ps1" -InstallerPath "$EXEPATH"'
  Pop $0
  Pop $1
  ${If} $0 != 0
    !insertmacro KIDOS_ABORT_WITH_ROLLBACK "KidOS could not provision its protected Guardian credentials. Installation was rolled back."
  ${EndIf}

  ; Register/start services through a standalone PowerShell 5.1 script.
  nsExec::ExecToStack '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$PROGRAMFILES64\KidOS\Guardian\install-kidos-services.ps1" -GuardianExe "$PROGRAMFILES64\KidOS\Guardian\kidos-guardian-host.exe" -ClassifierExe "$PROGRAMFILES64\KidOS\MediaClassifier\kidos-media-classifier.exe"'
  Pop $0
  Pop $1
  ${If} $0 != 0
    !insertmacro KIDOS_ABORT_WITH_ROLLBACK "KidOS could not register/start its Windows protection services. Installation was rolled back."
  ${EndIf}

  Sleep 2500
  nsExec::ExecToStack '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$PROGRAMFILES64\KidOS\Guardian\verify-kidos-services.ps1"'
  Pop $0
  Pop $1
  ${If} $0 != 0
    !insertmacro KIDOS_ABORT_WITH_ROLLBACK "KidOS Guardian did not pass its startup health check. Installation was rolled back."
  ${EndIf}

  nsExec::ExecToLog '"$SYSDIR\schtasks.exe" /Delete /TN "KidOS Guardian Recovery" /F'
  nsExec::ExecToStack '"$SYSDIR\schtasks.exe" /Create /TN "KidOS Guardian Recovery" /SC ONSTART /RU SYSTEM /RL HIGHEST /TR "$\"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe$\" -NoProfile -NonInteractive -ExecutionPolicy Bypass -File $\"$PROGRAMFILES64\KidOS\Recovery\kidos-recovery.ps1$\"" /F'
  Pop $0
  Pop $1
  ${If} $0 != 0
    !insertmacro KIDOS_ABORT_WITH_ROLLBACK "KidOS could not install its recovery health check. Installation was rolled back."
  ${EndIf}

  ; Cache the successfully installed package in a SYSTEM/administrator-only recovery area.
  ; On an upgrade, move the last known-good package to Previous before recording the new one.
  CreateDirectory "$PROGRAMDATA\KidOS\Recovery\Current"
  CreateDirectory "$PROGRAMDATA\KidOS\Recovery\Previous"
  IfFileExists "$PROGRAMDATA\KidOS\Recovery\Current\KidOS-current.exe" 0 +4
    CopyFiles /SILENT "$PROGRAMDATA\KidOS\Recovery\Current\KidOS-current.exe" "$PROGRAMDATA\KidOS\Recovery\Previous\KidOS-previous.exe"
    nsExec::ExecToLog '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$PROGRAMFILES64\KidOS\Guardian\hash-recovery-installer.ps1" -InstallerPath "$PROGRAMDATA\KidOS\Recovery\Previous\KidOS-previous.exe" -HashPath "$PROGRAMDATA\KidOS\Recovery\Previous\KidOS-previous.sha256"'
  CopyFiles /SILENT "$EXEPATH" "$PROGRAMDATA\KidOS\Recovery\Current\KidOS-current.exe"
  nsExec::ExecToLog '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$PROGRAMFILES64\KidOS\Guardian\hash-recovery-installer.ps1" -InstallerPath "$PROGRAMDATA\KidOS\Recovery\Current\KidOS-current.exe" -HashPath "$PROGRAMDATA\KidOS\Recovery\Current\KidOS-current.sha256"'
  nsExec::ExecToLog '"$SYSDIR\icacls.exe" "$PROGRAMDATA\KidOS\Recovery" /inheritance:r /grant:r "SYSTEM:(OI)(CI)(F)" "Administrators:(OI)(CI)(F)"'

  DetailPrint "KidOS Guardian and local media classifier are installed and running."
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  DetailPrint "Restoring the Windows child account before removing KidOS..."

  ; The uninstaller carries its own recovery payload so partial installations can
  ; always be removed even if Program Files\KidOS\Recovery was never completed.
  SetOutPath "$PLUGINSDIR\KidOSRecovery"
  File /oname=restore-windows-account.ps1 "${KIDOS_HOOK_DIR}\..\..\..\..\scripts\windows\restore-windows-account.ps1"
  File /oname=verify-restore-result.ps1 "${KIDOS_HOOK_DIR}\..\..\..\..\scripts\windows\verify-restore-result.ps1"

  Delete "$PROGRAMDATA\KidOS\Recovery\restore-windows.result"
  nsExec::ExecToLog '"$SYSDIR\schtasks.exe" /Delete /TN "KidOS Restore Windows Account" /F'
  nsExec::ExecToStack '"$SYSDIR\schtasks.exe" /Create /TN "KidOS Restore Windows Account" /SC ONSTART /RU SYSTEM /RL HIGHEST /TR "$\"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe$\" -NoProfile -NonInteractive -ExecutionPolicy Bypass -File $\"$PLUGINSDIR\KidOSRecovery\restore-windows-account.ps1$\"" /F'
  Pop $0
  Pop $1
  ${If} $0 != 0
    MessageBox MB_ICONSTOP|MB_OK "KidOS could not create its Windows recovery task. Uninstall will stop to avoid leaving the child account locked."
    Abort
  ${EndIf}
  nsExec::ExecToStack '"$SYSDIR\schtasks.exe" /Run /TN "KidOS Restore Windows Account"'
  Pop $0
  Pop $1
  ${If} $0 != 0
    MessageBox MB_ICONSTOP|MB_OK "KidOS could not start Windows account recovery. Uninstall will stop."
    Abort
  ${EndIf}

  ; Give the SYSTEM task time to remove Assigned Access and write its result.
  Sleep 5000
  nsExec::ExecToStack '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$PLUGINSDIR\KidOSRecovery\verify-restore-result.ps1"'
  Pop $0
  Pop $1
  ${If} $0 != 0
    MessageBox MB_ICONSTOP|MB_OK "KidOS could not verify that Windows lockdown was removed. Uninstall will stop."
    Abort
  ${EndIf}
  nsExec::ExecToLog '"$SYSDIR\schtasks.exe" /Delete /TN "KidOS Restore Windows Account" /F'

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
