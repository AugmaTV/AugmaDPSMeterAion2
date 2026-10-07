!macro NSIS_HOOK_POSTINSTALL
	${If} $UpdateMode = 1
		Exec '"$INSTDIR\${MAINBINARYNAME}.exe"'
	${EndIf}
!macroend