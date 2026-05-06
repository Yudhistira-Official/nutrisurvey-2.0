@echo off
setlocal enabledelayedexpansion
title NutriSurvey 2.0 - Setup

set "ROOT=%~dp0"
set "ASSETS=%ROOT%Assets"
set "VBS_LAUNCHER=%ASSETS%\NutriSurvey.vbs"
set "ICON=%ASSETS%\NutriSurvey.ico"
set "SHORTCUT_NAME=NutriSurvey 2.0.lnk"
set "DOTNET_INSTALL_SCRIPT=%temp%\dotnet-install.ps1"
set "DOTNET_USER_DIR=%LocalAppData%\Microsoft\dotnet"
set "SETUP_OK=0"

echo ============================================
echo         NutriSurvey 2.0 - Setup
echo ============================================
echo.

if not exist "%VBS_LAUNCHER%" (
    echo [ERROR] File launcher tidak ditemukan: %VBS_LAUNCHER%
    exit /b 1
)

echo [1/3] Membuat shortcut .lnk...
powershell -NoProfile -ExecutionPolicy Bypass -Command "$desktop=[Environment]::GetFolderPath('Desktop'); $shortcutPath=Join-Path $desktop 'NutriSurvey 2.0.lnk'; $localPath=Join-Path '%ROOT%' 'NutriSurvey 2.0.lnk'; $w=New-Object -ComObject WScript.Shell; foreach ($p in @($shortcutPath,$localPath)) { $s=$w.CreateShortcut($p); $s.TargetPath=Join-Path $env:WINDIR 'System32\wscript.exe'; $s.Arguments='"""%VBS_LAUNCHER%"""'; $s.WorkingDirectory='%ROOT%'; if (Test-Path '%ICON%') { $s.IconLocation='%ICON%' }; $s.Description='Launcher NutriSurvey 2.0'; $s.Save(); if (-not (Test-Path $p)) { throw ('Shortcut gagal dibuat: ' + $p) } }; Write-Host ('[OK] Shortcut Desktop: ' + $shortcutPath); Write-Host ('[OK] Shortcut Lokal: ' + $localPath)"
if errorlevel 1 (
    echo [ERROR] Gagal membuat shortcut di Desktop.
    exit /b 1
)
echo.

echo [2/3] Cek dependency (.NET 8)...
dotnet --list-sdks >nul 2>&1
if %errorlevel% equ 0 (
    echo [OK] .NET SDK sudah terinstall.
) else (
    echo [INFO] .NET belum ditemukan. Install per-user (tanpa admin)...
    curl -L -o "%DOTNET_INSTALL_SCRIPT%" "https://dot.net/v1/dotnet-install.ps1"
    if not exist "%DOTNET_INSTALL_SCRIPT%" (
        echo [ERROR] Gagal mengunduh dotnet-install.ps1.
        exit /b 1
    )

    powershell -NoProfile -ExecutionPolicy Bypass -File "%DOTNET_INSTALL_SCRIPT%" -Channel 8.0 -InstallDir "%DOTNET_USER_DIR%"
    if errorlevel 1 (
        echo [ERROR] Instalasi .NET per-user gagal.
        exit /b 1
    )

    setx PATH "%PATH%;%DOTNET_USER_DIR%" >nul
    if exist "%DOTNET_USER_DIR%\dotnet.exe" (
        echo [OK] .NET per-user berhasil terinstall: %DOTNET_USER_DIR%
    ) else (
        echo [ERROR] dotnet.exe tidak ditemukan setelah instalasi.
        exit /b 1
    )
)

set "SETUP_OK=1"
echo.
echo [3/3] Finalisasi...
echo [OK] Setup selesai. Jalankan shortcut Desktop untuk membuka NutriSurvey.
echo [INFO] Jika terminal lama masih terbuka, tutup dan buka ulang agar PATH baru aktif.

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
