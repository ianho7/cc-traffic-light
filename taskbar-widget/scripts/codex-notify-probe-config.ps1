param(
    [string]$ConfigPath = "$env:USERPROFILE\.codex\config.toml",
    [string]$ProbeOutDir = "$env:TEMP\cc-traffic-light-codex-notify-probe",
    [string]$WrapperPath = "",
    [switch]$Apply,
    [switch]$Restore
)

$ErrorActionPreference = "Stop"

function Get-DefaultWrapperPath {
    $scriptDir = Split-Path -Parent $PSCommandPath
    Join-Path $scriptDir "codex-notify-probe-wrapper.ps1"
}

function Read-ConfigText {
    param([string]$Path)

    if (-not (Test-Path -LiteralPath $Path)) {
        throw "Codex config not found: $Path"
    }
    Get-Content -LiteralPath $Path -Raw
}

function Get-NotifyArrayLiteral {
    param([string]$Text)

    $match = [regex]::Match($Text, '(?m)^\s*notify\s*=\s*(\[[^\r\n]*\])\s*$')
    if (-not $match.Success) {
        throw "notify array not found in config"
    }
    $match.Groups[1].Value
}

function Convert-TomlStringArrayToItems {
    param([string]$ArrayLiteral)

    $items = @()
    foreach ($match in [regex]::Matches($ArrayLiteral, '"((?:\\.|[^"\\])*)"')) {
        $items += ($match.Groups[1].Value -replace '\\"', '"' -replace '\\\\', '\')
    }
    if ($items.Count -eq 0) {
        throw "notify array contains no string items"
    }
    $items
}

function Convert-ItemsToJsonBase64 {
    param([string[]]$Items)

    $jsonItems = $Items | ForEach-Object {
        '"' + ($_ -replace '\\', '\\' -replace '"', '\"') + '"'
    }
    $json = '[' + ($jsonItems -join ',') + ']'
    [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($json))
}

function Convert-StringToTomlBasicString {
    param([string]$Value)

    '"' + ($Value -replace '\\', '\\' -replace '"', '\"') + '"'
}

function New-ProbeNotifyLine {
    param(
        [string]$Wrapper,
        [string]$OutDir,
        [string]$ForwardJsonBase64
    )

    $items = @(
        "powershell.exe",
        "-NoProfile",
        "-ExecutionPolicy",
        "Bypass",
        "-File",
        $Wrapper,
        "-OutDir",
        $OutDir,
        "-ForwardJsonBase64",
        $ForwardJsonBase64
    )
    "notify = [ " + (($items | ForEach-Object { Convert-StringToTomlBasicString $_ }) -join ", ") + " ]"
}

function Get-BackupPath {
    param([string]$Path)
    "$Path.cc-traffic-light-notify-probe.bak"
}

$WrapperPath = if ([string]::IsNullOrWhiteSpace($WrapperPath)) {
    Get-DefaultWrapperPath
} else {
    $WrapperPath
}

$backupPath = Get-BackupPath $ConfigPath

if ($Restore) {
    if (-not (Test-Path -LiteralPath $backupPath)) {
        throw "backup not found: $backupPath"
    }
    if ($Apply) {
        Copy-Item -LiteralPath $backupPath -Destination $ConfigPath -Force
        Write-Output "restored=true"
    } else {
        Write-Output "restore_dry_run=true"
    }
    Write-Output "config_path=<redacted>"
    Write-Output "backup_path=<redacted>"
    exit 0
}

$configText = Read-ConfigText $ConfigPath
$notifyLiteral = Get-NotifyArrayLiteral $configText
$notifyItems = Convert-TomlStringArrayToItems $notifyLiteral
$forwardJsonBase64 = Convert-ItemsToJsonBase64 $notifyItems
$probeNotifyLine = New-ProbeNotifyLine -Wrapper $WrapperPath -OutDir $ProbeOutDir -ForwardJsonBase64 $forwardJsonBase64
$updatedText = [regex]::Replace($configText, '(?m)^\s*notify\s*=\s*\[[^\r\n]*\]\s*$', [System.Text.RegularExpressions.MatchEvaluator]{ param($m) $probeNotifyLine }, 1)

Write-Output "config_path=<redacted>"
Write-Output "wrapper_path=<redacted>"
Write-Output "probe_out_dir=<redacted>"
Write-Output "original_notify_count=$($notifyItems.Count)"
Write-Output "probe_notify_count=10"
Write-Output "apply=$($Apply.IsPresent)"

if ($Apply) {
    if (-not (Test-Path -LiteralPath $backupPath)) {
        Copy-Item -LiteralPath $ConfigPath -Destination $backupPath
    }
    Set-Content -LiteralPath $ConfigPath -Value $updatedText -Encoding UTF8
    Write-Output "updated=true"
    Write-Output "backup_path=<redacted>"
} else {
    Write-Output "updated=false"
    Write-Output "dry_run=true"
}
