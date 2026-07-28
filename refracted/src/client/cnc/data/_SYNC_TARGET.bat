@echo off
title WebUI Sync to Target
echo ===================================================
echo Syncing WebUI data to target\release directory...
echo ===================================================

:: %~dp0 gives the directory of the script including trailing backslash
set "SOURCE=%~dp0"
:: Remove trailing backslash for robocopy
set "SOURCE=%SOURCE:~0,-1%"
set "TARGET=%~dp0..\..\..\..\target\release\data\client\cnc"

:: /E copies all subdirectories including empty ones
:: /XF _SYNC_TARGET.bat excludes this script from being copied
:: /MT:4 enables multi-threading for faster copying
robocopy "%SOURCE%" "%TARGET%" /E /XF _SYNC_TARGET.bat /MT:4

:: Robocopy exit codes < 8 are considered successful
if %ERRORLEVEL% GEQ 8 (
    echo.
    echo [ERROR] Sync failed! Robocopy exit code: %ERRORLEVEL%
    exit /b %ERRORLEVEL%
)

echo.
echo [SUCCESS] Sync complete!
