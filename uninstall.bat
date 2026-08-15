@echo off
cd /d "%~dp0"
echo [*] Uninstalling progit...
if exist "%USERPROFILE%\.cargo\bin\progit.exe" del /f /q "%USERPROFILE%\.cargo\bin\progit.exe" >nul 2>nul
if exist "target" rmdir /s /q "target" >nul 2>nul
echo [✓] Successfully uninstalled progit.
