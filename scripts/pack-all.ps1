param(
    [switch]$SkipFrontend,
    [switch]$ValidateOnly
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

& "$PSScriptRoot\assert-version-sync.ps1"

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
        throw "$Label missing: $path"
    }

    $item = Get-Item $path
    Write-Host ("    {0}: {1} | {2:yyyy-MM-dd HH:mm:ss} | {3} bytes" -f $Label, $item.FullName, $item.LastWriteTime, $item.Length)
}

if ($ValidateOnly) {
    Invoke-Step -Name 'Validate release hook CLI' -Action {
        $hookPath = Join-Path $repoRoot 'target\release\taskbar_widget_hook.exe'
        if (-not (Test-Path $hookPath)) {
            throw "Hook CLI artifact missing: $hookPath"
        }

        $version = (& $hookPath --version | Out-String).Trim()
        if ([string]::IsNullOrWhiteSpace($version)) {
            throw "Hook CLI version check returned no output: $hookPath"
        }
        Write-Host "    Hook CLI version: $version"
    }

    Write-Host '==> Release artifacts' -ForegroundColor Cyan
    Show-Artifact -Label 'Host' -RelativePath 'target\release\taskbar-widget.exe'
    Show-Artifact -Label 'Tauri settings' -RelativePath 'target\release\taskbar-settings-tauri.exe'
    Show-Artifact -Label 'Hook CLI' -RelativePath 'target\release\taskbar_widget_hook.exe'
    Write-Host '==> Release artifact validation complete' -ForegroundColor Green
    exit 0
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

Invoke-Step -Name 'Build hook CLI (release)' -Action {
    & cargo build -p taskbar-widget --bin taskbar_widget_hook --release --offline
    if ($LASTEXITCODE -ne 0) {
        throw "Hook CLI release build failed with exit code $LASTEXITCODE."
    }
}

Invoke-Step -Name 'Build Win32 host binary (release)' -Action {
    & cargo build -p taskbar-widget --release --offline
    if ($LASTEXITCODE -ne 0) {
        throw "Taskbar widget release build failed with exit code $LASTEXITCODE."
    }
}

Invoke-Step -Name 'Validate release hook CLI' -Action {
    $hookPath = Join-Path $repoRoot 'target\release\taskbar_widget_hook.exe'
    if (-not (Test-Path $hookPath)) {
        throw "Hook CLI artifact missing: $hookPath"
    }

    $version = (& $hookPath --version | Out-String).Trim()
    if ([string]::IsNullOrWhiteSpace($version)) {
        throw "Hook CLI version check returned no output: $hookPath"
    }
    Write-Host "    Hook CLI version: $version"
}

Write-Host '==> Release artifacts' -ForegroundColor Cyan
Show-Artifact -Label 'Host' -RelativePath 'target\release\taskbar-widget.exe'
Show-Artifact -Label 'Tauri settings' -RelativePath 'target\release\taskbar-settings-tauri.exe'
Show-Artifact -Label 'Hook CLI' -RelativePath 'target\release\taskbar_widget_hook.exe'

Write-Host '==> Pack complete — ready to compile installer.iss' -ForegroundColor Green
Write-Host '    Run: ISCC installer.iss' -ForegroundColor Yellow
