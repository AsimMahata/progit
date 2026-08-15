@echo off
setlocal enabledelayedexpansion
cd /d "%~dp0"

echo ======================================================================
echo   PROGIT - Git Productivity Enhancer (Installer)
echo ======================================================================
echo.
echo [*] Requirements:
echo     - Rust and Cargo in PATH (https://rustup.rs/)
echo     - Git in PATH
echo.
echo [*] Tips:
echo     - Compiles in optimized release mode
echo     - Installs binary to %USERPROFILE%\.cargo\bin\progit.exe
echo     - Run 'progit --help' to see productivity workflows
echo     - If install fails due to permissions, run with 'sudow install.bat' (run 'tom install sudow' to get sudow)
echo.

if "%1" neq "-y" if "%1" neq "--yes" (
    set /p "CONFIRM=Proceed with installation? [Y/n]: "
    if /i "!CONFIRM!"=="n" (
        echo [!] Installation cancelled.
        exit /b 0
    )
)

echo.
echo [1/3] Checking Rust toolchain...
where cargo >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo [ERROR] 'cargo' was not found in PATH. Please install Rust from https://rustup.rs
    exit /b 1
)
echo       Rust toolchain detected.

echo.
echo [2/3] Compiling progit in release mode...
cargo build --release
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Compilation failed. If this is a permission issue, run with 'sudow install.bat' (or install sudow via 'tom install sudow').
    exit /b 1
)

echo.
echo [3/3] Installing binary to %USERPROFILE%\.cargo\bin\progit.exe...
if not exist "%USERPROFILE%\.cargo\bin" mkdir "%USERPROFILE%\.cargo\bin" 2>nul
copy /Y "target\release\progit.exe" "%USERPROFILE%\.cargo\bin\progit.exe" >nul
if %ERRORLEVEL% neq 0 (
    powershell -NoProfile -Command "Copy-Item -Path 'target\release\progit.exe' -Destination '%USERPROFILE%\.cargo\bin\progit.exe' -Force" >nul 2>nul
)
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Copying binary failed. If this is a permission issue, run with 'sudow install.bat' (or install sudow via 'tom install sudow').
    exit /b 1
)

echo.
echo ======================================================================
echo   [+] Successfully installed progit!
echo   Try running: progit --help
echo ======================================================================
echo.
