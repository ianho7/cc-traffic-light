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

function Show-Artifact {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Label,
        [Parameter(Mandatory = $true)]
        [string]$RelativePath
    )

    $path = Join-Path $repoRoot $RelativePath
    if (-not (Test-Path $path)) {
        Write-Warning "$Label missing: $path"
        return
    }

    $item = Get-Item $path
    Write-Host ("    {0}: {1} | {2:yyyy-MM-dd HH:mm:ss} | {3} bytes" -f $Label, $item.FullName, $item.LastWriteTime, $item.Length)
}

if (-not $SkipFrontend) {
    Invoke-Step -Name 'Build settings frontend assets' -Action {
        & pnpm -C taskbar-settings-tauri build
        if ($LASTEXITCODE -ne 0) {
            throw "Frontend build failed with exit code $LASTEXITCODE."
        }
    }
}

Invoke-Step -Name 'Build standalone Tauri settings binary (release)' -Action {
    & cargo build -p taskbar-settings-tauri --release --offline
    if ($LASTEXITCODE -ne 0) {
        throw "Tauri settings release build failed with exit code $LASTEXITCODE."
    }
}

Invoke-Step -Name 'Build Win32 host binary (release)' -Action {
    & cargo build -p taskbar-widget --release --offline
    if ($LASTEXITCODE -ne 0) {
        throw "Taskbar widget release build failed with exit code $LASTEXITCODE."
    }
}

Write-Host '==> Release artifacts' -ForegroundColor Cyan
Show-Artifact -Label 'Host' -RelativePath 'target\release\taskbar-widget.exe'
Show-Artifact -Label 'Tauri settings' -RelativePath 'target\release\taskbar-settings-tauri.exe'

Write-Host '==> Pack complete — ready to compile installer.iss' -ForegroundColor Green
Write-Host '    Run: ISCC installer.iss' -ForegroundColor Yellow
