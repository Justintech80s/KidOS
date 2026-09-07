!define KIDOS_HOOK_DIR "${__FILEDIR__}"

!macro NSIS_HOOK_POSTINSTALL
  DetailPrint "Installing KidOS Guardian and local media classifier..."

  SetOutPath "$PROGRAMFILES64\KidOS\Guardian"
  File /oname=kidos-guardian-host.exe "${KIDOS_HOOK_DIR}\..\..\..\..\target\release\kidos-guardian-host.exe"
  File /oname=provision-guardian-credentials.ps1 "${KIDOS_HOOK_DIR}\..\..\..\..\scripts\windows\provision-guardian-credentials.ps1"
  File /oname=install-kidos-services.ps1 "${KIDOS_HOOK_DIR}\..\..\..\..\scripts\windows\install-kidos-services.ps1"

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
  nsExec::ExecToLog '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command "$names=@(''KidOSMediaClassifier'',''KidOSGuardian''); foreach($name in $names){ $svc=Get-Service -Name $name -ErrorAction SilentlyContinue; if($null -ne $svc){ if($svc.Status -ne ''Stopped''){ Stop-Service -Name $name -Force -ErrorAction SilentlyContinue; Start-Sleep -Milliseconds 300 }; & $env:SystemRoot\System32\sc.exe delete $name | Out-Null } }"'
  Sleep 1000

  ; Protect service binaries and model files so a standard child account cannot replace them.
  nsExec::ExecToStack '"$SYSDIR\icacls.exe" "$PROGRAMFILES64\KidOS" /inheritance:r /grant:r "SYSTEM:(OI)(CI)(F)" "Administrators:(OI)(CI)(F)" "Users:(OI)(CI)(RX)"'
  Pop $0
  Pop $1
  ${If} $0 != 0
    MessageBox MB_ICONSTOP|MB_OK "KidOS could not secure the Guardian service directory. Installation will stop."
    Abort
  ${EndIf}

  ; Provision protected Guardian credentials from a standalone PowerShell 5.1 script.
  ; Keeping the logic out of an inline -Command avoids NSIS/PowerShell nested-quote parsing bugs.
  nsExec::ExecToStack '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$PROGRAMFILES64\KidOS\Guardian\provision-guardian-credentials.ps1" -InstallerPath "$EXEPATH"'
  Pop $0
  Pop $1
  ${If} $0 != 0
    MessageBox MB_ICONSTOP|MB_OK "KidOS could not provision its protected Guardian credentials. Installation will stop."
    Abort
  ${EndIf}

  ; Register/start services through a standalone PowerShell 5.1 script.
  ; This avoids fragile nested NSIS/sc.exe quoting and cleans up on failure.
  nsExec::ExecToStack '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$PROGRAMFILES64\KidOS\Guardian\install-kidos-services.ps1" -GuardianExe "$PROGRAMFILES64\KidOS\Guardian\kidos-guardian-host.exe" -ClassifierExe "$PROGRAMFILES64\KidOS\MediaClassifier\kidos-media-classifier.exe"'
  Pop $0
  Pop $1
  ${If} $0 != 0
    MessageBox MB_ICONSTOP|MB_OK "KidOS could not register/start its Windows protection services. Installation will stop."
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
  File /oname=restore-windows-account.ps1 "${KIDOS_HOOK_DIR}\..\..\..\..\scripts\windows\restore-windows-account.ps1"
  File /oname=rollback-kidos.ps1 "${KIDOS_HOOK_DIR}\..\..\..\..\scripts\windows\rollback-kidos.ps1"
  nsExec::ExecToLog '"$SYSDIR\schtasks.exe" /Delete /TN "KidOS Guardian Recovery" /F'
  nsExec::ExecToStack '"$SYSDIR\schtasks.exe" /Create /TN "KidOS Guardian Recovery" /SC ONSTART /RU SYSTEM /RL HIGHEST /TR "$\"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe$\" -NoProfile -NonInteractive -ExecutionPolicy Bypass -File $\"$PROGRAMFILES64\KidOS\Recovery\kidos-recovery.ps1$\"" /F'
  Pop $0
  Pop $1
  ${If} $0 != 0
    MessageBox MB_ICONSTOP|MB_OK "KidOS could not install its recovery health check. Installation will stop."
    Abort
  ${EndIf}

  ; Cache the successfully installed package in a SYSTEM/administrator-only recovery area.
  ; On an upgrade, move the last known-good package to Previous before recording the new one.
  CreateDirectory "$PROGRAMDATA\KidOS\Recovery\Current"
  CreateDirectory "$PROGRAMDATA\KidOS\Recovery\Previous"
  IfFileExists "$PROGRAMDATA\KidOS\Recovery\Current\KidOS-current.exe" 0 +4
    CopyFiles /SILENT "$PROGRAMDATA\KidOS\Recovery\Current\KidOS-current.exe" "$PROGRAMDATA\KidOS\Recovery\Previous\KidOS-previous.exe"
    nsExec::ExecToLog '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command "(Get-FileHash -LiteralPath ''$PROGRAMDATA\KidOS\Recovery\Previous\KidOS-previous.exe'' -Algorithm SHA256).Hash.ToLowerInvariant() | Set-Content -LiteralPath ''$PROGRAMDATA\KidOS\Recovery\Previous\KidOS-previous.sha256'' -NoNewline -Encoding ASCII"'
  CopyFiles /SILENT "$EXEPATH" "$PROGRAMDATA\KidOS\Recovery\Current\KidOS-current.exe"
  nsExec::ExecToLog '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command "(Get-FileHash -LiteralPath ''$PROGRAMDATA\KidOS\Recovery\Current\KidOS-current.exe'' -Algorithm SHA256).Hash.ToLowerInvariant() | Set-Content -LiteralPath ''$PROGRAMDATA\KidOS\Recovery\Current\KidOS-current.sha256'' -NoNewline -Encoding ASCII"'
  nsExec::ExecToLog '"$SYSDIR\icacls.exe" "$PROGRAMDATA\KidOS\Recovery" /inheritance:r /grant:r "SYSTEM:(OI)(CI)(F)" "Administrators:(OI)(CI)(F)"'

  DetailPrint "KidOS Guardian and local media classifier are installed and running."
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  DetailPrint "Restoring the Windows child account before removing KidOS..."

  ; Run Assigned Access removal as SYSTEM. This deliberately happens before
  ; Guardian is removed so an uninstall cannot leave the child account locked.
  Delete "$PROGRAMDATA\KidOS\Recovery\restore-windows.result"
  nsExec::ExecToLog '"$SYSDIR\schtasks.exe" /Delete /TN "KidOS Restore Windows Account" /F'
  nsExec::ExecToStack '"$SYSDIR\schtasks.exe" /Create /TN "KidOS Restore Windows Account" /SC ONSTART /RU SYSTEM /RL HIGHEST /TR "$\"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe$\" -NoProfile -NonInteractive -ExecutionPolicy Bypass -File $\"$PROGRAMFILES64\KidOS\Recovery\restore-windows-account.ps1$\"" /F'
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
  IfFileExists "$PROGRAMDATA\KidOS\Recovery\restore-windows.result" +2 0
    Abort
  nsExec::ExecToStack '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command "$r=(Get-Content -LiteralPath ''$env:ProgramData\KidOS\Recovery\restore-windows.result'' -Raw); if($r -ne ''restored''){ exit 40 }"'
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
