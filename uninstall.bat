@echo off
setlocal enabledelayedexpansion
cd /d "%~dp0"

echo ======================================================================
echo   PROGIT - Git Productivity Enhancer (Uninstaller)
echo ======================================================================
echo.

if "%1" neq "-y" if "%1" neq "--yes" (
    set /p "CONFIRM=Are you sure you want to uninstall progit? [Y/n]: "
    if /i "!CONFIRM!"=="n" (
        echo [!] Uninstallation cancelled.
        exit /b 0
    )
)

echo.
echo [*] Removing progit executable...
if exist "%USERPROFILE%\.cargo\bin\progit.exe" del /f /q "%USERPROFILE%\.cargo\bin\progit.exe" >nul 2>nul
if exist "target" rmdir /s /q "target" >nul 2>nul

echo.
echo ======================================================================
echo   [+] Successfully uninstalled progit.
echo ======================================================================
echo.
