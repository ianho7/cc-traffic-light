$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repoRoot = Split-Path -Parent $PSScriptRoot

function Get-FileText {
    param([Parameter(Mandatory = $true)][string]$RelativePath)

    Get-Content -Raw (Join-Path $repoRoot $RelativePath)
}

function Get-RequiredMatch {
    param(
        [Parameter(Mandatory = $true)][string]$Text,
        [Parameter(Mandatory = $true)][string]$Pattern,
        [Parameter(Mandatory = $true)][string]$Label
    )

    $match = [regex]::Match($Text, $Pattern, [System.Text.RegularExpressions.RegexOptions]::Multiline)
    if (-not $match.Success) {
        throw "Could not read $Label version."
    }
    $match.Groups['version'].Value
}

$workspaceVersion = Get-RequiredMatch -Text (Get-FileText 'Cargo.toml') -Pattern '^version\s*=\s*"(?<version>[^"]+)"\r?$' -Label 'workspace'
$expectedInheritedVersion = 'version.workspace = true'
foreach ($manifest in @('taskbar-widget/Cargo.toml', 'taskbar-settings-tauri/src-tauri/Cargo.toml')) {
    if ((Get-FileText $manifest) -notmatch [regex]::Escape($expectedInheritedVersion)) {
        throw "$manifest must inherit version from the workspace."
    }
}

$versions = [ordered]@{
    'root package.json' = Get-RequiredMatch -Text (Get-FileText 'package.json') -Pattern '^\s*"version"\s*:\s*"(?<version>[^"]+)"' -Label 'root package.json'
    'settings package.json' = Get-RequiredMatch -Text (Get-FileText 'taskbar-settings-tauri/package.json') -Pattern '^\s*"version"\s*:\s*"(?<version>[^"]+)"' -Label 'settings package.json'
    'Tauri config' = Get-RequiredMatch -Text (Get-FileText 'taskbar-settings-tauri/src-tauri/tauri.conf.json') -Pattern '^\s*"version"\s*:\s*"(?<version>[^"]+)"' -Label 'Tauri config'
    'installer' = Get-RequiredMatch -Text (Get-FileText 'installer.iss') -Pattern '^#define MyAppVersion "(?<version>[^"]+)"\r?$' -Label 'installer'
}

$mismatches = $versions.GetEnumerator() | Where-Object { $_.Value -ne $workspaceVersion }
if ($mismatches) {
    $details = $mismatches | ForEach-Object { "$($_.Key)=$($_.Value)" }
    throw "Version mismatch: workspace=$workspaceVersion; $($details -join '; ')"
}

Write-Host "Version metadata synchronized: $workspaceVersion" -ForegroundColor Green
