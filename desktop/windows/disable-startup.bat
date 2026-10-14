@echo off
reg delete "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v Baddel /f >/dev/null 2>&1
del "%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup\Baddel.lnk" >/dev/null 2>&1
echo Baddel will no longer start with Windows.
pause
