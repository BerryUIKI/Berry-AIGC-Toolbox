; Omera custom NSIS installer hooks
; Enhances installation lifecycle, process safety and shortcut cleanup

!macro NSIS_HOOK_PREINSTALL
  ; Terminate any running Omera instances to prevent file lock during install / update
  nsExec::Exec 'taskkill /IM Omera.exe /F'
  nsExec::Exec 'taskkill /IM omera.exe /F'
  nsExec::Exec 'taskkill /IM Berry.exe /F'
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ; Clean up desktop shortcuts if created by user
  Delete "$DESKTOP\${PRODUCTNAME}.lnk"
  Delete "$DESKTOP\Omera.lnk"
  ; Clean up Start Menu folder if left behind
  RMDir "$SMPROGRAMS\${PRODUCTNAME}"
  RMDir "$SMPROGRAMS\Omera"
!macroend
