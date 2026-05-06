@echo off
setlocal
title NutriSurvey 2.0 - Setup

set "ROOT=%~dp0"
set "ASSETS=%ROOT%Assets"
set "INSTALLER=%ASSETS%\INSTALL.bat"
set "LAUNCHER=%ROOT%Launch_NutriSurvey.bat"
set "ICON=%ASSETS%\NutriSurvey.ico"
set "SHORTCUT_NAME=NutriSurvey 2.0.lnk"

echo ============================================
echo         NutriSurvey 2.0 - Setup
echo ============================================
echo.

if not exist "%INSTALLER%" (
    echo [ERROR] File installer tidak ditemukan: %INSTALLER%
    pause
    exit /b 1
)

if not exist "%LAUNCHER%" (
    echo [ERROR] File launcher tidak ditemukan: %LAUNCHER%
    pause
    exit /b 1
)

echo [1/2] Menjalankan installer dependency...
call "%INSTALLER%"
if errorlevel 1 (
    echo [WARN] Installer selesai dengan status error. Lanjut membuat shortcut.
)

echo.
echo [2/2] Membuat shortcut .lnk...

powershell -NoProfile -ExecutionPolicy Bypass -Command "$desktop=[Environment]::GetFolderPath('Desktop'); $shortcutPath=Join-Path $desktop '%SHORTCUT_NAME%'; $w=New-Object -ComObject WScript.Shell; $s=$w.CreateShortcut($shortcutPath); $s.TargetPath='%LAUNCHER%'; $s.WorkingDirectory='%ROOT%'; if (Test-Path '%ICON%') { $s.IconLocation='%ICON%' }; $s.Description='Launcher NutriSurvey 2.0'; $s.Save()"

if errorlevel 1 (
    echo [ERROR] Gagal membuat shortcut di Desktop.
    pause
    exit /b 1
)

echo [OK] Shortcut berhasil dibuat: %UserProfile%\Desktop\%SHORTCUT_NAME%
echo [OK] Setup selesai. Jalankan shortcut untuk membuka NutriSurvey.
pause
endlocal
exit /b 0
