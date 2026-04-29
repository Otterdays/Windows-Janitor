@echo off
REM Janitor PC Cleaner - Launch Script
REM Safe Windows PC Cleaning Tool

setlocal enabledelayedexpansion

cls
echo.
echo  ============================================================
echo  Janitor - Safe Windows PC Cleaner (Read-Only Phase 1)
echo  ============================================================
echo.
echo  UI available: run "cargo build --release -p janitor-ui" first.
echo.

REM Check if cargo is installed
cargo --version >nul 2>&1
if errorlevel 1 (
    echo Error: Rust/Cargo not found.
    echo Please install Rust from https://rustup.rs/
    pause
    exit /b 1
)

echo Building Janitor...
echo.

cd /d "%~dp0"
cargo build --release --quiet 2>nul
if errorlevel 1 (
    echo.
    echo Build failed. Running verbose build for diagnostics...
    cargo build --release
    pause
    exit /b 1
)

echo Build successful!
echo.
echo Choose an action:
echo.
echo   1) Quick Scan Report (human-readable)
echo   2) Full JSON Report
echo   3) Scan with Filters (by size, risk, category)
echo   4) Export to HTML
echo   5) List Available Scanners
echo   6) Show About / Safety Info
echo   7) Launch Desktop UI (requires janitor-ui build)
echo   8) Exit
echo.
set /p choice="Enter your choice (1-8): "

if "%choice%"=="1" (
    cls
    echo Scanning your system...
    echo.
    call target\release\janitor scan
) else if "%choice%"=="2" (
    cls
    echo Scanning your system...
    echo.
    call target\release\janitor scan --json
) else if "%choice%"=="3" (
    cls
    echo Filter Options:
    echo.
    set /p minsize="  Minimum size (MB, or press Enter to skip): "
    set /p risk="  Risk level (low/medium/high, or press Enter to skip): "
    set /p category="  Category name (or press Enter to skip): "
    echo.
    echo Scanning...
    echo.
    if "!minsize!"=="" (
        if "!risk!"=="" (
            if "!category!"=="" (
                call target\release\janitor scan
            ) else (
                call target\release\janitor scan --category "!category!"
            )
        ) else (
            if "!category!"=="" (
                call target\release\janitor scan --risk "!risk!"
            ) else (
                call target\release\janitor scan --risk "!risk!" --category "!category!"
            )
        )
    ) else (
        if "!risk!"=="" (
            if "!category!"=="" (
                call target\release\janitor scan --min-size-mb !minsize!
            ) else (
                call target\release\janitor scan --min-size-mb !minsize! --category "!category!"
            )
        ) else (
            if "!category!"=="" (
                call target\release\janitor scan --min-size-mb !minsize! --risk "!risk!"
            ) else (
                call target\release\janitor scan --min-size-mb !minsize! --risk "!risk!" --category "!category!"
            )
        )
    )
) else if "%choice%"=="4" (
    cls
    set /p filename="Enter output filename (e.g., report.html): "
    if "!filename!"=="" (
        set filename=janitor-report.html
    )
    echo.
    echo Scanning and generating HTML report...
    echo.
    call target\release\janitor scan --html "!filename!"
    if errorlevel 0 (
        echo.
        echo Report saved as: !filename!
        timeout /t 2 /nobreak
        start "" "!filename!"
    )
) else if "%choice%"=="5" (
    cls
    call target\release\janitor list
) else if "%choice%"=="6" (
    cls
    call target\release\janitor about
) else if "%choice%"=="7" (
    cls
    echo Launching Janitor Desktop UI...
    if exist "target\release\janitor-ui.exe" (
        start "" target\release\janitor-ui.exe
    ) else (
        echo.
        echo UI binary not found. Building now (this takes a few minutes)...
        echo Note: Requires Tauri system dependencies (WebView2).
        echo.
        cargo build --release -p janitor-ui
        if errorlevel 1 (
            echo.
            echo UI build failed. Install WebView2: https://developer.microsoft.com/en-us/microsoft-edge/webview2/
            pause
        ) else (
            start "" target\release\janitor-ui.exe
        )
    )
) else if "%choice%"=="8" (
    exit /b 0
) else (
    echo Invalid choice. Exiting.
    exit /b 1
)

echo.
pause
