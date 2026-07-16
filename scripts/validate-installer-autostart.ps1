param(
    [string]$InstallerPath = (Join-Path $PSScriptRoot "..\installer.iss")
)

$task = Get-Content $InstallerPath | Where-Object { $_ -match '^Name: "autostart";' }

if ($task.Count -ne 1) {
    throw "Expected exactly one autostart task in $InstallerPath, found $($task.Count)."
}

if ($task -notmatch 'Flags:\s*[^;]*\bunchecked\b') {
    throw "The autostart task must include Flags: unchecked so installation does not enable startup by default."
}

Write-Output "PASS: the autostart task defaults to unchecked."
