# Quick build script - builds all tools then the hub
# Usage: .\build.ps1 or .\build.ps1 -Release

param([switch]$Release)

$ErrorActionPreference = "Stop"
$args = if ($Release) { @("--release") } else { @() }

Write-Host "Building all tools..." -ForegroundColor Cyan

@("desk-talk", "speak-selected", "quick-assistant", "flatten-string", "typo-fix", "ocr-paste") | ForEach-Object {
    Write-Host "  Building $_..." -ForegroundColor Gray
    Push-Location "tools\$_"
    cargo build @args
    if ($LASTEXITCODE -ne 0) { throw "Failed to build $_" }
    Pop-Location
}

Write-Host "Building hub-dashboard..." -ForegroundColor Cyan
cargo build -p hub-dashboard @args

Write-Host "`nDone! Run with: cargo run -p hub-dashboard $($Release ? '--release' : '')" -ForegroundColor Green
