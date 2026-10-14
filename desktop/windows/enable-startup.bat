@echo off
rem Baddel: start automatically with Windows (current user, no admin needed).
rem Put this file next to baddel.exe and double-click it.
set "EXE=%~dp0baddel.exe"
if not exist "%EXE%" (
  echo baddel.exe was not found next to this file.
  pause
  exit /b 1
)
reg add "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v Baddel /t REG_SZ /d "\"%EXE%\"" /f >nul
if errorlevel 1 (
  echo Failed to write the startup entry.
  pause
  exit /b 1
)
echo Done. Baddel will start with Windows: %EXE%
echo If you move baddel.exe, run this file again.
start "" "%EXE%"
pause
