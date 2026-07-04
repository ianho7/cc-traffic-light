$ErrorActionPreference = "Stop"

$packageRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$frontendTypesPath = Join-Path $packageRoot "src\types.ts"
$frontendAppPath = Join-Path $packageRoot "src\App.tsx"
$frontendApiPath = Join-Path $packageRoot "src\lib\tauri.ts"
$backendPath = Join-Path $packageRoot "src-tauri\src\lib.rs"
$tauriCargoPath = Join-Path $packageRoot "src-tauri\Cargo.toml"
$hostIpcPath = Join-Path $packageRoot "..\taskbar-widget\src\tauri_settings_ipc.rs"
$hostMainPath = Join-Path $packageRoot "..\taskbar-widget\src\main.rs"
$hostCargoPath = Join-Path $packageRoot "..\taskbar-widget\Cargo.toml"
$hostBuildPath = Join-Path $packageRoot "..\taskbar-widget\build.rs"
$reportDir = Join-Path $packageRoot "target\verify-settings-coverage"
$reportPath = Join-Path $reportDir "report.json"

if (-not (Test-Path $reportDir)) {
    New-Item -ItemType Directory -Path $reportDir -Force | Out-Null
}

function Get-FileText {
    param([string]$Path)
    Get-Content -Raw $Path
}

function Test-ContainsAll {
    param(
        [string]$Text,
        [string[]]$Tokens
    )

    foreach ($token in $Tokens) {
        if ($Text -notmatch [regex]::Escape($token)) {
            return $false
        }
    }

    return $true
}

$pageIds = @(
    "overview",
    "general",
    "monitoring",
    "appearance",
    "diagnostics",
    "about"
)

$configBindings = @(
    "draft.general.autostart_enabled",
    "draft.general.start_minimized_to_tray",
    "draft.general.close_to_tray",
    "draft.localization.language",
    "draft.monitoring.codex_enabled",
    "draft.monitoring.claude_enabled",
    "draft.appearance.ui_theme",
    "draft.appearance.indicator_style",
    "draft.appearance.widget_size",
    "draft.appearance.show_labels",
    "draft.appearance.reduced_motion",
    "draft.diagnostics.last_opened_page"
)

$snapshotBindings = @(
    "snapshot.overall_state",
    "snapshot.widget_mount_state",
    "snapshot.last_widget_attach_at",
    "snapshot.last_detection_refresh_at",
    "snapshot.last_error_summary",
    "source.method",
    "source.confidence",
    "source.updated_at",
    "source.message"
)

$typesText = Get-FileText $frontendTypesPath
$appText = Get-FileText $frontendAppPath
$apiText = Get-FileText $frontendApiPath
$backendText = Get-FileText $backendPath
$tauriCargoText = Get-FileText $tauriCargoPath
$hostIpcText = Get-FileText $hostIpcPath
$hostMainText = Get-FileText $hostMainPath
$hostCargoText = Get-FileText $hostCargoPath
$hostBuildText = Get-FileText $hostBuildPath

$checks = [ordered]@{}

$checks.settings_page_type_complete = Test-ContainsAll -Text $typesText -Tokens ($pageIds | ForEach-Object { "`"$_`"" })
$checks.page_shell_declares_all_pages = Test-ContainsAll -Text $appText -Tokens ($pageIds | ForEach-Object { "$($_):" })
$checks.page_shell_renders_all_pages = Test-ContainsAll -Text $appText -Tokens ($pageIds | ForEach-Object { "page === `"$($_)`"" })
$checks.frontend_reads_live_bootstrap_and_polling = Test-ContainsAll -Text $appText -Tokens @(
    "bootstrapWindow",
    "getSnapshot",
    "getSettings",
    "saveSettings",
    "requestRefresh",
    "notifySettingsApplied",
    "Promise.all([getSnapshot(), getSettings()])"
)
$checks.frontend_binds_all_config_fields = Test-ContainsAll -Text $appText -Tokens $configBindings
$checks.frontend_binds_snapshot_fields = Test-ContainsAll -Text $appText -Tokens $snapshotBindings
$checks.frontend_about_page_uses_runtime_metadata = Test-ContainsAll -Text $appText -Tokens @(
    "about.product_name",
    "about.version",
    "about.runtime_description",
    "about.config_path",
    "bootstrap.transport.kind",
    "bootstrap.transport.endpoint"
)
$checks.tauri_api_exposes_all_commands = Test-ContainsAll -Text $apiText -Tokens @(
    "invoke(`"bootstrap_window`")",
    "invoke(`"get_snapshot`")",
    "invoke(`"get_settings`")",
    "invoke(`"save_settings`", { settings })",
    "invoke(`"request_refresh`")",
    "invoke(`"notify_settings_applied`", { appliedKeys })"
)
$checks.tauri_build_defaults_to_embedded_assets = Test-ContainsAll -Text $tauriCargoText -Tokens @(
    'default = ["custom-protocol"]',
    'custom-protocol = ["tauri/custom-protocol"]'
)
$checks.tauri_backend_uses_named_pipe_for_live_read_write = Test-ContainsAll -Text $backendText -Tokens @(
    "call_pipe(SettingsIpcCommand::GetSnapshot)",
    "call_pipe(SettingsIpcCommand::GetSettings)",
    "call_pipe(SettingsIpcCommand::SaveSettings",
    "call_pipe(SettingsIpcCommand::RequestRefresh)",
    "call_pipe(SettingsIpcCommand::NotifySettingsApplied",
    "fake_mode: false"
)
$checks.host_ipc_serves_live_snapshot_and_settings = Test-ContainsAll -Text $hostIpcText -Tokens @(
    "SettingsIpcCommand::GetSnapshot => SettingsIpcResponse::GetSnapshot",
    "StatusSnapshotView::from(settings_bridge::current_snapshot())",
    "SettingsIpcCommand::GetSettings => SettingsIpcResponse::GetSettings",
    "settings: settings_bridge::current_config()",
    "SettingsIpcCommand::SaveSettings { settings }",
    "settings_bridge::apply_full_settings(settings)"
)
$checks.host_no_longer_references_slint_runtime = ($hostMainText -notmatch "settings_slint|slint::")
$checks.host_build_graph_no_longer_compiles_slint = ($hostCargoText -notmatch "slint|raw-window-handle") -and ($hostBuildText -notmatch "slint_build::compile|settings.slint")

$failed = @(
    $checks.GetEnumerator() |
        Where-Object { -not $_.Value } |
        Select-Object -ExpandProperty Key
)

$result = [ordered]@{
    timestamp = (Get-Date).ToString("s")
    passed = ($failed.Count -eq 0)
    failed_checks = $failed
    pages = $pageIds
    config_bindings = $configBindings
    snapshot_bindings = $snapshotBindings
}

$result | ConvertTo-Json -Depth 4 | Set-Content -Path $reportPath -Encoding UTF8
$result | ConvertTo-Json -Depth 4

if ($failed.Count -gt 0) {
    exit 1
}
