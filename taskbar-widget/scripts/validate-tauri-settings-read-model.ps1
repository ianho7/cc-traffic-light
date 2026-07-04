param(
    [switch]$SkipBuild,
    [ValidateSet("debug", "release")]
    [string]$Configuration = "debug",
    [int]$TimeoutSeconds = 20,
    [string]$HostExePath
)

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$taskbarWidgetDir = Resolve-Path (Join-Path $scriptDir "..")
$repoRoot = Resolve-Path (Join-Path $taskbarWidgetDir "..")
$candidateTargetDirs = @(
    (Join-Path $repoRoot "target\$Configuration"),
    (Join-Path $taskbarWidgetDir "target\$Configuration")
)
$reportDir = Join-Path $taskbarWidgetDir "target\validate-tauri-settings-read-model"
$reportPath = Join-Path $reportDir "report.json"
$hostClassName = "TaskbarWidgetWindow"
$wmClose = 0x0010
$protocolVersion = "cc_traffic_light.settings.v1"
$pipeName = "\\.\pipe\cc-traffic-light-settings-v1"

if (-not (Test-Path $reportDir)) {
    New-Item -ItemType Directory -Path $reportDir -Force | Out-Null
}

Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
using System.Text;

public static class ReadModelUser32 {
    private delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);

    [DllImport("user32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    public static extern IntPtr FindWindowW(string lpClassName, string lpWindowName);

    [DllImport("user32.dll", SetLastError = true)]
    public static extern bool PostMessageW(IntPtr hWnd, uint Msg, UIntPtr wParam, IntPtr lParam);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern bool EnumWindows(EnumWindowsProc lpEnumFunc, IntPtr lParam);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern bool EnumChildWindows(IntPtr hWndParent, EnumWindowsProc lpEnumFunc, IntPtr lParam);

    [DllImport("user32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    private static extern int GetClassNameW(IntPtr hWnd, StringBuilder lpClassName, int nMaxCount);

    public static IntPtr FindWindowRecursiveByClass(string className) {
        var found = IntPtr.Zero;
        EnumWindows((hWnd, _) => {
            if (HasClassName(hWnd, className)) {
                found = hWnd;
                return false;
            }

            EnumChildWindows(hWnd, (child, __) => {
                if (HasClassName(child, className)) {
                    found = child;
                    return false;
                }
                return true;
            }, IntPtr.Zero);

            return found == IntPtr.Zero;
        }, IntPtr.Zero);

        return found;
    }

    private static bool HasClassName(IntPtr hWnd, string expectedClassName) {
        var buffer = new StringBuilder(256);
        var length = GetClassNameW(hWnd, buffer, buffer.Capacity);
        if (length <= 0) {
            return false;
        }
        return string.Equals(buffer.ToString(), expectedClassName, StringComparison.Ordinal);
    }
}

public static class ReadModelPipeNative {
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    public static extern bool WaitNamedPipeW(string name, uint timeoutMs);

    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    public static extern bool CallNamedPipeW(
        string lpNamedPipeName,
        byte[] lpInBuffer,
        uint nInBufferSize,
        byte[] lpOutBuffer,
        uint nOutBufferSize,
        out uint lpBytesRead,
        uint nTimeOut
    );
}
"@

function Resolve-ExecutableCandidate {
    param(
        [string[]]$TargetDirs,
        [string]$ExecutableName
    )

    foreach ($targetDir in $TargetDirs) {
        $candidate = Join-Path $targetDir $ExecutableName
        if (Test-Path $candidate) {
            return [System.IO.Path]::GetFullPath($candidate)
        }
    }

    return $null
}

function Wait-Until {
    param(
        [scriptblock]$Condition,
        [string]$Description,
        [int]$TimeoutSeconds = 20,
        [int]$PollMs = 200
    )

    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    while ((Get-Date) -lt $deadline) {
        if (& $Condition) {
            return
        }
        Start-Sleep -Milliseconds $PollMs
    }

    throw "Timed out waiting for: $Description"
}

function Find-HostWindow {
    [ReadModelUser32]::FindWindowRecursiveByClass($hostClassName)
}

function Send-HostClose {
    $hwnd = Find-HostWindow
    if ($hwnd -eq [IntPtr]::Zero) {
        throw "Host window not found for class $hostClassName"
    }

    $ok = [ReadModelUser32]::PostMessageW($hwnd, [uint32]$wmClose, [System.UIntPtr]::Zero, [IntPtr]::Zero)
    if (-not $ok) {
        throw "PostMessageW(WM_CLOSE) failed for host"
    }
}

function Invoke-PipeCommand {
    param(
        [string]$RequestId,
        [Object]$Command
    )

    $envelope = [ordered]@{
        protocol_version = $protocolVersion
        request_id = $RequestId
        command = $Command
    }

    $payload = [System.Text.Encoding]::UTF8.GetBytes(($envelope | ConvertTo-Json -Compress -Depth 8))
    $null = [ReadModelPipeNative]::WaitNamedPipeW($pipeName, 150)
    $responseBuffer = New-Object byte[] (64 * 1024)
    $bytesRead = 0
    $ok = [ReadModelPipeNative]::CallNamedPipeW(
        $pipeName,
        $payload,
        [uint32]$payload.Length,
        $responseBuffer,
        [uint32]$responseBuffer.Length,
        [ref]$bytesRead,
        500
    )

    if (-not $ok) {
        throw "CallNamedPipeW failed for request $RequestId"
    }

    $json = [System.Text.Encoding]::UTF8.GetString($responseBuffer, 0, $bytesRead)
    $response = $json | ConvertFrom-Json
    if ($response.protocol_version -ne $protocolVersion) {
        throw "Protocol mismatch for $RequestId"
    }
    if ($response.request_id -ne $RequestId) {
        throw "Request id mismatch for $RequestId"
    }

    return $response.response
}

if (-not $SkipBuild) {
    Push-Location $repoRoot
    try {
        $cargoArgs = @("build", "--offline")
        if ($Configuration -eq "release") {
            $cargoArgs += "--release"
        }

        cargo @cargoArgs -p taskbar-widget
    }
    finally {
        Pop-Location
    }
}

$hostExe = if ($HostExePath) {
    [System.IO.Path]::GetFullPath($HostExePath)
} else {
    Resolve-ExecutableCandidate -TargetDirs $candidateTargetDirs -ExecutableName "taskbar-widget.exe"
}
if (-not $hostExe -or -not (Test-Path $hostExe)) {
    throw "Host executable not found in candidate target directories: $($candidateTargetDirs -join ', ')"
}

$workspaceTargetDir = Split-Path -Parent $hostExe
$configPath = Join-Path $env:APPDATA "CcTrafficLight\config.json"
$startedHost = $null
$oldHostSetting = $env:CC_TRAFFIC_LIGHT_SETTINGS_HOST

$report = [ordered]@{
    timestamp = (Get-Date).ToString("s")
    configuration = $Configuration
    host_exe = $hostExe
    config_path = $configPath
    state_path = (Join-Path $env:APPDATA "CcTrafficLight\state.json")
    snapshot_read = $false
    settings_read = $false
    snapshot_matches_state_file = $false
    codex_snapshot_matches_state = $false
    claude_snapshot_matches_state = $false
    refresh_updated_timestamp = $false
    save_roundtrip_persisted = $false
    save_roundtrip_restored = $false
    host_exit_code = $null
    original_last_opened_page = $null
    alternate_last_opened_page = $null
    original_last_manual_refresh_at = $null
    updated_last_manual_refresh_at = $null
    expected_sources = $null
    actual_sources = $null
}

function Get-StateAgentSummary {
    param(
        [object]$State,
        [string]$SourceId
    )

    if (-not $State -or -not $State.agents) {
        return $null
    }

    $agentProperty = $State.agents.PSObject.Properties[$SourceId]
    if (-not $agentProperty -or -not $agentProperty.Value) {
        return $null
    }

    return $agentProperty.Value.summary
}

function Get-ExpectedSourceSnapshot {
    param(
        [object]$Summary,
        [string]$SourceId
    )

    if (-not $Summary) {
        return $null
    }

    $state = switch ($Summary.state) {
        "idle" { "idle" }
        "working" { "working" }
        "done" { "attention" }
        "waiting" { "blocking" }
        "error" { "blocking" }
        default { "undiscovered" }
    }

    $confidence = if ($Summary.has_stale) {
        "untrusted"
    } elseif ($Summary.state -eq "idle" -and [int]$Summary.active_task_count -eq 0) {
        "degraded"
    } else {
        "confirmed"
    }

    return [ordered]@{
        source_id = $SourceId
        state = $state
        confidence = $confidence
        method = "state_file"
        min_updated_at = [uint64]$Summary.updated_at
    }
}

function Test-SourceSnapshotMatch {
    param(
        [object]$Actual,
        [object]$Expected
    )

    if (-not $Actual -or -not $Expected) {
        return $false
    }

    return $Actual.state -eq $Expected.state `
        -and $Actual.confidence -eq $Expected.confidence `
        -and $Actual.method -eq $Expected.method `
        -and [uint64]$Actual.updated_at -ge [uint64]$Expected.min_updated_at
}

try {
    $env:CC_TRAFFIC_LIGHT_SETTINGS_HOST = "tauri"
    $startedHost = Start-Process -FilePath $hostExe -WorkingDirectory $workspaceTargetDir -PassThru -WindowStyle Hidden

    Wait-Until -Description "host window to appear" -TimeoutSeconds $TimeoutSeconds -Condition {
        if ($startedHost.HasExited) {
            throw "Host exited before window appeared with code $($startedHost.ExitCode)"
        }

        (Find-HostWindow) -ne [IntPtr]::Zero
    }

    if (-not (Test-Path $report.state_path)) {
        throw "State file not found: $($report.state_path)"
    }

    $stateFile = Get-Content -Raw $report.state_path | ConvertFrom-Json
    $configFile = Get-Content -Raw $configPath | ConvertFrom-Json
    $expectedSources = [ordered]@{
        codex = Get-ExpectedSourceSnapshot -Summary (Get-StateAgentSummary -State $stateFile -SourceId "codex") -SourceId "codex"
        claude = Get-ExpectedSourceSnapshot -Summary (Get-StateAgentSummary -State $stateFile -SourceId "claude") -SourceId "claude"
    }
    $report.expected_sources = $expectedSources

    $snapshotResponse = Invoke-PipeCommand -RequestId "get-snapshot" -Command "get_snapshot"
    $settingsResponse = Invoke-PipeCommand -RequestId "get-settings" -Command "get_settings"

    if ($snapshotResponse.get_snapshot.snapshot.overall_state -and $snapshotResponse.get_snapshot.snapshot.sources.codex) {
        $report.snapshot_read = $true
    }
    if ($settingsResponse.get_settings.settings.general -and $settingsResponse.get_settings.settings.appearance) {
        $report.settings_read = $true
    }

    Wait-Until -Description "snapshot to reflect state file" -TimeoutSeconds $TimeoutSeconds -Condition {
        $latestSnapshot = Invoke-PipeCommand -RequestId "get-snapshot-match" -Command "get_snapshot"
        $actualSources = $latestSnapshot.get_snapshot.snapshot.sources
        $script:latestActualSources = $actualSources

        $codexEnabled = [bool]$configFile.monitoring.codex_enabled
        $claudeEnabled = [bool]$configFile.monitoring.claude_enabled
        $script:codexMatches = (-not $codexEnabled) -or (Test-SourceSnapshotMatch -Actual $actualSources.codex -Expected $expectedSources.codex)
        $script:claudeMatches = (-not $claudeEnabled) -or (Test-SourceSnapshotMatch -Actual $actualSources.claude -Expected $expectedSources.claude)

        $script:codexMatches -and $script:claudeMatches
    }

    $report.actual_sources = $script:latestActualSources
    $report.codex_snapshot_matches_state = $script:codexMatches
    $report.claude_snapshot_matches_state = $script:claudeMatches
    $report.snapshot_matches_state_file = $report.codex_snapshot_matches_state -and $report.claude_snapshot_matches_state

    $originalSettings = $settingsResponse.get_settings.settings
    $report.original_last_opened_page = $originalSettings.diagnostics.last_opened_page
    $report.original_last_manual_refresh_at = $originalSettings.diagnostics.last_manual_refresh_at
    $alternatePage = if ($originalSettings.diagnostics.last_opened_page -eq "about") { "overview" } else { "about" }
    $report.alternate_last_opened_page = $alternatePage

    $mutatedSettings = $originalSettings | ConvertTo-Json -Depth 20 | ConvertFrom-Json
    $mutatedSettings.diagnostics.last_opened_page = $alternatePage

    $saveResponse = Invoke-PipeCommand -RequestId "save-settings" -Command ([ordered]@{
        save_settings = [ordered]@{
            settings = $mutatedSettings
        }
    })

    if ($saveResponse.save_settings.result.settings.diagnostics.last_opened_page -eq $alternatePage) {
        Wait-Until -Description "config file to persist alternate page" -TimeoutSeconds $TimeoutSeconds -Condition {
            if (-not (Test-Path $configPath)) {
                return $false
            }

            $persisted = Get-Content -Raw $configPath | ConvertFrom-Json
            $persisted.diagnostics.last_opened_page -eq $alternatePage
        }
        $report.save_roundtrip_persisted = $true
    }

    $restoreResponse = Invoke-PipeCommand -RequestId "restore-settings" -Command ([ordered]@{
        save_settings = [ordered]@{
            settings = $originalSettings
        }
    })

    if ($restoreResponse.save_settings.result.settings.diagnostics.last_opened_page -eq $originalSettings.diagnostics.last_opened_page) {
        Wait-Until -Description "config file to restore original page" -TimeoutSeconds $TimeoutSeconds -Condition {
            if (-not (Test-Path $configPath)) {
                return $false
            }

            $persisted = Get-Content -Raw $configPath | ConvertFrom-Json
            $persisted.diagnostics.last_opened_page -eq $originalSettings.diagnostics.last_opened_page
        }
        $report.save_roundtrip_restored = $true
    }

    $refreshResponse = Invoke-PipeCommand -RequestId "request-refresh" -Command "request_refresh"
    if (-not $refreshResponse.request_refresh.result.accepted) {
        throw "request_refresh was not accepted"
    }

    Wait-Until -Description "last_manual_refresh_at to update" -TimeoutSeconds $TimeoutSeconds -Condition {
        $latestSettings = Invoke-PipeCommand -RequestId "get-settings-after-refresh" -Command "get_settings"
        $latestTimestamp = $latestSettings.get_settings.settings.diagnostics.last_manual_refresh_at
        if ($latestTimestamp -and $latestTimestamp -ne $report.original_last_manual_refresh_at) {
            $script:latestRefreshTimestamp = $latestTimestamp
            return $true
        }

        return $false
    }

    $report.updated_last_manual_refresh_at = $script:latestRefreshTimestamp
    $report.refresh_updated_timestamp = $true

    Send-HostClose
    Wait-Until -Description "host process to exit" -TimeoutSeconds $TimeoutSeconds -Condition {
        $startedHost.HasExited
    }
    $report.host_exit_code = $startedHost.ExitCode
    $startedHost = $null
}
finally {
    if ($startedHost -and -not $startedHost.HasExited) {
        Stop-Process -Id $startedHost.Id -Force -ErrorAction SilentlyContinue
    }

    $env:CC_TRAFFIC_LIGHT_SETTINGS_HOST = $oldHostSetting
}

$report | ConvertTo-Json -Depth 6 | Set-Content -Path $reportPath -Encoding UTF8
$report | ConvertTo-Json -Depth 6

if (-not ($report.snapshot_read -and $report.settings_read -and $report.snapshot_matches_state_file -and $report.refresh_updated_timestamp -and $report.save_roundtrip_persisted -and $report.save_roundtrip_restored -and $report.host_exit_code -eq 0)) {
    exit 1
}
