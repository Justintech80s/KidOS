!macro customInstall
  DetailPrint "Installing KidOS Guardian, local media classifier, and recovery..."
  nsExec::ExecToStack '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$INSTDIR\resources\backend\scripts\install-electron-backend.ps1" -ResourceRoot "$INSTDIR\resources\backend" -InstallerPath "$EXEPATH"'
  Pop $0
  Pop $1
  ${If} $0 != 0
    MessageBox MB_ICONSTOP|MB_OK "KidOS protection services did not install correctly. Setup will stop instead of leaving an unprotected child environment."
    Abort
  ${EndIf}
!macroend

!macro customUnInstall
  DetailPrint "Restoring Windows and removing KidOS protection services..."
  nsExec::ExecToStack '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$INSTDIR\resources\backend\scripts\uninstall-electron-backend.ps1" -ResourceRoot "$INSTDIR\resources\backend"'
  Pop $0
  Pop $1
  ${If} $0 != 0
    MessageBox MB_ICONSTOP|MB_OK "KidOS could not verify safe Windows recovery. Uninstall will stop to avoid leaving the child account locked."
    Abort
  ${EndIf}
!macroend
