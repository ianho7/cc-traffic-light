param(
    [string]$SettingsPath = "$env:USERPROFILE\.claude\settings.json",
    [string]$HookExecutablePath = "",
    [switch]$Apply,
    [switch]$Uninstall,
    [switch]$Restore,
    [switch]$ShowPaths,
    [string]$LogPath = ""
)

$ErrorActionPreference = "Stop"

function Write-HookUninstallLog([string]$Message) {
    if ([string]::IsNullOrWhiteSpace($LogPath)) { return }

    $directory = Split-Path -Parent $LogPath
    if (-not (Test-Path -LiteralPath $directory)) {
        New-Item -ItemType Directory -Path $directory -Force | Out-Null
    }
    Add-Content -LiteralPath $LogPath -Value "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss.fff') [claude-hooks] $Message" -Encoding utf8
}

trap {
    Write-HookUninstallLog "error=$($_.Exception.Message)"
    exit 1
}
$ManagedStatusPrefix = "CcTrafficLight Claude"
$BackupSuffix = ".cc-traffic-light-hooks.bak"
$BackupMetaSuffix = ".cc-traffic-light-hooks.bak.meta.json"

if ([string]::IsNullOrWhiteSpace($HookExecutablePath)) {
    $installedHookPath = Join-Path (Split-Path -Parent $PSScriptRoot) "taskbar_widget_hook.exe"
    $HookExecutablePath = if (Test-Path -LiteralPath $installedHookPath -PathType Leaf) {
        $installedHookPath
    } else {
        Join-Path $env:LOCALAPPDATA "Programs\CC Traffic Light\taskbar_widget_hook.exe"
    }
}

function Format-PathForOutput([string]$Path) {
    if ($ShowPaths) { return $Path }
    "<redacted>"
}

function Write-Utf8NoBom([string]$Path, [string]$Content) {
    $parent = Split-Path -Parent $Path
    if (-not (Test-Path -LiteralPath $parent)) {
        New-Item -ItemType Directory -Path $parent -Force | Out-Null
    }
    [System.IO.File]::WriteAllText($Path, $Content, (New-Object System.Text.UTF8Encoding($false)))
}

function Get-PropertyValue($Object, [string]$Name) {
    if ($null -eq $Object) { return $null }
    $property = $Object.PSObject.Properties[$Name]
    if ($null -eq $property) { return $null }
    return $property.Value
}

function Set-PropertyValue($Object, [string]$Name, $Value) {
    $property = $Object.PSObject.Properties[$Name]
    if ($null -eq $property) {
        $Object | Add-Member -NotePropertyName $Name -NotePropertyValue $Value
    } else {
        $property.Value = $Value
    }
}

function Get-BackupPath { "$SettingsPath$BackupSuffix" }
function Get-BackupMetaPath { "$SettingsPath$BackupMetaSuffix" }

function Get-DesiredEventSpecs([string]$ExecutablePath) {
    @(
        [pscustomobject]@{ Event = "UserPromptSubmit"; Matcher = $null },
        [pscustomobject]@{ Event = "PreToolUse"; Matcher = "*" },
        [pscustomobject]@{ Event = "PermissionRequest"; Matcher = "*" },
        [pscustomobject]@{ Event = "PostToolUse"; Matcher = "*" },
        [pscustomobject]@{ Event = "PostToolUseFailure"; Matcher = "*" },
        [pscustomobject]@{ Event = "Stop"; Matcher = $null },
        [pscustomobject]@{ Event = "StopFailure"; Matcher = "*" }
    ) | ForEach-Object {
        $_ | Add-Member -NotePropertyName StatusMessage -NotePropertyValue "$ManagedStatusPrefix $($_.Event)"
        $_ | Add-Member -NotePropertyName Command -NotePropertyValue $ExecutablePath
        $_ | Add-Member -NotePropertyName Args -NotePropertyValue @("claude", $_.Event)
        $_
    }
}

function Test-IsManagedEntry($Entry, $Spec) {
    $handlers = @((Get-PropertyValue $Entry "hooks"))
    if ($handlers.Count -ne 1) { return $false }
    $handler = $handlers[0]
    if ((Get-PropertyValue $handler "type") -ne "command") { return $false }
    if ((Get-PropertyValue $handler "statusMessage") -eq $Spec.StatusMessage) { return $true }
    return ((Get-PropertyValue $handler "command") -eq $Spec.Command -and
        @((Get-PropertyValue $handler "args")) -join "`0" -eq $Spec.Args -join "`0")
}

function New-ManagedEntry($Spec) {
    $handler = [pscustomobject][ordered]@{
        type = "command"
        command = $Spec.Command
        args = $Spec.Args
        statusMessage = $Spec.StatusMessage
    }
    $entry = [pscustomobject][ordered]@{ hooks = @($handler) }
    if (-not [string]::IsNullOrEmpty($Spec.Matcher)) {
        $entry = [pscustomobject][ordered]@{ matcher = $Spec.Matcher; hooks = @($handler) }
    }
    $entry
}

function Read-SettingsConfig {
    if (-not (Test-Path -LiteralPath $SettingsPath)) {
        return [pscustomobject]@{ Exists = $false; RawText = $null; Config = [pscustomobject]@{} }
    }
    $raw = Get-Content -LiteralPath $SettingsPath -Raw
    if ([string]::IsNullOrWhiteSpace($raw)) { throw "settings.json is empty: $SettingsPath" }
    try { $config = $raw | ConvertFrom-Json } catch { throw "failed to parse settings.json: $SettingsPath" }
    return [pscustomobject]@{ Exists = $true; RawText = $raw; Config = $config }
}

