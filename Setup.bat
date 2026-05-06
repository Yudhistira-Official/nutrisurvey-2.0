@echo off
setlocal enabledelayedexpansion
title NutriSurvey 2.0 - Setup

net session >nul 2>&1
if %errorlevel% neq 0 (
    echo Meminta akses Administrator...
    powershell -NoProfile -ExecutionPolicy Bypass -Command "Start-Process -FilePath '%~f0' -Verb RunAs"
    exit /b
)

set "ROOT=%~dp0"
set "ASSETS=%ROOT%Assets"
set "VBS_LAUNCHER=%ASSETS%\NutriSurvey.vbs"
set "ICON=%ASSETS%\NutriSurvey.ico"
set "SHORTCUT_NAME=NutriSurvey 2.0.lnk"
set "DOTNET_URL=https://aka.ms/dotnet/8.0/dotnet-sdk-win-x64.exe"
set "INSTALLER_EXE=%temp%\dotnet_sdk_installer.exe"
set "SETUP_OK=0"

echo ============================================
echo         NutriSurvey 2.0 - Setup
echo ============================================
echo.

if not exist "%VBS_LAUNCHER%" (
    echo [ERROR] File launcher tidak ditemukan: %VBS_LAUNCHER%
    pause
    exit /b 1
)

echo [1/3] Membuat shortcut .lnk...
powershell -NoProfile -ExecutionPolicy Bypass -Command "$desktop=[Environment]::GetFolderPath('Desktop'); $shortcutPath=Join-Path $desktop 'NutriSurvey 2.0.lnk'; $localPath=Join-Path '%ROOT%' 'NutriSurvey 2.0.lnk'; $w=New-Object -ComObject WScript.Shell; foreach ($p in @($shortcutPath,$localPath)) { $s=$w.CreateShortcut($p); $s.TargetPath=Join-Path $env:WINDIR 'System32\wscript.exe'; $s.Arguments='"""%VBS_LAUNCHER%"""'; $s.WorkingDirectory='%ROOT%'; if (Test-Path '%ICON%') { $s.IconLocation='%ICON%' }; $s.Description='Launcher NutriSurvey 2.0'; $s.Save(); if (-not (Test-Path $p)) { throw ('Shortcut gagal dibuat: ' + $p) } }; Write-Host ('[OK] Shortcut Desktop: ' + $shortcutPath); Write-Host ('[OK] Shortcut Lokal: ' + $localPath)"
if errorlevel 1 (
    echo [ERROR] Gagal membuat shortcut di Desktop.
    pause
    exit /b 1
)
echo.

echo [2/3] Cek dependency (.NET 8 SDK)...
dotnet --version >nul 2>&1
if %errorlevel% equ 0 (
    echo [OK] .NET SDK sudah terinstall.
) else (
    echo [INFO] .NET 8 SDK belum terinstall.
    set /p "choice=Install .NET 8 SDK sekarang? (Y/N): "
    if /i not "!choice!"=="Y" (
        echo [WARN] Instalasi dependency dibatalkan user.
        echo [WARN] Aplikasi mungkin tidak berjalan sebelum .NET 8 SDK terpasang.
    ) else (
        echo [INFO] Mengunduh .NET 8 SDK...
        curl -L -o "%INSTALLER_EXE%" "%DOTNET_URL%"
        if not exist "%INSTALLER_EXE%" (
            echo [ERROR] Gagal mengunduh installer .NET.
            pause
            exit /b 1
        )

        echo [INFO] Menjalankan installer .NET 8 SDK...
        start /wait "" "%INSTALLER_EXE%" /install /passive /norestart
        dotnet --version >nul 2>&1
        if %errorlevel% equ 0 (
            echo [OK] .NET SDK berhasil terinstall.
        ) else (
            echo [ERROR] Instalasi .NET gagal diverifikasi.
            pause
            exit /b 1
        )
    )
)

set "SETUP_OK=1"
echo.
echo [3/3] Finalisasi...
echo [OK] Setup selesai. Jalankan shortcut Desktop untuk membuka NutriSurvey.

if "%SETUP_OK%"=="1" (
    set "SELF_PATH=%~f0"
    set "DELETER=%temp%\nutrisurvey_cleanup_%random%%random%.cmd"
    >"!DELETER!" echo @echo off
    >>"!DELETER!" echo ping 127.0.0.1 -n 3 ^>nul
    >>"!DELETER!" echo :retry
    >>"!DELETER!" echo del /f /q ""!SELF_PATH!"" ^>nul 2^>^&1
    >>"!DELETER!" echo if exist ""!SELF_PATH!"" ^(
    >>"!DELETER!" echo   ping 127.0.0.1 -n 2 ^>nul
    >>"!DELETER!" echo   goto retry
    >>"!DELETER!" echo ^)
    >>"!DELETER!" echo del /f /q "%%~f0" ^>nul 2^>^&1
    start "" /min cmd /c ""!DELETER!""
)

endlocal
exit /b 0
