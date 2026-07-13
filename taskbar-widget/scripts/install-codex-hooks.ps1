param(
    [string]$HooksPath = "$env:USERPROFILE\.codex\hooks.json",
    [string]$HookExecutablePath = "",
    [string]$WrapperPath = "",
    [switch]$Apply,
    [switch]$Restore,
    [switch]$ShowPaths
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($HookExecutablePath)) {
    $installedHookPath = Join-Path (Split-Path -Parent $PSScriptRoot) "taskbar_widget_hook.exe"
    if (Test-Path -LiteralPath $installedHookPath -PathType Leaf) {
        $HookExecutablePath = $installedHookPath
    } else {
        $HookExecutablePath = Join-Path $env:LOCALAPPDATA "Programs\CC Traffic Light\taskbar_widget_hook.exe"
    }
}

if ([string]::IsNullOrWhiteSpace($WrapperPath)) {
    $WrapperPath = Join-Path $env:LOCALAPPDATA "CcTrafficLight\codex-taskbar-widget-hook.cmd"
}

$ManagedStatusPrefix = "CcTrafficLight Codex"
$BackupSuffix = ".cc-traffic-light-global-hooks.bak"
$BackupMetaSuffix = ".cc-traffic-light-global-hooks.bak.meta.json"

function Format-PathForOutput {
    param([string]$Path)

    if ($ShowPaths) {
        return $Path
    }
    "<redacted>"
}

function Get-BackupPath {
    param([string]$Path)

    "$Path$BackupSuffix"
}

function Get-BackupMetaPath {
    param([string]$Path)

    "$Path$BackupMetaSuffix"
}

function ConvertTo-PrettyJson {
    param($Value)

    $Value | ConvertTo-Json -Depth 20
}

function Write-Utf8NoBom {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path,
        [Parameter(Mandatory = $true)]
        [AllowEmptyString()]
        [string]$Content
    )

    $encoding = New-Object System.Text.UTF8Encoding($false)
    [System.IO.File]::WriteAllText($Path, $Content, $encoding)
}

function Get-PropertyValue {
    param(
        $Object,
        [string]$Name
    )

    if ($null -eq $Object) {
        return $null
    }

    $property = $Object.PSObject.Properties[$Name]
    if ($null -eq $property) {
        return $null
    }
    $property.Value
}

function Set-PropertyValue {
    param(
        $Object,
        [string]$Name,
        $Value
    )

    $property = $Object.PSObject.Properties[$Name]
    if ($null -eq $property) {
        $Object | Add-Member -NotePropertyName $Name -NotePropertyValue $Value
    } else {
        $property.Value = $Value
    }
}

function Test-CommandShape {
    param(
        $Entry,
        [string]$ExpectedMatcher,
        [string]$ExpectedCommand,
        [string]$ExpectedStatusMessage
    )

    $hooks = @((Get-PropertyValue $Entry "hooks"))
    if ($null -eq $hooks -or $hooks.Count -ne 1) {
        return $false
    }

    $hook = $hooks[0]
    $type = Get-PropertyValue $hook "type"
    if ($type -ne "command") {
        return $false
    }

    $entryMatcher = Get-PropertyValue $Entry "matcher"
    if ([string]::IsNullOrEmpty($ExpectedMatcher)) {
        if (-not [string]::IsNullOrEmpty($entryMatcher)) {
            return $false
        }
    } elseif ($entryMatcher -ne $ExpectedMatcher) {
        return $false
    }

    $command = Get-PropertyValue $hook "command"
    $commandWindows = Get-PropertyValue $hook "commandWindows"
    $statusMessage = Get-PropertyValue $hook "statusMessage"

    if ($statusMessage -eq $ExpectedStatusMessage) {
        return $true
    }

    $command -eq $ExpectedCommand -and $commandWindows -eq $ExpectedCommand
}

function Test-IsManagedEntry {
    param(
        $Entry,
        $Spec
    )

    Test-CommandShape `
        -Entry $Entry `
        -ExpectedMatcher $Spec.Matcher `
        -ExpectedCommand $Spec.Command `
        -ExpectedStatusMessage $Spec.StatusMessage
}

function New-CommandString {
    param(
        [string]$ExecutablePath,
        [string]$HookName
    )

    '"' + $ExecutablePath + '" codex ' + $HookName
}

function New-WindowsCommandString {
    param(
        [string]$WrapperPath,
        [string]$HookName
    )

    # `call` preserves `%*` forwarding from the wrapper and keeps the actual
    # hook EXE out of Codex's direct Windows process launch path.
    'cmd.exe /d /s /c call "' + $WrapperPath + '" codex ' + $HookName
}

