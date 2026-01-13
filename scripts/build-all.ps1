# Productivity Hub - Build All Script
# This script builds all tools and then the hub-dashboard with installer

param(
    [switch]$Release = $false,
    [switch]$SkipTools = $false
)

$ErrorActionPreference = "Stop"

# Get the workspace root (parent of scripts directory)
$WorkspaceRoot = Split-Path -Parent $PSScriptRoot

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Productivity Hub Build Script" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$BuildType = if ($Release) { "release" } else { "debug" }
$CargoArgs = if ($Release) { @("--release") } else { @() }

Write-Host "Build type: $BuildType" -ForegroundColor Yellow
Write-Host "Workspace: $WorkspaceRoot" -ForegroundColor Yellow
Write-Host ""

# Change to workspace root
Push-Location $WorkspaceRoot

try {
    # Build all tools first
    if (-not $SkipTools) {
        $tools = @(
            @{ Name = "desk-talk"; Path = "tools/desk-talk" },
            @{ Name = "speak-selected"; Path = "tools/speak-selected" },
            @{ Name = "quick-assistant"; Path = "tools/quick-assistant" },
            @{ Name = "flatten-string"; Path = "tools/flatten-string" },
            @{ Name = "typo-fix"; Path = "tools/typo-fix" },
            @{ Name = "ocr-paste"; Path = "tools/ocr-paste" }
        )

        foreach ($tool in $tools) {
            Write-Host "Building $($tool.Name)..." -ForegroundColor Green
            
            Push-Location $tool.Path
            try {
                # For Tauri apps, use tauri build; for others, use cargo build
                $cargoToml = Get-Content "Cargo.toml" -Raw
                if ($cargoToml -match "tauri") {
                    Write-Host "  (Tauri app - using cargo build for binary only)" -ForegroundColor Gray
                    cargo build @CargoArgs
                } else {
                    cargo build @CargoArgs
                }
                
                if ($LASTEXITCODE -ne 0) {
                    throw "Failed to build $($tool.Name)"
                }
                Write-Host "  Done!" -ForegroundColor Green
            }
            finally {
                Pop-Location
            }
        }

        Write-Host ""
        Write-Host "All tools built successfully!" -ForegroundColor Green
        Write-Host ""
    }

    # Now copy the tool binaries to the hub-dashboard resources directory
    Write-Host "Copying tool binaries..." -ForegroundColor Green
    
    $toolsBinDir = "crates/hub-dashboard/resources/tools"
    New-Item -ItemType Directory -Force -Path $toolsBinDir | Out-Null

    $binaries = @(
        @{ Source = "tools/desk-talk/target/$BuildType/desk-talk.exe"; Name = "desk-talk.exe" },
        @{ Source = "tools/speak-selected/target/$BuildType/speak-selected.exe"; Name = "speak-selected.exe" },
        @{ Source = "tools/quick-assistant/target/$BuildType/quick-assistant.exe"; Name = "quick-assistant.exe" },
        @{ Source = "tools/flatten-string/target/$BuildType/strflatten.exe"; Name = "strflatten.exe" },
        @{ Source = "tools/typo-fix/target/$BuildType/typo-fix.exe"; Name = "typo-fix.exe" },
        @{ Source = "tools/ocr-paste/target/$BuildType/ocrp.exe"; Name = "ocrp.exe" }
    )

    foreach ($bin in $binaries) {
        if (Test-Path $bin.Source) {
            Copy-Item -Path $bin.Source -Destination "$toolsBinDir/$($bin.Name)" -Force
            Write-Host "  Copied $($bin.Name)" -ForegroundColor Gray
        } else {
            Write-Host "  Warning: $($bin.Source) not found" -ForegroundColor Yellow
        }
    }

    Write-Host ""
    
    # Build the hub-dashboard
    Write-Host "Building hub-dashboard..." -ForegroundColor Green
    
    if ($Release) {
        # For release, use the release config and tauri build to create the installer
        Push-Location "crates/hub-dashboard"
        try {
            # Swap to release config
            Copy-Item -Path "tauri.conf.json" -Destination "tauri.conf.dev.json" -Force
            Copy-Item -Path "tauri.conf.release.json" -Destination "tauri.conf.json" -Force
            
            try {
                cargo tauri build
                if ($LASTEXITCODE -ne 0) {
                    throw "Failed to build hub-dashboard installer"
                }
            }
            finally {
                # Restore dev config
                Copy-Item -Path "tauri.conf.dev.json" -Destination "tauri.conf.json" -Force
                Remove-Item -Path "tauri.conf.dev.json" -Force
            }
        }
        finally {
            Pop-Location
        }
        
        Write-Host ""
        Write-Host "========================================" -ForegroundColor Cyan
        Write-Host "  Build Complete!" -ForegroundColor Cyan
        Write-Host "========================================" -ForegroundColor Cyan
        Write-Host ""
        Write-Host "Installer location:" -ForegroundColor Yellow
        Write-Host "  target/release/bundle/nsis/" -ForegroundColor White
    } else {
        cargo build -p hub-dashboard @CargoArgs
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to build hub-dashboard"
        }
        
        Write-Host ""
        Write-Host "========================================" -ForegroundColor Cyan
        Write-Host "  Build Complete!" -ForegroundColor Cyan
        Write-Host "========================================" -ForegroundColor Cyan
        Write-Host ""
        Write-Host "To run the hub (development):" -ForegroundColor Yellow
        Write-Host "  cargo run -p hub-dashboard" -ForegroundColor White
    }
}
finally {
    Pop-Location
}
