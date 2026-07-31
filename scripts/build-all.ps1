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
        # Cargo package names (they do not all match their directory names).
        # Building from the workspace root with -p keeps the working directory
        # stable, which matters because these tools are git submodules.
        $tools = @("desk-talk", "speak-selected", "quick-assistant", "strflatten", "typo-fix", "ocrp")

        foreach ($tool in $tools) {
            Write-Host "Building $tool..." -ForegroundColor Green
            cargo build -p $tool @CargoArgs
            if ($LASTEXITCODE -ne 0) {
                throw "Failed to build $tool"
            }
            Write-Host "  Done!" -ForegroundColor Green
        }

        Write-Host ""
        Write-Host "All tools built successfully!" -ForegroundColor Green
        Write-Host ""
    }

    # Now copy the tool binaries to the hub-dashboard resources directory
    Write-Host "Copying tool binaries..." -ForegroundColor Green
    
    $toolsBinDir = "crates/hub-dashboard/resources/tools"
    New-Item -ItemType Directory -Force -Path $toolsBinDir | Out-Null

    # Every tool is a member of this workspace, so cargo emits all of them into
    # the shared workspace target directory - not into tools/<tool>/target.
    $binaries = @(
        @{ Source = "target/$BuildType/desk-talk.exe"; Name = "desk-talk.exe" },
        @{ Source = "target/$BuildType/speak-selected.exe"; Name = "speak-selected.exe" },
        @{ Source = "target/$BuildType/quick-assistant.exe"; Name = "quick-assistant.exe" },
        @{ Source = "target/$BuildType/strflatten.exe"; Name = "strflatten.exe" },
        @{ Source = "target/$BuildType/typo-fix.exe"; Name = "typo-fix.exe" },
        @{ Source = "target/$BuildType/ocrp.exe"; Name = "ocrp.exe" }
    )

    foreach ($bin in $binaries) {
        if (-not (Test-Path $bin.Source)) {
            # Staging a stale binary would silently ship an old build.
            throw "Expected tool binary not found: $($bin.Source)"
        }
        Copy-Item -Path $bin.Source -Destination "$toolsBinDir/$($bin.Name)" -Force
        Write-Host "  Copied $($bin.Name)" -ForegroundColor Gray
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
