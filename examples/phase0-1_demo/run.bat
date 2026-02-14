@echo off
REM Quick run script for phase0-1_demo

echo ========================================
echo Luminara Phase 0-1 Demo - Quick Run
echo ========================================
echo.

REM Check for release build
if exist "..\..\target\release\ultimate_demo.exe" (
    echo Running RELEASE build...
    echo.
    ..\..\target\release\ultimate_demo.exe
) else (
    echo Release build not found. Building and running...
    echo.
    cargo run --release
)

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo ERROR: Demo failed to run!
    pause
    exit /b 1
)
