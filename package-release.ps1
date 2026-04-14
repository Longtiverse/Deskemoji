#!/usr/bin/env pwsh
# Package Deskemoji portable release

$ErrorActionPreference = "Stop"

$ProjectRoot = $PSScriptRoot
$ExePath = Join-Path $ProjectRoot "target\release\deskemoji.exe"
$AssetsDir = Join-Path $ProjectRoot "assets\emoji"
$DistDir = Join-Path $ProjectRoot "dist"

if (-not (Test-Path $ExePath)) {
    Write-Host "ERROR: Release executable not found: $ExePath"
    Write-Host "Please run: cargo build --release"
    exit 1
}

if (-not (Test-Path $AssetsDir)) {
    Write-Host "ERROR: Assets directory not found: $AssetsDir"
    exit 1
}

$Timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$ReleaseName = "deskemoji-portable-$Timestamp"
$TempDir = Join-Path $DistDir $ReleaseName
$ZipPath = Join-Path $DistDir "$ReleaseName.zip"

if (Test-Path $TempDir) { Remove-Item -Recurse -Force $TempDir }
if (Test-Path $ZipPath) { Remove-Item -Force $ZipPath }
New-Item -ItemType Directory -Path $TempDir | Out-Null

Copy-Item $ExePath (Join-Path $TempDir "deskemoji.exe")
$EmojiDest = Join-Path $TempDir "assets\emoji"
New-Item -ItemType Directory -Path $EmojiDest | Out-Null
Get-ChildItem -Path $AssetsDir -Filter "*.png" | ForEach-Object {
    Copy-Item $_.FullName $EmojiDest
}

Compress-Archive -Path "$TempDir\*" -DestinationPath $ZipPath -Force
Remove-Item -Recurse -Force $TempDir

$pngCount = (Get-ChildItem $AssetsDir -Filter "*.png").Count
Write-Host "SUCCESS: Portable package created!" -ForegroundColor Green
Write-Host "  Output: $ZipPath"
Write-Host "  Contents:"
Write-Host "    - deskemoji.exe"
Write-Host "    - assets/emoji/ ($pngCount PNG files)"
Write-Host ""
Write-Host "USAGE: Extract the ZIP to any folder and double-click deskemoji.exe to run." -ForegroundColor Cyan
