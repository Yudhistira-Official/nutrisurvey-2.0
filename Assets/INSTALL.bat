@echo off
setlocal enabledelayedexpansion
title NutriSurvey 2.0 - Installer

:: ==========================================
:: 0. MEMINTA AKSES ADMINISTRATOR OTOMATIS
:: ==========================================
net session >nul 2>&1
if %errorlevel% neq 0 (
    echo Meminta akses Administrator...
    powershell -Command "Start-Process -FilePath '%~dpnx0' -Verb RunAs"
    exit /b
)

:: PENTING: Kembalikan direktori aktif ke lokasi file .bat ini berada.
cd /d "%~dp0"

:: Gunakan link alias permanen Microsoft untuk versi terbaru .NET 8
set "DOTNET_URL=https://aka.ms/dotnet/8.0/dotnet-sdk-win-x64.exe"
set "INSTALLER_EXE=%temp%\dotnet_sdk_installer.exe"

echo ============================================
echo        NutriSurvey 2.0 - System Setup
echo ============================================
echo                    [INFO]
echo   INI ADALAH PROJECT DARI ANGKATAN 2024 FK
echo    UNTUK MENGGANTIKAN NutriSurvey.de YANG
echo  YANG LAMA, UNTUK PENGGUNAANNYA MEMBUTUHKAN
echo             BEBERAPA KONFIGURASI
echo.

:: 1. Cek apakah .NET 8 sudah ada
dotnet --version >nul 2>&1
if %errorlevel% equ 0 (
    echo [OK] .NET 8 sudah terinstall di sistem Anda.
    echo Anda bisa langsung menutup jendela ini dan menjalankan NutriSurvey.lnk
    echo.
    pause
    exit /b
)

:: 2. Jika tidak ada, tawarkan instalasi otomatis
echo [INFO] .NET 8 SDK tidak ditemukan di sistem Anda.
echo [INFO] .NET 8 diperlukan untuk menjalankan fitur penghitungan gizi.
echo.
set /p "choice=Apakah Anda ingin mengunduh dan menginstall .NET 8 sekarang? (Y/N): "

if /i "%choice%" neq "Y" (
    echo.
    echo [WARN] Instalasi dibatalkan. Aplikasi tidak akan berfungsi.
    pause
    exit /b
)

:: 3. Proses Unduh menggunakan CURL
echo.
echo [1/2] Sedang mengunduh .NET 8 SDK (sekitar 200MB)...
echo [INFO] Harap tunggu, proses ini bergantung pada kecepatan internet Anda...
curl -L -o "%INSTALLER_EXE%" "%DOTNET_URL%"

if not exist "%INSTALLER_EXE%" (
    echo [ERROR] Gagal mengunduh installer. Periksa koneksi internet Anda.
    pause
    exit /b
)

:: 4. Jalankan Installer
echo [2/2] Menjalankan Installer .NET 8...
echo [INFO] Jendela instalasi akan muncul. Silakan tunggu hingga progress bar selesai...
start /wait "" "%INSTALLER_EXE%" /install /passive /norestart

echo.
echo ============================================
echo [OK] PROSES INSTALASI SELESAI.
echo Silakan TUTUP jendela ini, kemudian jalankan file NutriSurvey.lnk 
echo untuk membuka aplikasi NutriSurvey 2.0.
echo ============================================
pause
exit
