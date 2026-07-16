param(
    [Parameter(Mandatory = $true, Position = 0)]
    [string]$Version
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

if ($Version -notmatch '^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$') {
    throw "Version must be valid semantic versioning, for example 0.1.1 or 0.1.1-rc.1."
}

$repoRoot = Split-Path -Parent $PSScriptRoot

function Replace-Required {
    param(
        [Parameter(Mandatory = $true)][string]$RelativePath,
        [Parameter(Mandatory = $true)][string]$Pattern,
        [Parameter(Mandatory = $true)][string]$Replacement
    )

    $path = Join-Path $repoRoot $RelativePath
    $text = Get-Content -Raw $path
    $updated = [regex]::Replace($text, $Pattern, $Replacement, [System.Text.RegularExpressions.RegexOptions]::Multiline)
    if ($updated -eq $text) {
        throw "Could not update version in $RelativePath."
    }
    [System.IO.File]::WriteAllText($path, $updated, [System.Text.UTF8Encoding]::new($false))
}

Replace-Required -RelativePath 'Cargo.toml' -Pattern '^version\s*=\s*"[^"]+"\r?$' -Replacement "version = `"$Version`""
Replace-Required -RelativePath 'package.json' -Pattern '^(\s*"version"\s*:\s*)"[^"]+"' -Replacement "`${1}`"$Version`""
Replace-Required -RelativePath 'taskbar-settings-tauri/package.json' -Pattern '^(\s*"version"\s*:\s*)"[^"]+"' -Replacement "`${1}`"$Version`""
Replace-Required -RelativePath 'taskbar-settings-tauri/src-tauri/tauri.conf.json' -Pattern '^(\s*"version"\s*:\s*)"[^"]+"' -Replacement "`${1}`"$Version`""
Replace-Required -RelativePath 'installer.iss' -Pattern '^#define MyAppVersion "[^"]+"\r?$' -Replacement "#define MyAppVersion `"$Version`""

& "$PSScriptRoot\assert-version-sync.ps1"
