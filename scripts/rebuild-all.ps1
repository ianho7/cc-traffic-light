param(
    [switch]$SkipFrontend
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

& "$PSScriptRoot\assert-version-sync.ps1"

function Get-ArtifactPath {
    param(
        [Parameter(Mandatory = $true)]
        [string]$RelativePath
    )

    Join-Path $repoRoot $RelativePath
}

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

    $path = Get-ArtifactPath -RelativePath $RelativePath
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

Invoke-Step -Name 'Build standalone Tauri settings binary' -Action {
    & cargo build -p taskbar-settings-tauri --offline
    if ($LASTEXITCODE -ne 0) {
        throw "Tauri settings build failed with exit code $LASTEXITCODE."
    }
}

Invoke-Step -Name 'Build hook CLI explicitly' -Action {
    & cargo build -p taskbar-widget --bin taskbar_widget_hook --offline
    if ($LASTEXITCODE -ne 0) {
        throw "Hook CLI build failed with exit code $LASTEXITCODE."
    }
}

Invoke-Step -Name 'Build Win32 host binary' -Action {
    & cargo build -p taskbar-widget --offline
    if ($LASTEXITCODE -ne 0) {
        throw "Taskbar widget build failed with exit code $LASTEXITCODE."
    }
}

Write-Host '==> Build artifacts' -ForegroundColor Cyan
Show-Artifact -Label 'Hook CLI' -RelativePath 'target\debug\taskbar_widget_hook.exe'
Show-Artifact -Label 'Host' -RelativePath 'target\debug\taskbar-widget.exe'
Show-Artifact -Label 'Tauri settings' -RelativePath 'target\debug\taskbar-settings-tauri.exe'

Write-Host '==> Rebuild complete' -ForegroundColor Green