function Write-WindowsHookWrapper {
    param(
        [string]$Path,
        [string]$ExecutablePath
    )

    $directory = Split-Path -Parent $Path
    if (-not (Test-Path -LiteralPath $directory)) {
        New-Item -ItemType Directory -Path $directory -Force | Out-Null
    }

    $content = "@echo off`r`n`"$ExecutablePath`" %*`r`nexit /b %ERRORLEVEL%`r`n"
    Write-Utf8NoBom -Path $Path -Content $content
}

function New-ManagedEntry {
    param($Spec)

    $hook = [pscustomobject][ordered]@{
        type = "command"
        command = $Spec.Command
        commandWindows = $Spec.CommandWindows
        statusMessage = $Spec.StatusMessage
    }

    $entry = [pscustomobject][ordered]@{
        hooks = @($hook)
    }

    if (-not [string]::IsNullOrEmpty($Spec.Matcher)) {
        $entry = [pscustomobject][ordered]@{
            matcher = $Spec.Matcher
            hooks = @($hook)
        }
    }

    $entry
}

function Get-DesiredEventSpecs {
    param(
        [string]$ExecutablePath,
        [string]$WrapperPath
    )

    @(
        [pscustomobject][ordered]@{
            Event = "SessionStart"
            Matcher = "startup|resume|clear|compact"
            StatusMessage = "$ManagedStatusPrefix SessionStart"
            Command = New-CommandString -ExecutablePath $ExecutablePath -HookName "SessionStart"
            CommandWindows = New-WindowsCommandString -WrapperPath $WrapperPath -HookName "SessionStart"
        }
        [pscustomobject][ordered]@{
            Event = "UserPromptSubmit"
            Matcher = $null
            StatusMessage = "$ManagedStatusPrefix UserPromptSubmit"
            Command = New-CommandString -ExecutablePath $ExecutablePath -HookName "UserPromptSubmit"
            CommandWindows = New-WindowsCommandString -WrapperPath $WrapperPath -HookName "UserPromptSubmit"
        }
        [pscustomobject][ordered]@{
            Event = "PreToolUse"
            Matcher = "*"
            StatusMessage = "$ManagedStatusPrefix PreToolUse"
            Command = New-CommandString -ExecutablePath $ExecutablePath -HookName "PreToolUse"
            CommandWindows = New-WindowsCommandString -WrapperPath $WrapperPath -HookName "PreToolUse"
        }
        [pscustomobject][ordered]@{
            Event = "PermissionRequest"
            Matcher = "*"
            StatusMessage = "$ManagedStatusPrefix PermissionRequest"
            Command = New-CommandString -ExecutablePath $ExecutablePath -HookName "PermissionRequest"
            CommandWindows = New-WindowsCommandString -WrapperPath $WrapperPath -HookName "PermissionRequest"
        }
        [pscustomobject][ordered]@{
            Event = "PostToolUse"
            Matcher = "*"
            StatusMessage = "$ManagedStatusPrefix PostToolUse"
            Command = New-CommandString -ExecutablePath $ExecutablePath -HookName "PostToolUse"
            CommandWindows = New-WindowsCommandString -WrapperPath $WrapperPath -HookName "PostToolUse"
        }
        [pscustomobject][ordered]@{
            Event = "SubagentStop"
            Matcher = "*"
            StatusMessage = "$ManagedStatusPrefix SubagentStop"
            Command = New-CommandString -ExecutablePath $ExecutablePath -HookName "SubagentStop"
            CommandWindows = New-WindowsCommandString -WrapperPath $WrapperPath -HookName "SubagentStop"
        }
        [pscustomobject][ordered]@{
            Event = "Stop"
            Matcher = $null
            StatusMessage = "$ManagedStatusPrefix Stop"
            Command = New-CommandString -ExecutablePath $ExecutablePath -HookName "Stop"
            CommandWindows = New-WindowsCommandString -WrapperPath $WrapperPath -HookName "Stop"
        }
    )
}

function New-EmptyHooksConfig {
    [pscustomobject][ordered]@{
        hooks = [pscustomobject][ordered]@{}
    }
}

function Read-HooksConfig {
    param([string]$Path)

    if (-not (Test-Path -LiteralPath $Path)) {
        return [pscustomobject][ordered]@{
            Exists = $false
            Config = New-EmptyHooksConfig
            RawText = $null
        }
    }

    $rawText = Get-Content -LiteralPath $Path -Raw
    if ([string]::IsNullOrWhiteSpace($rawText)) {
        throw "hooks.json is empty: $Path"
    }

    try {
        $config = $rawText | ConvertFrom-Json
    } catch {
        throw "failed to parse hooks.json: $Path"
    }

    if ($null -eq $config -or $null -eq (Get-PropertyValue $config "hooks")) {
        throw "hooks.json must contain a top-level hooks object"
    }

    [pscustomobject][ordered]@{
        Exists = $true
        Config = $config
        RawText = $rawText
    }
}

function Get-EventEntries {
    param(
        $HooksConfig,
        [string]$EventName
    )

    $hooks = Get-PropertyValue $HooksConfig "hooks"
    $entries = Get-PropertyValue $hooks $EventName
    if ($null -eq $entries) {
        return @()
    }
    @($entries)
}

function Set-EventEntries {
    param(
        $HooksConfig,
        [string]$EventName,
        [object[]]$Entries
    )

    $hooks = Get-PropertyValue $HooksConfig "hooks"
    Set-PropertyValue -Object $hooks -Name $EventName -Value $Entries
}

function Merge-HooksConfig {
    param(
        $HooksConfig,
        [object[]]$Specs
    )

    $eventSummaries = @()
    foreach ($spec in $Specs) {
        $existingEntries = Get-EventEntries -HooksConfig $HooksConfig -EventName $spec.Event
        $managedEntries = @($existingEntries | Where-Object { Test-IsManagedEntry -Entry $_ -Spec $spec })
        $otherEntries = @($existingEntries | Where-Object { -not (Test-IsManagedEntry -Entry $_ -Spec $spec) })
        $desiredEntry = New-ManagedEntry -Spec $spec

        $action = if ($managedEntries.Count -eq 0) { "add" } else { "update" }
        if ($managedEntries.Count -eq 1) {
            $currentJson = ConvertTo-PrettyJson $managedEntries[0]
            $desiredJson = ConvertTo-PrettyJson $desiredEntry
            if ($currentJson -eq $desiredJson) {
                $action = "unchanged"
            }
        }

        Set-EventEntries -HooksConfig $HooksConfig -EventName $spec.Event -Entries ($otherEntries + @($desiredEntry))

        $eventSummaries += [pscustomobject][ordered]@{
            event = $spec.Event
            action = $action
            matcher = if ([string]::IsNullOrEmpty($spec.Matcher)) { "<none>" } else { $spec.Matcher }
            managed_before = $managedEntries.Count
            other_entries = $otherEntries.Count
        }
    }

    [pscustomobject][ordered]@{
        Config = $HooksConfig
        EventSummaries = $eventSummaries
        ChangesRequired = @($eventSummaries | Where-Object { $_.action -ne "unchanged" }).Count -gt 0
    }
}

function Assert-StableHookExecutablePath {
    param([string]$Path)

    if ([string]::IsNullOrWhiteSpace($Path)) {
        throw "HookExecutablePath must not be empty"
    }

    if ($Path -match '[\\/](target)[\\/](debug|release)[\\/]') {
        throw "HookExecutablePath must point to a stable install location, not a cargo target directory"
    }
}

function Write-HooksConfigAtomically {
    param(
        [string]$Path,
        [string]$Content
    )

    $directory = Split-Path -Parent $Path
    if (-not (Test-Path -LiteralPath $directory)) {
        New-Item -ItemType Directory -Path $directory | Out-Null
    }

    $tempPath = "$Path.cc-traffic-light.tmp"
    Write-Utf8NoBom -Path $tempPath -Content $Content

    if (Test-Path -LiteralPath $Path) {
        Remove-Item -LiteralPath $Path -Force
    }

    Move-Item -LiteralPath $tempPath -Destination $Path
}

function Write-BackupState {
    param(
        [string]$HooksPath,
        [bool]$OriginalExisted,
        [string]$RawText
    )

    $backupPath = Get-BackupPath -Path $HooksPath
    $backupMetaPath = Get-BackupMetaPath -Path $HooksPath
    $backupDirectory = Split-Path -Parent $backupPath

    if (-not (Test-Path -LiteralPath $backupDirectory)) {
        New-Item -ItemType Directory -Path $backupDirectory | Out-Null
    }

    if (-not (Test-Path -LiteralPath $backupPath)) {
        if ($OriginalExisted) {
            Write-Utf8NoBom -Path $backupPath -Content $RawText
        } else {
            Write-Utf8NoBom -Path $backupPath -Content ""
        }
    }

    $meta = [pscustomobject][ordered]@{
        originalExisted = $OriginalExisted
    }
    Write-Utf8NoBom -Path $backupMetaPath -Content (ConvertTo-PrettyJson $meta)

    [pscustomobject][ordered]@{
        BackupPath = $backupPath
        BackupMetaPath = $backupMetaPath
    }
}

function Read-BackupMeta {
    param([string]$HooksPath)

    $backupPath = Get-BackupPath -Path $HooksPath
    $backupMetaPath = Get-BackupMetaPath -Path $HooksPath

    if (-not (Test-Path -LiteralPath $backupPath)) {
        throw "backup not found: $backupPath"
    }
    if (-not (Test-Path -LiteralPath $backupMetaPath)) {
        throw "backup metadata not found: $backupMetaPath"
    }

    $meta = Get-Content -LiteralPath $backupMetaPath -Raw | ConvertFrom-Json
    [pscustomobject][ordered]@{
        BackupPath = $backupPath
        BackupMetaPath = $backupMetaPath
        OriginalExisted = [bool]$meta.originalExisted
    }
}

function Restore-HooksConfig {
    param([string]$HooksPath)

    $backupState = Read-BackupMeta -HooksPath $HooksPath
    if (-not $Apply) {
        return [pscustomobject][ordered]@{
            mode = "restore-dry-run"
            hooks_path = Format-PathForOutput $HooksPath
            backup_path = Format-PathForOutput $backupState.BackupPath
            backup_meta_path = Format-PathForOutput $backupState.BackupMetaPath
            original_existed = $backupState.OriginalExisted
            restored = $false
        }
    }

    if ($backupState.OriginalExisted) {
        $content = Get-Content -LiteralPath $backupState.BackupPath -Raw
        Write-HooksConfigAtomically -Path $HooksPath -Content $content
    } elseif (Test-Path -LiteralPath $HooksPath) {
        Remove-Item -LiteralPath $HooksPath -Force
    }

    [pscustomobject][ordered]@{
        mode = "restore"
        hooks_path = Format-PathForOutput $HooksPath
        backup_path = Format-PathForOutput $backupState.BackupPath
        backup_meta_path = Format-PathForOutput $backupState.BackupMetaPath
        original_existed = $backupState.OriginalExisted
        restored = $true
    }
}

Assert-StableHookExecutablePath -Path $HookExecutablePath
$hookExecutableExists = Test-Path -LiteralPath $HookExecutablePath -PathType Leaf

if ($Apply -and -not $Restore -and -not $hookExecutableExists) {
    throw "HookExecutablePath does not point to an existing file: $HookExecutablePath"
}

if ($Apply -and -not $Restore) {
    Write-WindowsHookWrapper -Path $WrapperPath -ExecutablePath $HookExecutablePath
}

if ($Apply -and -not $Restore -and (Test-Path -LiteralPath $HooksPath -PathType Leaf)) {
    $hooksAttributes = (Get-Item -LiteralPath $HooksPath).Attributes
    if (($hooksAttributes -band [System.IO.FileAttributes]::ReadOnly) -ne 0) {
        throw "hooks.json is read-only and was not modified: $HooksPath"
    }
}

if ($Restore) {
    ConvertTo-PrettyJson (Restore-HooksConfig -HooksPath $HooksPath)
    exit 0
}

$readResult = Read-HooksConfig -Path $HooksPath
$specs = Get-DesiredEventSpecs -ExecutablePath $HookExecutablePath -WrapperPath $WrapperPath
$mergeResult = Merge-HooksConfig -HooksConfig $readResult.Config -Specs $specs
$updatedJson = ConvertTo-PrettyJson $mergeResult.Config
$backupState = [pscustomobject][ordered]@{
    BackupPath = Get-BackupPath -Path $HooksPath
    BackupMetaPath = Get-BackupMetaPath -Path $HooksPath
}

if ($Apply) {
    $backupState = Write-BackupState -HooksPath $HooksPath -OriginalExisted $readResult.Exists -RawText $readResult.RawText
    Write-HooksConfigAtomically -Path $HooksPath -Content $updatedJson
}

$summary = [pscustomobject][ordered]@{
    mode = if ($Apply) { "apply" } else { "dry-run" }
    hooks_path = Format-PathForOutput $HooksPath
    hook_executable_path = Format-PathForOutput $HookExecutablePath
    hook_wrapper_path = Format-PathForOutput $WrapperPath
    hook_executable_exists = $hookExecutableExists
    backup_path = Format-PathForOutput $backupState.BackupPath
    backup_meta_path = Format-PathForOutput $backupState.BackupMetaPath
    config_existed = $readResult.Exists
    apply = $Apply.IsPresent
    changes_required = $mergeResult.ChangesRequired
    managed_event_count = $specs.Count
    managed_status_prefix = $ManagedStatusPrefix
    event_summary = $mergeResult.EventSummaries
}

if ($Apply) {
    $summary | Add-Member -NotePropertyName "written" -NotePropertyValue $true
} else {
    $summary | Add-Member -NotePropertyName "written" -NotePropertyValue $false
}

ConvertTo-PrettyJson $summary