function Merge-HooksConfig($Config, [object[]]$Specs) {
    $hooks = Get-PropertyValue $Config "hooks"
    if ($null -eq $hooks) {
        $hooks = [pscustomobject]@{}
        Set-PropertyValue $Config "hooks" $hooks
    }
    $summary = @()
    foreach ($spec in $Specs) {
        $eventEntries = Get-PropertyValue $hooks $spec.Event
        $existing = if ($null -eq $eventEntries) { @() } else { @($eventEntries) }
        $managed = @($existing | Where-Object { Test-IsManagedEntry $_ $spec })
        $other = @($existing | Where-Object { -not (Test-IsManagedEntry $_ $spec) })
        Set-PropertyValue $hooks $spec.Event ($other + @(New-ManagedEntry $spec))
        $summary += [pscustomobject]@{ event = $spec.Event; action = if ($managed.Count -eq 1) { "update" } else { "add" }; other_entries = $other.Count }
    }
    [pscustomobject]@{ Config = $Config; EventSummary = $summary }
}

function Remove-ManagedHooks($Config, [object[]]$Specs) {
    $hooks = Get-PropertyValue $Config "hooks"
    if ($null -eq $hooks) {
        return [pscustomobject]@{ Config = $Config; EventSummary = @(); RemovedCount = 0 }
    }
    $summary = @()
    foreach ($spec in $Specs) {
        $eventEntries = Get-PropertyValue $hooks $spec.Event
        $existing = if ($null -eq $eventEntries) { @() } else { @($eventEntries) }
        $managed = @($existing | Where-Object { Test-IsManagedEntry $_ $spec })
        $other = @($existing | Where-Object { -not (Test-IsManagedEntry $_ $spec) })
        if ($managed.Count -gt 0) { Set-PropertyValue $hooks $spec.Event $other }
        $summary += [pscustomobject]@{ event = $spec.Event; removed = $managed.Count; other_entries = $other.Count }
    }
    [pscustomobject]@{ Config = $Config; EventSummary = $summary; RemovedCount = @($summary | ForEach-Object { $_.removed } | Measure-Object -Sum).Sum }
}

function Write-Backup($OriginalExisted, [string]$RawText) {
    $backup = Get-BackupPath
    if (-not (Test-Path -LiteralPath $backup)) { Write-Utf8NoBom $backup ([string]$RawText) }
    Write-Utf8NoBom (Get-BackupMetaPath) (@{ originalExisted = $OriginalExisted } | ConvertTo-Json)
}

function Restore-Settings {
    $backup = Get-BackupPath
    $meta = Get-BackupMetaPath
    if (-not (Test-Path -LiteralPath $backup) -or -not (Test-Path -LiteralPath $meta)) { throw "Claude hook backup not found" }
    $originalExisted = [bool]((Get-Content -Raw $meta | ConvertFrom-Json).originalExisted)
    if ($Apply) {
        if ($originalExisted) { Write-Utf8NoBom $SettingsPath (Get-Content -Raw $backup) }
        elseif (Test-Path -LiteralPath $SettingsPath) { Remove-Item -LiteralPath $SettingsPath -Force }
    }
    [pscustomobject]@{ mode = if ($Apply) { "restore" } else { "restore-dry-run" }; restored = $Apply.IsPresent; settings_path = Format-PathForOutput $SettingsPath }
}

if ($HookExecutablePath -match '[\\/](target)[\\/](debug|release)[\\/]') { throw "HookExecutablePath must use a stable install location" }
if ($Apply -and -not $Restore -and -not $Uninstall -and -not (Test-Path -LiteralPath $HookExecutablePath -PathType Leaf)) { throw "hook executable not found: $HookExecutablePath" }

if ($Restore) { Restore-Settings | ConvertTo-Json -Depth 12; exit 0 }

$read = Read-SettingsConfig
$specs = Get-DesiredEventSpecs $HookExecutablePath
if ($Uninstall) {
    Write-HookUninstallLog "start settings_path=$(Format-PathForOutput $SettingsPath)"
    $remove = Remove-ManagedHooks $read.Config $specs
    if ($Apply -and $read.Exists -and $remove.RemovedCount -gt 0) {
        Write-Utf8NoBom $SettingsPath ($remove.Config | ConvertTo-Json -Depth 20)
    }
    $uninstallSummary = [pscustomobject]@{
        mode = if ($Apply) { "uninstall" } else { "uninstall-dry-run" }
        settings_path = Format-PathForOutput $SettingsPath
        written = ($Apply.IsPresent -and $read.Exists -and $remove.RemovedCount -gt 0)
        removed_count = $remove.RemovedCount
        event_summary = $remove.EventSummary
    }
    Write-HookUninstallLog "complete removed_count=$($uninstallSummary.removed_count) written=$($uninstallSummary.written)"
    foreach ($event in $remove.EventSummary) {
        Write-HookUninstallLog "event=$($event.event) removed=$($event.removed) other_entries=$($event.other_entries)"
    }
    $uninstallSummary | ConvertTo-Json -Depth 20
    exit 0
}
$merged = Merge-HooksConfig $read.Config $specs
$updatedJson = $merged.Config | ConvertTo-Json -Depth 20
if ($Apply) {
    Write-Backup $read.Exists $read.RawText
    Write-Utf8NoBom $SettingsPath $updatedJson
}

[pscustomobject]@{
    mode = if ($Apply) { "apply" } else { "dry-run" }
    settings_path = Format-PathForOutput $SettingsPath
    hook_executable_path = Format-PathForOutput $HookExecutablePath
    config_existed = $read.Exists
    written = $Apply.IsPresent
    managed_event_count = @($merged.EventSummary).Count
    managed_status_prefix = $ManagedStatusPrefix
    event_summary = $merged.EventSummary
} | ConvertTo-Json -Depth 20
