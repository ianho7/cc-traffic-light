param(
    [switch]$SkipFrontend
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

function Invoke-Step {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Name,
        [Parameter(Mandatory = $true)]
        [scriptblock]$Action
    )

    Write-Host "==> $Name" -ForegroundColor Cyan
    & $Action
}

if (-not $SkipFrontend) {
    Invoke-Step -Name 'Build settings frontend assets' -Action {
        & pnpm -C taskbar-settings-tauri build
        if ($LASTEXITCODE -ne 0) {
            throw "Frontend build failed with exit code $LASTEXITCODE."
        }
    }
}

Invoke-Step -Name 'Build standalone Tauri settings binary' -Action {
    & cargo build -p taskbar-settings-tauri --offline
    if ($LASTEXITCODE -ne 0) {
        throw "Tauri settings build failed with exit code $LASTEXITCODE."
    }
}

Invoke-Step -Name 'Build Win32 host binary' -Action {
    & cargo build -p taskbar-widget --offline
    if ($LASTEXITCODE -ne 0) {
        throw "Taskbar widget build failed with exit code $LASTEXITCODE."
    }
}

Write-Host '==> Rebuild complete' -ForegroundColor Green
