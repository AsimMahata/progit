@echo off
cd /d "%~dp0"
echo [*] Building progit in release mode...
cargo build --release
if %ERRORLEVEL% equ 0 (
    if not exist "%USERPROFILE%\.cargo\bin" mkdir "%USERPROFILE%\.cargo\bin" 2>nul
    copy /Y "target\release\progit.exe" "%USERPROFILE%\.cargo\bin\progit.exe" >nul
    echo [✓] Successfully installed progit to %USERPROFILE%\.cargo\bin\progit.exe
)
