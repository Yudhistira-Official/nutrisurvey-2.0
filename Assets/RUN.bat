@echo off
setlocal enabledelayedexpansion
title NutriSurvey 2.0 - Launcher

cd /d "%~dp0"

echo ============================================
echo      NutriSurvey 2.0 - Peluncur Aplikasi
echo ============================================
echo.

dotnet --version >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] .NET 8 SDK tidak ditemukan.
    echo Harap jalankan Assets\INSTALL.bat terlebih dahulu.
    pause
    exit /b
)

echo [1/4] Memeriksa komponen antarmuka lokal...
dotnet serve --version >nul 2>&1
if %errorlevel% neq 0 (
    call dotnet tool install -g dotnet-serve >nul 2>&1
)

echo [2/4] Memulai server Backend...
echo [3/4] Memulai server Frontend...
echo [4/4] Membuka Microsoft Edge...
echo.
echo Antarmuka: http://localhost:8080
echo Backend API: http://localhost:5000
echo.

powershell -NoProfile -ExecutionPolicy Bypass -Command "& { $assetsDir = (Get-Location).Path; $baseDir = Split-Path -Parent $assetsDir; $backendDir = Join-Path $baseDir 'Backend'; $frontendDir = Join-Path $baseDir 'Frontend'; $s1 = Join-Path $assetsDir 'launch.step1'; $s2 = Join-Path $assetsDir 'launch.step2'; $s3 = Join-Path $assetsDir 'launch.step3'; $s4 = Join-Path $assetsDir 'launch.step4'; Remove-Item -Force -ErrorAction SilentlyContinue $s1,$s2,$s3,$s4; New-Item -ItemType File -Force -Path $s1 | Out-Null; $logDir = Join-Path $env:TEMP 'NutriSurveyLogs'; New-Item -ItemType Directory -Force -Path $logDir | Out-Null; $backendOut = Join-Path $logDir 'backend.out.log'; $backendErr = Join-Path $logDir 'backend.err.log'; $frontOut = Join-Path $logDir 'frontend.out.log'; $frontErr = Join-Path $logDir 'frontend.err.log'; $edgeProfile = Join-Path $env:TEMP 'NutriSurveyEdgeProfile'; $backend = Start-Process dotnet -ArgumentList 'run --urls http://localhost:5000' -WorkingDirectory $backendDir -WindowStyle Hidden -RedirectStandardOutput $backendOut -RedirectStandardError $backendErr -PassThru; Start-Sleep -Seconds 4; New-Item -ItemType File -Force -Path $s2 | Out-Null; $frontend = Start-Process dotnet -ArgumentList 'serve -d . -p 8080' -WorkingDirectory $frontendDir -WindowStyle Hidden -RedirectStandardOutput $frontOut -RedirectStandardError $frontErr -PassThru; Start-Sleep -Seconds 2; New-Item -ItemType File -Force -Path $s3 | Out-Null; try { $edge = Start-Process msedge -ArgumentList ('--new-window --user-data-dir="' + $edgeProfile + '" http://localhost:8080') -PassThru } catch { Start-Process 'http://localhost:8080' | Out-Null; $edge = $null }; New-Item -ItemType File -Force -Path $s4 | Out-Null; if ($edge -ne $null) { Wait-Process -Id $edge.Id } else { [void](Read-Host) }; if ($frontend -and -not $frontend.HasExited) { Stop-Process -Id $frontend.Id -Force }; if ($backend -and -not $backend.HasExited) { Stop-Process -Id $backend.Id -Force }; Remove-Item -Force -ErrorAction SilentlyContinue $s1,$s2,$s3,$s4 }"

if %errorlevel% neq 0 (
    echo.
    echo [ERROR] Launcher gagal jalan. Buka CMD manual, jalankan Assets\RUN.bat, lalu kirim pesan error.
    pause
)

exit /b
